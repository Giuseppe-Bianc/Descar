// crates/descar-core/src/syntax/mod.rs

//! Parsing and syntax-tree definitions.
//!
//! * [`ast`]: Data types of the abstract syntax tree.
//! * [`parser`]: The parser that converts tokens into a syntax tree.
//! * [`precendence`]: Binding powers of operators.

/// Abstract syntax tree types.
pub mod ast;
/// Descar source parser.
pub mod parser;
/// Binding powers of operators for the expression parser.
pub mod precendence;