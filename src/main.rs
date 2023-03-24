use std::fs;

fn ofx_bundles() -> Vec<std::path::PathBuf> {
    if let Ok(dir) = fs::read_dir("/usr/OFX/Plugins/") {
        let x = dir.filter_map(|entry| {
            let path: std::path::PathBuf = entry.ok()?.path();
            if path.is_dir() {
                if let Some(f) = path.file_name() {
                    if f.to_str().map_or(false, |s| s.ends_with(".ofx.bundle")) {
                        return Some(path);
                    }
                }
            }
            return None;
        });
        return x.collect();
    }
    return [].to_vec();
}

fn main() {
    for path in ofx_bundles() {
        println!("{}", path.display())
    }
}
