use std::sync::Arc;

use descar_core::{
    location::{source_location::SourceLocation, source_span::SourceSpan},
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
};
use insta::assert_snapshot;

fn span(start_offset: usize, end_offset: usize) -> SourceSpan {
    let start = SourceLocation::new(1, start_offset + 1, start_offset, start_offset, start_offset, start_offset);
    let end = SourceLocation::new(1, end_offset + 1, end_offset, end_offset, end_offset, end_offset);
    SourceSpan::new(Arc::from("test.lang"), start, end)
}

fn literal(value: LiteralValue) -> Expr {
    Expr::Literal { value, span: span(0, 1) }
}

#[test]
fn snapshots_expression_variants() {
    let node_span = span(0, 1);
    let variable = Expr::Variable { name: "value".into(), span: node_span.clone() };
    let number = literal(LiteralValue::Numeric(Number::Integer(42)));
    let expressions = [
        Expr::Binary {
            left: Box::new(variable.clone()),
            op: BinaryOp::Add,
            right: Box::new(number.clone()),
            span: node_span.clone(),
        },
        Expr::Unary {
            op: UnaryOp::Increment,
            side: UnaryOpSide::Postfix,
            expr: Box::new(variable.clone()),
            span: node_span.clone(),
        },
        Expr::Grouping { expr: Box::new(number.clone()), span: node_span.clone() },
        number,
        Expr::ArrayLiteral {
            elements: vec![variable.clone(), literal(LiteralValue::Bool(true))],
            span: node_span.clone(),
        },
        variable.clone(),
        Expr::Assign {
            target: Box::new(variable.clone()),
            value: Box::new(literal(LiteralValue::StringLit("hello".into()))),
            span: node_span.clone(),
        },
        Expr::Call {
            callee: Box::new(variable.clone()),
            arguments: vec![literal(LiteralValue::CharLit("c".into())), literal(LiteralValue::NullPtr)],
            span: node_span.clone(),
        },
        Expr::ArrayAccess {
            array: Box::new(variable),
            index: Box::new(literal(LiteralValue::Numeric(Number::UnsignedInteger(0)))),
            span: node_span,
        },
    ];

    let rendered = expressions
        .iter()
        .map(|expression| format!("{expression:?}"))
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("expression_variants", rendered);
}

#[test]
fn snapshots_statement_variants() {
    let node_span = span(0, 1);
    let condition = Expr::new_bool_literal(true, node_span.clone());
    let block = Stmt::Block { statements: vec![Stmt::Break { span: node_span.clone() }], span: node_span.clone() };
    let parameter = Parameter::new("value".into(), Type::I64, node_span.clone());
    let statements = [
        Stmt::Expression { expr: Box::new(condition.clone()) },
        Stmt::VarDeclaration {
            bindings: vec![VarBinding { name: "value".into(), initializer: Some(literal(LiteralValue::Numeric(Number::Integer(1)))) }],
            type_annotation: Type::I32,
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
                bindings: vec![VarBinding { name: "i".into(), initializer: Some(literal(LiteralValue::Numeric(Number::Integer(0)))) }],
                type_annotation: Type::I32,
                is_mutable: true,
                span: node_span.clone(),
            })),
            condition: Some(condition),
            increment: Some(Expr::Unary {
                op: UnaryOp::Increment,
                side: UnaryOpSide::Postfix,
                expr: Box::new(Expr::Variable { name: "i".into(), span: node_span.clone() }),
                span: node_span.clone(),
            }),
            body: Box::new(block.clone()),
            span: node_span.clone(),
        },
        block,
        Stmt::Return { value: Some(literal(LiteralValue::NullPtr)), span: node_span.clone() },
        Stmt::Break { span: node_span.clone() },
        Stmt::Continue { span: node_span.clone() },
        Stmt::MainFunction {
            body: Box::new(Stmt::Block { statements: Vec::new(), span: node_span.clone() }),
            span: node_span,
        },
    ];

    let rendered = statements
        .iter()
        .map(|statement| format!("{statement:?}"))
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("statement_variants", rendered);
}

#[test]
fn snapshots_types_and_branches() {
    let node_span = span(0, 1);
    let types = [
        Type::I8,
        Type::I16,
        Type::I32,
        Type::I64,
        Type::U8,
        Type::U16,
        Type::U32,
        Type::U64,
        Type::F32,
        Type::F64,
        Type::Char,
        Type::String,
        Type::Bool,
        Type::Custom { name: Arc::from("UserType") },
        Type::Array {
            element_type: Box::new(Type::U32),
            size: Box::new(Expr::new_number_literal(Number::Integer(8), node_span.clone())),
        },
        Type::Vector { element_type: Box::new(Type::String) },
        Type::Void,
        Type::NullPtr,
    ];
    let branches = [
        ElseBranch::None,
        ElseBranch::Block(Box::new(Stmt::Break { span: node_span.clone() })),
        ElseBranch::ElseIf(Box::new(Stmt::Continue { span: node_span })),
    ];

    let rendered = format!(
        "types:\n{}\nbranches:\n{}",
        types.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        branches.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
    );

    assert_snapshot!("types_and_branches", rendered);
}

#[test]
fn snapshots_operator_representations() {
    let binary = [
        BinaryOp::Add,
        BinaryOp::AddEqual,
        BinaryOp::Subtract,
        BinaryOp::SubtractEqual,
        BinaryOp::Multiply,
        BinaryOp::MultiplyEqual,
        BinaryOp::Divide,
        BinaryOp::DivideEqual,
        BinaryOp::Modulo,
        BinaryOp::ModuloEqual,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::BitwiseAnd,
        BinaryOp::BitwiseAndEqual,
        BinaryOp::BitwiseOr,
        BinaryOp::BitwiseOrEqual,
        BinaryOp::BitwiseXor,
        BinaryOp::BitwiseXorEqual,
        BinaryOp::ShiftLeft,
        BinaryOp::ShiftLeftEqual,
        BinaryOp::ShiftRight,
        BinaryOp::ShiftRightEqual,
    ];
    let unary = [
        UnaryOp::Negate,
        UnaryOp::Not,
        UnaryOp::BitwiseNot,
        UnaryOp::Increment,
        UnaryOp::Decrement,
    ];
    let sides = [UnaryOpSide::Prefix, UnaryOpSide::Postfix];

    let rendered = format!(
        "binary:\n{}\nunary:\n{}\nsides:\n{}",
        binary.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        unary.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        sides.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
    );

    assert_snapshot!("operator_representations", rendered);
}
