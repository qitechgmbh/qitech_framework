use std::{fs, path::Path};

pub fn resolve_serial_by_path(binding: &str) -> Option<String> {
    let base = Path::new("/dev/serial/by-path");

    for entry in fs::read_dir(base).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name == binding {
            let device = fs::canonicalize(entry.path()).ok()?;
            return Some(device.to_string_lossy().to_string());
        }
    }

    None
}
