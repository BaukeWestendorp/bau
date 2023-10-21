use crate::parser::source::Source;
use crate::tokenizer::token::Span;
use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum BauError {
    ParserError { span: Span, message: String },
    ExecutionError { message: String },
    TypecheckerError { span: Span, message: String },
}

impl BauError {
    pub fn log(&self, source: &Source) {
        match self {
            BauError::ParserError { span, message }
            | BauError::TypecheckerError { span, message } => {
                let print_line_gutter = |line_number: Option<usize>| {
                    match line_number {
                        Some(line_number) => {
                            eprint!("{: <1$}", "", line_number.to_string().len());
                            eprint!("{}", line_number);
                        }
                        None => {
                            eprint!("{: <1$} ", "", " ".len());
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
                eprintln!("{}: {}", "error".bright_red(), message);
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
                eprintln!("{}{}", "^ ".bright_red(), message.bright_red());
            }
            BauError::ExecutionError { message } => {
                eprintln!("Error: {}", message);
            }
        }
    }
}

pub type BauResult<T> = Result<T, BauError>;
