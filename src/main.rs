use crate::error::BauResult;
use crate::tokenizer::source::Source;
use clap::Parser;

pub mod error;
pub mod parser;
pub mod tokenizer;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_path: String,
}

fn main() -> BauResult<()> {
    let args = Args::parse();

    let source = match std::fs::read_to_string(&args.file_path) {
        Ok(source) => Source::from(source),
        Err(_) => {
            eprintln!("Could not find file '{}'", args.file_path);
            std::process::exit(1);
        }
    };

    let mut parser = parser::Parser::new(&source);
    let items = parser.parse_top_level();

    match items {
        Err(error) => error.log(args.file_path.as_str(), &source),
        Ok(items) => {
            for item in items {
                println!("{:#?}", item);
            }
        }
    }

    Ok(())
}
