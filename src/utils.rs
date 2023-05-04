use cargo_metadata::{CompilerMessage, Message};
use jwalk::{DirEntry, WalkDir};
use std::{
    path::Path,
    process::{Command, Stdio},
};

use log::info;
use serde_json::Value;

pub fn is_file_with_ext(path: &Path, ext: &str) -> bool {
    path.is_file() && path.extension().map_or(false, |e| e == ext)
}

pub fn process_files_with_ext<F>(ext: &str, mut func: F)
where
    F: FnMut(String),
{
    WalkDir::new(".")
        .sort(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_file_with_ext(&e.path(), ext))
        .for_each(|e: DirEntry<((), ())>| {
            let path = e.path();
            let file = path.to_string_lossy().to_string();
            info!("working on {}", path.file_name().unwrap().to_str().unwrap());
            func(file);
        });
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

pub fn run_clippy_json_output() -> Vec<Value> {
    info!("running clippy");
    let output = Command::new("cargo")
        .args(["clippy", "--message-format=json"])
        .output()
        .expect("Failed to run cargo clippy");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut json_values: Vec<Value> = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let value: Value = serde_json::from_str(trimmed).unwrap();

            if let Some(reason) = value.get("reason") {
                if reason == "compiler-message" {
                    json_values.push(value);
                }
            }
        }
    }
    // FIXME: this is a hack to delete the target folder, should be ignored during copy instead of delete
    if Path::new("target").exists() {
        std::fs::remove_dir_all("target").expect("failed to delete target folder folder");
    }

    json_values
}

// TODO: for clippy and cargo check, serialize to a struct instead of a vector of values
pub fn run_cargo_check_json_output() -> Vec<CompilerMessage> {
    info!("running cargo check");
    let mut command = Command::new("cargo")
        .args(["+nightly", "check", "--message-format=json"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let mut compiler_messages: Vec<CompilerMessage> = Vec::new();
    for message in cargo_metadata::Message::parse_stream(reader) {
        if let Ok(msg) = message {
            if let Message::CompilerMessage(compiler_message) = msg {
                compiler_messages.push(compiler_message);
            }
        }
    }

    let _output = command.wait().expect("Couldn't get cargo's exit status");

    // FIXME: this is a hack to delete the target folder, should be ignored during copy instead of delete
    if Path::new("target").exists() {
        std::fs::remove_dir_all("target").expect("failed to delete target folder folder");
    }

    compiler_messages
}
