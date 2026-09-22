use std::sync::Arc;

use descar_core::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    location::{source_location::SourceLocation, source_span::SourceSpan},
    semantic::symbol_table::{FunctionSymbol, ScopeKind, Symbol, SymbolTable, VariableSymbol},
    syntax::ast::{Parameter, Type},
};

fn span(start_offset: usize, end_offset: usize) -> SourceSpan {
    let start = SourceLocation::new(1, start_offset + 1, start_offset, start_offset, start_offset, start_offset);
    let end = SourceLocation::new(1, end_offset + 1, end_offset, end_offset, end_offset, end_offset);

    SourceSpan::new(Arc::from("test.vn"), start, end)
}

fn variable_symbol(
    name: &str, ty: Type, mutable: bool, defined_at: SourceSpan, last_assignment: Option<SourceSpan>,
) -> Symbol {
    Symbol::Variable(VariableSymbol { name: Arc::from(name), ty, mutable, defined_at, last_assignment })
}

fn function_symbol(
    name: &str, parameters: Vec<Parameter>, return_type: Type, defined_at: SourceSpan,
) -> FunctionSymbol {
    FunctionSymbol { name: Arc::from(name), parameters, return_type, defined_at }
}

#[test]
fn new_and_default_start_with_single_empty_global_scope() {
    let new_table = SymbolTable::new();
    let default_table = SymbolTable::default();

    assert_eq!(new_table, default_table);
    assert_eq!(new_table.scope_count(), 1);
    assert_eq!(new_table.current_scope_kind(), Some(ScopeKind::Global));
    assert!(new_table.current_scope().is_some_and(|scope| {
        scope.kind() == ScopeKind::Global && scope.is_empty() && scope.defined_at().is_none()
    }));
    assert!(new_table.current_function().is_none());
}

#[test]
fn push_scope_creates_empty_scope_with_kind_and_definition_location() {
    let mut table = SymbolTable::new();
    let definition_span = span(4, 9);

    table.push_scope(ScopeKind::Block, Some(definition_span.clone()));

    assert_eq!(table.scope_count(), 2);
    assert_eq!(table.current_scope_kind(), Some(ScopeKind::Block));

    let current = table.current_scope().expect("current scope must exist");
    assert_eq!(current.kind(), ScopeKind::Block);
    assert_eq!(current.defined_at(), Some(definition_span).as_ref());
    assert!(current.is_empty());
}

#[test]
fn pop_scope_removes_inner_scopes_and_keeps_global_scope() {
    let mut table = SymbolTable::new();

    table.pop_scope();
    assert_eq!(table.scope_count(), 1);
    assert_eq!(table.current_scope_kind(), Some(ScopeKind::Global));

    table.push_scope(ScopeKind::Function, None);
    table.push_scope(ScopeKind::Block, None);

    table.pop_scope();
    assert_eq!(table.scope_count(), 2);
    assert_eq!(table.current_scope_kind(), Some(ScopeKind::Function));

    table.pop_scope();
    assert_eq!(table.scope_count(), 1);
    assert_eq!(table.current_scope_kind(), Some(ScopeKind::Global));

    table.pop_scope();
    assert_eq!(table.scope_count(), 1);
}

#[test]
fn declare_and_lookup_preserve_variable_function_and_type_alias_metadata() {
    let mut table = SymbolTable::new();
    let variable_definition = span(0, 2);
    let variable_assignment = span(8, 10);
    let function_definition = span(12, 20);
    let parameter_span = span(15, 16);

    let variable =
        variable_symbol("value", Type::I32, true, variable_definition.clone(), Some(variable_assignment.clone()));
    let function = function_symbol(
        "compute",
        vec![Parameter::new("input".into(), Type::U16, parameter_span.clone())],
        Type::F64,
        function_definition.clone(),
    );
    let type_alias = Symbol::TypeAlias(Type::Custom { name: Arc::from("Number") });

    assert!(table.declare("value", variable.clone(), variable_definition.clone()).is_ok());
    assert!(table.declare("compute", Symbol::Function(function.clone()), function_definition).is_ok());
    assert!(table.declare("Number", type_alias.clone(), parameter_span).is_ok());

    assert_eq!(table.lookup("value"), Some(variable));
    assert_eq!(
        table.lookup_variable("value"),
        Some(VariableSymbol {
            name: Arc::from("value"),
            ty: Type::I32,
            mutable: true,
            defined_at: variable_definition,
            last_assignment: Some(variable_assignment),
        })
    );

    assert_eq!(table.lookup("compute"), Some(Symbol::Function(function.clone())));
    assert_eq!(table.lookup_function("compute"), Some(function));

    assert_eq!(table.lookup("Number"), Some(type_alias));
    assert_eq!(table.lookup_variable("Number"), None);
    assert_eq!(table.lookup_function("Number"), None);
}

