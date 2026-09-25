use crate::location::source_span::SourceSpan;
use crate::syntax::ast::binary_op::BinaryOp;
use crate::syntax::ast::literal_value::LiteralValue;
use crate::syntax::ast::unary_op::UnaryOp;
use crate::syntax::ast::unary_op_side::UnaryOpSide;
use crate::tokens::number::Number;
use crate::syntax::ast::Type;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Expr {
    Binary { left: Box<Self>, op: BinaryOp, right: Box<Self>, span: SourceSpan, ty: Option<Type> },
    Unary { op: UnaryOp, side: UnaryOpSide, expr: Box<Self>, span: SourceSpan, ty: Option<Type> },
    Grouping { expr: Box<Self>, span: SourceSpan, ty: Option<Type> },
    Literal { value: LiteralValue, span: SourceSpan, ty: Option<Type> },
    ArrayLiteral { elements: Vec<Self>, span: SourceSpan, ty: Option<Type> },
    Variable { name: String, span: SourceSpan, ty: Option<Type> },
    Assign { target: Box<Self>, value: Box<Self>, span: SourceSpan, ty: Option<Type> },
    Call { callee: Box<Self>, arguments: Vec<Self>, span: SourceSpan, ty: Option<Type> },
    ArrayAccess { array: Box<Self>, index: Box<Self>, span: SourceSpan, ty: Option<Type> },
}

impl Expr {
    #[must_use]
    pub const fn span(&self) -> &SourceSpan {
        match self {
            Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Grouping { span, .. }
            | Self::Literal { span, .. }
            | Self::ArrayLiteral { span, .. }
            | Self::Variable { span, .. }
            | Self::Assign { span, .. }
            | Self::Call { span, .. }
            | Self::ArrayAccess { span, .. } => span,
        }
    }

    #[must_use]
    pub fn ty(&self) -> Option<&Type> {
        match self {
            Self::Binary { ty, .. }
            | Self::Unary { ty, .. }
            | Self::Grouping { ty, .. }
            | Self::Literal { ty, .. }
            | Self::ArrayLiteral { ty, .. }
            | Self::Variable { ty, .. }
            | Self::Assign { ty, .. }
            | Self::Call { ty, .. }
            | Self::ArrayAccess { ty, .. } => ty.as_ref(),
        }
    }

    pub(crate) fn set_ty(&mut self, ty: Type) {
        match self {
            Self::Binary { ty: slot, .. }
            | Self::Unary { ty: slot, .. }
            | Self::Grouping { ty: slot, .. }
            | Self::Literal { ty: slot, .. }
            | Self::ArrayLiteral { ty: slot, .. }
            | Self::Variable { ty: slot, .. }
            | Self::Assign { ty: slot, .. }
            | Self::Call { ty: slot, .. }
            | Self::ArrayAccess { ty: slot, .. } => *slot = Some(ty),
        }
    }

    #[must_use]
    pub const fn null_expr(span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::NullPtr, span, ty: None }
    }

    #[must_use]
    pub const fn new_number_literal(value: Number, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::Numeric(value), span, ty: None }
    }

    #[must_use]
    pub const fn new_bool_literal(value: bool, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::Bool(value), span, ty: None }
    }

    #[must_use]
    pub const fn new_nullptr_literal(span: SourceSpan) -> Self {
        Self::null_expr(span)
    }

    #[must_use]
    pub const fn new_string_literal(value: String, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::StringLit(value), span, ty: None }
    }

    #[must_use]
    pub const fn new_char_literal(value: String, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::CharLit(value), span, ty: None }
    }
}
