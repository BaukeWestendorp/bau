use error::BauError;
use interpreter::value::Value;
use parser::Parser;
use source::Source;

pub mod error;
pub mod interpreter;
pub mod parser;
pub mod source;
pub mod tokenizer;
mod typechecker;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bau {}

impl Bau {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, input: &str) -> Result<Option<Value>, Vec<BauError>> {
        let source = Source::new(input);
        match Parser::new(&source).parse_top_level() {
            Ok(items) => {
                let mut typechecker = typechecker::Typechecker::new();
                let checked_items = typechecker.check_items(&items);
                if !typechecker.errors().is_empty() {
                    let errors = typechecker
                        .errors()
                        .iter()
                        .map(|err| BauError::from(err.clone()))
                        .collect();
                    Err(errors)
                } else {
                    let mut interpreter = interpreter::Interpreter::new();
                    match interpreter.run(&checked_items) {
                        Ok(value) => Ok(value),
                        Err(error) => Err(vec![BauError::from(error)]),
                    }
                }
            }
            Err(error) => Err(vec![BauError::from(error)]),
        }
    }

    pub fn run_file(&self, path: &str) -> Result<Option<Value>, Vec<BauError>> {
        let file_content = std::fs::read_to_string(path).unwrap();
        self.run(&file_content)
    }
}
