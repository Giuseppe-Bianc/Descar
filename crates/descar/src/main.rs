use console::style;
use std::path::Path;
use std::{fs, process};

use clap::{CommandFactory, Parser};
use descar_cli::cli::{Args, Command};
use descar_core::error::compile_error::CompileError;
use descar_core::error::error_code::ErrorCode::E0005;
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::location::line_tracker::LineTracker;

use descar_core::file::{FileSizeInfo, FileSizeReport, SizeSystems};

fn handle_io_error<T: std::fmt::Display>(error_type: &str, e: T) {
    eprintln!("{} {}: {}\n", style("ERROR:").red().bold(), style(error_type).red(), style(e).yellow());
}

fn print_file_size_report(path: &Path) {
    match fs::metadata(path) {
        Ok(metadata) => {
            let size_info = FileSizeInfo::new(metadata.len());
            let report = FileSizeReport::new(size_info, &SizeSystems::SI_SYSTEM, &SizeSystems::IEC);
            println!("{report}\n");
        }
        Err(e) => handle_io_error("File Metadata", e),
    }
}
fn main() {
    let args = Args::parse();
    match args.command {
        None => Args::command().print_help().expect("failed to print help"),
        Some(Command::Compile(args)) => {
            let file_path: &Path = args.input.as_path();

            let input = {
                fs::read_to_string(file_path).unwrap_or_else(|e| {
                    handle_io_error("I/O", e);
                    process::exit(1); // esce con codice 1
                })
            };

            let file_path_str: &str = file_path.to_str().unwrap_or_else(|| {
                handle_io_error("I/O", std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid file path"));
                process::exit(1);
            });

            if !args.logging.quiet {
                match args.logging.verbose {
                    0 => {}
                    1 => {
                        println!("Compiling {file_path_str}");
                        print_file_size_report(file_path);
                    }
                    2 => {
                        println!("Compiling {file_path_str} with debug diagnostics");
                        print_file_size_report(file_path);
                    }
                    _ => {
                        println!("Compiling {file_path_str} with trace diagnostics");
                        print_file_size_report(file_path);
                    }
                }
            }
            let line_tracker = LineTracker::new(file_path_str, input);
            let reporter = ErrorReporter::new(line_tracker.clone());
            let e_span = line_tracker.span_for(3..4);
            let error = CompileError::LexerError {
                code: Option::Some(E0005),
                message: "Invalid character".into(),
                span: e_span,
                help: None,
            };
            println!("{}", reporter.report_errors(vec![error]));
        }
        Some(Command::Check(args)) => {
            if !args.logging.quiet {
                match args.logging.verbose {
                    0 => {}
                    1 => println!("Checking {}", args.input.display()),
                    2 => {
                        println!("Checking {} with debug diagnostics", args.input.display());
                        print_file_size_report(&args.input);
                    }
                    _ => {
                        println!("Checking {} with trace diagnostics", args.input.display());
                        print_file_size_report(&args.input);
                    }
                }
            }
        }
    }
}
