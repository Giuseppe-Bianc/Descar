// crates/descar-core/src/syntax/ast/unary_op.rs

/// Unary operators supported in syntax expressions.
///
/// The operators `Increment` and `Decrement` can be prefix or postfix. See `UnaryOpSide`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// Negation operator (-).
    Negate,

    /// Logical NOT operator (!).
    Not,

    /// Bitwise complement operator (~).
    BitwiseNot,

    /// Increment operator (++).
    Increment,

    /// Decrement operator (--).
    Decrement,
}
