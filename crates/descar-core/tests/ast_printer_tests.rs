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

fn span(start_offset: usize, end_offset: usize) -> SourceSpan {
    let start = SourceLocation::new(1, start_offset + 1, start_offset, start_offset, start_offset, start_offset);
    let end = SourceLocation::new(1, end_offset + 1, end_offset, end_offset, end_offset, end_offset);
    SourceSpan::new(Arc::from("test.lang"), start, end)
}

fn variable(name: &str) -> Expr {
    Expr::Variable { name: name.into(), span: span(0, 1) }
}

fn number(value: i64) -> Expr {
    Expr::new_number_literal(Number::Integer(value), span(0, 1))
}

#[test]
fn pretty_print_expression_contains_expected_structure() {
    let expression = Expr::Binary {
        left: Box::new(variable("x")),
        op: BinaryOp::Add,
        right: Box::new(number(42)),
        span: span(0, 5),
    };

    let output = strip_ansi_codes(&pretty_print(&expression));

    assert!(output.contains("BinaryOp ADD"));
    assert!(output.contains("Left:"));
    assert!(output.contains("Variable 'x'"));
    assert!(output.contains("Right:"));
    assert!(output.contains("Literal 42"));
}

#[test]
fn pretty_print_expression_handles_empty_collections() {
    let expression = Expr::Call {
        callee: Box::new(variable("f")),
        arguments: Vec::new(),
        span: span(0, 3),
    };

    let output = strip_ansi_codes(&pretty_print(&expression));

    assert!(output.contains("Function Call"));
    assert!(output.contains("Callee:"));
    assert!(output.contains("Arguments:"));
}

#[test]
fn pretty_print_expression_covers_unary_grouping_assignment_and_array_access() {
    let expressions = [
        Expr::Unary {
            op: UnaryOp::Increment,
            side: UnaryOpSide::Prefix,
            expr: Box::new(variable("x")),
            span: span(0, 2),
        },
        Expr::Grouping { expr: Box::new(number(1)), span: span(0, 3) },
        Expr::Assign {
            target: Box::new(variable("x")),
            value: Box::new(number(2)),
            span: span(0, 5),
        },
        Expr::ArrayAccess {
            array: Box::new(variable("items")),
            index: Box::new(number(0)),
            span: span(0, 8),
        },
    ];

    for expression in expressions {
        let output = strip_ansi_codes(&pretty_print(&expression));
        assert!(!output.is_empty());
    }
}

#[test]
fn pretty_print_statement_handles_expression_and_control_flow() {
    let condition = Expr::new_bool_literal(true, span(0, 1));
    let body = Stmt::Block {
        statements: vec![Stmt::Break { span: span(0, 1) }],
        span: span(0, 1),
    };
    let statement = Stmt::If {
        condition: Box::new(condition),
        then_branch: Box::new(body),
        else_branch: ElseBranch::Block(Box::new(Stmt::Continue { span: span(0, 1) })),
        span: span(0, 10),
    };

    let output = strip_ansi_codes(&pretty_print_stmt(&statement));

    assert!(output.contains("If"));
    assert!(output.contains("Condition:"));
    assert!(output.contains("Then:"));
    assert!(output.contains("Else:"));
    assert!(output.contains("Break"));
    assert!(output.contains("Continue"));
}

#[test]
fn pretty_print_statement_handles_empty_block_and_declaration_without_bindings() {
    let cases = [
        Stmt::Block { statements: Vec::new(), span: span(0, 0) },
        Stmt::VarDeclaration {
            bindings: Vec::new(),
            type_annotation: Type::I64,
            is_mutable: false,
            span: span(0, 0),
        },
    ];

    for statement in cases {
        let output = strip_ansi_codes(&pretty_print_stmt(&statement));
        assert!(!output.is_empty());
    }
}

#[test]
fn pretty_print_statement_handles_function_parameters_and_for_components() {
    let node_span = span(0, 3);
    let parameter = Parameter::new("value".into(), Type::I32, node_span.clone());
    let initializer = Stmt::VarDeclaration {
        bindings: vec![VarBinding {
            name: "i".into(),
            initializer: Some(number(0)),
        }],
        type_annotation: Type::I32,
        is_mutable: true,
        span: node_span.clone(),
    };
    let function = Stmt::Function {
        name: "compute".into(),
        parameters: vec![parameter],
        return_type: Type::F64,
        body: Box::new(Stmt::For {
            initializer: Some(Box::new(initializer)),
            condition: Some(Expr::new_bool_literal(true, node_span.clone())),
            increment: Some(Expr::Unary {
                op: UnaryOp::Increment,
                side: UnaryOpSide::Postfix,
                expr: Box::new(variable("i")),
                span: node_span.clone(),
            }),
            body: Box::new(Stmt::Block { statements: Vec::new(), span: node_span.clone() }),
            span: node_span,
        }),
        span: span(0, 20),
    };

    let output = strip_ansi_codes(&pretty_print_stmt(&function));

    assert!(output.contains("Function"));
    assert!(output.contains("compute"));
    assert!(output.contains("Parameters:"));
    assert!(output.contains("value"));
    assert!(output.contains("For"));
    assert!(output.contains("Initializer:"));
    assert!(output.contains("Condition:"));
    assert!(output.contains("Increment:"));
}

#[test]
fn pretty_print_statement_handles_all_simple_statements() {
    let span = span(0, 1);
    let statements = [
        Stmt::Return { value: None, span: span.clone() },
        Stmt::Return { value: Some(number(1)), span: span.clone() },
        Stmt::Break { span: span.clone() },
        Stmt::Continue { span: span.clone() },
        Stmt::MainFunction {
            body: Box::new(Stmt::Block { statements: Vec::new(), span: span.clone() }),
            span,
        },
    ];

    for statement in statements {
        let output = strip_ansi_codes(&pretty_print_stmt(&statement));
        assert!(!output.is_empty());
    }
}

#[test]
fn pretty_print_handles_deeply_nested_expression_without_changing_semantics() {
    let mut expression = number(1);

    for _ in 0..16 {
        expression = Expr::Grouping {
            expr: Box::new(expression),
            span: span(0, 3),
        };
    }

    let output = strip_ansi_codes(&pretty_print(&expression));

    assert_eq!(output.matches("Grouping").count(), 16);
    assert!(output.contains("Literal 1"));
}

#[test]
fn pretty_print_literal_variants_are_rendered() {
    let literals = [
        Expr::new_number_literal(Number::Integer(42), span(0, 2)),
        Expr::new_bool_literal(false, span(0, 5)),
        Expr::new_string_literal("hello".into(), span(0, 7)),
        Expr::new_char_literal("c".into(), span(0, 3)),
        Expr::new_nullptr_literal(span(0, 7)),
        Expr::ArrayLiteral {
            elements: vec![number(1), number(2)],
            span: span(0, 6),
        },
    ];

    for expression in literals {
        let output = strip_ansi_codes(&pretty_print(&expression));
        assert!(!output.is_empty());
    }
}

#[test]
fn pretty_print_is_ansi_independent_for_assertions() {
    let expression = Expr::new_bool_literal(true, span(0, 4));
    let raw = pretty_print(&expression);
    let stripped = strip_ansi_codes(&raw);

    assert_eq!(stripped, "Literal true\n");
}
