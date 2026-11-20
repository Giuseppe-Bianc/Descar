//! Core compiler infrastructure for Descar.
//!
//! This crate contains the compiler functions. The crate intentionally has no
//! dependency on the command-line interface.
//!
//! The crate has these modules:
//!
//! * [`error`]: Diagnostic types and reporting.
//! * [`file`](mod@file): File and size utilities.
//! * [`lex`]: Lexical analysis of Descar source code.
//! * [`location`]: Source positions, spans, and line tracking.
//! * [`printers`]: Syntax-tree and terminal output utilities.
//! * [`semantic`]: Semantic analysis, symbols, scopes, and type checking.
//! * [`syntax`]: Parsing and syntax-tree definitions.
//! * [`tokens`]: Lexer tokens and numeric literal representations.
//! * [`utils`]: Helper functions and utilities that the compiler uses.

pub mod error;
pub mod file;
pub mod lex;
pub mod location;
pub mod printers;
pub mod semantic;
pub mod syntax;
pub mod tokens;
pub mod utils;
