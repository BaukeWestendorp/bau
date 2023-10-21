use crate::error::BauResult;
use crate::parser::source::Source;
use clap::Parser;

pub mod builtins;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod tokenizer;
pub mod typechecker;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_path: String,
}

fn main() {
    let args = Args::parse();

    let source = match std::fs::read_to_string(&args.file_path) {
        Ok(text) => Source::new(text, args.file_path),
        Err(_) => {
            eprintln!("Could not find file `{}`", args.file_path);
            std::process::exit(1);
        }
    };

    match run(&source) {
        Ok(_) => {}
        Err(error) => error.log(&source),
    }
}

fn run(source: &Source) -> BauResult<()> {
    let mut parser = parser::Parser::new(source);
    let top_level = parser.parse_top_level()?;

    let mut typechecker = typechecker::Typechecker::new();
    typechecker.check_top_level(&top_level)?;

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.evaluate_top_level(top_level)?;

    interpreter.execute_main()?;

    Ok(())
}
