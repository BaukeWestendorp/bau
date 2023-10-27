use crate::error::{BauError, BauResult};
use crate::source::Source;
use crate::tokenizer::token::TokenKind;
use crate::tokenizer::{Token, Tokenizer};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedItem {
    Function(ParsedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionItem {
    pub name: String,
    pub arguments: Vec<ParsedFunctionArgument>,
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
    cursor: usize,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source Source) -> Self {
        let mut tokens = Tokenizer::new(source.text()).tokenize();
        preprocess_tokens(&mut tokens);
        Self {
            source,
            tokens,
            cursor: 0,
        }
    }

    pub fn parse_top_level(&mut self) -> Vec<ParsedItem> {
        let mut items = vec![];
        while let Some(item) = self.parse_item() {
            items.push(item);
        }
        items
    }

    fn parse_item(&mut self) -> Option<ParsedItem> {
        match self.peek_kind() {
            Ok(TokenKind::Fn) => self.parse_function_item().map(|f| ParsedItem::Function(f)),
            Err(err) => {
                self.error(err);
                None
            }
            _ => None,
        }
    }

    fn parse_function_item(&mut self) -> Option<ParsedFunctionItem> {
        self.consume_specific(TokenKind::Fn)?;

        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&name_ident);

        self.consume_specific(TokenKind::ParenOpen)?;
        let arguments = self.parse_function_arguments()?;
        self.consume_specific(TokenKind::ParenClose)?;

        self.consume_specific(TokenKind::Arrow)?;
        let return_type_ident = self.consume_specific(TokenKind::Identifier)?;
        let return_type_name = self.text(&return_type_ident);

        Some(ParsedFunctionItem {
            name,
            arguments,
            return_type_name,
        })
    }

    fn parse_function_arguments(&mut self) -> Option<Vec<ParsedFunctionArgument>> {
        let mut arguments = vec![];
        while self.peek_kind() != Ok(TokenKind::ParenClose) {
            let argument = self.parse_function_argument()?;
            arguments.push(argument);
            if self.peek_kind() == Ok(TokenKind::Comma) {
                self.consume_specific(TokenKind::Comma)?;
            }
        }
        Some(arguments)
    }

    fn parse_function_argument(&mut self) -> Option<ParsedFunctionArgument> {
        let type_ident = self.consume_specific(TokenKind::Identifier)?;
        let type_name = self.text(&type_ident);

        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&name_ident);

        Some(ParsedFunctionArgument { name, type_name })
    }

    fn peek(&self) -> BauResult<&Token> {
        self.tokens
            .iter()
            .nth(self.cursor)
            .map(Ok)
            .unwrap_or(Err(BauError::UnexpectedEndOfFile(
                self.tokens
                    .last()
                    .cloned()
                    .expect("input should have at least one character"),
            )))
    }

    fn peek_kind(&self) -> BauResult<TokenKind> {
        self.peek().map(|t| t.kind)
    }

    fn consume(&mut self) -> Option<Token> {
        let token = self.peek().cloned();
        match token {
            Err(error) => {
                self.error(error);
                return None;
            }
            Ok(token) => {
                self.cursor += 1;
                return Some(token);
            }
        }
    }

    fn consume_specific(&mut self, expected: TokenKind) -> Option<Token> {
        let token = self.consume()?.clone();
        if !token.is(expected) {
            self.error(BauError::UnexpectedToken(token.clone()));
            return None;
        }
        Some(token)
    }

    fn text(&self, token: &Token) -> String {
        self.source.text()[token.span.start..token.span.end].to_string()
    }

    fn error(&self, error: BauError) {
        eprintln!("{}", error);
    }
}

pub fn preprocess_tokens(tokens: &mut Vec<Token>) {
    tokens.retain(|token| !token.is(TokenKind::Whitespace));
}
