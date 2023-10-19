use crate::error::{BauError, BauResult};
use crate::parser::source::Source;
use crate::tokenizer::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub mod ast;
pub mod expr;
pub mod item;
pub mod source;
pub mod stmt;

pub struct Parser<'source> {
    source: &'source Source,
    tokens: Vec<Token>,
    cursor: usize,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source Source) -> Self {
        Self {
            source,
            tokens: Tokenizer::new(source.text())
                .tokenize()
                .iter()
                .filter(|t| t.kind != TokenKind::Whitespace)
                .cloned()
                .collect(),
            cursor: 0,
        }
    }
}

impl<'source> Parser<'source> {
    /// Get the source text of a token.
    pub fn text(&self, token: Token) -> &'source str {
        token.text(&self.source.text())
    }

    /// Look at the next token's kind without consuming it.
    pub(crate) fn peek_kind(&mut self) -> TokenKind {
        self.peek().kind
    }

    /// Look at the token's kind after the next token without consuming it.
    pub(crate) fn peek_next_kind(&mut self) -> TokenKind {
        self.peek_next().kind
    }

    /// Look at the next token without consuming it.
    pub(crate) fn peek(&mut self) -> Token {
        self.tokens
            .get(self.cursor)
            .unwrap_or(&self.source.eof_token())
            .clone()
    }

    /// Look at the token after the next token without consuming it.
    pub(crate) fn peek_next(&mut self) -> Token {
        self.tokens
            .get(self.cursor + 1)
            .unwrap_or(&self.source.eof_token())
            .clone()
    }

    /// Check if the next token is of a certain kind.
    pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    /// Consume the current token and advance the iterator.
    pub(crate) fn consume(&mut self) -> BauResult<Token> {
        let token = self.peek();
        self.cursor += 1;
        match token.kind {
            TokenKind::EndOfFile => Err(self.error("Unexpected end of file".to_string())),
            _ => Ok(token),
        }
    }

    /// Progress the iterator by one token and check if it is of a certain kind.
    pub(crate) fn consume_specific(&mut self, expected: TokenKind) -> BauResult<Token> {
        let current = self.peek();

        if current.kind == TokenKind::Error {
            return Err(self.error(format!("Invalid token: {:?}", current.kind)));
        }

        if current.kind != expected {
            return Err(self.error(format!("Expected {:?}, found {:?}", expected, current.kind)));
        }

        match self.consume() {
            Ok(token) => Ok(token),
            Err(_) => Err(self.error(format!("Unexpected EOF. Expected {:?}", expected))),
        }
    }

    fn error(&mut self, message: String) -> BauError {
        BauError::ParserError {
            token: self.peek(),
            message,
        }
    }
}
