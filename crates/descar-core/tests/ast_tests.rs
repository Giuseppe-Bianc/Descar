use std::sync::Arc;

use descar_core::{
    error::{compile_error::CompileError, error_code::ErrorCode},
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
    tokens::{number::Number, token::Token, token_kind::TokenKind},
};

fn span(start_offset: usize, end_offset: usize) -> SourceSpan {
    let start = SourceLocation::new(1, start_offset + 1, start_offset, start_offset, start_offset, start_offset);
    let end = SourceLocation::new(1, end_offset + 1, end_offset, end_offset, end_offset, end_offset);
    SourceSpan::new(Arc::from("test.lang"), start, end)
}

#[test]
fn binary_op_get_op_maps_supported_tokens() {
    let cases = [
        (TokenKind::Plus, BinaryOp::Add),
        (TokenKind::PlusEqual, BinaryOp::AddEqual),
        (TokenKind::Minus, BinaryOp::Subtract),
        (TokenKind::MinusEqual, BinaryOp::SubtractEqual),
        (TokenKind::Star, BinaryOp::Multiply),
        (TokenKind::StarEqual, BinaryOp::MultiplyEqual),
        (TokenKind::Slash, BinaryOp::Divide),
        (TokenKind::SlashEqual, BinaryOp::DivideEqual),
        (TokenKind::Percent, BinaryOp::Modulo),
        (TokenKind::PercentEqual, BinaryOp::ModuloEqual),
        (TokenKind::EqualEqual, BinaryOp::Equal),
        (TokenKind::NotEqual, BinaryOp::NotEqual),
        (TokenKind::Less, BinaryOp::Less),
        (TokenKind::LessEqual, BinaryOp::LessEqual),
        (TokenKind::Greater, BinaryOp::Greater),
        (TokenKind::GreaterEqual, BinaryOp::GreaterEqual),
        (TokenKind::AndAnd, BinaryOp::And),
        (TokenKind::OrOr, BinaryOp::Or),
        (TokenKind::And, BinaryOp::BitwiseAnd),
        (TokenKind::AndEqual, BinaryOp::BitwiseAndEqual),
        (TokenKind::Or, BinaryOp::BitwiseOr),
        (TokenKind::OrEqual, BinaryOp::BitwiseOrEqual),
        (TokenKind::Xor, BinaryOp::BitwiseXor),
        (TokenKind::XorEqual, BinaryOp::BitwiseXorEqual),
        (TokenKind::ShiftLeft, BinaryOp::ShiftLeft),
        (TokenKind::ShiftLeftEqual, BinaryOp::ShiftLeftEqual),
        (TokenKind::ShiftRight, BinaryOp::ShiftRight),
        (TokenKind::ShiftRightEqual, BinaryOp::ShiftRightEqual),
    ];

    for (kind, expected) in cases {
        let token = Token { kind, span: span(0, 1) };

        assert_eq!(BinaryOp::get_op(&token).unwrap(), expected);
    }
}

#[test]
fn binary_op_get_op_rejects_non_binary_tokens() {
    let token = Token { kind: TokenKind::IdentifierAscii("value".into()), span: span(3, 8) };

    let error = BinaryOp::get_op(&token).unwrap_err();

    assert!(matches!(error, CompileError::SyntaxError { code: Some(ErrorCode::E1005), .. }));
}

#[test]
fn expression_literal_constructors_create_expected_values() {
    let literal_span = span(2, 6);

    assert_eq!(
        Expr::new_number_literal(Number::I32(-7), literal_span.clone()),
        Expr::Literal { value: LiteralValue::Numeric(Number::I32(-7)), span: literal_span.clone() }
    );
    assert_eq!(
        Expr::new_bool_literal(true, literal_span.clone()),
        Expr::Literal { value: LiteralValue::Bool(true), span: literal_span.clone() }
    );
    assert_eq!(
        Expr::new_string_literal("hello".into(), literal_span.clone()),
        Expr::Literal { value: LiteralValue::StringLit("hello".into()), span: literal_span.clone() }
    );
    assert_eq!(
        Expr::new_char_literal("é".into(), literal_span.clone()),
        Expr::Literal { value: LiteralValue::CharLit("é".into()), span: literal_span.clone() }
    );
    assert_eq!(Expr::new_nullptr_literal(literal_span.clone()), Expr::null_expr(literal_span.clone()));
    assert_eq!(
        Expr::null_expr(literal_span.clone()),
        Expr::Literal { value: LiteralValue::NullPtr, span: literal_span }
    );
}

