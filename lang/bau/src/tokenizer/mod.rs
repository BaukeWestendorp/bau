use crate::source::Source;

pub use token::Token;

pub mod token;

#[derive(Debug, Clone, PartialEq)]
pub struct Tokenizer<'source> {
    source: &'source Source<'source>,
    tokens: Vec<Token>,
    cursor: u32,
    line: u32,
    column: u32,
}

impl<'source> Tokenizer<'source> {
    pub fn new(source: &'source Source) -> Self {
        Self {
            source,
            tokens: vec![],
            cursor: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        todo!()
    }
}
