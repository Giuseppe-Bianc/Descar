// crates/descar-core/src/syntax/precendence.rs

use crate::tokens::{token::Token, token_kind::TokenKind};

/// The binding powers of an operator.
///
/// The expression parser compares binding powers to decide the precedence and the associativity of
/// operators. A higher number binds more tightly. The parser uses the left value to decide if an
/// operator extends the expression on its left. It uses the right value as the minimum binding
/// power for the expression on the right of the operator.
///
/// If the right value is less than the left value, the operator associates to the right. If both
/// values are equal, the operator associates to the left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precedence {
    /// The binding power that the operator has toward the expression on its left.
    pub left: u8,
    /// The binding power that the operator has toward the expression on its right.
    pub right: u8,
}

impl Precedence {
    /// Returns the binding powers of an infix or postfix operator token.
    ///
    /// The binding powers are (left, right):
    ///
    /// | Operators | Binding powers |
    /// |-----------|----------------|
    /// | `=` and the compound assignment operators | (2, 1) |
    /// | `\|\|` | (4, 4) |
    /// | `&&` | (6, 6) |
    /// | `==` `!=` | (8, 8) |
    /// | `<` `<=` `>` `>=` | (10, 10) |
    /// | `\|` | (12, 12) |
    /// | `^` | (14, 14) |
    /// | `&` | (16, 16) |
    /// | `<<` `>>` | (18, 18) |
    /// | `+` `-` | (20, 20) |
    /// | `*` `/` `%` | (22, 22) |
    /// | `as` | (23, 24) |
    /// | `(` `[` `.` `++` `--` | (27, 26) |
    ///
    /// # Arguments
    ///
    /// * `token` - The operator token.
    ///
    /// # Returns
    ///
    /// The binding powers of the token. The result is (0, 0) for a token that is not in the table.
    /// The parser stops at such a token.
    ///
    /// # Notes
    ///
    /// The parser handles `(`, `[`, `++`, and `--` as postfix operators. The parser does not handle
    /// `.`. It reports `.` as an unexpected operator.
    #[must_use]
    pub const fn binding_power(token: &Token) -> Self {
        match token.kind {
            // Assignment associates to the right: a = (b = c).
            TokenKind::Equal
            | TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::AndEqual
            | TokenKind::OrEqual
            | TokenKind::PercentEqual
            | TokenKind::XorEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::ShiftLeftEqual
            | TokenKind::ShiftRightEqual => Self::new(2, 1),

            // Binary operators associate to the left: a op b op c parses as (a op b) op c.
            TokenKind::OrOr => Self::new(4, 4),

            TokenKind::AndAnd => Self::new(6, 6),

            TokenKind::EqualEqual | TokenKind::NotEqual => Self::new(8, 8),

            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => Self::new(10, 10),

            TokenKind::Or => Self::new(12, 12),

            TokenKind::Xor => Self::new(14, 14),

            TokenKind::And => Self::new(16, 16),

            TokenKind::ShiftLeft | TokenKind::ShiftRight => Self::new(18, 18),

            TokenKind::Plus | TokenKind::Minus => Self::new(20, 20),

            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Self::new(22, 22),

            TokenKind::OpenParen
            | TokenKind::OpenBracket
            | TokenKind::Dot
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus => Self::new(27, 26),

            // Casts bind more tightly than arithmetic and less tightly than unary operators.
            TokenKind::KeywordAs => Self::new(23, 24),

            _ => Self::new(0, 0),
        }
    }

    /// Returns the binding powers of a prefix operator token.
    ///
    /// The prefix operators are `!`, `-`, `~`, `++`, and `--`. Their binding powers are (24, 23). The
    /// parser uses the right value as the minimum binding power for the operand. Because the right
    /// value is 23, an operand does not include a cast. For example, `-a as i32` is `(-a) as i32`.
    ///
    /// # Arguments
    ///
    /// * `token` - The operator token.
    ///
    /// # Returns
    ///
    /// The binding powers of the token. The result is (0, 0) for any other token.
    #[must_use]
    pub const fn unary_binding_power(token: &Token) -> Self {
        match token.kind {
            TokenKind::Not | TokenKind::Minus | TokenKind::BitwiseNot | TokenKind::PlusPlus | TokenKind::MinusMinus => {
                Self::new(24, 23)
            }
            _ => Self::new(0, 0),
        }
    }

    /// Creates a pair of binding powers.
    #[inline]
    const fn new(left: u8, right: u8) -> Self {
        Self { left, right }
    }
}
