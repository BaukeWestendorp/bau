use crate::error::BauResult;
use clap::Parser;

pub mod error;
pub mod parser;
pub mod tokenizer;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_path: String,
}

fn main() -> BauResult<()> {
    let args = Args::parse();

    let source = match std::fs::read_to_string(&args.file_path) {
        Ok(source) => source,
        Err(_) => {
            eprintln!("Could not find file '{}'", args.file_path);
            std::process::exit(1);
        }
    };

    let mut parser = parser::Parser::new(source.as_str());
    let items = parser.parse_top_level();
    eprintln!("{:#?}", items);

    // match parser.parse() {
    //     Ok(Some(node)) => println!("{:#?}", node),
    //     Ok(None) => {}
    //     Err(error) => match error {
    //         BauError::ParseError {
    //             line,
    //             column,
    //             message,
    //         } => {
    //             eprintln!("Error at {}:{}:{}", args.file_path, line, column);
    //             eprintln!();
    //
    //             let source_line = source.lines().nth(line - 1).unwrap().replace("\t", "    ");
    //             eprint!("\x1b[37m"); // WHITE
    //             eprintln!("{}", source_line);
    //             eprint!("\x1b[31m"); // RED
    //             eprintln!("{: <1$}^ {message}", "", column - 2);
    //             eprint!("\x1b[0m"); // RESET
    //         }
    //     },
    // };

    Ok(())
}
