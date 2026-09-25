use crate::location::source_span::SourceSpan;
use crate::syntax::ast::{BinaryOp, ElseBranch, Expr, LiteralValue, Parameter, Stmt, Type, UnaryOp, UnaryOpSide};

/// The semantic type attached to a node in a fully typed AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedType {
    /// A value with one of the language's regular types.
    Value(Type),
    /// A callable function signature.
    Function { parameters: Vec<Type>, return_type: Type },
}

impl ResolvedType {
    #[must_use]
    pub fn value(ty: Type) -> Self {
        Self::Value(ty)
    }

    #[must_use]
    pub fn function(parameters: Vec<Type>, return_type: Type) -> Self {
        Self::Function { parameters, return_type }
    }

    #[must_use]
    pub fn return_type(&self) -> &Type {
        match self {
            Self::Value(ty) => ty,
            Self::Function { return_type, .. } => return_type,
        }
    }
}

/// A syntax expression paired with the type resolved by semantic analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: ResolvedType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedExprKind {
    Binary { left: Box<TypedExpr>, op: BinaryOp, right: Box<TypedExpr> },
    Unary { op: UnaryOp, side: UnaryOpSide, expr: Box<TypedExpr> },
    Grouping { expr: Box<TypedExpr> },
    Literal { value: LiteralValue },
    ArrayLiteral { elements: Vec<TypedExpr> },
    Variable { name: String },
    Assign { target: Box<TypedExpr>, value: Box<TypedExpr> },
    Call { callee: Box<TypedExpr>, arguments: Vec<TypedExpr> },
    ArrayAccess { array: Box<TypedExpr>, index: Box<TypedExpr> },
}

impl TypedExpr {
    #[must_use]
    pub fn from_expr(expr: &Expr, types: &std::collections::HashMap<usize, ResolvedType>) -> Result<Self, String> {
        let ty = types.get(&(std::ptr::from_ref(expr) as usize))
            .cloned()
            .ok_or_else(|| format!("missing resolved type for expression at {}", expr.span()))?;

        let kind = match expr {
            Expr::Binary { left, op, right, .. } => TypedExprKind::Binary {
                left: Box::new(Self::from_expr(left, types)?),
                op: *op,
                right: Box::new(Self::from_expr(right, types)?),
            },
            Expr::Unary { op, side, expr, .. } => TypedExprKind::Unary {
                op: *op,
                side: *side,
                expr: Box::new(Self::from_expr(expr, types)?),
            },
            Expr::Grouping { expr, .. } => TypedExprKind::Grouping {
                expr: Box::new(Self::from_expr(expr, types)?),
            },
            Expr::Literal { value, .. } => TypedExprKind::Literal { value: value.clone() },
            Expr::ArrayLiteral { elements, .. } => TypedExprKind::ArrayLiteral {
                elements: elements.iter().map(|e| Self::from_expr(e, types)).collect::<Result<_, _>>()?,
            },
            Expr::Variable { name, .. } => TypedExprKind::Variable { name: name.clone() },
            Expr::Assign { target, value, .. } => TypedExprKind::Assign {
                target: Box::new(Self::from_expr(target, types)?),
                value: Box::new(Self::from_expr(value, types)?),
            },
            Expr::Call { callee, arguments, .. } => TypedExprKind::Call {
                callee: Box::new(Self::from_expr(callee, types)?),
                arguments: arguments.iter().map(|e| Self::from_expr(e, types)).collect::<Result<_, _>>()?,
            },
            Expr::ArrayAccess { array, index, .. } => TypedExprKind::ArrayAccess {
                array: Box::new(Self::from_expr(array, types)?),
                index: Box::new(Self::from_expr(index, types)?),
            },
        };

        Ok(Self { kind, ty, span: expr.span().clone() })
    }
}

/// A declaration whose initializer and declared type have been semantically resolved.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedVarBinding {
    pub name: String,
    pub initializer: Option<TypedExpr>,
    pub declared_type: Type,
}

