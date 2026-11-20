//! Command-line interface (CLI) definitions for the Descar compiler.
//!
//! The CLI uses a command structure similar to Git. Each compiler function has
//! its own subcommand. The module defines the `compile` and `check` subcommands.
//! Clap parses the arguments and uses the types in this module to store them.
//!
//! Clap uses most of the doc comments on the argument fields, the subcommand
//! variants, and the optimization level variants as help text. Do not change
//! those comments without checking the help output.

use clap::{
    ArgAction, Args as ClapArgs, Parser, Subcommand, ValueEnum,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use descar_core::error::internal::DescarError;
use std::path::PathBuf;

/// Help template that Clap uses for the Descar CLI.
///
/// The template contains Clap placeholders, for example `{name}` and `{usage}`.
/// Clap replaces each placeholder with the related value when it prints the help text.
const HELP_STR: &str = r"
{before-help}{name} {version}
{author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

/// Creates the terminal styles that the Descar CLI uses for help and error output.
///
/// # Returns
///
/// A [`Styles`] value. Headers and usage text use bright cyan. Literals use bright
/// magenta. Errors use bright red. Valid values use bright green. Invalid values use
/// bright yellow with an underline. Placeholders use bright blue.
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

/// Parses a command-line value into the path of a Descar source file.
///
/// The function accepts the value when the file extension is `dr`. The comparison
/// ignores the case of the extension. The function does not check that the file exists.
///
/// # Arguments
///
/// * `value` - The path text from the command line.
///
/// # Returns
///
/// The value as a [`PathBuf`].
///
/// # Errors
///
/// Returns [`DescarError::UnsupportedSourceExtension`] when the extension is not `dr`.
/// The error contains the rejected extension. The error contains `<none>` when the path
/// has no extension or when the extension is not valid UTF-8.
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
///
/// The default level is [`OptimizationLevel::None`].
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
///
/// The `compile` and `check` subcommands include these arguments with `#[command(flatten)]`.
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
///
/// The `command` field is `None` when the user selects no subcommand.
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

impl Args {
    /// Returns the logging arguments of the selected subcommand.
    ///
    /// # Returns
    ///
    /// * `Some` with a reference to the [`LoggingArgs`] of the `compile` or `check` subcommand.
    /// * `None` when the user selects no subcommand. In this case, the program prints the help text.
    #[must_use]
    pub const fn logging(&self) -> Option<&LoggingArgs> {
        match &self.command {
            Some(Command::Compile(args)) => Some(&args.logging),
            Some(Command::Check(args)) => Some(&args.logging),
            None => None,
        }
    }
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
///
/// The `input` field accepts only paths with the `.dr` extension. The `optimize`
/// field uses [`OptimizationLevel::None`] when the user gives no value.
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
///
/// The `input` field accepts only paths with the `.dr` extension.
#[derive(Debug, ClapArgs)]
pub struct CheckArgs {
    /// Source `.dr` file to check.
    #[arg(value_name = "FILE", value_parser = parse_dr_file)]
    pub input: PathBuf,

    /// Logging configuration.
    #[command(flatten)]
    pub logging: LoggingArgs,
}