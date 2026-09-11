use crate::location::source_span::SourceSpan;
use crate::syntax::ast::binary_op::BinaryOp;
use crate::syntax::ast::literal_value::LiteralValue;
use crate::syntax::ast::unary_op::UnaryOp;
use crate::syntax::ast::unary_op_side::UnaryOpSide;
use crate::tokens::number::Number;

/// Abstract syntax tree node representing an expression.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Expr {
    /// Binary operation expression.
    Binary {
        /// Left operand.
        left: Box<Self>,

        /// Binary operator.
        op: BinaryOp,

        /// Right operand.
        right: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Unary operation expression.
    Unary {
        /// Unary operator.
        op: UnaryOp,

        /// Whether the operator is prefix or postfix.
        side: UnaryOpSide,

        /// Operand expression.
        expr: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Parenthesized or grouped expression.
    Grouping {
        /// Inner expression.
        expr: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Literal value expression.
    Literal {
        /// Literal value.
        value: LiteralValue,

        /// Source extent.
        span: SourceSpan,
    },

    /// Array literal expression containing zero or more elements.
    ArrayLiteral {
        /// Element expressions.
        elements: Vec<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Variable reference expression.
    Variable {
        /// Identifier name.
        name: String,

        /// Source extent.
        span: SourceSpan,
    },

    /// Assignment expression.
    Assign {
        /// Assignment target.
        target: Box<Self>,

        /// Right-hand side expression.
        value: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Function or method call expression.
    Call {
        /// Expression producing the callable value.
        callee: Box<Self>,

        /// Arguments passed to the call.
        arguments: Vec<Self>,

        /// Source extent.
        span: SourceSpan,
    },

    /// Array indexing expression.
    ArrayAccess {
        /// Expression producing the indexed array.
        array: Box<Self>,

        /// Expression producing the element index.
        index: Box<Self>,

        /// Source extent.
        span: SourceSpan,
    },
}

impl Expr {
    /// Returns the source span for this expression.
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

    /// Creates a null pointer literal expression.
    #[must_use]
    pub const fn null_expr(span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::NullPtr, span }
    }

    /// Creates a numeric literal expression.
    #[must_use]
    pub const fn new_number_literal(value: Number, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::Numeric(value), span }
    }

    /// Creates a boolean literal expression.
    #[must_use]
    pub const fn new_bool_literal(value: bool, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::Bool(value), span }
    }

    /// Creates a null pointer literal expression.
    #[must_use]
    pub const fn new_nullptr_literal(span: SourceSpan) -> Self {
        Self::null_expr(span)
    }

    /// Creates a string literal expression.
    #[must_use]
    pub const fn new_string_literal(value: String, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::StringLit(value), span }
    }

    /// Creates a character literal expression.
    #[must_use]
    pub const fn new_char_literal(value: String, span: SourceSpan) -> Self {
        Self::Literal { value: LiteralValue::CharLit(value), span }
    }
}
