use console::style;

use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::location::line_tracker::LineTracker;
use crate::location::source_span::SourceSpan;
use std::fmt::{Display, Write};

/// Enhanced error reporter with source context display.
pub struct ErrorReporter {
    line_tracker: LineTracker,
}

impl ErrorReporter {
    /// Creates a new error reporter backed by the given line tracker.
    #[must_use]
    pub const fn new(line_tracker: LineTracker) -> Self {
        Self { line_tracker }
    }

    /// Returns a formatted string containing all compile errors with source context.
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

                // Mantieni questo ramo quando riattiverai IrGeneratorError.
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

                // Mantieni questo ramo quando riattiverai AsmGeneratorError.
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

    /// Formats an error with source context and visual indicators.
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
            style(message).yellow(),
        );

        let _ = write!(output, "{} {}", style("Location:").blue(), style(&span.to_string()).cyan());

        if !source_line.is_empty() {
            let _ = write!(output, "\n{start_line:4} │ {source_line}");

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

/// Formats an error that has no source span.
fn format_simple_error(error_type: &str, message: impl Display, code: Option<ErrorCode>) -> String {
    format!(
        "{}{}{}: {}\n",
        style("ERROR").red().bold(),
        code_prefix(code),
        style(error_type).red(),
        style(message).yellow(),
    )
}

/// Formats the optional error code prefix.
///
/// With a code:
/// ` [E0001] `
///
/// Without a code:
/// ` `
fn code_prefix(code: Option<ErrorCode>) -> String {
    code.map_or_else(|| " ".to_string(), |code| format!(" [{}] ", style(code.code()).red().bold()))
}
