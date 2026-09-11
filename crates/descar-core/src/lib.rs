//! Core compiler infrastructure for Descar.
//!
//! This crate contains compiler-facing functionality and intentionally has no
//! dependency on the command-line interface.

pub mod error;
pub mod file;
pub mod lex;
pub mod location;
pub mod printers;
pub mod syntax;
pub mod tokens;
pub mod utils;
