use std::error::Error;
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr, CString};
use std::fs;

struct OfxPropertySet {}
type OfxPropertySetHandle = *mut OfxPropertySet;
type OfxImageEffectHandle = *mut c_void;
type OfxParamSetHandle = *mut c_void;
type OfxImageClipHandle = *mut c_void;
type OfxImageMemoryHandle = *mut c_void;
type OfxStatus = c_int;
type OfxTime = c_double;
#[allow(dead_code)]
struct OfxRectD {
    x1: c_double,
    y1: c_double,
    x2: c_double,
    y2: c_double,
}

#[allow(non_snake_case)]
#[repr(C)]
struct OfxHost {
    host: OfxPropertySetHandle,
    fetchSuite: extern "C" fn(OfxPropertySetHandle, *const c_char, c_int) -> *const c_void,
}

#[allow(non_snake_case)]
#[repr(C)]
struct OfxPluginRaw {
    pluginApi: *const c_char,
    apiVersion: c_int,
    pluginIdentifier: *const c_char,
    pluginVersionMajor: c_uint,
    pluginVersionMinor: c_uint,
    setHost: extern "C" fn(*const OfxHost),
    mainEntry: extern "C" fn(
        *const c_char,
        *const c_void,
        OfxPropertySetHandle,
        OfxPropertySetHandle,
    ) -> OfxStatus,
}

#[derive(Debug)]
#[allow(dead_code)]
struct OfxPlugin {
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

impl OfxPlugin {
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

// ========= ImageEffectSuite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn getPropertySet(
    imageEffect: OfxImageEffectHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
fn getParamSet(imageEffect: OfxImageEffectHandle, paramSet: *mut OfxParamSetHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn clipDefine(
    imageEffect: OfxImageEffectHandle,
    name: *const char,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn clipGetHandle(
    imageEffect: OfxImageEffectHandle,
    name: *const char,
    clip: *mut OfxImageClipHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
fn clipGetPropertySet(
    clip: OfxImageClipHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn clipGetImage(
    clip: OfxImageClipHandle,
    time: OfxTime,
    region: *const OfxRectD,
    imageHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn clipReleaseImage(imageHandle: OfxPropertySetHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
fn clipGetRegionOfDefinition(
    clip: OfxImageClipHandle,
    time: OfxTime,
    bounds: *const OfxRectD,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn abort(imageEffect: OfxImageEffectHandle) -> c_int {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn imageMemoryAlloc(
    instanceHandle: OfxImageEffectHandle,
    nBytes: usize,
    memoryHandle: *mut OfxImageMemoryHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn imageMemoryFree(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
fn imageMemoryLock(memoryHandle: OfxImageMemoryHandle, returnedPtr: *mut *mut c_void) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn imageMemoryUnlock(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxImageEffectSuiteV1 {
    getPropertySet:
        fn(imageEffect: OfxImageEffectHandle, propHandle: *mut OfxPropertySetHandle) -> OfxStatus,
    getParamSet:
        fn(imageEffect: OfxImageEffectHandle, paramSet: *mut OfxParamSetHandle) -> OfxStatus,
    clipDefine: fn(
        imageEffect: OfxImageEffectHandle,
        name: *const char,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipGetHandle: fn(
        imageEffect: OfxImageEffectHandle,
        name: *const char,
        clip: *mut OfxImageClipHandle,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipGetPropertySet:
        fn(clip: OfxImageClipHandle, propHandle: *mut OfxPropertySetHandle) -> OfxStatus,
    clipGetImage: fn(
        clip: OfxImageClipHandle,
        time: OfxTime,
        region: *const OfxRectD,
        imageHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipReleaseImage: fn(imageHandle: OfxPropertySetHandle) -> OfxStatus,
    clipGetRegionOfDefinition:
        fn(clip: OfxImageClipHandle, time: OfxTime, bounds: *const OfxRectD) -> OfxStatus,
    abort: fn(imageEffect: OfxImageEffectHandle) -> c_int,
    imageMemoryAlloc: fn(
        instanceHandle: OfxImageEffectHandle,
        nBytes: usize,
        memoryHandle: *mut OfxImageMemoryHandle,
    ) -> OfxStatus,
    imageMemoryFree: fn(memoryHandle: OfxImageMemoryHandle) -> OfxStatus,
    imageMemoryLock:
        fn(memoryHandle: OfxImageMemoryHandle, returnedPtr: *mut *mut c_void) -> OfxStatus,
    imageMemoryUnlock: fn(memoryHandle: OfxImageMemoryHandle) -> OfxStatus,
}

const IMAGE_EFFECT_SUITE: OfxImageEffectSuiteV1 = OfxImageEffectSuiteV1 {
    getPropertySet,
    getParamSet,
    clipDefine,
    clipGetHandle,
    clipGetPropertySet,
    clipGetImage,
    clipReleaseImage,
    clipGetRegionOfDefinition,
    abort,
    imageMemoryAlloc,
    imageMemoryFree,
    imageMemoryLock,
    imageMemoryUnlock,
};

// ========= Property Suite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetPointerN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propSetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *mut c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetPointerN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *mut c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propReset(properties: OfxPropertySetHandle, property: *const c_char) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
fn propGetDimension(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxPropertySuiteV1 {
    propSetPointer: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_void,
    ) -> OfxStatus,
    propSetString: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *const c_char,
    ) -> OfxStatus,
    propSetDouble: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: c_double,
    ) -> OfxStatus,
    propSetInt: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: c_int,
    ) -> OfxStatus,
    propSetPointerN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const *mut c_void,
    ) -> OfxStatus,
    propSetStringN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const *const c_char,
    ) -> OfxStatus,
    propSetDoubleN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const c_double,
    ) -> OfxStatus,
    propSetIntN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const c_int,
    ) -> OfxStatus,
    propGetPointer: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut *mut c_void,
    ) -> OfxStatus,
    propGetString: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut *mut c_char,
    ) -> OfxStatus,
    propGetDouble: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_double,
    ) -> OfxStatus,
    propGetInt: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_int,
    ) -> OfxStatus,
    propGetPointerN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut *mut c_void,
    ) -> OfxStatus,
    propGetStringN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut *mut c_char,
    ) -> OfxStatus,
    propGetDoubleN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut c_double,
    ) -> OfxStatus,
    propGetIntN: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut c_int,
    ) -> OfxStatus,
    propReset: fn(properties: OfxPropertySetHandle, property: *const c_char) -> OfxStatus,
    propGetDimension: fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: *mut c_int,
    ) -> OfxStatus,
}

