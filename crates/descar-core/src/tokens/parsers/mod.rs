// src/tokens/parsers/mod.rs
//! Parsing of numeric literals.
//!
//! This module converts numeric text into [`Number`](crate::tokens::number::Number)
//! values during lexical analysis.
//!
//! The module provides these submodules:
//! - [`base`] detects and parses binary, octal, and hexadecimal literals.
//! - [`numeric`] provides the decimal literal entry point.
//! - [`suffix`] identifies numeric suffixes and routes literals to value parsers.
//! - [`value`] constructs numeric values.

/// Parsing arbitrary integer bases.
pub mod base;
/// Numeric token dispatch.
pub mod numeric;
/// Recognition and dispatch of numeric suffixes.
pub mod suffix;
/// Integer and floating-point literal values.
pub mod value;
