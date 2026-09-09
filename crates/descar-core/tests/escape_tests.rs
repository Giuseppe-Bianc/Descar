use descar_core::lex::lexer::{lexer_tokenize_with_errors, Lexer};
use descar_core::tokens::token_kind::TokenKind;

#[test]
fn accepts_supported_string_escapes() {
    let mut lexer = Lexer::new("test", r#""\n\r\t\\\'\"\0\u{41}""#);
    let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].kind, TokenKind::StringLiteral(_)));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn accepts_supported_char_escapes() {
    for input in [r#"'\n'"#, r#"'\r'"#, r#"'\t'"#, r#"'\\'"#, r#"'\''"#, r#"'\"'"#, r#"'\0'"#, r#"'\u{41}'"#] {
        let mut lexer = Lexer::new("test", input);
        let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);

        assert!(errors.is_empty(), "unexpected errors for {input:?}: {errors:?}");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].kind, TokenKind::CharLiteral(_)));
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }
}

#[test]
fn rejects_invalid_string_escape() {
    let mut lexer = Lexer::new("test", r#""hello\q""#);
    let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().starts_with("[E0007] Invalid escape sequence:"));
}

#[test]
fn rejects_invalid_char_escape() {
    let mut lexer = Lexer::new("test", r#"'\q'"#);
    let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().starts_with("[E0007] Invalid escape sequence:"));
}

#[test]
fn rejects_malformed_unicode_escape() {
    for input in [r#""\u""#, r#""\u{}""#, r#""\u{110000}""#, r#""\u{D800}""#, r#""\u{1234567}""#] {
        let mut lexer = Lexer::new("test", input);
        let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);

        assert_eq!(tokens.len(), 1, "unexpected tokens for {input:?}");
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(errors.len(), 1, "unexpected errors for {input:?}: {errors:?}");
        assert!(errors[0].to_string().starts_with("[E0007] Invalid escape sequence:"));
    }
}
