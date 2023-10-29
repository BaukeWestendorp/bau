use crate::source::Source;
use crate::tokenizer::token::TokenKind;
use crate::tokenizer::Token;
use crate::typechecker::Type;

use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum BauError {
    UnexpectedToken {
        token: Token,
        expected: TokenKind,
    },
    UnexpectedEndOfFile {
        token: Token,
    },
    ExpectedItem {
        token: Token,
    },
    ExpectedExpression {
        token: Token,
    },
    UnknownType {
        token: Token,
        type_name: String,
    },
    TypeMismatch {
        token: Token,
        expected: Type,
        actual: Type,
    },
    VariableAlreadyExists {
        token: Token,
        name: String,
    },
}

impl std::fmt::Display for BauError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::UnexpectedToken { token, expected } => {
                format!(
                    "Expected token `{}`, but found `{}` instead",
                    expected, token.kind,
                )
            }
            Self::UnexpectedEndOfFile { .. } => {
                format!("Expected a token, but found end of file instead")
            }
            Self::ExpectedItem { token } => {
                format!(
                    "Expected an item (`fn` or `extend`), but found `{}` instead",
                    token.kind
                )
            }
            Self::ExpectedExpression { token } => {
                format!("Expected an expression, but found `{}` instead", token.kind)
            }
            Self::UnknownType { type_name, .. } => {
                format!("Unknown type `{}`", type_name)
            }
            Self::TypeMismatch {
                expected, actual, ..
            } => {
                format!(
                    "Expected type `{}`, but found `{}` instead",
                    expected, actual
                )
            }
            Self::VariableAlreadyExists { name, .. } => {
                format!("Variable `{}` already exists", name)
            }
        };

        write!(f, "{}", str)
    }
}

pub type BauResult<T> = Result<T, BauError>;

pub fn print_error(source: &Source, error: &BauError) {
    let max_line_number_len = source.lines().len().to_string().len();

    match error {
        BauError::UnexpectedToken { token, .. }
        | BauError::UnexpectedEndOfFile { token, .. }
        | BauError::ExpectedItem { token }
        | BauError::ExpectedExpression { token }
        | BauError::UnknownType { token, .. }
        | BauError::TypeMismatch { token, .. }
        | BauError::VariableAlreadyExists { token, .. } => {
            eprintln!("{}: {}", "error".bright_red(), error.to_string());

            print_source_line(
                source,
                max_line_number_len,
                token.coords.line,
                token.coords.column,
                token.len(),
            );

            print_line_gutter(max_line_number_len, None);

            eprintln!(
                "{}",
                format!(
                    "{}{} {}",
                    " ".repeat(token.coords.column),
                    "^".repeat(usize::max(1, token.len())),
                    error.to_string()
                )
                .bright_red()
            );
        }
    }
}

fn print_line_gutter(max_line_number_len: usize, line_number: Option<usize>) {
    match line_number {
        Some(line_number) => {
            let padding = max_line_number_len - line_number.to_string().len();
            eprint!(" {}{}", " ".repeat(padding), line_number);
        }
        None => {
            eprint!(" {}", " ".repeat(max_line_number_len));
        }
    }
    eprint!(" {} ", "|".bright_red());
}

fn print_source_line(
    source: &Source,
    max_line_number_len: usize,
    line_number: usize,
    column: usize,
    len: usize,
) {
    let line_number = match line_number >= source.lines().len() {
        true => source.lines().len() - 1,
        false => line_number,
    };
    let (start, end) = source.lines()[line_number].split_at(column);
    let (mid_error, end) = end.split_at(len);
    print_line_gutter(max_line_number_len, Some(line_number + 1));
    eprintln!("{}{}{}", start.white(), mid_error.bright_red(), end.white());
}
