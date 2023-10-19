use std::ops::{Index, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Error,
    EndOfFile,
    Whitespace,

    Fn,
    Let,
    If,
    Return,
    Loop,

    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    SquareOpen,
    SquareClose,
    Semicolon,
    Comma,
    Equals,
    Plus,
    Minus,
    ExclamationMark,

    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn len(&self) -> usize {
        self.span.end - self.span.start
    }

    pub fn text<'input>(&self, input: &'input str) -> &'input str {
        &input[self.span]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[Range::from(index)]
    }
}
