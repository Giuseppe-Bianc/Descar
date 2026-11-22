// crates/descar-core/src/syntax/ast/type_syntax.rs

use crate::syntax::ast::{expr::Expr, literal_value::LiteralValue};
use std::{fmt, sync::Arc};

/// Type written in source code before semantic resolution.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TypeSyntax {
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
    /// Type that an identifier names.
    Custom {
        /// Type identifier.
        name: Arc<str>,
    },
    /// Array type that has a size expression.
    Array {
        /// Array element type.
        element_type: Box<Self>,
        /// Expression of the array size.
        size: Box<Expr>,
    },
    /// Vector type, written `vector<element>`.
    Vector {
        /// Vector element type.
        element_type: Box<Self>,
    },
    /// Implicit return type used when a function has no return annotation.
    Void,
}

/// Writes the type as it appears in source code.
///
/// An array type has the form `[element; length]`. The length is the text of a numeric literal. For any
/// other size expression, the length is `?`. A vector type has the form `vector<element>`.
impl fmt::Display for TypeSyntax {
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
            Self::Array { element_type, size } => write!(f, "[{element_type}; {}]", format_array_size(size)),
            Self::Vector { element_type } => write!(f, "vector<{element_type}>"),
            Self::Void => f.write_str("void"),
        }
    }
}

/// Returns the text of a numeric literal size, or `?` for any other size expression.
fn format_array_size(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value: LiteralValue::Numeric(number), .. } => number.to_string(),
        _ => "?".to_string(),
    }
}
