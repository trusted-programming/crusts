use crate::utils::command_exists;
use log::info;
use std::process::Command;
pub fn run() {
    if command_exists("crown") {
        info!("crown command found! Running...");
        Command :: new ("crown").arg ("main.rs").arg ("preprocess").arg ("in-place").output ().expect ("failed to run crown main.rs preprocess in-place, please install crown if you haven't or check that main.rs is present",);
        Command::new("crown")
            .arg("main.rs")
            .arg("explicit-addr")
            .arg("in-place")
            .output()
            .expect("failed to run crown main.rs explicit-addr in-place");
        Command::new("crown")
            .arg("main.rs")
            .arg("rewrite")
            .output()
            .expect("failed to run crown main.rs rewrite");
        info!("crown command operations successfully completed");
    } else {
        info!("crown command not found, skipping...");
    }
}
