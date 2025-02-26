use std::fmt;
use std::fmt::Formatter;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Run custom build command.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate code.
    Codegen(CodegenArgs),
}

#[derive(Args, Debug)]
pub struct CodegenArgs {
    /// Specify mode for codegen. When not specified, runs all codegen tasks.
    #[arg(value_enum)]
    pub mode: Option<CodegenMode>,
    /// Check and overwrite file contents.
    #[arg(short, long)]
    pub check: bool,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Debug, ValueEnum)]
pub enum CodegenMode {
    /// Run all codegen tasks.
    #[default]
    All,
    /// Run grammar codegen.
    Grammar,
}

impl fmt::Display for CodegenMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CodegenMode::All => write!(f, "all"),
            CodegenMode::Grammar => write!(f, "grammar"),
        }
    }
}
