use bau_core::parser::source::Source;
use bau_core::Bau;
use clap::Parser;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_path: String,
}

fn main() {
    let args = Args::parse();

    let source = match std::fs::read_to_string(&args.file_path) {
        Ok(text) => Source::new(text, args.file_path),
        Err(_) => {
            eprintln!("Could not find file `{}`", args.file_path);
            std::process::exit(1);
        }
    };

    let bau = Bau::new();
    match bau.run(&source) {
        Ok(_) => {}
        Err(error) => {
            error.log(&source);
            std::process::exit(1);
        }
    }
}
