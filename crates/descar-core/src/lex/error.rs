use std::fmt;

/// Errors produced directly by the lexical analyzer.
///
/// `LexError` describes the lexical problem itself. Conversion to the
/// compiler-facing `CompileError` is performed by the lexer wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LexError {
    /// The input does not start with any valid token.
    #[default]
    InvalidToken,

    /// A binary literal is malformed.
    ///
    /// Examples:
    /// - `#b`
    /// - `#b2`
    /// - `#b102`
    /// - `#b123u`
    MalformedBinary,

    /// An octal literal is malformed.
    ///
    /// Examples:
    /// - `#o`
    /// - `#o8`
    /// - `#o789`
    MalformedOctal,

    /// A hexadecimal literal is malformed.
    ///
    /// Examples:
    /// - `#x`
    /// - `#xG`
    /// - `#xDEADG`
    MalformedHexadecimal,

    /// A numeric literal contains an unsupported suffix.
    ///
    /// Examples:
    /// - `42i64`
    /// - `42u64`
    /// - `42i128`
    /// - `3.14f32`
    InvalidNumberSuffix,

    /// A numeric literal could not be represented by its requested type.
    NumberOverflow,

    /// A string or character literal contains an unsupported escape sequence.
    InvalidEscapeSequence,

    /// A string literal was not terminated.
    UnterminatedString,

    /// A character literal was not terminated.
    UnterminatedChar,

    /// A multi-line comment was not terminated.
    UnterminatedComment,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken => f.write_str("invalid token"),
            Self::MalformedBinary => f.write_str("malformed binary number"),
            Self::MalformedOctal => f.write_str("malformed octal number"),
            Self::MalformedHexadecimal => f.write_str("malformed hexadecimal number"),
            Self::InvalidNumberSuffix => f.write_str("invalid number suffix"),
            Self::NumberOverflow => f.write_str("number literal overflow"),
            Self::InvalidEscapeSequence => f.write_str("invalid escape sequence"),
            Self::UnterminatedString => f.write_str("unterminated string literal"),
            Self::UnterminatedChar => f.write_str("unterminated character literal"),
            Self::UnterminatedComment => f.write_str("unterminated multi-line comment"),
        }
    }
}
