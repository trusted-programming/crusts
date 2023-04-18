use crate::utils::is_file_with_ext;
use jwalk::WalkDir;
use std::process::{Command, Stdio};

#[cfg(target_os = "macos")]
const BEAR: &str = "bear";
#[cfg(target_os = "macos")]
const BEAR_ARGS: [&str; 2] = ["--", "make"];
#[cfg(target_os = "windows")]
const BEAR: &str = "intercept-build";
#[cfg(target_os = "windows")]
const BEAR_ARGS: [&str; 1] = ["make"];
#[cfg(target_os = "linux")]
const BEAR: &str = "bear";
#[cfg(target_os = "linux")]
const BEAR_ARGS: [&str; 2] = ["--", "make"];

pub fn run() {
    if !std::path::Path::new("Cargo.toml").exists() {
        if !std::path::Path::new("compile_commands.json").exists() {
            if !std::path::Path::new("Makefile").exists()
                && !std::path::Path::new("makefile").exists()
                && !std::path::Path::new("configure").exists()
                && !std::path::Path::new("configure.ac").exists()
            {
                let mut c_files = Vec::new();
                WalkDir::new(".").sort(true).into_iter().for_each(|entry| {
                    if let Ok(e) = entry {
                        let p = e.path();
                        if !is_file_with_ext(&p, "c") && !is_file_with_ext(&p, "cpp") {
                            return;
                        }
                        let file = format!("{}", &p.into_os_string().to_string_lossy());
                        c_files.push(file);
                    }
                });
                let mut obj = "".to_string();
                for c_file in c_files {
                    obj.push_str(" \\\n");
                    obj.push_str(&c_file.replace(".c", ".o"));
                }
                std :: fs :: write ("Makefile", format! ("main: {}\n\tgcc -o main {}\n\n.c.o: \n\tgcc -c $<\n\n.cpp.o: \n\tg++ -c $<\n\nclean::\n\trm -rf Makefile main c2rust crusts compile_commands.json Cargo.lock target", obj, obj)).ok ();
            }
            if !std::path::Path::new("Makefile").exists()
                && !std::path::Path::new("configure").exists()
                && std::path::Path::new("configure.ac").exists()
            {
                if let Ok(command) = Command::new("autoreconf")
                    .args(["-fi"])
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    if let Ok(output) = command.wait_with_output() {
                        println!("{:?}", output);
                    }
                }
            }
            if !std::path::Path::new("Makefile").exists()
                && std::path::Path::new("configure").exists()
            {
                if let Ok(command) = Command::new("./configure").stdout(Stdio::piped()).spawn() {
                    if let Ok(output) = command.wait_with_output() {
                        println!("{:?}", output);
                    }
                }
            }
            if std::path::Path::new("Makefile").exists() {
                if let Ok(bear_version) = Command::new(BEAR)
                    .args(["--version"])
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    if let Ok(output) = bear_version.wait_with_output() {
                        let s = String::from_utf8_lossy(&output.stdout);
                        if s.contains("bear 2.4.3") {
                            if let Ok(command) = Command::new(BEAR)
                                .args(["make"])
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                if let Ok(output) = command.wait_with_output() {
                                    println!("{}", String::from_utf8_lossy(&output.stdout));
                                }
                            }
                        } else {
                            if let Ok(command) = Command::new(BEAR)
                                .args(BEAR_ARGS)
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                if let Ok(output) = command.wait_with_output() {
                                    println!("{}", String::from_utf8_lossy(&output.stdout));
                                }
                            }
                        }
                    }
                } else {
                    if let Ok(command) = Command::new("intercept-build")
                        .args(["make"])
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        if let Ok(output) = command.wait_with_output() {
                            println!("{}", String::from_utf8_lossy(&output.stdout));
                        }
                    } else {
                        panic!("Please install bear or scan-build\n");
                    }
                }
            }
        }
        match Command::new("c2rust-transpile")
            .args(["-e", "-b", "main", "-o", ".", "compile_commands.json"])
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(command) => {
                if let Ok(output) = command.wait_with_output() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            }
            Err(_) => {
                Command::new("cargo")
                    .args(["install", "c2rust-transpile"])
                    .stdout(Stdio::piped())
                    .spawn()
                    .ok();
            }
        }
    }
}
