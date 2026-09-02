mod cli;

use clap::Parser;
use cli::{Args, Command};

fn main() {
    let args = Args::parse();

    match args.command {
        None => Args::command().print_help().expect("failed to print help"),
        Some(Command::Compile(args)) => {
            if !args.logging.quiet {
                match args.logging.verbose {
                    0 => {}
                    1 => println!("Compiling {}", args.input.display()),
                    2 => println!("Compiling {} with debug diagnostics", args.input.display()),
                    _ => println!("Compiling {} with trace diagnostics", args.input.display()),
                }
            }
        }
        Some(Command::Check(args)) => {
            if !args.logging.quiet {
                match args.logging.verbose {
                    0 => {}
                    1 => println!("Checking {}", args.input.display()),
                    2 => println!("Checking {} with debug diagnostics", args.input.display()),
                    _ => println!("Checking {} with trace diagnostics", args.input.display()),
                }
            }
        }
    }
}
