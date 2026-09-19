use std::sync::Arc;

use descar_core::{
    error::compile_error::CompileError,
    location::{source_location::SourceLocation, source_span::SourceSpan},
    semantic::symbol_table::{FunctionSymbol, Symbol, SymbolTable, VariableSymbol},
    syntax::ast::Type,
};
use insta::assert_debug_snapshot;

fn span(start_offset: usize, end_offset: usize) -> SourceSpan {
    let start = SourceLocation::new(1, start_offset + 1, start_offset, start_offset, start_offset, start_offset);
    let end = SourceLocation::new(1, end_offset + 1, end_offset, end_offset, end_offset, end_offset);

    SourceSpan::new(Arc::from("test.vn"), start, end)
}

fn variable(name: &str, ty: Type, defined_at: SourceSpan) -> Symbol {
    Symbol::Variable(VariableSymbol { name: Arc::from(name), ty, mutable: true, defined_at, last_assignment: None })
}

fn function(name: &str, return_type: Type, defined_at: SourceSpan) -> FunctionSymbol {
    FunctionSymbol { name: Arc::from(name), parameters: vec![], return_type, defined_at }
}

#[test]
fn duplicate_declaration_errors_have_stable_structured_output() {
    let mut table = SymbolTable::new();
    let first_var_span = span(0, 1);
    let first_function_span = span(4, 5);
    let first_alias_span = span(8, 9);
    let duplicate_span = span(12, 13);

    table
        .declare("value", variable("value", Type::I32, first_var_span.clone()), first_var_span)
        .expect("first variable declaration must succeed");

    let variable_error = table
        .declare("value", variable("value", Type::Bool, duplicate_span.clone()), duplicate_span.clone())
        .expect_err("duplicate variable declaration must fail");

    table
        .declare(
            "compute",
            Symbol::Function(function("compute", Type::I32, first_function_span.clone())),
            first_function_span,
        )
        .expect("first function declaration must succeed");

    let function_error = table
        .declare(
            "compute",
            Symbol::Function(function("compute", Type::Bool, duplicate_span.clone())),
            duplicate_span.clone(),
        )
        .expect_err("duplicate function declaration must fail");

    table
        .declare("Number", Symbol::TypeAlias(Type::I32), first_alias_span)
        .expect("first alias declaration must succeed");

    let alias_error = table
        .declare("Number", Symbol::TypeAlias(Type::F64), duplicate_span)
        .expect_err("duplicate alias declaration must fail");

    let errors: Vec<CompileError> = vec![variable_error, function_error, alias_error];

    assert_debug_snapshot!(errors);
}
