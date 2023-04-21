use std::{
    path::Path,
    process::{Command, Stdio},
};

pub fn is_file_with_ext(p: &Path, file_ext: &str) -> bool {
    p.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext_str| ext_str == file_ext)
}

pub fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("-h")
        .stdout(Stdio::null())
        .status()
        .map_or(false, |status| status.success())
}
