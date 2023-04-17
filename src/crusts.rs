use crate::utils::is_file_with_ext;
use flate2::read::GzDecoder;
use jwalk::WalkDir;
use reqwest;
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};
use tar::Archive;

#[cfg(target_os = "macos")]
pub const URL: &str = "http://bertrust.s3.amazonaws.com/crusts-macosx.tar.gz";
#[cfg(target_os = "macos")]
#[cfg(target_os = "linux")]
pub const URL: &str = "http://bertrust.s3.amazonaws.com/crusts-linux.tar.gz";
#[cfg(target_os = "windows")]
pub const URL: &str = "http://bertrust.s3.amazonaws.com/crusts-windows.tar.gz";

pub fn run(txl: Option<PathBuf>) {
    let mut home = "/home/ubuntu".to_string();
    if let Ok(h) = std::env::var("HOME") {
        home = format!("{}", h);
    }
    let p = format!("{}/.cargo/bin", home);
    if !std::path::Path::new(&format!("{}/Rust/unsafe.x", p)).exists() {
        println!("downloading txl rules ... ");
        if let Ok(resp) = reqwest::blocking::get(URL) {
            if let Ok(bytes) = resp.bytes() {
                let tar = GzDecoder::new(&bytes[..]);
                let mut archive = Archive::new(tar);
                archive.unpack(&p).ok();
            } else {
                eprintln!("Couldn't download, please check your network connection.");
            }
            println!("downloaded ... ");
        } else {
            eprintln!("Couldn't download, please check your network connection.");
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
    let txl_on = txl.is_some();
    let mut filename = String::new();
    let mut filepath = String::new();

    if let Some(file_path) = txl {
        // Get file name without path
        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                filename = name_str.to_string();
            } else {
                println!("File name is not valid UTF-8.");
            }
        }

        // Get path without file name
        if let Some(parent_path) = file_path.parent() {
            if let Some(path_str) = parent_path.to_str() {
                filepath = path_str.to_string();
            } else {
                println!("Parent path is not valid UTF-8.");
            }
        }
    }

    //store current directory
    let old_dir = env::current_dir().unwrap();
    //jump to the directory where txl file is
    let _dir = env::set_current_dir(&filepath);
    //compile the .txl file to .x file
    let _txl_command = Command::new("txlc")
        .arg(&filename)
        .stdout(Stdio::piped())
        .output()
        .expect("failed txl command");
    //build the name of .x file
    let exe_filename = filename.split(".").nth(0).unwrap();
    let exe_file = format!("{}{}", exe_filename, ".x");
    if txl_on {
        //copy .x file to dedicated directory
        let _cp_command = Command::new("cp")
            .arg(&exe_file)
            .arg(&p)
            .stdout(Stdio::piped())
            .spawn()
            .expect("copying .x file faild");
        //go back to original folder
        let _dir = env::set_current_dir(&old_dir);
        //push the new .x file into the vector
        rules.push(&exe_file);
    }

    let var_path = format!("{}/Rust:{}:{}", &p, &p, std::env::var("PATH").unwrap());
    std::env::set_var("PATH", var_path);
    for r in rules {
        println!("applying {r}...");
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
                        format!("{}/Rust", p),
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
