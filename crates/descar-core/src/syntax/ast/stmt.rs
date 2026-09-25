use crate::{
    location::source_span::SourceSpan,
    syntax::ast::{ast_type::Type, else_branch::ElseBranch, expr::Expr, parameter::Parameter},
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Stmt {
    Expression { expr: Box<Expr> },
    VarDeclaration {
        bindings: Vec<VarBinding>,
        type_annotation: Type,
        is_mutable: bool,
        span: SourceSpan,
    },
    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Type,
        body: Box<Self>,
        span: SourceSpan,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Self>,
        else_branch: ElseBranch,
        span: SourceSpan,
    },
    While { condition: Box<Expr>, body: Box<Self>, span: SourceSpan },
    For {
        initializer: Option<Box<Self>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Self>,
        span: SourceSpan,
    },
    Block { statements: Vec<Self>, span: SourceSpan },
    Return { value: Option<Expr>, span: SourceSpan },
    Break { span: SourceSpan },
    Continue { span: SourceSpan },
    MainFunction { body: Box<Self>, span: SourceSpan },
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct VarBinding {
    pub name: String,
    pub initializer: Option<Expr>,
    pub ty: Option<Type>,
}

impl VarBinding {
    #[must_use]
    pub const fn new(name: String, initializer: Option<Expr>) -> Self {
        Self { name, initializer, ty: None }
    }

    #[must_use]
    pub fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

impl Stmt {
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
