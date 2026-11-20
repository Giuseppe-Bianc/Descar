use crate::printers::branch_type::{BranchType, StyleManager, append_line, get_indent, print_children};
use crate::syntax::ast::assignment_op::AssignmentOp;
use crate::syntax::ast::unary_op_side::UnaryOpSide;
use crate::syntax::ast::{ElseBranch, Expr, LiteralValue, Stmt, TypeSyntax};

/// Estimated number of output bytes for each expression node.
///
/// [`pretty_print`] uses this value to reserve the initial capacity of the output string.
const EXPR_CAPACITY_PER_NODE: usize = 45;
/// Estimated number of output bytes for each statement node.
///
/// [`pretty_print_stmt`] uses this value to reserve the initial capacity of the output string.
const STMT_CAPACITY_PER_NODE: usize = 50;

/// Formats a literal value as text.
///
/// * A numeric literal uses the `Display` text of the number.
/// * A string literal is in double quotes. The function does not escape the content.
/// * A character literal is in single quotes. The function does not escape the content.
/// * A boolean literal is `true` or `false`.
/// * The null pointer literal is `nullptr`.
fn format_literal(value: &LiteralValue) -> String {
    match value {
        LiteralValue::Numeric(number) => number.to_string(),
        LiteralValue::StringLit(value) => format!("\"{value}\""),
        LiteralValue::CharLit(value) => format!("'{value}'"),
        LiteralValue::Bool(value) => value.to_string(),
        LiteralValue::NullPtr => "nullptr".to_string(),
    }
}

/// Formats a type annotation as text.
///
/// A built-in type uses its source name, for example `i32`, `string`, or `void`.
/// A custom type uses its name. An array type has the form `[<element>; <size>]`.
/// A vector type has the form `vector<<element>>`.
fn format_type(type_annotation: &TypeSyntax) -> String {
    match type_annotation {
        TypeSyntax::I8 => "i8".to_string(),
        TypeSyntax::I16 => "i16".to_string(),
        TypeSyntax::I32 => "i32".to_string(),
        TypeSyntax::I64 => "i64".to_string(),
        TypeSyntax::U8 => "u8".to_string(),
        TypeSyntax::U16 => "u16".to_string(),
        TypeSyntax::U32 => "u32".to_string(),
        TypeSyntax::U64 => "u64".to_string(),
        TypeSyntax::F32 => "f32".to_string(),
        TypeSyntax::F64 => "f64".to_string(),
        TypeSyntax::Char => "char".to_string(),
        TypeSyntax::String => "string".to_string(),
        TypeSyntax::Bool => "bool".to_string(),
        TypeSyntax::Void => "void".to_string(),
        TypeSyntax::Custom { name } => name.to_string(),
        TypeSyntax::Array { element_type, size } => {
            format!("[{}; {}]", format_type(element_type), format_type_size(size))
        }
        TypeSyntax::Vector { element_type } => format!("vector<{}>", format_type(element_type)),
    }
}

/// Formats the size expression of an array type as one line of text.
///
/// The function formats each kind of expression in a form that is similar to source code:
///
/// * A literal uses the text from [`format_literal`]. A variable uses its name.
/// * A grouping has parentheses around the inner expression.
/// * A binary expression has the form `<left> <OPERATOR> <right>`. The operator is in
///   uppercase (see [`format_binary_op`]).
/// * A cast has the form `(<expr> as <type>)`.
/// * A prefix unary expression has the form `<OPERATOR><expr>`. A postfix unary expression
///   has the form `<expr><OPERATOR>`.
/// * An assignment has the form `<target> <symbol> <value>`.
/// * A call has the form `<callee>(<arguments>)`. The arguments are separated by `, `.
/// * An array access has the form `<array>[<index>]`.
/// * An array literal has the form `{<elements>}`. The elements are separated by `, `.
fn format_type_size(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value, .. } => format_literal(value),
        Expr::Variable { name, .. } => name.clone(),
        Expr::Grouping { expr, .. } => format!("({})", format_type_size(expr)),
        Expr::Binary { left, op, right, .. } => {
            format!("{} {} {}", format_type_size(left), format_binary_op(op), format_type_size(right))
        }
        Expr::Cast { expr, target_type, .. } => format!("({} as {target_type})", format_type_size(expr)),
        Expr::Unary { op, side, expr, .. } => match side {
            UnaryOpSide::Prefix => format!("{}{}", format_unary_op(op), format_type_size(expr)),
            UnaryOpSide::Postfix => format!("{}{}", format_type_size(expr), format_unary_op(op)),
        },
        Expr::Assign { target, op, value, .. } => {
            format!("{} {} {}", format_type_size(target), format_assignment_op(*op), format_type_size(value))
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

/// Formats a binary operator as the uppercase form of its `Debug` text.
fn format_binary_op(op: &impl std::fmt::Debug) -> String {
    format!("{op:?}").to_uppercase()
}

/// Returns the source symbol of an assignment operator, for example `+=` for `AddAssign`.
const fn format_assignment_op(op: AssignmentOp) -> &'static str {
    match op {
        AssignmentOp::Assign => "=",
        AssignmentOp::AddAssign => "+=",
        AssignmentOp::SubtractAssign => "-=",
        AssignmentOp::MultiplyAssign => "*=",
        AssignmentOp::DivideAssign => "/=",
        AssignmentOp::ModuloAssign => "%=",
        AssignmentOp::BitwiseAndAssign => "&=",
        AssignmentOp::BitwiseOrAssign => "|=",
        AssignmentOp::BitwiseXorAssign => "^=",
        AssignmentOp::ShiftLeftAssign => "<<=",
        AssignmentOp::ShiftRightAssign => ">>=",
    }
}

