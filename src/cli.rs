//! Command-Line Interface (CLI) module for the Descar compiler.
//!
//! This module defines the command-line argument structure and custom styling
//! for the Descar compiler binary. It uses `clap` for argument parsing with
//! enhanced terminal help output.

use clap::{
    builder::{styling::{AnsiColor, Effects}, Styles},
    Parser, ValueHint,
};
use std::path::PathBuf;

/// Custom help template used by the Descar CLI.
const HELP_STR: &str = r"
{before-help}{name} {version}
{author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

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
fn parse_vn_file(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    let is_vn = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dr"));

    if is_vn {
        Ok(path)
    } else {
        Err("expected a path to a .dr file".into())
    }
}

/// Command-line arguments accepted by the Descar compiler.
#[derive(Debug, Parser)]
#[command(
    name = "descar",
    version = env!("CARGO_PKG_VERSION"),
    about = "A compiler for the Descar language",
    long_about = None,
    help_template = HELP_STR,
    styles = custom_styles()
)]
pub struct Args {
    /// Input `.dr` source file to compile.
    #[arg(
        short,
        long,
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        value_parser = parse_vn_file
    )]
    pub input: PathBuf,

    /// Show verbose compiler output.
    #[arg(short, long)]
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::parse_vn_file;

    #[test]
    fn accepts_vn_extension_case_insensitively() {
        assert!(parse_vn_file("program.dr").is_ok());
        assert!(parse_vn_file("program.DR").is_ok());
    }

    #[test]
    fn rejects_non_vn_extension() {
        assert!(parse_vn_file("program.txt").is_err());
        assert!(parse_vn_file("program").is_err());
    }
}
