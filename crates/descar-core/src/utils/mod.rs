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

/// Funzione helper per recuperare l'hash SHA-1 del commit corrente.
/// Restituisce `Some(hash)` se riesce, altrimenti `None`.
#[must_use]
pub fn get_git_commit_hash() -> Option<String> {
    let output =
        Command::new("git").args(["rev-parse", "HEAD"]).stdout(Stdio::piped()).stderr(Stdio::null()).output().ok()?; // ritorna None se il comando fallisce

    if !output.status.success() {
        return None;
    }

    let hash = String::from_utf8(output.stdout).ok()?;
    let hash = hash.trim();

    // Verifica formato SHA-1
    if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) { Some(hash.to_string()) } else { None }
}

static ANSI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").expect("ANSI regex pattern is valid"));
/*static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("UUID regex pattern is valid")
});*/

/// Creates a dummy source span for testing purposes.
///
/// Returns a default-initialized `SourceSpan` suitable for use in unit tests
/// where actual source location information is not needed.
///
/// # Returns
///
/// A default `SourceSpan` with no meaningful location data.
#[must_use]
pub fn dummy_span() -> Span {
    Span::default()
}

/// Strips ANSI escape codes from a string for easier comparison in tests.
///
/// Removes all ANSI color and formatting codes, leaving only the plain text.
/// Useful for testing terminal output without dealing with formatting codes.
///
/// # Arguments
///
/// * `s` - String containing ANSI escape sequences
///
/// # Returns
///
/// A new string with all ANSI codes removed.
///
/// # Examples
///
/// ```
/// use descar_core::utils::strip_ansi_codes;
/// let colored = "\x1B[31mError\x1B[0m";
/// let plain = strip_ansi_codes(colored);
/// assert_eq!(plain, "Error");
/// ```
#[must_use]
pub fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

// Test di merging
#[must_use]
pub fn create_span(file_path: &str, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
    Span::new(
        Arc::from(file_path),
        SourceLocation::new(start_line, start_col, 0, 0, UNKNOWN, UNKNOWN),
        SourceLocation::new(end_line, end_col, 1, 1, UNKNOWN, UNKNOWN),
    )
}

/// Helper function to create a `SourceSpan` for a given line.
#[must_use]
pub fn t_span(line: usize) -> Span {
    create_span("test_file", line, 1, line, 2)
}

/// Helper macro to construct a `CompileError::<Variant>` instance, optionally mutable, with a default message but no span.
#[macro_export]
macro_rules! make_error {
    // Immutable binding
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
    // Mutable binding
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

/// Helper macro to construct a `CompileError::<Variant>` instance, optionally mutable, with a default message and span.
#[macro_export]
macro_rules! make_error_lineless {
    // Immutable binding
    ($var:ident, $error_type:ident) => {
        let $var = CompileError::$error_type { code: None, message: "Unexpected token \"@\"".into() };
    };
    // Mutable binding
    (mut $var:ident, $error_type:ident) => {
        let mut $var = CompileError::$error_type { code: None, message: "Unexpected token \"@\"".into() };
    };
}

/*#[must_use]
pub const fn int_type() -> Type {
    Type::I32
}

#[must_use]
pub fn sanitize_uuids(input: &str) -> String {
    sanitize_uuids_with_prefix(input, "SCOPE_")
}

#[must_use]
pub fn sanitize_mdata_uuids(input: &str) -> String {
    sanitize_uuids_with_prefix(input, "UUID_")
}

#[must_use]
fn sanitize_uuids_with_prefix(input: &str, prefix: &str) -> String {
    let mut counter = 0;
    let mut mapping = HashMap::new();

    UUID_REGEX
        .replace_all(input, |captures: &regex::Captures| {
            let uuid = captures.get(0).unwrap().as_str();
            let id = *mapping.entry(uuid.to_string()).or_insert_with(|| {
                let id = counter;
                counter += 1;
                id
            });
            format!("{prefix}{id}")
        })
        .to_string()
}

#[must_use]
pub fn vec_to_string<T: Display>(vec: Vec<T>) -> String {
    sanitize_uuids(vec.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ").as_str())
}*/

/// Thread-safe object pool for reusing frequently allocated objects.
///
/// This pool reduces allocation overhead by storing and reusing objects of type `T`.
/// All operations are thread-safe but may contend on a single mutex.
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
            Err(poisoned) => {
                // Clear the poisoned state and continue
                poisoned.into_inner().pop()
            }
        }
    }
    pub fn release(&self, obj: T) {
        match self.pool.lock() {
            Ok(mut guard) => guard.push(obj),
            Err(poisoned) => {
                // Clear poisoned state and continue
                poisoned.into_inner().push(obj);
            }
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
