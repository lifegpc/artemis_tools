use clap::Parser;
use std::io::Write;

mod args;
mod dump;
mod galtransl;
mod parser;
mod render;
mod types;
mod utils;

fn fmt_file(f: &str, args: &args::Arg, sort_blocks: bool) -> anyhow::Result<()> {
    let content = utils::read_file(f)?;
    let parser = parser::Parser::new(&content);
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

fn to_json(f: &str, output: &str, lang: Option<String>) -> anyhow::Result<bool> {
    let content = utils::read_file(f)?;
    let parser = parser::Parser::new(&content);
    let ast = parser.parse()?;
    let output_json = ast.to_galtransl_json(lang)?;
    if output_json.is_empty() {
        return Ok(false);
    }
    let f = utils::write_file(output)?;
    let mut f = std::io::BufWriter::new(f);
    f.write(output_json.as_bytes())?;
    Ok(true)
}

fn main() {
    let args = args::Arg::parse();
    if args.backtrace {
        unsafe { std::env::set_var("RUST_LIB_BACKTRACE", "1") };
    }
    match &args.command {
        args::Commands::TestParse { file } => {
            let content = utils::read_file(file).unwrap();
            let parser = parser::Parser::new(&content);
            let ast = parser.parse().unwrap();
            println!("{:#?}", ast);
        }
        args::Commands::Fmt { files, sort_blocks } => {
            let files = utils::collect_ast_files(files, args.recursive).unwrap();
            let mut error = 0;
            for f in files.iter() {
                if let Err(e) = fmt_file(f, &args, *sort_blocks) {
                    eprintln!("Error formatting file {}: {}", f, e);
                    if args.backtrace {
                        eprintln!("{}", e.backtrace());
                    }
                    error += 1;
                }
            }
            eprintln!("Formatted {} files", files.len() - error);
            if error != 0 {
                eprintln!("Failed to format {} files", error);
                std::process::exit(1);
            }
        }
        args::Commands::Message { cmd } => match cmd {
            args::MessageCmds::Test { file } => {
                let content = utils::read_file(file).unwrap();
                let parser = parser::Parser::new(&content);
                let ast = parser.parse().unwrap();
                println!("{:#?}", ast.get_messages().unwrap());
            }
            args::MessageCmds::Render {
                file,
                output,
                r#type,
            } => match r#type {
                args::RenderType::Markdown => {
                    let content = utils::read_file(file).unwrap();
                    let parser = parser::Parser::new(&content);
                    let ast = parser.parse().unwrap();
                    let f = utils::write_file(output.as_deref().unwrap_or("-")).unwrap();
                    let f = std::io::BufWriter::new(f);
                    let renderer = render::MarkdownRenderer::new(f);
                    renderer.render(&ast.get_messages().unwrap()).unwrap();
                }
            },
            args::MessageCmds::ToJson { file, output, lang } => {
                let files = utils::collect_ast_files(file, args.recursive).unwrap();
                if files.len() == 1 {
                    let content = utils::read_file(&files[0]).unwrap();
                    let parser = parser::Parser::new(&content);
                    let ast = parser.parse().unwrap();
                    let output_json = ast.to_galtransl_json(lang.clone()).unwrap();
                    if output_json.is_empty() {
                        eprintln!("Skipped empty file {}", files[0]);
                        std::process::exit(0);
                    }
                    let f = utils::write_file(output).unwrap();
                    let mut f = std::io::BufWriter::new(f);
                    f.write(output_json.as_bytes()).unwrap();
                } else {
                    let mut error = 0;
                    let mut skiped = 0;
                    for f in files.iter() {
                        let basename = match std::path::Path::new(f).file_name() {
                            Some(b) => b.to_string_lossy().to_string(),
                            None => {
                                eprintln!("Error: {} is not a valid file", f);
                                error += 1;
                                continue;
                            }
                        };
                        let mut output_file = std::path::PathBuf::from(output).join(basename);
                        output_file.set_extension("json");
                        let output_file = output_file.to_string_lossy().to_string();
                        match to_json(f, &output_file, lang.clone()) {
                            Ok(s) => {
                                if !s {
                                    skiped += 1;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error converting file {}: {}", f, e);
                                if args.backtrace {
                                    eprintln!("{}", e.backtrace());
                                }
                                error += 1;
                            }
                        }
                    }
                    eprintln!("Converted {} files", files.len() - error - skiped);
                    if skiped != 0 {
                        eprintln!("Skipped {} empty files", skiped);
                    }
                    if error != 0 {
                        eprintln!("Failed to convert {} files", error);
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}
