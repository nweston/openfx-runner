use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use exr::prelude::{read_first_rgba_layer_from_file, write_rgba_file};
use openfx_rs::constants;
use openfx_rs::constants::ofxstatus;
use openfx_rs::strings::OfxStr;
use openfx_rs::types::*;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::{c_char, c_int, c_void, CString};
use std::fs;
use std::string::String;
use std::sync::{LazyLock, Mutex};
use std::thread;

mod commands;
use commands::*;
#[macro_use]
mod handles;
use handles::*;
mod suite_impls;

/// An integer frame time
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameNumber(u32);

impl_handle!(ImageEffectHandle, OfxImageEffectHandle, ImageEffect);
impl_handle!(ParamSetHandle, OfxParamSetHandle, ParamSet);
impl_handle!(PropertySetHandle, OfxPropertySetHandle, PropertySet);
impl_handle!(ImageClipHandle, OfxImageClipHandle, Clip);
impl_handle!(ParamHandle, OfxParamHandle, Param);

type GenericResult = Result<()>;

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
            status: ofxstatus::OK,
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

fn rect_from_dims(width: f64, height: f64) -> OfxRectD {
    OfxRectD {
        x1: 0.0,
        y1: 0.0,
        x2: width as _,
        y2: height as _,
    }
}

fn rect_to_double(r: OfxRectI) -> OfxRectD {
    OfxRectD {
        x1: r.x1 as _,
        y1: r.y1 as _,
        x2: r.x2 as _,
        y2: r.y2 as _,
    }
}

fn rect_to_int(r: OfxRectD) -> OfxRectI {
    OfxRectI {
        x1: r.x1 as _,
        y1: r.y1 as _,
        x2: r.x2 as _,
        y2: r.y2 as _,
    }
}

