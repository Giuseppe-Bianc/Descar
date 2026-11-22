// crates/descar-core/src/syntax/ast/assignment_op.rs

use std::sync::Arc;

use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::syntax::ast::binary_op::BinaryOp;
use crate::tokens::token::Token;
use crate::tokens::token_kind::TokenKind;

/// Assignment operators supported by assignment expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignmentOp {
    /// Simple assignment (=).
    Assign,
    /// Addition assignment (+=).
    AddAssign,
    /// Subtraction assignment (-=).
    SubtractAssign,
    /// Multiplication assignment (*=).
    MultiplyAssign,
    /// Division assignment (/=).
    DivideAssign,
    /// Remainder assignment (%=).
    ModuloAssign,
    /// Bitwise AND assignment (&=).
    BitwiseAndAssign,
    /// Bitwise OR assignment (|=).
    BitwiseOrAssign,
    /// Bitwise XOR assignment (^=).
    BitwiseXorAssign,
    /// Left shift assignment (<<=).
    ShiftLeftAssign,
    /// Right shift assignment (>>=).
    ShiftRightAssign,
}

impl AssignmentOp {
    /// Converts an assignment token into its corresponding assignment operator.
    ///
    /// # Arguments
    ///
    /// * `token` - The token to convert.
    ///
    /// # Errors
    ///
    /// Returns a [`CompileError::SyntaxError`] with code `E1005` if the token is not an
    /// assignment operator.
    #[allow(clippy::result_large_err)]
    pub fn get_op(token: &Token) -> Result<Self, CompileError> {
        Ok(match token.kind {
            TokenKind::Equal => Self::Assign,
            TokenKind::PlusEqual => Self::AddAssign,
            TokenKind::MinusEqual => Self::SubtractAssign,
            TokenKind::StarEqual => Self::MultiplyAssign,
            TokenKind::SlashEqual => Self::DivideAssign,
            TokenKind::PercentEqual => Self::ModuloAssign,
            TokenKind::AndEqual => Self::BitwiseAndAssign,
            TokenKind::OrEqual => Self::BitwiseOrAssign,
            TokenKind::XorEqual => Self::BitwiseXorAssign,
            TokenKind::ShiftLeftEqual => Self::ShiftLeftAssign,
            TokenKind::ShiftRightEqual => Self::ShiftRightAssign,
            _ => {
                return Err(CompileError::SyntaxError {
                    code: Some(ErrorCode::E1005),
                    message: Arc::from(format!("Invalid assignment operator: {:?}", token.kind)),
                    span: token.span.clone(),
                    help: None,
                });
            }
        })
    }

    /// Returns the binary operation that a compound assignment performs.
    ///
    /// # Returns
    ///
    /// `Some` with the binary operator for a compound assignment, for example [`BinaryOp::Add`] for
    /// `+=`. `None` for the simple assignment `=`.
    #[must_use]
    pub const fn compound_binary_op(self) -> Option<BinaryOp> {
        match self {
            Self::Assign => None,
            Self::AddAssign => Some(BinaryOp::Add),
            Self::SubtractAssign => Some(BinaryOp::Subtract),
            Self::MultiplyAssign => Some(BinaryOp::Multiply),
            Self::DivideAssign => Some(BinaryOp::Divide),
            Self::ModuloAssign => Some(BinaryOp::Modulo),
            Self::BitwiseAndAssign => Some(BinaryOp::BitwiseAnd),
            Self::BitwiseOrAssign => Some(BinaryOp::BitwiseOr),
            Self::BitwiseXorAssign => Some(BinaryOp::BitwiseXor),
            Self::ShiftLeftAssign => Some(BinaryOp::ShiftLeft),
            Self::ShiftRightAssign => Some(BinaryOp::ShiftRight),
        }
    }
}
