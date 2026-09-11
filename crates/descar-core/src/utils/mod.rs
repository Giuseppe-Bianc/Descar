//! # Utilities Module
//!
//! The utilities module provides helper functions, macros, and utilities used
//! throughout the compiler. It contains common functionality that doesn't belong
//! to a specific compilation phase.
use crate::location::source_location::SourceLocation;
use crate::location::source_location::UNKNOWN;
use crate::location::source_span::SourceSpan as Span;
use regex::Regex;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::process::{Command, Stdio};

static ANSI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").expect("ANSI regex pattern is valid"));

#[must_use]
pub fn get_git_commit_hash() -> Option<String> {
    let output =
        Command::new("git").args(["rev-parse", "HEAD"]).stdout(Stdio::piped()).stderr(Stdio::null()).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let hash = String::from_utf8(output.stdout).ok()?;
    let hash = hash.trim();

    if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) { Some(hash.to_string()) } else { None }
}

/// Creates a dummy source span for testing purposes.
#[must_use]
pub fn dummy_span() -> Span {
    Span::default()
}

/// Strips ANSI escape codes from a string for easier comparison in tests.
#[must_use]
pub fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

#[must_use]
pub fn create_span(file_path: &str, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
    Span::new(
        Arc::from(file_path),
        SourceLocation::new(start_line, start_col, 0, 0, UNKNOWN, UNKNOWN),
        SourceLocation::new(end_line, end_col, 1, 1, UNKNOWN, UNKNOWN),
    )
}

#[must_use]
pub fn t_span(line: usize) -> Span {
    create_span("test_file", line, 1, line, 2)
}

#[macro_export]
macro_rules! make_error {
    ($var:ident, $error_type:ident, $line:expr) => {
        let $var = CompileError::$error_type {
            code: None,
            message: "Unexpected token \"@\"".into(),
            span: t_span($line),
            help: None,
        };
    };
    ($var:ident, $error_type:ident, $line:expr, $help:expr) => {
        let $var = CompileError::$error_type {
            code: None,
            message: "Unexpected token \"@\"".into(),
            span: t_span($line),
            help: $help,
        };
    };
    (mut $var:ident, $error_type:ident, $line:expr) => {
        let mut $var = CompileError::$error_type {
            code: None,
            message: "Unexpected token \"@\"".into(),
            span: t_span($line),
            help: None,
        };
    };
    (mut $var:ident, $error_type:ident, $line:expr, $help:expr) => {
        let mut $var = CompileError::$error_type {
            code: None,
            message: "Unexpected token \"@\"".into(),
            span: t_span($line),
            help: $help,
        };
    };
}

#[macro_export]
macro_rules! make_error_lineless {
    ($var:ident, $error_type:ident) => {
        let $var = CompileError::$error_type { code: None, message: "Unexpected token \"@\"".into() };
    };
    (mut $var:ident, $error_type:ident) => {
        let mut $var = CompileError::$error_type { code: None, message: "Unexpected token \"@\"".into() };
    };
}

pub struct ObjectPool<T> {
    pool: Mutex<Vec<T>>,
}

impl<T> ObjectPool<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self { pool: Mutex::new(Vec::new()) }
    }

    pub fn acquire(&self) -> Option<T> {
        match self.pool.lock() {
            Ok(mut guard) => guard.pop(),
            Err(poisoned) => poisoned.into_inner().pop(),
        }
    }

    pub fn release(&self, obj: T) {
        match self.pool.lock() {
            Ok(mut guard) => guard.push(obj),
            Err(poisoned) => poisoned.into_inner().push(obj),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { pool: Mutex::new(Vec::with_capacity(capacity)) }
    }
}

impl<T> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}
