use std::sync::Arc;

use descar_core::{
    location::{source_location::SourceLocation, source_span::SourceSpan},
    printers::ast_printer::{pretty_print, pretty_print_stmt},
    syntax::ast::{ast_type::Type, binary_op::BinaryOp, else_branch::ElseBranch, expr::Expr, literal_value::LiteralValue, stmt::Stmt},
    tokens::number::Number,
    utils::strip_ansi_codes,
};
use insta::assert_snapshot;

fn span() -> SourceSpan {
    SourceSpan::new(
        Arc::from("test.lang"),
        SourceLocation::new(1, 1, 0, 0, 0, 0),
        SourceLocation::new(1, 1, 1, 1, 1, 1),
    )
}

fn variable(name: &str) -> Expr {
    Expr::Variable { name: name.into(), span: span() }
}

fn literal(value: LiteralValue) -> Expr {
    Expr::Literal { value, span: span() }
}

#[test]
fn snapshot_binary_expression_printer() {
    let expression = Expr::Binary {
        left: Box::new(variable("left")),
        op: BinaryOp::Add,
        right: Box::new(literal(LiteralValue::Numeric(Number::Integer(42)))),
        span: span(),
    };

    assert_snapshot!("binary_expression", strip_ansi_codes(&pretty_print(&expression)));
}

#[test]
fn snapshot_nested_statement_printer() {
    let statement = Stmt::If {
        condition: Box::new(Expr::new_bool_literal(true, span())),
        then_branch: Box::new(Stmt::Block {
            statements: vec![Stmt::Return {
                value: Some(Expr::new_number_literal(Number::Integer(1), span())),
                span: span(),
            }],
            span: span(),
        }),
        else_branch: ElseBranch::None,
        span: span(),
    };

    assert_snapshot!("nested_statement", strip_ansi_codes(&pretty_print_stmt(&statement)));
}

#[test]
fn snapshot_declaration_type_printer() {
    let statement = Stmt::VarDeclaration {
        bindings: Vec::new(),
        type_annotation: Type::Array {
            element_type: Box::new(Type::U32),
            size: Box::new(Expr::new_number_literal(Number::Integer(8), span())),
        },
        is_mutable: false,
        span: span(),
    };

    assert_snapshot!("declaration_type", strip_ansi_codes(&pretty_print_stmt(&statement)));
}
