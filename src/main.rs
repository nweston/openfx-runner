use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs;
use std::string::String;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

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
use openexr::prelude::*;
use types::*;

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
                panic!(
                    "OfxPropertySetHandle {:?} points to deallocated object",
                    self
                )
            }))
        } else {
            panic!("Bad OfxPropertySetHandle {:?}", self);
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "v")]
pub enum ParamValue {
    Boolean(bool),
    Choice(usize),
    Custom,
    Double(f64),
    Double2D,
    Double3D,
    Group,
    Integer(i32),
    Integer2D,
    Integer3D,
    Page,
    ParamValueParametric,
    PushButton,
    Rgb { r: f64, g: f64, b: f64 },
    Rgba,
    StrChoice,
    String(String),
}

impl ParamValue {
    fn from_descriptor(props: &PropertySet) -> Self {
        #[allow(non_upper_case_globals)]
        match props
            .get_type::<String>(OfxParamPropType, 0)
            .unwrap()
            .as_str()
        {
            OfxParamTypeBoolean => Self::Boolean(
                props
                    .get_type::<bool>(OfxParamPropDefault, 0)
                    .unwrap_or(false),
            ),
            OfxParamTypeInteger => {
                Self::Integer(props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0))
            }
            OfxParamTypeDouble => {
                Self::Double(props.get_type::<f64>(OfxParamPropDefault, 0).unwrap_or(0.0))
            }
            OfxParamTypeString => Self::String(
                props
                    .get_type::<String>(OfxParamPropDefault, 0)
                    .unwrap_or("".into()),
            ),
            OfxParamTypeChoice => Self::Choice(
                props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0) as usize,
            ),
            OfxParamTypePushButton => Self::PushButton,
            OfxParamTypePage => Self::Page,
            OfxParamTypeGroup => Self::Group,
            s => panic!("ParamValue type not implemented: {}", s),
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
struct ParamSet {
    properties: Object<PropertySet>,
    descriptors: Vec<Object<PropertySet>>,
    params: HashMap<String, Object<Param>>,
}

impl ParamSet {
    fn create_param(&mut self, kind: &str, name: &str) -> OfxPropertySetHandle {
        let props = PropertySet::new(
            &("param_".to_string() + name),
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

impl From<Rgba> for Pixel {
    fn from(p: Rgba) -> Self {
        Pixel {
            r: p.r.to_f32(),
            g: p.g.to_f32(),
            b: p.b.to_f32(),
            a: p.a.to_f32(),
        }
    }
}

impl From<Pixel> for Rgba {
    fn from(p: Pixel) -> Self {
        Rgba::from_f32(p.r, p.g, p.b, p.a)
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
    fn new(name: &str, width: usize, height: usize, mut pixels: Vec<Pixel>) -> Self {
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
                (OfxImagePropBounds, [0, 0, width, height].into()),
                (OfxImagePropRegionOfDefinition, [0, 0, width, height].into()),
                (
                    OfxImagePropRowBytes,
                    (width * std::mem::size_of::<Pixel>()).into(),
                ),
                (OfxImagePropField, OfxImageFieldNone.into()),
            ],
        )
        .into_object();
        Self {
            width,
            height,
            pixels,
            properties,
        }
    }

    fn empty(name: &str, width: usize, height: usize) -> Self {
        let mut pixels = Vec::new();
        pixels.resize(width * height, Pixel::zero());
        Self::new(name, width, height, pixels)
    }
}

#[derive(Debug)]
pub struct Clip {
    properties: Object<PropertySet>,
    image: Option<Image>,
}

impl Clone for Clip {
    fn clone(&self) -> Self {
        // Deep copy the properties
        Self {
            properties: self.properties.lock().clone().into_object(),
            image: self.image.clone(),
        }
    }
}

impl IntoObject for Clip {}

#[derive(Clone, Debug)]
pub struct ImageEffect {
    properties: Object<PropertySet>,
    param_set: Object<ParamSet>,
    clips: HashMap<String, Object<Clip>>,
}

impl ImageEffect {
    fn create_clip(&mut self, name: &str) -> Object<Clip> {
        self.clips.insert(
            name.into(),
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
                image: None,
            }
            .into_object(),
        );
        self.clips.get(name).unwrap().clone()
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
        action: &str,
        handle: OfxImageEffectHandle,
        in_args: OfxPropertySetHandle,
        out_args: OfxPropertySetHandle,
    ) -> OfxStatus {
        let handle_ptr: *mut c_void = handle.into();
        let c_action = CString::new(action).unwrap();
        (self.main_entry)(c_action.as_ptr(), handle_ptr, in_args, out_args)
    }

    fn try_call_action(
        &self,
        action: &str,
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

#[derive(Clone, Default, Debug)]
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

#[derive(Clone, Default, Debug)]
pub struct PropertySet {
    name: String,
    values: HashMap<String, Property>,
}

impl PropertySet {
    fn new<const S: usize>(name: &str, values: [(&str, Property); S]) -> Self {
        let mut properties = HashMap::new();
        for (name, value) in values {
            properties.insert(name.into(), value);
        }
        Self {
            name: name.to_string(),
            values: properties,
        }
    }

    fn get(&self, key: &str, index: usize) -> Result<&PropertyValue, OfxStatus> {
        self.values
            .get(key)
            .ok_or_else(|| {
                println!("Property {} not found on {}", key, self.name);
                OfxStatus::ErrUnknown
            })
            .and_then(|values| values.0.get(index).ok_or(OfxStatus::ErrBadIndex))
    }

    /// Get a value and convert to the desired type.
    ///
    /// Returns None for missing property, panics on wrong type.
    fn get_type<T>(&self, key: &str, index: usize) -> Option<T>
    where
        T: Clone + From<PropertyValue>,
    {
        self.get(key, index).ok().map(|v| v.clone().into())
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

fn cstr_to_string(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s).to_str().unwrap().to_string() }
}

extern "C" fn fetch_suite(
    _host: OfxPropertySetHandle,
    name: *const c_char,
    version: c_int,
) -> *const c_void {
    let suite = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    if suite == OfxImageEffectSuite {
        assert!(version == 1);
        &suite_impls::IMAGE_EFFECT_SUITE as *const _ as *const c_void
    } else if suite == OfxPropertySuite {
        assert!(version == 1);
        &suite_impls::PROPERTY_SUITE as *const _ as *const c_void
    } else if suite == OfxParameterSuite {
        assert!(version == 1);
        &suite_impls::PARAMETER_SUITE as *const _ as *const c_void
    } else if suite == OfxMemorySuite {
        assert!(version == 1);
        &suite_impls::MEMORY_SUITE as *const _ as *const c_void
    } else if suite == OfxMultiThreadSuite {
        assert!(version == 1);
        &suite_impls::MULTI_THREAD_SUITE as *const _ as *const c_void
    } else if suite == OfxMessageSuite {
        assert!(version == 1);
        &suite_impls::MESSAGE_SUITE as *const _ as *const c_void
    } else {
        println!("fetch_suite: {} v{} is not available", suite, version);
        std::ptr::null()
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
            plugins.push(Plugin {
                plugin_api: cstr_to_string(p.pluginApi),
                api_version: p.apiVersion,
                plugin_identifier: cstr_to_string(p.pluginIdentifier),
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
                    .get(OfxPluginPropFilePath)
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
    }
}

fn create_images(effect: &mut ImageEffect, input: Image) {
    let width = input.width;
    let height = input.height;
    let project_dims: Property = [(width as f64), (height as f64)].into();
    effect.properties.lock().values.insert(
        OfxImageEffectPropProjectSize.to_string(),
        project_dims.clone(),
    );
    effect
        .properties
        .lock()
        .values
        .insert(OfxImageEffectPropProjectExtent.to_string(), project_dims);

    effect.clips.get("Source").unwrap().lock().image = Some(input);
    effect.clips.get("Output").unwrap().lock().image =
        Some(Image::empty("Output", width, height));
}

fn read_exr(name: &str, path: &str) -> Result<Image, Box<dyn Error>> {
    let mut file = RgbaInputFile::new(path, 1)?;
    // Note that windows in OpenEXR are ***inclusive*** bounds, so a
    // 1920x1080 image has window [0, 0, 1919, 1079].
    let data_window: [i32; 4] = *file.header().data_window();
    let width = data_window.width() + 1;
    let height = data_window.height() + 1;
    let uwidth = width as usize;
    let uheight = height as usize;

    let mut half_pixels =
        vec![Rgba::from_f32(0.0, 0.0, 0.0, 0.0); (width * height) as usize];
    file.set_frame_buffer(&mut half_pixels, 1, width as usize)?;
    unsafe {
        file.read_pixels(0, height - 1)?;
    }

    let pixels = half_pixels.into_iter().map(|p| p.into()).collect();

    Ok(Image::new(name, uwidth, uheight, pixels))
}

fn write_exr(filename: &str, image: Image) -> Result<(), Box<dyn Error>> {
    let header = Header::from_dimensions(image.width as i32, image.height as i32);
    let mut file = RgbaOutputFile::new(filename, &header, RgbaChannels::WriteRgba, 1)?;

    let half_pixels: Vec<Rgba> = image.pixels.into_iter().map(|p| p.into()).collect();

    file.set_frame_buffer(&half_pixels, 1, image.width)?;
    unsafe {
        file.write_pixels(image.height as i32)?;
    }

    Ok(())
}

struct LoadedPlugin {
    bundle: Bundle,
    plugin: Plugin,
    descriptor: Object<ImageEffect>,
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

fn create_plugin(
    bundle_name: &str,
    plugin_name: &str,
    context: &mut CommandContext,
) -> Result<(), Box<dyn Error>> {
    let path = format!("/usr/OFX/Plugins/{}.ofx.bundle", bundle_name);
    let bundle = Bundle::new(path.into()).map_err(|e| GenericError {
        message: format!("Error loading bundle {}", bundle_name),
        source: e,
    })?;
    let lib = bundle.load()?;
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

    let descriptor: Object<ImageEffect> = Default::default();
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
        },
    );
    Ok(())
}

fn create_filter(
    plugin_name: &str,
    instance_name: &str,
    context: &mut CommandContext,
) -> Result<(), Box<dyn Error>> {
    let effect = {
        let plugin = context.get_plugin(plugin_name)?;
        let descriptor = plugin.descriptor.lock();
        let values = &descriptor.properties.lock().values;
        if !values
            .get(OfxImageEffectPropSupportedContexts)
            .map(|p| p.0.contains(&OfxImageEffectContextFilter.into()))
            .unwrap_or(false)
        {
            return Err("Filter context not supported".into());
        }
        if !values
            .get(OfxImageEffectPropSupportedPixelDepths)
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
            create_instance(&filter.lock(), OfxImageEffectContextFilter).into_object();

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
    output_file: &str,
    context: &mut CommandContext,
) -> Result<(), Box<dyn Error>> {
    let instance = context.get_instance(instance_name)?;
    let plugin = context.get_plugin(&instance.plugin_name)?;
    let input = read_exr("input", input_file)?;
    let width = input.width;
    let height = input.height;

    create_images(&mut instance.effect.lock(), input);
    let render_inargs = PropertySet::new(
        "render_inargs",
        [
            (OfxPropTime, (0.0).into()),
            (OfxImageEffectPropFieldToRender, OfxImageFieldNone.into()),
            (OfxImageEffectPropRenderWindow, [0, 0, width, height].into()),
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
    write_exr(
        output_file,
        instance
            .effect
            .lock()
            .clips
            .get("Output")
            .unwrap()
            .lock()
            .image
            .as_ref()
            .unwrap()
            .clone(),
    )?;
    Ok(())
}

fn set_params(
    instance_name: &str,
    values: &[(String, ParamValue)],
    call_instance_changed: bool,
    context: &mut CommandContext,
) -> Result<(), Box<dyn Error>> {
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

fn process_command(
    command: &Command,
    context: &mut CommandContext,
) -> Result<(), Box<dyn Error>> {
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
            output_file,
        } => render_filter(instance_name, input_file, output_file, context),
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

    let args: Vec<String> = env::args().collect();
    let input = &args[1];

    match read_commands(input) {
        Ok(commands) => {
            for ref c in commands {
                if let Err(e) = process_command(c, &mut context) {
                    println!("Error running command: {}", e);
                    break;
                }
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    };
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
