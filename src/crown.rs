use crate::utils::{command_exists, run_command};
use log::info;
// create toolchain.toml with project to nightly-2023-01-26
// remove use ::c2rust_out::*; line from main.rs or lib.rs
// use walk to apply to all files
//
pub fn run() {
    if command_exists("crown") {
        info!("crown command found! Running...");
        run_command("bash", &["/home/c00834010/dev/crown/run.sh"]);
        // run_command(
        //     "/home/c00834010/dev/crown/target/release/crown",
        //     &["src/avl.rs", "preprocess", "in-place"],
        // );
        // run_command(
        //     "/home/c00834010/dev/crown/target/release/crown",
        //     &["src/avl.rs", "explicit-addr", "in-place"],
        // );
        // run_command(
        //     "/home/c00834010/dev/crown/target/release/crown",
        //     &["src/avl.rs", "rewrite"],
        // );
        info!("crown command operations successfully completed");
    } else {
        info!("crown command not found, skipping...");
    }
}
