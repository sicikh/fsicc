mod cli;
mod codegen;

use std::env;
use std::path::PathBuf;

use clap::Parser;
use xshell::Shell;

use self::cli::Cli;
use crate::cli::Command;

fn main() -> anyhow::Result<()> {
    let config = Cli::parse();

    let sh = &Shell::new()?;
    sh.change_dir(project_root());

    match config.command {
        Command::Codegen(args) => codegen::run(args, sh),
    }
}

fn project_root() -> PathBuf {
    let dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());

    PathBuf::from(dir).parent().unwrap().to_owned()
}