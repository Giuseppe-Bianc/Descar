mod cli;

use clap::Parser;
use cli::Args;

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("Compiling {:?}", args.input);
    }
}
