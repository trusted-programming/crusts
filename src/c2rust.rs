use crate::constants::CONFIG;
use crate::utils::{command_exists, is_file_with_ext, path_exists, run_command};
use jwalk::WalkDir;
use log::info;
use std::process::{Command, Stdio};

pub fn run() {
    if !command_exists("c2rust") {
        panic!("no c2rust command found")
    }

    let cargo_toml_exists = path_exists("Cargo.toml");
    let compile_commands_exists = path_exists("compile_commands.json");
    let makefile_exists = path_exists("Makefile") || path_exists("makefile");
    let configure_exists = path_exists("configure");
    let configure_ac_exists = path_exists("configure.ac");

    if !cargo_toml_exists && !compile_commands_exists {
        if !makefile_exists && !configure_exists && !configure_ac_exists {
            create_makefile();
        }
        if !makefile_exists && !configure_exists && configure_ac_exists {
            run_command("autoreconf", &["-fi"]);
        }
        if !makefile_exists && configure_exists {
            run_command("./configure", &[]);
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

    let obj = c_files
        .iter()
        .map(|c_file| c_file.replace(".c", ".o"))
        .collect::<Vec<String>>()
        .join(" \\\n");

    std::fs::write(
        "Makefile",
        format!(
            "main: {obj}\n\tgcc -o main {obj}\n\n.c.o: \n\tgcc -c $<\n\n.cpp.o: \n\tg++ -c $<\n\nclean::\n\trm -rf Makefile main c2rust crusts compile_commands.json Cargo.lock target",
        ),
    )
    .expect("failed to write Makefile");
}

fn run_bear_or_intercept_build() {
    info!("found makefile");
    let run_bear_command = |bear_args: &[&str]| {
        Command::new(CONFIG.bear)
            .args(bear_args)
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|command| command.wait_with_output())
            .map(|output| info!("{}", String::from_utf8_lossy(&output.stdout)))
    };

    let bear_result = Command::new(CONFIG.bear)
        .args(["--version"])
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|bear_version| bear_version.wait_with_output())
        .and_then(|output| {
            info!("running bear");
            let s = String::from_utf8_lossy(&output.stdout);
            if s.contains("bear 2.4.3") {
                run_bear_command(&["make"])
            } else {
                run_bear_command(CONFIG.bear_args)
            }
        });
    if bear_result.is_err() {
        run_command("intercept-build", &["make"]);
    }
}

fn run_c2rust_transpile() {
    info!("starting c2rust transpile");
    if command_exists("c2rust-transpile") {
        run_command(
            "c2rust-transpile",
            &["-e", "-b", "main", "-o", ".", "compile_commands.json"],
        )
    } else {
        run_command("cargo", &["install", "c2rust-transpile"]);
        run_c2rust_transpile();
    }
}
