use crate::printers::branch_type::{BranchConfig, BranchType, StyleManager, append_line, get_indent, print_children};
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
        Type::Array { element_type, .. } => format!("{}[]", format_type(element_type)),
        Type::Vector { element_type } => format!("vector<{}>", format_type(element_type)),
    }
}

/// Pretty-print an expression AST into a styled, tree-like string.
/// Optimized with capacity preallocation.
#[must_use]
pub fn pretty_print(expr: &Expr) -> String {
    let node_count = count_expr_nodes(expr);
    // Estimate ~45 chars per node (branch chars + label + styling)
    let mut output = String::with_capacity(node_count * EXPR_CAPACITY_PER_NODE);
    let styles = StyleManager::new();
    print_expr(expr, "", BranchType::Last, &mut output, &styles);
    output
}

/// Counts total nodes in an expression tree for capacity estimation.
///
/// Performs a recursive traversal to compute the total number of nodes,
/// which is used to preallocate string capacity in [`pretty_print`].
///
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
                for elem in elements {
                    stack.push(elem);
                }
            }
            Expr::Literal { .. } | Expr::Variable { .. } => {}
        }
    }

    count
}

/// Unified function to print labeled branches
fn print_branch(
    label: &str, expr: &Expr, parent_indent: &str, branch_config: &BranchConfig, output: &mut String,
    styles: &StyleManager,
) {
    let indent = get_indent(parent_indent, &branch_config.parent_type);
    append_line(output, &indent, branch_config.current_type, &styles.structure.clone(), label);
    print_expr(expr, &get_indent(&indent, &branch_config.current_type), branch_config.child_type, output, styles);
}

/// Prints an expression with the given indentation and branch type.
#[allow(clippy::too_many_lines)]
fn print_expr(expr: &Expr, indent: &str, branch_type: BranchType, output: &mut String, styles: &StyleManager) {
    match expr {
        Expr::Binary { left, op, right, .. } => {
            append_line(output, indent, branch_type, &styles.operator.clone(), &format!("BinaryOp {op:?}"));
            print_branch(
                "Left:",
                left,
                indent,
                &BranchConfig::new(branch_type, BranchType::Middle, BranchType::Last),
                output,
                styles,
            );
            print_branch(
                "Right:",
                right,
                indent,
                &BranchConfig::new(branch_type, BranchType::Last, BranchType::Last),
                output,
                styles,
            );
        }
        Expr::Unary { op, expr, .. } => {
            append_line(output, indent, branch_type, &styles.operator.clone(), &format!("UnaryOp {op:?}"));
            print_branch(
                "Expr:",
                expr,
                indent,
                &BranchConfig::new(branch_type, BranchType::Last, BranchType::Last),
                output,
                styles,
            );
        }
        Expr::Grouping { expr, .. } => {
            append_line(output, indent, branch_type, &styles.clone().punctuation, "Grouping");
            print_branch(
                "Expr:",
                expr,
                indent,
                &BranchConfig::new(branch_type, BranchType::Last, BranchType::Last),
                output,
                styles,
            );
        }
        Expr::Literal { value, .. } => {
            append_line(
                output,
                indent,
                branch_type,
                &styles.literal.clone(),
                &format!("Literal {}", format_literal(value)),
            );
        }
        Expr::Variable { name, .. } => {
            append_line(output, indent, branch_type, &styles.variable.clone(), &format!("Variable '{name}'"));
        }
        Expr::Assign { target, value, .. } => {
            append_line(output, indent, branch_type, &styles.clone().variable, "Assignment");
            let new_indent = get_indent(indent, &branch_type);
            // Target
            append_line(output, &new_indent, BranchType::Middle, &styles.structure.clone(), "Target:");
            print_expr(target, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            append_line(output, &new_indent, BranchType::Last, &styles.structure.clone(), "Value:");
            print_expr(value, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
        }
        Expr::Call { callee, arguments, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation.clone(), "Function Call");
            let new_indent = get_indent(indent, &branch_type);
            // Callee
            append_line(output, &new_indent, BranchType::Middle, &styles.structure.clone(), "Callee:");
            print_expr(callee, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            append_line(output, &new_indent, BranchType::Last, &styles.structure.clone(), "Arguments:");
            let args_indent = get_indent(&new_indent, &BranchType::Last);
            print_children(
                arguments,
                &args_indent,
                output,
                styles,
                |arg, child_indent, branch_type, output, styles| {
                    append_line(output, child_indent, branch_type, &styles.structure.clone(), "Arg:");
                    print_expr(arg, &get_indent(child_indent, &branch_type), BranchType::Last, output, styles);
                },
            );
        }
        Expr::ArrayAccess { array, index, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation.clone(), "Array Access");
            print_branch(
                "Array:",
                array,
                indent,
                &BranchConfig::new(branch_type, BranchType::Middle, BranchType::Last),
                output,
                styles,
            );
            print_branch(
                "Index:",
                index,
                indent,
                &BranchConfig::new(branch_type, BranchType::Last, BranchType::Last),
                output,
                styles,
            );
        }
        Expr::ArrayLiteral { elements, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation.clone(), "Array Literal");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Last, &styles.structure.clone(), "Elements:");
            print_children(elements, &get_indent(&new_indent, &BranchType::Last), output, styles, print_expr);
        }
    }
}

