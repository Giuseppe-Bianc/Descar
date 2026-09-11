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

#[test]
fn snapshots_expression_printer_variants() {
    let expressions = [
        Expr::Binary {
            left: Box::new(variable("left")),
            op: BinaryOp::Add,
            right: Box::new(literal(LiteralValue::Numeric(Number::Integer(42)))),
            span: span(),
        },
        Expr::Unary {
            op: UnaryOp::Increment,
            side: UnaryOpSide::Postfix,
            expr: Box::new(variable("value")),
            span: span(),
        },
        Expr::Grouping { expr: Box::new(variable("grouped")), span: span() },
        literal(LiteralValue::StringLit("hello".into())),
        literal(LiteralValue::CharLit("c".into())),
        literal(LiteralValue::Bool(true)),
        literal(LiteralValue::NullPtr),
        Expr::ArrayLiteral {
            elements: vec![literal(LiteralValue::Numeric(Number::Integer(1))), variable("item")],
            span: span(),
        },
        variable("name"),
        Expr::Assign {
            target: Box::new(variable("target")),
            value: Box::new(number_expression(7)),
            span: span(),
        },
        Expr::Call {
            callee: Box::new(variable("compute")),
            arguments: vec![number_expression(1), variable("arg")],
            span: span(),
        },
        Expr::ArrayAccess {
            array: Box::new(variable("items")),
            index: Box::new(number_expression(0)),
            span: span(),
        },
    ];

    let output = expressions
        .iter()
        .map(|expression| strip_ansi_codes(&pretty_print(expression)))
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("expression_printer_variants", output);
}

fn number_expression(value: i64) -> Expr {
    Expr::new_number_literal(Number::Integer(value), span())
}

#[test]
fn snapshots_statement_printer_variants() {
    let node_span = span();
    let condition = Expr::new_bool_literal(true, node_span.clone());
    let block = Stmt::Block {
        statements: vec![Stmt::Break { span: node_span.clone() }],
        span: node_span.clone(),
    };
    let parameter = Parameter::new("value".into(), Type::I32, node_span.clone());
    let statements = [
        Stmt::Expression { expr: Box::new(number_expression(1)) },
        Stmt::VarDeclaration {
            bindings: vec![VarBinding { name: "value".into(), initializer: Some(number_expression(10)) }],
            type_annotation: Type::I64,
            is_mutable: true,
            span: node_span.clone(),
        },
        Stmt::Function {
            name: "compute".into(),
            parameters: vec![parameter],
            return_type: Type::F64,
            body: Box::new(block.clone()),
            span: node_span.clone(),
        },
        Stmt::If {
            condition: Box::new(condition.clone()),
            then_branch: Box::new(block.clone()),
            else_branch: ElseBranch::ElseIf(Box::new(Stmt::Return { value: None, span: node_span.clone() })),
            span: node_span.clone(),
        },
        Stmt::While {
            condition: Box::new(condition.clone()),
            body: Box::new(block.clone()),
            span: node_span.clone(),
        },
        Stmt::For {
            initializer: Some(Box::new(Stmt::VarDeclaration {
                bindings: vec![VarBinding { name: "i".into(), initializer: Some(number_expression(0)) }],
                type_annotation: Type::I32,
                is_mutable: true,
                span: node_span.clone(),
            })),
            condition: Some(condition),
            increment: Some(Expr::Unary {
                op: UnaryOp::Increment,
                side: UnaryOpSide::Postfix,
                expr: Box::new(variable("i")),
                span: node_span.clone(),
            }),
            body: Box::new(block.clone()),
            span: node_span.clone(),
        },
        block,
        Stmt::Return { value: Some(number_expression(5)), span: node_span.clone() },
        Stmt::Break { span: node_span.clone() },
        Stmt::Continue { span: node_span.clone() },
        Stmt::MainFunction {
            body: Box::new(Stmt::Block { statements: Vec::new(), span: node_span.clone() }),
            span: node_span,
        },
    ];

    let output = statements
        .iter()
        .map(|statement| strip_ansi_codes(&pretty_print_stmt(statement)))
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("statement_printer_variants", output);
}

#[test]
fn snapshots_type_formatting_through_ast_printer() {
    let node_span = span();
    let statements = [
        Stmt::VarDeclaration {
            bindings: vec![],
            type_annotation: Type::I8,
            is_mutable: true,
            span: node_span.clone(),
        },
        Stmt::VarDeclaration {
            bindings: vec![],
            type_annotation: Type::Custom { name: Arc::from("UserType") },
            is_mutable: false,
            span: node_span.clone(),
        },
        Stmt::VarDeclaration {
            bindings: vec![],
            type_annotation: Type::Array {
                element_type: Box::new(Type::U32),
                size: Box::new(number_expression(8)),
            },
            is_mutable: true,
            span: node_span.clone(),
        },
        Stmt::VarDeclaration {
            bindings: vec![],
            type_annotation: Type::Vector { element_type: Box::new(Type::String) },
            is_mutable: true,
            span: node_span,
        },
    ];

    let output = statements
        .iter()
        .map(|statement| strip_ansi_codes(&pretty_print_stmt(statement)))
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("type_formatting", output);
}
