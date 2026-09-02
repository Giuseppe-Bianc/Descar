mod cli;

use clap::Parser;
use cli::{Args, Command};

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Compile(args) => {
            if args.verbose {
                println!("Compiling {}", args.input.display());
            }
        }
        Command::Check(args) => {
            if args.verbose {
                println!("Checking {}", args.input.display());
            }
        }
    }
}
