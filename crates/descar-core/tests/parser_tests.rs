use descar_core::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    lex::lexer::{Lexer, lexer_tokenize_with_errors},
    syntax::{
        ast::{
            assignment_op::AssignmentOp, binary_op::BinaryOp, else_branch::ElseBranch, expr::Expr,
            literal_value::LiteralValue, stmt::Stmt, type_syntax::TypeSyntax, unary_op::UnaryOp,
            unary_op_side::UnaryOpSide,
        },
        parser::JsavParser,
        precendence::Precedence,
    },
};

fn parse_unit(input: &str) -> (descar_core::syntax::ast::CompilationUnit, Vec<CompileError>) {
    let mut lexer = Lexer::new("parser_test", input);
    let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);
    assert!(lexer_errors.is_empty(), "unexpected lexer errors: {lexer_errors:#?}");
    JsavParser::new(&tokens).parse()
}

fn parse(input: &str) -> (descar_core::syntax::ast::CompilationUnit, Vec<CompileError>) {
    let trimmed = input.trim_start();
    if trimmed.is_empty()
        || trimmed.starts_with("fun ")
        || trimmed.starts_with("main ")
        || trimmed.starts_with("var ")
        || trimmed.starts_with("const ")
    {
        return parse_unit(input);
    }

    // Most tests in this module exercise statement parsing. Parse those
    // statements in a main body so the compilation-unit grammar stays strict.
    let wrapped = format!("main {{ {input} }}");
    let (mut unit, errors) = parse_unit(&wrapped);
    if let Some(Stmt::MainFunction { body, .. }) = unit.declarations.pop()
        && let Stmt::Block { statements, .. } = *body
    {
        unit.declarations = statements;
    }
    (unit, errors)
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

    assert!(
        matches!(&statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. }))
    );
    assert!(
        matches!(&statements[1], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::Bool(true), .. }))
    );
    assert!(
        matches!(&statements[2], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::NullPtr, .. }))
    );
    assert!(
        matches!(&statements[3], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::StringLit(value), .. } if value == "hello"))
    );
    assert!(
        matches!(&statements[4], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::CharLit(value), .. } if value == "a"))
    );
    assert!(
        matches!(&statements[5], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Variable { name, .. } if name == "value"))
    );
}

#[test]
fn parses_assignment_calls_and_array_access() {
    let input = "items = {1, 2, 3}\nitems[1]\nprint(items[0])";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 3);

    assert!(
        matches!(&statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Assign { op: AssignmentOp::Assign, target, value, .. } if matches!(target.as_ref(), Expr::Variable { name, .. } if name == "items") && matches!(value.as_ref(), Expr::ArrayLiteral { elements, .. } if elements.len() == 3)))
    );
    assert!(
        matches!(&statements[1], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::ArrayAccess { array, index, .. } if matches!(array.as_ref(), Expr::Variable { name, .. } if name == "items") && matches!(index.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. })))
    );
    assert!(
        matches!(&statements[2], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Call { callee, arguments, .. } if matches!(callee.as_ref(), Expr::Variable { name, .. } if name == "print") && arguments.len() == 1 && matches!(arguments[0], Expr::ArrayAccess { .. })))
    );
}

#[test]
fn parses_all_compound_assignment_operators() {
    let cases = [
        ("x += 1", AssignmentOp::AddAssign),
        ("x -= 1", AssignmentOp::SubtractAssign),
        ("x *= 1", AssignmentOp::MultiplyAssign),
        ("x /= 1", AssignmentOp::DivideAssign),
        ("x %= 1", AssignmentOp::ModuloAssign),
        ("x &= 1", AssignmentOp::BitwiseAndAssign),
        ("x |= 1", AssignmentOp::BitwiseOrAssign),
        ("x ^= 1", AssignmentOp::BitwiseXorAssign),
        ("x <<= 1", AssignmentOp::ShiftLeftAssign),
        ("x >>= 1", AssignmentOp::ShiftRightAssign),
    ];
    for (input, expected_op) in cases {
        let (statements, errors) = parse(input);
        assert_no_errors(&errors);
        assert!(
            matches!(
                statements.declarations.as_slice(),
                [Stmt::Expression { expr }]
                    if matches!(
                        expr.as_ref(),
                        Expr::Assign { op, target, value, .. }
                            if *op == expected_op
                                && matches!(target.as_ref(), Expr::Variable { name, .. } if name == "x")
                                && matches!(value.as_ref(), Expr::Literal { .. })
                    )
            ),
            "unexpected AST for {input:?}: {statements:#?}"
        );
    }
}

