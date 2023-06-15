use crate::constants::{CONFIG, RULES};
use crate::utils::{process_files_with_ext, run_command};
use flate2::read::GzDecoder;
use log::info;
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tar::Archive;

pub fn run(txl: Option<PathBuf>) {
    info!("STARTING TXL RULES APP");
    let path = dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".cargo/bin");
    let path_string = path
        .to_str()
        .expect("Failed to convert path to string")
        .to_string();

    let mut rules: Vec<String> = RULES.into_iter().map(|rule| rule.to_string()).collect();
    let rules_folder = path.join("Rust/");
    let all_rules_exist = rules.iter().all(|rule| rules_folder.join(rule).exists());

    if !all_rules_exist {
        download_and_extract_rules();
    } else {
        info!("TXL rules found locally");
    }

    if let Some(file_path) = txl {
        let exe_file = process_txl_file(&file_path, &path);
        rules.push(exe_file);
    }

    //build the name of .x file
    let var_path = format!(
        "{}/Rust:{}:{}",
        &path_string,
        &path_string,
        std::env::var("PATH").expect("Failed to get PATH environment variable")
    );
    std::env::set_var("PATH", var_path);
    apply_transformation_rules(&rules, &path_string);
}

fn download_and_extract_rules() {
    info!("TXL rules not found locally, downloading all TXL rules... ");
    let resp =
        reqwest::blocking::get(CONFIG.url).expect("failed to get a response for the rules request");
    let bytes = resp
        .bytes()
        .expect("failed to read response bytes for the rules");
    let tar = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(tar);
    let path = dirs::home_dir().unwrap().join(".cargo/bin");
    archive.unpack(&path).expect("failed to unpack rules");
    info!("downloaded txl rules successfully");
}

fn get_filename_and_filepath(file_path: &Path) -> (String, String) {
    let filename = file_path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_string();
    let filepath = file_path
        .parent()
        .and_then(Path::to_str)
        .unwrap_or("")
        .to_string();
    (filename, filepath)
}

fn apply_transformation_rules(rules: &Vec<String>, path_string: &str) {
    for r in rules {
        info!("applying {r}...");
        process_files_with_ext("rs", |file| {
            let txl_command = Command::new(r)
                .args([&file, "-", &format!("{path_string}/Rust")])
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed txl command");
            let rustfmt = Command::new("rustfmt")
                .stdin(txl_command.stdout.unwrap())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed rustfmt command");
            let output = rustfmt
                .wait_with_output()
                .expect("failed to write to stdout");
            std::fs::write(file, output.stdout).expect("can't write to the file");
        });
    }
}

fn process_txl_file(txl: &Path, path: &Path) -> String {
    let old_dir = env::current_dir().unwrap();

    let (filename, filepath) = get_filename_and_filepath(txl);

    env::set_current_dir(filepath).unwrap();
    run_command("txlc", &[&filename]);
    let exe_filename = filename.split('.').next().unwrap();
    let exe_file = format!("{}{}", exe_filename, ".x");
    std::fs::copy(&exe_file, path.join(&exe_file)).unwrap();
    env::set_current_dir(old_dir).unwrap();

    exe_file
}
