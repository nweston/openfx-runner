use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir("/usr/OFX/Plugins/")? {
        let path = entry?.path();
        if path.is_dir() {
            if let Some(f) = path.file_name() {
                if f.to_str().map_or(false, |s| s.ends_with(".ofx.bundle")) {
                    println!("{}", path.display());
                }
            }
        }
    }
    return Ok(());
}
