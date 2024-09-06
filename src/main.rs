use once_cell::sync::Lazy;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::{c_char, c_int, c_void, CString};
use std::fs;
use std::string::String;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

mod commands;
use commands::*;
pub mod constants;
use constants::actions::*;
use constants::host::*;
use constants::image_effect::*;
use constants::misc::*;
use constants::param::*;
use constants::properties::*;
use constants::suites::*;
mod suite_impls;
mod suites;
mod types;
use clap::{Parser, Subcommand};
use exr::prelude::{read_first_rgba_layer_from_file, write_rgba_file};
use types::*;
mod strings;
use strings::OfxStr;

/// An integer frame time
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameNumber(u32);

/// Holder for objects which can cross the API boundary.
///
/// Essentially an Arc<Mutex<T>> with some convenience
/// features.
#[derive(Default)]
struct Object<T>(Arc<Mutex<T>>);

impl<T> Object<T> {
    fn lock(&self) -> MutexGuard<'_, T> {
        // Locking should never fail since the app is single-threaded
        // for now, so just unwrap.
        self.0.lock().unwrap()
    }
}

impl<T> Clone for Object<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Serialize> Serialize for Object<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.lock().serialize(serializer)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Object<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(v) = self.0.try_lock() {
            write!(f, "{:?}", v)
        } else {
            write!(f, "Object([locked]{:?})", self.0)
        }
    }
}

trait IntoObject: Sized {
    fn into_object(self) -> Object<Self> {
        Object(Arc::new(Mutex::new(self)))
    }
}

// ========= Handles =========

/// Keep track of valid handles for a single type.
///
/// Handles are defined in the OFX API as void pointers to opaque
/// objects controlled by the host. Plugins can only access the
/// contents through API functions.
///
/// Here, objects which can be referred to by a handle are stored in
/// an Object<T>. A handle stores the address of the underlying object
/// (which won't move because it's boxed by the Object
/// wrapper). However, to preserve safety, handles are never actually
/// dereferenced. Instead, the HandleManager maintains of map of
/// handles, and Weak pointers to the underlying object. This has
/// several benefits: - Avoids unsafe code - Invalid handles are
/// detected because they don't exist in the map - Handles to dead
/// objects are detected by the Weak pointer
struct HandleManager<T, H> {
    handle_to_ptr: HashMap<H, Weak<Mutex<T>>>,
}

impl<T, H> HandleManager<T, H>
where
    H: From<*mut c_void> + Eq + std::hash::Hash + Copy,
{
    fn new() -> Self {
        HandleManager {
            handle_to_ptr: HashMap::new(),
        }
    }

    /// Create a handle for an object.
    fn get_handle(&mut self, obj: Object<T>) -> H {
        let handle: H = (Arc::as_ptr(&obj.0) as *mut c_void).into();
        self.handle_to_ptr.insert(handle, Arc::downgrade(&obj.0));
        handle
    }
}

/// A trait for handles to OFX objects.
///
/// Provides methods to access the underlying objects referred to by a
/// handle.
trait Handle: Sized + Eq + std::hash::Hash + std::fmt::Debug + 'static {
    type Object;
    fn handle_manager() -> &'static Lazy<Mutex<HandleManager<Self::Object, Self>>>;

    /// Get the underlying object of a handle.
    ///
    /// Panics if the handle is invalid or points to a deallocated
    /// object (these are errors in the plugin and if they occur we
    /// can't reasonably recover, so it's best to fail immediately
    /// with the option of backtrace).
    fn as_arc(&self) -> Object<Self::Object> {
        if let Some(weak) = Self::handle_manager()
            .lock()
            .unwrap()
            .handle_to_ptr
            .get(self)
        {
            Object(weak.upgrade().unwrap_or_else(|| {
                panic!("Handle {:?} points to deallocated object", self)
            }))
        } else {
            panic!("Bad handle {:?}", self);
        }
    }

    /// Run a function on the underlying object.
    ///
    /// This uses as_arc() and can panic under the same conditions.
    fn with_object<F, T>(self, callback: F) -> T
    where
        F: FnOnce(&mut Self::Object) -> T,
    {
        let mutex = self.as_arc();
        let guard = &mut mutex.lock();
        callback(guard)
    }
}

/// Implement Handle and From traits for a handle. Provides convenient
/// conversion between handles and corresponding objects.
macro_rules! impl_handle {
    ($handle_name: ident, $object_name: ident) => {
        impl Handle for $handle_name {
            type Object = $object_name;
            fn handle_manager() -> &'static Lazy<Mutex<HandleManager<Self::Object, Self>>>
            {
                static MANAGER: Lazy<Mutex<HandleManager<$object_name, $handle_name>>> =
                    Lazy::new(|| Mutex::new(HandleManager::new()));
                &MANAGER
            }
        }

        impl From<Object<$object_name>> for $handle_name {
            fn from(obj: Object<$object_name>) -> Self {
                $handle_name::handle_manager()
                    .lock()
                    .unwrap()
                    .get_handle(obj)
            }
        }
    };
}
impl_handle!(OfxImageEffectHandle, ImageEffect);
impl_handle!(OfxParamSetHandle, ParamSet);
impl_handle!(OfxPropertySetHandle, PropertySet);
impl_handle!(OfxImageClipHandle, Clip);
impl_handle!(OfxParamHandle, Param);

type GenericResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct GenericError {
    message: String,
    source: Box<dyn Error>,
}

impl std::fmt::Display for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for GenericError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.source)
    }
}

impl From<&str> for GenericError {
    fn from(s: &str) -> Self {
        Self {
            message: s.into(),
            source: s.into(),
        }
    }
}

#[derive(Debug)]
/// The result of an OFX API call.
///
/// We can use this within the Rust code as an Error object, but it
/// can also represent a successful operation (with
/// status=OfxStatus::OK or ReplyDefault).
struct OfxError {
    message: String,
    status: OfxStatus,
}

impl OfxError {
    fn ok() -> Self {
        Self {
            message: "".to_string(),
            status: OfxStatus::OK,
        }
    }

    /// Return the OFX status code. If it's an error
    fn get_status(&self, error_message_prefix: &str) -> OfxStatus {
        if self.status.failed() {
            eprintln!("{}{}", error_message_prefix, self.message);
        }
        self.status
    }
}

impl std::fmt::Display for OfxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for OfxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

trait Rect {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

impl Rect for OfxRectD {
    fn width(&self) -> usize {
        (self.x2 - self.x1) as usize
    }

    fn height(&self) -> usize {
        (self.y2 - self.y1) as usize
    }
}

impl Rect for OfxRectI {
    fn width(&self) -> usize {
        (self.x2 - self.x1) as usize
    }

    fn height(&self) -> usize {
        (self.y2 - self.y1) as usize
    }
}

impl OfxRectI {
    fn from_dims(width: usize, height: usize) -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: width as _,
            y2: height as _,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "v")]
pub enum ParamValue {
    Boolean(bool),
    Choice(usize),
    Custom(CString),
    Double(f64),
    Double2D(f64, f64),
    Double3D(f64, f64, f64),
    Group,
    Integer(i32),
    Integer2D(i32, i32),
    Integer3D(i32, i32, i32),
    Page,
    Parametric,
    PushButton,
    #[serde(rename = "RGB")]
    Rgb(f64, f64, f64),
    #[serde(rename = "RGBA")]
    Rgba(f64, f64, f64, f64),
    String(CString),
}

