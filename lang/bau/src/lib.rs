use parser::Parser;
use source::Source;

pub mod error;
pub mod parser;
pub mod source;
pub mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub struct Bau {}

impl Bau {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, input: &str) {
        let source = Source::new(input);
        let items = Parser::new(&source).parse_top_level();
        for item in items.iter() {
            println!("{:?}", item)
        }
    }

    pub fn run_file(&self, path: &str) {
        let file_content = std::fs::read_to_string(path).unwrap();
        self.run(&file_content);
    }
}