#[test]
fn rejects_invalid_compound_assignment_targets() {
    for input in ["1 += 2", "x + y += 2"] {
        let (statements, errors) = parse(input);
        assert_has_error(&errors, ErrorCode::E1003);
        assert!(statements.is_empty(), "invalid compound assignment should not produce a statement: {statements:#?}");
    }
}

#[test]
fn parses_compound_assignment_to_array_element() {
    let (statements, errors) = parse("items[1] += 2");
    assert_no_errors(&errors);
    assert!(matches!(
        &statements[..],
        [Stmt::Expression { expr }]
            if matches!(
                expr.as_ref(),
                Expr::Assign {
                    op: AssignmentOp::AddAssign,
                    target,
                    ..
                } if matches!(target.as_ref(), Expr::ArrayAccess { .. })
            )
    ));
}

#[test]
fn parses_prefix_and_postfix_unary_expressions() {
    let input = "++value\nvalue--\n!ready\n~bits";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);

    assert!(
        matches!(&statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Increment, side: UnaryOpSide::Prefix, .. }))
    );
    assert!(
        matches!(&statements[1], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Decrement, side: UnaryOpSide::Postfix, .. }))
    );
    assert!(
        matches!(&statements[2], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Not, side: UnaryOpSide::Prefix, .. }))
    );
    assert!(
        matches!(&statements[3], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::BitwiseNot, side: UnaryOpSide::Prefix, .. }))
    );
}

#[test]
fn respects_binary_precedence_and_grouping() {
    let (precedence_statements, errors) = parse("a + b * c");
    assert_no_errors(&errors);
    assert_eq!(precedence_statements.len(), 1);
    assert!(
        matches!(&precedence_statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Add, right, .. } if matches!(right.as_ref(), Expr::Binary { op: BinaryOp::Multiply, .. })))
    );

    let (grouping_statements, errors) = parse("(a + b) * c");
    assert_no_errors(&errors);
    assert_eq!(grouping_statements.len(), 1);
    assert!(
        matches!(&grouping_statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Multiply, left, .. } if matches!(left.as_ref(), Expr::Grouping { .. })))
    );
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
            assert_eq!(*type_annotation, TypeSyntax::I32);
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
            assert_eq!(*type_annotation, TypeSyntax::I64);
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
            assert_eq!(parameters[0].type_annotation, TypeSyntax::I32);
            assert_eq!(parameters[1].name, "b");
            assert_eq!(parameters[1].type_annotation, TypeSyntax::I32);
            assert_eq!(*return_type, TypeSyntax::I32);
            assert!(
                matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Return { value: Some(_), .. }]))
            );
        }
        other => panic!("unexpected statement: {other:#?}"),
    }

    assert!(
        matches!(&statements[1], Stmt::MainFunction { body, .. } if matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Return { value: None, .. }])))
    );
}

