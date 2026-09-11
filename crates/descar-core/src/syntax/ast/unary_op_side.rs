/// Represents the side on which a unary operator is applied
/// relative to its operand.
///
/// A unary operator can either precede (prefix) or follow
/// (postfix) its operand.
///
/// Examples:
///
/// - `Prefix` for `-x`, `!flag`, `++i`.
/// - `Postfix` for `i++`, `i--`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOpSide {
    /// The operator appears before the operand.
    Prefix,

    /// The operator appears after the operand.
    Postfix,
}
