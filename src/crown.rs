use crate::utils::command_exists;
use log::info;
use std::process::{Command, Stdio};

pub fn run() {
    if command_exists("crown") {
        info!("crown command found! Running...");
        run_crown(&["main.rs", "preprocess", "in-place"]);
        run_crown(&["main.rs", "explicit-addr", "in-place"]);
        run_crown(&["main.rs", "rewrite"]);
        info!("crown command operations successfully completed");
    } else {
        info!("crown command not found, skipping...");
    }
}

fn run_crown(args: &[&str]) {
    let mut command = Command::new("crown");
    for arg in args {
        command.arg(arg);
    }
    command
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("failed to run crown with arguments: {:?}", args));
}