/// Fully typed statement node. Its hierarchy is isomorphic to the parser AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedStmt {
    Expression { expr: TypedExpr },
    VarDeclaration { bindings: Vec<TypedVarBinding>, type_annotation: Type, is_mutable: bool, span: SourceSpan },
    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Type,
        body: Box<TypedStmt>,
        span: SourceSpan,
    },
    If {
        condition: TypedExpr,
        then_branch: Box<TypedStmt>,
        else_branch: TypedElseBranch,
        span: SourceSpan,
    },
    While { condition: TypedExpr, body: Box<TypedStmt>, span: SourceSpan },
    For {
        initializer: Option<Box<TypedStmt>>,
        condition: Option<TypedExpr>,
        increment: Option<TypedExpr>,
        body: Box<TypedStmt>,
        span: SourceSpan,
    },
    Block { statements: Vec<TypedStmt>, span: SourceSpan },
    Return { value: Option<TypedExpr>, span: SourceSpan },
    Break { span: SourceSpan },
    Continue { span: SourceSpan },
    MainFunction { body: Box<TypedStmt>, span: SourceSpan },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedElseBranch {
    None,
    Block(Box<TypedStmt>),
    ElseIf(Box<TypedStmt>),
}

/// Result of successful static typing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullyTypedAst {
    pub statements: Vec<TypedStmt>,
}

impl FullyTypedAst {
    #[must_use]
    pub fn from_statements(
        statements: &[Stmt],
        types: &std::collections::HashMap<usize, ResolvedType>,
    ) -> Result<Self, String> {
        Ok(Self { statements: statements.iter().map(|s| TypedStmt::from_stmt(s, types)).collect::<Result<_, _>>()? })
    }
}

impl TypedStmt {
    fn from_stmt(stmt: &Stmt, types: &std::collections::HashMap<usize, ResolvedType>) -> Result<Self, String> {
        Ok(match stmt {
            Stmt::Expression { expr } => Self::Expression { expr: TypedExpr::from_expr(expr, types)? },
            Stmt::VarDeclaration { bindings, type_annotation, is_mutable, span } => Self::VarDeclaration {
                bindings: bindings.iter().map(|b| Ok(TypedVarBinding {
                    name: b.name.clone(),
                    initializer: b.initializer.as_ref().map(|e| TypedExpr::from_expr(e, types)).transpose()?,
                    declared_type: type_annotation.clone(),
                })).collect::<Result<_, String>>()?,
                type_annotation: type_annotation.clone(),
                is_mutable: *is_mutable,
                span: span.clone(),
            },
            Stmt::Function { name, parameters, return_type, body, span } => Self::Function {
                name: name.clone(),
                parameters: parameters.clone(),
                return_type: return_type.clone(),
                body: Box::new(Self::from_stmt(body, types)?),
                span: span.clone(),
            },
            Stmt::If { condition, then_branch, else_branch, span } => Self::If {
                condition: TypedExpr::from_expr(condition, types)?,
                then_branch: Box::new(Self::from_stmt(then_branch, types)?),
                else_branch: TypedElseBranch::from_else(else_branch, types)?,
                span: span.clone(),
            },
            Stmt::While { condition, body, span } => Self::While {
                condition: TypedExpr::from_expr(condition, types)?,
                body: Box::new(Self::from_stmt(body, types)?),
                span: span.clone(),
            },
            Stmt::For { initializer, condition, increment, body, span } => Self::For {
                initializer: initializer.as_deref().map(|s| Self::from_stmt(s, types)).transpose()?.map(Box::new),
                condition: condition.as_ref().map(|e| TypedExpr::from_expr(e, types)).transpose()?,
                increment: increment.as_ref().map(|e| TypedExpr::from_expr(e, types)).transpose()?,
                body: Box::new(Self::from_stmt(body, types)?),
                span: span.clone(),
            },
            Stmt::Block { statements, span } => Self::Block {
                statements: statements.iter().map(|s| Self::from_stmt(s, types)).collect::<Result<_, _>>()?,
                span: span.clone(),
            },
            Stmt::Return { value, span } => Self::Return {
                value: value.as_ref().map(|e| TypedExpr::from_expr(e, types)).transpose()?,
                span: span.clone(),
            },
            Stmt::Break { span } => Self::Break { span: span.clone() },
            Stmt::Continue { span } => Self::Continue { span: span.clone() },
            Stmt::MainFunction { body, span } => Self::MainFunction {
                body: Box::new(Self::from_stmt(body, types)?),
                span: span.clone(),
            },
        })
    }
}

impl TypedElseBranch {
    fn from_else(
        branch: &ElseBranch,
        types: &std::collections::HashMap<usize, ResolvedType>,
    ) -> Result<Self, String> {
        Ok(match branch {
            ElseBranch::None => Self::None,
            ElseBranch::Block(stmt) => Self::Block(Box::new(TypedStmt::from_stmt(stmt, types)?)),
            ElseBranch::ElseIf(stmt) => Self::ElseIf(Box::new(TypedStmt::from_stmt(stmt, types)?)),
        })
    }
}
