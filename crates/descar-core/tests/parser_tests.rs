use descar_core::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    lex::lexer::{Lexer, lexer_tokenize_with_errors},
    syntax::{
        ast::{
            ast_type::Type, binary_op::BinaryOp, else_branch::ElseBranch, expr::Expr, literal_value::LiteralValue,
            stmt::Stmt, unary_op::UnaryOp, unary_op_side::UnaryOpSide,
        },
        parser::JsavParser,
        precendence::Precedence,
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

    assert_eq!(statements, [] as [Stmt; 0]);
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
        matches!(&statements[0], Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Assign { target, value, .. } if matches!(target.as_ref(), Expr::Variable { name, .. } if name == "items") && matches!(value.as_ref(), Expr::ArrayLiteral { elements, .. } if elements.len() == 3)))
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
        ("x += 1", BinaryOp::AddEqual),
        ("x -= 1", BinaryOp::SubtractEqual),
        ("x *= 1", BinaryOp::MultiplyEqual),
        ("x /= 1", BinaryOp::DivideEqual),
        ("x %= 1", BinaryOp::ModuloEqual),
        ("x &= 1", BinaryOp::BitwiseAndEqual),
        ("x |= 1", BinaryOp::BitwiseOrEqual),
        ("x ^= 1", BinaryOp::BitwiseXorEqual),
        ("x <<= 1", BinaryOp::ShiftLeftEqual),
        ("x >>= 1", BinaryOp::ShiftRightEqual),
    ];

    for (input, expected_op) in cases {
        let (statements, errors) = parse(input);
        assert_no_errors(&errors);
        assert!(
            matches!(
                statements.as_slice(),
                [Stmt::Expression { expr }]
                    if matches!(
                        expr.as_ref(),
                        Expr::Binary { op, left, right, .. }
                            if *op == expected_op
                                && matches!(left.as_ref(), Expr::Variable { name, .. } if name == "x")
                                && matches!(right.as_ref(), Expr::Literal { .. })
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
                Expr::Binary {
                    op: BinaryOp::AddEqual,
                    left,
                    ..
                } if matches!(left.as_ref(), Expr::ArrayAccess { .. })
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
                matches!(type_annotation, Type::Array { element_type, size, .. } if matches!(element_type.as_ref(), Type::Array { element_type, .. } if matches!(element_type.as_ref(), Type::I32)) && matches!(size.as_ref(), Expr::Literal { value: LiteralValue::Numeric(_), .. }))
            );
            assert!(bindings[0].initializer.is_some());
        }
        other => panic!("unexpected statement: {other:#?}"),
    }

    assert!(
        matches!(&statements[1], Stmt::VarDeclaration { type_annotation: Type::Vector { element_type }, .. } if matches!(element_type.as_ref(), Type::String))
    );
    assert!(
        matches!(&statements[2], Stmt::VarDeclaration { type_annotation: Type::Custom { name }, .. } if name.as_ref() == "Record")
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
            if matches!(expr.as_ref(), Expr::Cast { target_type: Type::I32, expr: inner, .. }
                if matches!(inner.as_ref(), Expr::Variable { name, .. } if name == "value"))
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Binary { op: BinaryOp::Add, right, .. }
                if matches!(right.as_ref(), Expr::Cast { target_type: Type::F64, .. }))
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: Type::F64, expr: inner, .. }
                if matches!(inner.as_ref(), Expr::Cast { target_type: Type::I32, .. }))
    ));
}

/// Renders an expression with one pair of parentheses around every binary, assignment, prefix, postfix, and cast
/// node, so a test can compare the exact tree shape with a string.
fn shape(expr: &Expr) -> String {
    match expr {
        Expr::Binary { left, op, right, .. } => {
            let symbol = match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Modulo => "%",
                BinaryOp::Less => "<",
                BinaryOp::AddEqual => "+=",
                other => panic!("shape() has no symbol for {other:?}"),
            };
            format!("({} {symbol} {})", shape(left), shape(right))
        }
        Expr::Assign { target, value, .. } => format!("({} = {})", shape(target), shape(value)),
        Expr::Cast { expr, target_type, .. } => format!("({} as {target_type})", shape(expr)),
        Expr::Unary { op, side, expr, .. } => {
            let symbol = match op {
                UnaryOp::Negate => "-",
                UnaryOp::Not => "!",
                UnaryOp::BitwiseNot => "~",
                UnaryOp::Increment => "++",
                UnaryOp::Decrement => "--",
            };
            match side {
                UnaryOpSide::Prefix => format!("({symbol}{})", shape(expr)),
                UnaryOpSide::Postfix => format!("({}{symbol})", shape(expr)),
            }
        }
        Expr::Call { callee, arguments, .. } => {
            let arguments: Vec<String> = arguments.iter().map(shape).collect();
            format!("{}({})", shape(callee), arguments.join(", "))
        }
        Expr::ArrayAccess { array, index, .. } => format!("{}[{}]", shape(array), shape(index)),
        Expr::Variable { name, .. } => name.clone(),
        other => panic!("shape() does not render {other:?}"),
    }
}

