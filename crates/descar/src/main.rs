mod logging;

use console::style;
use descar_core::error::compile_error::CompileError;
use descar_core::lex::lexer::{Lexer, lexer_tokenize_with_errors};
use descar_core::semantic::type_checker::TypeChecker;
use std::path::Path;
use std::time::Instant;
use std::{fs, process};

use clap::{CommandFactory, Parser};
use descar_cli::cli::{Args, Command, CompileArgs, OptimizationLevel};
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::syntax::ast::Stmt;
use descar_core::syntax::parser::JsavParser;
use tracing::{debug, error_span, info, trace, warn};

use descar_core::file::{FileSizeInfo, SizeSystems};

/// Reports an I/O error using the CLI error formatting.
///
/// The error category and underlying error value are printed to standard error.
/// This message is a user-facing error, not a log event, so `-q` does not hide it.
fn handle_io_error<T: std::fmt::Display>(error_type: &str, e: T) {
    eprintln!("{} {}: {}\n", style("ERROR:").red().bold(), style(error_type).red(), style(e).yellow());
}

/// Logs the size of the source text at debug level.
///
/// The values are formatted only when the debug level is enabled.
fn log_source_size(bytes: usize) {
    let size = FileSizeInfo::new(u64::try_from(bytes).unwrap_or(u64::MAX));
    debug!(
        bytes = size.bytes,
        si = %size.format(&SizeSystems::SI_SYSTEM),
        iec = %size.format(&SizeSystems::IEC),
        "source file size"
    );
}

/// Reads a source file, reporting an I/O error and exiting if the read fails.
fn read_input(path: &Path) -> String {
    trace!("reading source file");
    let input = fs::read_to_string(path).unwrap_or_else(|e| {
        handle_io_error("I/O", format!("failed to read '{}': {}", path.to_string_lossy(), e));
        process::exit(1);
    });
    log_source_size(input.len());
    input
}
/// Returns the path as UTF-8, reporting an I/O error and exiting if conversion fails.
fn path_to_str(path: &Path) -> &str {
    path.to_str().unwrap_or_else(|| {
        handle_io_error("I/O", format!("invalid file path '{}'", path.to_string_lossy()));
        process::exit(1);
    })
}

/// Prints the diagnostics of a failed stage to stderr and exits with status code 1.
///
/// The diagnostics are the output of the compiler, not log events, so `-q` does not hide them.
fn stop_frontend(stage: &'static str, error_reporter: &ErrorReporter, errors: Vec<CompileError>) -> ! {
    debug!(stage, errors = errors.len(), "frontend stopped");
    eprintln!("{}", error_reporter.report_errors(errors));
    process::exit(1);
}

/// Lexes, parses, and type-checks source input.
///
/// Returns the parsed statements when all stages succeed. If any stage reports
/// diagnostics, prints them and exits with status code 1.
fn run_frontend(file_path: &str, input: &str) -> Vec<Stmt> {
    trace!(source_len = input.len(), "lexing started");
    let started = Instant::now();
    let mut lexer = Lexer::new(file_path, input);
    let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);
    debug!(tokens = tokens.len(), errors = lexer_errors.len(), elapsed = ?started.elapsed(), "lexing finished");
    let error_reporter = ErrorReporter::new(lexer.get_line_tracker().clone());

    if !lexer_errors.is_empty() {
        stop_frontend("lexing", &error_reporter, lexer_errors);
    }

    trace!(tokens = tokens.len(), "parsing started");
    let started = Instant::now();
    let (statements, parser_errors) = JsavParser::new(&tokens).parse();
    debug!(statements = statements.len(), errors = parser_errors.len(), elapsed = ?started.elapsed(), "parsing finished");
    if !parser_errors.is_empty() {
        stop_frontend("parsing", &error_reporter, parser_errors);
    }

    trace!(statements = statements.len(), "type checking started");
    let started = Instant::now();
    let mut type_checker = TypeChecker::new();
    let type_checker_errors = type_checker.check(&statements);
    debug!(errors = type_checker_errors.len(), elapsed = ?started.elapsed(), "type checking finished");
    if !type_checker_errors.is_empty() {
        stop_frontend("type checking", &error_reporter, type_checker_errors);
    }

    statements
}

/// Reads a source file and runs the frontend on it.
fn analyze(path: &Path) -> Vec<Stmt> {
    let input = read_input(path);
    let file_path_str = path_to_str(path);
    run_frontend(file_path_str, &input)
}

/// Warns about options that the parser accepts but the compiler does not use yet.
///
/// Remove an entry when the matching option is implemented.
fn warn_ignored_options(args: &CompileArgs) {
    let options = [
        ("--output", args.output.is_some()),
        ("--optimize", args.optimize != OptimizationLevel::None),
        ("--emit-ir", args.emit_ir),
        ("--diagnostics", args.diagnostics),
    ];

    for (option, _) in options.into_iter().filter(|&(_, is_set)| is_set) {
        warn!(option, "option is accepted but has no effect yet");
    }
}

fn main() {
    let args = Args::parse();

    if let Some(Err(error)) = args.logging().map(logging::init) {
        eprintln!("{} logging is disabled: {error}", style("WARNING:").yellow().bold());
    }

    match args.command {
        None => Args::command().print_help().expect("failed to print help"),
        Some(Command::Compile(args)) => {
            let _command = error_span!("compile", file = ?args.input).entered();
            debug!(optimize = ?args.optimize, emit_ir = args.emit_ir, diagnostics = args.diagnostics, output = ?args.output, "command started");
            warn_ignored_options(&args);

            let _statements = analyze(args.input.as_path());
            info!("compilation succeeded");
        }
        Some(Command::Check(args)) => {
            let _command = error_span!("check", file = ?args.input).entered();
            debug!("command started");

            let _statements = analyze(args.input.as_path());
            info!("check succeeded");
        }
    }
}
