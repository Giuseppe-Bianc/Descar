use descar_core::{
    error::compile_error::CompileError,
    lex::lexer::{Lexer, lexer_tokenize_with_errors},
    syntax::{
        ast::{ast_type::Type, else_branch::ElseBranch, expr::Expr, stmt::Stmt},
        parser::JsavParser,
    },
};
use insta::assert_snapshot;

fn parse(input: &str) -> (Vec<Stmt>, Vec<CompileError>) {
    let mut lexer = Lexer::new("parser_snapshot", input);
    let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);

    if !lexer_errors.is_empty() {
        return (Vec::new(), lexer_errors);
    }

    JsavParser::new(&tokens).parse()
}

fn summarize_type(type_: &Type) -> String {
    match type_ {
        Type::Array { element_type, size } => {
            format!("Array(size={}, {})", summarize_expr(size), summarize_type(element_type))
        }
        Type::Vector { element_type } => format!("Vector({})", summarize_type(element_type)),
        Type::Custom { name } => format!("Custom({name})"),
        other => format!("{other:?}"),
    }
}

fn summarize_expr(expression: &Expr) -> String {
    match expression {
        Expr::Binary { left, op, right, .. } => {
            format!("Binary({op:?}, {}, {})", summarize_expr(left), summarize_expr(right))
        }
        Expr::Unary { op, side, expr, .. } => {
            format!("Unary({op:?}, {side:?}, {})", summarize_expr(expr))
        }
        Expr::Grouping { expr, .. } => format!("Grouping({})", summarize_expr(expr)),
        Expr::Literal { value, .. } => format!("Literal({value:?})"),
        Expr::ArrayLiteral { elements, .. } => {
            format!("ArrayLiteral([{}])", elements.iter().map(summarize_expr).collect::<Vec<_>>().join(", "))
        }
        Expr::Variable { name, .. } => format!("Variable({name})"),
        Expr::Assign { target, value, .. } => {
            format!("Assign({}, {})", summarize_expr(target), summarize_expr(value))
        }
        Expr::Call { callee, arguments, .. } => format!(
            "Call({}, [{}])",
            summarize_expr(callee),
            arguments.iter().map(summarize_expr).collect::<Vec<_>>().join(", ")
        ),
        Expr::ArrayAccess { array, index, .. } => {
            format!("ArrayAccess({}, {})", summarize_expr(array), summarize_expr(index))
        }
    }
}

fn summarize_statement(statement: &Stmt, indent: usize, lines: &mut Vec<String>) {
    let prefix = "  ".repeat(indent);
    match statement {
        Stmt::Expression { expr } => lines.push(format!("{prefix}Expression {}", summarize_expr(expr))),
        Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => {
            let kind = if *is_mutable { "Var" } else { "Const" };
            let names = bindings
                .iter()
                .map(|binding| {
                    let initializer = binding.initializer.as_ref().map_or_else(|| "none".to_string(), summarize_expr);
                    format!("{} = {initializer}", binding.name)
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("{prefix}{kind} [{names}] : {}", summarize_type(type_annotation)));
        }
        Stmt::Function { name, parameters, return_type, body, .. } => {
            let params = parameters
                .iter()
                .map(|parameter| format!("{}: {}", parameter.name, summarize_type(&parameter.type_annotation)))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("{prefix}Function {name}({params}) -> {}", summarize_type(return_type)));
            summarize_statement(body, indent + 1, lines);
        }
        Stmt::If { condition, then_branch, else_branch, .. } => {
            lines.push(format!("{prefix}If {}", summarize_expr(condition)));
            summarize_statement(then_branch, indent + 1, lines);
            match else_branch {
                ElseBranch::None => {}
                ElseBranch::Block(branch) => {
                    lines.push(format!("{prefix}else"));
                    summarize_statement(branch, indent + 1, lines);
                }
                ElseBranch::ElseIf(branch) => {
                    lines.push(format!("{prefix}else-if"));
                    summarize_statement(branch, indent + 1, lines);
                }
            }
        }
        Stmt::While { condition, body, .. } => {
            lines.push(format!("{prefix}While {}", summarize_expr(condition)));
            summarize_statement(body, indent + 1, lines);
        }
        Stmt::For { initializer, condition, increment, body, .. } => {
            lines.push(format!(
                "{prefix}For init={} condition={} increment={}",
                initializer.is_some(),
                condition.as_ref().map_or_else(|| "none".into(), summarize_expr),
                increment.as_ref().map_or_else(|| "none".into(), summarize_expr)
            ));
            if let Some(initializer) = initializer {
                summarize_statement(initializer, indent + 1, lines);
            }
            summarize_statement(body, indent + 1, lines);
        }
        Stmt::Block { statements, .. } => {
            lines.push(format!("{prefix}Block statements={}", statements.len()));
            for statement in statements {
                summarize_statement(statement, indent + 1, lines);
            }
        }
        Stmt::Return { value, .. } => {
            lines.push(format!("{prefix}Return {}", value.as_ref().map_or_else(|| "none".into(), summarize_expr)));
        }
        Stmt::Break { .. } => lines.push(format!("{prefix}Break")),
        Stmt::Continue { .. } => lines.push(format!("{prefix}Continue")),
        Stmt::MainFunction { body, .. } => {
            lines.push(format!("{prefix}MainFunction"));
            summarize_statement(body, indent + 1, lines);
        }
    }
}

