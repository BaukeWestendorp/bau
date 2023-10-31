use crate::error::print_error;
use crate::source::{CodeRange, Source};
use crate::tokenizer::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorKind {
    UnexpectedToken {
        found: TokenKind,
        expected: TokenKind,
    },
    UnexpectedEndOfFile,
    ExpectedItem {
        found: TokenKind,
    },
    ExpectedExpression {
        found: TokenKind,
    },
    InvalidExpressionStart {
        found: TokenKind,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    kind: ParserErrorKind,
    range: CodeRange,
}

impl ParserError {
    pub fn new(kind: ParserErrorKind, range: CodeRange) -> Self {
        Self { kind, range }
    }

    pub fn print(&self, source: &Source) {
        print_error(source, Some(&self.range), &self.to_string());
    }
}

impl std::error::Error for ParserError {}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match &self.kind {
            ParserErrorKind::UnexpectedToken { found, expected } => {
                format!(
                    "Expected token `{}`, but found `{}` instead",
                    expected, found
                )
            }
            ParserErrorKind::UnexpectedEndOfFile { .. } => {
                "Expected a token, but found end of file instead".to_string()
            }
            ParserErrorKind::ExpectedItem { found } => {
                format!(
                    "Expected an item (`fn` or `extend`), but found `{}` instead",
                    found
                )
            }
            ParserErrorKind::ExpectedExpression { found } => {
                format!("Expected an expression, but found `{}` instead", found)
            }
            ParserErrorKind::InvalidExpressionStart { found } => {
                format!("Invalid start of expression `{}`", found)
            }
        };

        write!(f, "{}", str)
    }
}

pub type ParserResult<T> = Result<T, ParserError>;
