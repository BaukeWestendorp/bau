use crate::source::{CodeRange, Source};
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
        range: CodeRange,
    },
    ExpectedItem {
        token: Token,
    },
    ExpectedExpression {
        token: Token,
    },
    UnknownType {
        range: CodeRange,
        type_name: String,
    },
    TypeMismatch {
        range: CodeRange,
        expected: Type,
        actual: Type,
    },
    VariableAlreadyExists {
        range: CodeRange,
        name: String,
    },
    VariableDoesNotExist {
        range: CodeRange,
        name: String,
    },
    ReturnValueInVoidFunction {
        range: CodeRange,
    },
    ExpectedReturnValue {
        range: CodeRange,
    },
}

impl std::fmt::Display for BauError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::UnexpectedToken {
                token, expected, ..
            } => {
                format!(
                    "Expected token `{}`, but found `{}` instead",
                    expected, token.kind
                )
            }
            Self::UnexpectedEndOfFile { .. } => {
                format!("Expected a token, but found end of file instead")
            }
            Self::ExpectedItem { token, .. } => {
                format!(
                    "Expected an item (`fn` or `extend`), but found `{}` instead",
                    token.kind
                )
            }
            Self::ExpectedExpression { token, .. } => {
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
            Self::VariableDoesNotExist { name, .. } => {
                format!("Variable `{}` does not exist", name)
            }
            Self::ReturnValueInVoidFunction { .. } => {
                format!("Cannot return a value in a void function")
            }
            Self::ExpectedReturnValue { .. } => {
                format!("Expected a return value")
            }
        };

        write!(f, "{}", str)
    }
}

pub type BauResult<T> = Result<T, BauError>;

pub fn print_error(source: &Source, error: &BauError) {
    let max_line_number_len = source.lines().len().to_string().len();

    let range = match error {
        BauError::UnexpectedToken { token, .. } => token.range.clone(),
        BauError::UnexpectedEndOfFile { range, .. } => range.clone(),
        BauError::ExpectedItem { token, .. } => token.range.clone(),
        BauError::ExpectedExpression { token, .. } => token.range.clone(),
        BauError::UnknownType { range, .. } => range.clone(),
        BauError::TypeMismatch { range, .. } => range.clone(),
        BauError::VariableAlreadyExists { range, .. } => range.clone(),
        BauError::VariableDoesNotExist { range, .. } => range.clone(),
        BauError::ReturnValueInVoidFunction { range, .. } => range.clone(),
        BauError::ExpectedReturnValue { range } => range.clone(),
    };

    eprintln!("{}: {}", "error".bright_red(), error.to_string());

    // Multiline error
    let lines = source.text()[range.span.start..range.span.end].lines();
    let line_count = lines.clone().count();
    let mut cursor = 0;
    for (line_number, line) in lines.clone().enumerate() {
        if line_number == 0 {
            print_source_line(
                source,
                max_line_number_len,
                range.coords.line,
                range.coords.column,
                line.len(),
            )
        } else if line_number == line_count - 1 {
            let len = range.span.len() - cursor;
            print_source_line(
                source,
                max_line_number_len,
                range.coords.line + line_number,
                0,
                len,
            )
        } else {
            print_source_line(
                source,
                max_line_number_len,
                range.coords.line + line_number,
                0,
                line.len(),
            )
        }
        cursor += line.len() + 1;
    }

    let underline_length = match line_count {
        1 => range.span.len(),
        _ => lines.map(|line| line.len()).max().unwrap(),
    };

    print_line_gutter(max_line_number_len, None);
    eprintln!(
        "{}",
        format!(
            "{}{} {}",
            " ".repeat(range.coords.column),
            "^".repeat(usize::max(1, underline_length)),
            error.to_string()
        )
        .bright_red()
    );
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
