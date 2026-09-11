use descar_core::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    lex::lexer::{Lexer, lexer_tokenize_with_errors},
    syntax::{
        ast::{
            ast_type::Type, binary_op::BinaryOp, else_branch::ElseBranch, expr::Expr, literal_value::LiteralValue,
            stmt::Stmt, unary_op::UnaryOp, unary_op_side::UnaryOpSide,
        },
        parser::JsavParser,
    },
};

fn parse(input: &str) -> (Vec<Stmt>, Vec<CompileError>) {
    let mut lexer = Lexer::new("parser_test", input);
    let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);
    assert!(lexer_errors.is_empty(), "unexpected lexer errors: {lexer_errors:#?}");

    JsavParser::new(&tokens).parse()
}

fn assert_no_errors(errors: &[CompileError]) {
    assert!(errors.is_empty(), "unexpected parser errors: {errors:#?}");
}

fn assert_has_error(errors: &[CompileError], expected: ErrorCode) {
    assert!(
        errors.iter().any(|error| error.error_code() == Some(&expected)),
        "expected {expected:?} in parser errors: {errors:#?}"
    );
}

#[test]
fn empty_input_produces_no_statements_or_errors() {
    let (statements, errors) = parse("");

    assert!(statements.is_empty());
    assert_no_errors(&errors);
}

#[test]
fn parses_literal_and_variable_expression_statements() {
    let input = "42\ntrue\nnullptr\n\"hello\"\n'a'\nvalue";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 6);

    assert!(matches!(
        &statements[0],
        Stmt::Expression {
            expr: expression
        } if matches!(expression.as_ref(), Expr::Literal {
            value: LiteralValue::Numeric(_),
            ..
        })
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression {
            expr: expression
        } if matches!(expression.as_ref(), Expr::Literal {
            value: LiteralValue::Bool(true),
            ..
        })
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Expression {
            expr: expression
        } if matches!(expression.as_ref(), Expr::Literal {
            value: LiteralValue::NullPtr,
            ..
        })
    ));
    assert!(matches!(
        &statements[3],
        Stmt::Expression {
            expr: expression
        } if matches!(expression.as_ref(), Expr::Literal {
            value: LiteralValue::StringLit(value),
            ..
        } if value == "hello")
    ));
    assert!(matches!(
        &statements[4],
        Stmt::Expression {
            expr: expression
        } if matches!(expression.as_ref(), Expr::Literal {
            value: LiteralValue::CharLit(value),
            ..
        } if value == "a")
    ));
    assert!(matches!(
        &statements[5],
        Stmt::Expression {
            expr: expression
        } if matches!(expression.as_ref(), Expr::Variable { name, .. } if name == "value")
    ));
}

#[test]
fn parses_assignment_calls_and_array_access() {
    let input = "items = {1, 2, 3}\nitems[1]\nprint(items[0])";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 3);

    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Assign {
            target,
            value,
            ..
        } if matches!(target.as_ref(), Expr::Variable { name, .. } if name == "items")
            && matches!(value.as_ref(), Expr::ArrayLiteral { elements, .. } if elements.len() == 3))
    ));

    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::ArrayAccess { array, index, .. }
            if matches!(array.as_ref(), Expr::Variable { name, .. } if name == "items")
                && matches!(index.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. }))
    ));

    assert!(matches!(
        &statements[2],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Call { callee, arguments, .. }
            if matches!(callee.as_ref(), Expr::Variable { name, .. } if name == "print")
                && arguments.len() == 1
                && matches!(arguments[0], Expr::ArrayAccess { .. }))
    ));
}

#[test]
fn parses_prefix_and_postfix_unary_expressions() {
    let input = "++value\nvalue--\n!ready\n~bits";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);

    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Increment, side: UnaryOpSide::Prefix, .. })
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Decrement, side: UnaryOpSide::Postfix, .. })
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Not, side: UnaryOpSide::Prefix, .. })
    ));
    assert!(matches!(
        &statements[3],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::BitwiseNot, side: UnaryOpSide::Prefix, .. })
    ));
}

#[test]
fn respects_binary_precedence_and_grouping() {
    let input = "a + b * c\n(a + b) * c";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 2);

    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Add, right, .. }
            if matches!(right.as_ref(), Expr::Binary { op: BinaryOp::Multiply, .. }))
    ));

    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
        if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Multiply, left, .. }
            if matches!(left.as_ref(), Expr::Grouping { .. }))
    ));
}