/// Source and required shape pairs for SYN-004. The vectors match Table SYN-004 in the specification.
const CAST_SHAPE_VECTORS: &[(&str, &str)] = &[
    ("a + b as f64", "(a + (b as f64))"),
    ("a - b as f64", "(a - (b as f64))"),
    ("a * b as f64", "(a * (b as f64))"),
    ("a / b as f64", "(a / (b as f64))"),
    ("a % b as i32", "(a % (b as i32))"),
    ("a as i32 * b", "((a as i32) * b)"),
    ("a as i32 * b as i32", "((a as i32) * (b as i32))"),
    ("a as i32 + b", "((a as i32) + b)"),
    ("a as i32 < b", "((a as i32) < b)"),
    ("-a as f64", "((-a) as f64)"),
    ("~a as u8", "((~a) as u8)"),
    ("-a * b as f64", "((-a) * (b as f64))"),
    ("-f(x)", "(-f(x))"),
    ("-a[i] as i32", "((-a[i]) as i32)"),
    ("f(x) as i32", "(f(x) as i32)"),
    ("a[i] as i32", "(a[i] as i32)"),
    ("a++ as i32", "((a++) as i32)"),
    ("a as i32 as f64", "((a as i32) as f64)"),
    ("x = y as i32", "(x = (y as i32))"),
    ("x += y as f64", "(x += (y as f64))"),
];

#[test]
fn cast_shapes_follow_the_syn_004_vectors() {
    for (source, expected) in CAST_SHAPE_VECTORS {
        let (statements, errors) = parse(source);
        assert_no_errors(&errors);
        assert_eq!(statements.len(), 1, "expected one statement for `{source}`");
        let Stmt::Expression { expr } = &statements[0] else {
            panic!("expected an expression statement for `{source}`");
        };
        assert_eq!(&shape(expr), expected, "wrong shape for `{source}`");
    }
}

#[test]
fn cast_and_prefix_powers_satisfy_the_syn_004_ordering_constraint() {
    // M < C <= P < K, where M is the multiplicative right power, C the cast left power,
    // P the prefix right power, and K the call, index, and postfix-update left power.
    let cast_left = Precedence::binding_power(&token_from("as")).left;
    let prefix_right = Precedence::unary_binding_power(&token_from("-")).right;
    let call_left = Precedence::binding_power(&token_from("(")).left;

    for operator in ["*", "/", "%"] {
        let multiplicative_right = Precedence::binding_power(&token_from(operator)).right;
        assert!(
            multiplicative_right < cast_left,
            "cast left power {cast_left} must exceed the right power {multiplicative_right} of `{operator}`"
        );
    }
    assert!(cast_left <= prefix_right, "cast left power {cast_left} must not exceed prefix right power {prefix_right}");
    assert!(prefix_right < call_left, "prefix right power {prefix_right} must be below call left power {call_left}");
}

#[test]
fn rejects_literal_cast_target() {
    let (statements, errors) = parse("value as 5");
    assert_has_error(&errors, ErrorCode::E1002);
    assert!(statements.is_empty(), "no statements should be produced when the cast target is a literal");
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
        Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Cast { target_type: Type::Char, .. })
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Cast { target_type: Type::Bool, .. })
    ));
}

#[test]
fn parses_string_array_vector_and_custom_cast_targets() {
    let (statements, errors) = parse("value as string\nvalue as i32[4]\nvalue as vector<i32>\nvalue as Record");
    assert_no_errors(&errors);
    assert_eq!(statements.len(), 4);
    assert!(matches!(
        &statements[0],
        Stmt::Expression { expr } if matches!(expr.as_ref(), Expr::Cast { target_type: Type::String, .. })
    ));
    assert!(matches!(
        &statements[1],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: Type::Array { .. }, .. })
    ));
    assert!(matches!(
        &statements[2],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: Type::Vector { .. }, .. })
    ));
    assert!(matches!(
        &statements[3],
        Stmt::Expression { expr }
            if matches!(expr.as_ref(), Expr::Cast { target_type: Type::Custom { name }, .. } if name.as_ref() == "Record")
    ));
}
