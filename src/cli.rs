use clap::Parser;
use std::path::PathBuf;
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg (short = 'r', long = "skip_c2rust", action = clap :: ArgAction :: SetTrue, help = "skip c2rust code translation")]
    pub skip_c2rust: bool,
    #[arg (short = 't', long = "skip_txl", action = clap :: ArgAction :: SetTrue, help = "skip txl rules application")]
    pub skip_txl_rules: bool,
    #[arg (short = 'c', long = "skip-crown", action = clap :: ArgAction :: SetTrue, help = "skip running crown")]
    pub skip_crown: bool,
    #[arg(
        short = 'a',
        long = "add_txl",
        help = "run customized txl rule after crusts completed"
    )]
    pub custom_txl: Option<PathBuf>,
}
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
