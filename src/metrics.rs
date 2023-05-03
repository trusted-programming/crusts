use crate::utils::{is_file_with_ext, run_clippy_json_output};
use fs_extra::dir::CopyOptions;
use jwalk::WalkDir;
use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

// TODO: METRICS ALL IN A SINGLE JSON FILE, APPEND IF FILE ALREADY EXISTS
// TODO: metrics inside the project folder, ignore target and metrics folders when copying
/// this does tree things:
/// - store files
/// - get unsafe % and write to metrics.json
/// - get clippy warnings number to metrics.json
pub fn run(step: &str) {
    info!("STARTING METRICS COLLECTIONS");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let current_dir_name = current_dir
        .file_name()
        .expect("Failed to get current directory name")
        .to_str()
        .expect("Failed to convert current directory name to string");
    let parent = current_dir
        .parent()
        .expect("Failed to get parent directory");
    let metrics_dir = parent.join("metrics");
    if !metrics_dir.exists() {
        fs::create_dir(&metrics_dir).expect("Failed to create metrics directory");
    }
    let metrics_proj_dir = metrics_dir.join(current_dir_name);
    if !metrics_proj_dir.exists() {
        fs::create_dir(&metrics_proj_dir).expect("Failed to create metrics directory");
    }
    let step_dir = metrics_proj_dir.join(step);
    if !step_dir.exists() {
        fs::create_dir(&step_dir).unwrap_or_else(|_| panic!("Failed to create {step} directory"));
    }

    let clippy_warnings_count = run_clippy_json_output().len();

    info!("found {clippy_warnings_count} clippy_warnings");

    // TODO: stop copying target the target folder
    info!(
        "copying {} to {}",
        current_dir.to_str().unwrap(),
        step_dir.to_str().unwrap()
    );
    fs_extra::dir::copy(&current_dir, &step_dir, &CopyOptions::new().overwrite(true))
        .expect("failed to copy rust files to metrics");

    let mut unsafe_functions_count = 0;
    let mut total_functions_count = 0;
    WalkDir::new(".")
        .sort(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_file_with_ext(&e.path(), "rs"))
        .for_each(|e| {
            let path = e.path();
            info!("found file_path: {}", path.to_str().unwrap());
            let (unsafe_functions, total_functions) =
                calculate_number_of_unsafe_function_and_safe(path.to_str().unwrap());
            unsafe_functions_count += unsafe_functions;
            total_functions_count += total_functions;
        });
    let safe_percentage =
        100.0 - unsafe_functions_count as f32 * 100.0 / total_functions_count as f32;
    let metrics = Metrics {
        step_dir: step_dir,
        unsafe_functions_count,
        total_functions_count,
        safe_percentage,
        clippy_warnings_count,
    };
    info!("writing metrics to file");
    metrics.write_to_file();
}

// TODO: add rust-analysis-tool for more metrics
#[derive(Serialize, Deserialize)]
struct Metrics {
    step_dir: PathBuf,
    total_functions_count: usize,
    unsafe_functions_count: usize,
    safe_percentage: f32,
    clippy_warnings_count: usize,
}

impl Metrics {
    fn write_to_file(&self) {
        let metrics_file = self.step_dir.join("metrics.json");
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write(metrics_file, json).expect("Failed to write metrics to file");
    }
}

// TODO: use tree-sitter for this or rust sitter or cargo geiger, maybe using syn and quote
fn calculate_number_of_unsafe_function_and_safe(path: &str) -> (usize, usize) {
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
    info!("found: {unsafe_functions_count} unsafe functions; {total_functions_count} safe functions numbers");

    (unsafe_functions_count, total_functions_count)
}
