use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Parse AST file and print it in debug format
    TestParse {
        /// AST file to parse
        file: String,
    },
    /// Format AST file
    Fmt {
        /// AST file to format or directory to search for .ast files.
        /// If empty, use current working directory
        files: Vec<String>,
        #[arg(short, long)]
        /// Sort blocks in AST file
        sort_blocks: bool,
    },
}

/// Tools to process Artemis Engine AST files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arg {
    /// Indentation level
    #[arg(global = true, short, long)]
    pub indent: Option<usize>,
    /// Disable indentation
    #[arg(global = true, short, long)]
    pub no_indent: bool,
    /// Maximum line width for formatting
    #[arg(global = true, short, long)]
    pub max_line_width: Option<usize>,
    #[arg(global = true, short = 'R', long)]
    /// Recursively search subdirectories
    pub recursive: bool,
    #[command(subcommand)]
    pub command: Commands,
}