/// Pretty-print a single statement AST into a styled, tree-like string.
/// Mirrors `pretty_print` for expressions.
#[must_use]
pub fn pretty_print_stmt(stmt: &Stmt) -> String {
    let node_count = count_stmt_nodes(stmt);
    // Statements typically have longer labels, estimate ~50 chars per node
    let mut output = String::with_capacity(node_count * STMT_CAPACITY_PER_NODE);
    let styles = StyleManager::new();
    print_stmt(stmt, "", BranchType::Last, &mut output, &styles);
    output
}

/// Counts total nodes in a statement tree for capacity estimation.
///
/// Performs a recursive traversal to compute the total number of nodes,
/// which is used to preallocate string capacity in [`pretty_print_stmt`].
/// The count is an estimate; complex statements with many sub-labels
/// (e.g., function parameters with type annotations) may generate more
/// output lines than the node count suggests.
///
/// # Performance
///
/// Runs in O(n) time when n is the number of statement and expression nodes.
/// This upfront traversal cost is amortized by avoiding reallocations
/// during string building.
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
            initializer.as_ref().map_or(0, |initializer| count_stmt_nodes(initializer))
                + condition.as_ref().map_or(0, count_expr_nodes)
                + increment.as_ref().map_or(0, count_expr_nodes)
                + count_stmt_nodes(body)
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => 0,
    }
}

