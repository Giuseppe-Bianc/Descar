// Snapshot test for lexer using insta

use descar_core::lex::lexer::Lexer;

#[test]
fn lexer_snapshot() {
    let input = "foo + 123 - bar * 45.6";
    let mut lexer = Lexer::new("snapshot_test", input);
    let mut token_kinds = Vec::new();
    while let Some(tok) = lexer.next_token() {
        token_kinds.push(tok.map(|t| t.kind));
    }
    // Debug snapshot of token kinds (including errors if any)
    insta::assert_debug_snapshot!(token_kinds);
}

#[test]
fn lexer_snapshot_complex() {
    let input = r#"if (x >= 10) { print("big"); } else { print("small"); }"#;
    let mut lexer = Lexer::new("snapshot_complex", input);
    let mut token_kinds = Vec::new();
    while let Some(tok) = lexer.next_token() {
        token_kinds.push(tok.map(|t| t.kind));
    }
    insta::assert_debug_snapshot!(token_kinds);
}
