use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::location::line_tracker::LineTracker;
use crate::location::source_span::SourceSpan;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use std::fmt::Display;

/// Enhanced error reporter using `ariadne` for source diagnostics.
pub struct ErrorReporter {
    line_tracker: LineTracker,
}

impl ErrorReporter {
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
                CompileError::LexerError {
                    message,
                    span,
                    help,
                    code,
                } => self.format_error(
                    "LEX",
                    &message,
                    &span,
                    help.as_deref(),
                    code,
                ),

                CompileError::SyntaxError {
                    message,
                    span,
                    help,
                    code,
                } => self.format_error(
                    "SYNTAX",
                    &message,
                    &span,
                    help.as_deref(),
                    code,
                ),

                CompileError::TypeError {
                    message,
                    span,
                    help,
                    code,
                } => self.format_error(
                    "TYPE",
                    &message,
                    &span,
                    help.as_deref(),
                    code,
                ),

                /*CompileError::IrGeneratorError {
                    message,
                    span,
                    help,
                    code,
                } => self.format_error(
                    "IR GEN",
                    &message,
                    &span,
                    help.as_deref(),
                    code,
                ),

                CompileError::AsmGeneratorError { message, code } => {
                    format_simple_error("ASM GEN", &message, code)
                }*/

                CompileError::IoError(error) => {
                    format_simple_error("I/O", &error, None)
                }
            };

            output.push_str(&formatted);
        }

        output
    }

    fn format_error(
        &self,
        category: &str,
        message: &str,
        span: &SourceSpan,
        help: Option<&str>,
        code: Option<ErrorCode>,
    ) -> String {
        let start = span.start().offset();
        let end = span.end().offset().max(start + 1);

        let primary_span = start..end;

        let mut report = Report::build(
            ReportKind::Error,
            primary_span.clone(),
        )
        .with_config(
            Config::default()
                .with_index_type(IndexType::Byte)
                .with_color(true)
                .with_underlines(true),
        )
        .with_message(format!(
            "{}{}: {}",
            code.map_or_else(
                || String::new(),
                |code| format!("[{}] ", code.code())
            ),
            category,
            message,
        ))
        .with_label(
            Label::new(primary_span)
                .with_color(Color::Red)
                .with_message(category),
        );

        if let Some(help) = help {
            report = report.with_help(help);
        }

        let report = report.finish();

        let mut buffer = Vec::with_capacity(512);

        report
            .write_for_stdout(
                Source::from(self.line_tracker.source()),
                &mut buffer,
            )
            .expect("writing Ariadne diagnostic should not fail");

        String::from_utf8(buffer)
            .expect("Ariadne diagnostic output must be valid UTF-8")
    }
}

fn format_simple_error(
    error_type: &str,
    message: impl Display,
    code: Option<ErrorCode>,
) -> String {
    match code {
        Some(code) => format!(
            "ERROR [{}] {}: {}\n",
            code.code(),
            error_type,
            message
        ),
        None => format!("ERROR {}: {}\n", error_type, message),
    }
}