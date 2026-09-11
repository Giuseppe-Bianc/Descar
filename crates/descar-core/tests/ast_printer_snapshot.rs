use std::sync::Arc;

use descar_core::{
    location::{source_location::SourceLocation, source_span::SourceSpan},
    printers::ast_printer::{pretty_print, pretty_print_stmt},
    syntax::ast::{
        ast_type::Type,
        binary_op::BinaryOp,
        else_branch::ElseBranch,
        expr::Expr,
        literal_value::LiteralValue,
        parameter::Parameter,
        stmt::{Stmt, VarBinding},
        unary_op::UnaryOp,
        unary_op_side::UnaryOpSide,
    },
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

fn number(value: i64) -> Expr {
    Expr::new_number_literal(Number::Integer(value), span())
}

fn rendered_expr(expr: &Expr) -> String {
    strip_ansi_codes(&pretty_print(expr))
}

fn rendered_stmt(stmt: &Stmt) -> String {
    strip_ansi_codes(&pretty_print_stmt(stmt))
}

#[test]
fn snapshot_literal_values() {
    let expressions = [
        number(-42),
        Expr::new_bool_literal(false, span()),
        Expr::new_string_literal("hello".into(), span()),
        Expr::new_char_literal("é".into(), span()),
        Expr::new_nullptr_literal(span()),
        variable("counter"),
    ];

    let rendered = expressions
        .iter()
        .map(|expression| rendered_expr(expression).trim_end().to_owned())
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("literal_values", rendered);
}

#[test]
fn snapshot_nested_expression_printer() {
    let expression = Expr::Assign {
        target: Box::new(Expr::ArrayAccess {
            array: Box::new(Expr::Grouping { expr: Box::new(variable("items")), span: span() }),
            index: Box::new(Expr::Binary {
                left: Box::new(number(1)),
                op: BinaryOp::Add,
                right: Box::new(number(2)),
                span: span(),
            }),
            span: span(),
        }),
        value: Box::new(Expr::Call {
            callee: Box::new(variable("build")),
            arguments: vec![number(10), Expr::new_bool_literal(false, span())],
            span: span(),
        }),
        span: span(),
    };

    assert_snapshot!("nested_expression", rendered_expr(&expression));
}

#[test]
fn snapshot_collection_expression_printer() {
    let expressions = [
        Expr::Call { callee: Box::new(variable("f")), arguments: vec![], span: span() },
        Expr::Call {
            callee: Box::new(variable("f")),
            arguments: vec![number(1), number(2), variable("x")],
            span: span(),
        },
        Expr::ArrayLiteral { elements: vec![], span: span() },
        Expr::ArrayLiteral { elements: vec![number(1), number(2), variable("x")], span: span() },
    ];

    let rendered = expressions
        .iter()
        .map(|expression| rendered_expr(expression).trim_end().to_owned())
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("collection_expressions", rendered);
}

#[test]
fn snapshot_declaration_and_function_printers() {
    let declaration = Stmt::VarDeclaration {
        bindings: vec![
            VarBinding { name: "first".into(), initializer: Some(number(1)) },
            VarBinding { name: "second".into(), initializer: Some(Expr::new_bool_literal(true, span())) },
        ],
        type_annotation: Type::Array { element_type: Box::new(Type::U32), size: Box::new(number(8)) },
        is_mutable: true,
        span: span(),
    };
    let function = Stmt::Function {
        name: "compute".into(),
        parameters: vec![
            Parameter::new("left".into(), Type::I32, span()),
            Parameter::new("right".into(), Type::Vector { element_type: Box::new(Type::Bool) }, span()),
        ],
        return_type: Type::F64,
        body: Box::new(Stmt::Block {
            statements: vec![Stmt::Return { value: Some(number(42)), span: span() }],
            span: span(),
        }),
        span: span(),
    };

    let rendered = format!(
        "declaration:\n{}\nfunction:\n{}",
        rendered_stmt(&declaration).trim_end(),
        rendered_stmt(&function).trim_end()
    );

    assert_snapshot!("declaration_and_function", rendered);
}

#[test]
fn snapshot_control_flow_printer() {
    let statement = Stmt::If {
        condition: Box::new(Expr::Binary {
            left: Box::new(variable("count")),
            op: BinaryOp::Greater,
            right: Box::new(number(0)),
            span: span(),
        }),
        then_branch: Box::new(Stmt::While {
            condition: Box::new(Expr::new_bool_literal(true, span())),
            body: Box::new(Stmt::Block { statements: vec![Stmt::Break { span: span() }], span: span() }),
            span: span(),
        }),
        else_branch: ElseBranch::ElseIf(Box::new(Stmt::For {
            initializer: Some(Box::new(Stmt::VarDeclaration {
                bindings: vec![VarBinding { name: "i".into(), initializer: Some(number(0)) }],
                type_annotation: Type::I32,
                is_mutable: true,
                span: span(),
            })),
            condition: Some(Expr::new_bool_literal(true, span())),
            increment: Some(Expr::Unary {
                op: UnaryOp::Increment,
                side: UnaryOpSide::Postfix,
                expr: Box::new(variable("i")),
                span: span(),
            }),
            body: Box::new(Stmt::Block { statements: vec![Stmt::Continue { span: span() }], span: span() }),
            span: span(),
        })),
        span: span(),
    };

    assert_snapshot!("control_flow", rendered_stmt(&statement));
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
