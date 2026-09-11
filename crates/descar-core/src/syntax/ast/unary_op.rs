/// Unary operators supported in syntax expressions.
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
