use crate::utils::run_clippy_json_output;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

/// this does tree things:
/// - store files
/// - get unsafe % and write to metrics.json
/// - get clippy warnings number to metrics.json
pub fn run(step: &str) {
    let metrics_dir = Path::new("metrics");
    let step_dir = metrics_dir.join(step);
    if !metrics_dir.exists() {
        fs::create_dir(metrics_dir).expect("Failed to create metrics directory");
    } else if !step_dir.exists() {
        fs::create_dir(&step_dir).expect(&format!("Failed to create {step} directory"));
    }

    fs_extra::dir::copy(
        ".",
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
    let mut file_path = step_dir.join("src");
    if file_path.join("main.rs").exists() {
        file_path = file_path.join("main.rs")
    } else {
        file_path = file_path.join("lib.rs")
    }

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
    let unsafe_functions = Command::new("tree-grepper")
        .args(&[
            "-q",
            "rust",
            r#"((function_item (function_modifiers) @_m)@f (#match? @_m "unsafe"))"#,
            &path,
        ])
        .output()
        .expect("Failed to run tree-grepper")
        .stdout;
    let unsafe_functions_count = String::from_utf8_lossy(&unsafe_functions)
        .lines()
        .filter(|line| line.contains(":f:"))
        .count();

    let total_functions = Command::new("tree-grepper")
        .args(&["-q", "rust", r#"((function_item)@f)"#, &path])
        .output()
        .expect("Failed to run tree-grepper")
        .stdout;
    let total_functions_count = String::from_utf8_lossy(&total_functions)
        .lines()
        .filter(|line| line.contains(":f:"))
        .count();

    (unsafe_functions_count, total_functions_count)
}

pub fn run_cargo_check() -> Value {
    let output = Command::new("cargo")
        .args(&["check", "--message-format=json"])
        .output()
        .expect("Failed to run cargo check");

    let stdout = String::from_utf8_lossy(&output.stdout);

    serde_json::from_str(&stdout).unwrap()
}