#[test]
fn parses_array_vector_custom_and_unicode_types() {
    let input = "var matrix: i32[2][3] = {{1, 2, 3}, {4, 5, 6}}\nvar values: vector<string> = {\"a\", \"b\"}\nvar record: Record = value\nvar 變數: i64 = 1";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);

    match &statements[0] {
        Stmt::VarDeclaration { type_annotation, bindings, .. } => {
            assert!(
                matches!(type_annotation, TypeSyntax::Array { element_type, size, .. } if matches!(element_type.as_ref(), TypeSyntax::Array { element_type, .. } if matches!(element_type.as_ref(), TypeSyntax::I32)) && matches!(size.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. }))
            );
            assert!(bindings[0].initializer.is_some());
        }
        other => panic!("unexpected statement: {other:#?}"),
    }

    assert!(
        matches!(&statements[1], Stmt::VarDeclaration { type_annotation: TypeSyntax::Vector { element_type }, .. } if matches!(element_type.as_ref(), TypeSyntax::String))
    );
    assert!(
        matches!(&statements[2], Stmt::VarDeclaration { type_annotation: TypeSyntax::Custom { name }, .. } if name.as_ref() == "Record")
    );
    assert!(matches!(&statements[3], Stmt::VarDeclaration { bindings, .. } if bindings[0].name == "變數"));
}

#[test]
fn parses_if_else_while_and_for_control_flow() {
    let input = "if (ready) { value } else if (fallback) { other } else { final }\nwhile (ready) { break }\nfor (var i: i32 = 0; i < 10; i++) { continue }\nfor (;;) { break }";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);

    assert!(matches!(&statements[0], Stmt::If { else_branch: ElseBranch::ElseIf(_), .. }));
    assert!(
        matches!(&statements[1], Stmt::While { body, .. } if matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Break { .. }])))
    );
    assert!(matches!(&statements[2], Stmt::For { initializer: Some(_), condition: Some(_), increment: Some(_), .. }));
    assert!(
        matches!(&statements[3], Stmt::For { initializer: None, condition: None, increment: None, body, .. } if matches!(body.as_ref(), Stmt::Block { statements, .. } if matches!(statements.as_slice(), [Stmt::Break { .. }])))
    );
}

#[test]
fn rejects_invalid_assignment_target_without_panicking() {
    let (statements, errors) = parse("42 = 1");

    assert_has_error(&errors, ErrorCode::E1003);
    assert!(statements.is_empty(), "invalid assignment should not produce a statement: {statements:#?}");
}

#[test]
fn reports_missing_delimiters_as_syntax_errors() {
    let (_, errors) = parse("if (true) { value\nfun broken(a: i32 { return a");

    assert_has_error(&errors, ErrorCode::E1004);
}

#[test]
fn recovers_from_unexpected_tokens_and_keeps_following_statements() {
    let (statements, errors) = parse(") )\n42");

    assert_has_error(&errors, ErrorCode::E1004);
    assert_eq!(statements.len(), 1);
    assert!(
        matches!(&statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. }))
    );
}

#[test]
fn reports_maximum_recursion_depth_for_deep_grouping() {
    let input = format!("{}42{}", "(".repeat(1_005), ")".repeat(1_005));
    let handle = std::thread::Builder::new()
        .name("parser-recursion-limit".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || parse(&input))
        .expect("failed to spawn parser recursion test thread");
    let (_, errors) = handle.join().expect("parser recursion test thread panicked");

    assert_has_error(&errors, ErrorCode::E1001);
}

fn token_from(input: &str) -> descar_core::tokens::token::Token {
    let mut lexer = Lexer::new("precedence_test", input);

    lexer.next_token().expect("expected a token").expect("expected valid token")
}

#[test]
fn binary_operators_are_left_associative() {
    let cases = [
        ("a - b - c", BinaryOp::Subtract),
        ("a / b / c", BinaryOp::Divide),
        ("a % b % c", BinaryOp::Modulo),
        ("a << b << c", BinaryOp::ShiftLeft),
        ("a & b & c", BinaryOp::BitwiseAnd),
        ("a ^ b ^ c", BinaryOp::BitwiseXor),
        ("a | b | c", BinaryOp::BitwiseOr),
        ("a == b == c", BinaryOp::Equal),
        ("a && b && c", BinaryOp::And),
        ("a || b || c", BinaryOp::Or),
    ];

    for (input, expected_op) in cases {
        let (statements, errors) = parse(input);
        assert_no_errors(&errors);
        assert!(
            matches!(
                statements.declarations.as_slice(),
                [Stmt::Expression { expr }]
                    if matches!(
                        expr.as_ref(),
                        Expr::Binary { op, left, .. }
                            if *op == expected_op
                                && matches!(
                                    left.as_ref(),
                                    Expr::Binary { op: inner_op, .. } if *inner_op == expected_op
                                )
                    )
            ),
            "expected left-associated AST for {input:?}, got {statements:#?}"
        );
    }
}