#[test]
fn expression_span_returns_node_span_for_every_variant() {
    let node_span = span(10, 20);
    let literal = Expr::Literal { value: LiteralValue::Bool(true), span: node_span.clone() };
    let variable = Expr::Variable { name: "value".into(), span: node_span.clone() };
    let expressions = [
        Expr::Binary {
            left: Box::new(variable.clone()),
            op: BinaryOp::Add,
            right: Box::new(literal.clone()),
            span: node_span.clone(),
        },
        Expr::Unary {
            op: UnaryOp::Negate,
            side: UnaryOpSide::Prefix,
            expr: Box::new(variable.clone()),
            span: node_span.clone(),
        },
        Expr::Grouping { expr: Box::new(literal.clone()), span: node_span.clone() },
        literal.clone(),
        Expr::ArrayLiteral { elements: Vec::new(), span: node_span.clone() },
        variable,
        Expr::Assign {
            target: Box::new(Expr::Variable { name: "value".into(), span: node_span.clone() }),
            value: Box::new(literal.clone()),
            span: node_span.clone(),
        },
        Expr::Call {
            callee: Box::new(Expr::Variable { name: "call".into(), span: node_span.clone() }),
            arguments: vec![literal.clone()],
            span: node_span.clone(),
        },
        Expr::ArrayAccess {
            array: Box::new(Expr::ArrayLiteral { elements: vec![literal], span: node_span.clone() }),
            index: Box::new(Expr::new_number_literal(Number::Integer(0), node_span.clone())),
            span: node_span.clone(),
        },
    ];

    for expression in expressions {
        assert_eq!(expression.span(), &node_span);
    }
}

#[test]
fn expression_edge_cases_preserve_empty_collections_and_deep_nesting() {
    let node_span = span(0, 0);
    let empty_array = Expr::ArrayLiteral { elements: Vec::new(), span: node_span.clone() };
    let empty_call = Expr::Call {
        callee: Box::new(Expr::Variable { name: "f".into(), span: node_span.clone() }),
        arguments: Vec::new(),
        span: node_span.clone(),
    };
    let nested = Expr::Grouping {
        expr: Box::new(Expr::Grouping {
            expr: Box::new(Expr::Grouping { expr: Box::new(empty_array.clone()), span: node_span.clone() }),
            span: node_span.clone(),
        }),
        span: node_span.clone(),
    };

    assert_eq!(empty_array.span(), &node_span);
    assert_eq!(empty_call.span(), &node_span);
    assert_eq!(nested.span(), &node_span);
}

