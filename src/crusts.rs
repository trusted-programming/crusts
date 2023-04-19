use crate::utils::is_file_with_ext;
use dirs;
use flate2::read::GzDecoder;
use jwalk::WalkDir;
use log::{error, info};
use reqwest;
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};
use tar::Archive;

#[cfg(target_os = "macos")]
pub const URL: &str = "http://bertrust.s3.amazonaws.com/crusts-macosx.tar.gz";
#[cfg(target_os = "linux")]
pub const URL: &str = "http://bertrust.s3.amazonaws.com/crusts-linux.tar.gz";
#[cfg(target_os = "windows")]
pub const URL: &str = "http://bertrust.s3.amazonaws.com/crusts-windows.tar.gz";

pub fn run(txl: Option<PathBuf>) {
    let path = dirs::home_dir().unwrap().join(".cargo/bin");
    let path_string = path.to_str().unwrap().to_string();

    if !path.join("c/unsafe.x").exists() {
        info!("unsafe.x not found, downloading all txl rules... ");
        if let Ok(resp) = reqwest::blocking::get(URL) {
            if let Ok(bytes) = resp.bytes() {
                let tar = GzDecoder::new(&bytes[..]);
                let mut archive = Archive::new(tar);
                archive.unpack(&path).ok();
                info!("downloaded txl rules successfully");
            } else {
                error!("Couldn't download, please check your network connection.");
                return;
            }
        } else {
            error!("Couldn't download, please check your network connection.");
            return;
        }
    }

    let mut rules = vec![
        "formalizeCode.x",
        "varTypeNoBounds.x",
        "null.x",
        "array.x",
        "fn.x",
        "errnoLocation.x",
        "atoi.x",
        "time.x",
        "const2mut.x",
        "stdio.x",
        "unsafe.x",
    ];
    let old_dir = env::current_dir().unwrap();
    let exe_file;
    if let Some(file_path) = txl {
        let mut filename = String::new();
        let mut filepath = String::new();
        // Get file name without path
        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                filename = name_str.to_string();
            } else {
                info!("File name is not valid UTF-8.");
            }
        }

        // Get path without file name
        if let Some(parent_path) = file_path.parent() {
            if let Some(path_str) = parent_path.to_str() {
                filepath = path_str.to_string();
            } else {
                info!("Parent path is not valid UTF-8.");
            }
        }
        //store current directory
        //jump to the directory where txl file is
        env::set_current_dir(&filepath).unwrap();
        let _txl_command = Command::new("txlc")
            .arg(&filename)
            .stdout(Stdio::piped())
            .output()
            .expect("failed txl command");
        let exe_filename = filename.split(".").nth(0).unwrap();
        exe_file = format!("{}{}", exe_filename, ".x");
        //copy .x file to dedicated directory
        let _cp_command = Command::new("cp")
            .arg(&exe_file)
            .arg(&path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("copying .x file faild");
        //go back to original folder
        let _dir = env::set_current_dir(&old_dir);
        //push the new .x file into the vector
        rules.push(&exe_file);
    }

    //build the name of .x file

    let var_path = format!(
        "{}/Rust:{}:{}",
        &path_string,
        &path_string,
        std::env::var("PATH").unwrap()
    );
    std::env::set_var("PATH", var_path);
    for r in rules {
        info!("applying {r}...");
        WalkDir::new(".").sort(true).into_iter().for_each(|entry| {
            if let Ok(e) = entry {
                let path = e.path();
                if !is_file_with_ext(&path, "rs") {
                    return;
                }
                let file = &format!("{}", &path.into_os_string().to_string_lossy());
                let _txl_command = Command::new(r)
                    .args(vec![
                        file.to_string(),
                        "-".to_string(),
                        format!("{}/Rust", &path_string),
                    ])
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed txl command");
                let _rustfmt = Command::new("rustfmt")
                    .stdin(_txl_command.stdout.unwrap())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed rustfmt command");
                let output = _rustfmt
                    .wait_with_output()
                    .expect("failed to write to stdout");
                std::fs::write(&file, &output.stdout).expect("can't write to the file");
            }
        });
    }
}