#[test]
fn assignment_operators_are_right_associative() {
    for input in ["a = b = c", "a += b += c"] {
        let (statements, errors) = parse(input);
        assert_no_errors(&errors);
        assert!(
            matches!(
                statements.declarations.as_slice(),
                [Stmt::Expression { expr }]
                    if matches!(
                        expr.as_ref(),
                        Expr::Assign { value, .. }
                            if matches!(value.as_ref(), Expr::Assign { .. })
                    )
            ),
            "expected right-associated AST for {input:?}, got {statements:#?}"
        );
    }
}

#[test]
fn unary_binding_power_returns_zero_for_non_unary_operator() {
    let token = token_from("+");

    let precedence = Precedence::unary_binding_power(&token);

    assert_eq!(precedence.left, 0);
    assert_eq!(precedence.right, 0);
}

#[test]
fn unary_binding_power_returns_expected_value_for_unary_operator() {
    let token = token_from("!");

    let precedence = Precedence::unary_binding_power(&token);

    assert_eq!(precedence.left, 24);
    assert_eq!(precedence.right, 23);
}

#[test]
fn parses_explicit_numeric_casts_and_preserves_precedence() {
    let (statements, errors) = parse("value as i32\nleft + right as f64\nvalue as i32 as f64");
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 3);

    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::I32, expr: inner, .. }
                if matches!(inner.as_ref(), Expr::Variable { name, .. } if name == "value"))
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Add, right, .. }
                if matches!(right.as_ref(), Expr::Cast { target_type: TypeSyntax::F64, .. }))
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::F64, expr: inner, .. }
                if matches!(inner.as_ref(), Expr::Cast { target_type: TypeSyntax::I32, .. }))
    ));
}

#[test]
fn rejects_non_type_cast_targets() {
    let (_, errors) = parse("value as nullptr");
    assert_has_error(&errors, ErrorCode::E1002);
}

#[test]
fn rejects_missing_cast_target_and_no_statement() {
    let (statements, errors) = parse("value as");
    assert_has_error(&errors, ErrorCode::E1002);
    assert!(statements.is_empty(), "no statements should be produced when cast target missing");
}

#[test]
fn rejects_missing_second_cast_target() {
    let (statements, errors) = parse("value as i32 as");
    assert_has_error(&errors, ErrorCode::E1002);
    assert!(statements.is_empty(), "no statements should be produced when second cast target missing");
}

#[test]
fn parses_char_and_bool_cast_targets() {
    let (statements, errors) = parse("value as char\nvalue as bool");
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 2);
    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::Char, .. })
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::Bool, .. })
    ));
}

#[test]
fn parses_string_array_vector_and_custom_cast_targets() {
    let (statements, errors) = parse("value as string\nvalue as i32[4]\nvalue as vector<i32>\nvalue as Record");
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);
    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::String, .. })
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::Array { .. }, .. })
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::Vector { .. }, .. })
    ));
    assert!(matches!(
        &statements[3],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: TypeSyntax::Custom { name }, .. } if name.as_ref() == "Record")
    ));
}

