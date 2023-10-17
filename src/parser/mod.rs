use crate::parser::tokenizer::{Token, Tokenizer};

mod tokenizer;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            tokens: Tokenizer::new(input).tokenize(),
        }
    }

    pub fn parse(&mut self) {
        for token in &self.tokens {
            println!("{:?}", token);
        }
    }
}
