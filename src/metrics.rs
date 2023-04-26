use serde::{Deserialize, Serialize};
use std::process::Command;

/// this does tree things:
/// - store files
/// - get unsafe % and write to metrics.json
/// - get clippy warnings number to metrics.json
pub fn run(step: &str) {
    fs_extra::dir::copy(
        ".",
        format!("metrics/{step}"),
        &fs_extra::dir::CopyOptions::new(),
    )
    .expect("failed to copy rust files to metrics");

    //TODO: check out prof yu script for calulating unsafe percentage

    // FIXME: this is not a Json output
    let clippy_output = Command::new("cargo").arg("clippy").output().unwrap();
    let clippy_output_json = Command::new("cargo").arg("clippy").output().unwrap();
    unimplemented!()
}

#[derive(Serialize, Deserialize)]
struct Metrics {
    unsafe_percentage: String,
    clippy_warnings: usize,
}
