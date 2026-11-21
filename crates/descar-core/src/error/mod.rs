// crates/descar-core/src/error/mod.rs

//! Diagnostic types and error reporting for the Descar compiler.
//!
//! * [`compile_error`](mod@compile_error): Compile errors that the lexer, the parser, and the type checker report.
//! * [`error_code`]: Error codes, severity levels, and compiler phases.
//! * [`error_reporter`]: Text output of compile errors with source context.
//! * [`internal`]: Errors about source files and I/O operations.

pub mod compile_error;
pub mod error_code;
pub mod error_reporter;
pub mod internal;
