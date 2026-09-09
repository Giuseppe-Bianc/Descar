use descar_core::{
    error::compile_error::CompileError,
    lex::lexer::{Lexer, lexer_tokenize_with_errors},
    tokens::{number::Number, token_kind::TokenKind},
};

fn lex(input: &str) -> (Vec<TokenKind>, Vec<CompileError>) {
    let mut lexer = Lexer::new("test", input);
    let (tokens, errors) = lexer_tokenize_with_errors(&mut lexer);
    (tokens.into_iter().map(|token| token.kind).collect(), errors)
}

#[test]
fn supported_integer_suffixes_are_centralized() {
    let (tokens, errors) =
        lex("100 100u 100U 100i8 100I8 100i16 100I16 100i32 100I32 100u8 100U8 100u16 100U16 100u32 100U32");

    assert!(errors.is_empty());
    assert_eq!(
        tokens,
        vec![
            TokenKind::Numeric(Number::Integer(100)),
            TokenKind::Numeric(Number::UnsignedInteger(100)),
            TokenKind::Numeric(Number::UnsignedInteger(100)),
            TokenKind::Numeric(Number::I8(100)),
            TokenKind::Numeric(Number::I8(100)),
            TokenKind::Numeric(Number::I16(100)),
            TokenKind::Numeric(Number::I16(100)),
            TokenKind::Numeric(Number::I32(100)),
            TokenKind::Numeric(Number::I32(100)),
            TokenKind::Numeric(Number::U8(100)),
            TokenKind::Numeric(Number::U8(100)),
            TokenKind::Numeric(Number::U16(100)),
            TokenKind::Numeric(Number::U16(100)),
            TokenKind::Numeric(Number::U32(100)),
            TokenKind::Numeric(Number::U32(100)),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn explicit_i64_and_u64_suffixes_are_single_invalid_candidates() {
    for input in ["100i64", "100I64", "100u64", "100U64"] {
        let (tokens, errors) = lex(input);

        assert_eq!(tokens, vec![TokenKind::Eof], "input: {input}");
        assert_eq!(errors.len(), 1, "input: {input}");
        assert!(errors[0].to_string().contains("[E0009]"), "input: {input}, error: {}", errors[0]);
        assert!(errors[0].to_string().contains(input), "input: {input}, error: {}", errors[0]);
    }
}

#[test]
fn valid_scientific_literals_still_split_suffix_correctly() {
    let (tokens, errors) = lex("1e5 1e5f 1e5F 1e5d 1e5D");

    assert!(errors.is_empty());
    assert_eq!(
        tokens,
        vec![
            TokenKind::Numeric(Number::Scientific64(1.0, 5)),
            TokenKind::Numeric(Number::Scientific32(1.0, 5)),
            TokenKind::Numeric(Number::Scientific32(1.0, 5)),
            TokenKind::Numeric(Number::Scientific64(1.0, 5)),
            TokenKind::Numeric(Number::Scientific64(1.0, 5)),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn malformed_alphabetic_suffix_is_not_split_into_identifier_tokens() {
    let (tokens, errors) = lex("123abc");

    assert_eq!(tokens, vec![TokenKind::Eof]);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().contains("[E0009]"));
    assert!(errors[0].to_string().contains("123abc"));
}