#[allow(clippy::too_many_lines)]
fn print_stmt(stmt: &Stmt, indent: &str, branch_type: BranchType, output: &mut String, styles: &StyleManager) {
    match stmt {
        Stmt::Expression { expr } => {
            append_line(output, indent, branch_type, &styles.keyword, "Expression");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Last, &styles.structure, "Expr:");
            print_expr(expr, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
        }
        Stmt::VarDeclaration { bindings, type_annotation, is_mutable, .. } => {
            let declaration_kind = if *is_mutable { "VarDeclaration" } else { "ConstDeclaration" };
            append_line(output, indent, branch_type, &styles.keyword, declaration_kind);
            let new_indent = get_indent(indent, &branch_type);

            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Type:");
            let type_indent = get_indent(&new_indent, &BranchType::Middle);
            append_line(output, &type_indent, BranchType::Last, &styles.type_style, &format_type(type_annotation));

            append_line(output, &new_indent, BranchType::Last, &styles.structure, "Bindings:");
            let bindings_indent = get_indent(&new_indent, &BranchType::Last);
            if bindings.is_empty() {
                append_line(output, &bindings_indent, BranchType::Last, &styles.metadata, "(none)");
            } else {
                print_children(
                    bindings,
                    &bindings_indent,
                    output,
                    styles,
                    |binding, child_indent, binding_branch, output, styles| {
                        append_line(
                            output,
                            child_indent,
                            binding_branch,
                            &styles.variable,
                            &format!("Binding '{}'", binding.name),
                        );
                        if let Some(initializer) = &binding.initializer {
                            let initializer_indent = get_indent(child_indent, &binding_branch);
                            append_line(
                                output,
                                &initializer_indent,
                                BranchType::Last,
                                &styles.structure,
                                "Initializer:",
                            );
                            print_expr(
                                initializer,
                                &get_indent(&initializer_indent, &BranchType::Last),
                                BranchType::Last,
                                output,
                                styles,
                            );
                        }
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

            let parameters_label = if parameters.is_empty() { "Parameters: (none)" } else { "Parameters:" };
            append_line(output, &new_indent, BranchType::Middle, &styles.structure, parameters_label);
            let params_indent = get_indent(&new_indent, &BranchType::Middle);
            print_children(
                parameters,
                &params_indent,
                output,
                styles,
                |parameter, child_indent, parameter_branch, output, styles| {
                    append_line(
                        output,
                        child_indent,
                        parameter_branch,
                        &styles.structure,
                        &format!("Parameter '{}'", parameter.name),
                    );
                    append_line(
                        output,
                        &get_indent(child_indent, &parameter_branch),
                        BranchType::Last,
                        &styles.type_style,
                        &format!("Type: {}", format_type(&parameter.type_annotation)),
                    );
                },
            );

            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Return Type:");
            append_line(
                output,
                &get_indent(&new_indent, &BranchType::Middle),
                BranchType::Last,
                &styles.type_style,
                &format_type(return_type),
            );

            append_line(output, &new_indent, BranchType::Last, &styles.structure, "Body:");
            print_stmt(body, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
        }
        Stmt::If { condition, then_branch, else_branch, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "If");
            let new_indent = get_indent(indent, &branch_type);

            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Condition:");
            print_expr(condition, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);

            let then_branch_type =
                if matches!(else_branch, ElseBranch::None) { BranchType::Last } else { BranchType::Middle };
            append_line(output, &new_indent, then_branch_type, &styles.structure, "Then:");
            print_stmt(then_branch, &get_indent(&new_indent, &then_branch_type), BranchType::Last, output, styles);

            match else_branch {
                ElseBranch::None => {}
                ElseBranch::Block(branch) | ElseBranch::ElseIf(branch) => {
                    append_line(output, &new_indent, BranchType::Last, &styles.structure, "Else:");
                    print_stmt(branch, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
                }
            }
        }
        Stmt::MainFunction { body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword.clone(), "MainFunction");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Last, &styles.structure, "Body:");
            print_stmt(body, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
        }
        Stmt::Block { statements, .. } => {
            if statements.is_empty() {
                append_line(output, indent, branch_type, &styles.keyword.clone(), "Block: (empty)");
            } else {
                append_line(output, indent, branch_type, &styles.keyword.clone(), "Block");
                print_children(statements, &get_indent(indent, &branch_type), output, styles, print_stmt);
            }
        }
        Stmt::Return { value, .. } => {
            append_line(output, indent, branch_type, &styles.keyword.clone(), "Return");
            if let Some(expr) = value {
                let new_indent = get_indent(indent, &branch_type);
                append_line(output, &new_indent, BranchType::Last, &styles.structure.clone(), "Value:");
                print_expr(expr, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
            }
        }
        Stmt::While { condition, body, .. } => {
            append_line(output, indent, branch_type, &styles.clone().keyword, "While");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Middle, &styles.structure.clone(), "Condition:");
            print_expr(condition, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            append_line(output, &new_indent, BranchType::Last, &styles.structure.clone(), "Body:");
            print_stmt(body, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
        }
        Stmt::For { initializer, condition, increment, body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword.clone(), "For");
            let new_indent = get_indent(indent, &branch_type);
            if let Some(init) = initializer {
                append_line(output, &new_indent, BranchType::Middle, &styles.structure.clone(), "Initializer:");
                print_stmt(init, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            }

            if let Some(cond) = condition {
                append_line(output, &new_indent, BranchType::Middle, &styles.structure.clone(), "Condition:");
                print_expr(cond, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            }

            if let Some(inc) = increment {
                append_line(output, &new_indent, BranchType::Middle, &styles.structure.clone(), "Increment:");
                print_expr(inc, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            }

            append_line(output, &new_indent, BranchType::Last, &styles.structure.clone(), "Body:");
            print_stmt(body, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
        }
        Stmt::Break { .. } => append_line(output, indent, branch_type, &styles.keyword.clone(), "Break"),
        Stmt::Continue { .. } => append_line(output, indent, branch_type, &styles.keyword.clone(), "Continue"),
    }
}