const PROPERTY_SUITE: OfxPropertySuiteV1 = OfxPropertySuiteV1 {
    propSetPointer,
    propSetString,
    propSetDouble,
    propSetInt,
    propSetPointerN,
    propSetStringN,
    propSetDoubleN,
    propSetIntN,
    propGetPointer,
    propGetString,
    propGetDouble,
    propGetInt,
    propGetPointerN,
    propGetStringN,
    propGetDoubleN,
    propGetIntN,
    propReset,
    propGetDimension,
};

fn plist_path(bundle_path: &std::path::Path) -> std::path::PathBuf {
    return bundle_path.join("Contents/Info.plist");
}

struct OfxBundle {
    path: std::path::PathBuf,
    plist: plist::Value,
}

fn make_bundle(path: std::path::PathBuf) -> Result<OfxBundle, Box<dyn Error>> {
    let plist = plist::Value::from_file(plist_path(&path))?;
    return Ok(OfxBundle { path, plist });
}

fn library_path(bundle: &OfxBundle) -> std::path::PathBuf {
    let lib = bundle
        .plist
        .as_dictionary()
        .unwrap()
        .get("CFBundleExecutable")
        .unwrap()
        .as_string()
        .unwrap();
    if cfg!(target_os = "linux") {
        return bundle.path.join("Contents/Linux-x86-64").join(lib);
    } else if cfg!(windows) {
        return bundle.path.join("Contents/Win64").join(lib);
    } else {
        return bundle.path.join("Contents/MacOS").join(lib);
    }
}

fn ofx_bundles() -> Vec<OfxBundle> {
    if let Ok(dir) = fs::read_dir("/usr/OFX/Plugins/") {
        let x = dir.filter_map(|entry| {
            let path: std::path::PathBuf = entry.ok()?.path();
            if path.is_dir() {
                if let Some(f) = path.file_name() {
                    if f.to_str().map_or(false, |s| s.ends_with(".ofx.bundle")) {
                        return Some(make_bundle(path).ok()?);
                    }
                }
            }
            return None;
        });
        return x.collect();
    }
    return Vec::new();
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
    println!("fetch_suite {} {}", suite, version);
    if suite == "OfxImageEffectSuite" {
        assert!(version == 1);
        &IMAGE_EFFECT_SUITE as *const _ as *const c_void
    } else if suite == "OfxPropertySuite" {
        assert!(version == 1);
        &PROPERTY_SUITE as *const _ as *const c_void
    } else {
        std::ptr::null()
    }
}

fn main() {
    let mut host_props = OfxPropertySet {};
    let host = OfxHost {
        host: &mut host_props,
        fetchSuite: fetch_suite,
    };

    for bundle in ofx_bundles() {
        let count;
        let mut plugins = Vec::new();

        unsafe {
            let lib = libloading::Library::new(library_path(&bundle)).unwrap();
            let func: libloading::Symbol<unsafe extern "C" fn() -> i32> =
                lib.get(b"OfxGetNumberOfPlugins").unwrap();
            count = func();
            let func2: libloading::Symbol<unsafe extern "C" fn(i32) -> *const OfxPluginRaw> =
                lib.get(b"OfxGetPlugin").unwrap();
            for i in 0..count {
                let p = &*func2(i);
                plugins.push(OfxPlugin {
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
        println!(
            "{}, {} => {}",
            bundle.path.display(),
            library_path(&bundle).display(),
            count
        );
        for p in plugins {
            (p.set_host)(&host);
            let stat = p.call_action(
                "OfxActionLoad",
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            let stat2 = p.call_action(
                "OfxActionUnload",
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            println!(
                "  {:?}, Load returned {}, Unload returned {}",
                p, stat, stat2
            );
        }
        println!()
    }
}
