use crate::tokenizer::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;
use ast::Expr;
use std::iter::Peekable;

pub mod ast;
pub mod expr;
pub mod hierarchy;
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
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            tokens: TokenIter::new(input).peekable(),
        }
    }

    pub fn parse(&mut self) -> Expr {
        let expr = self.parse_expression();
        expr
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

    /// Look at the next token without consuming it.
    pub(crate) fn peek(&mut self) -> TokenKind {
        self.tokens
            .peek()
            .map(|token| token.kind)
            .unwrap_or(TokenKind::EndOfFile)
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
    pub(crate) fn consume(&mut self, expected: TokenKind) {
        let token = self
            .next()
            .expect(&format!("Expected {:?}, found EOF", expected));
        assert_eq!(
            token.kind, expected,
            "Expected {:?}, found {:?}",
            expected, token.kind
        );
    }
}