/// Formats a unary operator as the uppercase form of its `Debug` text.
fn format_unary_op(op: &impl std::fmt::Debug) -> String {
    format!("{op:?}").to_uppercase()
}

/// Formats the side of a unary operator as `PREFIX` or `POSTFIX`.
fn format_unary_side(side: UnaryOpSide) -> String {
    format!("{side:?}").to_uppercase()
}

/// Renders an expression as a tree of text lines.
///
/// Each node of the expression has one line. A line has the form
/// `<indent><branch symbol><text>` and ends with a line break. The root node uses the
/// symbol `└── `. A node that has children shows a label line for each child, for example
/// `Left:` and `Right:` for a binary expression.
///
/// The output can contain ANSI style sequences. Use
/// [`strip_ansi_codes`](crate::utils::strip_ansi_codes) to remove them.
///
/// # Arguments
///
/// * `expr` - The expression to render.
///
/// # Returns
///
/// The text of the tree.
///
/// # Examples
///
/// ```
/// use descar_core::printers::ast_printer::pretty_print;
/// use descar_core::syntax::ast::Expr;
/// use descar_core::utils::{dummy_span, strip_ansi_codes};
///
/// let expr = Expr::new_bool_literal(true, dummy_span());
/// assert_eq!(strip_ansi_codes(&pretty_print(&expr)), "└── Literal true\n");
/// ```
#[must_use]
pub fn pretty_print(expr: &Expr) -> String {
    let node_count = count_expr_nodes(expr);
    let mut output = String::with_capacity(node_count * EXPR_CAPACITY_PER_NODE);
    let styles = StyleManager::new();
    print_expr(expr, "", BranchType::Last, &mut output, &styles);
    output
}

