use crate::error::{BauError, BauResult};
use crate::parser::Parser;

pub mod error;
pub mod node;
pub mod parser;

fn main() -> BauResult<()> {
    let file_name = "main.bau";
    let source = "fn main() { let x; }";

    let mut parser = Parser::new(source);
    match parser.parse() {
        Ok(Some(node)) => println!("{:#?}", node),
        Ok(None) => println!("No node"),
        Err(error) => match error {
            BauError::ParseError {
                line,
                column,
                message,
            } => {
                eprintln!("Error at {}:{}:{}", file_name, line, column);
                eprintln!();

                let line = source.lines().nth(line - 1).unwrap();
                eprint!("\x1b[37m"); // WHITE
                eprintln!("{}", line);
                eprint!("\x1b[31m"); // RED
                eprintln!("{: <1$}^ {message}", "", column - 1);
                eprint!("\x1b[0m"); // RESET
            }
        },
    };
    Ok(())
}
