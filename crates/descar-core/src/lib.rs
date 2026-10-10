//! Core compiler infrastructure for Descar.
//!
//! This crate contains compiler-facing functionality and intentionally has no
//! dependency on the command-line interface.
//!
//! # Logging
//!
//! The crate emits [`tracing`] events and never installs a subscriber. The binary decides
//! where the events go and which levels are shown.
//!
//! Rules for new events:
//! - never log source text, which includes literals, identifiers, token text and diagnostic
//!   messages;
//! - never log file paths, because the command span of the binary already carries the file;
//! - describe a location with `line` and `column`, and a diagnostic with its error code;
//! - use `warn` for broken invariants and recovered failures, `debug` for diagnostics and
//!   decisions, and `trace` for fine-grained state changes;
//! - do not log per token or per expression, which are hot paths.

pub mod error;
pub mod file;
pub mod lex;
pub mod location;
pub mod printers;
pub mod semantic;
pub mod syntax;
pub mod tokens;
pub mod utils;
