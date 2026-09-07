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

        Some(match result {
            Ok(kind) => Ok(Token { kind, span }),

            Err(error) => Err(Self::convert_lex_error(error, span)),
        })
    }

    fn convert_lex_error(error: LexError, span: crate::location::source_span::SourceSpan) -> CompileError {
        let (code, message, help) = match error {
            LexError::InvalidToken => (
                ErrorCode::E0001,
                "Invalid or unrecognized token",
                Some(
                    "Check for unsupported characters \
                         or malformed syntax.",
                ),
            ),

            LexError::MalformedBinary => {
                (ErrorCode::E0002, "Malformed binary number", Some("Binary numbers use the form #b1010."))
            }

            LexError::MalformedOctal => {
                (ErrorCode::E0003, "Malformed octal number", Some("Octal numbers use the form #o755."))
            }

            LexError::MalformedHexadecimal => {
                (ErrorCode::E0004, "Malformed hexadecimal number", Some("Hexadecimal numbers use the form #xDEAD."))
            }

            LexError::UnterminatedString => {
                (ErrorCode::E0005, "Unterminated string literal", Some("Add a closing double quote."))
            }

            LexError::UnterminatedChar => {
                (ErrorCode::E0006, "Unterminated character literal", Some("Add a closing single quote."))
            }

            LexError::UnterminatedComment => {
                (ErrorCode::E0008, "Unterminated multi-line comment", Some("Add a closing */ to the comment."))
            }

            LexError::InvalidNumberSuffix => {
                (ErrorCode::E0009, "Invalid number suffix", Some("Use a supported numeric suffix."))
            }

            LexError::NumberOverflow => {
                (ErrorCode::E0010, "Number literal overflow", Some("Use a smaller value or a compatible type."))
            }
        };

        CompileError::LexerError { code: Some(code), message: Arc::from(message), span, help: help.map(str::to_owned) }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, CompileError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
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
