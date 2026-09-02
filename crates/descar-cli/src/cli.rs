//! Command-Line Interface (CLI) for the Descar compiler.
//!
//! The CLI follows a git-style command structure so compiler functionality can
//! grow through dedicated subcommands such as `compile` and `check`.

use clap::{
    ArgAction, Args as ClapArgs, Parser, Subcommand, ValueEnum,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use descar_core::error::internal::DescarError;
use std::path::PathBuf;

/// Custom help template used by the Descar CLI.
const HELP_STR: &str = r"
{before-help}{name} {version}
{author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

/// Creates the custom styles used by the Descar CLI.
#[must_use]
pub fn custom_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightCyan.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightMagenta.on_default() | Effects::BOLD)
        .error(AnsiColor::BrightRed.on_default() | Effects::BOLD)
        .valid(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .invalid(AnsiColor::BrightYellow.on_default() | Effects::BOLD | Effects::UNDERLINE)
        .placeholder(AnsiColor::BrightBlue.on_default())
        .usage(AnsiColor::BrightCyan.on_default() | Effects::BOLD | Effects::UNDERLINE)
}

/// Validates and parses a Descar source file path.
fn parse_dr_file(value: &str) -> Result<PathBuf, DescarError> {
    let path = PathBuf::from(value);
    let is_dr = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dr"));

    if is_dr {
        Ok(path)
    } else {
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("<none>").to_owned();

        Err(DescarError::UnsupportedSourceExtension { extension })
    }
}

/// Optimization levels accepted by the compiler CLI.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OptimizationLevel {
    /// Disable compiler optimizations.
    #[default]
    None,
    /// Enable basic optimizations.
    Basic,
    /// Enable aggressive optimizations.
    Aggressive,
}

/// Logging controls shared by CLI subcommands.
#[derive(Clone, Debug, Default, ClapArgs)]
pub struct LoggingArgs {
    /// Increase log verbosity. Repeatable: `-v`, `-vv`, `-vvv`.
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-essential output.
    #[arg(short, long)]
    pub quiet: bool,
}

/// Root command-line arguments for the Descar compiler.
#[derive(Debug, Parser)]
#[command(
    name = "descar",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "Modern compiler for the Descar programming language.",
    long_about = None,
    help_template = HELP_STR,
    styles = custom_styles()
)]
pub struct Args {
    /// Descar operation to execute.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Top-level Descar CLI commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile a Descar source file.
    Compile(CompileArgs),

    /// Check a Descar source file without producing compiler output.
    Check(CheckArgs),
}

/// Arguments for the `compile` command.
#[derive(Debug, ClapArgs)]
pub struct CompileArgs {
    /// Source `.dr` file to compile.
    #[arg(value_name = "FILE", value_parser = parse_dr_file)]
    pub input: PathBuf,

    /// Write compiler output to this file.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Optimization level: none, basic, aggressive.
    #[arg(short = 'O', long, value_enum, default_value_t)]
    pub optimize: OptimizationLevel,

    /// Also emit the intermediate representation (IR).
    #[arg(long)]
    pub emit_ir: bool,

    /// Enable advanced diagnostics such as extended warnings and statistics.
    #[arg(long)]
    pub diagnostics: bool,

    /// Logging configuration.
    #[command(flatten)]
    pub logging: LoggingArgs,
}

/// Arguments for the `check` command.
#[derive(Debug, ClapArgs)]
pub struct CheckArgs {
    /// Source `.dr` file to check.
    #[arg(value_name = "FILE", value_parser = parse_dr_file)]
    pub input: PathBuf,

    /// Logging configuration.
    #[command(flatten)]
    pub logging: LoggingArgs,
}
