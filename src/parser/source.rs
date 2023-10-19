use crate::tokenizer::token::{Span, Token, TokenKind};

pub struct Source {
    text: String,
    line_count: usize,
}

impl Source {
    pub fn line_and_column(&self, index: usize) -> (usize, usize) {
        let mut line = 1;
        let mut column = 1;
        for (i, c) in self.text.chars().enumerate() {
            if i == index {
                return (line, column);
            }
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        (line, column)
    }

    pub fn eof_token(&self) -> Token {
        Token {
            kind: TokenKind::EndOfFile,
            span: Span {
                start: self.text().len(),
                end: self.text().len(),
            },
        }
    }
}

impl Source {
    pub fn new(text: &str) -> Self {
        let clean_text = text.replace('\t', "    ");
        let line_count = clean_text.lines().count();
        Self {
            text: clean_text,
            line_count,
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_count
    }

    pub fn line(&self, line: usize) -> &str {
        self.text.lines().nth(line - 1).unwrap()
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl From<&str> for Source {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Source {
    fn from(value: String) -> Self {
        Self::new(value.as_str())
    }
}
