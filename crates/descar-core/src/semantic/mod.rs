// crates/descar-core/src/semantic/mod.rs

//! Semantic analysis of Descar programs.
//!
//! The parser creates a syntax tree. The semantic analysis reads this tree and reports
//! semantic errors, for example undefined names and type mismatches.
//!
//! * [`symbol_table`]: Symbols, lexical scopes, and the current function context.
//! * [`type_checker`]: Type checking of statements and expressions.

pub mod symbol_table;
pub mod type_checker;