impl ParamValue {
    fn from_descriptor(props: &PropertySet) -> Self {
        #[allow(non_upper_case_globals)]
        match OfxStr::from_cstring(
            &props.get_type::<CString>(OfxParamPropType, 0).unwrap(),
        ) {
            OfxParamTypeBoolean => Self::Boolean(
                props
                    .get_type::<bool>(OfxParamPropDefault, 0)
                    .unwrap_or(false),
            ),
            OfxParamTypeChoice => Self::Choice(
                props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0) as usize,
            ),
            OfxParamTypeCustom => Self::Custom(
                props
                    .get_type::<CString>(OfxParamPropDefault, 0)
                    .unwrap_or_else(|| CString::new("".to_string()).unwrap()),
            ),
            OfxParamTypeDouble => {
                Self::Double(props.get_type::<f64>(OfxParamPropDefault, 0).unwrap_or(0.0))
            }
            OfxParamTypeDouble2D => Self::Double2D(
                props.get_type::<f64>(OfxParamPropDefault, 0).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 1).unwrap_or(0.0),
            ),
            OfxParamTypeDouble3D => Self::Double3D(
                props.get_type::<f64>(OfxParamPropDefault, 0).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 1).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 2).unwrap_or(0.0),
            ),
            OfxParamTypeGroup => Self::Group,
            OfxParamTypeInteger => {
                Self::Integer(props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0))
            }
            OfxParamTypeInteger2D => Self::Integer2D(
                props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0),
                props.get_type::<i32>(OfxParamPropDefault, 1).unwrap_or(0),
            ),
            OfxParamTypeInteger3D => Self::Integer3D(
                props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0),
                props.get_type::<i32>(OfxParamPropDefault, 1).unwrap_or(0),
                props.get_type::<i32>(OfxParamPropDefault, 2).unwrap_or(0),
            ),
            OfxParamTypePage => Self::Page,
            OfxParamTypeParametric => Self::Parametric,
            OfxParamTypePushButton => Self::PushButton,
            OfxParamTypeRGB => Self::Rgb(
                props.get_type::<f64>(OfxParamPropDefault, 0).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 1).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 2).unwrap_or(0.0),
            ),
            OfxParamTypeRGBA => Self::Rgba(
                props.get_type::<f64>(OfxParamPropDefault, 0).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 1).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 2).unwrap_or(0.0),
                props.get_type::<f64>(OfxParamPropDefault, 3).unwrap_or(0.0),
            ),
            OfxParamTypeString => Self::String(
                props
                    .get_type::<CString>(OfxParamPropDefault, 0)
                    .unwrap_or_else(|| CString::new("".to_string()).unwrap()),
            ),
            s => panic!("Unknown param type: {}", s),
        }
    }
}

#[derive(Debug, Serialize)]
struct Param {
    value: ParamValue,
    properties: Object<PropertySet>,
}

impl Param {
    fn from_descriptor(props: &PropertySet) -> Self {
        Self {
            value: ParamValue::from_descriptor(props),
            properties: props.clone().into_object(),
        }
    }
}
impl IntoObject for Param {}

#[derive(Debug, Serialize)]
struct ParamSet {
    properties: Object<PropertySet>,
    descriptors: Vec<Object<PropertySet>>,
    params: HashMap<String, Object<Param>>,
}

impl ParamSet {
    fn create_param(&mut self, kind: OfxStr, name: OfxStr) -> OfxPropertySetHandle {
        let props = PropertySet::new(
            &("param_".to_string() + name.as_str()),
            [(OfxPropName, name.into()), (OfxParamPropType, kind.into())],
        )
        .into_object();
        self.descriptors.push(props.clone());
        props.into()
    }
}

impl Default for ParamSet {
    fn default() -> Self {
        Self {
            properties: PropertySet::new("paramSet", []).into_object(),
            descriptors: Default::default(),
            params: Default::default(),
        }
    }
}

impl IntoObject for ParamSet {}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Pixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Pixel {
    fn zero() -> Self {
        Pixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,
    properties: Object<PropertySet>,
}

impl Image {
    fn new(name: &str, bounds: &OfxRectI, mut pixels: Vec<Pixel>) -> Self {
        let properties = PropertySet::new(
            &format!("{}_image", name),
            [
                (OfxPropType, OfxTypeImage.into()),
                (OfxImageEffectPropPixelDepth, OfxBitDepthFloat.into()),
                (OfxImageEffectPropComponents, OfxImageComponentRGBA.into()),
                (
                    OfxImageEffectPropPreMultiplication,
                    OfxImagePreMultiplied.into(),
                ),
                (OfxImageEffectPropRenderScale, [1.0, 1.0].into()),
                (OfxImagePropPixelAspectRatio, (1.0).into()),
                (
                    OfxImagePropData,
                    (pixels.as_mut_ptr() as *mut c_void).into(),
                ),
                (OfxImagePropBounds, bounds.into()),
                (OfxImagePropRegionOfDefinition, bounds.into()),
                (
                    OfxImagePropRowBytes,
                    (bounds.width() * std::mem::size_of::<Pixel>()).into(),
                ),
                (OfxImagePropField, OfxImageFieldNone.into()),
            ],
        )
        .into_object();
        Self {
            width: bounds.width(),
            height: bounds.height(),
            pixels,
            properties,
        }
    }

    fn empty(name: &str, bounds: &OfxRectI) -> Self {
        let mut pixels = Vec::new();
        pixels.resize(bounds.width() * bounds.height(), Pixel::zero());
        Self::new(name, bounds, pixels)
    }
}

#[derive(Debug, Clone)]
enum ClipImages {
    NoImage,
    Static(Image),
    Sequence(HashMap<FrameNumber, Image>),
}

impl ClipImages {
    fn image_at_time(&self, time: OfxTime) -> Option<&Image> {
        if time.0 >= 0.0 {
            self.image_at_frame(FrameNumber(time.0 as u32))
        } else {
            None
        }
    }

    fn image_at_frame(&self, frame: FrameNumber) -> Option<&Image> {
        match self {
            ClipImages::Static(ref image) => Some(image),
            ClipImages::Sequence(m) => m.get(&frame),
            ClipImages::NoImage => None,
        }
    }
}

#[derive(Debug)]
pub struct Clip {
    properties: Object<PropertySet>,
    images: ClipImages,
    region_of_definition: Option<OfxRectD>,
}

impl Clip {
    fn set_image(&mut self, image: Image) {
        self.region_of_definition = Some(OfxRectD {
            x1: 0.0,
            y1: 0.0,
            x2: image.width as f64,
            y2: image.height as f64,
        });
        self.images = ClipImages::Static(image);
    }

    fn set_images(
        &mut self,
        width: usize,
        height: usize,
        images: HashMap<FrameNumber, Image>,
    ) {
        self.region_of_definition = Some(OfxRectD {
            x1: 0.0,
            y1: 0.0,
            x2: width as f64,
            y2: height as f64,
        });
        self.images = ClipImages::Sequence(images);
    }
}

impl Clone for Clip {
    fn clone(&self) -> Self {
        // Deep copy the properties
        Self {
            properties: self.properties.lock().clone().into_object(),
            images: self.images.clone(),
            region_of_definition: self.region_of_definition,
        }
    }
}

impl Serialize for Clip {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.properties.serialize(serializer)
    }
}

impl IntoObject for Clip {}

#[derive(Clone, Debug)]
pub struct ImageEffect {
    properties: Object<PropertySet>,
    param_set: Object<ParamSet>,
    clips: HashMap<String, Object<Clip>>,
    // Stored in reverse order (next response at end of list)
    message_suite_responses: Vec<OfxStatus>,
}

impl Serialize for ImageEffect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("properties", &self.properties)?;
        map.serialize_entry("param_set", &self.param_set)?;
        map.serialize_entry("clips", &self.clips)?;
        map.end()
    }
}

