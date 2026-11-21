// crates/descar-core/src/error/compile_error.rs

//! Compile errors with an optional code, a message, a source span, and optional help text.
//!
//! [`CompileError`] is the error type that the lexer, the parser, and the type checker
//! report.

use std::sync::Arc;

// src/error/compile_error.rs
use crate::error::error_code::ErrorCode;
use crate::location::source_span::SourceSpan as Span;
use thiserror::Error;

/// An error that the compiler reports while it processes Descar source code.
///
/// The variants are:
///
/// * [`CompileError::LexerError`]: An error of the lexical analysis.
/// * [`CompileError::SyntaxError`]: An error of the syntax analysis.
/// * [`CompileError::TypeError`]: An error of the type checker. The symbol table also uses this
///   variant.
/// * [`CompileError::IoError`]: An I/O error.
///
/// The first three variants have the same fields: an optional [`ErrorCode`], a message, a
/// source span, and optional help text. The `Display` text of these variants contains the
/// code in square brackets if a code is present, the message, the span, and the help text
/// on a separate line if help text is present.
///
/// The `Display` text of `LexerError` has no prefix before the message. The `Display` text of
/// `SyntaxError` has the prefix `Syntax error: ` and the `Display` text of `TypeError` has the
/// prefix `Type error: `.
#[derive(Debug, Error)]
pub enum CompileError {
    /// An error of the lexical analysis.
    ///
    /// The lexer reports errors with codes in the `E0xxx` range.
    #[error("{}{message} at {span}{}",
        .code.map_or(String::new(), |c| format!("[{}] ", c.code())),
        .help.as_ref().map_or(String::new(), |h| format!("\nhelp: {h}"))
    )]
    LexerError {
        /// The error code, if the error has one.
        code: Option<ErrorCode>,
        /// The description of the error.
        message: Arc<str>,
        /// The source span of the error.
        span: Span,
        /// Text that explains how to fix the error, if available.
        help: Option<String>,
    },

    /// An error of the syntax analysis.
    ///
    /// The parser reports errors with codes in the `E1xxx` range. The parser also reports
    /// `E2001`.
    #[error("{}Syntax error: {message} at {span}{}",
        .code.map_or(String::new(), |c| format!("[{}] ", c.code())),
        .help.as_ref().map_or(String::new(), |h| format!("\nhelp: {h}"))
    )]
    SyntaxError {
        /// The error code, if the error has one.
        code: Option<ErrorCode>,
        /// The description of the syntax violation.
        message: Arc<str>,
        /// The source span of the error.
        span: Span,
        /// Text that explains how to fix the error, if available.
        help: Option<String>,
    },

    /// An error of the type checker or of the symbol table.
    ///
    /// These errors have codes in the `E2xxx` range. The type checker also reports `E1003`.
    #[error("{}Type error: {message} at {span}{}",
        .code.map_or(String::new(), |c| format!("[{}] ", c.code())),
        .help.as_ref().map_or(String::new(), |h| format!("\nhelp: {h}"))
    )]
    TypeError {
        /// The error code, if the error has one.
        code: Option<ErrorCode>,
        /// The description of the error.
        message: Arc<str>,
        /// The source span of the error.
        span: Span,
        /// Text that explains how to fix the error, if available.
        help: Option<String>,
    },
    /*/// Error during intermediate representation (IR) generation.
    ///
    /// Contains:
    /// - `code`: Optional standardized error code (E3xxx range)
    /// - `message`: Description of the IR generation failure
    /// - `span`: Location associated with the error
    /// - `help`: Optional guidance for fixing the error
    #[error("{}IR generator error: {message} at {span}{}",
        .code.map_or(String::new(), |c| format!("[{}] ", c.code())),
        .help.as_ref().map_or(String::new(), |h| format!("\nhelp: {h}"))
    )]
    IrGeneratorError { code: Option<ErrorCode>, message: Arc<str>, span: SourceSpan, help: Option<String> },

    /// Error during assembly code generation.
    ///
    /// Contains:
    /// - `code`: Optional standardized error code (E4xxx range)
    /// - `message`: Description of the assembly generation failure
    #[error("{}Assembly generation error: {message}",
        .code.map_or(String::new(), |c| format!("[{}] ", c.code()))
    )]
    AsmGeneratorError { code: Option<ErrorCode>, message: Arc<str> },*/
    /// An I/O operation failed.
    ///
    /// The variant wraps a [`std::io::Error`]. The `From` implementation converts a
    /// [`std::io::Error`] into this variant.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl CompileError {
    /// Returns the error code of the error.
    ///
    /// # Returns
    ///
    /// `Some` with a reference to the code if the error has a code. `None` if the error has no
    /// code, or if the error is a [`CompileError::IoError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::error::error_code::ErrorCode;
    /// use descar_core::location::source_span::SourceSpan;
    /// use std::sync::Arc;
    /// let err = CompileError::TypeError {
    ///     code: Some(ErrorCode::E2023),
    ///     message: Arc::from("Undefined variable 'x'"),
    ///     span: SourceSpan::default(),
    ///     help: None,
    /// };
    /// assert_eq!(err.error_code(), Some(&ErrorCode::E2023));
    /// ```
    #[must_use]
    pub const fn error_code(&self) -> Option<&ErrorCode> {
        match self {
            Self::LexerError { code, .. } | Self::SyntaxError { code, .. } | Self::TypeError { code, .. } => {
                code.as_ref()
            }
            Self::IoError(_) => None,
        }
    }

    /// Returns the message of the error.
    ///
    /// # Returns
    ///
    /// `Some` with the message for [`CompileError::LexerError`], [`CompileError::SyntaxError`],
    /// and [`CompileError::TypeError`]. `None` for [`CompileError::IoError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::location::source_span::SourceSpan;
    /// use std::sync::Arc;
    /// let err = CompileError::LexerError {
    ///     code: None,
    ///     message: Arc::from("Invalid token"),
    ///     span: SourceSpan::default(),
    ///     help: None,
    /// };
    /// assert_eq!(err.message(), Some("Invalid token"));
    /// ```
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::LexerError { message, .. } | Self::SyntaxError { message, .. } | Self::TypeError { message, .. } => {
                Some(message)
            }
            Self::IoError(_) => None,
        }
    }

    /// Returns the source span of the error.
    ///
    /// # Returns
    ///
    /// `Some` with a reference to the span for [`CompileError::LexerError`],
    /// [`CompileError::SyntaxError`], and [`CompileError::TypeError`]. `None` for
    /// [`CompileError::IoError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::location::source_location::SourceLocation;
    /// use descar_core::location::source_span::SourceSpan;
    /// let span = SourceSpan::new(Arc::from("file"), SourceLocation::new(1,1,1,1,1,1), SourceLocation::new(1,1,1,1,1,1));
    /// let err = CompileError::SyntaxError {
    ///     code: None,
    ///     message: Arc::from("Unexpected token"),
    ///     span: span.clone(),
    ///     help: None,
    /// };
    /// assert_eq!(err.span(), Some(&span));
    /// ```
    #[must_use]
    pub const fn span(&self) -> Option<&Span> {
        match self {
            Self::LexerError { span, .. } | Self::SyntaxError { span, .. } | Self::TypeError { span, .. } => Some(span),
            Self::IoError(_) => None,
        }
    }

    /// Returns the help text of the error.
    ///
    /// # Returns
    ///
    /// `Some` with the help text if the error has help text. `None` if the error has no help
    /// text, or if the error is a [`CompileError::IoError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::location::source_span::SourceSpan;
    /// use std::sync::Arc;
    /// let err = CompileError::TypeError {
    ///     code: None,
    ///     message: Arc::from("Type mismatch"),
    ///     span: SourceSpan::default(),
    ///     help: Some("Try adding a type annotation".to_string()),
    /// };
    /// assert_eq!(err.help(), Some("Try adding a type annotation"));
    /// ```
    #[must_use]
    pub fn help(&self) -> Option<&str> {
        match self {
            Self::LexerError { help, .. } | Self::SyntaxError { help, .. } | Self::TypeError { help, .. } => {
                help.as_deref()
            }
            Self::IoError(_) => None,
        }
    }

    /// Replaces the message of the error.
    ///
    /// The function does nothing for [`CompileError::IoError`].
    ///
    /// # Arguments
    ///
    /// * `new_message` - The new message.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::location::source_span::SourceSpan;
    /// use std::sync::Arc;
    /// let mut err = CompileError::LexerError {
    ///     code: None,
    ///     message: Arc::from("Old message"),
    ///     span: SourceSpan::default(),
    ///     help: None,
    /// };
    /// err.set_message(Arc::from("New message"));
    /// assert_eq!(err.message(), Some("New message"));
    /// ```
    pub fn set_message(&mut self, new_message: Arc<str>) {
        match self {
            Self::LexerError { message, .. } | Self::SyntaxError { message, .. } | Self::TypeError { message, .. } => {
                *message = new_message;
            }
            Self::IoError(_) => {}
        }
    }

    /// Replaces the source span of the error.
    ///
    /// The function does nothing for [`CompileError::IoError`].
    ///
    /// # Arguments
    ///
    /// * `new_span` - The new source span.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::location::source_location::SourceLocation;
    /// use descar_core::location::source_span::SourceSpan;
    /// let mut err = CompileError::SyntaxError {
    ///     code: None,
    ///     message: Arc::from(""),
    ///     span: SourceSpan::new(Arc::from("file"), SourceLocation::new(1,1,1,1,1,1), SourceLocation::new(1,1,1,1,1,1)),
    ///     help: None,
    /// };
    /// let new_span = SourceSpan::new(Arc::from("file"), SourceLocation::new(1,2,1,1,1,1), SourceLocation::new(1,2,1,1,1,1));
    /// err.set_span(new_span.clone());
    /// assert_eq!(err.span(), Some(&new_span));
    /// ```
    pub fn set_span(&mut self, new_span: Span) {
        match self {
            Self::LexerError { span, .. } | Self::SyntaxError { span, .. } | Self::TypeError { span, .. } => {
                *span = new_span;
            }
            Self::IoError(_) => {}
        }
    }

    /// Replaces the help text of the error.
    ///
    /// The function does nothing for [`CompileError::IoError`].
    ///
    /// # Arguments
    ///
    /// * `new_help` - The new help text. `None` removes the existing help text.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::error::compile_error::CompileError;
    /// use descar_core::location::source_span::SourceSpan;
    /// use std::sync::Arc;
    /// let mut err = CompileError::TypeError {
    ///     code: None,
    ///     message: Arc::from("Type mismatch"),
    ///     span: SourceSpan::default(),
    ///     help: None,
    /// };
    /// err.set_help(Some("Try adding a type annotation".to_string()));
    /// assert_eq!(err.help(), Some("Try adding a type annotation"));
    /// ```
    pub fn set_help(&mut self, new_help: Option<String>) {
        match self {
            Self::LexerError { help, .. } | Self::SyntaxError { help, .. } | Self::TypeError { help, .. } => {
                *help = new_help;
            }
            Self::IoError(_) => {}
        }
    }
}
