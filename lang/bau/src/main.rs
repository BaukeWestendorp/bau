use bau::Bau;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

fn main() {
    let args = Args::parse();
    Bau::new().run_file(&args.file)
}