impl ImageEffect {
    fn new(name: &str) -> Object<Self> {
        Self {
            properties: PropertySet {
                name: name.to_string(),
                ..Default::default()
            }
            .into_object(),
            ..Default::default()
        }
        .into_object()
    }

    fn create_clip(&mut self, name: OfxStr) -> Object<Clip> {
        self.clips.insert(
            name.to_string(),
            Clip {
                properties: PropertySet::new(
                    &format!("clip_{}", name),
                    [
                        (OfxImageEffectPropPixelDepth, OfxBitDepthFloat.into()),
                        (OfxImageEffectPropComponents, OfxImageComponentRGBA.into()),
                        (OfxImageEffectPropFrameRate, (24.0).into()),
                        (OfxImagePropPixelAspectRatio, (1.0).into()),
                        (OfxImageEffectPropFrameRange, [0.0, 1.0].into()),
                        (OfxImageClipPropConnected, 1.into()),
                    ],
                )
                .into_object(),
                images: ClipImages::NoImage,
                region_of_definition: None,
            }
            .into_object(),
        );
        self.clips.get(name.as_str()).unwrap().clone()
    }

    fn get_param(&self, name: &str) -> Option<Object<Param>> {
        self.param_set.lock().params.get(name).cloned()
    }
}

impl Default for ImageEffect {
    fn default() -> Self {
        Self {
            properties: PropertySet::new("ImageEffect", []).into_object(),
            param_set: Default::default(),
            clips: Default::default(),
            message_suite_responses: vec![OfxStatus::ReplyYes, OfxStatus::ReplyNo], // Default::default(),
        }
    }
}

impl IntoObject for ImageEffect {}

#[derive(Debug)]
#[allow(dead_code)]
struct Plugin {
    plugin_api: String,
    api_version: i32,
    plugin_identifier: String,
    plugin_version_major: u32,
    plugin_version_minor: u32,
    set_host: extern "C" fn(*const OfxHost),
    main_entry: extern "C" fn(
        *const c_char,
        *const c_void,
        OfxPropertySetHandle,
        OfxPropertySetHandle,
    ) -> OfxStatus,
}

impl Plugin {
    fn call_action(
        &self,
        action: OfxStr,
        handle: OfxImageEffectHandle,
        in_args: OfxPropertySetHandle,
        out_args: OfxPropertySetHandle,
    ) -> OfxStatus {
        let handle_ptr: *mut c_void = handle.into();
        (self.main_entry)(action.as_ptr(), handle_ptr, in_args, out_args)
    }

    fn try_call_action(
        &self,
        action: OfxStr,
        handle: OfxImageEffectHandle,
        in_args: OfxPropertySetHandle,
        out_args: OfxPropertySetHandle,
    ) -> Result<(), GenericError> {
        let stat = self.call_action(action, handle, in_args, out_args);
        if stat.succeeded() {
            Ok(())
        } else {
            Err(format!("{} failed: {:?}", action, stat).as_str().into())
        }
    }
}

/// An opaque memory address. Used for pointer properties which are
/// never dereferenced by the host, but only pass back to the plugin.
#[derive(Clone, Debug, PartialEq)]
struct Addr(*const c_void);
unsafe impl Send for Addr {}

#[derive(Clone, PartialEq)]
enum PropertyValue {
    Pointer(Addr),
    String(CString),
    Double(f64),
    Int(c_int),
    Unset,
}

impl Serialize for PropertyValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            PropertyValue::Pointer(_) => serializer.serialize_str("<pointer>"),
            PropertyValue::String(s) => {
                serializer.serialize_str(OfxStr::from_ptr(s.as_ptr()).as_str())
            }
            PropertyValue::Double(v) => serializer.serialize_f64(*v),
            PropertyValue::Int(v) => serializer.serialize_i32(*v),
            PropertyValue::Unset => serializer.serialize_str("<unset>"),
        }
    }
}

impl std::fmt::Debug for PropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PropertyValue::Pointer(Addr(a)) => write!(f, "{:?}", a),
            PropertyValue::String(s) => write!(f, "{:?}", s),
            PropertyValue::Double(d) => write!(f, "{:?}", d),
            PropertyValue::Int(i) => write!(f, "{:?}", i),
            PropertyValue::Unset => write!(f, "Unset"),
        }
    }
}

// Basic conversions
impl From<CString> for PropertyValue {
    fn from(s: CString) -> Self {
        PropertyValue::String(s)
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(CString::new(s).unwrap())
    }
}

impl From<OfxStr<'_>> for PropertyValue {
    fn from(s: OfxStr) -> Self {
        PropertyValue::String(s.to_cstring())
    }
}

impl From<*const c_char> for PropertyValue {
    fn from(s: *const c_char) -> Self {
        OfxStr::from_ptr(s).into()
    }
}

impl From<c_int> for PropertyValue {
    fn from(i: c_int) -> Self {
        PropertyValue::Int(i)
    }
}

impl From<usize> for PropertyValue {
    fn from(i: usize) -> Self {
        PropertyValue::Int(i as c_int)
    }
}

// OFX uses integers with 0/1 value for boolean properties
impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Int(if b { 1 } else { 0 })
    }
}

impl From<f64> for PropertyValue {
    fn from(i: f64) -> Self {
        PropertyValue::Double(i)
    }
}

impl From<OfxTime> for PropertyValue {
    fn from(OfxTime(i): OfxTime) -> Self {
        PropertyValue::Double(i)
    }
}

impl From<*mut c_void> for PropertyValue {
    fn from(i: *mut c_void) -> Self {
        PropertyValue::Pointer(Addr(i))
    }
}

impl From<PropertyValue> for String {
    fn from(p: PropertyValue) -> Self {
        if let PropertyValue::String(val) = p {
            val.into_string().unwrap()
        } else {
            panic!("Expected String value, got {:?}", p);
        }
    }
}

impl From<PropertyValue> for CString {
    fn from(p: PropertyValue) -> Self {
        if let PropertyValue::String(val) = p {
            val
        } else {
            panic!("Expected String value, got {:?}", p);
        }
    }
}

impl From<PropertyValue> for bool {
    fn from(p: PropertyValue) -> Self {
        if let PropertyValue::Int(val) = p {
            val != 0
        } else {
            panic!("Expected Boolean value, got {:?}", p);
        }
    }
}

impl From<PropertyValue> for i32 {
    fn from(p: PropertyValue) -> Self {
        if let PropertyValue::Int(val) = p {
            val
        } else {
            panic!("Expected Int value, got {:?}", p);
        }
    }
}

impl From<PropertyValue> for f64 {
    fn from(p: PropertyValue) -> Self {
        if let PropertyValue::Double(val) = p {
            val
        } else {
            panic!("Expected Double value, got {:?}", p);
        }
    }
}

trait FromProperty: Sized {
    fn from_property(value: &PropertyValue) -> Option<Self>;
}

impl FromProperty for *const c_void {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::Pointer(Addr(p)) => Some(*p),
            _ => None,
        }
    }
}

impl FromProperty for *const c_char {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::String(s) => Some(s.as_ptr()),
            _ => None,
        }
    }
}

impl FromProperty for f64 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::Double(d) => Some(*d),
            _ => None,
        }
    }
}

impl FromProperty for i32 {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::Int(i) => Some(*i),
            _ => None,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize)]
struct Property(Vec<PropertyValue>);

// Make a PropertyValue from a single value
impl<A: Into<PropertyValue>> From<A> for Property {
    fn from(a: A) -> Self {
        Property([a.into()].into())
    }
}

// Make a PropertyValue from an array of values
impl<T: Copy, const S: usize> From<[T; S]> for Property
where
    PropertyValue: From<T>,
{
    fn from(a: [T; S]) -> Self {
        Property(a.into_iter().map(PropertyValue::from).collect())
    }
}

