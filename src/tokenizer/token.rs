use std::ops::{Index, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Error,
    EndOfFile,
    Whitespace,

    Fn,
    Let,
    If,
    Else,
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
    Asterisk,
    Slash,
    ExclamationMark,

    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Error => "error".to_string(),
            Self::EndOfFile => "end of file".to_string(),
            Self::Whitespace => "whitespace".to_string(),

            Self::Fn => "fn".to_string(),
            Self::Let => "let".to_string(),
            Self::If => "if".to_string(),
            Self::Else => "else".to_string(),
            Self::Return => "return".to_string(),
            Self::Loop => "loop".to_string(),

            Self::ParenOpen => "(".to_string(),
            Self::ParenClose => ")".to_string(),
            Self::BraceOpen => "{".to_string(),
            Self::BraceClose => "}".to_string(),
            Self::SquareOpen => "[".to_string(),
            Self::SquareClose => "]".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::Comma => ",".to_string(),
            Self::Equals => "=".to_string(),
            Self::Plus => "+".to_string(),
            Self::Minus => "-".to_string(),
            Self::Asterisk => "*".to_string(),
            Self::Slash => "/".to_string(),
            Self::ExclamationMark => "!".to_string(),

            Self::Identifier => "identifier".to_string(),
            Self::IntLiteral => "integer literal".to_string(),
            Self::FloatLiteral => "float literal".to_string(),
            Self::StringLiteral => "string literal".to_string(),
        };

        write!(f, "{}", str)
    }
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
