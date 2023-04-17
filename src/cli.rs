use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short = 'r', long = "c2rust", action = clap::ArgAction::SetTrue, help = "c2rust only without further refactoring")]
    pub stop_refactoring: bool,
    #[arg(
        short = 't',
        long = "txl",
        help = "run customized txl rule after crusts completed"
    )]
    pub custom_txl: Option<PathBuf>,
}
