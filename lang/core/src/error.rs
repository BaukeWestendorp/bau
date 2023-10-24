use crate::parser::source::Source;
use crate::tokenizer::token::{Span, TokenKind};
use colored::Colorize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    UnknownToken(TokenKind),
    UnexpectedToken(TokenKind, Option<TokenKind>),
    UnexpectedEof(Option<TokenKind>),
    InvalidStartOfExpression(TokenKind),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnknownToken(token) => format!("Unknown token: `{}`", token),
            Self::UnexpectedToken(token, expected) => match expected {
                Some(expected) => {
                    format!("Unexpected token: `{}`, expected: `{}`", token, expected)
                }
                None => format!("Unexpected token: `{}`", token),
            },
            Self::UnexpectedEof(expected) => match expected {
                Some(expected) => format!("Unexpected end of file, expected: `{}`", expected),
                None => format!("Unexpected end of file"),
            },
            Self::InvalidStartOfExpression(token) => {
                format!("Invalid start of expression: `{}`", token)
            }
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BauError {
    ParserError { span: Span, error: ParserError },
    ExecutionError { message: String },
    TypecheckerError { span: Span, message: String },
}

impl BauError {
    pub fn log(&self, source: &Source) {
        match self {
            BauError::ParserError { span, .. } => self.internal_log(source, span),
            BauError::TypecheckerError { span, .. } => self.internal_log(source, span),
            BauError::ExecutionError { .. } => {
                self.internal_log(source, &Span { start: 0, end: 0 })
            }
        }
    }

    fn internal_log(&self, source: &Source, span: &Span) {
        let max_line_number_len = source.line_count().to_string().len();
        let error_message = match self {
            BauError::ParserError { error, .. } => error.to_string(),
            BauError::TypecheckerError { message, .. } => message.clone(),
            BauError::ExecutionError { message } => message.clone(),
        };

        let print_line_gutter = |line_number: Option<usize>| {
            match line_number {
                Some(line_number) => {
                    eprint!(
                        "{: <1$}",
                        "",
                        max_line_number_len - line_number.to_string().len()
                    );
                    eprint!("{}", line_number);
                }
                None => {
                    eprint!("{: <1$}", "", max_line_number_len);
                }
            }
            eprint!(" {} ", "|".bright_red());
        };

        let print_line = |line: usize, column: usize, len: usize| {
            let (start, end) = source.line(line).split_at(column - 1);
            let (mid_error, end) = end.split_at(len);
            print_line_gutter(Some(line));
            eprintln!("{}{}{}", start.white(), mid_error.bright_red(), end.white());
        };

        let (line, column) = source.line_and_column(span.start);
        eprintln!("{}: {}", "error".bright_red(), error_message);
        eprintln!(
            "{}{} {}:{}:{}",
            "-".repeat(line.to_string().len() + 2).bright_red(),
            ">".bright_red(),
            source.file_path(),
            line,
            column
        );
        print_line(line, column, span.len());
        print_line_gutter(None);
        eprint!("{: <1$}", "", column - 1);
        eprintln!("{}{}", "^ ".bright_red(), error_message.bright_red());
    }
}

pub type BauResult<T> = Result<T, BauError>;
