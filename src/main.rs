use crate::parser::Parser;

pub mod error;
pub mod parser;

fn main() {
    let mut parser = Parser::new("fn main() { let x = 69.42; }");
    parser.parse();
}
