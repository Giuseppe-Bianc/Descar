//! Compiler diagnostic types and reporting infrastructure.

/// Structured compile errors attached to source locations.
pub mod compile_error;
/// Stable compiler error codes and severity levels.
pub mod error_code;
/// Formatting and terminal rendering for diagnostics.
pub mod error_reporter;
/// Internal compiler errors.
pub mod internal;
