use descar_core::semantic::{FullyTypedAst, ResolvedType};
use descar_core::semantic::type_checker::TypeChecker;
use descar_core::syntax::ast::{BinaryOp, Expr, LiteralValue, Stmt, Type};
use descar_core::tokens::number::Number;

fn span() -> descar_core::location::source_span::SourceSpan {
    Default::default()
}

#[test]
fn typed_ast_preserves_expression_hierarchy_and_types_every_expression() {
    let s = span();
    let expr = Expr::Binary {
        left: Box::new(Expr::new_number_literal(Number::I32(1), s.clone())),
        op: BinaryOp::Add,
        right: Box::new(Expr::new_number_literal(Number::I64(2), s.clone())),
        span: s.clone(),
    };
    let statements = [Stmt::Expression { expr: Box::new(expr) }];

    let mut checker = TypeChecker::new();
    let typed = checker.check_typed(&statements).expect("program must type-check");

    let FullyTypedAst { statements } = typed;
    let Stmt::Expression { expr } = &statements[0] else { panic!("expected expression statement") };
    assert_eq!(expr.ty, ResolvedType::Value(Type::I64));

    match &expr.kind {
        descar_core::semantic::TypedExprKind::Binary { left, right, .. } => {
            assert_eq!(left.ty, ResolvedType::Value(Type::I32));
            assert_eq!(right.ty, ResolvedType::Value(Type::I64));
        }
        other => panic!("unexpected typed node: {other:?}"),
    }
}

#[test]
fn typed_ast_resolves_function_identifier_as_callable_signature() {
    let s = span();
    let function = Stmt::Function {
        name: "id".into(),
        parameters: vec![descar_core::syntax::ast::Parameter::new("x".into(), Type::I32, s.clone())],
        return_type: Type::I32,
        body: Box::new(Stmt::Block {
            statements: vec![Stmt::Return {
                value: Some(Expr::Variable { name: "x".into(), span: s.clone() }),
                span: s.clone(),
            }],
            span: s.clone(),
        }),
        span: s.clone(),
    };
    let call = Stmt::Expression {
        expr: Box::new(Expr::Call {
            callee: Box::new(Expr::Variable { name: "id".into(), span: s.clone() }),
            arguments: vec![Expr::new_number_literal(Number::I32(7), s.clone())],
            span: s.clone(),
        }),
    };

    let mut checker = TypeChecker::new();
    let typed = checker.check_typed(&[function, call]).expect("program must type-check");
    let Stmt::Expression { expr } = &typed.statements[1] else { panic!("expected call statement") };

    assert_eq!(expr.ty, ResolvedType::Value(Type::I32));
    match &expr.kind {
        descar_core::semantic::TypedExprKind::Call { callee, arguments } => {
            assert_eq!(
                callee.ty,
                ResolvedType::Function { parameters: vec![Type::I32], return_type: Type::I32 }
            );
            assert_eq!(arguments[0].ty, ResolvedType::Value(Type::I32));
        }
        other => panic!("unexpected typed node: {other:?}"),
    }
}

#[test]
fn failed_type_check_does_not_produce_a_partial_typed_ast() {
    let s = span();
    let statements = [Stmt::Expression {
        expr: Box::new(Expr::Binary {
            left: Box::new(Expr::Literal {
                value: LiteralValue::StringLit("bad".into()),
                span: s.clone(),
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr::new_number_literal(Number::I32(1), s.clone())),
            span: s.clone(),
        }),
    }];

    let mut checker = TypeChecker::new();
    let result = checker.check_typed(&statements);
    assert!(result.is_err(), "invalid typing must not produce a typed AST");
}
