use crate::error::{BauError, BauResult};
use crate::tokenizer::source::Source;
use crate::tokenizer::token::{Span, Token, TokenKind};
use crate::tokenizer::Tokenizer;
use std::iter::Peekable;

pub mod ast;
pub mod expr;
pub mod item;
pub mod stmt;

pub struct TokenIter<'input> {
    tokenizer: Tokenizer<'input>,
}

impl<'input> TokenIter<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
        }
    }
}

impl<'input> Iterator for TokenIter<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_token = self.tokenizer.next()?;
            if !matches!(next_token.kind, TokenKind::Whitespace) {
                return Some(next_token);
            }
        }
    }
}

pub struct Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    input: &'input str,
    tokens: Peekable<I>,
}

impl<'input> Parser<'input, TokenIter<'input>> {
    pub fn new(source: &'input Source) -> Self {
        Self {
            input: source.text(),
            tokens: TokenIter::new(source.text()).peekable(),
        }
    }
}

impl<'input, I> Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    /// Get the source text of a token.
    pub fn text(&self, token: Token) -> &'input str {
        token.text(&self.input)
    }

    /// Look at the next token's kind without consuming it.
    pub(crate) fn peek(&mut self) -> TokenKind {
        self.peek_token().kind
    }

    pub(crate) fn peek_token(&mut self) -> Token {
        self.tokens
            .peek()
            .unwrap_or(&Token {
                kind: TokenKind::EndOfFile,
                span: Span {
                    start: self.input.len(),
                    end: self.input.len(),
                },
            })
            .clone()
    }

    /// Check if the next token is of a certain kind.
    pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    /// Consume the current token and advance the iterator.
    pub(crate) fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    /// Progress the iterator by one token and check if it is of a certain kind.
    pub(crate) fn consume(&mut self, expected: TokenKind) -> BauResult<()> {
        let current = self.peek();

        if current == TokenKind::Error {
            return Err(self.error(format!("Invalid token: {:?}", current)));
        }

        if current != expected {
            return Err(self.error(format!("Expected {:?}, found {:?}", expected, current)));
        }

        match self.next() {
            Some(_) => Ok(()),
            None => Err(self.error(format!("Expected {:?}, found EOF", expected))),
        }
    }

    fn error(&mut self, message: String) -> BauError {
        BauError::ParserError {
            token: self.peek_token(),
            message,
        }
    }
}
