//use std::fmt;
use std::sync::Arc;

use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::tokens::token::Token;
use crate::tokens::token_kind::TokenKind;

/// Binary operators supported in syntax expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Addition operator (+).
    Add,

    /// Addition assignment operator (+=).
    AddEqual,

    /// Subtraction operator (-).
    Subtract,

    /// Subtraction assignment operator (-=).
    SubtractEqual,

    /// Multiplication operator (*).
    Multiply,

    /// Multiplication assignment operator (*=).
    MultiplyEqual,

    /// Division operator (/).
    Divide,

    /// Division assignment operator (/=).
    DivideEqual,

    /// Modulo operator (%).
    Modulo,

    /// Modulo assignment operator (%=).
    ModuloEqual,

    /// Equality comparison operator (==).
    Equal,

    /// Inequality comparison operator (!=).
    NotEqual,

    /// Less-than comparison operator (<).
    Less,

    /// Less-than-or-equal comparison operator (<=).
    LessEqual,

    /// Greater-than comparison operator (>).
    Greater,

    /// Greater-than-or-equal comparison operator (>=).
    GreaterEqual,

    /// Logical AND operator (&&).
    And,

    /// Logical OR operator (||).
    Or,

    /// Bitwise AND operator (&).
    BitwiseAnd,

    /// Bitwise AND assignment operator (&=).
    BitwiseAndEqual,

    /// Bitwise OR operator (|).
    BitwiseOr,

    /// Bitwise OR assignment operator (|=).
    BitwiseOrEqual,

    /// Bitwise XOR operator (^).
    BitwiseXor,

    /// Bitwise XOR assignment operator (^=).
    BitwiseXorEqual,

    /// Shift left operator (<<).
    ShiftLeft,

    /// Shift left assignment operator (<<=).
    ShiftLeftEqual,

    /// Shift right operator (>>).
    ShiftRight,

    /// Shift right assignment operator (>>=).
    ShiftRightEqual,
}

impl BinaryOp {
    /// Converts a token into its corresponding binary operator.
    ///
    /// # Errors
    ///
    /// Returns a `CompileError::SyntaxError` if the token kind is not
    /// a valid binary operator.
    #[allow(clippy::result_large_err)]
    pub fn get_op(token: &Token) -> Result<Self, CompileError> {
        Ok(match token.kind {
            TokenKind::Plus => Self::Add,
            TokenKind::PlusEqual => Self::AddEqual,

            TokenKind::Minus => Self::Subtract,
            TokenKind::MinusEqual => Self::SubtractEqual,

            TokenKind::Star => Self::Multiply,
            TokenKind::StarEqual => Self::MultiplyEqual,

            TokenKind::Slash => Self::Divide,
            TokenKind::SlashEqual => Self::DivideEqual,

            TokenKind::Percent => Self::Modulo,
            TokenKind::PercentEqual => Self::ModuloEqual,

            TokenKind::EqualEqual => Self::Equal,
            TokenKind::NotEqual => Self::NotEqual,

            TokenKind::Less => Self::Less,
            TokenKind::LessEqual => Self::LessEqual,

            TokenKind::Greater => Self::Greater,
            TokenKind::GreaterEqual => Self::GreaterEqual,

            TokenKind::AndAnd => Self::And,
            TokenKind::OrOr => Self::Or,

            TokenKind::And => Self::BitwiseAnd,
            TokenKind::AndEqual => Self::BitwiseAndEqual,

            TokenKind::Or => Self::BitwiseOr,
            TokenKind::OrEqual => Self::BitwiseOrEqual,

            TokenKind::Xor => Self::BitwiseXor,
            TokenKind::XorEqual => Self::BitwiseXorEqual,

            TokenKind::ShiftLeft => Self::ShiftLeft,
            TokenKind::ShiftLeftEqual => Self::ShiftLeftEqual,

            TokenKind::ShiftRight => Self::ShiftRight,
            TokenKind::ShiftRightEqual => Self::ShiftRightEqual,

            _ => {
                return Err(CompileError::SyntaxError {
                    code: Some(ErrorCode::E1005),
                    message: Arc::from(format!("Invalid binary operator: {:?}", token.kind)),
                    span: token.span.clone(),
                    help: None,
                });
            }
        })
    }
}
