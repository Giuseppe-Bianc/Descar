//! # Utilities Module
//!
//! The utilities module provides helper functions, macros, and utilities that the
//! compiler uses in many places. It contains common functionality that does not
//! belong to a specific compilation phase.
use crate::location::source_location::SourceLocation;
use crate::location::source_location::UNKNOWN;
use crate::location::source_span::SourceSpan as Span;
use regex::Regex;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

use std::process::{Command, Stdio};

/// Returns the SHA-1 hash of the current Git commit.
///
/// The function runs `git rev-parse HEAD` in the current working directory.
/// The function discards the error output of the command.
///
/// # Returns
///
/// * `Some` with the hash as a string of 40 hexadecimal characters.
/// * `None` when the `git` command cannot start, when the command exits with an
///   error status, when the output is not valid UTF-8, or when the output is not
///   a string of 40 hexadecimal characters.
#[must_use]
pub fn get_git_commit_hash() -> Option<String> {
    let output =
        Command::new("git").args(["rev-parse", "HEAD"]).stdout(Stdio::piped()).stderr(Stdio::null()).output().ok()?; // Return `None` if the command cannot start.

    if !output.status.success() {
        return None;
    }

    let hash = String::from_utf8(output.stdout).ok()?;
    let hash = hash.trim();

    // Check the SHA-1 format: 40 hexadecimal characters.
    if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) { Some(hash.to_string()) } else { None }
}

/// Regular expression that matches ANSI escape sequences.
static ANSI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").expect("ANSI regex pattern is valid"));
/*static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("UUID regex pattern is valid")
});*/

/// Creates a default source span for tests.
///
/// Use this function when a test does not need real source location data.
///
/// # Returns
///
/// The value of [`SourceSpan::default`](crate::location::source_span::SourceSpan::default).
#[must_use]
pub fn dummy_span() -> Span {
    Span::default()
}

/// Removes ANSI escape sequences from a string.
///
/// The function removes the ANSI sequences that start with the escape character
/// and `[`, for example color and formatting sequences. The function keeps the
/// plain text. Use this function to compare terminal output in tests.
///
/// # Arguments
///
/// * `s` - The string that can contain ANSI escape sequences.
///
/// # Returns
///
/// A new string without the ANSI escape sequences.
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

/// Creates a source span from a file path and line and column numbers.
///
/// The start location has offset 0 and index 0. The end location has offset 1
/// and index 1. The UTF-8 offset and the code point offset of both locations
/// are [`UNKNOWN`].
///
/// # Arguments
///
/// * `file_path` - The path of the source file.
/// * `start_line` - The line number of the start location.
/// * `start_col` - The column number of the start location.
/// * `end_line` - The line number of the end location.
/// * `end_col` - The column number of the end location.
///
/// # Returns
///
/// A [`SourceSpan`](crate::location::source_span::SourceSpan) from the start location to the end location.
#[must_use]
pub fn create_span(file_path: &str, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
    Span::new(
        Arc::from(file_path),
        SourceLocation::new(start_line, start_col, 0, 0, UNKNOWN, UNKNOWN),
        SourceLocation::new(end_line, end_col, 1, 1, UNKNOWN, UNKNOWN),
    )
}

/// Creates a [`SourceSpan`](crate::location::source_span::SourceSpan) for a given line.
///
/// The span uses the file path `test_file`. The span starts at column 1 and
/// ends at column 2 of the given line.
///
/// # Arguments
///
/// * `line` - The line number of the span.
///
/// # Returns
///
/// The result of [`create_span`] with the arguments `"test_file"`, `line`, `1`, `line`, and `2`.
#[must_use]
pub fn t_span(line: usize) -> Span {
    create_span("test_file", line, 1, line, 2)
}

/// Creates a `let` binding for a `CompileError::<Variant>` value that has a default message and a span.
///
/// The macro sets the fields of the error value as follows:
///
/// * `code` is `None`.
/// * `message` is `Unexpected token "@"`.
/// * `span` is the result of `t_span(line)`.
/// * `help` is `None`, or the `help` expression when the call gives one.
///
/// Use the `mut` keyword before the variable name to create a mutable binding.
///
/// The caller must have `CompileError` and `t_span` in scope. The selected
/// variant must have the fields `code`, `message`, `span`, and `help`.
///
/// # Arguments
///
/// * `var` - The name of the new variable.
/// * `error_type` - The name of the `CompileError` variant.
/// * `line` - The line number that the macro passes to `t_span`.
/// * `help` - Optional. The expression for the `help` field.
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

/// Creates a `let` binding for a `CompileError::<Variant>` value that has a default message and no span.
///
/// The macro sets the fields of the error value as follows:
///
/// * `code` is `None`.
/// * `message` is `Unexpected token "@"`.
///
/// Use the `mut` keyword before the variable name to create a mutable binding.
///
/// The caller must have `CompileError` in scope. The selected variant must
/// have only the fields `code` and `message`.
///
/// # Arguments
///
/// * `var` - The name of the new variable.
/// * `error_type` - The name of the `CompileError` variant.
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
/// The pool stores the objects in a vector that a mutex protects. The pool does not
/// create objects. It returns only the objects that callers put in it.
/// All operations are thread-safe but may contend on a single mutex.
pub struct ObjectPool<T> {
    pool: Mutex<Vec<T>>,
}

impl<T> ObjectPool<T> {
    /// Creates an empty pool.
    ///
    /// # Returns
    ///
    /// A new pool that contains no objects.
    #[must_use]
    pub const fn new() -> Self {
        Self { pool: Mutex::new(Vec::new()) }
    }

    /// Removes the most recently released object from the pool and returns it.
    ///
    /// If another thread panicked while it held the lock of the pool, the method
    /// ignores the poisoned state of the lock and continues.
    ///
    /// # Returns
    ///
    /// * `Some` with the object that was released last.
    /// * `None` when the pool is empty.
    pub fn acquire(&self) -> Option<T> {
        match self.pool.lock() {
            Ok(mut guard) => guard.pop(),
            Err(poisoned) => {
                // The lock is poisoned. Use the data in the lock and continue.
                poisoned.into_inner().pop()
            }
        }
    }

    /// Puts an object in the pool.
    ///
    /// The pool has no maximum size. The method always stores the object.
    /// If another thread panicked while it held the lock of the pool, the method
    /// ignores the poisoned state of the lock and continues.
    ///
    /// # Arguments
    ///
    /// * `obj` - The object to store in the pool.
    pub fn release(&self, obj: T) {
        match self.pool.lock() {
            Ok(mut guard) => guard.push(obj),
            Err(poisoned) => {
                // The lock is poisoned. Use the data in the lock and continue.
                poisoned.into_inner().push(obj);
            }
        }
    }

    /// Creates an empty pool with space for a given number of objects.
    ///
    /// The pool does not store objects until callers release them. The pool
    /// allocates the space in advance, but it can store more than `capacity` objects.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of objects for which the pool reserves space.
    ///
    /// # Returns
    ///
    /// A new pool that contains no objects.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { pool: Mutex::new(Vec::with_capacity(capacity)) }
    }
}

impl<T> Default for ObjectPool<T> {
    /// Creates an empty pool. This method calls [`ObjectPool::new`].
    fn default() -> Self {
        Self::new()
    }
}
