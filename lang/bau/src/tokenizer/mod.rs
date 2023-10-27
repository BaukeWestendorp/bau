pub use token::Token;

use self::token::{SourceCoords, Span, TokenKind};

mod rule;
pub mod token;

#[derive(Debug, Clone, PartialEq)]
pub struct Tokenizer<'input> {
    input: &'input str,
    cursor: usize,
    line: usize,
    column: usize,
    eof: bool,
    rules: Vec<rule::Rule>,
}

impl<'input> Tokenizer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            cursor: 0,
            line: 0,
            column: 0,
            eof: false,
            rules: rule::get_rules(),
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
        match next {
            '\n' => Some(self.token(TokenKind::EndOfLine, 1)),
            char if char.is_whitespace() => {
                let len = input
                    .char_indices()
                    .take_while(|(_, c)| c.is_whitespace() && *c != '\n')
                    .last()
                    .expect("At least one whitespace char should exist")
                    .0
                    + 1;
                Some(self.token(TokenKind::Whitespace, len))
            }
            char => {
                if let Some(kind) = rule::get_unambiguous_token(char) {
                    return Some(self.token(kind, 1));
                }

                let (len, kind) = self
                    .rules
                    .iter()
                    // `max_by_key` returns the last element if multiple
                    // rules match, but we want earlier rules to "win"
                    // against later ones
                    .rev()
                    .filter_map(|rule| Some(((rule.matches)(input)?, rule.kind)))
                    .max_by_key(|&(len, _)| len)?;

                Some(self.token(kind, len))
            }
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
        self.token(TokenKind::Invalid, len)
    }

    fn token(&mut self, kind: TokenKind, len: usize) -> Token {
        let token = Token::new(
            kind,
            Span::new(self.cursor, self.cursor + len),
            SourceCoords::new(self.line, self.column),
        );
        for char in self.input[self.cursor..self.cursor + len].chars() {
            self.column += 1;
            if char == '\n' {
                self.line += 1;
                self.column = 0;
            }
        }
        self.cursor += len;
        token
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
            Some(self.token(TokenKind::EndOfFile, 0))
        } else {
            Some(self.next_token(&self.input[self.cursor..]))
        }
    }
}
