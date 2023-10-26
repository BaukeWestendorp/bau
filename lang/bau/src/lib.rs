use source::Source;

use crate::tokenizer::Tokenizer;

pub mod source;
pub mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub struct Bau {}

impl Bau {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, source: &str) {
        let source = Source::new(source);
        let tokens = Tokenizer::new(source.text()).tokenize();

        for token in tokens.iter() {
            println!(
                "{:?} {:?}",
                token,
                source.text()[token.span.start..token.span.end].to_string()
            );
        }
    }

    pub fn run_file(&self, path: &str) {
        let file_content = std::fs::read_to_string(path).unwrap();
        self.run(&file_content);
    }
}
