use crate::tokenizer::source::Source;
use clap::Parser;

pub mod error;
pub mod interpreter;
pub mod parser;
pub mod tokenizer;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_path: String,
}

fn main() {
    let args = Args::parse();

    let source = match std::fs::read_to_string(&args.file_path) {
        Ok(source) => Source::from(source),
        Err(_) => {
            eprintln!("Could not find file '{}'", args.file_path);
            std::process::exit(1);
        }
    };

    let mut parser = parser::Parser::new(&source);
    let top_level = parser.parse_top_level();

    match top_level {
        Err(error) => error.log(args.file_path.as_str(), &source),
        Ok(top_level) => {
            let mut interpreter = interpreter::Interpreter::new();
            match interpreter.evaluate_top_level(top_level) {
                Err(error) => error.log(args.file_path.as_str(), &source),
                Ok(_) => match interpreter.execute_main() {
                    Err(error) => error.log(args.file_path.as_str(), &source),
                    Ok(_) => {}
                },
            }
        }
    }
}