#[test]
fn parses_variable_and_const_declarations_with_multiple_bindings() {
    let input = "var first, second: i32 = 1, 2\nconst answer: i64 = 42";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 2);

    match &statements[0] {
        Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => {
            assert!(*is_mutable);
            assert_eq!(*type_annotation, Type::I32);
            assert_eq!(bindings.len(), 2);
            assert_eq!(bindings[0].name, "first");
            assert_eq!(bindings[1].name, "second");
            assert!(bindings[0].initializer.is_some());
            assert!(bindings[1].initializer.is_some());
        }
        other => panic!("unexpected statement: {other:#?}"),
    }

    match &statements[1] {
        Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => {
            assert!(!*is_mutable);
            assert_eq!(*type_annotation, Type::I64);
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "answer");
            assert!(bindings[0].initializer.is_some());
        }
        other => panic!("unexpected statement: {other:#?}"),
    }
}

#[test]
fn parses_functions_and_main_with_parameters_and_return_type() {
    let input = "fun add(a: i32, b: i32): i32 { return a + b }\nmain { return }";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 2);

    match &statements[0] {
        Stmt::Function { name, parameters, return_type, body, .. } => {
            assert_eq!(name, "add");
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "a");
            assert_eq!(parameters[0].type_annotation, Type::I32);
            assert_eq!(parameters[1].name, "b");
            assert_eq!(parameters[1].type_annotation, Type::I32);
            assert_eq!(*return_type, Type::I32);
            assert!(
                matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Return { value: Some(_), .. }]))
            );
        }
        other => panic!("unexpected statement: {other:#?}"),
    }

    assert!(matches!(
        &statements[1],
        Stmt::MainFunction { body, .. }
        if matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Return { value: None, .. }]))
    ));
}

#[test]
fn parses_array_vector_custom_and_unicode_types() {
    let input = "var matrix: i32[2][3] = {{1, 2, 3}, {4, 5, 6}}\nvar values: vector<string> = {\"a\", \"b\"}\nvar record: Record = value\nvar 変数: i64 = 1";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);

    match &statements[0] {
        Stmt::VarDeclaration { type_annotation, bindings, .. } => {
            assert!(matches!(
                type_annotation,
                Type::Array { element_type, size, .. }
                if matches!(element_type.as_ref(), Type::Array { element_type, .. } if matches!(element_type.as_ref(), Type::I32))
                    && matches!(size.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. })
            ));
            assert!(bindings[0].initializer.is_some());
        }
        other => panic!("unexpected statement: {other:#?}"),
    }

    assert!(matches!(
        &statements[1],
        Stmt::VarDeclaration { type_annotation: Type::Vector { element_type }, .. }
        if matches!(element_type.as_ref(), Type::String)
    ));
    assert!(matches!(
        &statements[2],
        Stmt::VarDeclaration { type_annotation: Type::Custom { name }, .. }
        if name.as_ref() == "Record"
    ));
    assert!(matches!(
        &statements[3],
        Stmt::VarDeclaration { bindings, .. }
        if bindings[0].name == "變數"
    ));
}

#[test]
fn parses_if_else_while_and_for_control_flow() {
    let input = "if (ready) { value } else if (fallback) { other } else { final }\nwhile (ready) { break }\nfor (var i: i32 = 0; i < 10; i++) { continue }\nfor (;;) { break }";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);

    assert!(matches!(&statements[0], Stmt::If { else_branch: ElseBranch::ElseIf(_), .. }));
    assert!(matches!(
        &statements[1],
        Stmt::While { body, .. }
        if matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Break { .. }]))
    ));
    assert!(matches!(&statements[2], Stmt::For { initializer: Some(_), condition: Some(_), increment: Some(_), .. }));
    assert!(matches!(
        &statements[3],
        Stmt::For { initializer: None, condition: None, increment: None, body, .. }
        if matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Break { .. }]))
    ));
}

#[test]
fn rejects_invalid_assignment_target_without_panicking() {
    let (statements, errors) = parse("42 = 1\nvalue = 2");

    assert_has_error(&errors, ErrorCode::E1003);
    assert!(statements.iter().any(|statement| {
        matches!(
            statement,
            Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Assign { target, .. }
                if matches!(target.as_ref(), Expr::Variable { name, .. } if name == "value"))
        )
    }));
}

#[test]
fn reports_missing_delimiters_as_syntax_errors() {
    let (_, errors) = parse("if (true) { value\nfun broken(a: i32 { return a");

    assert_has_error(&errors, ErrorCode::E1004);
}

#[test]
fn recovers_from_unexpected_tokens_and_keeps_following_statements() {
    let (statements, errors) = parse(")\n42\nvalue");

    assert_has_error(&errors, ErrorCode::E1004);
    assert!(statements.len() >= 2);
    assert!(statements.iter().any(|statement| {
        matches!(
            statement,
            Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. })
        )
    }));
    assert!(statements.iter().any(|statement| {
        matches!(
            statement,
            Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Variable { name, .. } if name == "value")
        )
    }));
}

#[test]
fn reports_maximum_recursion_depth_for_deep_grouping() {
    let input = format!("{}42{}", "(".repeat(1_005), ")".repeat(1_005));
    let (_, errors) = parse(&input);

    assert_has_error(&errors, ErrorCode::E1001);
}