fn crop(a: OfxRectI, b: OfxRectI) -> OfxRectI {
    OfxRectI {
        x1: max(a.x1, b.x1),
        y1: max(a.y1, b.y1),
        x2: min(a.x2, b.x2),
        y2: min(a.y2, b.y2),
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
            &props
                .get_type::<CString>(constants::ParamPropType, 0)
                .unwrap(),
        ) {
            constants::ParamTypeBoolean => Self::Boolean(
                props
                    .get_type::<bool>(constants::ParamPropDefault, 0)
                    .unwrap_or(false),
            ),
            constants::ParamTypeChoice => Self::Choice(
                props
                    .get_type::<i32>(constants::ParamPropDefault, 0)
                    .unwrap_or(0) as usize,
            ),
            constants::ParamTypeCustom => Self::Custom(
                props
                    .get_type::<CString>(constants::ParamPropDefault, 0)
                    .unwrap_or_else(|| CString::new("".to_string()).unwrap()),
            ),
            constants::ParamTypeDouble => Self::Double(
                props
                    .get_type::<f64>(constants::ParamPropDefault, 0)
                    .unwrap_or(0.0),
            ),
            constants::ParamTypeDouble2D => Self::Double2D(
                props
                    .get_type::<f64>(constants::ParamPropDefault, 0)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 1)
                    .unwrap_or(0.0),
            ),
            constants::ParamTypeDouble3D => Self::Double3D(
                props
                    .get_type::<f64>(constants::ParamPropDefault, 0)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 1)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 2)
                    .unwrap_or(0.0),
            ),
            constants::ParamTypeGroup => Self::Group,
            constants::ParamTypeInteger => Self::Integer(
                props
                    .get_type::<i32>(constants::ParamPropDefault, 0)
                    .unwrap_or(0),
            ),
            constants::ParamTypeInteger2D => Self::Integer2D(
                props
                    .get_type::<i32>(constants::ParamPropDefault, 0)
                    .unwrap_or(0),
                props
                    .get_type::<i32>(constants::ParamPropDefault, 1)
                    .unwrap_or(0),
            ),
            constants::ParamTypeInteger3D => Self::Integer3D(
                props
                    .get_type::<i32>(constants::ParamPropDefault, 0)
                    .unwrap_or(0),
                props
                    .get_type::<i32>(constants::ParamPropDefault, 1)
                    .unwrap_or(0),
                props
                    .get_type::<i32>(constants::ParamPropDefault, 2)
                    .unwrap_or(0),
            ),
            constants::ParamTypePage => Self::Page,
            constants::ParamTypeParametric => Self::Parametric,
            constants::ParamTypePushButton => Self::PushButton,
            constants::ParamTypeRGB => Self::Rgb(
                props
                    .get_type::<f64>(constants::ParamPropDefault, 0)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 1)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 2)
                    .unwrap_or(0.0),
            ),
            constants::ParamTypeRGBA => Self::Rgba(
                props
                    .get_type::<f64>(constants::ParamPropDefault, 0)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 1)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 2)
                    .unwrap_or(0.0),
                props
                    .get_type::<f64>(constants::ParamPropDefault, 3)
                    .unwrap_or(0.0),
            ),
            constants::ParamTypeString => Self::String(
                props
                    .get_type::<CString>(constants::ParamPropDefault, 0)
                    .unwrap_or_else(|| CString::new("".to_string()).unwrap()),
            ),
            s => panic!("Unknown param type: {}", s),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Param {
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
pub struct ParamSet {
    properties: Object<PropertySet>,
    descriptors: Vec<Object<PropertySet>>,
    params: HashMap<String, Object<Param>>,
}

impl ParamSet {
    fn create_param(&mut self, kind: OfxStr, name: OfxStr) -> PropertySetHandle {
        let props = PropertySet::new(
            &("param_".to_string() + name.as_str()),
            &[
                (constants::PropName, name.into()),
                (constants::ParamPropType, kind.into()),
            ],
        )
        .into_object();
        self.descriptors.push(props.clone());
        props.into()
    }
}

impl Default for ParamSet {
    fn default() -> Self {
        Self {
            properties: PropertySet::new("paramSet", &[]).into_object(),
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
    bounds: OfxRectI,
    pixels: Vec<Pixel>,
    stride: usize,
    properties: Object<PropertySet>,
}

impl Image {
    fn new(name: &str, bounds: &OfxRectI, mut pixels: Vec<Pixel>, stride: usize) -> Self {
        let properties = PropertySet::new(
            &format!("{}_image", name),
            &[
                (constants::PropType, constants::TypeImage.into()),
                (
                    constants::ImageEffectPropPixelDepth,
                    constants::BitDepthFloat.into(),
                ),
                (
                    constants::ImageEffectPropComponents,
                    constants::ImageComponentRGBA.into(),
                ),
                (
                    constants::ImageEffectPropPreMultiplication,
                    constants::ImagePreMultiplied.into(),
                ),
                (constants::ImageEffectPropRenderScale, [1.0, 1.0].into()),
                (constants::ImagePropPixelAspectRatio, (1.0).into()),
                (
                    constants::ImagePropData,
                    (pixels.as_mut_ptr() as *mut c_void).into(),
                ),
                (constants::ImagePropBounds, bounds.into()),
                (constants::ImagePropRegionOfDefinition, bounds.into()),
                (
                    constants::ImagePropRowBytes,
                    (stride * std::mem::size_of::<Pixel>()).into(),
                ),
                (constants::ImagePropField, constants::ImageFieldNone.into()),
            ],
        )
        .into_object();
        Self {
            bounds: *bounds,
            pixels,
            stride,
            properties,
        }
    }

    fn empty(name: &str, bounds: &OfxRectI, rowbytes: Option<usize>) -> Self {
        let stride = get_image_stride(bounds.width(), rowbytes);
        let mut pixels = Vec::new();
        pixels.resize(stride * bounds.height(), Pixel::zero());
        Self::new(name, bounds, pixels, stride)
    }

    // Adjust bounds and data pointer so image appears cropped to
    // given bounds, without changing the underlying pixel data.
    fn crop(&self, bounds: &OfxRectI) {
        // Clamp bounds to actual image dimensions
        let bounds = OfxRectI {
            x1: max(bounds.x1, self.bounds.x1),
            x2: min(bounds.x2, self.bounds.x2),
            y1: max(bounds.y1, self.bounds.y1),
            y2: min(bounds.y2, self.bounds.y2),
        };

        let offset = self.bounds.width() as isize * (bounds.y1 - self.bounds.y1) as isize
            + (bounds.x1 - self.bounds.x1) as isize;
        let data = unsafe {
            PropertyValue::Pointer(Addr(self.pixels.as_ptr().offset(offset) as _))
        };

        let mut props = self.properties.lock();
        props
            .values
            .insert(constants::ImagePropBounds.to_string(), (&bounds).into());
        props.set(constants::ImagePropData.as_str(), 0, data)
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
            ClipImages::Static(image) => Some(image),
            ClipImages::Sequence(m) => m.get(&frame),
            ClipImages::NoImage => None,
        }
    }
}

#[derive(Debug)]
pub struct Clip {
    name: String,
    properties: Object<PropertySet>,
    images: ClipImages,
    region_of_definition: Option<OfxRectD>,
}

// Images which have been passed to a plugin via clipGetImage, and not
// yet released
static CLIP_IMAGES: Mutex<Vec<Object<PropertySet>>> = Mutex::new(Vec::new());

impl Clip {
    fn set_image(&mut self, image: Image) {
        self.region_of_definition = Some(OfxRectD {
            x1: 0.0,
            y1: 0.0,
            x2: image.bounds.width() as f64,
            y2: image.bounds.height() as f64,
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

    fn get_image_handle_at_time(&self, time: OfxTime) -> Option<PropertySetHandle> {
        // clipGetImage is supposed to return a unique handle for each
        // call, which must be released by the plugin. Since our
        // handles are pointers to the underlying objects, we must
        // clone the image properties to get a new handle.
        self.images.image_at_time(time).map(|image| {
            let props = image.properties.clone();
            //  Give each clone a unique name for debugging
            props.lock().name = format!("{} image at {:?}", self.name, time);
            let handle = props.to_handle();
            CLIP_IMAGES.lock().unwrap().push(props);

            handle
        })
    }

    fn release_image_handle(handle: PropertySetHandle) {
        // Find the image corresponding to this handle and remove it
        // from the active list. It's an error to call this with an
        // image handle which isn't in use.
        let mut images = CLIP_IMAGES.lock().unwrap();
        if let Some(i) = images.iter().position(|item| item.to_handle() == handle) {
            images.remove(i);
        } else {
            panic!("Image handle {:?} is not in use", handle);
        }
    }

    /// Panic if any image handles are still in use. Don't call this
    /// when any renders are in progress.
    fn check_for_unreleased_images() {
        let images = CLIP_IMAGES.lock().unwrap();
        if images.is_empty() {
            return;
        }
        panic!(
            "Some images were not released: {:?}",
            images
                .iter()
                .map(|img| img.lock().name.clone())
                .collect::<Vec<_>>()
        );
    }
}

impl Clone for Clip {
    fn clone(&self) -> Self {
        // Deep copy the properties
        Self {
            name: self.name.clone(),
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
                name: name.to_string(),
                properties: PropertySet::new(
                    &format!("clip_{}", name),
                    &[
                        (
                            constants::ImageEffectPropPixelDepth,
                            constants::BitDepthFloat.into(),
                        ),
                        (
                            constants::ImageEffectPropComponents,
                            constants::ImageComponentRGBA.into(),
                        ),
                        (constants::ImageEffectPropFrameRate, (24.0).into()),
                        (constants::ImagePropPixelAspectRatio, (1.0).into()),
                        (constants::ImageEffectPropFrameRange, [0.0, 1.0].into()),
                        (constants::ImageClipPropConnected, 1.into()),
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
            properties: PropertySet::new("ImageEffect", &[]).into_object(),
            param_set: Default::default(),
            clips: Default::default(),
            message_suite_responses: vec![ofxstatus::ReplyYes, ofxstatus::ReplyNo], // Default::default(),
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
    set_host: unsafe extern "C" fn(*mut OfxHost),
    main_entry: unsafe extern "C" fn(
        *const c_char,
        *const c_void,
        openfx_rs::types::OfxPropertySetHandle,
        openfx_rs::types::OfxPropertySetHandle,
    ) -> openfx_sys::OfxStatus,
}

impl Plugin {
    fn call_action(
        &self,
        action: OfxStr,
        handle: ImageEffectHandle,
        in_args: PropertySetHandle,
        out_args: PropertySetHandle,
    ) -> OfxStatus {
        let handle_ptr: *mut c_void = handle.into();
        unsafe {
            (self.main_entry)(
                action.as_ptr(),
                handle_ptr,
                in_args.into(),
                out_args.into(),
            )
        }
    }

    fn try_call_action(
        &self,
        action: OfxStr,
        handle: ImageEffectHandle,
        in_args: PropertySetHandle,
        out_args: PropertySetHandle,
    ) -> GenericResult {
        let stat = self.call_action(action, handle, in_args, out_args);
        if stat.succeeded() {
            Ok(())
        } else {
            bail!("{} failed: {:?}", action, stat);
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

impl From<PropertyValue> for *const c_void {
    fn from(p: PropertyValue) -> Self {
        if let PropertyValue::Pointer(Addr(val)) = p {
            val
        } else {
            panic!("Expected Pointer value, got {:?}", p);
        }
    }
}

trait FromProperty: Sized {
    fn from_property(value: &PropertyValue) -> Option<Self>;
}

impl FromProperty for *mut c_void {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::Pointer(Addr(p)) => Some(*p as _),
            _ => None,
        }
    }
}

impl FromProperty for *mut c_char {
    fn from_property(value: &PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::String(s) => Some(s.as_ptr() as _),
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
    fn new(name: &str, values: &[(OfxStr, Property)]) -> Self {
        let mut properties = HashMap::new();
        for (name, value) in values {
            properties.insert(name.as_str().into(), value.clone());
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
                status: ofxstatus::ErrUnknown,
            })
            .map(|values| values.0.as_slice())
    }

    fn get(&self, key: OfxStr, index: usize) -> Result<&PropertyValue, OfxError> {
        self.get_all(key).and_then(|values| {
            values.get(index).ok_or(OfxError {
                message: format!("Property {} bad index {} on {}", key, index, self.name),
                status: ofxstatus::ErrBadIndex,
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
                status: ofxstatus::ErrBadIndex,
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
    fn new(path: std::path::PathBuf) -> Result<Self> {
        let file = plist_path(&path);
        let plist = plist::Value::from_file(file.clone())
            .with_context(|| format!("Reading plist \"{}\"", file.display()))?;
        Ok(Self { path, plist })
    }

    fn library_path(&self) -> Result<std::path::PathBuf> {
        self.plist
            .as_dictionary()
            .ok_or(anyhow!("Malformed plist"))?
            .get("CFBundleExecutable")
            .ok_or(anyhow!("CFBundleExecutable not found in plist"))?
            .as_string()
            .ok_or(anyhow!("CFBundleExecutable is not a string"))
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

    fn load(&self) -> Result<libloading::Library> {
        Ok(unsafe { libloading::Library::new(self.library_path()?)? })
    }
}

extern "C" fn fetch_suite(
    _host: openfx_rs::types::OfxPropertySetHandle,
    name: *const c_char,
    version: c_int,
) -> *const c_void {
    let suite = OfxStr::from_ptr(name);
    #[allow(non_upper_case_globals)]
    match suite {
        constants::ImageEffectSuite => {
            assert!(version == 1);
            &suite_impls::IMAGE_EFFECT_SUITE as *const _ as *const c_void
        }
        constants::PropertySuite => {
            assert!(version == 1);
            &suite_impls::PROPERTY_SUITE as *const _ as *const c_void
        }
        constants::ParameterSuite => {
            assert!(version == 1);
            &suite_impls::PARAMETER_SUITE as *const _ as *const c_void
        }
        constants::MemorySuite => {
            assert!(version == 1);
            &suite_impls::MEMORY_SUITE as *const _ as *const c_void
        }
        constants::MultiThreadSuite => {
            assert!(version == 1);
            &suite_impls::MULTI_THREAD_SUITE as *const _ as *const c_void
        }
        constants::MessageSuite => {
            assert!(version == 1);
            &suite_impls::MESSAGE_SUITE as *const _ as *const c_void
        }
        _ => {
            eprintln!("fetch_suite: {} v{} is not available", suite, version);
            std::ptr::null()
        }
    }
}

fn get_plugins(lib: &libloading::Library) -> Result<Vec<Plugin>> {
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
            if api != constants::ImageEffectPluginApi {
                bail!(
                    "Unknown API '{}' (only '{}' is supported)",
                    api,
                    constants::ImageEffectPluginApi
                );
            }

            plugins.push(Plugin {
                plugin_api: api.to_string(),
                api_version: p.apiVersion,
                plugin_identifier: OfxStr::from_ptr(p.pluginIdentifier).to_string(),
                plugin_version_major: p.pluginVersionMajor,
                plugin_version_minor: p.pluginVersionMinor,
                set_host: p.setHost.unwrap(),
                main_entry: p.mainEntry.0.unwrap(),
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
                props.get_type::<String>(constants::PropName, 0).unwrap(),
                Param::from_descriptor(&props).into_object(),
            )
        })
        .collect()
}

fn create_instance(descriptor: &ImageEffect, context: &str) -> ImageEffect {
    let clips = copy_map(&descriptor.clips);
    let properties = PropertySet::new(
        "instance",
        &[
            (constants::ImageEffectPropContext, context.into()),
            (
                constants::PluginPropFilePath,
                descriptor
                    .properties
                    .lock()
                    .values
                    .get(constants::PluginPropFilePath.as_str())
                    .unwrap()
                    .clone(),
            ),
            (constants::ImageEffectPropFrameRate, (24.0).into()),
            (constants::ImagePropPixelAspectRatio, (1.0).into()),
            (
                constants::ImageEffectInstancePropEffectDuration,
                (1.0).into(),
            ),
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
    output_rowbytes: Option<usize>,
    frame_min: u32,
    frame_limit: u32,
) {
    effect.properties.lock().values.insert(
        constants::ImageEffectPropProjectSize.to_string(),
        project_dims.clone(),
    );
    effect.properties.lock().values.insert(
        constants::ImageEffectPropProjectExtent.to_string(),
        project_dims,
    );

    effect.clips.get("Source").unwrap().lock().set_image(input);
    let mut output = effect.clips.get("Output").unwrap().lock();

    output.set_images(
        output_rect.width(),
        output_rect.height(),
        (frame_min..frame_limit)
            .map(|f| {
                (
                    FrameNumber(f),
                    Image::empty("Output", output_rect, output_rowbytes),
                )
            })
            .collect(),
    );
}

// Number of pixels per row. If rowbytes is provided, try to make
// pixel count match it, but always return at least the original
// width.
fn get_image_stride(width: usize, rowbytes: Option<usize>) -> usize {
    rowbytes
        .map(|b| max(b / std::mem::size_of::<Pixel>(), width))
        .unwrap_or(width)
}

fn read_exr(
    name: &str,
    path: &str,
    rowbytes: Option<usize>,
    origin: (i32, i32),
) -> Result<Image> {
    // Rowbytes calculation is a bit weird:
    // read_first_rgba_layer_from_file can't return a separate
    // rowbytes/stride value, so we have to return the width an
    // recalculate stride several times.

    let (width, height, pixels) = read_first_rgba_layer_from_file(
        path,
        // Construct pixel storage. We use a tuple which includes
        // width and height, so we can correctly interpret the flat
        // vector in the next step
        move |dims, _| {
            (
                dims.width(),
                dims.height(),
                vec![
                    Pixel::zero();
                    get_image_stride(dims.width(), rowbytes) * dims.height()
                ],
            )
        },
        // Fill in pixel data
        move |&mut (width, height, ref mut pixels),
              position,
              (r, g, b, a): (f32, f32, f32, f32)| {
            // Flip y and convert to flat index
            let index = (height - 1 - position.y()) * get_image_stride(width, rowbytes)
                + position.x();
            pixels[index] = Pixel {
                r: r,
                g: g,
                b: b,
                a: a,
            };
        },
    )
    .with_context(|| format!("Read EXR \"{}\"", path))?
    .layer_data
    .channel_data
    .pixels; // Get the pixel storage we constructed

    eprintln!("width: {}", width);

    let (x1, y1) = origin;
    let bounds = OfxRectI {
        x1,
        y1,
        x2: x1 + width as i32,
        y2: y1 + height as i32,
    };

    // Discard the exr image struct and build our own
    Ok(Image::new(
        name,
        &bounds,
        pixels,
        get_image_stride(width, rowbytes),
    ))
}

fn write_exr(filename: &str, image: Image) -> GenericResult {
    write_rgba_file(
        filename,
        image.bounds.width(),
        image.bounds.height(),
        |x, y| {
            // Flip y and convert to flat index
            let pixel = &image.pixels[(image.bounds.height() - 1 - y) * image.stride + x];
            (pixel.r, pixel.g, pixel.b, pixel.a)
        },
    )?;

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

// Mutable state for running commands
struct CommandState<'a> {
    host: &'a OfxHost,
    plugins: HashMap<String, LoadedPlugin>,
    instances: HashMap<String, Instance>,
}

impl<'a> CommandState<'a> {
    fn get_plugin(&self, name: &str) -> Result<&LoadedPlugin> {
        self.plugins
            .get(name)
            .ok_or(anyhow!("Plugin {} not loaded", name))
    }

    fn get_instance(&self, name: &str) -> Result<&Instance> {
        self.instances
            .get(name)
            .ok_or(anyhow!("No instance named {}", name))
    }
}

fn bundle_path(bundle_name: &str) -> String {
    #[cfg(target_os = "windows")]
    return format!(
        "C:/Program Files/Common Files/OFX/Plugins/{}.ofx.bundle",
        bundle_name
    );

    #[cfg(target_os = "linux")]
    return format!("/usr/OFX/Plugins/{}.ofx.bundle", bundle_name);

    #[cfg(target_os = "macos")]
    return format!("/Library/OFX/Plugins/{}.ofx.bundle", bundle_name);
}

fn load_bundle(bundle_name: &str) -> Result<(Bundle, libloading::Library)> {
    let path = bundle_path(bundle_name);
    let bundle = Bundle::new(path.into())
        .with_context(|| format!("Loading bundle {}", bundle_name))?;
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
    state: &mut CommandState,
) -> GenericResult {
    let (bundle, lib) = load_bundle(bundle_name)?;
    let plugin = get_plugins(&lib)?
        .into_iter()
        .find(|p| p.plugin_identifier == plugin_name)
        .ok_or(anyhow!("Plugin {} not found in bundle", plugin_name))?;
    unsafe { (plugin.set_host)((state.host as *const _) as *mut _) };
    plugin.try_call_action(
        constants::ActionLoad,
        ImageEffectHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
    )?;

    let descriptor = ImageEffect::new(plugin_name);
    plugin.try_call_action(
        constants::ActionDescribe,
        descriptor.clone().into(),
        PropertySetHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
    )?;

    state.plugins.insert(
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

fn image_effect_context_str(context: ImageEffectContext) -> OfxStr<'static> {
    match context {
        ImageEffectContext::Filter => constants::ImageEffectContextFilter,
        ImageEffectContext::General => constants::ImageEffectContextGeneral,
        ImageEffectContext::Generator => constants::ImageEffectContextGenerator,
        ImageEffectContext::Paint => constants::ImageEffectContextPaint,
        ImageEffectContext::Retimer => constants::ImageEffectContextRetimer,
        ImageEffectContext::Transition => constants::ImageEffectContextTransition,
    }
}

fn create(
    plugin_name: &str,
    instance_name: &str,
    context: ImageEffectContext,
    state: &mut CommandState,
) -> GenericResult {
    let effect = {
        let plugin = state.get_plugin(plugin_name)?;
        let descriptor = plugin.descriptor.lock();
        let values = &descriptor.properties.lock().values;
        let context_str = image_effect_context_str(context);

        if !values
            .get(constants::ImageEffectPropSupportedContexts.as_str())
            .map(|p| p.0.contains(&context_str.into()))
            .unwrap_or(false)
        {
            bail!("Filter context not supported");
        }
        if !values
            .get(constants::ImageEffectPropSupportedPixelDepths.as_str())
            .map(|p| p.0.contains(&constants::BitDepthFloat.into()))
            .unwrap_or(false)
        {
            bail!("OfxBitDepthFloat not supported");
        }

        // Descriptor for the plugin in Filter context
        let filter = ImageEffect {
            properties: PropertySet::new(
                "filter",
                &[(
                    constants::PluginPropFilePath,
                    plugin.bundle.path.to_str().unwrap().into(),
                )],
            )
            .into_object(),
            ..Default::default()
        }
        .into_object();

        let filter_inargs = PropertySet::new(
            "filter_inargs",
            &[(constants::ImageEffectPropContext, context_str.into())],
        )
        .into_object();
        #[allow(clippy::redundant_clone)]
        plugin.plugin.try_call_action(
            constants::ImageEffectActionDescribeInContext,
            filter.clone().into(),
            PropertySetHandle::from(filter_inargs.clone()),
            PropertySetHandle::from(std::ptr::null_mut()),
        )?;

        // Instance of the filter. Both instances and descriptors are
        // ImageEffect objects.
        let filter_instance: Object<ImageEffect> =
            create_instance(&filter.lock(), context_str.as_str()).into_object();

        plugin.plugin.try_call_action(
            constants::ActionCreateInstance,
            filter_instance.clone().into(),
            PropertySetHandle::from(std::ptr::null_mut()),
            PropertySetHandle::from(std::ptr::null_mut()),
        )?;
        filter_instance
    };
    state.instances.insert(
        instance_name.to_string(),
        Instance {
            plugin_name: plugin_name.to_string(),
            effect,
        },
    );
    Ok(())
}

fn get_output_rect(
    input: &Image,
    layout: Option<&RenderLayout>,
    project_rect: OfxRectD,
    instance: &Instance,
    plugin: &LoadedPlugin,
) -> Result<OfxRectI> {
    Ok(if let Some(l) = layout {
        if let Some(w) = l.render_window {
            w
        } else {
            // If layout is given but doesn't specify the render
            // window, compute it with the plugin's RoD action
            crop(
                rect_to_int(get_rod_for_instance(
                    (project_rect.x2, project_rect.y2),
                    &rect_to_double(input.bounds),
                    instance,
                    plugin,
                )?),
                rect_to_int(project_rect),
            )
        }
    } else {
        rect_to_int(project_rect)
    })
}

fn get_input_image(name: &str, input: &Input) -> Result<Image> {
    read_exr(name, &input.filename, input.rowbytes, input.origin)
}

fn render_filter(
    instance_name: &str,
    input: &Input,
    output_directory: Option<&String>,
    layout: Option<&RenderLayout>,
    frame_range: (FrameNumber, FrameNumber),
    thread_count: u32,
    state: &mut CommandState,
) -> GenericResult {
    let (FrameNumber(frame_min), FrameNumber(frame_limit)) = frame_range;
    if frame_limit <= frame_min {
        bail!(format!("Invalid frame range {frame_min}..{frame_limit}"));
    }

    let instance = state.get_instance(instance_name)?;
    let plugin = state.get_plugin(&instance.plugin_name)?;

    let input = get_input_image("input", input)?;
    let width = input.bounds.width();
    let height = input.bounds.height();

    // If no layout is given, default project dims and output to match
    // the input image
    let project_dims = layout
        .map(|l| [l.project_dims.0, l.project_dims.1])
        .unwrap_or([(width as f64), (height as f64)]);
    let project_rect = rect_from_dims(project_dims[0], project_dims[1]);

    let output_rect = get_output_rect(&input, layout, project_rect, instance, plugin)?;

    if layout.map(|l| l.crop_inputs_to_roi).unwrap_or(false) {
        let roi = get_rois_for_instance(
            (project_dims[0], project_dims[1]),
            &rect_to_double(output_rect),
            instance,
            plugin,
        )?;
        input.crop(&rect_to_int(roi));
    }

    create_images(
        &mut instance.effect.lock(),
        input,
        project_dims.into(),
        &output_rect,
        layout.and_then(|l| l.rowbytes),
        frame_min,
        frame_limit,
    );

    let render_range = move |start, limit| -> GenericResult {
        for frame in start..limit {
            let render_inargs = PropertySet::new(
                "render_inargs",
                &[
                    (constants::PropTime, (frame as f64).into()),
                    (
                        constants::ImageEffectPropFieldToRender,
                        constants::ImageFieldNone.into(),
                    ),
                    (
                        constants::ImageEffectPropRenderWindow,
                        (&output_rect).into(),
                    ),
                    (constants::ImageEffectPropRenderScale, [1.0, 1.0].into()),
                    (
                        constants::ImageEffectPropSequentialRenderStatus,
                        false.into(),
                    ),
                    (
                        constants::ImageEffectPropInteractiveRenderStatus,
                        false.into(),
                    ),
                    (constants::ImageEffectPropRenderQualityDraft, false.into()),
                ],
            )
            .into_object();

            #[allow(clippy::redundant_clone)]
            plugin.plugin.try_call_action(
                constants::ImageEffectActionRender,
                instance.effect.clone().into(),
                PropertySetHandle::from(render_inargs.clone()),
                PropertySetHandle::from(std::ptr::null_mut()),
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
                    s.spawn(move || render_range(min, limit))
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

    // Check after all renders are finished
    Clip::check_for_unreleased_images();

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
    state: &mut CommandState,
) -> Result<OfxRectD> {
    let instance = state.get_instance(instance_name)?;
    let plugin = state.get_plugin(&instance.plugin_name)?;

    get_rois_for_instance(project_extent, region_of_interest, instance, plugin)
}

fn get_rois_for_instance(
    project_extent: (f64, f64),
    region_of_interest: &OfxRectD,
    instance: &Instance,
    plugin: &LoadedPlugin,
) -> Result<OfxRectD> {
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
        &[
            (constants::PropTime, (0.0).into()),
            (constants::ImageEffectPropRenderScale, [1.0, 1.0].into()),
            (
                constants::ImageEffectPropRegionOfInterest,
                region_of_interest.into(),
            ),
            // Not mentioned in the spec, but plugins appear to look
            // for them in practice
            (
                constants::ImageEffectPropFieldToRender,
                constants::ImageFieldNone.into(),
            ),
            (
                constants::ImageEffectPropRenderWindow,
                [0, 0, width as c_int, height as c_int].into(),
            ),
        ],
    )
    .into_object();

    let outargs =
        PropertySet::new("getRoI_outargs", &[(roi_prop, region_of_interest.into())])
            .into_object();

    #[allow(clippy::redundant_clone)]
    plugin.plugin.try_call_action(
        constants::ImageEffectActionGetRegionsOfInterest,
        instance.effect.clone().into(),
        PropertySetHandle::from(inargs.clone()),
        PropertySetHandle::from(outargs.clone()),
    )?;

    let out = outargs.lock();
    Ok(out.get_rectd(roi_prop)?)
}

fn set_project_props(instance: &Instance, width: f64, height: f64) {
    let effect = &mut instance.effect.lock();
    let mut props = effect.properties.lock();
    props.values.insert(
        constants::ImageEffectPropProjectSize.to_string(),
        [width, height].into(),
    );
    props.values.insert(
        constants::ImageEffectPropProjectOffset.to_string(),
        [0.0, 0.0].into(),
    );
    props.values.insert(
        constants::ImageEffectPropProjectExtent.to_string(),
        [width, height].into(),
    );
}

// Call GetRegionOfDefinition action and return the resulting RoD
fn get_rod(
    instance_name: &str,
    project_extent: (f64, f64),
    input_rod: &OfxRectD,
    state: &mut CommandState,
) -> Result<OfxRectD> {
    let instance = state.get_instance(instance_name)?;
    let plugin = state.get_plugin(&instance.plugin_name)?;

    get_rod_for_instance(project_extent, input_rod, instance, plugin)
}

fn get_rod_for_instance(
    project_extent: (f64, f64),
    input_rod: &OfxRectD,
    instance: &Instance,
    plugin: &LoadedPlugin,
) -> Result<OfxRectD> {
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
        &[
            (constants::PropTime, (0.0).into()),
            (constants::ImageEffectPropRenderScale, [1.0, 1.0].into()),
            // Not mentioned in the spec, but plugins appear to look
            // for them in practice
            (
                constants::ImageEffectPropFieldToRender,
                constants::ImageFieldNone.into(),
            ),
            (
                constants::ImageEffectPropRenderWindow,
                [0, 0, width as c_int, height as c_int].into(),
            ),
        ],
    )
    .into_object();

    let outargs = PropertySet::new(
        "getRoD_outargs",
        &[(
            constants::ImageEffectPropRegionOfDefinition,
            input_rod.into(),
        )],
    )
    .into_object();

    #[allow(clippy::redundant_clone)]
    plugin.plugin.try_call_action(
        constants::ImageEffectActionGetRegionOfDefinition,
        instance.effect.clone().into(),
        PropertySetHandle::from(inargs.clone()),
        PropertySetHandle::from(outargs.clone()),
    )?;

    let out = outargs.lock();
    // Ok(out.get_rectd(roi_prop)?)
    Ok(out.get_rectd(constants::ImageEffectPropRegionOfDefinition)?)
}

fn set_params(
    instance_name: &str,
    values: &[(String, ParamValue)],
    call_instance_changed: bool,
    state: &mut CommandState,
) -> GenericResult {
    let instance = state.get_instance(instance_name)?;
    let plugin = state.get_plugin(&instance.plugin_name)?;

    let inargs1 = PropertySet::new(
        "begin_instance_changed",
        &[(
            constants::PropChangeReason,
            constants::ChangeUserEdited.into(),
        )],
    )
    .into_object();

    if call_instance_changed {
        plugin.plugin.try_call_action(
            constants::ActionBeginInstanceChanged,
            instance.effect.clone().into(),
            inargs1.clone().into(),
            PropertySetHandle::from(std::ptr::null_mut()),
        )?;
    }

    for (name, val) in values.iter() {
        let param = instance
            .effect
            .lock()
            .get_param(name)
            .ok_or(anyhow!("No such param: {}", name))?;
        param.lock().value = val.clone();

        if call_instance_changed {
            let inargs2 = PropertySet::new(
                "instance_changed",
                &[
                    (constants::PropType, constants::TypeParameter.into()),
                    (constants::PropName, name.as_str().into()),
                    (
                        constants::PropChangeReason,
                        constants::ChangeUserEdited.into(),
                    ),
                    (constants::PropTime, (0.0).into()),
                    (constants::ImageEffectPropRenderScale, [1.0, 1.0].into()),
                ],
            )
            .into_object();
            plugin.plugin.try_call_action(
                constants::ActionInstanceChanged,
                instance.effect.clone().into(),
                inargs2.clone().into(),
                PropertySetHandle::from(std::ptr::null_mut()),
            )?;
        }
    }

    if call_instance_changed {
        plugin.plugin.try_call_action(
            constants::ActionEndInstanceChanged,
            instance.effect.clone().into(),
            inargs1.clone().into(),
            PropertySetHandle::from(std::ptr::null_mut()),
        )?;
    }

    Ok(())
}

fn describe(
    bundle_name: &str,
    plugin_name: &str,
    state: &mut CommandState,
) -> Result<ImageEffect> {
    create_plugin(bundle_name, plugin_name, state)?;

    let plugin = state.get_plugin(plugin_name)?;
    plugin.plugin.try_call_action(
        constants::ActionDescribe,
        plugin.descriptor.clone().into(),
        PropertySetHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
    )?;

    Ok(plugin.descriptor.lock().clone())
}

fn describe_filter(
    bundle_name: &str,
    plugin_name: &str,
    state: &mut CommandState,
) -> GenericResult {
    describe(bundle_name, plugin_name, state)?;

    let plugin = state.get_plugin(plugin_name)?;

    // Descriptor for the plugin in Filter context
    let filter = ImageEffect {
        properties: PropertySet::new(
            "filter",
            &[(
                constants::PluginPropFilePath,
                plugin.bundle.path.to_str().unwrap().into(),
            )],
        )
        .into_object(),
        ..Default::default()
    }
    .into_object();

    let filter_inargs = PropertySet::new(
        "filter_inargs",
        &[(
            constants::ImageEffectPropContext,
            constants::ImageEffectContextFilter.into(),
        )],
    )
    .into_object();
    #[allow(clippy::redundant_clone)]
    plugin.plugin.try_call_action(
        constants::ImageEffectActionDescribeInContext,
        filter.clone().into(),
        PropertySetHandle::from(filter_inargs.clone()),
        PropertySetHandle::from(std::ptr::null_mut()),
    )?;

    println!("{}", serde_json::to_string(&*filter.lock())?);

    Ok(())
}

fn configure_message_suite_responses(
    instance_name: &str,
    responses: &[MessageSuiteResponses],
    state: &mut CommandState,
) -> GenericResult {
    let instance = state.get_instance(instance_name)?;
    use MessageSuiteResponses::*;
    instance.effect.lock().message_suite_responses = responses
        .iter()
        .rev()
        .map(|r| match r {
            OK => ofxstatus::OK,
            Yes => ofxstatus::ReplyYes,
            No => ofxstatus::ReplyNo,
            Failed => ofxstatus::Failed,
        })
        .collect::<Vec<_>>();
    Ok(())
}

fn set_host_properties(
    props: &HashMap<String, Vec<commands::PropertyValue>>,
    state: &mut CommandState,
) {
    state.host.host.with_object(|host_properties| {
        props.iter().for_each(|(name, value)| {
            host_properties.values.insert(
                name.clone(),
                Property(
                    value
                        .iter()
                        .map(|v| match v {
                            commands::PropertyValue::String(s) => (&(**s)).into(),
                            commands::PropertyValue::Double(d) => (*d).into(),
                            commands::PropertyValue::Int(i) => (*i).into(),
                        })
                        .collect(),
                ),
            );
        })
    });
}

fn print_params(instance_name: &str, state: &mut CommandState) -> GenericResult {
    let instance = state.get_instance(instance_name)?;
    println!(
        "{}",
        serde_json::to_string(&*instance.effect.lock().param_set.lock())?
    );
    Ok(())
}

fn destroy_instance(instance_name: &str, state: &mut CommandState) -> GenericResult {
    let instance = state.get_instance(instance_name)?;
    let plugin = state.get_plugin(&instance.plugin_name)?;
    plugin.plugin.try_call_action(
        constants::ActionDestroyInstance,
        instance.effect.clone().into(),
        PropertySetHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
    )?;
    state.instances.remove(instance_name);
    Ok(())
}

fn unload_plugin(plugin_name: &str, state: &mut CommandState) -> GenericResult {
    let plugin = state.get_plugin(plugin_name)?;
    plugin.plugin.try_call_action(
        constants::ActionUnload,
        ImageEffectHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
        PropertySetHandle::from(std::ptr::null_mut()),
    )?;
    state.plugins.remove(plugin_name);
    Ok(())
}

fn process_command(command: &Command, state: &mut CommandState) -> GenericResult {
    use commands::Command::*;

    match command {
        CreatePlugin {
            bundle_name,
            plugin_name,
        } => create_plugin(bundle_name, plugin_name, state).context("CreatePlugin"),
        CreateInstance {
            plugin_name,
            instance_name,
            context,
        } => create(plugin_name, instance_name, *context, state).context("CreateFilter"),
        RenderFilter {
            instance_name,
            input,
            output_directory,
            layout,
            frame_range,
            thread_count,
        } => render_filter(
            instance_name,
            input,
            output_directory.as_ref(),
            layout.as_ref(),
            *frame_range,
            *thread_count,
            state,
        )
        .context("RenderFilter"),
        PrintParams { instance_name } => {
            print_params(instance_name, state).context("PrintParams")
        }
        DestroyInstance { instance_name } => {
            destroy_instance(instance_name, state).context("DestroyInstance")
        }
        UnloadPlugin { plugin_name } => {
            unload_plugin(plugin_name, state).context("UnloadPlugin")
        }
        SetParams {
            instance_name,
            values,
            call_instance_changed,
        } => set_params(instance_name, values, *call_instance_changed, state)
            .context("SetParams"),
        ListPlugins { bundle_name } => list_plugins(bundle_name).context("ListPlugins"),
        Describe {
            bundle_name,
            plugin_name,
        } => {
            let effect = describe(bundle_name, plugin_name, state).context("Describe")?;
            println!("{}", serde_json::to_string(&*effect.properties.lock())?);
            Ok(())
        }
        DescribeFilter {
            bundle_name,
            plugin_name,
        } => describe_filter(bundle_name, plugin_name, state).context("DescribeFilter"),
        PrintRoIs {
            instance_name,
            region_of_interest,
            project_extent,
        } => {
            let roi = get_rois(instance_name, *project_extent, region_of_interest, state)
                .context("PrintRoIs")?;
            println!("{}", serde_json::to_string(&roi)?);
            Ok(())
        }
        PrintRoD {
            instance_name,
            input_rod,
            project_extent,
        } => {
            let rod = get_rod(instance_name, *project_extent, input_rod, state)
                .context("PrintRoD")?;
            println!("{}", serde_json::to_string(&rod)?);
            Ok(())
        }
        ConfigureMessageSuiteResponses {
            instance_name,
            responses,
        } => configure_message_suite_responses(instance_name, responses, state)
            .context("ConfigureMessageSuiteResponses"),
        SetHostProperties { props } => {
            set_host_properties(props, state);
            Ok(())
        }
    }
}

fn read_commands(path: &str) -> Result<Vec<Command>> {
    fs::read_to_string(path)
        .with_context(|| format!("Reading file {}", path))
        .and_then(|s| serde_json::from_str(&s).with_context(|| "Error parsing JSON"))
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
        &[
            (constants::PropName, "openfx-driver".into()),
            (constants::PropLabel, "OpenFX Driver".into()),
            (constants::PropVersion, version.into()),
            (constants::PropVersionLabel, VERSION_NAME.into()),
            (constants::PropAPIVersion, [1, 4].into()),
            (constants::ImageEffectHostPropIsBackground, false.into()),
            (constants::ImageEffectPropSupportsOverlays, false.into()),
            (
                constants::ImageEffectPropSupportsMultiResolution,
                false.into(),
            ),
            (constants::ImageEffectPropSupportsTiles, false.into()),
            (constants::ImageEffectPropTemporalClipAccess, false.into()),
            (
                constants::ImageEffectPropSupportsMultipleClipDepths,
                false.into(),
            ),
            (
                constants::ImageEffectPropSupportsMultipleClipPARs,
                false.into(),
            ),
            (constants::ImageEffectPropSetableFrameRate, false.into()),
            (constants::ImageEffectPropSetableFielding, false.into()),
            (
                constants::ImageEffectInstancePropSequentialRender,
                false.into(),
            ),
            (
                constants::ParamHostPropSupportsStringAnimation,
                false.into(),
            ),
            (constants::ParamHostPropSupportsCustomInteract, false.into()),
            (
                constants::ParamHostPropSupportsChoiceAnimation,
                false.into(),
            ),
            (
                constants::ParamHostPropSupportsStrChoiceAnimation,
                false.into(),
            ),
            (
                constants::ParamHostPropSupportsBooleanAnimation,
                false.into(),
            ),
            (
                constants::ParamHostPropSupportsCustomAnimation,
                false.into(),
            ),
            (
                constants::ParamHostPropSupportsParametricAnimation,
                false.into(),
            ),
            // Resolve GPU extensions weirdly use "false"/"true" strings
            (
                constants::ImageEffectPropOpenCLRenderSupported,
                "false".into(),
            ),
            (
                constants::ImageEffectPropCudaRenderSupported,
                "false".into(),
            ),
            (
                constants::ImageEffectPropCudaStreamSupported,
                "false".into(),
            ),
            (
                constants::ImageEffectPropMetalRenderSupported,
                "false".into(),
            ),
            (constants::ImageEffectPropRenderQualityDraft, false.into()),
            (constants::ParamHostPropMaxParameters, (-1).into()),
            (constants::ParamHostPropMaxPages, 0.into()),
            (constants::ParamHostPropPageRowColumnCount, [0, 0].into()),
            (
                constants::ImageEffectPropSupportedComponents,
                constants::ImageComponentRGBA.into(),
            ),
            (
                constants::ImageEffectPropSupportedContexts,
                constants::ImageEffectContextFilter.into(),
            ),
            (
                constants::ImageEffectPropSupportedPixelDepths,
                constants::BitDepthFloat.into(),
            ),
        ],
    )
    .into_object();
    // Clippy complains here, but we need to keep the original
    // host_props alive or it will be deallocated while a handle to it
    // still exists.
    #[allow(clippy::redundant_clone)]
    let host = OfxHost {
        host: host_props.clone().to_handle().into(),
        fetchSuite: Some(fetch_suite),
    };

    let mut state = CommandState {
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
                eprintln!("{:?}", e);
                std::process::exit(64);
            }),
    };

    for ref c in commands {
        if let Err(e) = process_command(c, &mut state) {
            eprintln!("Error running command: {:?}", e);
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
            "Reading plist \"test/Empty.ofx.bundle/Contents/Info.plist\""
        );
    }

    #[test]
    fn unparseable_plist() {
        assert_eq!(
            Bundle::new("test/Unparseable.ofx.bundle".into())
                .unwrap_err()
                .to_string(),
            "Reading plist \"test/Unparseable.ofx.bundle/Contents/Info.plist\""
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
