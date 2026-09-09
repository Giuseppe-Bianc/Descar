use descar_core::{
    lex::lexer::{lexer_tokenize_with_errors, Lexer},
    tokens::token_kind::TokenKind,
};

fn lex(input: &str) -> (Vec<TokenKind>, Vec<String>) {
    let mut lexer = Lexer::new("test", input);
    let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);
    (
        tokens.into_iter().map(|token| token.kind).collect(),
        errors.into_iter().map(|error| error.to_string()).collect(),
    )
}

#[test]
fn accepts_supported_escapes() {
    let (tokens, errors) = lex(r#""\n\r\t\\\'\"\0" 'a' '\\' '\''"#);
    assert!(errors.is_empty());
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[4], TokenKind::Eof);
}

#[test]
fn rejects_invalid_string_escape() {
    let (tokens, errors) = lex(r#""hello\q""#);
    assert_eq!(tokens, vec![TokenKind::Eof]);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("[E0007] Invalid escape sequence:"));
}

#[test]
fn rejects_invalid_char_escape() {
    let (tokens, errors) = lex(r"'\q'");
    assert_eq!(tokens, vec![TokenKind::Eof]);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("[E0007] Invalid escape sequence:"));
}

#[test]
fn accepts_valid_unicode_scalar_escape() {
    let (tokens, errors) = lex(r#""\u{1F600}""#);
    assert!(errors.is_empty());
    assert!(matches!(tokens[0], TokenKind::StringLiteral(_)));
    assert_eq!(tokens[1], TokenKind::Eof);
}

#[test]
fn rejects_invalid_unicode_scalar_escape() {
    for input in [r#""\u{}""#, r#""\u{110000}""#, r#""\u{D800}""#] {
        let (tokens, errors) = lex(input);
        assert_eq!(tokens, vec![TokenKind::Eof]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("[E0007] Invalid escape sequence:"));
    }
}
