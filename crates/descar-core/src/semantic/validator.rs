use std::collections::BTreeSet;

use crate::syntax::ast::{ElseBranch, Expr, Stmt, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstValidationError {
    InvalidSpan { file: String },
    MultipleSourceFiles { first: String, second: String },
}

/// Validates structural invariants required before semantic analysis begins.
pub fn validate_ast_shape(ast: &[Stmt]) -> Result<(), AstValidationError> {
    let mut files = BTreeSet::new();
    for stmt in ast {
        validate_stmt(stmt, &mut files)?;
    }
    Ok(())
}

fn validate_stmt(stmt: &Stmt, files: &mut BTreeSet<String>) -> Result<(), AstValidationError> {
    validate_span(stmt.span(), files)?;
    match stmt {
        Stmt::Expression { expr } => validate_expr(expr, files),
        Stmt::VarDeclaration { bindings, type_annotation, .. } => {
            validate_type(type_annotation, files)?;
            for binding in bindings {
                if let Some(expr) = &binding.initializer {
                    validate_expr(expr, files)?;
                }
            }
            Ok(())
        }
        Stmt::Function { parameters, return_type, body, .. } => {
            for parameter in parameters {
                validate_type(&parameter.type_annotation, files)?;
                validate_span(&parameter.span, files)?;
            }
            validate_type(return_type, files)?;
            validate_stmt(body, files)
        }
        Stmt::If { condition, then_branch, else_branch, .. } => {
            validate_expr(condition, files)?;
            validate_stmt(then_branch, files)?;
            match else_branch {
                ElseBranch::None => Ok(()),
                ElseBranch::Block(stmt) | ElseBranch::ElseIf(stmt) => validate_stmt(stmt, files),
            }
        }
        Stmt::While { condition, body, .. } => {
            validate_expr(condition, files)?;
            validate_stmt(body, files)
        }
        Stmt::For { initializer, condition, increment, body, .. } => {
            if let Some(initializer) = initializer {
                validate_stmt(initializer, files)?;
            }
            if let Some(condition) = condition {
                validate_expr(condition, files)?;
            }
            if let Some(increment) = increment {
                validate_expr(increment, files)?;
            }
            validate_stmt(body, files)
        }
        Stmt::Block { statements, .. } => {
            for stmt in statements {
                validate_stmt(stmt, files)?;
            }
            Ok(())
        }
        Stmt::Return { value, .. } => value.as_ref().map_or(Ok(()), |expr| validate_expr(expr, files)),
        Stmt::Break { .. } | Stmt::Continue { .. } => Ok(()),
        Stmt::MainFunction { body, .. } => validate_stmt(body, files),
    }
}

fn validate_expr(expr: &Expr, files: &mut BTreeSet<String>) -> Result<(), AstValidationError> {
    validate_span(expr.span(), files)?;
    match expr {
        Expr::Binary { left, right, .. } => {
            validate_expr(left, files)?;
            validate_expr(right, files)
        }
        Expr::Unary { expr, .. } | Expr::Grouping { expr, .. } => validate_expr(expr, files),
        Expr::Literal { .. } | Expr::Variable { .. } => Ok(()),
        Expr::ArrayLiteral { elements, .. } => {
            for element in elements {
                validate_expr(element, files)?;
            }
            Ok(())
        }
        Expr::Assign { target, value, .. } => {
            validate_expr(target, files)?;
            validate_expr(value, files)
        }
        Expr::Call { callee, arguments, .. } => {
            validate_expr(callee, files)?;
            for argument in arguments {
                validate_expr(argument, files)?;
            }
            Ok(())
        }
        Expr::ArrayAccess { array, index, .. } => {
            validate_expr(array, files)?;
            validate_expr(index, files)
        }
    }
}

fn validate_type(ty: &Type, files: &mut BTreeSet<String>) -> Result<(), AstValidationError> {
    match ty {
        Type::Array { element_type, size } => {
            validate_type(element_type, files)?;
            validate_expr(size, files)
        }
        Type::Vector { element_type } => validate_type(element_type, files),
        Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::F32
        | Type::F64
        | Type::Char
        | Type::String
        | Type::Bool
        | Type::Custom { .. }
        | Type::Void
        | Type::NullPtr => Ok(()),
    }
}

fn validate_span(
    span: &crate::location::source_span::SourceSpan, files: &mut BTreeSet<String>,
) -> Result<(), AstValidationError> {
    if span.end().offset() < span.start().offset() {
        return Err(AstValidationError::InvalidSpan { file: span.file_path().to_owned() });
    }
    if let Some(first) = files.iter().next()
        && first != span.file_path()
    {
        return Err(AstValidationError::MultipleSourceFiles {
            first: first.clone(),
            second: span.file_path().to_owned(),
        });
    }
    files.insert(span.file_path().to_owned());
    Ok(())
}
