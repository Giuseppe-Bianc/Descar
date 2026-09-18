//! Type checking and semantic analysis for the Descar language.
//!
//! This module performs semantic analysis on the AST, validating type correctness,
//! variable scoping, and control flow constraints. It implements type inference,
//! type promotion rules, and comprehensive error detection.
//!
//! # Type System Features
//!
//! - **Type Inference**: Automatic type deduction from literals and expressions
//! - **Type Promotion**: Implicit conversion between compatible numeric types
//! - **Scope Management**: Lexical scoping with symbol table management
//! - **Control Flow Analysis**: Validation of break/continue/return statements
//!
//! # Type Hierarchy
//!
//! The type checker implements a numeric type hierarchy for promotion:
//! `F64 > F32 > U64 > I64 > U32 > I32 > U16 > I16 > U8 > I8`
//!
//! # Examples
//!
//! ```rust,no_run
//! use descar_core::semantic::type_checker::TypeChecker;
//! use descar_core::syntax::ast::Stmt;
//!
//! let mut checker = TypeChecker::new();
//! let statements: Vec<Stmt> = vec![];
//! let errors = checker.check(&statements);
//!
//! if errors.is_empty() {
//!     // Type checking succeeded
//! } else {
//!     // Report type errors
//! }
//! ```
use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::location::source_span::SourceSpan;
use crate::semantic::symbol_table::{FunctionSymbol, ScopeKind, Symbol, SymbolTable, VariableSymbol};
use crate::syntax::ast::binary_op::BinaryOp;
use crate::syntax::ast::else_branch::ElseBranch;
use crate::syntax::ast::stmt::VarBinding;
use crate::syntax::ast::unary_op::UnaryOp;
use crate::syntax::ast::{Expr, LiteralValue, Parameter, Stmt, Type};
use crate::tokens::number::Number;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Type checker for semantic analysis of Descar programs.
///
/// The type checker validates that all operations are type-safe, variables are
/// properly declared and used, and control flow constructs are valid. It maintains
/// a symbol table for scoping and tracks context for validation (loop depth,
/// function return types).
///
/// # Fields
///
/// * `in_loop` - Tracks whether currently inside a loop (for break/continue validation)
/// * `return_type_stack` - Stack of expected return types for nested functions
/// * `errors` - Accumulated type errors found during checking
/// * `symbol_table` - Symbol table for variable and function declarations
///
/// # Type Checking Process
///
/// 1. Traverse the AST depth-first
/// 2. For each node, validate types and scoping rules
/// 3. Apply type promotion where necessary
/// 4. Accumulate errors without stopping (to report multiple issues)
/// 5. Return all errors found
///
/// # Examples
///
/// ```rust,no_run
/// use descar_core::semantic::type_checker::TypeChecker;
/// use descar_core::syntax::ast::Stmt;
/// let mut checker = TypeChecker::new();
/// let statements :Vec<Stmt> = vec![];
/// let errors = checker.check(&statements);
/// ```
pub struct TypeChecker {
    in_loop: bool,
    return_type_stack: Vec<Type>,
    errors: Vec<CompileError>,
    symbol_table: SymbolTable,
}

// Gerarchia per la promozione dei tipi numerici
const HIERARCHY: [Type; 10] =
    [Type::F64, Type::F32, Type::U64, Type::I64, Type::U32, Type::I32, Type::U16, Type::I16, Type::U8, Type::I8];

// Global cache for type promotion results
static TYPE_PROMOTION_CACHE: OnceLock<Mutex<HashMap<(Type, Type), Type>>> = OnceLock::new();

