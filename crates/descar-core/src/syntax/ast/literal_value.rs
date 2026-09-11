use crate::tokens::number::Number;

/// Value of a literal expression.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum LiteralValue {
    /// Numeric literal value.
    Numeric(Number),

    /// String literal value.
    StringLit(String),

    /// Character literal value.
    CharLit(String),

    /// Boolean literal value.
    Bool(bool),

    /// Null pointer literal value.
    NullPtr,
}
