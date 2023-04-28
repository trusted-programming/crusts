use crate::utils::run_clippy_json_output;
use fs_extra::dir::CopyOptions;
use fs_extra::file;
use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::{env, fs};

/// this does tree things:
/// - store files
/// - get unsafe % and write to metrics.json
/// - get clippy warnings number to metrics.json
pub fn run(step: &str) {
    info!("STARTING METRICS COLLECTIONS");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let metrics_dir = Path::new("metrics");
    let step_dir = metrics_dir.join(step);
    if !metrics_dir.exists() {
        fs::create_dir(metrics_dir).expect("Failed to create metrics directory");
    } else if !step_dir.exists() {
        fs::create_dir(&step_dir).unwrap_or_else(|_| panic!("Failed to create {step} directory"));
    }

    let mut options = CopyOptions::new();
    options.overwrite = true;
    info!(
        "copying {} to {}",
        current_dir.to_str().unwrap(),
        step_dir.to_str().unwrap()
    );
    fs_extra::dir::copy(
        current_dir,
        format!("metrics/{step}"),
        &fs_extra::dir::CopyOptions::new(),
    )
    .expect("failed to copy rust files to metrics");

    let clippy_warnings = run_clippy_json_output()
        .as_array()
        .unwrap()
        .iter()
        .filter(|msg| msg["message"]["level"].as_str().unwrap() == "warning")
        .count();
    info!("found {clippy_warnings} clippy_warnings");
    let mut file_path = step_dir.join("src");
    if file_path.join("main.rs").exists() {
        file_path = file_path.join("main.rs")
    } else {
        file_path = file_path.join("lib.rs")
    }
    info!("found file_path: {}", file_path.to_str().unwrap());

    let (unsafe_functions_count, total_functions_count) =
        calculate_number_of_unsafe_function_and_safe(file_path.to_str().unwrap());
    let unsafe_percentage =
        100.0 - unsafe_functions_count as f32 * 100.0 / total_functions_count as f32;
    let metrics = Metrics {
        unsafe_functions_count,
        total_functions_count,
        unsafe_percentage,
        clippy_warnings,
    };
    info!("writing metrics to file");
    metrics.write_to_file();
}

// TODO: add rust-analysis-tool for more metrics
#[derive(Serialize, Deserialize)]
struct Metrics {
    total_functions_count: usize,
    unsafe_functions_count: usize,
    unsafe_percentage: f32,
    clippy_warnings: usize,
}

impl Metrics {
    fn write_to_file(&self) {
        let metrics_file = Path::new("metrics").join("metrics.json");
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write(metrics_file, json).expect("Failed to write metrics to file");
    }
}

// TODO: use tree-sitter for this or rust sitter or cargo geiger
fn calculate_number_of_unsafe_function_and_safe(path: &str) -> (usize, usize) {
    info!("calculating unsafe functions and safe functions numbers");
    let unsafe_functions = Command::new("tree-grepper")
        .args([
            "-q",
            "rust",
            r#"((function_item (function_modifiers) @_m)@f (#match? @_m "unsafe"))"#,
            path,
        ])
        .output()
        .expect("Failed to run tree-grepper")
        .stdout;
    let unsafe_functions_count = String::from_utf8_lossy(&unsafe_functions)
        .lines()
        .filter(|line| line.contains(":f:"))
        .count();

    let total_functions = Command::new("tree-grepper")
        .args(["-q", "rust", r#"((function_item)@f)"#, path])
        .output()
        .expect("Failed to run tree-grepper")
        .stdout;
    let total_functions_count = String::from_utf8_lossy(&total_functions)
        .lines()
        .filter(|line| line.contains(":f:"))
        .count();

    (unsafe_functions_count, total_functions_count)
}