#[test]
fn lookup_of_missing_name_returns_none_for_all_lookup_variants() {
    let table = SymbolTable::new();

    assert_eq!(table.lookup("missing"), None);
    assert_eq!(table.lookup_variable("missing"), None);
    assert_eq!(table.lookup_function("missing"), None);
}

#[test]
fn lookup_returns_owned_clones_without_mutating_stored_variable() {
    let mut table = SymbolTable::new();
    let definition = span(0, 1);

    table
        .declare("value", variable_symbol("value", Type::I32, true, definition.clone(), None), definition)
        .expect("initial declaration must succeed");

    let mut cloned = table.lookup_variable("value").expect("variable must exist");
    cloned.mutable = false;

    assert!(!cloned.mutable);
    assert_eq!(table.lookup_variable("value").map(|variable| variable.mutable), Some(true));
}

#[test]
fn nested_scope_shadows_outer_symbol_and_pop_restores_outer_symbol() {
    let mut table = SymbolTable::new();
    let outer_span = span(0, 1);
    let inner_span = span(4, 5);

    table
        .declare("value", variable_symbol("value", Type::I32, true, outer_span.clone(), None), outer_span)
        .expect("outer declaration must succeed");

    table.push_scope(ScopeKind::Block, Some(inner_span.clone()));
    table
        .declare("value", variable_symbol("value", Type::Bool, false, inner_span.clone(), None), inner_span)
        .expect("shadowing declaration must succeed");

    assert_eq!(table.lookup_variable("value").map(|variable| variable.ty), Some(Type::Bool));
    assert_eq!(table.lookup_variable("value").map(|variable| variable.mutable), Some(false));

    table.pop_scope();

    assert_eq!(table.lookup_variable("value").map(|variable| variable.ty), Some(Type::I32));
    assert_eq!(table.lookup_variable("value").map(|variable| variable.mutable), Some(true));
}

#[test]
fn typed_lookup_does_not_skip_an_inner_symbol_of_the_wrong_kind() {
    let mut table = SymbolTable::new();
    let outer_variable_span = span(0, 1);
    let inner_function_span = span(4, 5);

    table
        .declare(
            "name",
            variable_symbol("name", Type::I32, true, outer_variable_span.clone(), None),
            outer_variable_span,
        )
        .expect("outer declaration must succeed");

    table.push_scope(ScopeKind::Block, Some(inner_function_span.clone()));
    let inner_function = function_symbol("name", vec![], Type::Void, inner_function_span.clone());

    table
        .declare("name", Symbol::Function(inner_function.clone()), inner_function_span)
        .expect("inner declaration must succeed");

    assert_eq!(table.lookup("name"), Some(Symbol::Function(inner_function.clone())));
    assert_eq!(table.lookup_function("name"), Some(inner_function));
    assert_eq!(table.lookup_variable("name"), None);

    table.pop_scope();

    assert_eq!(table.lookup_variable("name").map(|variable| variable.ty), Some(Type::I32));
    assert_eq!(table.lookup_function("name"), None);
}

