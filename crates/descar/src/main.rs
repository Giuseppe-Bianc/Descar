use console::style;
use descar_core::lex::lexer::{Lexer, lexer_tokenize_with_errors};
use descar_core::semantic::type_checker::TypeChecker;
use std::path::Path;
use std::{fs, process};

use clap::{CommandFactory, Parser};
use descar_cli::cli::{Args, Command};
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::syntax::ast::Stmt;
use descar_core::syntax::parser::JsavParser;

use descar_core::file::{FileSizeInfo, FileSizeReport, SizeSystems};

/// Reports an I/O error using the CLI error formatting.
///
/// The error category and underlying error value are printed to standard error.
fn handle_io_error<T: std::fmt::Display>(error_type: &str, e: T) {
    eprintln!("{} {}: {}\n", style("ERROR:").red().bold(), style(error_type).red(), style(e).yellow());
}

/// Reads filesystem metadata and prints a file size report.
///
/// The report contains both SI and IEC representations. Metadata failures are
/// forwarded to the CLI I/O error handler.
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

/// Reads a source file, reporting an I/O error and exiting if the read fails.
fn read_input(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        handle_io_error("I/O", format!("failed to read '{}': {}", path.to_string_lossy(), e));
        process::exit(1);
    })
}
/// Returns the path as UTF-8, reporting an I/O error and exiting if conversion fails.
fn path_to_str(path: &Path) -> &str {
    path.to_str().unwrap_or_else(|| {
        handle_io_error("I/O", format!("invalid file path '{}'", path.to_string_lossy()));
        process::exit(1);
    })
}

/// Lexes, parses, and type-checks source input.
///
/// Returns the parsed statements when all stages succeed. If any stage reports
/// diagnostics, prints them and exits with status code 1.
fn run_frontend(file_path: &str, input: &str) -> Vec<Stmt> {
    let mut lexer = Lexer::new(file_path, input);
    let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);
    let error_reporter = ErrorReporter::new(lexer.get_line_tracker().clone());

    if !lexer_errors.is_empty() {
        eprintln!("{}", error_reporter.report_errors(lexer_errors));
        process::exit(1);
    }

    let (statements, parser_errors) = JsavParser::new(&tokens).parse();
    if !parser_errors.is_empty() {
        eprintln!("{}", error_reporter.report_errors(parser_errors));
        process::exit(1);
    }

    let mut type_checker = TypeChecker::new();
    let type_checker_errors = type_checker.check(&statements);
    if !type_checker_errors.is_empty() {
        eprintln!("{}", error_reporter.report_errors(type_checker_errors));
        process::exit(1);
    }

    statements
}

fn main() {
    let args = Args::parse();
    match args.command {
        None => Args::command().print_help().expect("failed to print help"),
        Some(Command::Compile(args)) => {
            let file_path: &Path = args.input.as_path();

            let input = read_input(file_path);

            let file_path_str: &str = path_to_str(file_path);

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

            let _statements = run_frontend(file_path_str, &input);
            if !args.logging.quiet {
                println!("Compilation successful: {file_path_str}");
            }
        }
        Some(Command::Check(args)) => {
            let file_path: &Path = args.input.as_path();

            let input = read_input(file_path);

            let file_path_str: &str = path_to_str(file_path);

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

            let _statements = run_frontend(file_path_str, &input);
        }
    }
}
