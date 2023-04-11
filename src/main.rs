use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs;
use std::string::String;

pub mod constants;
use constants::actions::*;
use constants::host::*;
use constants::image_effect::*;
use constants::misc::*;
use constants::properties::*;
use constants::suites::*;
mod suite_impls;
mod suites;
mod types;
use types::*;

trait Handle {
    type Object;

    fn as_ref(self) -> &'static mut Self::Object
    where
        Self: Sized + Into<*mut c_void>,
    {
        let ptr: *mut c_void = self.into();
        unsafe { &mut *(ptr as *mut Self::Object) }
    }
}

macro_rules! impl_handle {
    ($handle_name: ident, $object_name: ident) => {
        impl Handle for $handle_name {
            type Object = $object_name;
        }
        impl From<&mut $object_name> for $handle_name {
            fn from(obj: &mut $object_name) -> Self {
                Self::from(obj as *mut $object_name as *mut c_void)
            }
        }
    };
}

impl_handle!(OfxImageEffectHandle, ImageEffect);
impl_handle!(OfxParamSetHandle, ParamSet);
impl_handle!(OfxPropertySetHandle, PropertySet);

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

#[derive(Default, Debug)]
pub struct ParamSet {
    properties: PropertySet,
}

#[derive(Default, Debug)]
pub struct ImageEffect {
    properties: PropertySet,
    params: ParamSet,
}

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
        handle: *const c_void,
        in_args: OfxPropertySetHandle,
        out_args: OfxPropertySetHandle,
    ) -> OfxStatus {
        let c_action = CString::new(action).unwrap();
        (self.main_entry)(c_action.as_ptr(), handle, in_args, out_args)
    }
}

/// An opaque memory address. Used for pointer properties which are
/// never dereferenced by the host, but only pass back to the plugin.
#[derive(Debug, PartialEq)]
struct Addr(*const c_void);
unsafe impl Send for Addr {}

#[derive(Debug)]
#[allow(dead_code)]
enum PropertyValue {
    Pointer(Addr),
    String(CString),
    Double(f64),
    Int(c_int),
    Unset,
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

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct PropertySet(HashMap<String, Property>);

impl<const S: usize> From<[(&str, Property); S]> for PropertySet {
    fn from(slice: [(&str, Property); S]) -> Self {
        let mut map = HashMap::new();
        for (name, value) in slice {
            map.insert(name.into(), value);
        }
        Self(map)
    }
}

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

unsafe fn cstr_to_string(s: *const c_char) -> String {
    CStr::from_ptr(s).to_str().unwrap().to_string()
}

#[allow(unused_variables)]
extern "C" fn fetch_suite(
    host: OfxPropertySetHandle,
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
                std::ptr::null(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );
        let effect: ImageEffect = Default::default();
        println!(
            " describe: {:?}",
            p.call_action(
                OfxActionDescribe,
                std::ptr::addr_of!(effect) as *const c_void,
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        println!(
            " unload: {:?}",
            p.call_action(
                OfxActionUnload,
                std::ptr::null(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            )
        );

        println!(" effect: {:?}", effect);
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
    let mut host_props = PropertySet::from([
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
    ]);
    let host = OfxHost {
        host: (&mut host_props).into(),
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
