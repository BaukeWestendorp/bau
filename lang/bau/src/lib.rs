use parser::Parser;
use source::Source;

pub mod error;
pub mod interpreter;
pub mod parser;
pub mod source;
pub mod tokenizer;
mod typechecker;

#[derive(Debug, Clone, PartialEq)]
pub struct Bau {}

impl Bau {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, input: &str) {
        let source = Source::new(input);
        match Parser::new(&source).parse_top_level() {
            Ok(items) => {
                let mut typechecker = typechecker::Typechecker::new();
                let checked_items = typechecker.check_items(&items);
                if !typechecker.errors().is_empty() {
                    for error in typechecker.errors() {
                        error.print(&source);
                    }
                    std::process::exit(1);
                } else {
                    let mut interpreter = interpreter::Interpreter::new();
                    match interpreter.run(&checked_items) {
                        Ok(value) => {
                            eprintln!("{:?}", value);
                        }
                        Err(error) => {
                            error.print(&source);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(error) => {
                error.print(&source);
                std::process::exit(1);
            }
        };
    }

    pub fn run_file(&self, path: &str) {
        let file_content = std::fs::read_to_string(path).unwrap();
        self.run(&file_content);
    }
}
