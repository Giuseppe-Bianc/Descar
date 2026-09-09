use super::numeric::{handle_default_suffix, handle_float_suffix, parse_integer};
use crate::{lex::error::LexError, tokens::number::Number};

/// Supported decimal numeric suffixes.
///
/// The language deliberately does not support explicit `i64` or `u64` suffixes.
/// An unsuffixed integer defaults to `i64`, while `u`/`U` requests `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericSuffix {
    U64,
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    F32,
    F64,
}

impl NumericSuffix {
    /// Canonical spellings of all supported suffixes.
    ///
    /// The order is longest-first so suffix recognition follows longest-match
    /// semantics without duplicating the supported suffix set elsewhere.
    const SPELLINGS: &[(&str, Self)] = &[
        ("i16", Self::I16),
        ("I16", Self::I16),
        ("i32", Self::I32),
        ("I32", Self::I32),
        ("u16", Self::U16),
        ("U16", Self::U16),
        ("u32", Self::U32),
        ("U32", Self::U32),
        ("i8", Self::I8),
        ("I8", Self::I8),
        ("u8", Self::U8),
        ("U8", Self::U8),
        ("u", Self::U64),
        ("U", Self::U64),
        ("f", Self::F32),
        ("F", Self::F32),
        ("d", Self::F64),
        ("D", Self::F64),
    ];

    /// Parses a suffix case-insensitively according to the language rules.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::SPELLINGS
            .iter()
            .find_map(|(spelling, suffix)| (*spelling == value).then_some(*suffix))
    }

    /// Finds the longest supported suffix at the end of a numeric candidate.
    #[must_use]
    fn split_supported(slice: &str) -> Option<(&str, &str)> {
        Self::SPELLINGS
            .iter()
            .filter_map(|(spelling, _)| {
                slice
                    .strip_suffix(spelling)
                    .filter(|numeric_part| !numeric_part.is_empty())
                    .map(|numeric_part| (numeric_part, *spelling))
            })
            .max_by_key(|(_, suffix)| suffix.len())
    }
}

/// Splits a numeric literal into its numeric part and optional suffix.
///
/// Supported suffixes are matched case-insensitively while the original
/// spelling is preserved in the returned slice. Unknown suffixes are also
/// separated so the caller can report `InvalidNumberSuffix` for the complete
/// malformed numeric candidate instead of splitting it into unrelated tokens.
#[must_use]
pub fn split_numeric_and_suffix(slice: &str) -> (&str, Option<&str>) {
    if let Some((numeric_part, suffix)) = NumericSuffix::split_supported(slice) {
        return (numeric_part, Some(suffix));
    }

    let suffix_start = slice
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_alphabetic())
        .map(|(index, _)| index);

    match suffix_start {
        Some(index) if index > 0 => (&slice[..index], Some(&slice[index..])),
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
    let Some(raw_suffix) = suffix else {
        return handle_default_suffix(numeric_part);
    };

    let Some(suffix) = NumericSuffix::parse(raw_suffix) else {
        return Err(LexError::InvalidNumberSuffix);
    };

    match suffix {
        NumericSuffix::U64 => parse_integer::<u64>(numeric_part, Number::UnsignedInteger),
        NumericSuffix::U8 => parse_integer::<u8>(numeric_part, Number::U8),
        NumericSuffix::U16 => parse_integer::<u16>(numeric_part, Number::U16),
        NumericSuffix::U32 => parse_integer::<u32>(numeric_part, Number::U32),
        NumericSuffix::I8 => parse_integer::<i8>(numeric_part, Number::I8),
        NumericSuffix::I16 => parse_integer::<i16>(numeric_part, Number::I16),
        NumericSuffix::I32 => parse_integer::<i32>(numeric_part, Number::I32),
        NumericSuffix::F32 => handle_float_suffix(numeric_part),
        NumericSuffix::F64 => handle_default_suffix(numeric_part),
    }
}
