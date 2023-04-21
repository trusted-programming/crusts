use crate::utils::command_exists;
use crate::utils::is_file_with_ext;
use jwalk::WalkDir;
use log::info;
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
    if !command_exists("c2rust") {
        panic!("no c2rust command found")
    }
    let cargo_toml_exists = std::path::Path::new("Cargo.toml").exists();
    let compile_commands_exists = std::path::Path::new("compile_commands.json").exists();
    let makefile_exists =
        std::path::Path::new("Makefile").exists() || std::path::Path::new("makefile").exists();
    let configure_exists = std::path::Path::new("configure").exists();
    let configure_ac_exists = std::path::Path::new("configure.ac").exists();

    if !cargo_toml_exists && !compile_commands_exists {
        if !makefile_exists && !configure_exists && !configure_ac_exists {
            create_makefile();
        }
        if !makefile_exists && !configure_exists && configure_ac_exists {
            run_autoreconf();
        }
        if !makefile_exists && configure_exists {
            run_configure();
        }
        if makefile_exists {
            run_bear_or_intercept_build();
        }
        run_c2rust_transpile()
    }
}

fn create_makefile() {
    info!("No  makefile, No configure and no configure.ac found, generating makefile.");
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

fn run_autoreconf() {
    info!("found configure.ac! running autoreconf");
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

fn run_configure() {
    info!("found configure! running configure");
    if let Ok(command) = Command::new("./configure").stdout(Stdio::piped()).spawn() {
        if let Ok(output) = command.wait_with_output() {
            println!("{:?}", output);
        }
    }
}

fn run_bear_or_intercept_build() {
    info!("found makefile");
    if let Ok(bear_version) = Command::new(BEAR)
        .args(["--version"])
        .stdout(Stdio::piped())
        .spawn()
    {
        if let Ok(output) = bear_version.wait_with_output() {
            info!("running bear");
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
            } else if let Ok(command) = Command::new(BEAR)
                .args(BEAR_ARGS)
                .stdout(Stdio::piped())
                .spawn()
            {
                if let Ok(output) = command.wait_with_output() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            }
        }
    } else if let Ok(command) = Command::new("intercept-build")
        .args(["make"])
        .stdout(Stdio::piped())
        .spawn()
    {
        info!("running intercept-build");
        if let Ok(output) = command.wait_with_output() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        panic!("Please install bear or scan-build\n");
    }
}

fn run_c2rust_transpile() {
    info!("starting c2rust transpile");
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
