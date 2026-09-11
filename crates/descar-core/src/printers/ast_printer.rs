use crate::printers::branch_type::{append_line, get_indent, print_children, BranchType, StyleManager};
use crate::syntax::ast::{ElseBranch, Expr, LiteralValue, Stmt, Type};

const EXPR_CAPACITY_PER_NODE: usize = 45;
const STMT_CAPACITY_PER_NODE: usize = 50;

fn format_literal(value: &LiteralValue) -> String {
    match value {
        LiteralValue::Numeric(number) => number.to_string(),
        LiteralValue::StringLit(value) => format!("\"{value}\""),
        LiteralValue::CharLit(value) => format!("'{value}'"),
        LiteralValue::Bool(value) => value.to_string(),
        LiteralValue::NullPtr => "nullptr".to_string(),
    }
}

fn format_type(type_annotation: &Type) -> String {
    match type_annotation {
        Type::I8 => "i8".to_string(),
        Type::I16 => "i16".to_string(),
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U8 => "u8".to_string(),
        Type::U16 => "u16".to_string(),
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F32 => "f32".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Char => "char".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Void => "void".to_string(),
        Type::NullPtr => "nullptr".to_string(),
        Type::Custom { name } => name.to_string(),
        Type::Array { element_type, size } => {
            format!("[{}; {}]", format_type(element_type), format_type_size(size))
        }
        Type::Vector { element_type } => format!("vector<{}>", format_type(element_type)),
    }
}

fn format_type_size(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value, .. } => format_literal(value),
        Expr::Variable { name, .. } => name.clone(),
        Expr::Grouping { expr, .. } => format!("({})", format_type_size(expr)),
        Expr::Binary { left, op, right, .. } => {
            format!("{} {} {}", format_type_size(left), format_binary_op(op), format_type_size(right))
        }
        Expr::Unary { op, expr, .. } => format!("{}{}", format_unary_op(op), format_type_size(expr)),
        Expr::Assign { target, value, .. } => {
            format!("{} = {}", format_type_size(target), format_type_size(value))
        }
        Expr::Call { callee, arguments, .. } => {
            let args = arguments.iter().map(format_type_size).collect::<Vec<_>>().join(", ");
            format!("{}({args})", format_type_size(callee))
        }
        Expr::ArrayAccess { array, index, .. } => {
            format!("{}[{}]", format_type_size(array), format_type_size(index))
        }
        Expr::ArrayLiteral { elements, .. } => {
            let elements = elements.iter().map(format_type_size).collect::<Vec<_>>().join(", ");
            format!("{{{elements}}}")
        }
    }
}

fn format_binary_op(op: &impl std::fmt::Debug) -> String {
    format!("{op:?}").to_uppercase()
}

fn format_unary_op(op: &impl std::fmt::Debug) -> String {
    format!("{op:?}").to_uppercase()
}

#[must_use]
pub fn pretty_print(expr: &Expr) -> String {
    let node_count = count_expr_nodes(expr);
    let mut output = String::with_capacity(node_count * EXPR_CAPACITY_PER_NODE);
    let styles = StyleManager::new();
    print_expr(expr, "", BranchType::Last, &mut output, &styles);
    output
}

fn count_expr_nodes(expr: &Expr) -> usize {
    let mut count = 0;
    let mut stack = vec![expr];

    while let Some(current) = stack.pop() {
        count += 1;
        match current {
            Expr::Binary { left, right, .. } => {
                stack.push(left);
                stack.push(right);
            }
            Expr::Unary { expr, .. } | Expr::Grouping { expr, .. } => {
                stack.push(expr);
            }
            Expr::Assign { target, value, .. } => {
                stack.push(target);
                stack.push(value);
            }
            Expr::Call { callee, arguments, .. } => {
                stack.push(callee);
                for arg in arguments {
                    stack.push(arg);
                }
            }
            Expr::ArrayAccess { array, index, .. } => {
                stack.push(array);
                stack.push(index);
            }
            Expr::ArrayLiteral { elements, .. } => {
                for element in elements {
                    stack.push(element);
                }
            }
            Expr::Literal { .. } | Expr::Variable { .. } => {}
        }
    }

    count
}

fn print_labeled_expr(
    label: &str,
    expr: &Expr,
    parent_indent: &str,
    current_branch: BranchType,
    output: &mut String,
    styles: &StyleManager,
) {
    append_line(output, parent_indent, current_branch, &styles.structure, label);
    print_expr(
        expr,
        &get_indent(parent_indent, &current_branch),
        BranchType::Last,
        output,
        styles,
    );
}

