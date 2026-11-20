use crate::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    lex::error::LexError,
    location::line_tracker::LineTracker,
    tokens::{token::Token, token_kind::TokenKind},
};
use logos::Logos;
use std::sync::Arc;

/// Upper limit for the initial capacity of the token vector.
///
/// The limit prevents a large initial memory allocation for a large source text.
const MAX_TOKENS_CAPACITY: usize = 10_000_000;

/// Lexes Descar source text while preserving source locations.
///
/// The lexer produces one token at a time. After the last token of the source text,
/// the lexer produces one [`TokenKind::Eof`] token. Then the lexer produces no more results.
/// The type implements [`Iterator`]. Each item is a [`Token`] or a [`CompileError`].
pub struct Lexer<'a> {
    /// Tokenizer that matches the source text against the token kinds.
    inner: logos::Lexer<'a, TokenKind>,
    /// Tracker that converts byte ranges to source spans.
    line_tracker: LineTracker,
    /// Length of the source text in bytes.
    source_len: usize,
    /// `true` after the lexer returns the `Eof` token.
    eof_emitted: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a lexer for source text from the given file path.
    ///
    /// The lexer keeps a copy of the source text in its line tracker.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the source file. The line tracker uses the path for the spans.
    /// * `source` - The source text to tokenize.
    #[must_use]
    pub fn new(file_path: &str, source: &'a str) -> Self {
        let line_tracker = LineTracker::new(file_path, source.to_owned());
        let source_len = source.len();
        let inner = TokenKind::lexer(source);
        Self { inner, line_tracker, source_len, eof_emitted: false }
    }

    /// Returns the line and source-location tracker.
    ///
    /// # Returns
    ///
    /// A reference to the [`LineTracker`] that the lexer uses to create spans.
    #[must_use]
    pub const fn get_line_tracker(&self) -> &LineTracker {
        &self.line_tracker
    }

    /// Returns the next token or lexer error.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(token))` for a valid token. For a string literal or a character literal,
    ///   the method first checks the escape sequences.
    /// * `Some(Ok(token))` with the [`TokenKind::Eof`] kind at the end of the source text.
    ///   The span of this token is empty and starts at the end of the source text.
    ///   The lexer returns this token only once.
    /// * `Some(Err(error))` for a lexical error. The error is a [`CompileError::LexerError`]
    ///   with the error code and the span of the invalid text. The lexer does not stop
    ///   after an error.
    /// * `None` after the lexer returns the `Eof` token.
    #[inline]
    pub fn next_token(&mut self) -> Option<Result<Token, CompileError>> {
        if self.eof_emitted {
            return None;
        }

        let Some(result) = self.inner.next() else {
            self.eof_emitted = true;
            let range = self.source_len..self.source_len;
            let span = self.line_tracker.span_for(range);
            return Some(Ok(Token { kind: TokenKind::Eof, span }));
        };

        let range = self.inner.span();
        let span = self.line_tracker.span_for(range);
        let slice = self.inner.slice();

        match result {
            Ok(kind) => {
                if matches!(kind, TokenKind::StringLiteral(_) | TokenKind::CharLiteral(_))
                    && validate_escapes(slice).is_err()
                {
                    return Some(Err(Self::convert_lex_error(LexError::InvalidEscapeSequence, span, slice)));
                }

                Some(Ok(Token { kind, span }))
            }
            Err(error) => Some(Err(Self::convert_lex_error(error, span, slice))),
        }
    }

    /// Converts a [`LexError`] to a [`CompileError::LexerError`].
    ///
    /// The message has the form `<description>: "<slice>"`. The error code depends on the
    /// error kind:
    ///
    /// * `E0001`: [`LexError::InvalidToken`]
    /// * `E0002`: [`LexError::MalformedBinary`]
    /// * `E0003`: [`LexError::MalformedOctal`]
    /// * `E0004`: [`LexError::MalformedHexadecimal`]
    /// * `E0005`: [`LexError::UnterminatedString`]
    /// * `E0006`: [`LexError::UnterminatedChar`]
    /// * `E0007`: [`LexError::InvalidEscapeSequence`]
    /// * `E0008`: [`LexError::UnterminatedComment`]
    /// * `E0009`: [`LexError::InvalidNumberSuffix`]
    /// * `E0010`: [`LexError::NumberOverflow`]
    ///
    /// Each error kind has a help text, except `InvalidToken`, `MalformedBinary`,
    /// `MalformedOctal`, and `MalformedHexadecimal`. These kinds have no help text.
    ///
    /// # Arguments
    ///
    /// * `error` - The lexical error to convert.
    /// * `span` - The span of the text that caused the error.
    /// * `slice` - The text that caused the error. The message includes this text.
    fn convert_lex_error(error: LexError, span: crate::location::source_span::SourceSpan, slice: &str) -> CompileError {
        let (code, message, help) = match error {
            LexError::InvalidToken => (ErrorCode::E0001, format!("Invalid or unrecognized token: \"{slice}\""), None),
            LexError::MalformedBinary => (ErrorCode::E0002, format!("Malformed binary number: \"{slice}\""), None),
            LexError::MalformedOctal => (ErrorCode::E0003, format!("Malformed octal number: \"{slice}\""), None),
            LexError::MalformedHexadecimal => {
                (ErrorCode::E0004, format!("Malformed hexadecimal number: \"{slice}\""), None)
            }
            LexError::UnterminatedString => (
                ErrorCode::E0005,
                format!("Unterminated string literal: \"{slice}\""),
                Some(String::from("Add a closing double quote.")),
            ),
            LexError::UnterminatedChar => (
                ErrorCode::E0006,
                format!("Unterminated character literal: \"{slice}\""),
                Some(String::from("Add a closing single quote.")),
            ),
            LexError::InvalidEscapeSequence => (
                ErrorCode::E0007,
                format!("Invalid escape sequence: \"{slice}\""),
                Some(String::from("Use a supported escape sequence.")),
            ),
            LexError::UnterminatedComment => (
                ErrorCode::E0008,
                format!("Unterminated multi-line comment: \"{slice}\""),
                Some(String::from("Add a closing */ to the comment.")),
            ),
            LexError::InvalidNumberSuffix => (
                ErrorCode::E0009,
                format!("Invalid number suffix: \"{slice}\""),
                Some(String::from("Use a supported numeric suffix.")),
            ),
            LexError::NumberOverflow => (
                ErrorCode::E0010,
                format!("Number literal overflow: \"{slice}\""),
                Some(String::from("Use a smaller value or a compatible type.")),
            ),
        };

        CompileError::LexerError { code: Some(code), message: Arc::from(message), span, help }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, CompileError>;

    /// Returns the next token or lexer error. This method calls [`Lexer::next_token`].
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// Validates escape sequences in a string or character literal.
///
/// The function expects the complete literal, with the opening and closing quote
/// characters (`"` or `'`). The function checks only the text between the quotes.
///
/// Supported escapes are `\n`, `\r`, `\t`, `\\`, `\'`, `\"`, `\0`,
/// and Unicode escapes of the form `\u{HEX}` with one to six hexadecimal
/// digits representing a Unicode scalar value.
///
/// # Arguments
///
/// * `slice` - The complete text of the literal, including the quote characters.
///
/// # Errors
///
/// Returns [`LexError::InvalidEscapeSequence`] when the literal is not enclosed in
/// matching double quotes or matching single quotes, when a backslash is the last
/// character, or when an escape sequence is not supported.
fn validate_escapes(slice: &str) -> Result<(), LexError> {
    let inner = slice
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| slice.strip_prefix('\'').and_then(|value| value.strip_suffix('\'')))
        .ok_or(LexError::InvalidEscapeSequence)?;

    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            continue;
        }

        match chars.next() {
            Some('n' | 'r' | 't' | '\\' | '\'' | '"' | '0') => {}
            Some('u') => validate_unicode_escape(&mut chars)?,
            Some(_) | None => return Err(LexError::InvalidEscapeSequence),
        }
    }

    Ok(())
}