impl<T: Copy> From<Vec<T>> for Property
where
    PropertyValue: From<T>,
{
    fn from(vec: Vec<T>) -> Self {
        Property(vec.into_iter().map(PropertyValue::from).collect())
    }
}

impl From<&OfxRectD> for Property {
    fn from(r: &OfxRectD) -> Self {
        Property(
            [r.x1, r.y1, r.x2, r.y2]
                .into_iter()
                .map(PropertyValue::from)
                .collect(),
        )
    }
}

impl From<&OfxRectI> for Property {
    fn from(r: &OfxRectI) -> Self {
        Property(
            [r.x1, r.y1, r.x2, r.y2]
                .into_iter()
                .map(PropertyValue::from)
                .collect(),
        )
    }
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct PropertySet {
    name: String,
    values: HashMap<String, Property>,
}

impl PropertySet {
    fn new<const S: usize>(name: &str, values: [(OfxStr, Property); S]) -> Self {
        let mut properties = HashMap::new();
        for (name, value) in values {
            properties.insert(name.as_str().into(), value);
        }
        Self {
            name: name.to_string(),
            values: properties,
        }
    }

    fn get_all(&self, key: OfxStr) -> Result<&[PropertyValue], OfxError> {
        self.values
            .get(key.as_str())
            .ok_or_else(|| OfxError {
                message: format!("Property {} not found on {}", key, self.name),
                status: OfxStatus::ErrUnknown,
            })
            .map(|values| values.0.as_slice())
    }

    fn get(&self, key: OfxStr, index: usize) -> Result<&PropertyValue, OfxError> {
        self.get_all(key).and_then(|values| {
            values.get(index).ok_or(OfxError {
                message: format!("Property {} bad index {} on {}", key, index, self.name),
                status: OfxStatus::ErrBadIndex,
            })
        })
    }

    /// Get a value and convert to the desired type.
    ///
    /// Returns None for missing property, panics on wrong type.
    fn get_type<T>(&self, key: OfxStr, index: usize) -> Option<T>
    where
        T: Clone + From<PropertyValue>,
    {
        self.get(key, index).ok().map(|v| v.clone().into())
    }

    /// Get all values of a property and return as OfxRectD.
    fn get_rectd(&self, key: OfxStr) -> Result<OfxRectD, OfxError> {
        let values = self.get_all(key)?;
        if values.len() != 4 {
            Err(OfxError {
                message: format!(
                    "Property {} bad length {} on {}",
                    key,
                    values.len(),
                    self.name
                ),
                status: OfxStatus::ErrBadIndex,
            })
        } else {
            Ok(OfxRectD {
                x1: values[0].clone().into(),
                y1: values[1].clone().into(),
                x2: values[2].clone().into(),
                y2: values[3].clone().into(),
            })
        }
    }

    fn set(&mut self, key: &str, index: usize, value: PropertyValue) {
        let prop = self
            .values
            .entry(key.to_string())
            .or_insert(Default::default());
        if index >= prop.0.len() {
            prop.0.resize_with(index + 1, || PropertyValue::Unset)
        }
        prop.0[index] = value;
    }
}

impl IntoObject for PropertySet {}

fn plist_path(bundle_path: &std::path::Path) -> std::path::PathBuf {
    bundle_path.join("Contents/Info.plist")
}

#[derive(Debug)]
struct Bundle {
    path: std::path::PathBuf,
    plist: plist::Value,
}

impl Bundle {
    fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = plist_path(&path);
        let plist = plist::Value::from_file(file.clone()).map_err(|e| GenericError {
            message: format!("Failed reading plist \"{}\"", file.display()),
            source: e.into(),
        })?;
        Ok(Self { path, plist })
    }

    fn library_path(&self) -> Result<std::path::PathBuf, &str> {
        self.plist
            .as_dictionary()
            .ok_or("Malformed plist")?
            .get("CFBundleExecutable")
            .ok_or("CFBundleExecutable not found in plist")?
            .as_string()
            .ok_or("CFBundleExecutable is not a string")
            .map(|lib_name| {
                if cfg!(target_os = "linux") {
                    self.path.join("Contents/Linux-x86-64").join(lib_name)
                } else if cfg!(windows) {
                    self.path.join("Contents/Win64").join(lib_name)
                } else {
                    self.path.join("Contents/MacOS").join(lib_name)
                }
            })
    }

    fn load(&self) -> Result<libloading::Library, Box<dyn Error>> {
        Ok(unsafe { libloading::Library::new(self.library_path()?)? })
    }
}

extern "C" fn fetch_suite(
    _host: OfxPropertySetHandle,
    name: *const c_char,
    version: c_int,
) -> *const c_void {
    let suite = OfxStr::from_ptr(name);
    #[allow(non_upper_case_globals)]
    match suite {
        OfxImageEffectSuite => {
            assert!(version == 1);
            &suite_impls::IMAGE_EFFECT_SUITE as *const _ as *const c_void
        }
        OfxPropertySuite => {
            assert!(version == 1);
            &suite_impls::PROPERTY_SUITE as *const _ as *const c_void
        }
        OfxParameterSuite => {
            assert!(version == 1);
            &suite_impls::PARAMETER_SUITE as *const _ as *const c_void
        }
        OfxMemorySuite => {
            assert!(version == 1);
            &suite_impls::MEMORY_SUITE as *const _ as *const c_void
        }
        OfxMultiThreadSuite => {
            assert!(version == 1);
            &suite_impls::MULTI_THREAD_SUITE as *const _ as *const c_void
        }
        OfxMessageSuite => {
            assert!(version == 1);
            &suite_impls::MESSAGE_SUITE as *const _ as *const c_void
        }
        _ => {
            eprintln!("fetch_suite: {} v{} is not available", suite, version);
            std::ptr::null()
        }
    }
}

fn get_plugins(lib: &libloading::Library) -> Result<Vec<Plugin>, Box<dyn Error>> {
    let mut plugins = Vec::new();
    unsafe {
        let number_of_plugins: libloading::Symbol<unsafe extern "C" fn() -> i32> =
            lib.get(b"OfxGetNumberOfPlugins")?;
        let count = number_of_plugins();
        let get_plugin: libloading::Symbol<
            unsafe extern "C" fn(i32) -> *const OfxPlugin,
        > = lib.get(b"OfxGetPlugin")?;
        for i in 0..count {
            let p = &*get_plugin(i);
            let api = OfxStr::from_ptr(p.pluginApi);
            if api != OfxImageEffectPluginApi {
                return Err(format!(
                    "Unknown API '{}' (only '{}' is supported)",
                    api, OfxImageEffectPluginApi
                )
                .into());
            }

            plugins.push(Plugin {
                plugin_api: api.to_string(),
                api_version: p.apiVersion,
                plugin_identifier: OfxStr::from_ptr(p.pluginIdentifier).to_string(),
                plugin_version_major: p.pluginVersionMajor,
                plugin_version_minor: p.pluginVersionMinor,
                set_host: p.setHost,
                main_entry: p.mainEntry,
            })
        }
    }
    Ok(plugins)
}

fn copy_map<T>(h: &HashMap<String, Object<T>>) -> HashMap<String, Object<T>>
where
    T: Clone + IntoObject,
{
    h.iter()
        .map(|(key, val)| (key.clone(), val.lock().clone().into_object()))
        .collect()
}

fn create_params(descriptors: &[Object<PropertySet>]) -> HashMap<String, Object<Param>> {
    descriptors
        .iter()
        .map(|d| {
            let props = d.lock();
            (
                props.get_type::<String>(OfxPropName, 0).unwrap(),
                Param::from_descriptor(&props).into_object(),
            )
        })
        .collect()
}

