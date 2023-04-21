use std::{
    path::Path,
    process::{Command, Stdio},
};

use log::info;

#[derive(Debug, Copy, Clone)]
pub struct PlatformConfig {
    pub bear: &'static str,
    pub bear_args: &'static [&'static str],
    pub url: &'static str,
}

#[cfg(target_os = "macos")]
pub const CONFIG: PlatformConfig = PlatformConfig {
    bear: "bear",
    bear_args: &["--", "make"],
    url: "http://bertrust.s3.amazonaws.com/crusts-macosx.tar.gz",
};

#[cfg(target_os = "windows")]
pub const CONFIG: PlatformConfig = PlatformConfig {
    bear: "intercept-build",
    bear_args: &["make"],
    url: "http://bertrust.s3.amazonaws.com/crusts-windows.tar.gz",
};

#[cfg(target_os = "linux")]
pub const CONFIG: PlatformConfig = PlatformConfig {
    bear: "bear",
    bear_args: &["--", "make"],
    url: "http://bertrust.s3.amazonaws.com/crusts-linux.tar.gz",
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

pub fn path_exists(filename: &str) -> bool {
    std::path::Path::new(filename).exists()
}

pub fn run_command(command_name: &str, args: &[&str]) {
    info!("running {command_name} with arguments: {args:?}");
    let mut command = Command::new(command_name);
    for arg in args {
        command.arg(arg);
    }
    command
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|command| command.wait_with_output())
        .map(|output| info!("{output:?}"))
        .expect(&format!(
            "failed to run {command_name} with arguments: {args:?}"
        ));
}
