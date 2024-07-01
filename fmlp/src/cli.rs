use std::path::PathBuf;

use clap::Parser;

/// Fml parser
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub file: PathBuf,

    /// Output lexer tokens
    #[arg(short, long)]
    pub tokens: bool,

    /// Output AST
    #[arg(short, long)]
    pub ast: bool,
}
