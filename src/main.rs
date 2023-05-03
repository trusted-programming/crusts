mod auto_curs;
mod c2rust;
mod cli;
mod constants;
mod crown;
mod crusts;
mod metrics;
mod utils;
use clap::Parser;
use constants::VERBOSITY;
use humantime;
use log::{debug, info, trace, LevelFilter};
use std::{env, fs, io, time::SystemTime};

// TODO: extract the walk so doesn't have to be run on each step
fn main() {
    let cli = cli::Cli::parse();
    setup_logging(VERBOSITY, cli.metrics).expect("failed to initialize logging.");

    debug!("DEBUG output enabled.");
    trace!("TRACE output enabled.");
    info!("starting up");
    if cli.metrics {
        metrics::run("original");
    }

    if !cli.skip_c2rust {
        c2rust::run();
        if cli.metrics {
            metrics::run("c2rust");
        }
    }
    if !cli.skip_txl_rules {
        crusts::run(cli.custom_txl);
        if cli.metrics {
            metrics::run("txl");
        }
    }
    if !cli.skip_crown {
        crown::run();
        if cli.metrics {
            metrics::run("crown");
        }
    }
    if cli.auto_curs {
        auto_curs::run();
        if cli.metrics {
            metrics::run("auto_curs");
        }
    }
}

fn setup_logging(verbosity: LevelFilter, metrics: bool) -> Result<(), fern::InitError> {
    let mut config = fern::Dispatch::new()
        .level(verbosity)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(io::stdout());

    if metrics {
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
        let log_file_path = metrics_proj_dir.join("program.log");
        config = config.chain(fern::log_file(log_file_path)?);
    }
    config.apply()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    #[test]
    #[serial]
    fn test_crusts() {
        let dir = std::path::Path::new("test_crusts");
        if dir.exists() {
            std::fs::remove_dir_all(dir).unwrap();
        }
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("main.c"),
            r#"
#include <stdio.h>
int main() {
    printf("Hello, world!\n");
    return 0;
}
"#,
        )
        .unwrap();
        std :: fs :: write (dir.join ("Makefile"), "main: main.c\n\tgcc -o main main.c\n\nclean::\n\trm -rf main compile_commands.json src Cargo.toml *.rs rust-toolchain rust-toolchain.toml Cargo.lock target").unwrap ();
        std::env::set_current_dir(dir).unwrap();
        c2rust::run();
        crusts::run(None);
        crown::run();
        std::env::set_current_dir(std::env::current_dir().unwrap().parent().unwrap()).unwrap();
        let s = std::fs::read_to_string(dir.join("src/main.rs")).unwrap();
        insta :: assert_snapshot! (s, @
r###"
        #![allow(
            dead_code,
            mutable_transmutes,
            non_camel_case_types,
            non_snake_case,
            non_upper_case_globals,
            unused_assignments,
            unused_mut
        )]
        use c2rust_out::*;
        extern "C" {}
        fn main_0() -> i32 {
            print!("Hello, world!\n");
            return 0;
        }

        pub fn main() {
            ::std::process::exit(main_0() as i32);
        }
        "###
        );
    }

    #[test]
    #[serial]
    fn test_unsafe() {
        let dir = std::path::Path::new("test_unsafe");
        if dir.exists() {
            std::fs::remove_dir_all(dir).unwrap();
        }
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("main.rs"),
            r#"
    use libc;
    extern "C" {
        fn realloc(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;
    }
    #[no_mangle]
    pub unsafe extern "C" fn add_value(mut p: *mut tvm_program_t, val: i32) -> *mut i32 {
            (*p).values = realloc(
                (*p).values as *mut libc::c_void,
                (::core::mem::size_of::<*mut i32>() as u64)
                    .wrapping_mul(((*p).num_values + 1i32) as u64),
            ) as *mut *mut i32;
            let ref mut fresh7 = *((*p).values).offset((*p).num_values as isize);
            *fresh7 = calloc(1, ::core::mem::size_of::<i32>() as u64) as *mut i32;
            **((*p).values).offset((*p).num_values as isize) = val;
            let fresh8 = (*p).num_values;
            (*p).num_values = (*p).num_values + 1;
            return *((*p).values).offset(fresh8 as isize);
    }
    "#,
        )
        .unwrap();
        std::env::set_current_dir(dir).unwrap();
        crusts::run(None);
        crown::run();
        std::env::set_current_dir(std::env::current_dir().unwrap().parent().unwrap()).unwrap();
        if let Ok(s) = std::fs::read_to_string(dir.join("main.rs")) {
            insta :: assert_snapshot! (s, @
r###"
            use libc;
            extern "C" {
                fn realloc(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;
            }
            #[no_mangle]
            pub extern "C" fn add_value(mut p: *mut tvm_program_t, val: i32) -> *mut i32 {
                unsafe {
                    (*p).values = realloc(
                        (*p).values as *mut libc::c_void,
                        (::core::mem::size_of::<*mut i32>() as u64)
                            .wrapping_mul(((*p).num_values + 1i32) as u64),
                    ) as *mut *mut i32;
                    *((*p).values).offset((*p).num_values as isize) = calloc(1, ::core::mem::size_of::<i32>() as u64) as *mut i32;
                    **((*p).values).offset((*p).num_values as isize) = val;
                    let fresh8 = (*p).num_values;
                    (*p).num_values = (*p).num_values + 1;
                    return *((*p).values).offset(fresh8 as isize);
                }
            }
            "###
            );
        }
    }

    #[test]
    #[serial]
    fn test_stdio() {
        let dir = std::path::Path::new("test_stdio");
        if dir.exists() {
            std::fs::remove_dir_all(dir).unwrap();
        }
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("main.c"),
            r#"
    #include <stdio.h>
    int main() {
        printf("\n  \e[32m\u2713 \e[90mok\e[0m\n\n");
        printf(" %02x", 0);
        return 0;
    }
    "#,
        )
        .unwrap();
        std::env::set_current_dir(dir).unwrap();
        c2rust::run();
        std::env::set_current_dir(std::env::current_dir().unwrap().parent().unwrap()).unwrap();
        if let Ok(s) = std::fs::read_to_string(dir.join("src/main.rs")) {
            insta :: assert_snapshot! (s, @
r###"
            #![allow(
                dead_code,
                mutable_transmutes,
                non_camel_case_types,
                non_snake_case,
                non_upper_case_globals,
                unused_assignments,
                unused_mut
            )]
            use c2rust_out::*;
            extern "C" {}
            fn main_0() -> i32 {
                print!("\n  [32m✓ [90mok[0m");
                print!(" {:02x},9999");
                return 0;
            }

            pub fn main() {
                ::std::process::exit(main_0() as i32);
            }
            "###
            );
        }
    }
}
