use std::path::Path;

pub fn is_file_with_ext(p: &Path, file_ext: &str) -> bool {
    let ext = match p.extension() {
        Some(e) => e,
        None => return false,
    };
    ext.to_string_lossy() == file_ext
}