fn create_instance(descriptor: &ImageEffect, context: &str) -> ImageEffect {
    // TODO: adjust clips according to context
    let clips = copy_map(&descriptor.clips);
    let properties = PropertySet::new(
        "instance",
        [
            (OfxImageEffectPropContext, context.into()),
            (
                OfxPluginPropFilePath,
                descriptor
                    .properties
                    .lock()
                    .values
                    .get(OfxPluginPropFilePath.as_str())
                    .unwrap()
                    .clone(),
            ),
            (OfxImageEffectPropFrameRate, (24.0).into()),
            (OfxImagePropPixelAspectRatio, (1.0).into()),
            (OfxImageEffectInstancePropEffectDuration, (1.0).into()),
        ],
    )
    .into_object();
    let descriptors = &descriptor.param_set.lock().descriptors;
    let param_set = ParamSet {
        properties: Default::default(),
        descriptors: descriptors.clone(),
        params: create_params(descriptors),
    }
    .into_object();
    ImageEffect {
        properties,
        param_set,
        clips,
        ..Default::default()
    }
}

fn create_images(
    effect: &mut ImageEffect,
    input: Image,
    project_dims: Property,
    output_rect: &OfxRectI,
    frame_min: u32,
    frame_limit: u32,
) {
    effect.properties.lock().values.insert(
        OfxImageEffectPropProjectSize.to_string(),
        project_dims.clone(),
    );
    effect
        .properties
        .lock()
        .values
        .insert(OfxImageEffectPropProjectExtent.to_string(), project_dims);

    effect.clips.get("Source").unwrap().lock().set_image(input);
    let mut output = effect.clips.get("Output").unwrap().lock();

    output.set_images(
        output_rect.width(),
        output_rect.height(),
        (frame_min..frame_limit)
            .map(|f| (FrameNumber(f), Image::empty("Output", output_rect)))
            .collect(),
    );
}

fn read_exr(name: &str, path: &str, origin: (i32, i32)) -> Result<Image, Box<dyn Error>> {
    let (width, height, pixels) = read_first_rgba_layer_from_file(
        path,
        // Construct pixel storage. We use a tuple which includes
        // width and height, so we can correctly interpret the flat
        // vector in the next step
        |dims, _| {
            (
                dims.width(),
                dims.height(),
                vec![Pixel::zero(); dims.width() * dims.height()],
            )
        },
        // Fill in pixel data
        |&mut (width, height, ref mut pixels),
         position,
         (r, g, b, a): (f32, f32, f32, f32)| {
            // Flip y and convert to flat index
            let index = (height - 1 - position.y()) * width + position.x();
            pixels[index] = Pixel {
                r: r,
                g: g,
                b: b,
                a: a,
            };
        },
    )?
    .layer_data
    .channel_data
    .pixels; // Get the pixel storage we constructed

    let (x1, y1) = origin;
    let bounds = OfxRectI {
        x1,
        y1,
        x2: x1 + width as i32,
        y2: y1 + height as i32,
    };

    // Discard the exr image struct and build our own
    Ok(Image::new(name, &bounds, pixels))
}

fn write_exr(filename: &str, image: Image) -> GenericResult {
    write_rgba_file(filename, image.width, image.height, |x, y| {
        // Flip y and convert to flat index
        let pixel = &image.pixels[(image.height - 1 - y) * image.width + x];
        (pixel.r, pixel.g, pixel.b, pixel.a)
    })?;

    Ok(())
}

struct LoadedPlugin {
    bundle: Bundle,
    plugin: Plugin,
    descriptor: Object<ImageEffect>,
    // Lib is stored here to keep it loaded, but we never read it
    #[allow(dead_code)]
    lib: libloading::Library,
}

struct Instance {
    plugin_name: String,
    effect: Object<ImageEffect>,
}

struct CommandContext<'a> {
    host: &'a OfxHost,
    plugins: HashMap<String, LoadedPlugin>,
    instances: HashMap<String, Instance>,
}

impl<'a> CommandContext<'a> {
    fn get_plugin(&self, name: &str) -> Result<&LoadedPlugin, GenericError> {
        self.plugins
            .get(name)
            .ok_or(format!("Plugin {} not loaded", name).as_str().into())
    }

    fn get_instance(&self, name: &str) -> Result<&Instance, GenericError> {
        self.instances
            .get(name)
            .ok_or(format!("No instance named {}", name).as_str().into())
    }
}

fn load_bundle(
    bundle_name: &str,
) -> Result<(Bundle, libloading::Library), Box<dyn Error>> {
    let path = format!("/usr/OFX/Plugins/{}.ofx.bundle", bundle_name);
    let bundle = Bundle::new(path.into()).map_err(|e| GenericError {
        message: format!("Error loading bundle {}", bundle_name),
        source: e,
    })?;
    let lib = bundle.load()?;
    Ok((bundle, lib))
}

fn list_plugins(bundle_name: &str) -> GenericResult {
    let (_, lib) = load_bundle(bundle_name)?;
    for (i, p) in get_plugins(&lib)?.into_iter().enumerate() {
        println!(
            "{}: {}, v{}.{}",
            i, p.plugin_identifier, p.plugin_version_major, p.plugin_version_minor
        );
    }
    Ok(())
}

fn create_plugin(
    bundle_name: &str,
    plugin_name: &str,
    context: &mut CommandContext,
) -> GenericResult {
    let (bundle, lib) = load_bundle(bundle_name)?;
    let plugin = get_plugins(&lib)?
        .into_iter()
        .find(|p| p.plugin_identifier == plugin_name)
        .ok_or(format!("Plugin {} not found in bundle", plugin_name))?;
    (plugin.set_host)(context.host);
    plugin.try_call_action(
        OfxActionLoad,
        OfxImageEffectHandle::from(std::ptr::null_mut()),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
    )?;

    let descriptor = ImageEffect::new(plugin_name);
    plugin.try_call_action(
        OfxActionDescribe,
        descriptor.clone().into(),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
    )?;

    context.plugins.insert(
        plugin_name.to_string(),
        LoadedPlugin {
            bundle,
            plugin,
            descriptor,
            lib,
        },
    );
    Ok(())
}

fn create_filter(
    plugin_name: &str,
    instance_name: &str,
    context: &mut CommandContext,
) -> GenericResult {
    let effect = {
        let plugin = context.get_plugin(plugin_name)?;
        let descriptor = plugin.descriptor.lock();
        let values = &descriptor.properties.lock().values;
        if !values
            .get(OfxImageEffectPropSupportedContexts.as_str())
            .map(|p| p.0.contains(&OfxImageEffectContextFilter.into()))
            .unwrap_or(false)
        {
            return Err("Filter context not supported".into());
        }
        if !values
            .get(OfxImageEffectPropSupportedPixelDepths.as_str())
            .map(|p| p.0.contains(&OfxBitDepthFloat.into()))
            .unwrap_or(false)
        {
            return Err("OfxBitDepthFloat not supported".into());
        }

        // Descriptor for the plugin in Filter context
        let filter = ImageEffect {
            properties: PropertySet::new(
                "filter",
                [(
                    OfxPluginPropFilePath,
                    plugin.bundle.path.to_str().unwrap().into(),
                )],
            )
            .into_object(),
            ..Default::default()
        }
        .into_object();

        let filter_inargs = PropertySet::new(
            "filter_inargs",
            [(
                OfxImageEffectPropContext,
                OfxImageEffectContextFilter.into(),
            )],
        )
        .into_object();
        #[allow(clippy::redundant_clone)]
        plugin.plugin.try_call_action(
            OfxImageEffectActionDescribeInContext,
            filter.clone().into(),
            OfxPropertySetHandle::from(filter_inargs.clone()),
            OfxPropertySetHandle::from(std::ptr::null_mut()),
        )?;

        // Instance of the filter. Both instances and descriptors are
        // ImageEffect objects.
        let filter_instance: Object<ImageEffect> =
            create_instance(&filter.lock(), OfxImageEffectContextFilter.as_str())
                .into_object();

        plugin.plugin.try_call_action(
            OfxActionCreateInstance,
            filter_instance.clone().into(),
            OfxPropertySetHandle::from(std::ptr::null_mut()),
            OfxPropertySetHandle::from(std::ptr::null_mut()),
        )?;
        filter_instance
    };
    context.instances.insert(
        instance_name.to_string(),
        Instance {
            plugin_name: plugin_name.to_string(),
            effect,
        },
    );
    Ok(())
}

