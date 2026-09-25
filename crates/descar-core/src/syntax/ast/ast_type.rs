use crate::syntax::ast::{expr::Expr, literal_value::LiteralValue};
use std::{fmt, sync::Arc};

/// Representation of primitive, user-defined, composite, function, and special types.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    String,
    Bool,
    Custom { name: Arc<str> },
    Array { element_type: Box<Self>, size: Box<Expr> },
    Vector { element_type: Box<Self> },
    Function { parameters: Vec<Self>, return_type: Box<Self> },
    Void,
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
                write!(f, "[{}; {}]", element_type, format_array_size(size))
            }
            Self::Vector { element_type } => write!(f, "vector<{}>", element_type),
            Self::Function { parameters, return_type } => {
                let params = parameters.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                write!(f, "fn({}) -> {}", params, return_type)
            }
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
