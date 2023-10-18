use crate::tokenizer::source::Source;
use crate::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum BauError {
    ParserError { token: Token, message: String },
    InterpreterError { message: String },
    ExecutionError { message: String },
}

impl BauError {
    pub fn log(&self, file: &str, source: &Source) {
        match self {
            BauError::ParserError { token, message } => {
                let (line, column) = source.line_and_column(token.span.start);

                eprintln!("Error at {}:{}:{}", file, line, column);
                eprintln!();

                let print_source_line = |line: usize| {
                    let source_line = source.line(line);
                    eprint!("\x1b[37m"); // WHITE
                    eprint!("{}\n", source_line);
                    eprint!("\x1b[0m"); // RESET
                };

                let line_count = source.line_count();
                let line_numbers = (usize::max(1, line - 2))..=(usize::min(line + 2, line_count));
                for line_number in line_numbers {
                    print_source_line(line_number);
                    if line_number == line {
                        eprint!("\x1b[31m"); // RED
                        let underline = "^".repeat(token.len());
                        eprint!(
                            "{: <1$}{underline} {message}",
                            "",
                            column - 1,
                            message = message
                        );
                        eprint!("\x1b[0m"); // RESET
                        eprintln!();
                    }
                }
            }
            BauError::InterpreterError { message } => {
                eprintln!("Error: {}", message);
            }
            BauError::ExecutionError { message } => {
                eprintln!("Error: {}", message);
            }
        }
    }
}

pub type BauResult<T> = Result<T, BauError>;
