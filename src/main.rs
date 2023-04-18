use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs;
use std::string::String;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

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
use types::*;

/// Holder for objects which can cross the API boundary.
///
/// Essentially an Arc<Mutex<T>> with some convenience
/// features.
#[derive(Default)]
struct Object<T>(Arc<Mutex<T>>);

impl<T> Object<T> {
    fn get(&self) -> MutexGuard<'_, T> {
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
    fn as_arc(self) -> Object<Self::Object> {
        if let Some(weak) = Self::handle_manager()
            .lock()
            .unwrap()
            .handle_to_ptr
            .get(&self)
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
        let guard = &mut mutex.get();
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

#[derive(Debug)]
#[allow(dead_code)]
enum ParamValue {
    Boolean(bool),
    Choice { options: Vec<String>, index: usize },
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
    RGB { r: f64, g: f64, b: f64 },
    RGBA,
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
            OfxParamTypeChoice => Self::Choice {
                options: Vec::new(),
                index: props.get_type::<i32>(OfxParamPropDefault, 0).unwrap_or(0)
                    as usize,
            },
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
pub struct Image {
    properties: Object<PropertySet>,
    pixels: Vec<f32>,
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
            properties: self.properties.get().clone().into_object(),
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
                    ],
                )
                .into_object(),
                image: None,
            }
            .into_object(),
        );
        self.clips.get(name).unwrap().clone()
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

    fn set(&mut self, key: &str, index: usize, value: PropertyValue) -> () {
        let prop = self
            .values
            .entry(key.to_string())
            .or_insert(Default::default());
        let uindex = index as usize;
        if uindex >= prop.0.len() {
            prop.0.resize_with(uindex + 1, || PropertyValue::Unset)
        }
        prop.0[uindex] = value;
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
            message: format!("Failed reading plist \"{}\"", file.display()).into(),
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

fn ofx_bundles() -> Vec<Bundle> {
    if let Ok(dir) = fs::read_dir("/usr/OFX/Plugins/") {
        let x = dir.filter_map(|entry| {
            let path: std::path::PathBuf = entry.ok()?.path();
            if path.is_dir() {
                if let Some(f) = path.file_name() {
                    if f.to_str().map_or(false, |s| s.ends_with(".ofx.bundle")) {
                        match Bundle::new(path.clone()) {
                            Ok(b) => return Some(b),
                            Err(e) => {
                                println!(
                                    "Error loading bundle {}: {}",
                                    path.display(),
                                    e
                                );
                                return None;
                            }
                        }
                    }
                }
            }
            None
        });
        return x.collect();
    }
    Vec::new()
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
        .map(|(key, val)| (key.clone(), val.get().clone().into_object()))
        .collect()
}

fn create_params(
    descriptors: &Vec<Object<PropertySet>>,
) -> HashMap<String, Object<Param>> {
    descriptors
        .iter()
        .map(|d| {
            let props = d.get();
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
    let properties =
        PropertySet::new("instance", [(OfxImageEffectPropContext, context.into())])
            .into_object();
    let descriptors = &descriptor.param_set.get().descriptors;
    let param_set = ParamSet {
        properties: Default::default(),
        descriptors: descriptors.clone(),
        params: create_params(&descriptors),
    }
    .into_object();
    ImageEffect {
        properties,
        param_set,
        clips,
    }
}

fn create_images(effect: &mut ImageEffect, width: i32, height: i32) -> () {
    let uwidth: usize = width.try_into().unwrap();
    let uheight: usize = height.try_into().unwrap();
    for (name, c) in effect.clips.iter() {
        let mut pixels = Vec::new();
        pixels.resize(4 * uwidth * uheight, 0.0);
        let props = PropertySet::new(
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
                    (4 * uwidth * std::mem::size_of::<f32>()).into(),
                ),
                (OfxImagePropField, OfxImageFieldNone.into()),
            ],
        );
        c.get().image = Some(Image {
            properties: props.into_object(),
            pixels,
        });
    }
}

fn process_bundle(host: &OfxHost, bundle: &Bundle) -> Result<(), Box<dyn Error>> {
    let lib = bundle.load()?;
    let plugins = get_plugins(&lib)?;

    println!("{}, => {}", bundle.path.display(), plugins.len());
    for p in plugins {
        (p.set_host)(host);
        println!("{:?}", p);
        println!(
            " load: {:?}",
            p.call_action(
                OfxActionLoad,
                OfxImageEffectHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        // Overall descriptor for the plugin
        let effect: Object<ImageEffect> = Default::default();
        println!(
            " describe: {:?}",
            p.call_action(
                OfxActionDescribe,
                effect.clone().into(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        assert!(effect
            .get()
            .properties
            .get()
            .values
            .get(OfxImageEffectPropSupportedContexts)
            .map(|p| p.0.contains(&OfxImageEffectContextFilter.into()))
            .unwrap_or(false));
        assert!(effect
            .get()
            .properties
            .get()
            .values
            .get(OfxImageEffectPropSupportedPixelDepths)
            .map(|p| p.0.contains(&OfxBitDepthFloat.into()))
            .unwrap_or(false));

        // Descriptor for the plugin in Filter context
        let filter = ImageEffect {
            properties: PropertySet::new(
                "filter",
                [(OfxPluginPropFilePath, bundle.path.to_str().unwrap().into())],
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

        println!(
            " describe filter: {:?}",
            p.call_action(
                OfxImageEffectActionDescribeInContext,
                filter.clone().into(),
                OfxPropertySetHandle::from(filter_inargs.clone()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        // Instance of the filter. Both instances and descriptors are
        // ImageEffect objects.
        let filter_instance: Object<ImageEffect> =
            create_instance(&filter.get(), OfxImageEffectContextFilter).into_object();

        println!(
            " create instance: {:?}",
            p.call_action(
                OfxActionCreateInstance,
                filter_instance.clone().into(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        let width = 1920;
        let height = 1080;

        create_images(&mut filter_instance.get(), width, height);

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
        println!(
            " render: {:?}",
            p.call_action(
                OfxImageEffectActionRender,
                filter_instance.clone().into(),
                OfxPropertySetHandle::from(render_inargs.clone()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        println!(
            " destroy instance: {:?}",
            p.call_action(
                OfxActionDestroyInstance,
                filter_instance.clone().into(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        println!(
            " unload: {:?}",
            p.call_action(
                OfxActionUnload,
                OfxImageEffectHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        // println!(" effect: {:?}", effect);
        // println!(" filter: {:?}", filter);
        // println!(" instance: {:?}", filter_instance);
    }
    println!();
    Ok(())
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
    let host = OfxHost {
        host: host_props.clone().into(),
        fetchSuite: fetch_suite,
    };

    for bundle in ofx_bundles() {
        if let Err(e) = process_bundle(&host, &bundle) {
            println!("Error processing bundle {}: {}", bundle.path.display(), e);
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
