// mod auto_curs;
mod c2rust;
mod cli;
mod constants;
// mod crown;
mod crusts;
mod metrics;
mod utils;

use clap::Parser;
use constants::VERBOSITY;
use fern::colors::{Color, ColoredLevelConfig};
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
    // if !cli.skip_crown {
    //     crown::run();
    //     if cli.metrics {
    //         metrics::run("crown");
    //     }
    // }
    // if cli.auto_curs {
    //     auto_curs::run();
    //     if cli.metrics {
    //         metrics::run("auto_curs");
    //     }
    // }
}

fn setup_logging(verbosity: LevelFilter, metrics: bool) -> Result<(), fern::InitError> {
    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    // configure colors for the name of the level.
    // since almost all of them are the same as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line.info(Color::Green);

    let mut config = fern::Dispatch::new()
        .level(verbosity)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[ {level} {target} {color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
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