#[allow(clippy::too_many_lines)]
fn print_expr(
    expr: &Expr,
    indent: &str,
    branch_type: BranchType,
    output: &mut String,
    styles: &StyleManager,
) {
    match expr {
        Expr::Binary { left, op, right, .. } => {
            append_line(
                output,
                indent,
                branch_type,
                &styles.operator,
                &format!("BinaryOp {}", format_binary_op(op)),
            );
            print_labeled_expr("Left:", left, &get_indent(indent, &branch_type), BranchType::Middle, output, styles);
            print_labeled_expr("Right:", right, &get_indent(indent, &branch_type), BranchType::Last, output, styles);
        }
        Expr::Unary { op, expr, .. } => {
            append_line(
                output,
                indent,
                branch_type,
                &styles.operator,
                &format!("UnaryOp {}", format_unary_op(op)),
            );
            print_labeled_expr("Expr:", expr, &get_indent(indent, &branch_type), BranchType::Last, output, styles);
        }
        Expr::Grouping { expr, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation, "Grouping");
            print_labeled_expr("Expr:", expr, &get_indent(indent, &branch_type), BranchType::Last, output, styles);
        }
        Expr::Literal { value, .. } => {
            append_line(
                output,
                indent,
                branch_type,
                &styles.literal,
                &format!("Literal {}", format_literal(value)),
            );
        }
        Expr::Variable { name, .. } => {
            append_line(output, indent, branch_type, &styles.variable, &format!("Variable '{name}'"));
        }
        Expr::Assign { target, value, .. } => {
            append_line(output, indent, branch_type, &styles.variable, "Assignment");
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Target:", target, &new_indent, BranchType::Middle, output, styles);
            print_labeled_expr("Value:", value, &new_indent, BranchType::Last, output, styles);
        }
        Expr::Call { callee, arguments, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation, "Function Call");
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Callee:", callee, &new_indent, BranchType::Middle, output, styles);
            append_line(output, &new_indent, BranchType::Last, &styles.structure, "Arguments:");
            print_children(
                arguments,
                &get_indent(&new_indent, &BranchType::Last),
                output,
                styles,
                |argument, child_indent, child_branch, output, styles| {
                    append_line(output, child_indent, child_branch, &styles.structure, "Arg:");
                    print_expr(
                        argument,
                        &get_indent(child_indent, &child_branch),
                        BranchType::Last,
                        output,
                        styles,
                    );
                },
            );
        }
        Expr::ArrayAccess { array, index, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation, "Array Access");
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Array:", array, &new_indent, BranchType::Middle, output, styles);
            print_labeled_expr("Index:", index, &new_indent, BranchType::Last, output, styles);
        }
        Expr::ArrayLiteral { elements, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation, "Array Literal");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Last, &styles.structure, "Elements:");
            print_children(
                elements,
                &get_indent(&new_indent, &BranchType::Last),
                output,
                styles,
                print_expr,
            );
        }
    }
}

#[must_use]
pub fn pretty_print_stmt(stmt: &Stmt) -> String {
    let node_count = count_stmt_nodes(stmt);
    let mut output = String::with_capacity(node_count * STMT_CAPACITY_PER_NODE);
    let styles = StyleManager::new();
    print_stmt(stmt, "", BranchType::Last, &mut output, &styles);
    output
}

