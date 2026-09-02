//! Command-Line Interface (CLI) for the Descar compiler.
//!
//! The CLI follows a git-style command structure so compiler functionality can
//! grow through dedicated subcommands such as `compile` and `check`.

use clap::{
    builder::{styling::{AnsiColor, Effects}, Styles},
    Args as ClapArgs, Parser, Subcommand,
};
use std::path::PathBuf;

/// Custom help template used by the Descar CLI.
const HELP_STR: &str = r#"
{before-help}{name} {version}
{author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}"#;

/// Creates the custom styles used by clap's help output.
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
fn parse_dr_file(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    let is_dr = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dr"));

    if is_dr {
        Ok(path)
    } else {
        Err("expected a path to a .dr file".into())
    }
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
    styles = custom_styles(),
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Args {
    /// Descar operation to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level Descar CLI commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile a Descar source file.
    Compile(CompileArgs),

    /// Check a Descar source file without producing compiler output.
    Check(CheckArgs),
}

/// Arguments shared by compilation-oriented commands.
#[derive(Debug, ClapArgs)]
pub struct CompileArgs {
    /// Source `.dr` file to compile.
    #[arg(value_name = "FILE", value_parser = parse_dr_file)]
    pub input: PathBuf,

    /// Write compiler output to this file.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Show verbose compiler output.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Arguments for source validation.
#[derive(Debug, ClapArgs)]
pub struct CheckArgs {
    /// Source `.dr` file to check.
    #[arg(value_name = "FILE", value_parser = parse_dr_file)]
    pub input: PathBuf,

    /// Show verbose compiler output.
    #[arg(short, long)]
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::parse_dr_file;

    #[test]
    fn accepts_dr_extension_case_insensitively() {
        assert!(parse_dr_file("program.dr").is_ok());
        assert!(parse_dr_file("program.DR").is_ok());
    }

    #[test]
    fn rejects_non_dr_extension() {
        assert!(parse_dr_file("program.txt").is_err());
        assert!(parse_dr_file("program").is_err());
    }
}
