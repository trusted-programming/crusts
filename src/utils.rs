use std::{path::Path, process::Command};
pub fn is_file_with_ext(p: &Path, file_ext: &str) -> bool {
    let ext = match p.extension() {
        Some(e) => e,
        None => return false,
    };
    ext.to_string_lossy() == file_ext
}

pub fn command_exists(command: &str) -> bool {
    match Command::new(command).arg("-h").status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
