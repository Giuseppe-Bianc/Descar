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
    /// Subtraction operator (-).
    Subtract,
    /// Multiplication operator (*).
    Multiply,
    /// Division operator (/).
    Divide,
    /// Modulo operator (%).
    Modulo,
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
    /// Bitwise OR operator (|).
    BitwiseOr,
    /// Bitwise XOR operator (^).
    BitwiseXor,
    /// Shift left operator (<<).
    ShiftLeft,
    /// Shift right operator (>>).
    ShiftRight,
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
            TokenKind::Minus => Self::Subtract,
            TokenKind::Star => Self::Multiply,
            TokenKind::Slash => Self::Divide,
            TokenKind::Percent => Self::Modulo,
            TokenKind::EqualEqual => Self::Equal,
            TokenKind::NotEqual => Self::NotEqual,
            TokenKind::Less => Self::Less,
            TokenKind::LessEqual => Self::LessEqual,
            TokenKind::Greater => Self::Greater,
            TokenKind::GreaterEqual => Self::GreaterEqual,
            TokenKind::AndAnd => Self::And,
            TokenKind::OrOr => Self::Or,
            TokenKind::And => Self::BitwiseAnd,
            TokenKind::Or => Self::BitwiseOr,
            TokenKind::Xor => Self::BitwiseXor,
            TokenKind::ShiftLeft => Self::ShiftLeft,
            TokenKind::ShiftRight => Self::ShiftRight,
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