fn count_stmt_nodes(stmt: &Stmt) -> usize {
    1 + match stmt {
        Stmt::Expression { expr } => count_expr_nodes(expr),
        Stmt::VarDeclaration { bindings, .. } => {
            bindings.len()
                + bindings
                    .iter()
                    .filter_map(|binding| binding.initializer.as_ref())
                    .map(count_expr_nodes)
                    .sum::<usize>()
        }
        Stmt::Function { parameters, body, .. } => parameters.len() + count_stmt_nodes(body),
        Stmt::If { condition, then_branch, else_branch, .. } => {
            count_expr_nodes(condition)
                + count_stmt_nodes(then_branch)
                + match else_branch {
                    ElseBranch::None => 0,
                    ElseBranch::Block(branch) | ElseBranch::ElseIf(branch) => count_stmt_nodes(branch),
                }
        }
        Stmt::MainFunction { body, .. } => count_stmt_nodes(body),
        Stmt::Block { statements, .. } => statements.iter().map(count_stmt_nodes).sum::<usize>(),
        Stmt::Return { value, .. } => value.as_ref().map_or(0, count_expr_nodes),
        Stmt::While { condition, body, .. } => count_expr_nodes(condition) + count_stmt_nodes(body),
        Stmt::For { initializer, condition, increment, body, .. } => {
            initializer.as_ref().map_or(0, |value| count_stmt_nodes(value))
                + condition.as_ref().map_or(0, count_expr_nodes)
                + increment.as_ref().map_or(0, count_expr_nodes)
                + count_stmt_nodes(body)
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => 0,
    }
}

fn print_stmt_body(
    stmt: &Stmt,
    indent: &str,
    output: &mut String,
    styles: &StyleManager,
) {
    match stmt {
        Stmt::Block { statements, .. } => {
            if statements.is_empty() {
                append_line(output, indent, BranchType::Last, &styles.metadata, "(empty)");
            } else {
                print_children(statements, indent, output, styles, print_stmt);
            }
        }
        other => print_stmt(other, indent, BranchType::Last, output, styles),
    }
}

fn append_body_label(
    label: &str,
    body: &Stmt,
    parent_indent: &str,
    branch_type: BranchType,
    output: &mut String,
    styles: &StyleManager,
) {
    match body {
        Stmt::Block { statements, .. } if statements.is_empty() => {
            append_line(
                output,
                parent_indent,
                branch_type,
                &styles.structure,
                &format!("{label} (empty)"),
            );
        }
        _ => {
            append_line(output, parent_indent, branch_type, &styles.structure, label);
            print_stmt_body(
                body,
                &get_indent(parent_indent, &branch_type),
                output,
                styles,
            );
        }
    }
}

#[allow(clippy::too_many_lines)]
fn print_stmt(
    stmt: &Stmt,
    indent: &str,
    branch_type: BranchType,
    output: &mut String,
    styles: &StyleManager,
) {
    match stmt {
        Stmt::Expression { expr } => {
            append_line(output, indent, branch_type, &styles.keyword, "Expression");
            append_line(
                output,
                &get_indent(indent, &branch_type),
                BranchType::Last,
                &styles.structure,
                "Expr:",
            );
            print_expr(
                expr,
                &get_indent(&get_indent(indent, &branch_type), &BranchType::Last),
                BranchType::Last,
                output,
                styles,
            );
        }
        Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => {
            let declaration_kind = if *is_mutable { "VarDeclaration" } else { "ConstDeclaration" };
            append_line(output, indent, branch_type, &styles.keyword, declaration_kind);
            let new_indent = get_indent(indent, &branch_type);

            let variables_branch = BranchType::Middle;
            append_line(output, &new_indent, variables_branch, &styles.structure, "Variables:");
            let variables_indent = get_indent(&new_indent, &variables_branch);
            if bindings.is_empty() {
                append_line(output, &variables_indent, BranchType::Last, &styles.metadata, "(none)");
            } else {
                print_children(
                    bindings,
                    &variables_indent,
                    output,
                    styles,
                    |binding, child_indent, child_branch, output, styles| {
                        append_line(output, child_indent, child_branch, &styles.variable, &binding.name);
                    },
                );
            }

            let has_initializers = bindings.iter().any(|binding| binding.initializer.is_some());
            let type_branch = if has_initializers { BranchType::Middle } else { BranchType::Last };
            append_line(output, &new_indent, type_branch, &styles.structure, "Type:");
            append_line(
                output,
                &get_indent(&new_indent, &type_branch),
                BranchType::Last,
                &styles.type_style,
                &format_type(type_annotation),
            );

            if has_initializers {
                append_line(output, &new_indent, BranchType::Last, &styles.structure, "Initializers:");
                let initializers_indent = get_indent(&new_indent, &BranchType::Last);
                let initializers = bindings.iter().filter_map(|binding| binding.initializer.as_ref()).collect::<Vec<_>>();
                print_children(
                    &initializers,
                    &initializers_indent,
                    output,
                    styles,
                    |initializer, child_indent, child_branch, output, styles| {
                        print_expr(initializer, child_indent, child_branch, output, styles);
                    },
                );
            }
        }
        Stmt::Function { name, parameters, return_type, body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "Function");
            let new_indent = get_indent(indent, &branch_type);

            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Name:");
            append_line(
                output,
                &get_indent(&new_indent, &BranchType::Middle),
                BranchType::Last,
                &styles.variable,
                name,
            );

            let parameters_branch = if parameters.is_empty() { BranchType::Middle } else { BranchType::Middle };
            if parameters.is_empty() {
                append_line(output, &new_indent, parameters_branch, &styles.structure, "Parameters: (none)");
            } else {
                append_line(output, &new_indent, parameters_branch, &styles.structure, "Parameters:");
                let params_indent = get_indent(&new_indent, &parameters_branch);
                print_children(
                    parameters,
                    &params_indent,
                    output,
                    styles,
                    |parameter, child_indent, child_branch, output, styles| {
                        append_line(
                            output,
                            child_indent,
                            child_branch,
                            &styles.structure,
                            &format!("Parameter '{}'", parameter.name),
                        );
                        append_line(
                            output,
                            &get_indent(child_indent, &child_branch),
                            BranchType::Last,
                            &styles.type_style,
                            &format!("Type: {}", format_type(&parameter.type_annotation)),
                        );
                    },
                );
            }

            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Return Type:");
            append_line(
                output,
                &get_indent(&new_indent, &BranchType::Middle),
                BranchType::Last,
                &styles.type_style,
                &format_type(return_type),
            );

            append_body_label("Body:", body, &new_indent, BranchType::Last, output, styles);
        }
        Stmt::If { condition, then_branch, else_branch, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "If");
            let new_indent = get_indent(indent, &branch_type);

            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Condition:");
            print_expr(
                condition,
                &get_indent(&new_indent, &BranchType::Middle),
                BranchType::Last,
                output,
                styles,
            );

            let then_branch_type = if matches!(else_branch, ElseBranch::None) {
                BranchType::Last
            } else {
                BranchType::Middle
            };
            append_body_label(
                "Then:",
                then_branch,
                &new_indent,
                then_branch_type,
                output,
                styles,
            );

            if !matches!(else_branch, ElseBranch::None) {
                if let ElseBranch::Block(branch) | ElseBranch::ElseIf(branch) = else_branch {
                    append_body_label("Else:", branch, &new_indent, BranchType::Last, output, styles);
                }
            }
        }
        Stmt::MainFunction { body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "MainFunction");
            let new_indent = get_indent(indent, &branch_type);
            print_stmt_body(body, &new_indent, output, styles);
        }
        Stmt::Block { statements, .. } => {
            if statements.is_empty() {
                append_line(output, indent, branch_type, &styles.keyword, "Block: (empty)");
            } else {
                append_line(output, indent, branch_type, &styles.keyword, "Block");
                print_children(statements, &get_indent(indent, &branch_type), output, styles, print_stmt);
            }
        }
        Stmt::Return { value, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "Return");
            if let Some(expr) = value {
                append_line(
                    output,
                    &get_indent(indent, &branch_type),
                    BranchType::Last,
                    &styles.structure,
                    "Value:",
                );
                print_expr(
                    expr,
                    &get_indent(&get_indent(indent, &branch_type), &BranchType::Last),
                    BranchType::Last,
                    output,
                    styles,
                );
            }
        }
        Stmt::While { condition, body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "While");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Condition:");
            print_expr(
                condition,
                &get_indent(&new_indent, &BranchType::Middle),
                BranchType::Last,
                output,
                styles,
            );
            append_body_label("Body:", body, &new_indent, BranchType::Last, output, styles);
        }
        Stmt::For { initializer, condition, increment, body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "For");
            let new_indent = get_indent(indent, &branch_type);
            let mut parts = Vec::new();
            if initializer.is_some() {
                parts.push("initializer");
            }
            if condition.is_some() {
                parts.push("condition");
            }
            if increment.is_some() {
                parts.push("increment");
            }
            let has_body_after_header = !parts.is_empty();
            let mut emitted = 0usize;

            if let Some(init) = initializer {
                emitted += 1;
                let branch = if has_body_after_header || condition.is_some() || increment.is_some() { BranchType::Middle } else { BranchType::Last };
                append_line(output, &new_indent, branch, &styles.structure, "Initializer:");
                print_stmt(init, &get_indent(&new_indent, &branch), BranchType::Last, output, styles);
            }
            if let Some(cond) = condition {
                emitted += 1;
                let branch = if increment.is_some() { BranchType::Middle } else { BranchType::Last };
                append_line(output, &new_indent, branch, &styles.structure, "Condition:");
                print_expr(cond, &get_indent(&new_indent, &branch), BranchType::Last, output, styles);
            }
            if let Some(inc) = increment {
                emitted += 1;
                let branch = BranchType::Middle;
                append_line(output, &new_indent, branch, &styles.structure, "Increment:");
                print_expr(inc, &get_indent(&new_indent, &branch), BranchType::Last, output, styles);
            }
            let _ = emitted;
            append_body_label("Body:", body, &new_indent, BranchType::Last, output, styles);
        }
        Stmt::Break { .. } => append_line(output, indent, branch_type, &styles.keyword, "Break"),
        Stmt::Continue { .. } => append_line(output, indent, branch_type, &styles.keyword, "Continue"),
    }
}