#[test]
fn reverse_typed_lookup_case_also_stops_at_inner_variable() {
    let mut table = SymbolTable::new();
    let outer_function_span = span(0, 1);
    let inner_variable_span = span(4, 5);

    let outer_function = function_symbol("name", vec![], Type::I32, outer_function_span.clone());

    table
        .declare("name", Symbol::Function(outer_function.clone()), outer_function_span)
        .expect("outer function declaration must succeed");

    table.push_scope(ScopeKind::Block, Some(inner_variable_span.clone()));
    table
        .declare(
            "name",
            variable_symbol("name", Type::Bool, true, inner_variable_span.clone(), None),
            inner_variable_span,
        )
        .expect("inner variable declaration must succeed");

    assert_eq!(table.lookup("name"), table.lookup_variable("name").map(Symbol::Variable));
    assert_eq!(table.lookup_function("name"), None);
    assert_eq!(table.lookup_variable("name").map(|variable| variable.ty), Some(Type::Bool));

    table.pop_scope();

    assert_eq!(table.lookup_function("name"), Some(outer_function));
    assert_eq!(table.lookup_variable("name"), None);
}

#[test]
fn duplicate_variable_declaration_returns_e2032_and_keeps_original_symbol() {
    let mut table = SymbolTable::new();
    let original_span = span(0, 1);
    let duplicate_span = span(4, 5);

    let original = variable_symbol("value", Type::I32, true, original_span.clone(), None);

    assert!(table.declare("value", original.clone(), original_span.clone()).is_ok());

    let error = table
        .declare(
            "value",
            variable_symbol("value", Type::Bool, false, duplicate_span.clone(), None),
            duplicate_span.clone(),
        )
        .expect_err("duplicate declaration must be rejected");

    match error {
        CompileError::TypeError { code, message, span, help } => {
            assert_eq!(code, Some(ErrorCode::E2032));
            assert_eq!(message.as_ref(), "Identifier 'value' already declared in this Global scope");
            assert_eq!(span, duplicate_span);
            let expected_help = format!("Previous declaration at {original_span}");
            assert_eq!(help.as_deref(), Some(expected_help.as_str()));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert_eq!(table.lookup("value"), Some(original));
}

#[test]
fn duplicate_declaration_message_uses_current_scope_kind() {
    let mut table = SymbolTable::new();
    let declaration_span = span(0, 1);
    let duplicate_span = span(4, 5);

    table.push_scope(ScopeKind::Block, None);
    table
        .declare("value", variable_symbol("value", Type::I32, true, declaration_span.clone(), None), declaration_span)
        .expect("first declaration in block must succeed");

    let error = table
        .declare("value", variable_symbol("value", Type::Bool, false, duplicate_span.clone(), None), duplicate_span)
        .expect_err("duplicate declaration must be rejected");

    match error {
        CompileError::TypeError { message, .. } => {
            assert_eq!(message.as_ref(), "Identifier 'value' already declared in this Block scope");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn duplicate_function_declaration_includes_previous_definition_help() {
    let mut table = SymbolTable::new();
    let original_span = span(0, 1);
    let duplicate_span = span(4, 5);
    let original = function_symbol("compute", vec![], Type::I32, original_span.clone());

    assert!(table.declare("compute", Symbol::Function(original.clone()), original_span.clone(),).is_ok());

    let error = table
        .declare(
            "compute",
            Symbol::Function(function_symbol("compute", vec![], Type::Bool, duplicate_span.clone())),
            duplicate_span.clone(),
        )
        .expect_err("duplicate declaration must be rejected");

    match error {
        CompileError::TypeError { code, message, span, help } => {
            assert_eq!(code, Some(ErrorCode::E2032));
            assert_eq!(message.as_ref(), "Identifier 'compute' already declared in this Global scope");
            assert_eq!(span, duplicate_span);
            let expected_help = format!("Previous declaration at {original_span}");
            assert_eq!(help.as_deref(), Some(expected_help.as_str()));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert_eq!(table.lookup_function("compute"), Some(original));
}

#[test]
fn duplicate_type_alias_declaration_has_no_previous_definition_help() {
    let mut table = SymbolTable::new();
    let original_span = span(0, 1);
    let duplicate_span = span(4, 5);

    assert!(table.declare("Number", Symbol::TypeAlias(Type::I32), original_span).is_ok());

    let error = table
        .declare("Number", Symbol::TypeAlias(Type::F64), duplicate_span.clone())
        .expect_err("duplicate declaration must be rejected");

    match error {
        CompileError::TypeError { code, message, span, help } => {
            assert_eq!(code, Some(ErrorCode::E2032));
            assert_eq!(message.as_ref(), "Identifier 'Number' already declared in this Global scope");
            assert_eq!(span, duplicate_span);
            assert_eq!(help, None);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn declare_allows_empty_symbol_name_because_no_name_validation_is_implemented() {
    let mut table = SymbolTable::new();
    let definition = span(0, 0);

    assert!(table.declare("", Symbol::TypeAlias(Type::I32), definition).is_ok());

    assert_eq!(table.lookup(""), Some(Symbol::TypeAlias(Type::I32)));
}

#[test]
fn current_scope_exposes_read_only_scope_state() {
    let table = SymbolTable::new();

    let scope = table.current_scope().expect("global scope must exist");

    assert_eq!(scope.kind(), ScopeKind::Global);
    assert!(scope.defined_at().is_none());
    assert!(scope.is_empty());
    assert_eq!(scope.symbol_count(), 0);
    assert!(!scope.contains("injected"));
    assert!(scope.symbol("injected").is_none());
}

#[test]
fn current_function_context_starts_empty_can_be_replaced_and_can_be_cleared() {
    let mut table = SymbolTable::new();
    let first =
        function_symbol("first", vec![Parameter::new("arg".into(), Type::I8, span(1, 2))], Type::I32, span(0, 5));
    let second = function_symbol("second", vec![], Type::Void, span(8, 14));

    table.exit_function();
    assert!(table.current_function().is_none());
    assert!(table.current_function_return_type().is_none());

    table.enter_function(first.clone());

    assert_eq!(table.current_function(), Some(&first));
    assert_eq!(table.current_function_return_type(), Some(Type::I32));
    assert_eq!(table.scope_count(), 1);

    table.enter_function(second.clone());

    assert_eq!(table.current_function(), Some(&second));
    assert_eq!(table.current_function_return_type(), Some(Type::Void));

    table.exit_function();

    assert!(table.current_function().is_none());
    assert!(table.current_function_return_type().is_none());
}

#[test]
fn push_global_scope_is_ignored() {
    let mut table = SymbolTable::new();

    table.push_scope(ScopeKind::Global, None);

    assert_eq!(table.scope_count(), 1);
    assert_eq!(table.current_scope_kind(), Some(ScopeKind::Global));
}

#[test]
fn current_symbol_returns_only_symbol_from_current_scope() {
    let mut table = SymbolTable::new();
    let outer_span = span(0, 1);
    let inner_span = span(4, 5);

    let outer = variable_symbol("value", Type::I32, true, outer_span.clone(), None);
    let inner = variable_symbol("value", Type::Bool, false, inner_span.clone(), None);

    table.declare("value", outer.clone(), outer_span).expect("outer declaration must succeed");

    assert_eq!(table.current_symbol("value"), Some(&outer));
    assert_eq!(table.current_symbol("missing"), None);

    table.push_scope(ScopeKind::Block, Some(inner_span.clone()));

    assert_eq!(table.current_symbol("value"), None);

    table.declare("value", inner.clone(), inner_span).expect("inner declaration must succeed");

    assert_eq!(table.current_symbol("value"), Some(&inner));
    assert_eq!(table.current_symbol("missing"), None);

    table.pop_scope();

    assert_eq!(table.current_symbol("value"), Some(&outer));
}

#[test]
fn contains_current_checks_only_the_current_scope() {
    let mut table = SymbolTable::new();
    let outer_span = span(0, 1);
    let inner_span = span(4, 5);

    table
        .declare("value", variable_symbol("value", Type::I32, true, outer_span.clone(), None), outer_span)
        .expect("outer declaration must succeed");

    assert!(table.contains_current("value"));
    assert!(!table.contains_current("missing"));

    table.push_scope(ScopeKind::Block, Some(inner_span.clone()));

    assert!(!table.contains_current("value"));
    assert!(!table.contains_current("missing"));

    table
        .declare("value", variable_symbol("value", Type::Bool, false, inner_span.clone(), None), inner_span)
        .expect("inner declaration must succeed");

    assert!(table.contains_current("value"));
    assert!(!table.contains_current("missing"));

    table.pop_scope();

    assert!(table.contains_current("value"));
}
