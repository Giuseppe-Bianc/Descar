// crates/descar-core/src/syntax/ast/stmt.rs

use crate::{
    location::source_span::SourceSpan,
    syntax::ast::{else_branch::ElseBranch, expr::Expr, parameter::Parameter, type_syntax::TypeSyntax},
};

/// Abstract syntax tree node representing a statement.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Stmt {
    /// Expression statement.
    Expression {
        /// Wrapped expression.
        expr: Box<Expr>,
    },

    /// Variable declaration statement.
    VarDeclaration {
        /// Variable bindings declared by the statement.
        bindings: Vec<VarBinding>,

        /// Type annotation for declared variables.
        type_annotation: TypeSyntax,

        /// Whether the variables are mutable. The value is `true` for `var` and `false` for `const`.
        is_mutable: bool,

        /// Source extent.
        span: SourceSpan,
    },

    /// Function declaration statement.
    Function {
        /// Function name.
        name: String,

        /// Function parameter list.
        parameters: Vec<Parameter>,

        /// Declared return type. The value is `TypeSyntax::Void` if the function has no return type.
        return_type: TypeSyntax,

        /// Function body, a block.
        body: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Conditional if statement.
    If {
        /// Boolean condition expression.
        condition: Box<Expr>,

        /// Block that runs if the condition is true.
        then_branch: Box<Self>,

        /// Else branch.
        else_branch: ElseBranch,

        /// Source extent.
        span: SourceSpan,
    },

    /// While loop statement.
    While {
        /// Loop condition.
        condition: Box<Expr>,

        /// Loop body.
        body: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// For loop statement.
    For {
        /// Optional loop initialization statement: a variable declaration or an expression statement.
        initializer: Option<Box<Self>>,

        /// Optional loop condition.
        condition: Option<Expr>,

        /// Optional loop increment expression.
        increment: Option<Expr>,

        /// Loop body, one statement.
        body: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Block statement enclosing zero or more statements.
    Block {
        /// Statements contained in the block.
        statements: Vec<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Return statement.
    Return {
        /// Optional return value.
        value: Option<Expr>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Break statement.
    Break {
        /// Source extent.
        span: SourceSpan,
    },

    /// Continue statement.
    Continue {
        /// Source extent.
        span: SourceSpan,
    },

    /// Main function statement.
    MainFunction {
        /// Main function body, a block.
        body: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },
}

/// Pairs a variable name with an optional initializer expression.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct VarBinding {
    /// Variable name.
    pub name: String,

    /// Optional initializer expression.
    pub initializer: Option<Expr>,

    /// Source extent of this binding, including its initializer when present.
    pub span: SourceSpan,
}

impl Stmt {
    /// Returns the source span of this statement.
    ///
    /// For an expression statement, the span is the span of the expression.
    #[must_use]
    pub const fn span(&self) -> &SourceSpan {
        match self {
            Self::Expression { expr } => expr.span(),

            Self::VarDeclaration { span, .. }
            | Self::Function { span, .. }
            | Self::If { span, .. }
            | Self::While { span, .. }
            | Self::For { span, .. }
            | Self::Block { span, .. }
            | Self::Return { span, .. }
            | Self::Break { span }
            | Self::Continue { span }
            | Self::MainFunction { span, .. } => span,
        }
    }
}
