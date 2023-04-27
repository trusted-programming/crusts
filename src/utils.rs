use std::{
    path::Path,
    process::{Command, Stdio},
};

use log::info;
use serde_json::Value;

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
        .unwrap_or_else(|_| panic!("failed to run {command_name} with arguments: {args:?}"))
}

pub fn run_clippy_json_output() -> Value {
    let output = Command::new("cargo")
        .args(&["clippy", "--message-format=json"])
        .output()
        .expect("Failed to run cargo clippy");

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("failed to convert json output string to Json value")
}

pub fn run_check_json_output() -> Value {
    let output = Command::new("cargo")
        .args(&["check", "--message-format=json"])
        .output()
        .expect("Failed to run cargo check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("failed to convert json output string to Json value")
}
