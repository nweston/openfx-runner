use std::error::Error;
use std::fs;

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

fn main() {
    for bundle in ofx_bundles() {
        println!(
            "{}, {:?}",
            bundle.path.display(),
            bundle.plist.as_dictionary().unwrap()
        )
    }
}
