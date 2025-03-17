use clap::Parser;

mod args;
mod dump;
mod parser;
mod types;
mod utils;

fn fmt_file(f: &str, args: &args::Arg, sort_blocks: bool) -> anyhow::Result<()> {
    let content = utils::read_file(f)?;
    let mut parser = parser::Parser::new(&content);
    let mut ast = parser.parse()?;
    if sort_blocks {
        ast.sort_blocks();
    }
    let f = utils::write_file(f)?;
    let f = std::io::BufWriter::new(f);
    let mut dumper = dump::Dumper::new(f);
    if args.no_indent {
        dumper.set_no_indent();
    } else {
        dumper.set_indent(args.indent.unwrap_or(4));
    }
    if let Some(max_line_width) = args.max_line_width {
        dumper.set_max_line_width(max_line_width);
    }
    dumper.dump(&ast)?;
    Ok(())
}

fn main() {
    let args = args::Arg::parse();
    match &args.command {
        args::Commands::TestParse { file } => {
            let content = utils::read_file(file).unwrap();
            let mut parser = parser::Parser::new(&content);
            let ast = parser.parse().unwrap();
            println!("{:#?}", ast);
        }
        args::Commands::Fmt { files, sort_blocks } => {
            let files = utils::collect_ast_files(files, args.recursive).unwrap();
            let mut error = 0;
            for f in files.iter() {
                if let Err(e) = fmt_file(f, &args, *sort_blocks) {
                    eprintln!("Error formatting file {}: {}", f, e);
                    error += 1;
                }
            }
            eprintln!("Formatted {} files", files.len() - error);
            if error != 0 {
                eprintln!("Failed to format {} files", error);
                std::process::exit(1);
            }
        }
    }
}
