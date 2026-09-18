use crate::syntax::ast::{expr::Expr, literal_value::LiteralValue};
use std::{fmt, sync::Arc};

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


impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I8 => f.write_str("i8"),
            Self::I16 => f.write_str("i16"),
            Self::I32 => f.write_str("i32"),
            Self::I64 => f.write_str("i64"),
            Self::U8 => f.write_str("u8"),
            Self::U16 => f.write_str("u16"),
            Self::U32 => f.write_str("u32"),
            Self::U64 => f.write_str("u64"),
            Self::F32 => f.write_str("f32"),
            Self::F64 => f.write_str("f64"),
            Self::Char => f.write_str("char"),
            Self::String => f.write_str("string"),
            Self::Bool => f.write_str("bool"),
            Self::Custom { name } => f.write_str(name),
            Self::Array { element_type, size } => {
                write!(f, "[{element_type}; {}]", format_array_size(size))
            }
            Self::Vector { element_type } => write!(f, "vector<{element_type}>"),
            Self::Void => f.write_str("void"),
            Self::NullPtr => f.write_str("nullptr"),
        }
    }
}

fn format_array_size(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value: LiteralValue::Numeric(number), .. } => number.to_string(),
        _ => "?".to_string(),
    }
}
