use std::sync::Arc;

use descar_core::location::source_span::SourceSpan;
use descar_core::syntax::ast::{
    assignment_op::AssignmentOp, type_syntax::TypeSyntax, binary_op::BinaryOp, else_branch::ElseBranch, expr::Expr,
    literal_value::LiteralValue, stmt::Stmt, unary_op::UnaryOp, unary_op_side::UnaryOpSide,
};
use descar_core::tokens::number::Number;
use insta::assert_snapshot;

fn literal(value: LiteralValue) -> Expr {
    Expr::Literal { value, span: SourceSpan::default() }
}

#[test]
fn snapshots_expression_variants() {
    let variable = Expr::Variable { name: "value".into(), span: SourceSpan::default() };
    let number = literal(LiteralValue::Numeric(Number::Integer(42)));
    let expressions = [
        Expr::Binary {
            left: Box::new(variable.clone()),
            op: BinaryOp::Add,
            right: Box::new(number.clone()),
            span: SourceSpan::default(),
        },
        Expr::Cast { expr: Box::new(variable.clone()), target_type: TypeSyntax::I32, span: SourceSpan::default() },
        Expr::Unary {
            op: UnaryOp::Increment,
            side: UnaryOpSide::Postfix,
            expr: Box::new(variable.clone()),
            span: SourceSpan::default(),
        },
        Expr::Grouping { expr: Box::new(number.clone()), span: SourceSpan::default() },
        number,
        Expr::ArrayLiteral {
            elements: vec![variable.clone(), literal(LiteralValue::Bool(true))],
            span: SourceSpan::default(),
        },
        variable.clone(),
        Expr::Assign {
            op: AssignmentOp::Assign,
            target: Box::new(variable.clone()),
            value: Box::new(literal(LiteralValue::StringLit("hello".into()))),
            span: SourceSpan::default(),
        },
        Expr::Call {
            callee: Box::new(variable.clone()),
            arguments: vec![literal(LiteralValue::CharLit("c".into())), literal(LiteralValue::NullPtr)],
            span: SourceSpan::default(),
        },
        Expr::ArrayAccess {
            array: Box::new(variable),
            index: Box::new(literal(LiteralValue::Numeric(Number::UnsignedInteger(0)))),
            span: SourceSpan::default(),
        },
    ];

    let rendered = expressions
        .iter()
        .map(|expression| match expression {
            Expr::Binary { op, .. } => format!("Binary::{op:?}"),
            Expr::Cast { target_type, .. } => format!("Cast::{target_type}"),
            Expr::Unary { op, side, .. } => format!("Unary::{op:?}::{side:?}"),
            Expr::Grouping { .. } => String::from("Grouping"),
            Expr::Literal { value, .. } => format!("Literal::{value:?}"),
            Expr::ArrayLiteral { elements, .. } => format!("ArrayLiteral::len={}", elements.len()),
            Expr::Variable { name, .. } => format!("Variable::{name}"),
            Expr::Assign { .. } => String::from("Assign"),
            Expr::Call { arguments, .. } => format!("Call::arguments={}", arguments.len()),
            Expr::ArrayAccess { .. } => String::from("ArrayAccess"),
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("expression_variants", rendered);
}

#[test]
fn snapshots_statement_variants() {
    let condition = Expr::new_bool_literal(true, SourceSpan::default());
    let block =
        Stmt::Block { statements: vec![Stmt::Break { span: SourceSpan::default() }], span: SourceSpan::default() };
    let statements = [
        Stmt::Expression { expr: Box::new(condition.clone()) },
        Stmt::VarDeclaration {
            bindings: vec![],
            type_annotation: TypeSyntax::I32,
            is_mutable: true,
            span: SourceSpan::default(),
        },
        Stmt::Function {
            name: "compute".into(),
            parameters: vec![],
            return_type: TypeSyntax::F64,
            body: Box::new(block.clone()),
            span: SourceSpan::default(),
        },
        Stmt::If {
            condition: Box::new(condition.clone()),
            then_branch: Box::new(block.clone()),
            else_branch: ElseBranch::None,
            span: SourceSpan::default(),
        },
        Stmt::While { condition: Box::new(condition), body: Box::new(block.clone()), span: SourceSpan::default() },
        Stmt::For {
            initializer: None,
            condition: None,
            increment: None,
            body: Box::new(block.clone()),
            span: SourceSpan::default(),
        },
        block,
        Stmt::Return { value: Some(literal(LiteralValue::NullPtr)), span: SourceSpan::default() },
        Stmt::Break { span: SourceSpan::default() },
        Stmt::Continue { span: SourceSpan::default() },
        Stmt::MainFunction {
            body: Box::new(Stmt::Block { statements: vec![], span: SourceSpan::default() }),
            span: SourceSpan::default(),
        },
    ];

    let rendered = statements
        .iter()
        .map(|statement| match statement {
            Stmt::Expression { .. } => String::from("Expression"),
            Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => {
                if *is_mutable {
                    format!("VarDeclaration::{type_annotation:?}::bindings={}", bindings.len())
                } else {
                    format!("ConstDeclaration::{type_annotation:?}::bindings={}", bindings.len())
                }
            }
            Stmt::Function { name, parameters, return_type, .. } => {
                format!("Function::{name}::{return_type:?}::parameters={}", parameters.len())
            }
            Stmt::If { else_branch, .. } => format!("If::else={else_branch:?}"),
            Stmt::While { .. } => String::from("While"),
            Stmt::For { initializer, condition, increment, .. } => format!(
                "For::initializer={}::condition={}::increment={}",
                initializer.is_some(),
                condition.is_some(),
                increment.is_some()
            ),
            Stmt::Block { statements, .. } => format!("Block::statements={}", statements.len()),
            Stmt::Return { value, .. } => format!("Return::value={}", value.is_some()),
            Stmt::Break { .. } => String::from("Break"),
            Stmt::Continue { .. } => String::from("Continue"),
            Stmt::MainFunction { .. } => String::from("MainFunction"),
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("statement_variants", rendered);
}

#[test]
fn snapshots_types_and_branches() {
    let types = [
        TypeSyntax::I8,
        TypeSyntax::I16,
        TypeSyntax::I32,
        TypeSyntax::I64,
        TypeSyntax::U8,
        TypeSyntax::U16,
        TypeSyntax::U32,
        TypeSyntax::U64,
        TypeSyntax::F32,
        TypeSyntax::F64,
        TypeSyntax::Char,
        TypeSyntax::String,
        TypeSyntax::Bool,
        TypeSyntax::Custom { name: Arc::from("UserType") },
        TypeSyntax::Array {
            element_type: Box::new(TypeSyntax::U32),
            size: Box::new(literal(LiteralValue::Numeric(Number::Integer(8)))),
        },
        TypeSyntax::Vector { element_type: Box::new(TypeSyntax::String) },
        TypeSyntax::Void,
        TypeSyntax::NullPtr,
    ];
    let branches = [
        ElseBranch::None,
        ElseBranch::Block(Box::new(Stmt::Break { span: SourceSpan::default() })),
        ElseBranch::ElseIf(Box::new(Stmt::Continue { span: SourceSpan::default() })),
    ];
    let rendered = format!(
        "types:\n{}\nbranches:\n{}",
        types.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        branches.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n")
    );
    assert_snapshot!("types_and_branches", rendered);
}

#[test]
fn snapshots_operator_representations() {
    let binary = [
        BinaryOp::Add,
        BinaryOp::Subtract,
        BinaryOp::Multiply,
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::BitwiseAnd,
        BinaryOp::BitwiseOr,
        BinaryOp::BitwiseXor,
        BinaryOp::ShiftLeft,
        BinaryOp::ShiftRight,
    ];
    let assignment = [
        AssignmentOp::Assign,
        AssignmentOp::AddAssign,
        AssignmentOp::SubtractAssign,
        AssignmentOp::MultiplyAssign,
        AssignmentOp::DivideAssign,
        AssignmentOp::ModuloAssign,
        AssignmentOp::BitwiseAndAssign,
        AssignmentOp::BitwiseOrAssign,
        AssignmentOp::BitwiseXorAssign,
        AssignmentOp::ShiftLeftAssign,
        AssignmentOp::ShiftRightAssign,
    ];
    let unary = [UnaryOp::Negate, UnaryOp::Not, UnaryOp::BitwiseNot, UnaryOp::Increment, UnaryOp::Decrement];
    let sides = [UnaryOpSide::Prefix, UnaryOpSide::Postfix];
    let rendered = format!(
        "binary:\n{}\nassignment:\n{}\nunary:\n{}\nsides:\n{}",
        binary.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        assignment.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        unary.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n"),
        sides.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>().join("\n")
    );
    assert_snapshot!("operator_representations", rendered);
}
