use std::sync::Arc;

use descar_core::{
    location::{source_location::SourceLocation, source_span::SourceSpan},
    semantic::{BuiltinType, InitializationStatus, LanguageVersion, SemanticConfig, SemanticContext},
    syntax::ast::Stmt,
};

fn span(file: &str, start: usize, end: usize) -> SourceSpan {
    SourceSpan::new(
        Arc::from(file),
        SourceLocation::new(1, start + 1, start, start, start, start),
        SourceLocation::new(1, end + 1, end, end, end, end),
    )
}

#[test]
fn builtins_are_canonical_and_deterministic() {
    let a = SemanticContext::new(SemanticConfig::default());
    let b = SemanticContext::new(SemanticConfig::default());

    assert_eq!(a.types().builtin(BuiltinType::I32), b.types().builtin(BuiltinType::I32));
    assert_eq!(a.types().builtin(BuiltinType::I32), a.types().builtin(BuiltinType::I32));
    assert_eq!(a.types().len(), BuiltinType::ALL.len());
}

#[test]
fn global_scope_exists_and_has_no_program_symbols() {
    let context = SemanticContext::new(SemanticConfig::default());
    let global = context.symbols().scope(context.global_scope()).expect("global scope");

    assert_eq!(context.global_scope().index(), 0);
    assert!(global.is_empty());
}

#[test]
fn builtin_symbols_are_registered_outside_global_scope() {
    let context = SemanticContext::new(SemanticConfig::default());
    assert_eq!(context.builtins().type_symbols().len(), BuiltinType::ALL.len());
    assert_eq!(context.symbols().symbol_count(), BuiltinType::ALL.len());
    assert!(context.symbols().scope(context.global_scope()).expect("global scope").is_empty());
}

#[test]
fn empty_ast_initializes_successfully() {
    let result = SemanticContext::initialize(&[], SemanticConfig::default());
    let context = result.expect("empty AST is valid");
    assert_eq!(context.status(), InitializationStatus::Ready);
    assert!(context.is_frozen());
    assert!(context.diagnostics().is_empty());
}

#[test]
fn inconsistent_source_files_are_rejected_without_panic() {
    let first = Stmt::Break { span: span("first.dr", 0, 1) };
    let second = Stmt::Continue { span: span("second.dr", 0, 1) };

    let result = SemanticContext::initialize(&[first, second], SemanticConfig::default());
    let context = result.expect_err("multiple source files are invalid for one compilation unit");
    assert_eq!(context.status(), InitializationStatus::Failed);
    assert!(context.diagnostics().has_errors());
}

#[test]
fn language_version_is_part_of_frozen_configuration() {
    let config = SemanticConfig::new(LanguageVersion::new(0, 2, 0));
    let context = SemanticContext::new(config.clone());
    assert_eq!(context.config().language_version(), config.language_version());
}
