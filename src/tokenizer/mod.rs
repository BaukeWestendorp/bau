use crate::tokenizer::rules::Rule;
use crate::tokenizer::token::{Span, Token, TokenKind};

pub mod rules;
pub mod token;

#[derive(Debug, Clone)]
pub struct Tokenizer<'input> {
    input: &'input str,
    cursor: usize,
    eof: bool,
    rules: Vec<Rule>,
}

impl<'input> Tokenizer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            cursor: 0,
            eof: false,
            rules: rules::get_rules(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        self.collect()
    }

    fn next_token(&mut self, input: &str) -> Token {
        self.consume_token(input)
            .unwrap_or_else(|| self.invalid_token(input))
    }

    fn consume_token(&mut self, input: &str) -> Option<Token> {
        let next = input.chars().next().unwrap();
        let (len, kind) = if next.is_whitespace() {
            let len = input
                .char_indices()
                .take_while(|(_, c)| c.is_whitespace())
                .last()
                .expect("At least one whitespace char should exist")
                .0
                + 1;
            (len, TokenKind::Whitespace)
        } else if let Some(punc) = self.consume_punctuation(next) {
            (1, punc)
        } else {
            self.rules
                .iter()
                // `max_by_key` returns the last element if multiple
                // rules match, but we want earlier rules to "win"
                // against later ones
                .rev()
                .filter_map(|rule| Some(((rule.matches)(input)?, rule.kind)))
                .max_by_key(|&(len, _)| len)?
        };

        let start = self.cursor;
        self.cursor += len;
        Some(Token {
            kind,
            span: Span {
                start,
                end: start + len,
            },
        })
    }

    fn consume_punctuation(&self, char: char) -> Option<TokenKind> {
        match char {
            '(' => Some(TokenKind::ParenOpen),
            ')' => Some(TokenKind::ParenClose),
            '{' => Some(TokenKind::BraceOpen),
            '}' => Some(TokenKind::BraceClose),
            '[' => Some(TokenKind::SquareOpen),
            ']' => Some(TokenKind::SquareClose),
            ';' => Some(TokenKind::Semicolon),
            ',' => Some(TokenKind::Comma),
            '+' => Some(TokenKind::Plus),
            '-' => Some(TokenKind::Minus),
            '*' => Some(TokenKind::Asterisk),
            '/' => Some(TokenKind::Slash),
            _ => None,
        }
    }

    fn invalid_token(&mut self, input: &str) -> Token {
        let start = self.cursor;
        let len = input
            .char_indices()
            .find(|(pos, _)| self.consume_token(&input[*pos..]).is_some())
            .map(|(pos, _)| pos)
            .unwrap_or_else(|| input.len());
        debug_assert!(len <= input.len());

        self.cursor = start + len;
        Token {
            kind: TokenKind::Error,
            span: Span {
                start,
                end: start + len,
            },
        }
    }
}

impl<'input> Iterator for Tokenizer<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.input.len() {
            if self.eof {
                return None;
            }
            self.eof = true;
            Some(Token {
                kind: TokenKind::EndOfFile,
                span: Span {
                    start: self.cursor,
                    end: self.cursor,
                },
            })
        } else {
            Some(self.next_token(&self.input[self.cursor..]))
        }
    }
}
