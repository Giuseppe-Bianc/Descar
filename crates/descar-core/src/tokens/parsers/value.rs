use crate::{lex::error::LexError, tokens::number::Number};

/// Parses an integer literal into the requested numeric representation.
///
/// # Errors
///
/// Returns [`LexError::InvalidNumberSuffix`] when `numeric_part` is not a
/// valid unsigned decimal integer. Returns [`LexError::NumberOverflow`] when
/// parsing the integer fails because the value is outside the target type's
/// representable range.
pub fn parse_integer<T>(numeric_part: &str, map_fn: fn(T) -> Number) -> Result<Number, LexError>
where
    T: std::str::FromStr,
{
    if !is_valid_integer_literal(numeric_part) {
        return Err(LexError::InvalidNumberSuffix);
    }

    numeric_part.parse::<T>().map(map_fn).map_err(|_| LexError::NumberOverflow)
}

/// Checks whether a string contains only decimal digits.
///
/// This function does not classify parsing errors because it does not return a
/// `Result`.
#[must_use]
pub fn is_valid_integer_literal(numeric_part: &str) -> bool {
    !numeric_part.is_empty() && numeric_part.bytes().all(|byte| byte.is_ascii_digit())
}

/// Parses a floating-point literal with an explicit `f` suffix.
///
/// Scientific notation is preserved as [`Number::Scientific32`].
/// Ordinary floating-point notation becomes [`Number::Float32`].
///
/// # Errors
///
/// Returns [`LexError::NumberOverflow`] when the numeric value cannot be
/// represented as an `f32` or the scientific exponent is invalid.
pub fn handle_float_suffix(numeric_part: &str) -> Result<Number, LexError> {
    if let Some(number) = parse_scientific(numeric_part, true) {
        return Ok(number);
    }

    numeric_part.parse::<f32>().map(Number::Float32).map_err(|_| LexError::NumberOverflow)
}

/// Parses an unsuffixed number as its default numeric type.
///
/// Integers are represented as `i64`, ordinary floating-point values as `f64`,
/// and scientific notation as [`Number::Scientific64`].
///
/// # Errors
///
/// Returns [`LexError::NumberOverflow`] when the numeric value cannot be
/// represented by the selected type.
pub fn handle_default_suffix(numeric_part: &str) -> Result<Number, LexError> {
    if let Some(number) = parse_scientific(numeric_part, false) {
        return Ok(number);
    }

    handle_non_scientific(numeric_part)
}

/// Parses an explicitly `d`-suffixed numeric literal as `f64`.
///
/// Scientific notation is preserved as [`Number::Scientific64`]. An ordinary
/// numeric spelling, including an integer-shaped spelling such as `100d`, is
/// represented as [`Number::Float64`] so the suffix consistently requests
/// the 64-bit floating-point type.
///
/// # Errors
///
/// Returns [`LexError::NumberOverflow`] when the numeric value cannot be
/// represented as an `f64` or the scientific exponent is invalid.
pub fn handle_f64_suffix(numeric_part: &str) -> Result<Number, LexError> {
    if let Some(number) = parse_scientific(numeric_part, false) {
        return Ok(number);
    }

    numeric_part.parse::<f64>().map(Number::Float64).map_err(|_| LexError::NumberOverflow)
}

/// Parses a non-scientific numeric literal.
///
/// # Errors
///
/// Returns [`LexError::NumberOverflow`] when the numeric value cannot be
/// represented as `i64` or `f64`.
pub fn handle_non_scientific(numeric_part: &str) -> Result<Number, LexError> {
    if numeric_part.contains('.') {
        numeric_part.parse::<f64>().map(Number::Float64).map_err(|_| LexError::NumberOverflow)
    } else {
        numeric_part.parse::<i64>().map(Number::Integer).map_err(|_| LexError::NumberOverflow)
    }
}

/// Parses scientific notation.
///
/// The returned number stores the mantissa and exponent separately rather
/// than calculating the final numeric value.
///
/// # Errors
///
/// This function returns `Option` rather than `Result`. `None` indicates that
/// the input is not valid scientific notation or that its exponent or
/// mantissa cannot be parsed.
#[must_use]
pub fn parse_scientific(source: &str, is_f32: bool) -> Option<Number> {
    let exponent_position = source.find(['e', 'E'])?;

    let (base_str, exponent_str) = source.split_at(exponent_position);

    let exponent = exponent_str.get(1..)?.parse::<i32>().ok()?;

    if is_f32 {
        let base = base_str.parse::<f32>().ok()?;

        Some(Number::Scientific32(base, exponent))
    } else {
        let base = base_str.parse::<f64>().ok()?;

        Some(Number::Scientific64(base, exponent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integer_value() {
        assert_eq!(parse_integer::<u32>("42", Number::U32).unwrap(), Number::U32(42));
    }

    #[allow(clippy::approx_constant)]
    #[test]
    fn parses_default_integer_and_float_values() {
        assert_eq!(handle_default_suffix("42").unwrap(), Number::Integer(42));
        assert_eq!(handle_default_suffix("3.14").unwrap(), Number::Float64(3.14));
    }

    #[test]
    fn preserves_scientific_values() {
        assert_eq!(handle_float_suffix("6.02e23").unwrap(), Number::Scientific32(6.02, 23));
        assert_eq!(handle_f64_suffix("6.02e23").unwrap(), Number::Scientific64(6.02, 23));
    }
}