#[test]
fn expression_spans_cover_all_operands_and_operators() {
    let (statements, errors) = parse("a + b * c");
    assert_no_errors(&errors);
    let Stmt::Expression { expr } = &statements[0] else { panic!("expected expression statement") };
    let Expr::Binary { left, right, span, .. } = expr.as_ref() else { panic!("expected outer binary expression") };

    // The test helper parses statement fragments inside "main { ... }".
    const WRAPPER_PREFIX: usize = "main { ".len();
    assert_eq!(span.start().offset(), WRAPPER_PREFIX);
    assert_eq!(span.end().offset(), WRAPPER_PREFIX + 9);
    assert_eq!(left.span().start().offset(), WRAPPER_PREFIX);
    assert_eq!(left.span().end().offset(), WRAPPER_PREFIX + 1);
    assert_eq!(right.span().start().offset(), WRAPPER_PREFIX + 4);
    assert_eq!(right.span().end().offset(), WRAPPER_PREFIX + 9);

    let (statements, errors) = parse("!ready");
    assert_no_errors(&errors);
    let Stmt::Expression { expr } = &statements[0] else { panic!("expected unary expression statement") };
    let Expr::Unary { expr: operand, span, .. } = expr.as_ref() else { panic!("expected prefix unary expression") };
    assert_eq!(span.start().offset(), WRAPPER_PREFIX);
    assert_eq!(span.end().offset(), WRAPPER_PREFIX + 6);
    assert_eq!(operand.span().start().offset(), WRAPPER_PREFIX + 1);
    assert_eq!(operand.span().end().offset(), WRAPPER_PREFIX + 6);
}

#[test]
fn variable_binding_spans_cover_names_and_initializers() {
    let input = "var first, second: i32 = 1, 2";
    let (statements, errors) = parse(input);
    assert_no_errors(&errors);
    let Stmt::VarDeclaration { bindings, .. } = &statements[0] else { panic!("expected variable declaration") };

    let first_start = input.find("first").unwrap();
    let first_end = input.find('1').unwrap() + 1;
    let second_start = input.find("second").unwrap();
    let second_end = input.rfind('2').unwrap() + 1;

    assert_eq!(bindings[0].span.start().offset(), first_start);
    assert_eq!(bindings[0].span.end().offset(), first_end);
    assert_eq!(bindings[1].span.start().offset(), second_start);
    assert_eq!(bindings[1].span.end().offset(), second_end);
}

#[test]
fn call_and_array_access_spans_include_their_base_expression() {
    let (statements, errors) = parse("print(items[1])");
    assert_no_errors(&errors);
    let Stmt::Expression { expr } = &statements[0] else { panic!("expected call expression statement") };
    let Expr::Call { callee, arguments, span } = expr.as_ref() else { panic!("expected call expression") };
    const WRAPPER_PREFIX: usize = "main { ".len();
    assert_eq!(span.start().offset(), WRAPPER_PREFIX);
    assert_eq!(span.end().offset(), WRAPPER_PREFIX + 15);
    assert_eq!(callee.span().start().offset(), WRAPPER_PREFIX);
    assert_eq!(callee.span().end().offset(), WRAPPER_PREFIX + 5);

    let Expr::ArrayAccess { array, span: access_span, .. } = &arguments[0] else {
        panic!("expected array access argument")
    };
    assert_eq!(access_span.start().offset(), WRAPPER_PREFIX + 6);
    assert_eq!(access_span.end().offset(), WRAPPER_PREFIX + 14);
    assert_eq!(array.span().start().offset(), WRAPPER_PREFIX + 6);
    assert_eq!(array.span().end().offset(), WRAPPER_PREFIX + 11);
}

#[test]
fn rejects_executable_statements_at_compilation_unit_level() {
    let (_, errors) = parse_unit("print(1)");
    assert_has_error(&errors, ErrorCode::E1004);
}

#[test]
fn rejects_nested_function_and_main_declarations() {
    for input in ["main { fun nested() {} }", "fun outer() { fun nested() {} }", "main { main {} }"] {
        let (_, errors) = parse(input);
        assert_has_error(&errors, ErrorCode::E1004);
    }
}

#[test]
fn rejects_duplicate_main_declarations() {
    let (_, errors) = parse("main {} main {}");
    assert_has_error(&errors, ErrorCode::E1004);
}

#[test]
fn accepts_global_variables_and_local_variables() {
    let (unit, errors) = parse("var global: i32 = 1\nmain { var local: i32 = 2 }");
    assert_no_errors(&errors);
    assert_eq!(unit.len(), 2);
}
