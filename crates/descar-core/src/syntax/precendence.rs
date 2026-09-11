use crate::tokens::{token::Token, token_kind::TokenKind};

/// Defines operator binding powers (precedence and associativity)
/// used by the Pratt-style expression parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precedence {
    pub left: u8,
    pub right: u8,
}

impl Precedence {
    /// Returns the left/right binding powers for the given binary/infix
    /// operator token.
    ///
    /// The pair is used by the Pratt parser to determine when to stop
    /// or continue parsing an expression.
    #[must_use]
    pub const fn binding_power(token: &Token) -> Self {
        match token.kind {
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

            TokenKind::OrOr => Self::new(4, 3),

            TokenKind::AndAnd => Self::new(6, 5),

            TokenKind::EqualEqual | TokenKind::NotEqual => Self::new(8, 7),

            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => Self::new(10, 9),

            TokenKind::Or => Self::new(12, 11),

            TokenKind::Xor => Self::new(14, 13),

            TokenKind::And => Self::new(16, 15),

            TokenKind::ShiftLeft | TokenKind::ShiftRight => Self::new(18, 17),

            TokenKind::Plus | TokenKind::Minus => Self::new(20, 19),

            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Self::new(22, 21),

            TokenKind::OpenParen
            | TokenKind::OpenBracket
            | TokenKind::Dot
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus => Self::new(27, 26),

            _ => Self::new(0, 0),
        }
    }

    /// Returns the binding power for the given unary/prefix operator token.
    ///
    /// The returned value determines how tightly the operand binds
    /// to the prefix operator.
    #[must_use]
    pub const fn unary_binding_power(token: &Token) -> Self {
        match token.kind {
            TokenKind::Not | TokenKind::Minus | TokenKind::BitwiseNot | TokenKind::PlusPlus | TokenKind::MinusMinus => {
                Self::new(24, 23)
            }
            _ => Self::new(0, 0),
        }
    }

    #[inline]
    const fn new(left: u8, right: u8) -> Self {
        Self { left, right }
    }
}