fn summarize_errors(errors: &[CompileError]) -> String {
    errors
        .iter()
        .map(|error| format!("{:?}: {}", error.error_code(), error.message().unwrap_or("<no message>")))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn snapshots_complete_program() {
    let input = "fun add(a: i32, b: i32): i32 { return a + b }\nmain { var values: i32[2] = {1, 2}\n if (values[0] == 1) { print(values[0]) } else { print(\"x\") } }";
    let (statements, errors) = parse(input);
    let mut lines = Vec::new();
    for statement in &statements {
        summarize_statement(statement, 0, &mut lines);
    }
    if !errors.is_empty() {
        lines.push(format!("Errors:\n{}", summarize_errors(&errors)));
    }

    let rendered = lines.join("\n");
    assert_snapshot!("complete_program", rendered);
}

#[test]
fn snapshots_expression_forms() {
    let input = "x = -(a + b * 2)\nvalue++\n!ready\nsum(1, 2)[0]";
    let (statements, errors) = parse(input);
    let mut lines = Vec::new();
    for statement in &statements {
        summarize_statement(statement, 0, &mut lines);
    }
    if !errors.is_empty() {
        lines.push(format!("Errors:\n{}", summarize_errors(&errors)));
    }

    let rendered = lines.join("\n");
    assert_snapshot!("expression_forms", rendered);
}

#[test]
fn snapshots_type_forms() {
    let input = "var matrix: i32[2][3] = {{1, 2, 3}, {4, 5, 6}}\nvar values: vector<string> = {\"a\", \"b\"}\nvar record: Record = value\nconst flag: bool = true";
    let (statements, errors) = parse(input);
    let mut lines = Vec::new();
    for statement in &statements {
        summarize_statement(statement, 0, &mut lines);
    }
    if !errors.is_empty() {
        lines.push(format!("Errors:\n{}", summarize_errors(&errors)));
    }

    let rendered = lines.join("\n");
    assert_snapshot!("type_forms", rendered);
}

#[test]
fn snapshots_parser_errors() {
    let cases = [("invalid_assignment", "42 = 1"), ("missing_closing_brace", "if (true) {")];
    let mut lines = Vec::new();

    for (name, input) in cases {
        let (_, errors) = parse(input);
        lines.push(format!("{name}:"));
        for error in errors {
            lines.push(format!("{:?}: {}", error.error_code(), error.message().unwrap_or("<no message>")));
        }
    }

    let rendered = lines.join("\n");
    assert_snapshot!("parser_errors", rendered);
}
