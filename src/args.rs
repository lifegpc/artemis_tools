use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum RenderType {
    /// Render messages in Markdown format (GitHub Flavored Markdown)
    Markdown,
}

impl ToString for RenderType {
    fn to_string(&self) -> String {
        match self {
            RenderType::Markdown => "markdown".to_string(),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum MessageCmds {
    /// Parse messages from files and print them in debug format
    Test {
        /// AST file to parse
        file: String,
    },
    Render {
        /// AST file want to render
        file: String,
        #[arg(short, long)]
        /// Output file, by default, it print to stdout
        output: Option<String>,
        /// Output format
        #[arg(short, long, default_value_t = RenderType::Markdown)]
        r#type: RenderType,
    },
}

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
    /// Process messages from Artemis Engine
    Message {
        #[command(subcommand)]
        cmd: MessageCmds,
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
    #[arg(global = true, short, long)]
    /// Print backtrace on error
    pub backtrace: bool,
    #[command(subcommand)]
    pub command: Commands,
}
