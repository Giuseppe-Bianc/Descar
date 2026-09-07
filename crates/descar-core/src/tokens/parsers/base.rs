use crate::{lex::error::LexError, tokens::number::Number, tokens::token_kind::TokenKind};

/// Parses a base-specific numeric literal.
///
/// The lexer callback uses this function to validate binary, octal and
/// hexadecimal literals after Logos has consumed the complete candidate.
///
/// # Arguments
///
/// * `radix` - Numeric base. Supported values are `2`, `8` and `16`.
/// * `lex` - Mutable Logos lexer containing the current token slice.
///
/// # Errors
///
/// Returns:
///
/// * [`LexError::MalformedBinary`] when `radix` is `2` and the literal
///   contains no digits or contains characters other than `0`, `1` and
///   the optional `u`/`U` suffix.
/// * [`LexError::MalformedOctal`] when `radix` is `8` and the literal
///   contains no digits or contains characters other than `0..=7` and
///   the optional `u`/`U` suffix.
/// * [`LexError::MalformedHexadecimal`] when `radix` is `16` and the
///   literal contains no digits or contains characters other than
///   hexadecimal digits and the optional `u`/`U` suffix.
/// * [`LexError::NumberOverflow`] when the validated value does not fit
///   in the selected signed or unsigned integer type.
/// * [`LexError::InvalidToken`] when an unsupported radix is supplied.
#[inline]
pub fn parse_base_number(radix: u32, lex: &mut logos::Lexer<TokenKind>) -> Result<Number, LexError> {
    let slice = lex.slice();

    if slice.len() < 2 {
        return Err(error_for_radix(radix));
    }

    let (_, number_with_suffix) = slice.split_at(2);

    let (number_part, unsigned) = match number_with_suffix.chars().last() {
        Some('u' | 'U') => {
            let split = number_with_suffix.len() - 1;

            (&number_with_suffix[..split], true)
        }
        _ => (number_with_suffix, false),
    };

    if number_part.is_empty() {
        return Err(error_for_radix(radix));
    }

    let valid = match radix {
        2 => number_part.bytes().all(|byte| matches!(byte, b'0' | b'1')),

        8 => number_part.bytes().all(|byte| (b'0'..=b'7').contains(&byte)),

        16 => number_part.bytes().all(|byte| byte.is_ascii_hexdigit()),

        _ => false,
    };

    if !valid {
        return Err(error_for_radix(radix));
    }

    if unsigned {
        u64::from_str_radix(number_part, radix).map(Number::UnsignedInteger).map_err(|_| LexError::NumberOverflow)
    } else {
        i64::from_str_radix(number_part, radix).map(Number::Integer).map_err(|_| LexError::NumberOverflow)
    }
}

const fn error_for_radix(radix: u32) -> LexError {
    match radix {
        2 => LexError::MalformedBinary,
        8 => LexError::MalformedOctal,
        16 => LexError::MalformedHexadecimal,
        _ => LexError::InvalidToken,
    }
}

/// Parses a binary numeric literal.
///
/// # Errors
///
/// Returns [`LexError::MalformedBinary`] if the literal contains missing
/// digits or invalid characters. Returns [`LexError::NumberOverflow`] if
/// the resulting value cannot be represented by the selected integer type.
#[inline]
pub fn parse_binary(lex: &mut logos::Lexer<TokenKind>) -> Result<Number, LexError> {
    parse_base_number(2, lex)
}

/// Parses an octal numeric literal.
///
/// # Errors
///
/// Returns [`LexError::MalformedOctal`] if the literal contains missing
/// digits or invalid characters. Returns [`LexError::NumberOverflow`] if
/// the resulting value cannot be represented by the selected integer type.
#[inline]
pub fn parse_octal(lex: &mut logos::Lexer<TokenKind>) -> Result<Number, LexError> {
    parse_base_number(8, lex)
}

/// Parses a hexadecimal numeric literal.
///
/// # Errors
///
/// Returns [`LexError::MalformedHexadecimal`] if the literal contains
/// missing digits or invalid characters. Returns
/// [`LexError::NumberOverflow`] if the resulting value cannot be represented
/// by the selected integer type.
#[inline]
pub fn parse_hex(lex: &mut logos::Lexer<TokenKind>) -> Result<Number, LexError> {
    parse_base_number(16, lex)
}
