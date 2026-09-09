use crate::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    lex::error::LexError,
    location::line_tracker::LineTracker,
    tokens::{token::Token, token_kind::TokenKind},
};
use logos::Logos;
use std::sync::Arc;

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
    line_tracker: LineTracker,
    source_len: usize,
    eof_emitted: bool,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(file_path: &str, source: &'a str) -> Self {
        let line_tracker = LineTracker::new(file_path, source.to_owned());
        let source_len = source.len();
        let inner = TokenKind::lexer(source);
        Self { inner, line_tracker, source_len, eof_emitted: false }
    }

    #[must_use]
    pub const fn get_line_tracker(&self) -> &LineTracker {
        &self.line_tracker
    }

    #[inline]
    pub fn next_token(&mut self) -> Option<Result<Token, CompileError>> {
        if self.eof_emitted {
            return None;
        }

        let next = self.inner.next();
        let (result, range) = if let Some(result) = next {
            let range = self.inner.span();
            (result, range)
        } else {
            self.eof_emitted = true;
            let range = self.source_len..self.source_len;
            (Ok(TokenKind::Eof), range)
        };

        let span = self.line_tracker.span_for(range);
        let slice = self.inner.slice();

        Some(match result {
            Ok(kind) => {
                if matches!(kind, TokenKind::StringLiteral(_) | TokenKind::CharLiteral(_)) {
                    if let Err(error) = validate_escapes(slice) {
                        return Some(Err(Self::convert_lex_error(error, span, slice)));
                    }
                }
                Ok(Token { kind, span })
            }
            Err(error) => Err(Self::convert_lex_error(error, span, slice)),
        })
    }

    fn convert_lex_error(
        error: LexError,
        span: crate::location::source_span::SourceSpan,
        slice: &str,
    ) -> CompileError {
        let (code, message, help) = match error {
            LexError::InvalidToken => (
                ErrorCode::E0001,
                format!("Invalid or unrecognized token: \"{slice}\""),
                None,
            ),
            LexError::MalformedBinary => (
                ErrorCode::E0002,
                format!("Malformed binary number: \"{slice}\""),
                None,
            ),
            LexError::MalformedOctal => (
                ErrorCode::E0003,
                format!("Malformed octal number: \"{slice}\""),
                None,
            ),
            LexError::MalformedHexadecimal => (
                ErrorCode::E0004,
                format!("Malformed hexadecimal number: \"{slice}\""),
                None,
            ),
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
                String::from("Invalid number suffix"),
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

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// Validates escape sequences in a string or character literal.
///
/// Supported escapes are n, r, t, backslash, single quote, double quote,
/// zero, and Unicode escapes of the form backslash-u followed by braces
/// containing one to six hexadecimal digits.
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

fn validate_unicode_escape(chars: &mut std::str::Chars<'_>) -> Result<(), LexError> {
    if chars.next() != Some('{') {
        return Err(LexError::InvalidEscapeSequence);
    }

    let mut digits = 0;
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
                value = (value << 4) | ch.to_digit(16).ok_or(LexError::InvalidEscapeSequence)?;
                digits += 1;
            }
            Some(_) | None => return Err(LexError::InvalidEscapeSequence),
        }
    }
}

/// Tokenizes the complete input and collects errors.
pub fn lexer_tokenize_with_errors(lexer: &mut Lexer<'_>) -> (Vec<Token>, Vec<CompileError>) {
    let estimated_tokens = lexer.source_len / 8;
    let mut tokens = Vec::with_capacity(estimated_tokens.max(1));
    let mut errors = Vec::new();

    while let Some(result) = lexer.next_token() {
        match result {
            Ok(token) => tokens.push(token),
            Err(error) => errors.push(error),
        }
    }

    (tokens, errors)
}
