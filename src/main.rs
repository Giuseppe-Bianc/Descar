use clap::Parser;
use descar::cli::Args;

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("Compiling {:?}", args.input);
    }
}
