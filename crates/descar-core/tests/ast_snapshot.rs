use std::sync::Arc;

use descar_core::syntax::ast::{ast_type::Type, binary_op::BinaryOp, else_branch::ElseBranch, expr::Expr, literal_value::LiteralValue, stmt::Stmt, unary_op::UnaryOp, unary_op_side::UnaryOpSide};
use descar_core::tokens::number::Number;
use insta::assert_snapshot;

fn literal(value: LiteralValue) -> Expr {
    Expr::Literal { value, span: Default::default() }
}

#[test]
fn snapshots_expression_variants() {
    let variable = Expr::Variable { name: "value".into(), span: Default::default() };
    let number = literal(LiteralValue::Numeric(Number::Integer(42)));
    let expressions = [
        Expr::Binary { left: Box::new(variable.clone()), op: BinaryOp::Add, right: Box::new(number.clone()), span: Default::default() },
        Expr::Unary { op: UnaryOp::Increment, side: UnaryOpSide::Postfix, expr: Box::new(variable.clone()), span: Default::default() },
        Expr::Grouping { expr: Box::new(number.clone()), span: Default::default() },
        number,
        Expr::ArrayLiteral { elements: vec![variable.clone(), literal(LiteralValue::Bool(true))], span: Default::default() },
        variable.clone(),
        Expr::Assign { target: Box::new(variable.clone()), value: Box::new(literal(LiteralValue::StringLit("hello".into()))), span: Default::default() },
        Expr::Call { callee: Box::new(variable.clone()), arguments: vec![literal(LiteralValue::CharLit("c".into())), literal(LiteralValue::NullPtr)], span: Default::default() },
        Expr::ArrayAccess { array: Box::new(variable), index: Box::new(literal(LiteralValue::Numeric(Number::UnsignedInteger(0)))), span: Default::default() },
    ];

    let rendered = expressions.iter().map(|expression| match expression {
        Expr::Binary { op, .. } => format!("Binary::{op:?}"),
        Expr::Unary { op, side, .. } => format!("Unary::{op:?}::{side:?}"),
        Expr::Grouping { .. } => "Grouping".into(),
        Expr::Literal { value, .. } => format!("Literal::{value:?}"),
        Expr::ArrayLiteral { elements, .. } => format!("ArrayLiteral::len={}", elements.len()),
        Expr::Variable { name, .. } => format!("Variable::{name}"),
        Expr::Assign { .. } => "Assign".into(),
        Expr::Call { arguments, .. } => format!("Call::arguments={}", arguments.len()),
        Expr::ArrayAccess { .. } => "ArrayAccess".into(),
    }).collect::<Vec<_>>().join("\n");

    assert_snapshot!("expression_variants", rendered);
}

#[test]
fn snapshots_statement_variants() {
    let condition = Expr::new_bool_literal(true, Default::default());
    let block = Stmt::Block { statements: vec![Stmt::Break { span: Default::default() }], span: Default::default() };
    let statements = [
        Stmt::Expression { expr: Box::new(condition.clone()) },
        Stmt::VarDeclaration { bindings: vec![], type_annotation: Type::I32, is_mutable: true, span: Default::default() },
        Stmt::Function { name: "compute".into(), parameters: vec![], return_type: Type::F64, body: Box::new(block.clone()), span: Default::default() },
        Stmt::If { condition: Box::new(condition.clone()), then_branch: Box::new(block.clone()), else_branch: ElseBranch::None, span: Default::default() },
        Stmt::While { condition: Box::new(condition.clone()), body: Box::new(block.clone()), span: Default::default() },
        Stmt::For { initializer: None, condition: None, increment: None, body: Box::new(block.clone()), span: Default::default() },
        block,
        Stmt::Return { value: Some(literal(LiteralValue::NullPtr)), span: Default::default() },
        Stmt::Break { span: Default::default() },
        Stmt::Continue { span: Default::default() },
        Stmt::MainFunction { body: Box::new(Stmt::Block { statements: vec![], span: Default::default() }), span: Default::default() },
    ];

    let rendered = statements.iter().map(|statement| match statement {
        Stmt::Expression { .. } => "Expression",
        Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => if *is_mutable { format!("VarDeclaration::{type_annotation:?}::bindings={}", bindings.len()) } else { format!("ConstDeclaration::{type_annotation:?}::bindings={}", bindings.len()) },
        Stmt::Function { name, parameters, return_type, .. } => format!("Function::{name}::{return_type:?}::parameters={}", parameters.len()),
        Stmt::If { else_branch, .. } => format!("If::else={else_branch:?}"),
        Stmt::While { .. } => "While",
        Stmt::For { initializer, condition, increment, .. } => format!("For::initializer={}::condition={}::increment={}", initializer.is_some(), condition.is_some(), increment.is_some()),
        Stmt::Block { statements, .. } => format!("Block::statements={}", statements.len()),
        Stmt::Return { value, .. } => format!("Return::value={}", value.is_some()),
        Stmt::Break { .. } => "Break".into(),
        Stmt::Continue { .. } => "Continue".into(),
        Stmt::MainFunction { .. } => "MainFunction".into(),
    }).collect::<Vec<_>>().join("\n");

    assert_snapshot!("statement_variants", rendered);
}

#[test]
fn snapshots_types_and_branches() {
    let types = [
        Type::I8, Type::I16, Type::I32, Type::I64, Type::U8, Type::U16, Type::U32, Type::U64,
        Type::F32, Type::F64, Type::Char, Type::String, Type::Bool,
        Type::Custom { name: Arc::from("UserType") },
        Type::Array { element_type: Box::new(Type::U32), size: Box::new(literal(LiteralValue::Numeric(Number::Integer(8)))) },
        Type::Vector { element_type: Box::new(Type::String) }, Type::Void, Type::NullPtr,
    ];
    let branches = [ElseBranch::None, ElseBranch::Block(Box::new(Stmt::Break { span: Default::default() })), ElseBranch::ElseIf(Box::new(Stmt::Continue { span: Default::default() }))];
    let rendered = format!("types:\n{}\nbranches:\n{}", types.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"), branches.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"));
    assert_snapshot!("types_and_branches", rendered);
}

#[test]
fn snapshots_operator_representations() {
    let binary = [BinaryOp::Add, BinaryOp::AddEqual, BinaryOp::Subtract, BinaryOp::SubtractEqual, BinaryOp::Multiply, BinaryOp::MultiplyEqual, BinaryOp::Divide, BinaryOp::DivideEqual, BinaryOp::Modulo, BinaryOp::ModuloEqual, BinaryOp::Equal, BinaryOp::NotEqual, BinaryOp::Less, BinaryOp::LessEqual, BinaryOp::Greater, BinaryOp::GreaterEqual, BinaryOp::And, BinaryOp::Or, BinaryOp::BitwiseAnd, BinaryOp::BitwiseAndEqual, BinaryOp::BitwiseOr, BinaryOp::BitwiseOrEqual, BinaryOp::BitwiseXor, BinaryOp::BitwiseXorEqual, BinaryOp::ShiftLeft, BinaryOp::ShiftLeftEqual, BinaryOp::ShiftRight, BinaryOp::ShiftRightEqual];
    let unary = [UnaryOp::Negate, UnaryOp::Not, UnaryOp::BitwiseNot, UnaryOp::Increment, UnaryOp::Decrement];
    let sides = [UnaryOpSide::Prefix, UnaryOpSide::Postfix];
    let rendered = format!("binary:\n{}\nunary:\n{}\nsides:\n{}", binary.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"), unary.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"), sides.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"));
    assert_snapshot!("operator_representations", rendered);
}