fn render_filter(
    instance_name: &str,
    input_file: &str,
    output_directory: Option<&String>,
    layout: Option<&RenderLayout>,
    frame_range: (FrameNumber, FrameNumber),
    thread_count: u32,
    context: &mut CommandContext,
) -> GenericResult {
    let (FrameNumber(frame_min), FrameNumber(frame_limit)) = frame_range;
    if frame_limit <= frame_min {
        return Err(format!("Invalid frame range {frame_min}..{frame_limit}").into());
    }

    let input_origin = layout.map(|l| l.input_origin).unwrap_or((0, 0));

    let instance = context.get_instance(instance_name)?;
    let plugin = context.get_plugin(&instance.plugin_name)?;

    let input = read_exr("input", input_file, input_origin)?;
    let width = input.width;
    let height = input.height;

    // If no layout is given, default project dims and output to match
    // the input image
    let project_dims = layout
        .map(|l| [l.project_dims.0, l.project_dims.1])
        .unwrap_or([(width as f64), (height as f64)]);
    let output_rect = layout
        .map(|l| l.render_window)
        .unwrap_or(OfxRectI::from_dims(width, height));

    create_images(
        &mut instance.effect.lock(),
        input,
        project_dims.into(),
        &output_rect,
        frame_min,
        frame_limit,
    );

    let render_range = move |start, limit| -> GenericResult {
        for frame in start..limit {
            let render_inargs = PropertySet::new(
                "render_inargs",
                [
                    (OfxPropTime, (frame as f64).into()),
                    (OfxImageEffectPropFieldToRender, OfxImageFieldNone.into()),
                    (OfxImageEffectPropRenderWindow, (&output_rect).into()),
                    (OfxImageEffectPropRenderScale, [1.0, 1.0].into()),
                    (OfxImageEffectPropSequentialRenderStatus, false.into()),
                    (OfxImageEffectPropInteractiveRenderStatus, false.into()),
                    (OfxImageEffectPropRenderQualityDraft, false.into()),
                ],
            )
            .into_object();

            #[allow(clippy::redundant_clone)]
            plugin.plugin.try_call_action(
                OfxImageEffectActionRender,
                instance.effect.clone().into(),
                OfxPropertySetHandle::from(render_inargs.clone()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )?;
        }
        Ok(())
    };
    if thread_count <= 1 {
        render_range(frame_min, frame_limit)?;
    } else {
        let chunk_size =
            ((frame_limit - frame_min) as f32 / thread_count as f32).ceil() as u32;

        thread::scope(|s| -> GenericResult {
            let threads = (0..thread_count)
                .map(|i| {
                    let min = i * chunk_size;
                    let limit = (min + chunk_size).min(frame_limit);
                    // If render fails, convert error to a string so
                    // we can send it across threads
                    s.spawn(move || render_range(min, limit).map_err(|e| e.to_string()))
                })
                .collect::<Vec<_>>();

            for t in threads {
                // Unwrapping the join result gives us the Result returned by
                // the closure. Propagate any error it contains.
                t.join().unwrap()?;
            }
            Ok(())
        })?
    }

    if let Some(output_directory) = output_directory {
        std::fs::create_dir_all(output_directory)?;
        for frame in frame_min..frame_limit {
            let format_width = (frame_limit.ilog10() + 1) as usize;
            write_exr(
                &format!("{output_directory}/{frame:0format_width$}.exr"),
                instance
                    .effect
                    .lock()
                    .clips
                    .get("Output")
                    .unwrap()
                    .lock()
                    .images
                    .image_at_frame(FrameNumber(frame))
                    .unwrap()
                    .clone(),
            )?;
        }
    }
    Ok(())
}

// Call GetRegionsOfInterest action, return the RoI for the Source clip
fn get_rois(
    instance_name: &str,
    project_extent: (f64, f64),
    region_of_interest: &OfxRectD,
    context: &mut CommandContext,
) -> Result<OfxRectD, Box<dyn Error>> {
    let instance = context.get_instance(instance_name)?;
    let plugin = context.get_plugin(&instance.plugin_name)?;

    let (width, height) = project_extent;

    // Set effect properties
    set_project_props(instance, width, height);
    {
        instance
            .effect
            .lock()
            .clips
            .get("Source")
            .unwrap()
            .lock()
            .region_of_definition = Some(OfxRectD {
            x1: 0.0,
            y1: 0.0,
            x2: width,
            y2: height,
        });
    }

    let roi_prop = OfxStr::from_str("OfxImageClipPropRoI_Source\0");

    let inargs = PropertySet::new(
        "getRoI_inargs",
        [
            (OfxPropTime, (0.0).into()),
            (OfxImageEffectPropRenderScale, [1.0, 1.0].into()),
            (
                OfxImageEffectPropRegionOfInterest,
                region_of_interest.into(),
            ),
            // Not mentioned in the spec, but plugins appear to look
            // for them in practice
            (OfxImageEffectPropFieldToRender, OfxImageFieldNone.into()),
            (
                OfxImageEffectPropRenderWindow,
                [0, 0, width as c_int, height as c_int].into(),
            ),
        ],
    )
    .into_object();

    let outargs =
        PropertySet::new("getRoI_outargs", [(roi_prop, region_of_interest.into())])
            .into_object();

    #[allow(clippy::redundant_clone)]
    plugin.plugin.try_call_action(
        OfxImageEffectActionGetRegionsOfInterest,
        instance.effect.clone().into(),
        OfxPropertySetHandle::from(inargs.clone()),
        OfxPropertySetHandle::from(outargs.clone()),
    )?;

    let out = outargs.lock();
    Ok(out.get_rectd(roi_prop)?)
}

fn set_project_props(instance: &Instance, width: f64, height: f64) {
    let effect = &mut instance.effect.lock();
    let mut props = effect.properties.lock();
    props.values.insert(
        OfxImageEffectPropProjectSize.to_string(),
        [width, height].into(),
    );
    props.values.insert(
        OfxImageEffectPropProjectOffset.to_string(),
        [0.0, 0.0].into(),
    );
    props.values.insert(
        OfxImageEffectPropProjectExtent.to_string(),
        [width, height].into(),
    );
}

// Call GetRegionOfDefinition action and return the resulting RoD
fn get_rod(
    instance_name: &str,
    project_extent: (f64, f64),
    input_rod: &OfxRectD,
    context: &mut CommandContext,
) -> Result<OfxRectD, Box<dyn Error>> {
    let instance = context.get_instance(instance_name)?;
    let plugin = context.get_plugin(&instance.plugin_name)?;

    let (width, height) = project_extent;

    // Set effect properties
    set_project_props(instance, width, height);
    {
        instance
            .effect
            .lock()
            .clips
            .get("Source")
            .unwrap()
            .lock()
            .region_of_definition = Some(*input_rod);
    }

    let inargs = PropertySet::new(
        "getRoD_inargs",
        [
            (OfxPropTime, (0.0).into()),
            (OfxImageEffectPropRenderScale, [1.0, 1.0].into()),
            // Not mentioned in the spec, but plugins appear to look
            // for them in practice
            (OfxImageEffectPropFieldToRender, OfxImageFieldNone.into()),
            (
                OfxImageEffectPropRenderWindow,
                [0, 0, width as c_int, height as c_int].into(),
            ),
        ],
    )
    .into_object();

    let outargs = PropertySet::new(
        "getRoD_outargs",
        [(OfxImageEffectPropRegionOfDefinition, input_rod.into())],
    )
    .into_object();

    #[allow(clippy::redundant_clone)]
    plugin.plugin.try_call_action(
        OfxImageEffectActionGetRegionOfDefinition,
        instance.effect.clone().into(),
        OfxPropertySetHandle::from(inargs.clone()),
        OfxPropertySetHandle::from(outargs.clone()),
    )?;

    let out = outargs.lock();
    // Ok(out.get_rectd(roi_prop)?)
    Ok(out.get_rectd(OfxImageEffectPropRegionOfDefinition)?)
}

fn set_params(
    instance_name: &str,
    values: &[(String, ParamValue)],
    call_instance_changed: bool,
    context: &mut CommandContext,
) -> GenericResult {
    let instance = context.get_instance(instance_name)?;
    let plugin = context.get_plugin(&instance.plugin_name)?;

    let inargs1 = PropertySet::new(
        "begin_instance_changed",
        [(OfxPropChangeReason, OfxChangeUserEdited.into())],
    )
    .into_object();

    if call_instance_changed {
        plugin.plugin.try_call_action(
            OfxActionBeginInstanceChanged,
            instance.effect.clone().into(),
            inargs1.clone().into(),
            OfxPropertySetHandle::from(std::ptr::null_mut()),
        )?;
    }

    for (name, val) in values.iter() {
        let param = instance
            .effect
            .lock()
            .get_param(name)
            .ok_or(format!("No such param: {}", name))?;
        param.lock().value = val.clone();

        if call_instance_changed {
            let inargs2 = PropertySet::new(
                "instance_changed",
                [
                    (OfxPropType, OfxTypeParameter.into()),
                    (OfxPropName, name.as_str().into()),
                    (OfxPropChangeReason, OfxChangeUserEdited.into()),
                    (OfxPropTime, (0.0).into()),
                    (OfxImageEffectPropRenderScale, [1.0, 1.0].into()),
                ],
            )
            .into_object();
            plugin.plugin.try_call_action(
                OfxActionInstanceChanged,
                instance.effect.clone().into(),
                inargs2.clone().into(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )?;
        }
    }

    if call_instance_changed {
        plugin.plugin.try_call_action(
            OfxActionEndInstanceChanged,
            instance.effect.clone().into(),
            inargs1.clone().into(),
            OfxPropertySetHandle::from(std::ptr::null_mut()),
        )?;
    }

    Ok(())
}

fn describe(
    bundle_name: &str,
    plugin_name: &str,
    context: &mut CommandContext,
) -> Result<ImageEffect, Box<dyn Error>> {
    create_plugin(bundle_name, plugin_name, context)?;

    let plugin = context.get_plugin(plugin_name)?;
    plugin.plugin.try_call_action(
        OfxActionDescribe,
        plugin.descriptor.clone().into(),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
    )?;

    Ok(plugin.descriptor.lock().clone())
}

fn describe_filter(
    bundle_name: &str,
    plugin_name: &str,
    context: &mut CommandContext,
) -> GenericResult {
    describe(bundle_name, plugin_name, context)?;

    let plugin = context.get_plugin(plugin_name)?;

    // Descriptor for the plugin in Filter context
    let filter = ImageEffect {
        properties: PropertySet::new(
            "filter",
            [(
                OfxPluginPropFilePath,
                plugin.bundle.path.to_str().unwrap().into(),
            )],
        )
        .into_object(),
        ..Default::default()
    }
    .into_object();

    let filter_inargs = PropertySet::new(
        "filter_inargs",
        [(
            OfxImageEffectPropContext,
            OfxImageEffectContextFilter.into(),
        )],
    )
    .into_object();
    #[allow(clippy::redundant_clone)]
    plugin.plugin.try_call_action(
        OfxImageEffectActionDescribeInContext,
        filter.clone().into(),
        OfxPropertySetHandle::from(filter_inargs.clone()),
        OfxPropertySetHandle::from(std::ptr::null_mut()),
    )?;

    println!("{}", serde_json::to_string(&*filter.lock())?);

    Ok(())
}

fn configure_message_suite_responses(
    instance_name: &str,
    responses: &[MessageSuiteResponses],
    context: &mut CommandContext,
) -> GenericResult {
    let instance = context.get_instance(instance_name)?;
    use MessageSuiteResponses::*;
    instance.effect.lock().message_suite_responses = responses
        .iter()
        .rev()
        .map(|r| match r {
            OK => OfxStatus::OK,
            Yes => OfxStatus::ReplyYes,
            No => OfxStatus::ReplyNo,
            Failed => OfxStatus::Failed,
        })
        .collect::<Vec<_>>();
    Ok(())
}

fn process_command(command: &Command, context: &mut CommandContext) -> GenericResult {
    use commands::Command::*;

    match command {
        CreatePlugin {
            bundle_name,
            plugin_name,
        } => create_plugin(bundle_name, plugin_name, context),
        CreateFilter {
            plugin_name,
            instance_name,
        } => create_filter(plugin_name, instance_name, context),
        RenderFilter {
            instance_name,
            input_file,
            output_directory,
            layout,
            frame_range,
            thread_count,
        } => render_filter(
            instance_name,
            input_file,
            output_directory.as_ref(),
            layout.as_ref(),
            *frame_range,
            *thread_count,
            context,
        ),
        PrintParams { instance_name } => {
            let instance = context.get_instance(instance_name)?;
            println!(
                "{}",
                serde_json::to_string(&*instance.effect.lock().param_set.lock())?
            );
            Ok(())
        }
        DestroyInstance { instance_name } => {
            let instance = context.get_instance(instance_name)?;
            let plugin = context.get_plugin(&instance.plugin_name)?;
            plugin.plugin.try_call_action(
                OfxActionDestroyInstance,
                instance.effect.clone().into(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )?;
            context.instances.remove(instance_name);
            Ok(())
        }
        UnloadPlugin { plugin_name } => {
            let plugin = context.get_plugin(plugin_name)?;
            plugin.plugin.try_call_action(
                OfxActionUnload,
                OfxImageEffectHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )?;
            context.plugins.remove(plugin_name);
            Ok(())
        }
        SetParams {
            instance_name,
            values,
            call_instance_changed,
        } => set_params(instance_name, values, *call_instance_changed, context),
        ListPlugins { bundle_name } => list_plugins(bundle_name),
        Describe {
            bundle_name,
            plugin_name,
        } => {
            let effect = describe(bundle_name, plugin_name, context)?;
            println!("{}", serde_json::to_string(&*effect.properties.lock())?);
            Ok(())
        }
        DescribeFilter {
            bundle_name,
            plugin_name,
        } => describe_filter(bundle_name, plugin_name, context),
        PrintRoIs {
            instance_name,
            region_of_interest,
            project_extent,
        } => {
            let roi =
                get_rois(instance_name, *project_extent, region_of_interest, context)?;
            println!("{}", serde_json::to_string(&roi)?);
            Ok(())
        }
        PrintRoD {
            instance_name,
            input_rod,
            project_extent,
        } => {
            let rod = get_rod(instance_name, *project_extent, input_rod, context)?;
            println!("{}", serde_json::to_string(&rod)?);
            Ok(())
        }
        ConfigureMessageSuiteResponses {
            instance_name,
            responses,
        } => {
            configure_message_suite_responses(instance_name, responses, context)?;
            Ok(())
        }
    }
}

fn read_commands(path: &str) -> Result<Vec<Command>, GenericError> {
    fs::read_to_string(path)
        .map_err(|e| GenericError {
            message: format!("Failed reading file {}", path),
            source: e.into(),
        })
        .and_then(|s| {
            serde_json::from_str(&s).map_err(|e| GenericError {
                message: "Error parsing JSON".to_string(),
                source: e.into(),
            })
        })
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand)]
enum CliCommands {
    /// List all plugins in a bundle
    List { bundle_name: String },
    /// Describe a plugin
    Describe {
        bundle_name: String,
        plugin_name: String,
    },
    /// DescribeInContext with filter context
    DescribeFilter {
        bundle_name: String,
        plugin_name: String,
    },
    /// Run commands from a JSON file
    Run { command_file: String },
}

fn main() {
    const VERSION_NAME: &str = env!("CARGO_PKG_VERSION");
    let version: Vec<_> = VERSION_NAME
        .split('.')
        .map(|s| s.parse::<c_int>().unwrap())
        .collect();
    let host_props = PropertySet::new(
        "host",
        [
            (OfxPropName, "openfx-driver".into()),
            (OfxPropLabel, "OpenFX Driver".into()),
            (OfxPropVersion, version.into()),
            (OfxPropVersionLabel, VERSION_NAME.into()),
            (OfxPropAPIVersion, [1, 4].into()),
            (OfxImageEffectHostPropIsBackground, false.into()),
            (OfxImageEffectPropSupportsOverlays, false.into()),
            (OfxImageEffectPropSupportsMultiResolution, false.into()),
            (OfxImageEffectPropSupportsTiles, false.into()),
            (OfxImageEffectPropTemporalClipAccess, false.into()),
            (OfxImageEffectPropSupportsMultipleClipDepths, false.into()),
            (OfxImageEffectPropSupportsMultipleClipPARs, false.into()),
            (OfxImageEffectPropSetableFrameRate, false.into()),
            (OfxImageEffectPropSetableFielding, false.into()),
            (OfxImageEffectInstancePropSequentialRender, false.into()),
            (OfxParamHostPropSupportsStringAnimation, false.into()),
            (OfxParamHostPropSupportsCustomInteract, false.into()),
            (OfxParamHostPropSupportsChoiceAnimation, false.into()),
            (OfxParamHostPropSupportsStrChoiceAnimation, false.into()),
            (OfxParamHostPropSupportsBooleanAnimation, false.into()),
            (OfxParamHostPropSupportsCustomAnimation, false.into()),
            (OfxParamHostPropSupportsParametricAnimation, false.into()),
            // Resolve GPU extensions weirdly use "false"/"true" strings
            (OfxImageEffectPropOpenCLRenderSupported, "false".into()),
            (OfxImageEffectPropCudaRenderSupported, "false".into()),
            (OfxImageEffectPropCudaStreamSupported, "false".into()),
            (OfxImageEffectPropMetalRenderSupported, "false".into()),
            (OfxImageEffectPropRenderQualityDraft, false.into()),
            (OfxParamHostPropMaxParameters, (-1).into()),
            (OfxParamHostPropMaxPages, 0.into()),
            (OfxParamHostPropPageRowColumnCount, [0, 0].into()),
            (
                OfxImageEffectPropSupportedComponents,
                OfxImageComponentRGBA.into(),
            ),
            (
                OfxImageEffectPropSupportedContexts,
                OfxImageEffectContextFilter.into(),
            ),
            (
                OfxImageEffectPropSupportedPixelDepths,
                OfxBitDepthFloat.into(),
            ),
        ],
    )
    .into_object();
    // Clippy complains here, but we need to keep the original
    // host_props alive or it will be deallocated while a handle to it
    // still exists.
    #[allow(clippy::redundant_clone)]
    let host = OfxHost {
        host: host_props.clone().into(),
        fetchSuite: fetch_suite,
    };

    let mut context = CommandContext {
        host: &host,
        plugins: HashMap::new(),
        instances: HashMap::new(),
    };

    let commands = match Cli::parse().command {
        // Run ListPlugins on the given bundle
        CliCommands::List { bundle_name } => vec![Command::ListPlugins {
            bundle_name: bundle_name.clone(),
        }],
        CliCommands::Describe {
            bundle_name,
            plugin_name,
        } => vec![Command::Describe {
            bundle_name: bundle_name.clone(),
            plugin_name: plugin_name.clone(),
        }],
        CliCommands::DescribeFilter {
            bundle_name,
            plugin_name,
        } => vec![Command::DescribeFilter {
            bundle_name: bundle_name.clone(),
            plugin_name: plugin_name.clone(),
        }],
        // Otherwise read commands from file
        CliCommands::Run { ref command_file } => read_commands(command_file)
            .unwrap_or_else(|e| {
                eprintln!("{}: {}", e, e.source);
                std::process::exit(64);
            }),
    };

    for ref c in commands {
        if let Err(e) = process_command(c, &mut context) {
            eprintln!("Error running command: {}", e);
            std::process::exit(-1);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    fn bundle_from_plist(name: &str) -> Bundle {
        Bundle {
            path: "fake".into(),
            plist: plist::Value::from_file("test/".to_owned() + name + ".plist").unwrap(),
        }
    }

    #[test]
    fn missing_plist() {
        assert_eq!(
            Bundle::new("test/Empty.ofx.bundle".into())
                .unwrap_err()
                .to_string(),
            "Failed reading plist \"test/Empty.ofx.bundle/Contents/Info.plist\""
        );
    }

    #[test]
    fn unparseable_plist() {
        assert_eq!(
            Bundle::new("test/Unparseable.ofx.bundle".into())
                .unwrap_err()
                .to_string(),
            "Failed reading plist \"test/Unparseable.ofx.bundle/Contents/Info.plist\""
        );
    }

    #[test]
    fn no_exe() {
        assert_eq!(
            Bundle::new("test/NoExe.ofx.bundle".into())
                .unwrap()
                .load()
                .unwrap_err()
                .to_string(),
            "test/NoExe.ofx.bundle/Contents/Linux-x86-64/test.ofx: cannot open shared object file: No such file or directory"
        );
    }

    #[test]
    fn no_executable_name() {
        assert_eq!(
            bundle_from_plist("no-exe-name")
                .load()
                .unwrap_err()
                .to_string(),
            "CFBundleExecutable not found in plist"
        );
    }

    #[test]
    fn executable_name_not_a_string() {
        assert_eq!(
            bundle_from_plist("not-a-string")
                .load()
                .unwrap_err()
                .to_string(),
            "CFBundleExecutable is not a string"
        );
    }

    #[test]
    fn plist_not_a_dict() {
        assert_eq!(
            bundle_from_plist("not-a-dict")
                .load()
                .unwrap_err()
                .to_string(),
            "Malformed plist"
        );
    }

    #[test]
    fn missing_functions() {
        let lib1 = unsafe { libloading::Library::new("test/no-functions").unwrap() };
        assert_eq!(
            get_plugins(&lib1).unwrap_err().to_string(),
            "test/no-functions: undefined symbol: OfxGetNumberOfPlugins"
        );
        let lib2 = unsafe { libloading::Library::new("test/no-getplugin").unwrap() };
        assert_eq!(
            get_plugins(&lib2).unwrap_err().to_string(),
            "test/no-getplugin: undefined symbol: OfxGetPlugin"
        );
    }
}
