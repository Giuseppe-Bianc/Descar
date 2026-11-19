//! Lexer tokens and numeric literal representations.

/// Arbitrary numeric literal values.
pub mod number;
/// Numeric token parsers.
pub mod parsers;
/// Tokens with source spans.
pub mod token;
/// Token kinds recognized by the lexer.
pub mod token_kind;
/// Display implementations for token kinds.
pub mod token_kind_display;
