use clap::Parser;
use std::path::PathBuf;
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg (short = 'r', long = "skip_c2rust", action = clap :: ArgAction :: SetTrue, help = "skip c2rust code translation")]
    pub skip_c2rust: bool,
    #[arg (short = 't', long = "skip_txl", action = clap :: ArgAction :: SetTrue, help = "skip txl rules application")]
    pub skip_txl_rules: bool,
    #[arg (short = 'c', long = "skip_crown", action = clap :: ArgAction :: SetTrue, help = "skip running crown")]
    pub skip_crown: bool,
    #[arg (short = 'i', long = "skip_inlay_hints", action = clap :: ArgAction :: SetTrue, help = "skip running inlay hints")]
    pub skip_inlay_hints: bool,
    #[arg(
        short = 'x',
        long = "add_txl",
        help = "run customized txl rule after crusts completed"
    )]
    pub custom_txl: Option<PathBuf>,
    #[arg (short = 'm', long = "metrics", action = clap :: ArgAction :: SetTrue, help = "save intermediate artifacts and metrics in a folder")]
    pub metrics: bool,
    #[arg (short = 'a', long = "auto_curs", action = clap :: ArgAction :: SetTrue, help = "run auto_curs which will remove all unsafe based on curs suggestions and the cargo check will check which unsafe is compiling and readd the non compiling ones")]
    pub auto_curs: bool,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
