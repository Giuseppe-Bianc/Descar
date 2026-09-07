use super::numeric::{handle_default_suffix, handle_float_suffix, parse_integer};
use crate::{lex::error::LexError, tokens::number::Number};

/// Splits a numeric literal into its numeric part and optional suffix.
///
/// The suffix is matched case-insensitively while the returned slice keeps
/// its original spelling.
#[must_use]
pub fn split_numeric_and_suffix(slice: &str) -> (&str, Option<&str>) {
    if slice.is_empty() {
        return (slice, None);
    }

    let bytes = slice.as_bytes();

    // Three-character suffixes: i16, i32, u16, u32.
    if bytes.len() >= 3 {
        let suffix = &bytes[bytes.len() - 3..];

        let valid = matches!(suffix[0].to_ascii_lowercase(), b'i' | b'u')
            && matches!((suffix[1], suffix[2]), (b'1', b'6') | (b'3', b'2'));

        if valid {
            let split = bytes.len() - 3;

            return (&slice[..split], Some(&slice[split..]));
        }
    }

    // Two-character suffixes: i8, u8.
    if bytes.len() >= 2 {
        let suffix = &bytes[bytes.len() - 2..];

        let valid = matches!(suffix[0].to_ascii_lowercase(), b'i' | b'u') && suffix[1] == b'8';

        if valid {
            let split = bytes.len() - 2;

            return (&slice[..split], Some(&slice[split..]));
        }
    }

    // One-character suffixes: u, f, d.
    match bytes.last().copied() {
        Some(b'u' | b'U' | b'f' | b'F' | b'd' | b'D') => {
            let split = bytes.len() - 1;

            (&slice[..split], Some(&slice[split..]))
        }

        _ => (slice, None),
    }
}

/// Routes the numeric literal to the appropriate parser.
///
/// # Errors
///
/// Returns [`LexError::InvalidNumberSuffix`] when the supplied suffix is not
/// supported. Returns [`LexError::NumberOverflow`] when the selected numeric
/// parser cannot represent the value.
pub fn handle_suffix(numeric_part: &str, suffix: Option<&str>) -> Result<Number, LexError> {
    match suffix.map(str::to_ascii_lowercase).as_deref() {
        Some("u") => parse_integer::<u64>(numeric_part, Number::UnsignedInteger),

        Some("u8") => parse_integer::<u8>(numeric_part, Number::U8),

        Some("u16") => parse_integer::<u16>(numeric_part, Number::U16),

        Some("u32") => parse_integer::<u32>(numeric_part, Number::U32),

        Some("i8") => parse_integer::<i8>(numeric_part, Number::I8),

        Some("i16") => parse_integer::<i16>(numeric_part, Number::I16),

        Some("i32") => parse_integer::<i32>(numeric_part, Number::I32),

        Some("f") => handle_float_suffix(numeric_part),

        None | Some("d") => handle_default_suffix(numeric_part),

        Some(_) => Err(LexError::InvalidNumberSuffix),
    }
}
