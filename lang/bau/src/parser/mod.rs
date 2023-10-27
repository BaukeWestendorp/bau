use crate::source::Source;
use crate::tokenizer::{Token, Tokenizer};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedItem {
    Function(ParsedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionItem {
    pub name: String,
    pub arguments: ParsedFunctionArgument,
    pub return_type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionArgument {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parser<'source> {
    source: &'source Source<'source>,
    tokens: Vec<Token>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source Source) -> Self {
        let tokens = Tokenizer::new(source.text()).tokenize();

        Self { source, tokens }
    }

    pub fn parse(&mut self) -> Vec<ParsedItem> {
        vec![]
    }
}
