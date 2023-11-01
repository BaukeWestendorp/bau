use bau::source::Source;
use bau::Bau;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

fn main() {
    let args = Args::parse();
    let src = std::fs::read_to_string(&args.file)
        .expect(format!("Failed to read file: `{}`", args.file).as_str());
    match Bau::new().run(&src) {
        Ok(_) => {}
        Err(errors) => {
            let source = Source::new(&src);
            for error in errors.iter() {
                error.print(&source);
            }
        }
    }
}
