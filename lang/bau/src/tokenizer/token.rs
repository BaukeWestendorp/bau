use crate::source::CodeRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Keywords
    Fn,
    Extend,
    Let,
    If,
    Else,
    Loop,
    Return,
    Continue,
    Break,

    // Literals
    StringLiteral,
    IntLiteral,
    FloatLiteral,
    BoolLiteral,

    // Identifiers
    Identifier,

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    ExclamationMark,
    LessThan,
    GreaterThan,

    // Compound operators
    PlusEquals,
    MinusEquals,
    AsteriskEquals,
    SlashEquals,
    PercentEquals,

    EqualsEquals,
    ExclamationMarkEquals,
    LessThanEquals,
    GreaterThanEquals,
    AmpersandAmpersand,
    PipePipe,

    // Punctuation
    Equals,
    Arrow,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    SquareOpen,
    SquareClose,
    Semicolon,
    Period,
    Comma,

    // Misc
    Comment,
    Whitespace,
    EndOfFile,
    EndOfLine,
    Invalid,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Fn => "fn".to_string(),
            Self::Extend => "extend".to_string(),
            Self::Let => "let".to_string(),
            Self::If => "if".to_string(),
            Self::Else => "else".to_string(),
            Self::Loop => "loop".to_string(),
            Self::Return => "return".to_string(),
            Self::Continue => "continue".to_string(),
            Self::Break => "break".to_string(),

            Self::StringLiteral => "string literal".to_string(),
            Self::IntLiteral => "integer literal".to_string(),
            Self::FloatLiteral => "float literal".to_string(),
            Self::BoolLiteral => "bool literal".to_string(),

            Self::Identifier => "identifier".to_string(),

            Self::Plus => "+".to_string(),
            Self::Minus => "-".to_string(),
            Self::Asterisk => "*".to_string(),
            Self::Slash => "/".to_string(),
            Self::ExclamationMark => "!".to_string(),
            Self::LessThan => "<".to_string(),
            Self::GreaterThan => ">".to_string(),
            Self::Percent => "%".to_string(),

            Self::PlusEquals => "+=".to_string(),
            Self::MinusEquals => "-=".to_string(),
            Self::AsteriskEquals => "*=".to_string(),
            Self::SlashEquals => "/=".to_string(),
            Self::PercentEquals => "%=".to_string(),

            Self::EqualsEquals => "==".to_string(),
            Self::ExclamationMarkEquals => "!=".to_string(),
            Self::LessThanEquals => "<=".to_string(),
            Self::GreaterThanEquals => ">=".to_string(),
            Self::AmpersandAmpersand => "&&".to_string(),
            Self::PipePipe => "||".to_string(),

            Self::Equals => "=".to_string(),
            Self::Arrow => "->".to_string(),
            Self::ParenOpen => "(".to_string(),
            Self::ParenClose => ")".to_string(),
            Self::BraceOpen => "{".to_string(),
            Self::BraceClose => "}".to_string(),
            Self::SquareOpen => "[".to_string(),
            Self::SquareClose => "]".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::Period => ".".to_string(),
            Self::Comma => ",".to_string(),

            Self::Comment => "comment".to_string(),
            Self::Whitespace => "whitespace".to_string(),
            Self::EndOfFile => "end of file".to_string(),
            Self::EndOfLine => "end of line".to_string(),
            Self::Invalid => "invalid token".to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenKind,
    range: CodeRange,
}

impl Token {
    pub fn new(kind: TokenKind, range: CodeRange) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn range(&self) -> CodeRange {
        self.range
    }

    pub fn len(&self) -> usize {
        self.range.span.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }
}