/// Validates the part of a Unicode escape sequence that follows `\u`.
///
/// The function reads the characters from `chars` in this order: an opening brace `{`,
/// one to six hexadecimal digits, and a closing brace `}`. The digits must give a
/// Unicode scalar value. The value cannot be larger than `0x10FFFF` and cannot be in the
/// surrogate range `0xD800` to `0xDFFF`.
///
/// The function advances `chars` up to the closing brace, or up to the character that
/// causes the error.
///
/// # Arguments
///
/// * `chars` - The characters that follow `\u` in the literal.
///
/// # Errors
///
/// Returns [`LexError::InvalidEscapeSequence`] when the opening brace is missing, when
/// the escape has no digits, has more than six digits, or contains a character that is
/// not a hexadecimal digit, when the closing brace is missing, or when the value is not a
/// Unicode scalar value.
fn validate_unicode_escape(chars: &mut std::str::Chars<'_>) -> Result<(), LexError> {
    let Some('{') = chars.next() else {
        return Err(LexError::InvalidEscapeSequence);
    };

    let mut digits = 0_u8;
    let mut value = 0_u32;

    loop {
        match chars.next() {
            Some('}') if digits > 0 => {
                if value > 0x10_FFFF || (0xD800..=0xDFFF).contains(&value) {
                    return Err(LexError::InvalidEscapeSequence);
                }
                return Ok(());
            }
            Some(ch) if ch.is_ascii_hexdigit() => {
                if digits == 6 {
                    return Err(LexError::InvalidEscapeSequence);
                }
                let digit = ch.to_digit(16).ok_or(LexError::InvalidEscapeSequence)?;
                value = (value << 4) | digit;
                digits += 1;
            }
            Some(_) | None => return Err(LexError::InvalidEscapeSequence),
        }
    }
}

/// Tokenizes the complete input and collects errors.
///
/// The function calls [`Lexer::next_token`] until the lexer returns `None`. The function
/// stores the valid tokens and the errors in two separate vectors. Each vector keeps the
/// order of the source text. The token vector includes the [`TokenKind::Eof`] token that
/// the lexer returns at the end of the source text.
///
/// The initial capacity of the token vector is one eighth of the source length in bytes.
/// The capacity is at least 1 and at most 10 million tokens.
///
/// # Arguments
///
/// * `lexer` - The lexer to read from. The function reads all remaining results.
///
/// # Returns
///
/// A tuple with the vector of tokens and the vector of errors.
pub fn lexer_tokenize_with_errors(lexer: &mut Lexer<'_>) -> (Vec<Token>, Vec<CompileError>) {
    let estimated_tokens = lexer.source_len / 8;
    let mut tokens = Vec::with_capacity(estimated_tokens.clamp(1, MAX_TOKENS_CAPACITY));
    let mut errors = Vec::new();

    while let Some(result) = lexer.next_token() {
        match result {
            Ok(token) => tokens.push(token),
            Err(error) => errors.push(error),
        }
    }

    (tokens, errors)
}
