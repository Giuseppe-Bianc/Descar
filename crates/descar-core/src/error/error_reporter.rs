// crates/descar-core/src/error/error_reporter.rs

//! Text output of compile errors with source context.
//!
//! [`ErrorReporter`] converts [`CompileError`] values into text. For an error with a source
//! span, the text contains the error code, the category, the message, the location, the
//! source line, an underline, and the optional help text. The output uses terminal styles
//! from the `console` crate.

use console::style;

use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::location::line_tracker::LineTracker;
use crate::location::source_span::SourceSpan;
use std::fmt::{Display, Write};

/// Removes control characters from a string.
///
/// The function keeps the line feed, the carriage return, and the tab. It removes all other
/// control characters, including the escape character of ANSI escape sequences. This prevents
/// the source text and the error messages from sending control sequences to the terminal.
///
/// # Arguments
///
/// * `s` - The string to filter.
///
/// # Returns
///
/// A new string without the removed characters.
fn sanitize_output(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t')).collect()
}
/// Formats compile errors with source context.
///
/// The reporter reads the source lines from a [`LineTracker`].
pub struct ErrorReporter {
    line_tracker: LineTracker,
}

impl ErrorReporter {
    /// Creates an error reporter.
    ///
    /// # Arguments
    ///
    /// * `line_tracker` - The line tracker that supplies the source lines for the error output.
    #[must_use]
    pub const fn new(line_tracker: LineTracker) -> Self {
        Self { line_tracker }
    }

    /// Formats compile errors as text.
    ///
    /// The function formats the errors in the order of the vector and joins the results.
    /// The category of each error is `LEX` for a lexer error, `SYNTAX` for a syntax error,
    /// `TYPE` for a type error, and `I/O` for an I/O error. An I/O error has no code and no
    /// source context.
    ///
    /// # Arguments
    ///
    /// * `errors` - The compile errors to format.
    ///
    /// # Returns
    ///
    /// A string with the formatted errors. The string is empty if `errors` is empty.
    #[must_use]
    pub fn report_errors(&self, errors: Vec<CompileError>) -> String {
        let mut output = String::with_capacity(errors.len() * 500);

        for error in errors {
            let formatted = match error {
                CompileError::LexerError { message, span, help, code } => {
                    self.format_error("LEX", &message, &span, help.as_deref(), code)
                }

                CompileError::SyntaxError { message, span, help, code } => {
                    self.format_error("SYNTAX", &message, &span, help.as_deref(), code)
                }

                CompileError::TypeError { message, span, help, code } => {
                    self.format_error("TYPE", &message, &span, help.as_deref(), code)
                }

                // Keep this branch when `IrGeneratorError` is enabled again.
                //
                // CompileError::IrGeneratorError {
                //     message,
                //     span,
                //     help,
                //     code,
                // } => self.format_error(
                //     "IR GEN",
                //     &message,
                //     &span,
                //     help.as_deref(),
                //     code,
                // ),

                // Keep this branch when `AsmGeneratorError` is enabled again.
                //
                // CompileError::AsmGeneratorError { message, code } => {
                //     format_simple_error("ASM GEN", &message, code)
                // }
                CompileError::IoError(error) => format_simple_error("I/O", &error, None),
            };

            output.push_str(&formatted);
        }

        output
    }

    /// Formats one error that has a source span.
    ///
    /// The output has these lines:
    ///
    /// * The header: `ERROR`, the optional code, the category, and the message.
    /// * The location: the text of `span`.
    /// * The source block. The function writes it only if the line tracker returns a non-empty
    ///   text for the start line of the span. The block has the source line with its line
    ///   number and a line with an underline. The underline starts at the start column. For a
    ///   span on one line, the underline has at least one `^` and extends to the end column.
    ///   For a span on more than one line, the underline is one `^`, and a further line shows
    ///   the first and the last line of the span.
    /// * The help text, if `help` is `Some`.
    ///
    /// The function removes control characters from the message, the location, and the source
    /// line. It does not remove them from the help text.
    ///
    /// # Arguments
    ///
    /// * `category` - The category name in the header.
    /// * `message` - The error message.
    /// * `span` - The source span of the error.
    /// * `help` - The optional help text.
    /// * `code` - The optional error code.
    ///
    /// # Returns
    ///
    /// The formatted error. The text ends with a line break.
    fn format_error(
        &self, category: &str, message: &str, span: &SourceSpan, help: Option<&str>, code: Option<ErrorCode>,
    ) -> String {
        let start_line = span.start().line();
        let start_column = span.start().column();
        let end_line = span.end().line();
        let end_column = span.end().column();

        let source_line = self.line_tracker.get_line(start_line).unwrap_or_default();

        let estimated_capacity =
            100 + message.len() + category.len() + source_line.len() + help.map_or(0, |value| value.len() + 20) + 50;

        let mut output = String::with_capacity(estimated_capacity);

        let _ = writeln!(
            output,
            "{}{}{}: {}",
            style("ERROR").red().bold(),
            code_prefix(code),
            style(category).red(),
            style(sanitize_output(message)).yellow(),
        );

        let _ = write!(output, "{} {}", style("Location:").blue(), style(sanitize_output(&span.to_string())).cyan());

        if !source_line.is_empty() {
            let _ = write!(output, "\n{start_line:4} │ {}", sanitize_output(source_line));

            let start_offset = start_column.saturating_sub(1);

            let underline = if start_line == end_line {
                let underline_length = end_column.saturating_sub(start_column).max(1);

                format!("{:>width$}{}", "", "^".repeat(underline_length), width = start_offset)
            } else {
                format!("{:>width$}^", "", width = start_offset)
            };

            let _ = write!(output, "\n     │ {}", style(&underline).red().bold());

            if start_line != end_line {
                let _ =
                    write!(output, "\n     │ {} (error spans lines {}-{})", style("...").blue(), start_line, end_line);
            }
        }

        if let Some(help) = help {
            let _ = write!(output, "\n{} {}", style("help:").blue().bold(), style(help).green());
        }

        output.push('\n');

        output
    }
}

/// Formats one error that has no source span.
///
/// The output is one line: `ERROR`, the optional code, `error_type`, and the message. The
/// function removes control characters from the message. The text ends with a line break.
///
/// # Arguments
///
/// * `error_type` - The category name in the header.
/// * `message` - The error message.
/// * `code` - The optional error code.
fn format_simple_error(error_type: &str, message: impl Display, code: Option<ErrorCode>) -> String {
    format!(
        "{}{}{}: {}\n",
        style("ERROR").red().bold(),
        code_prefix(code),
        style(error_type).red(),
        style(sanitize_output(&message.to_string())).yellow(),
    )
}

/// Formats the optional error code prefix of an error header.
///
/// With a code, the result is a space, the code in square brackets, and a space, for
/// example ` [E0001] `. Without a code, the result is one space.
fn code_prefix(code: Option<ErrorCode>) -> String {
    code.map_or_else(|| " ".to_string(), |code| format!(" [{}] ", style(code.code()).red().bold()))
}
