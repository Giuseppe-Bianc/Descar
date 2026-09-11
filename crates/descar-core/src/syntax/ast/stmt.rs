use crate::{
    location::source_span::SourceSpan,
    syntax::ast::{ast_type::Type, else_branch::ElseBranch, expr::Expr, parameter::Parameter},
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
        type_annotation: Type,

        /// Whether the variables are mutable.
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

        /// Declared return type.
        return_type: Type,

        /// Function body.
        body: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Conditional if statement.
    If {
        /// Boolean condition expression.
        condition: Box<Expr>,

        /// Statements executed if the condition is true.
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
        /// Optional loop initialization statement.
        initializer: Option<Box<Self>>,

        /// Optional loop condition.
        condition: Option<Expr>,

        /// Optional loop increment expression.
        increment: Option<Expr>,

        /// Loop body.
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
        /// Main function body.
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
}

impl Stmt {
    /// Returns the source span for this statement.
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
