//! Lexical analysis for Descar source code.
//!
//! The [`lexer`] module converts source text to tokens.
//! The [`error`] module defines the lexical error kinds that the lexer reports.

/// Lexical error kinds that the lexer reports.
pub mod error;
/// Source tokenizer that produces tokens one by one.
pub mod lexer;