#[test]
fn statement_span_returns_expected_span_for_every_variant() {
    let statement_span = span(4, 12);
    let expression = Expr::Literal { value: LiteralValue::Bool(false), span: statement_span.clone() };
    let parameter = Parameter::new("value".into(), Type::I64, statement_span.clone());
    let statements = [
        Stmt::Expression { expr: Box::new(expression.clone()) },
        Stmt::VarDeclaration {
            bindings: vec![VarBinding { name: "value".into(), initializer: Some(expression.clone()) }],
            type_annotation: Type::I64,
            is_mutable: true,
            span: statement_span.clone(),
        },
        Stmt::Function {
            name: "f".into(),
            parameters: vec![parameter],
            return_type: Type::Void,
            body: Box::new(Stmt::Block { statements: Vec::new(), span: statement_span.clone() }),
            span: statement_span.clone(),
        },
        Stmt::If {
            condition: Box::new(expression.clone()),
            then_branch: Box::new(Stmt::Block { statements: Vec::new(), span: statement_span.clone() }),
            else_branch: ElseBranch::None,
            span: statement_span.clone(),
        },
        Stmt::While {
            condition: Box::new(expression),
            body: Box::new(Stmt::Block { statements: Vec::new(), span: statement_span.clone() }),
            span: statement_span.clone(),
        },
        Stmt::For {
            initializer: None,
            condition: None,
            increment: None,
            body: Box::new(Stmt::Block { statements: Vec::new(), span: statement_span.clone() }),
            span: statement_span.clone(),
        },
        Stmt::Block { statements: Vec::new(), span: statement_span.clone() },
        Stmt::Return { value: None, span: statement_span.clone() },
        Stmt::Break { span: statement_span.clone() },
        Stmt::Continue { span: statement_span.clone() },
        Stmt::MainFunction {
            body: Box::new(Stmt::Block { statements: Vec::new(), span: statement_span.clone() }),
            span: statement_span.clone(),
        },
    ];

    for statement in statements {
        assert_eq!(statement.span(), &statement_span);
    }

    let expression_span = span(20, 23);
    let expression_statement = Stmt::Expression {
        expr: Box::new(Expr::Literal { value: LiteralValue::NullPtr, span: expression_span.clone() }),
    };

    assert_eq!(expression_statement.span(), &expression_span);
}

#[test]
fn statement_edge_cases_support_empty_and_optional_components() {
    let node_span = span(0, 0);
    let empty_declaration = Stmt::VarDeclaration {
        bindings: Vec::new(),
        type_annotation: Type::Void,
        is_mutable: false,
        span: node_span.clone(),
    };
    let empty_for = Stmt::For {
        initializer: None,
        condition: None,
        increment: None,
        body: Box::new(Stmt::Block { statements: Vec::new(), span: node_span.clone() }),
        span: node_span.clone(),
    };
    let empty_if = Stmt::If {
        condition: Box::new(Expr::new_bool_literal(false, node_span.clone())),
        then_branch: Box::new(Stmt::Block { statements: Vec::new(), span: node_span.clone() }),
        else_branch: ElseBranch::Block(Box::new(Stmt::Block { statements: Vec::new(), span: node_span.clone() })),
        span: node_span.clone(),
    };

    assert_eq!(empty_declaration.span(), &node_span);
    assert_eq!(empty_for.span(), &node_span);
    assert_eq!(empty_if.span(), &node_span);
}

#[test]
fn parameter_new_preserves_name_type_and_span() {
    let expected_span = span(7, 11);
    let parameter = Parameter::new("count".into(), Type::U32, expected_span.clone());

    assert_eq!(parameter.name, "count");
    assert_eq!(parameter.type_annotation, Type::U32);
    assert_eq!(parameter.span, expected_span);
}

#[test]
fn complex_types_and_else_branches_preserve_nested_values() {
    let node_span = span(0, 2);
    let array_type = Type::Array {
        element_type: Box::new(Type::Custom { name: Arc::from("Record") }),
        size: Box::new(Expr::new_number_literal(Number::UnsignedInteger(0), node_span.clone())),
    };
    let vector_type = Type::Vector { element_type: Box::new(Type::String) };
    let else_if = ElseBranch::ElseIf(Box::new(Stmt::Break { span: node_span.clone() }));

    assert_eq!(
        array_type,
        Type::Array {
            element_type: Box::new(Type::Custom { name: Arc::from("Record") }),
            size: Box::new(Expr::new_number_literal(Number::UnsignedInteger(0), node_span.clone())),
        }
    );
    assert_eq!(vector_type, Type::Vector { element_type: Box::new(Type::String) });
    assert_eq!(else_if, ElseBranch::ElseIf(Box::new(Stmt::Break { span: node_span })));
}

#[test]
fn ast_values_clone_and_compare_structurally() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Variable { name: "x".into(), span: span(0, 1) }),
        op: BinaryOp::ShiftLeft,
        right: Box::new(Expr::new_number_literal(Number::Integer(2), span(4, 5))),
        span: span(0, 5),
    };
    let cloned = expression.clone();

    assert_eq!(expression, cloned);
    assert_ne!(expression, Expr::new_bool_literal(true, span(0, 1)));
}
