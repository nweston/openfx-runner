use std::error::Error;
use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use std::fs;

#[allow(non_snake_case)]
#[repr(C)]
struct OfxPluginRaw {
    pluginApi: *const c_char,
    apiVersion: c_int,
    pluginIdentifier: *const c_char,
    pluginVersionMajor: c_uint,
    pluginVersionMinor: c_uint,
    setHost: *const c_void, // (*setHost)(OfxHost *host)
    mainEntry: *const c_void,
}

#[derive(Debug)]
struct OfxPlugin {
    plugin_api: String,
    api_version: i32,
    plugin_identifier: String,
    plugin_version_major: u32,
    plugin_version_minor: u32,
}

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

fn main() {
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
            println!("  {:?}", p);
        }
        println!()
    }
}
