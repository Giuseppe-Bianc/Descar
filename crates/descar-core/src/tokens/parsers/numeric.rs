use crate::{
    lex::error::LexError,
    tokens::{
        number::Number,
        parsers::{
            suffix::{handle_suffix, split_numeric_and_suffix},
            value,
        },
        token_kind::TokenKind,
    },
};

/// Parses a decimal numeric literal.
///
/// The callback receives the complete numeric candidate from Logos,
/// separates the optional type suffix and converts the value into the
/// corresponding [`Number`] variant.
///
/// # Errors
///
/// Returns [`LexError::InvalidNumberSuffix`] when the candidate contains an
/// unsupported numeric suffix. Returns [`LexError::NumberOverflow`] when the
/// numeric value or exponent cannot be represented by the requested
/// numeric type.
#[inline]
pub fn parse_number(lex: &mut logos::Lexer<TokenKind>) -> Result<Number, LexError> {
    let slice = lex.slice();

    let (numeric_part, suffix) = split_numeric_and_suffix(slice);

    handle_suffix(numeric_part, suffix)
}

pub use value::{
    handle_default_suffix, handle_f64_suffix, handle_float_suffix, handle_non_scientific, is_valid_integer_literal,
    parse_integer, parse_scientific,
};