#[allow(clippy::collapsible_if)]
impl TypeChecker {
    /// Creates a new type checker with empty state.
    ///
    /// Initializes a fresh symbol table and error collection for a new
    /// type checking pass.
    ///
    /// # Returns
    ///
    /// A new `TypeChecker` instance ready to check statements.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use descar_core::semantic::type_checker::TypeChecker;
    /// use descar_core::syntax::ast::Stmt;
    /// let mut checker = TypeChecker::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { symbol_table: SymbolTable::new(), errors: Vec::new(), in_loop: false, return_type_stack: Vec::new() }
    }

    /// Records a type error with an optional error code.
    ///
    /// # Arguments
    ///
    /// * `code` - Optional error code for the error
    /// * `message` - Error message describing the type violation
    /// * `span` - Source location where the error occurred
    fn type_error_with_code(&mut self, code: Option<ErrorCode>, message: impl Into<Arc<str>>, span: &SourceSpan) {
        self.errors.push(CompileError::TypeError { code, message: message.into(), span: span.clone(), help: None });
    }

    /// Performs type checking on a list of statements.
    ///
    /// This is the main entry point for type checking. It traverses the entire
    /// AST, validating types and collecting errors. The checker continues after
    /// encountering errors to report as many issues as possible in one pass.
    ///
    /// # Arguments
    ///
    /// * `statements` - The AST statements to type check
    ///
    /// # Returns
    ///
    /// A vector of all type errors found. An empty vector indicates successful
    /// type checking with no errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use descar_core::semantic::type_checker::TypeChecker;
    /// use descar_core::syntax::ast::Stmt;
    /// let mut checker = TypeChecker::new();
    /// let statements :Vec<Stmt> = vec![];
    /// let errors = checker.check(&statements);
    ///
    /// if !errors.is_empty() {
    ///     for error in errors {
    ///         eprintln!("{}", error);
    ///     }
    /// }
    /// ```
    pub fn check(&mut self, statements: &[Stmt]) -> Vec<CompileError> {
        self.visit_statements(statements);
        std::mem::take(&mut self.errors)
    }

    // Helper method per dichiarare simboli
    fn declare_symbol(&mut self, name: &str, symbol: Symbol, declared_at: &SourceSpan) {
        if let Err(e) = self.symbol_table.declare(name, symbol, declared_at.clone()) {
            self.errors.push(e);
        }
    }

    fn visit_statements(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression { expr } => {
                self.visit_expr(expr);
            }
            Stmt::VarDeclaration { bindings, type_annotation, is_mutable, span } => {
                self.visit_var_declaration(bindings, type_annotation, *is_mutable, span);
            }
            Stmt::Function { name, parameters, return_type, body, span } => {
                self.visit_function(name, parameters, return_type, body, span);
            }
            Stmt::If { condition, then_branch, else_branch, span } => {
                self.visit_if(condition, then_branch, else_branch, span);
            }
            Stmt::While { condition, body, span } => {
                self.visit_while(condition, body, span);
            }
            Stmt::For { initializer, condition, increment, body, span } => {
                self.visit_for(initializer, condition, increment, body, span);
            }
            Stmt::Block { statements, span } => self.visit_block(statements, span),
            Stmt::Return { value, span } => self.visit_return(value.as_ref(), span),
            Stmt::Break { span } => self.visit_break(span),
            Stmt::Continue { span } => self.visit_continue(span),
            Stmt::MainFunction { body, span } => {
                self.visit_main_function(body, span);
            }
        }
    }

    fn visit_var_declaration(
        &mut self, bindings: &[VarBinding], type_annotation: &Type, is_mutable: bool, span: &SourceSpan,
    ) {
        for binding in bindings {
            if let Some(init_expr) = binding.initializer.as_ref() {
                if let Some(init_type) = self.visit_expr(init_expr) {
                    if !self.is_assignable(&init_type, type_annotation) {
                        self.type_error_with_code(
                            Some(ErrorCode::E2002),
                            format!("Cannot assign {init_type} to {type_annotation} for variable '{}'", binding.name),
                            init_expr.span(),
                        );
                    }
                }
            }

            self.declare_symbol(
                &binding.name,
                Symbol::Variable(VariableSymbol {
                    name: binding.name.clone().into(),
                    ty: type_annotation.clone(),
                    mutable: is_mutable,
                    defined_at: span.clone(),
                    last_assignment: None,
                }),
                span,
            );
        }
    }

    fn visit_function(
        &mut self, name: &str, parameters: &[Parameter], return_type: &Type, body: &Stmt, span: &SourceSpan,
    ) {
        let func_symbol = FunctionSymbol {
            name: name.into(),
            parameters: parameters.to_vec(),
            return_type: return_type.clone(),
            defined_at: span.clone(),
        };
        self.declare_symbol(name, Symbol::Function(func_symbol), span);
        self.symbol_table.push_scope(ScopeKind::Function, Some(span.clone()));
        self.return_type_stack.push(return_type.clone());
        for param in parameters {
            self.declare_symbol(
                &param.name,
                Symbol::Variable(VariableSymbol {
                    name: param.name.clone().into(),
                    ty: param.type_annotation.clone(),
                    mutable: true,
                    defined_at: param.span.clone(),
                    last_assignment: None,
                }),
                &param.span,
            );
        }
        match body {
            Stmt::Block { statements, .. } => self.visit_statements(statements),
            _ => self.visit_stmt(body),
        }

        if *return_type != Type::Void && !self.function_has_return(body) {
            self.type_error_with_code(
                Some(ErrorCode::E2003),
                format!(
                    "Function '{name}' may not return value in all code paths (expected return type: {return_type})"
                ),
                span,
            );
        }
        self.return_type_stack.pop();
        self.symbol_table.pop_scope();
    }

    fn visit_main_function(&mut self, body: &Stmt, span: &SourceSpan) {
        self.visit_function("main", &[], &Type::Void, body, span);
    }

    fn check_condition(&mut self, condition: &Expr, construct: &str) {
        if let Some(cond_type) = self.visit_expr(condition) {
            if cond_type != Type::Bool {
                self.type_error_with_code(
                    Some(ErrorCode::E2004),
                    format!("Condition in {construct} must be boolean, found {cond_type}"),
                    condition.span(),
                );
            }
        }
    }

    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &ElseBranch, _span: &SourceSpan) {
        self.check_condition(condition, "'if' statement");
        self.visit_stmt(then_branch);

        match else_branch {
            ElseBranch::None => {}
            ElseBranch::Block(stmt) | ElseBranch::ElseIf(stmt) => {
                self.visit_stmt(stmt);
            }
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt, _span: &SourceSpan) {
        self.check_condition(condition, "'while' loop");

        let was_in_loop = self.in_loop;
        self.in_loop = true;
        self.symbol_table.push_scope(ScopeKind::Block, Some(condition.span().clone()));
        self.visit_stmt(body);
        self.symbol_table.pop_scope();
        self.in_loop = was_in_loop;
    }

    #[allow(clippy::ref_option)]
    fn visit_for(
        &mut self, initializer: &Option<Box<Stmt>>, condition: &Option<Expr>, increment: &Option<Expr>, body: &Stmt,
        span: &SourceSpan,
    ) {
        self.symbol_table.push_scope(ScopeKind::Block, Some(span.clone()));

        if let Some(init) = initializer {
            self.visit_stmt(init);
        }
        if let Some(cond) = condition {
            self.check_condition(cond, "For loop");
        }
        if let Some(inc) = increment {
            self.visit_expr(inc);
        }

        let was_in_loop = self.in_loop;
        self.in_loop = true;
        self.visit_stmt(body);
        self.in_loop = was_in_loop;

        self.symbol_table.pop_scope();
    }

    fn visit_block(&mut self, statements: &[Stmt], span: &SourceSpan) {
        self.symbol_table.push_scope(ScopeKind::Block, Some(span.clone()));
        self.visit_statements(statements);
        self.symbol_table.pop_scope();
    }

    fn visit_return(&mut self, value: Option<&Expr>, span: &SourceSpan) {
        if self.return_type_stack.is_empty() {
            self.type_error_with_code(Some(ErrorCode::E2005), "Return statement must be inside function body", span);
            return;
        }
        let expected_type = self.return_type_stack.last().cloned().unwrap_or(Type::Void);
        match (value, &expected_type) {
            (Some(expr), Type::Void) => {
                self.type_error_with_code(
                    Some(ErrorCode::E2006),
                    "Cannot return a value from void function",
                    expr.span(),
                );
            }
            (Some(expr), _) => {
                if let Some(actual_type) = self.visit_expr(expr) {
                    if !self.is_assignable(&actual_type, &expected_type) {
                        self.type_error_with_code(
                            Some(ErrorCode::E2007),
                            format!("Return type mismatch: expected {expected_type} found {actual_type}"),
                            expr.span(),
                        );
                    }
                }
            }
            (None, Type::Void) => {}
            (None, _) => {
                self.type_error_with_code(
                    Some(ErrorCode::E2008),
                    format!("Return type mismatch, expected {expected_type} found Void"),
                    span,
                );
            }
        }
    }

    fn visit_break(&mut self, span: &SourceSpan) {
        if !self.in_loop {
            self.type_error_with_code(Some(ErrorCode::E2009), "Break statement outside loop", span);
        }
    }

    fn visit_continue(&mut self, span: &SourceSpan) {
        if !self.in_loop {
            self.type_error_with_code(Some(ErrorCode::E2010), "Continue statement outside loop", span);
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Binary { left, op, right, span } => self.visit_binary_expr(left, *op, right, span),
            Expr::Unary { op, expr, span, .. } => self.visit_unary_expr(*op, expr, span),
            Expr::Grouping { expr, span: _ } => self.visit_expr(expr),
            Expr::Literal { value, span } => self.visit_literal(value, span),
            Expr::ArrayLiteral { elements, span } => self.visit_array_literal(elements, span),
            Expr::Variable { name, span } => self.visit_variable(name, span),
            Expr::Assign { target, value, span } => self.visit_assign(target, value, span),
            Expr::Call { callee, arguments, span } => self.visit_call(callee, arguments, span),
            Expr::ArrayAccess { array, index, span } => self.visit_array_access(array, index, span),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn visit_binary_expr(&mut self, left: &Expr, op: BinaryOp, right: &Expr, span: &SourceSpan) -> Option<Type> {
        let left_type = self.visit_expr(left);

        let is_compound = matches!(
            op,
            BinaryOp::AddEqual
                | BinaryOp::SubtractEqual
                | BinaryOp::MultiplyEqual
                | BinaryOp::DivideEqual
                | BinaryOp::ModuloEqual
                | BinaryOp::BitwiseAndEqual
                | BinaryOp::BitwiseOrEqual
                | BinaryOp::BitwiseXorEqual
                | BinaryOp::ShiftLeftEqual
                | BinaryOp::ShiftRightEqual
        );

        if is_compound {
            if let Some(name) = Self::base_variable_name(left) {
                if let Some(var) = self.symbol_table.lookup_variable(name) {
                    if !var.mutable {
                        self.type_error_with_code(
                            Some(ErrorCode::E2024),
                            format!("Cannot assign to immutable variable '{name}'"),
                            left.span(),
                        );
                        return None;
                    }
                }
            }
        }

        let right_type = self.visit_expr(right);
        let (Some(mut left_type), Some(mut right_type)) = (left_type, right_type) else {
            return None;
        };

        if matches!(
            op,
            BinaryOp::BitwiseAnd
                | BinaryOp::BitwiseAndEqual
                | BinaryOp::BitwiseOr
                | BinaryOp::BitwiseOrEqual
                | BinaryOp::BitwiseXor
                | BinaryOp::BitwiseXorEqual
                | BinaryOp::ShiftLeft
                | BinaryOp::ShiftLeftEqual
                | BinaryOp::ShiftRight
                | BinaryOp::ShiftRightEqual
        ) {
            if Self::is_integer_type(&left_type) && Self::is_integer_type(&right_type) {
                let common_type = self.promote_numeric_types(&left_type, &right_type);
                left_type = common_type.clone();
                right_type = common_type;
            } else {
                self.type_error_with_code(
                    Some(ErrorCode::E2011),
                    format!(
                        "Bitwise operator '{op:?}' require integer operand types, found {left_type} and {right_type}"
                    ),
                    span,
                );
                return None;
            }
        } else if matches!(
            op,
            BinaryOp::Add
                | BinaryOp::AddEqual
                | BinaryOp::Subtract
                | BinaryOp::SubtractEqual
                | BinaryOp::Multiply
                | BinaryOp::MultiplyEqual
                | BinaryOp::Divide
                | BinaryOp::DivideEqual
                | BinaryOp::Modulo
                | BinaryOp::ModuloEqual
                | BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
        ) && Self::is_numeric(&left_type)
            && Self::is_numeric(&right_type)
        {
            let common_type = self.promote_numeric_types(&left_type, &right_type);
            left_type = common_type.clone();
            right_type = common_type;
        }

        if !self.are_compatible(&left_type, &right_type) {
            let (code, message) = match op {
                BinaryOp::And | BinaryOp::Or => (
                    ErrorCode::E2012,
                    format!(
                        "Logical operator '{op:?}' requires boolean operands types, found {left_type} and {right_type}"
                    ),
                ),
                BinaryOp::Add
                | BinaryOp::AddEqual
                | BinaryOp::Subtract
                | BinaryOp::SubtractEqual
                | BinaryOp::Multiply
                | BinaryOp::MultiplyEqual
                | BinaryOp::Divide
                | BinaryOp::DivideEqual
                | BinaryOp::Modulo
                | BinaryOp::ModuloEqual => (
                    ErrorCode::E2013,
                    format!("Binary operator '{op:?}' requires numeric operands, found {left_type} and {right_type}"),
                ),
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => (
                    ErrorCode::E2014,
                    format!(
                        "Comparison operator '{op:?}' requires compatible types, found {left_type} and {right_type}"
                    ),
                ),
                _ => (ErrorCode::E2015, format!("Type mismatch in binary operation: {left_type} and {right_type}")),
            };
            self.type_error_with_code(Some(code), message, span);
            return None;
        }

        Some(match op {
            BinaryOp::Add
            | BinaryOp::AddEqual
            | BinaryOp::Subtract
            | BinaryOp::SubtractEqual
            | BinaryOp::Multiply
            | BinaryOp::MultiplyEqual
            | BinaryOp::Divide
            | BinaryOp::DivideEqual
            | BinaryOp::Modulo
            | BinaryOp::ModuloEqual => {
                if !Self::is_numeric(&left_type) {
                    self.type_error_with_code(
                        Some(ErrorCode::E2016),
                        format!("Arithmetic operation not supported for {left_type}"),
                        left.span(),
                    );
                }
                left_type
            }
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => Type::Bool,
            BinaryOp::And | BinaryOp::Or => {
                if left_type != Type::Bool {
                    self.type_error_with_code(
                        Some(ErrorCode::E2017),
                        format!("Logical operation requires bool, found {left_type}"),
                        left.span(),
                    );
                }
                Type::Bool
            }
            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseAndEqual
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseOrEqual
            | BinaryOp::BitwiseXor
            | BinaryOp::BitwiseXorEqual
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftLeftEqual
            | BinaryOp::ShiftRight
            | BinaryOp::ShiftRightEqual => left_type,
        })
    }

    fn visit_unary_expr(&mut self, op: UnaryOp, expr: &Expr, _span: &SourceSpan) -> Option<Type> {
        let expr_type = self.visit_expr(expr)?;

        match op {
            UnaryOp::Negate => {
                if Self::is_numeric(&expr_type) {
                    Some(expr_type)
                } else {
                    self.type_error_with_code(
                        Some(ErrorCode::E2018),
                        format!("Negation requires numeric type operand, found {expr_type}"),
                        expr.span(),
                    );
                    None
                }
            }

            UnaryOp::Not => {
                if expr_type == Type::Bool {
                    Some(Type::Bool)
                } else {
                    self.type_error_with_code(
                        Some(ErrorCode::E2019),
                        format!("Logical not requires boolean type operand, found {expr_type}"),
                        expr.span(),
                    );
                    None
                }
            }

            UnaryOp::BitwiseNot => {
                if Self::is_integer_type(&expr_type) {
                    Some(expr_type)
                } else {
                    self.type_error_with_code(
                        Some(ErrorCode::E2018),
                        format!("Bitwise not requires integer operand type, found {expr_type}"),
                        expr.span(),
                    );
                    None
                }
            }

            UnaryOp::Increment | UnaryOp::Decrement => {
                if let Some(name) = Self::base_variable_name(expr) {
                    if let Some(var) = self.symbol_table.lookup_variable(name) {
                        if !var.mutable {
                            self.type_error_with_code(
                                Some(ErrorCode::E2024),
                                format!("Cannot assign to immutable variable '{name}'"),
                                expr.span(),
                            );
                            return None;
                        }
                    }
                }

                if Self::is_numeric(&expr_type) {
                    Some(expr_type)
                } else {
                    self.type_error_with_code(
                        Some(ErrorCode::E2018),
                        format!("Increment/decrement requires numeric operand, found {expr_type}"),
                        expr.span(),
                    );
                    None
                }
            }
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    const fn visit_literal(&self, value: &LiteralValue, _span: &SourceSpan) -> Option<Type> {
        Some(match value {
            LiteralValue::Numeric(n) => self.type_of_number(n),
            LiteralValue::StringLit(_) => Type::String,
            LiteralValue::CharLit(_) => Type::Char,
            LiteralValue::Bool(_) => Type::Bool,
            LiteralValue::NullPtr => Type::NullPtr,
        })
    }

    /// Returns the Descar type represented by a parsed numeric literal.
    #[must_use]
    pub const fn type_of_number(&self, n: &Number) -> Type {
        match n {
            Number::I8(_) => Type::I8,
            Number::I16(_) => Type::I16,
            Number::I32(_) => Type::I32,
            Number::Integer(_) => Type::I64,
            Number::U8(_) => Type::U8,
            Number::U16(_) => Type::U16,
            Number::U32(_) => Type::U32,
            Number::UnsignedInteger(_) => Type::U64,
            Number::Float32(_) | Number::Scientific32(_, _) => Type::F32,
            Number::Float64(_) | Number::Scientific64(_, _) => Type::F64,
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn visit_array_literal(&mut self, elements: &[Expr], span: &SourceSpan) -> Option<Type> {
        if elements.is_empty() {
            self.type_error_with_code(
                Some(ErrorCode::E2020),
                "Array literals must have at least one element for type inference",
                span,
            );
            return None; // Ritorna None dopo aver segnalato l'errore
        }
        let len = elements.len();
        let mut element_type = None;
        for element in elements {
            if let Some(ty) = self.visit_expr(element) {
                if let Some(prev) = &element_type {
                    if !self.is_same_type(prev, &ty) {
                        self.type_error_with_code(
                            Some(ErrorCode::E2021),
                            format!("All array elements must be same type, found mixed types: {prev} and {ty}"),
                            element.span(),
                        );
                    }
                } else {
                    element_type = Some(ty);
                }
            }
        }
        element_type.map(|ty| {
            // Create proper size expression with actual length
            let size_expr =
                Expr::Literal { value: LiteralValue::Numeric(Number::Integer(len as i64)), span: span.clone() };
            Type::Array { element_type: Box::new(ty), size: Box::new(size_expr) }
        })
    }

    /// Returns whether two types are identical for type-checking purposes.
    ///
    /// Array element types are compared recursively. Nonnegative integer literal
    /// lengths are normalized before comparison; if neither length can be
    /// evaluated, the expressions must be structurally equal.
    #[must_use]
    pub fn is_same_type(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::Array { element_type: elem1, size: size1 }, Type::Array { element_type: elem2, size: size2 }) => {
                // First check if element types are the same
                if !self.is_same_type(elem1, elem2) {
                    return false;
                }

                // Then compare the sizes by evaluating the expressions
                match (self.get_size(size1), self.get_size(size2)) {
                    (Some(s1), Some(s2)) => s1 == s2,
                    (None, None) => size1 == size2,
                    _ => false,
                }
            }
            _ => t1 == t2,
        }
    }

    /// Converts a signed integer to u64 if possible.
    #[allow(clippy::unused_self)]
    fn signed_to_size<T: Into<i64> + Copy>(&self, n: T) -> Option<u64> {
        n.into().try_into().ok()
    }

    /// Evaluates a nonnegative integer literal as an array size.
    ///
    /// Returns `None` for negative integers, floating-point literals, and
    /// expressions that are not integer literals.
    #[must_use]
    pub fn get_size(&self, expr: &Expr) -> Option<u64> {
        if let Expr::Literal { value, .. } = expr {
            match value {
                LiteralValue::Numeric(Number::I8(n)) => self.signed_to_size(*n),
                LiteralValue::Numeric(Number::I16(n)) => self.signed_to_size(*n),
                LiteralValue::Numeric(Number::I32(n)) => self.signed_to_size(*n),
                LiteralValue::Numeric(Number::Integer(n)) => self.signed_to_size(*n),
                // Unsigned types (already efficient)
                LiteralValue::Numeric(Number::U8(n)) => Some(u64::from(*n)),
                LiteralValue::Numeric(Number::U16(n)) => Some(u64::from(*n)),
                LiteralValue::Numeric(Number::U32(n)) => Some(u64::from(*n)),
                LiteralValue::Numeric(Number::UnsignedInteger(n)) => Some(*n),
                _ => None,
            }
        } else {
            None
        }
    }

    fn visit_variable(&mut self, name: &str, span: &SourceSpan) -> Option<Type> {
        if let Some(var) = self.symbol_table.lookup_variable(name) {
            Some(var.ty)
        } else {
            if self.symbol_table.lookup_function(name).is_some() {
                self.type_error_with_code(
                    Some(ErrorCode::E2022),
                    format!("'{name}' is a function and cannot be used as variable"),
                    span,
                );
            } else {
                self.type_error_with_code(Some(ErrorCode::E2023), format!("Undefined variable '{name}'"), span);
            }
            None
        }
    }

    fn base_variable_name(expr: &Expr) -> Option<&str> {
        match expr {
            Expr::Variable { name, .. } => Some(name),
            Expr::ArrayAccess { array, .. } | Expr::Grouping { expr: array, .. } => Self::base_variable_name(array),
            _ => None,
        }
    }

    fn visit_assign(&mut self, target: &Expr, value: &Expr, _span: &SourceSpan) -> Option<Type> {
        let target_type = match target {
            Expr::Variable { name, span } => {
                if let Some(var) = self.symbol_table.lookup_variable(name) {
                    if var.mutable {
                        Some(var.ty)
                    } else {
                        self.type_error_with_code(
                            Some(ErrorCode::E2024),
                            format!("Cannot assign to immutable variable '{name}'"),
                            span,
                        );
                        None
                    }
                } else {
                    self.type_error_with_code(Some(ErrorCode::E2025), format!("Undefined variable '{name}'"), span);
                    None
                }
            }
            Expr::ArrayAccess { array, index, span } => {
                let mut target_is_mutable = true;
                if let Some(name) = Self::base_variable_name(array) {
                    if let Some(var) = self.symbol_table.lookup_variable(name) {
                        if !var.mutable {
                            self.type_error_with_code(
                                Some(ErrorCode::E2024),
                                format!("Cannot assign to immutable variable '{name}'"),
                                span,
                            );
                            target_is_mutable = false;
                        }
                    }
                }
                let array_access_type = self.visit_array_access(array, index, span);
                if target_is_mutable { array_access_type } else { None }
            }
            _ => None,
        };
        let value_type = self.visit_expr(value);
        let (Some(target_type), Some(value_type)) = (target_type, value_type) else {
            return None;
        };
        if !self.is_assignable(&value_type, &target_type) {
            // Create specific error message for array elements
            let message = match target {
                Expr::ArrayAccess { .. } => {
                    format!("Cannot assign {value_type} to array element of type {target_type}")
                }
                _ => format!("Cannot assign {value_type} to {target_type}"),
            };
            self.type_error_with_code(Some(ErrorCode::E2002), message, value.span());
        }
        Some(target_type)
    }

    #[allow(clippy::manual_let_else)]
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr], span: &SourceSpan) -> Option<Type> {
        let callee_name = if let Expr::Variable { name, .. } = callee {
            name
        } else {
            self.type_error_with_code(Some(ErrorCode::E2026), "Callee must be a function name", callee.span());
            for arg in arguments {
                self.visit_expr(arg);
            }
            return None;
        };
        let Some(func) = self.symbol_table.lookup_function(callee_name) else {
            self.type_error_with_code(
                Some(ErrorCode::E2027),
                format!("Undefined function: '{callee_name}'"),
                callee.span(),
            );
            for arg in arguments {
                self.visit_expr(arg);
            }
            return None;
        };
        if arguments.len() != func.parameters.len() {
            self.type_error_with_code(
                Some(ErrorCode::E2028),
                format!(
                    "Function '{}' expects {} arguments, found {}",
                    callee_name,
                    func.parameters.len(),
                    arguments.len()
                ),
                span,
            );
        }
        for (i, (arg, param)) in arguments.iter().zip(&func.parameters).enumerate() {
            if let Some(arg_type) = self.visit_expr(arg) {
                if !self.is_assignable(&arg_type, &param.type_annotation) {
                    self.type_error_with_code(
                        Some(ErrorCode::E2029),
                        format!(
                            "Argument {} type mismatch: expected {}, found {}",
                            i + 1,
                            param.type_annotation,
                            arg_type
                        ),
                        arg.span(),
                    );
                }
            }
        }
        Some(func.return_type.clone())
    }

    fn visit_array_access(&mut self, array: &Expr, index: &Expr, _span: &SourceSpan) -> Option<Type> {
        let array_type = self.visit_expr(array);
        let index_type = self.visit_expr(index);
        let (Some(array_type), Some(index_type)) = (array_type, index_type) else {
            return None;
        };
        if !Self::is_integer_type(&index_type) {
            self.type_error_with_code(
                Some(ErrorCode::E2030),
                format!("Array index must be integer type, found {index_type}"),
                index.span(),
            );
            return None;
        }
        match array_type {
            Type::Array { element_type, .. } | Type::Vector { element_type } => Some(*element_type),
            other_type => {
                self.type_error_with_code(
                    Some(ErrorCode::E2031),
                    format!("Cannot index into non-array type {other_type}"),
                    array.span(),
                );
                None
            }
        }
    }

    // Funzione per la promozione automatica dei tipi numerici
    /// Selects a promoted type for a binary operation.
    ///
    /// Two numeric types produce the higher-ranked type. If only one input is in
    /// the numeric hierarchy, that type is returned; if neither is, `t1` is returned.
    ///
    /// # Panics
    ///
    /// Panics if the fallback promotion cache's mutex is poisoned.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn promote_numeric_types(&self, t1: &Type, t2: &Type) -> Type {
        if let (Some(rank1), Some(rank2)) = (Self::promotion_rank(t1), Self::promotion_rank(t2)) {
            return if rank1 >= rank2 { t1.clone() } else { t2.clone() };
        }

        // Preserve the existing path for non-numeric types and unsupported pairs.
        let cache = TYPE_PROMOTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        // Create key for cache lookup
        let key = (t1.clone(), t2.clone());

        #[allow(clippy::expect_used)]
        let mut cache_guard = cache.lock().expect("TYPE_PROMOTION_CACHE mutex poisoned");
        cache_guard.entry(key).or_insert_with(|| self.compute_promotion(t1, t2)).clone()
    }

    #[inline]
    fn promotion_rank(ty: &Type) -> Option<usize> {
        HIERARCHY.iter().position(|candidate| candidate == ty).map(|index| HIERARCHY.len() - index)
    }

    // Extract the original promotion logic into a separate function
    #[allow(clippy::unused_self)]
    fn compute_promotion(&self, t1: &Type, t2: &Type) -> Type {
        // Trova il tipo con rango più alto nella gerarchia
        for ty in &HIERARCHY {
            if t1 == ty || t2 == ty {
                return ty.clone();
            }
        }
        // This should never happen if HIERARCHY contains all numeric types
        // Return the first type as a fallback to maintain type safety
        t1.clone()
    }

    /// Returns whether a value of `source` type can be assigned to `target`.
    ///
    /// In addition to identical types, this accepts supported numeric widening,
    /// characters assigned to strings, null pointers assigned to arrays, vectors,
    /// or custom types, recursively assignable vectors, and arrays with assignable
    /// element types and equal evaluable lengths.
    #[inline]
    #[must_use]
    #[allow(clippy::unnested_or_patterns)]
    pub fn is_assignable(&self, source: &Type, target: &Type) -> bool {
        match (source, target) {
        // Numeric promotions
        (Type::I8, Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64)
        | (Type::I16, Type::I32 | Type::I64 | Type::F32 | Type::F64)
        | (Type::I32, Type::I64 | Type::F32 | Type::F64)
        | (Type::I64, Type::F64)
        | (Type::U8, Type::U16 | Type::U32 | Type::U64 | Type::F32 | Type::F64)
        | (Type::U16, Type::U32 | Type::U64 | Type::F32 | Type::F64)
        | (Type::U32, Type::U64 | Type::F32 | Type::F64)
        | (Type::U64, Type::F64)
        | (Type::F32, Type::F64)
        // Nullptr assignable to pointer types
        | (
            Type::NullPtr,
            Type::Array { .. } | Type::Vector { .. } | Type::Custom { .. },
        )
        // Char assignable to String
        | (Type::Char, Type::String) => true,
        // Array: requires compatible types and equal sizes
        (
            Type::Array {
                element_type: source_elem,
                size: source_size,
            },
            Type::Array {
                element_type: target_elem,
                size: target_size,
            },
        ) => {
            if !self.is_assignable(source_elem, target_elem) {
                return false;
            }

            match (self.get_size(source_size), self.get_size(target_size)) {
                (Some(source_val), Some(target_val)) => source_val == target_val,
                _ => false,
            }
        }

        (
            Type::Vector {
                element_type: source_elem,
            },
            Type::Vector {
                element_type: target_elem,
            },
        ) => self.is_assignable(source_elem, target_elem),

        // Identical types
        _ => source == target,
    }
    }

    /// Checks if a type is an integer type.
    const fn is_integer_type(ty: &Type) -> bool {
        matches!(ty, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::U8 | Type::U16 | Type::U32 | Type::U64)
    }

    /// Checks if two types are compatible (either can be assigned to the other).
    fn are_compatible(&self, t1: &Type, t2: &Type) -> bool {
        self.is_assignable(t1, t2) || self.is_assignable(t2, t1)
    }

    /// Checks if a type is numeric (integer or floating point).
    const fn is_numeric(ty: &Type) -> bool {
        Self::is_integer_type(ty) || matches!(ty, Type::F32 | Type::F64)
    }

    /// Returns whether a return appears in a statement, requiring both branches
    /// of a conditional to contain returns.
    #[inline]
    #[allow(clippy::only_used_in_recursion, clippy::self_only_used_in_recursion)]
    fn function_has_return(&self, body: &Stmt) -> bool {
        match body {
            Stmt::Return { .. } => true,

            Stmt::Block { statements, .. } => statements.iter().any(|stmt| self.function_has_return(stmt)),

            Stmt::While { body, .. } | Stmt::For { body, .. } => self.function_has_return(body),

            Stmt::If { then_branch, else_branch, .. } => {
                let then_has_return = self.function_has_return(then_branch);

                let else_has_return = match else_branch {
                    ElseBranch::None => false,
                    ElseBranch::Block(stmt) | ElseBranch::ElseIf(stmt) => self.function_has_return(stmt),
                };

                then_has_return && else_has_return
            }

            _ => false,
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::ast::unary_op_side::UnaryOpSide;

    fn variable(name: &str, span: SourceSpan) -> Expr {
        Expr::Variable { name: name.into(), span }
    }

    fn number(span: SourceSpan) -> Expr {
        Expr::new_number_literal(Number::Integer(1), span)
    }

    fn immutable_declarations(span: &SourceSpan) -> Vec<Stmt> {
        let array_type = Type::Array { element_type: Box::new(Type::I64), size: Box::new(number(span.clone())) };

        vec![
            Stmt::VarDeclaration {
                bindings: vec![VarBinding { name: "value".into(), initializer: Some(number(span.clone())) }],
                type_annotation: Type::I64,
                is_mutable: false,
                span: span.clone(),
            },
            Stmt::VarDeclaration {
                bindings: vec![VarBinding { name: "items".into(), initializer: None }],
                type_annotation: array_type,
                is_mutable: false,
                span: span.clone(),
            },
        ]
    }

    fn unary(op: UnaryOp, side: UnaryOpSide, expr: Expr, span: SourceSpan) -> Stmt {
        Stmt::Expression { expr: Box::new(Expr::Unary { op, side, expr: Box::new(expr), span }) }
    }

    #[test]
    fn function_return_inside_while_body_counts_as_return() {
        let span = SourceSpan::default();
        let statements = [Stmt::Function {
            name: "while_return".into(),
            parameters: vec![],
            return_type: Type::I64,
            body: Box::new(Stmt::While {
                condition: Box::new(Expr::new_bool_literal(true, span.clone())),
                body: Box::new(Stmt::Return {
                    value: Some(number(span.clone())),
                    span: span.clone(),
                }),
                span: span.clone(),
            }),
            span: span.clone(),
        }];

        let errors = TypeChecker::new().check(&statements);
        assert!(!errors.iter().any(|error| error.error_code() == Some(&ErrorCode::E2003)), "unexpected missing-return error: {errors:#?}");
    }

    #[test]
    fn function_return_inside_for_body_counts_as_return() {
        let span = SourceSpan::default();
        let statements = [Stmt::Function {
            name: "for_return".into(),
            parameters: vec![],
            return_type: Type::I64,
            body: Box::new(Stmt::For {
                initializer: None,
                condition: Some(Expr::new_bool_literal(true, span.clone())),
                increment: None,
                body: Box::new(Stmt::Return {
                    value: Some(number(span.clone())),
                    span: span.clone(),
                }),
                span: span.clone(),
            }),
            span: span.clone(),
        }];

        let errors = TypeChecker::new().check(&statements);
        assert!(!errors.iter().any(|error| error.error_code() == Some(&ErrorCode::E2003)), "unexpected missing-return error: {errors:#?}");
    }

    #[test]
    fn immutable_increment_and_decrement_reject_prefix_and_postfix_targets() {
        let span = SourceSpan::default();
        let array_access = || Expr::ArrayAccess {
            array: Box::new(variable("items", span.clone())),
            index: Box::new(number(span.clone())),
            span: span.clone(),
        };

        let cases = [
            (UnaryOp::Increment, UnaryOpSide::Prefix, variable("value", span.clone())),
            (UnaryOp::Increment, UnaryOpSide::Postfix, variable("value", span.clone())),
            (UnaryOp::Decrement, UnaryOpSide::Prefix, variable("value", span.clone())),
            (UnaryOp::Decrement, UnaryOpSide::Postfix, variable("value", span.clone())),
            (UnaryOp::Increment, UnaryOpSide::Prefix, array_access()),
            (UnaryOp::Increment, UnaryOpSide::Postfix, array_access()),
            (
                UnaryOp::Decrement,
                UnaryOpSide::Prefix,
                Expr::Grouping { expr: Box::new(array_access()), span: span.clone() },
            ),
            (
                UnaryOp::Decrement,
                UnaryOpSide::Postfix,
                Expr::Grouping { expr: Box::new(array_access()), span: span.clone() },
            ),
        ];

        for (op, side, target) in cases {
            let mut statements = immutable_declarations(&span);
            statements.push(unary(op, side, target, span.clone()));

            let errors = TypeChecker::new().check(&statements);

            assert_eq!(
                errors.iter().filter_map(CompileError::error_code).filter(|code| **code == ErrorCode::E2024).count(),
                1,
                "expected one E2024 for {op:?} {side:?}"
            );
            assert!(
                !errors.iter().any(|error| error.error_code() == Some(&ErrorCode::E2018)),
                "immutable mutation must be rejected with E2024 before numeric validation"
            );
        }
    }

    #[test]
    fn compound_operators_accept_mutable_targets() {
        let span = SourceSpan::default();
        let operators = [
            BinaryOp::AddEqual,
            BinaryOp::SubtractEqual,
            BinaryOp::MultiplyEqual,
            BinaryOp::DivideEqual,
            BinaryOp::ModuloEqual,
            BinaryOp::BitwiseAndEqual,
            BinaryOp::BitwiseOrEqual,
            BinaryOp::BitwiseXorEqual,
            BinaryOp::ShiftLeftEqual,
            BinaryOp::ShiftRightEqual,
        ];

        for op in operators {
            let statements = [
                Stmt::VarDeclaration {
                    bindings: vec![VarBinding { name: "value".into(), initializer: Some(number(span.clone())) }],
                    type_annotation: Type::I64,
                    is_mutable: true,
                    span: span.clone(),
                },
                Stmt::Expression {
                    expr: Box::new(Expr::Binary {
                        left: Box::new(variable("value", span.clone())),
                        op,
                        right: Box::new(number(span.clone())),
                        span: span.clone(),
                    }),
                },
            ];

            let errors = TypeChecker::new().check(&statements);
            assert!(errors.is_empty(), "unexpected errors for {op:?}: {errors:#?}");
        }
    }

    #[test]
    fn compound_operators_reject_immutable_targets_with_e2024() {
        let span = SourceSpan::default();
        let operators = [
            BinaryOp::AddEqual,
            BinaryOp::SubtractEqual,
            BinaryOp::MultiplyEqual,
            BinaryOp::DivideEqual,
            BinaryOp::ModuloEqual,
            BinaryOp::BitwiseAndEqual,
            BinaryOp::BitwiseOrEqual,
            BinaryOp::BitwiseXorEqual,
            BinaryOp::ShiftLeftEqual,
            BinaryOp::ShiftRightEqual,
        ];

        for op in operators {
            let statements = [
                Stmt::VarDeclaration {
                    bindings: vec![VarBinding { name: "value".into(), initializer: Some(number(span.clone())) }],
                    type_annotation: Type::I64,
                    is_mutable: false,
                    span: span.clone(),
                },
                Stmt::Expression {
                    expr: Box::new(Expr::Binary {
                        left: Box::new(variable("value", span.clone())),
                        op,
                        right: Box::new(number(span.clone())),
                        span: span.clone(),
                    }),
                },
            ];

            let errors = TypeChecker::new().check(&statements);
            assert_eq!(
                errors.iter().filter_map(CompileError::error_code).filter(|code| **code == ErrorCode::E2024).count(),
                1,
                "expected one E2024 for {op:?}: {errors:#?}"
            );
        }
    }

    #[test]
    fn compound_arithmetic_and_bitwise_operations_promote_numeric_operands() {
        let span = SourceSpan::default();
        let declarations = [Stmt::VarDeclaration {
            bindings: vec![VarBinding {
                name: "value".into(),
                initializer: Some(Expr::new_number_literal(Number::I8(1), span.clone())),
            }],
            type_annotation: Type::I8,
            is_mutable: true,
            span: span.clone(),
        }];

        let mut checker = TypeChecker::new();
        checker.visit_statements(&declarations);

        let arithmetic = checker.visit_binary_expr(
            &variable("value", span.clone()),
            BinaryOp::AddEqual,
            &Expr::new_number_literal(Number::Integer(1), span.clone()),
            &span,
        );
        assert_eq!(arithmetic, Some(Type::I64));

        let bitwise = checker.visit_binary_expr(
            &variable("value", span.clone()),
            BinaryOp::BitwiseAndEqual,
            &Expr::new_number_literal(Number::Integer(1), span.clone()),
            &span,
        );
        assert_eq!(bitwise, Some(Type::I64));
        assert!(checker.errors.is_empty(), "unexpected compound operation errors: {:#?}", checker.errors);
    }

    #[test]
    fn mutable_non_numeric_increment_still_uses_e2018() {
        let span = SourceSpan::default();
        let statements = [
            Stmt::VarDeclaration {
                bindings: vec![VarBinding {
                    name: "flag".into(),
                    initializer: Some(Expr::new_bool_literal(true, span.clone())),
                }],
                type_annotation: Type::Bool,
                is_mutable: true,
                span: span.clone(),
            },
            unary(UnaryOp::Increment, UnaryOpSide::Prefix, variable("flag", span.clone()), span),
        ];

        let errors = TypeChecker::new().check(&statements);

        assert!(errors.iter().any(|error| error.error_code() == Some(&ErrorCode::E2018)));
        assert!(!errors.iter().any(|error| error.error_code() == Some(&ErrorCode::E2024)));
    }
}
