use crate::syntax::ast::expr::Expr;
use std::sync::Arc;

/// Representation of primitive, user-defined, composite, and special types.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Type {
    /// Signed 8-bit integer type.
    I8,

    /// Signed 16-bit integer type.
    I16,

    /// Signed 32-bit integer type.
    I32,

    /// Signed 64-bit integer type.
    I64,

    /// Unsigned 8-bit integer type.
    U8,

    /// Unsigned 16-bit integer type.
    U16,

    /// Unsigned 32-bit integer type.
    U32,

    /// Unsigned 64-bit integer type.
    U64,

    /// 32-bit floating point type.
    F32,

    /// 64-bit floating point type.
    F64,

    /// Character type.
    Char,

    /// String type.
    String,

    /// Boolean type.
    Bool,

    /// User-defined custom type.
    Custom {
        /// Type identifier.
        name: Arc<str>,
    },

    /// Fixed-size array type.
    Array {
        /// Array element type.
        element_type: Box<Self>,

        /// Expression evaluating to array capacity.
        size: Box<Expr>,
    },

    /// Dynamic vector type.
    Vector {
        /// Vector element type.
        element_type: Box<Self>,
    },

    /// Void type.
    Void,

    /// Null pointer type.
    NullPtr,
}
