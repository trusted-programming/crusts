use crate::utils::{command_exists, run_command};
use log::info;

pub fn run() {
    
    if command_exists("crown") {
        info!("crown command found! Running...");
        run_command("crown", &["main.rs", "preprocess", "in-place"]);
        run_command("crown", &["main.rs", "explicit-addr", "in-place"]);
        run_command("crown", &["main.rs", "rewrite"]);
        info!("crown command operations successfully completed");
    } else {
        info!("crown command not found, skipping...");
    }
}
