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
use log::{debug, info, trace, LevelFilter};
use std::{env, fs, io};

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
                "[{} {}] {}",
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
