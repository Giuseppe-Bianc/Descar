// src/tokens/parsers/mod.rs
//! Numeric literal parsing modules.
//!
//! This module contains all parsing logic for converting string representations
//! of numeric literals into structured `Number` types during lexical analysis.
//!
//! # Overview
//!
//! The parsers module provides four complementary submodules for parsing numeric
//! literals: base detection, numeric entry-point parsing, suffix parsing, and
//! numeric value construction.
//!
//! # Submodules
//!
//! - [`numeric`]: Logos-facing numeric literal entry point
//! - [`suffix`]: Numeric suffix parsing and dispatch
//! - [`value`]: Numeric value construction
//! - [`base`]: Numeric base detection (binary, octal, decimal, hexadecimal)
pub mod base;
pub mod numeric;
pub mod suffix;
pub mod value;
