use crate::error::BauResult;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::parser::source::Source;
use crate::parser::Parser;
use crate::typechecker::Typechecker;

pub mod builtins;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod tokenizer;
pub mod typechecker;
pub mod types;

pub struct Bau {}

impl Bau {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, source: &Source) -> BauResult<Option<Value>> {
        let prelude_source = Source::from(include_str!("prelude.bau"));
        let mut prelude_parser = Parser::new(&prelude_source);
        let prelude_top_level = prelude_parser.parse_top_level();

        let mut source_parser = Parser::new(source);
        let source_top_level = source_parser.parse_top_level()?;

        let mut top_level = prelude_top_level?;
        top_level.extend(source_top_level);

        let mut typechecker = Typechecker::new();
        typechecker.check_top_level(&top_level)?;

        let mut interpreter = Interpreter::new();
        interpreter.register_functions(&typechecker);
        interpreter.execute_main()
    }
}