/// Counts the nodes of an expression tree, including the root node.
///
/// The function uses an explicit stack. It does not call itself.
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
            Expr::Cast { expr, .. } | Expr::Unary { expr, .. } | Expr::Grouping { expr, .. } => {
                stack.push(expr);
            }
            Expr::Assign { target, value, .. } => {
                stack.push(target);
                stack.push(value);
            }
            Expr::Call { callee, arguments, .. } => {
                stack.push(callee);
                for argument in arguments {
                    stack.push(argument);
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

/// Prints a label line and then the expression below the label.
///
/// The label line uses the `structure` style. The expression is the last child of the label.
///
/// # Arguments
///
/// * `label` - The text of the label line, for example `Left:`.
/// * `expr` - The expression to print below the label.
/// * `parent_indent` - The indentation text of the label line.
/// * `current_branch` - The branch type of the label line.
/// * `output` - The string that receives the lines.
/// * `styles` - The styles for the text.
fn print_labeled_expr(
    label: &str, expr: &Expr, parent_indent: &str, current_branch: BranchType, output: &mut String,
    styles: &StyleManager,
) {
    append_line(output, parent_indent, current_branch, &styles.structure, label);
    print_expr(expr, &get_indent(parent_indent, &current_branch), BranchType::Last, output, styles);
}

/// Prints an expression node and its children as a tree.
///
/// The title line of a node depends on the kind of the expression:
///
/// * `BinaryOp <OPERATOR>`, with the children `Left:` and `Right:`.
/// * `Cast to <type>`, with the child `Expr:`.
/// * `UnaryOp <OPERATOR> (<SIDE>)`, with the child `Expr:`.
/// * `Grouping`, with the child `Expr:`.
/// * `Literal <value>`, with no children.
/// * `Variable '<name>'`, with no children.
/// * `Assignment <symbol>`, with the children `Target:` and `Value:`.
/// * `Function Call`, with the child `Callee:` and the list `Arguments:`.
/// * `Array Access`, with the children `Array:` and `Index:`.
/// * `Array Literal`, with the list `Elements:`.
///
/// # Arguments
///
/// * `expr` - The expression to print.
/// * `indent` - The indentation text of the title line.
/// * `branch_type` - The branch type of the node.
/// * `output` - The string that receives the lines.
/// * `styles` - The styles for the text.
#[allow(clippy::too_many_lines)]
fn print_expr(expr: &Expr, indent: &str, branch_type: BranchType, output: &mut String, styles: &StyleManager) {
    match expr {
        Expr::Binary { left, op, right, .. } => {
            append_line(output, indent, branch_type, &styles.operator, &format!("BinaryOp {}", format_binary_op(op)));
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Left:", left, &new_indent, BranchType::Middle, output, styles);
            print_labeled_expr("Right:", right, &new_indent, BranchType::Last, output, styles);
        }
        Expr::Cast { expr, target_type, .. } => {
            append_line(output, indent, branch_type, &styles.operator, &format!("Cast to {target_type}"));
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Expr:", expr, &new_indent, BranchType::Last, output, styles);
        }
        Expr::Unary { op, side, expr, .. } => {
            append_line(
                output,
                indent,
                branch_type,
                &styles.operator,
                &format!("UnaryOp {} ({})", format_unary_op(op), format_unary_side(*side)),
            );
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Expr:", expr, &new_indent, BranchType::Last, output, styles);
        }
        Expr::Grouping { expr, .. } => {
            append_line(output, indent, branch_type, &styles.punctuation, "Grouping");
            let new_indent = get_indent(indent, &branch_type);
            print_labeled_expr("Expr:", expr, &new_indent, BranchType::Last, output, styles);
        }
        Expr::Literal { value, .. } => {
            append_line(output, indent, branch_type, &styles.literal, &format!("Literal {}", format_literal(value)));
        }
        Expr::Variable { name, .. } => {
            append_line(output, indent, branch_type, &styles.variable, &format!("Variable '{name}'"));
        }
        Expr::Assign { target, op, value, .. } => {
            append_line(
                output,
                indent,
                branch_type,
                &styles.variable,
                &format!("Assignment {}", format_assignment_op(*op)),
            );
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
                    print_expr(argument, &get_indent(child_indent, &child_branch), BranchType::Last, output, styles);
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
            print_children(elements, &get_indent(&new_indent, &BranchType::Last), output, styles, print_expr);
        }
    }
}

/// Renders a statement as a tree of text lines.
///
/// Each node of the statement has one line. A line has the form
/// `<indent><branch symbol><text>` and ends with a line break. The root node uses the
/// symbol `└── `. The tree includes the expressions in the statement. The format of the
/// expression nodes is the same as the format of [`pretty_print`].
///
/// The output can contain ANSI style sequences. Use
/// [`strip_ansi_codes`](crate::utils::strip_ansi_codes) to remove them.
///
/// # Arguments
///
/// * `stmt` - The statement to render.
///
/// # Returns
///
/// The text of the tree.
#[must_use]
pub fn pretty_print_stmt(stmt: &Stmt) -> String {
    let node_count = count_stmt_nodes(stmt);
    let mut output = String::with_capacity(node_count * STMT_CAPACITY_PER_NODE);
    let styles = StyleManager::new();
    print_stmt(stmt, "", BranchType::Last, &mut output, &styles);
    output
}

/// Counts the nodes of a statement tree, including the root node.
///
/// The function counts the statement itself and the nodes of its child statements and
/// expressions. A variable binding and a function parameter count as one node each.
/// A `break` statement and a `continue` statement have no child nodes.
///
/// The function calls itself for the child statements. [`pretty_print_stmt`] uses the
/// result to estimate the capacity of the output string.
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

/// Prints the content of a statement body.
///
/// For a block, the function prints each statement of the block as a child with the given
/// indentation. The function does not print a line for the block itself. If the block has
/// no statements, the function prints the line `(empty)`.
///
/// For any other statement, the function prints the statement as the last child.
///
/// # Arguments
///
/// * `stmt` - The body statement.
/// * `indent` - The indentation text of the printed lines.
/// * `output` - The string that receives the lines.
/// * `styles` - The styles for the text.
fn print_stmt_body(stmt: &Stmt, indent: &str, output: &mut String, styles: &StyleManager) {
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

/// Prints a label line and then the body below the label.
///
/// If the body is a block without statements, the function prints only one line with the
/// text `<label> (empty)`. Otherwise, the function prints the label line and then the
/// content of the body (see [`print_stmt_body`]).
///
/// # Arguments
///
/// * `label` - The text of the label line, for example `Body:`.
/// * `body` - The body statement.
/// * `parent_indent` - The indentation text of the label line.
/// * `branch_type` - The branch type of the label line.
/// * `output` - The string that receives the lines.
/// * `styles` - The styles for the text.
fn append_body_label(
    label: &str, body: &Stmt, parent_indent: &str, branch_type: BranchType, output: &mut String, styles: &StyleManager,
) {
    match body {
        Stmt::Block { statements, .. } if statements.is_empty() => {
            append_line(output, parent_indent, branch_type, &styles.structure, &format!("{label} (empty)"));
        }
        _ => {
            append_line(output, parent_indent, branch_type, &styles.structure, label);
            print_stmt_body(body, &get_indent(parent_indent, &branch_type), output, styles);
        }
    }
}

/// Prints a statement node and its children as a tree.
///
/// The title line of a node depends on the kind of the statement:
///
/// * `Expression`, with the child `Expr:`.
/// * `VarDeclaration` for a mutable declaration, or `ConstDeclaration` for a declaration
///   that is not mutable. The children are the list `Variables:` (or `(none)` if the
///   declaration has no bindings) and the child `Type:`. If at least one binding has an
///   initializer, the node also has the list `Initializers:`.
/// * `Function`, with the children `Name:`, `Parameters:` (or `Parameters: (none)`),
///   `Return Type:`, and `Body:`.
/// * `If`, with the children `Condition:` and `Then:`. If the statement has an `else`
///   branch, the node also has the child `Else:`.
/// * `MainFunction`, with the content of the body.
/// * `Block`, with one child for each statement, or `Block: (empty)` if the block has no
///   statements.
/// * `Return`, with the child `Value:` if the statement has a value.
/// * `While`, with the children `Condition:` and `Body:`.
/// * `For`, with the children `Initializer:`, `Condition:`, and `Increment:` when they
///   exist, and the child `Body:`.
/// * `Break` and `Continue`, with no children.
///
/// # Arguments
///
/// * `stmt` - The statement to print.
/// * `indent` - The indentation text of the title line.
/// * `branch_type` - The branch type of the node.
/// * `output` - The string that receives the lines.
/// * `styles` - The styles for the text.
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
                let initializers =
                    bindings.iter().filter_map(|binding| binding.initializer.as_ref()).collect::<Vec<_>>();
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

            if parameters.is_empty() {
                append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Parameters: (none)");
            } else {
                append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Parameters:");
                let params_indent = get_indent(&new_indent, &BranchType::Middle);
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
            print_expr(condition, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);

            let then_branch_type =
                if matches!(else_branch, ElseBranch::None) { BranchType::Last } else { BranchType::Middle };
            append_body_label("Then:", then_branch, &new_indent, then_branch_type, output, styles);

            if let ElseBranch::Block(branch) | ElseBranch::ElseIf(branch) = else_branch {
                append_body_label("Else:", branch, &new_indent, BranchType::Last, output, styles);
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
                let new_indent = get_indent(indent, &branch_type);
                append_line(output, &new_indent, BranchType::Last, &styles.structure, "Value:");
                print_expr(expr, &get_indent(&new_indent, &BranchType::Last), BranchType::Last, output, styles);
            }
        }
        Stmt::While { condition, body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "While");
            let new_indent = get_indent(indent, &branch_type);
            append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Condition:");
            print_expr(condition, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            append_body_label("Body:", body, &new_indent, BranchType::Last, output, styles);
        }
        Stmt::For { initializer, condition, increment, body, .. } => {
            append_line(output, indent, branch_type, &styles.keyword, "For");
            let new_indent = get_indent(indent, &branch_type);

            if let Some(init) = initializer {
                let branch =
                    if condition.is_some() || increment.is_some() { BranchType::Middle } else { BranchType::Last };
                append_line(output, &new_indent, branch, &styles.structure, "Initializer:");
                print_stmt(init, &get_indent(&new_indent, &branch), BranchType::Last, output, styles);
            }
            if let Some(cond) = condition {
                let branch = if increment.is_some() { BranchType::Middle } else { BranchType::Last };
                append_line(output, &new_indent, branch, &styles.structure, "Condition:");
                print_expr(cond, &get_indent(&new_indent, &branch), BranchType::Last, output, styles);
            }
            if let Some(inc) = increment {
                append_line(output, &new_indent, BranchType::Middle, &styles.structure, "Increment:");
                print_expr(inc, &get_indent(&new_indent, &BranchType::Middle), BranchType::Last, output, styles);
            }
            append_body_label("Body:", body, &new_indent, BranchType::Last, output, styles);
        }
        Stmt::Break { .. } => append_line(output, indent, branch_type, &styles.keyword, "Break"),
        Stmt::Continue { .. } => append_line(output, indent, branch_type, &styles.keyword, "Continue"),
    }
}
