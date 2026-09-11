use std::sync::Arc;

use descar_core::{
    location::{source_location::SourceLocation, source_span::SourceSpan},
    printers::ast_printer::{pretty_print, pretty_print_stmt},
    syntax::ast::{
        ast_type::Type,
        binary_op::BinaryOp,
        else_branch::ElseBranch,
        expr::Expr,
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

fn bool_literal(value: bool) -> Expr {
    Expr::new_bool_literal(value, span(0, 1))
}

fn statement_block(statements: Vec<Stmt>) -> Stmt {
    Stmt::Block { statements, span: span(0, 1) }
}

fn rendered_expr(expr: &Expr) -> String {
    strip_ansi_codes(&pretty_print(expr))
}

fn rendered_stmt(stmt: &Stmt) -> String {
    strip_ansi_codes(&pretty_print_stmt(stmt))
}

#[test]
fn pretty_print_literals_and_variable_values() {
    let cases = [
        (Expr::new_number_literal(Number::Integer(-42), span(0, 3)), "└── Literal -42\n"),
        (Expr::new_bool_literal(true, span(0, 4)), "└── Literal true\n"),
        (Expr::new_string_literal("hello".into(), span(0, 7)), "└── Literal \"hello\"\n"),
        (Expr::new_char_literal("é".into(), span(0, 3)), "└── Literal 'é'\n"),
        (Expr::new_nullptr_literal(span(0, 7)), "└── Literal nullptr\n"),
        (variable("counter"), "└── Variable 'counter'\n"),
    ];

    for (expression, expected) in cases {
        assert_eq!(rendered_expr(&expression), expected);
    }
}

#[test]
fn pretty_print_binary_expression_covers_every_operator() {
    let operators = [
        (BinaryOp::Add, "ADD"),
        (BinaryOp::AddEqual, "ADDEQUAL"),
        (BinaryOp::Subtract, "SUBTRACT"),
        (BinaryOp::SubtractEqual, "SUBTRACTEQUAL"),
        (BinaryOp::Multiply, "MULTIPLY"),
        (BinaryOp::MultiplyEqual, "MULTIPLYEQUAL"),
        (BinaryOp::Divide, "DIVIDE"),
        (BinaryOp::DivideEqual, "DIVIDEEQUAL"),
        (BinaryOp::Modulo, "MODULO"),
        (BinaryOp::ModuloEqual, "MODULOEQUAL"),
        (BinaryOp::Equal, "EQUAL"),
        (BinaryOp::NotEqual, "NOTEQUAL"),
        (BinaryOp::Less, "LESS"),
        (BinaryOp::LessEqual, "LESSEQUAL"),
        (BinaryOp::Greater, "GREATER"),
        (BinaryOp::GreaterEqual, "GREATEREQUAL"),
        (BinaryOp::And, "AND"),
        (BinaryOp::Or, "OR"),
        (BinaryOp::BitwiseAnd, "BITWISEAND"),
        (BinaryOp::BitwiseAndEqual, "BITWISEANDEQUAL"),
        (BinaryOp::BitwiseOr, "BITWISEOR"),
        (BinaryOp::BitwiseOrEqual, "BITWISEOREQUAL"),
        (BinaryOp::BitwiseXor, "BITWISEXOR"),
        (BinaryOp::BitwiseXorEqual, "BITWISEXOREQUAL"),
        (BinaryOp::ShiftLeft, "SHIFTLEFT"),
        (BinaryOp::ShiftLeftEqual, "SHIFTLEFTEQUAL"),
        (BinaryOp::ShiftRight, "SHIFTRIGHT"),
        (BinaryOp::ShiftRightEqual, "SHIFTRIGHTEQUAL"),
    ];

    for (operator, expected) in operators {
        let expression = Expr::Binary {
            left: Box::new(variable("left")),
            op: operator,
            right: Box::new(variable("right")),
            span: span(0, 5),
        };
        let output = rendered_expr(&expression);

        assert!(output.starts_with(&format!("└── BinaryOp {expected}\n")));
        assert!(output.contains("├── Left:\n"));
        assert!(output.contains("└── Right:\n"));
    }
}

#[test]
fn pretty_print_unary_expression_covers_all_operators_and_sides() {
    let operators = [UnaryOp::Negate, UnaryOp::Not, UnaryOp::BitwiseNot, UnaryOp::Increment, UnaryOp::Decrement];

    for operator in operators {
        for (side, label) in [(UnaryOpSide::Prefix, "PREFIX"), (UnaryOpSide::Postfix, "POSTFIX")] {
            let expression = Expr::Unary { op: operator, side, expr: Box::new(variable("value")), span: span(0, 3) };
            let output = rendered_expr(&expression);

            assert!(output.contains(&format!("UnaryOp {operator:?} ({label})")));
            assert!(output.contains("Expr:"));
            assert!(output.contains("Variable 'value'"));
        }
    }
}

#[test]
fn pretty_print_nested_expression_structures_keep_labels_and_order() {
    let expression = Expr::Assign {
        target: Box::new(Expr::ArrayAccess {
            array: Box::new(Expr::Grouping { expr: Box::new(variable("items")), span: span(0, 7) }),
            index: Box::new(Expr::Binary {
                left: Box::new(number(1)),
                op: BinaryOp::Add,
                right: Box::new(number(2)),
                span: span(0, 5),
            }),
            span: span(0, 8),
        }),
        value: Box::new(Expr::Call {
            callee: Box::new(variable("build")),
            arguments: vec![number(10), bool_literal(false)],
            span: span(0, 10),
        }),
        span: span(0, 20),
    };
    let output = rendered_expr(&expression);

    let target = output.find("Target:").unwrap();
    let value = output.find("Value:").unwrap();
    let array = output.find("Array Access").unwrap();
    let call = output.find("Function Call").unwrap();

    assert!(target < value);
    assert!(array < value);
    assert!(call > value);
    assert!(output.contains("Grouping"));
    assert!(output.contains("BinaryOp ADD"));
    assert!(output.contains("Callee:"));
    assert!(output.contains("Arguments:"));
    assert!(output.contains("Arg:"));
}

#[test]
fn pretty_print_collection_expressions_handle_empty_and_non_empty_cases() {
    let empty_call = Expr::Call { callee: Box::new(variable("f")), arguments: vec![], span: span(0, 3) };
    let empty_array = Expr::ArrayLiteral { elements: vec![], span: span(0, 2) };
    let populated_call = Expr::Call {
        callee: Box::new(variable("f")),
        arguments: vec![number(1), number(2), variable("x")],
        span: span(0, 8),
    };
    let populated_array = Expr::ArrayLiteral {
        elements: vec![number(1), Expr::new_string_literal("x".into(), span(0, 3)), bool_literal(true)],
        span: span(0, 10),
    };

    let empty_call_output = rendered_expr(&empty_call);
    let empty_array_output = rendered_expr(&empty_array);
    let populated_call_output = rendered_expr(&populated_call);
    let populated_array_output = rendered_expr(&populated_array);

    assert!(empty_call_output.ends_with("├── Callee:\n    │   └── Variable 'f'\n    └── Arguments:\n"));
    assert!(empty_array_output.ends_with("└── Array Literal\n    └── Elements:\n"));
    assert_eq!(populated_call_output.matches("Arg:").count(), 3);
    assert_eq!(populated_array_output.matches("Literal ").count(), 3);
    assert_eq!(populated_array_output.matches("Variable").count(), 0);
}

#[test]
fn pretty_print_declarations_cover_mutability_bindings_initializers_and_types() {
    let declarations = [
        (
            Stmt::VarDeclaration {
                bindings: vec![
                    VarBinding { name: "first".into(), initializer: Some(number(1)) },
                    VarBinding { name: "second".into(), initializer: None },
                ],
                type_annotation: Type::I32,
                is_mutable: true,
                span: span(0, 12),
            },
            ["VarDeclaration", "first", "second", "Type:", "i32", "Initializers:", "Literal 1"],
        ),
        (
            Stmt::VarDeclaration {
                bindings: vec![],
                type_annotation: Type::Custom { name: Arc::from("Widget") },
                is_mutable: false,
                span: span(0, 1),
            },
            ["ConstDeclaration", "Variables:", "(none)", "Type:", "Widget"],
        ),
    ];

    for (statement, expected_fragments) in declarations {
        let output = rendered_stmt(&statement);
        for fragment in expected_fragments {
            assert!(output.contains(fragment), "missing {fragment:?} in {output:?}");
        }
    }
}

#[test]
fn pretty_print_type_annotations_cover_all_type_variants() {
    let cases = [
        (Type::I8, "i8"),
        (Type::I16, "i16"),
        (Type::I32, "i32"),
        (Type::I64, "i64"),
        (Type::U8, "u8"),
        (Type::U16, "u16"),
        (Type::U32, "u32"),
        (Type::U64, "u64"),
        (Type::F32, "f32"),
        (Type::F64, "f64"),
        (Type::Char, "char"),
        (Type::String, "string"),
        (Type::Bool, "bool"),
        (Type::Void, "void"),
        (Type::NullPtr, "nullptr"),
        (Type::Custom { name: Arc::from("UserType") }, "UserType"),
        (Type::Array { element_type: Box::new(Type::U32), size: Box::new(number(8)) }, "[u32; 8]"),
        (Type::Vector { element_type: Box::new(Type::String) }, "vector<string>"),
    ];

    for (type_annotation, expected) in cases {
        let statement = Stmt::VarDeclaration { bindings: vec![], type_annotation, is_mutable: false, span: span(0, 1) };
        let output = rendered_stmt(&statement);

        assert!(output.contains(&format!("Type:\n    └── {expected}")), "missing {expected:?} in {output:?}");
    }
}

#[test]
fn pretty_print_functions_cover_empty_and_populated_parameters_and_bodies() {
    let parameter_a = Parameter::new("left".into(), Type::I32, span(0, 1));
    let parameter_b = Parameter::new("right".into(), Type::Vector { element_type: Box::new(Type::Bool) }, span(0, 1));
    let empty_function = Stmt::Function {
        name: "empty".into(),
        parameters: vec![],
        return_type: Type::Void,
        body: Box::new(statement_block(vec![])),
        span: span(0, 1),
    };
    let populated_function = Stmt::Function {
        name: "compute".into(),
        parameters: vec![parameter_a, parameter_b],
        return_type: Type::F64,
        body: Box::new(statement_block(vec![Stmt::Return { value: Some(number(42)), span: span(0, 1) }])),
        span: span(0, 10),
    };

    let empty_output = rendered_stmt(&empty_function);
    let populated_output = rendered_stmt(&populated_function);

    assert!(empty_output.contains("Parameters: (none)"));
    assert!(empty_output.contains("Body: (empty)"));
    assert!(populated_output.contains("Parameter 'left'"));
    assert!(populated_output.contains("Type: i32"));
    assert!(populated_output.contains("Parameter 'right'"));
    assert!(populated_output.contains("Type: vector<bool>"));
    assert!(populated_output.contains("Return"));
    assert!(populated_output.contains("Literal 42"));
}

#[test]
fn pretty_print_if_covers_none_block_and_else_if_branches() {
    let base_then = statement_block(vec![Stmt::Break { span: span(0, 1) }]);
    let cases = [
        (ElseBranch::None, "If\n"),
        (ElseBranch::Block(Box::new(statement_block(vec![Stmt::Continue { span: span(0, 1) }]))), "Else:"),
        (ElseBranch::ElseIf(Box::new(Stmt::Break { span: span(0, 1) })), "Else:\n        └── Break"),
    ];

    for (else_branch, expected) in cases {
        let statement = Stmt::If {
            condition: Box::new(bool_literal(true)),
            then_branch: Box::new(base_then.clone()),
            else_branch,
            span: span(0, 10),
        };
        let output = rendered_stmt(&statement);

        assert!(output.contains(expected), "missing {expected:?} in {output:?}");
        assert!(output.contains("Then:"));
        assert!(output.contains("Break"));
    }
}

#[test]
fn pretty_print_control_flow_cover_while_for_and_all_for_option_combinations() {
    let while_statement = Stmt::While {
        condition: Box::new(bool_literal(true)),
        body: Box::new(statement_block(vec![Stmt::Continue { span: span(0, 1) }])),
        span: span(0, 8),
    };
    let while_output = rendered_stmt(&while_statement);
    assert!(while_output.contains("While"));
    assert!(while_output.contains("Condition:"));
    assert!(while_output.contains("Body:"));
    assert!(while_output.contains("Continue"));

    for (has_initializer, has_condition, has_increment) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (true, true, true),
    ] {
        let initializer = has_initializer.then(|| {
            Box::new(Stmt::VarDeclaration {
                bindings: vec![VarBinding { name: "i".into(), initializer: Some(number(0)) }],
                type_annotation: Type::I32,
                is_mutable: true,
                span: span(0, 1),
            })
        });
        let condition = has_condition.then(|| bool_literal(true));
        let increment = has_increment.then(|| Expr::Unary {
            op: UnaryOp::Increment,
            side: UnaryOpSide::Postfix,
            expr: Box::new(variable("i")),
            span: span(0, 2),
        });
        let statement =
            Stmt::For { initializer, condition, increment, body: Box::new(statement_block(vec![])), span: span(0, 10) };
        let output = rendered_stmt(&statement);

        assert!(output.contains("For"));
        assert_eq!(output.contains("Initializer:"), has_initializer);
        assert_eq!(output.contains("Condition:"), has_condition);
        assert_eq!(output.contains("Increment:"), has_increment);
        assert!(output.contains("Body: (empty)"));
    }
}

#[test]
fn pretty_print_blocks_and_simple_statements_cover_empty_and_nested_cases() {
    let block = statement_block(vec![
        Stmt::Expression { expr: Box::new(number(1)) },
        Stmt::Return { value: Some(number(2)), span: span(0, 1) },
        Stmt::Break { span: span(0, 1) },
        Stmt::Continue { span: span(0, 1) },
    ]);
    let output = rendered_stmt(&block);

    assert!(output.contains("Block"));
    assert!(output.contains("Expression"));
    assert!(output.contains("Return"));
    assert!(output.contains("Break"));
    assert!(output.contains("Continue"));
    assert!(output.contains("Literal 1"));
    assert!(output.contains("Literal 2"));

    let empty_output = rendered_stmt(&statement_block(vec![]));
    assert_eq!(empty_output, "└── Block: (empty)\n");

    let return_without_value = rendered_stmt(&Stmt::Return { value: None, span: span(0, 1) });
    let return_with_value = rendered_stmt(&Stmt::Return { value: Some(number(7)), span: span(0, 1) });
    assert_eq!(return_without_value, "└── Return\n");
    assert!(return_with_value.contains("Value:"));
    assert!(return_with_value.contains("Literal 7"));

    assert_eq!(rendered_stmt(&Stmt::Break { span: span(0, 1) }), "└── Break\n");
    assert_eq!(rendered_stmt(&Stmt::Continue { span: span(0, 1) }), "└── Continue\n");
}

#[test]
fn pretty_print_expression_and_main_function_statements_render_expected_wrappers() {
    let expression_statement = Stmt::Expression { expr: Box::new(number(9)) };
    let main_function = Stmt::MainFunction {
        body: Box::new(statement_block(vec![Stmt::Expression { expr: Box::new(bool_literal(false)) }])),
        span: span(0, 5),
    };

    let expression_output = rendered_stmt(&expression_statement);
    let main_output = rendered_stmt(&main_function);

    assert_eq!(expression_output, "└── Expression\n    └── Expr:\n        └── Literal 9\n");
    assert!(main_output.starts_with("└── MainFunction\n"));
    assert!(main_output.contains("Expression"));
    assert!(main_output.contains("Literal false"));
}

#[test]
fn pretty_print_handles_deep_expression_nesting_and_preserves_every_node() {
    let mut expression = number(1);

    for _ in 0..64 {
        expression = Expr::Grouping { expr: Box::new(expression), span: span(0, 3) };
    }

    let output = rendered_expr(&expression);

    assert_eq!(output.matches("Grouping").count(), 64);
    assert_eq!(output.matches("Literal 1").count(), 1);
}

#[test]
fn pretty_print_output_is_deterministic_and_ansi_stripping_is_safe() {
    let expression = Expr::Binary {
        left: Box::new(number(1)),
        op: BinaryOp::Multiply,
        right: Box::new(Expr::Grouping { expr: Box::new(number(2)), span: span(0, 3) }),
        span: span(0, 5),
    };

    let first = pretty_print(&expression);
    let second = pretty_print(&expression);

    assert_eq!(first, second);
    assert_eq!(strip_ansi_codes(&first), rendered_expr(&expression));
    assert!(first.ends_with('\n'));
}
