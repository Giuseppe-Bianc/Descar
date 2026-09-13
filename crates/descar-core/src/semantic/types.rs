use std::collections::HashMap;

use crate::syntax::ast::Type;

use super::ids::TypeId;

/// Canonical builtin types currently represented by the Descar AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuiltinType {
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
    Void,
    NullPtr,
}

impl BuiltinType {
    pub const ALL: [Self; 15] = [
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::F32,
        Self::F64,
        Self::Char,
        Self::String,
        Self::Bool,
        Self::Void,
        Self::NullPtr,
    ];

    /// Returns the corresponding syntax-level type.
    #[must_use]
    pub const fn syntax(self) -> Type {
        match self {
            Self::I8 => Type::I8,
            Self::I16 => Type::I16,
            Self::I32 => Type::I32,
            Self::I64 => Type::I64,
            Self::U8 => Type::U8,
            Self::U16 => Type::U16,
            Self::U32 => Type::U32,
            Self::U64 => Type::U64,
            Self::F32 => Type::F32,
            Self::F64 => Type::F64,
            Self::Char => Type::Char,
            Self::String => Type::String,
            Self::Bool => Type::Bool,
            Self::Void => Type::Void,
            Self::NullPtr => Type::NullPtr,
        }
    }
}

/// Interned semantic types. Builtins occupy deterministic slots 0..15.
#[derive(Debug)]
pub struct TypeContext {
    types: Vec<Type>,
    builtin_ids: HashMap<BuiltinType, TypeId>,
    interned: HashMap<Type, TypeId>,
}

impl Default for TypeContext {
    /// Creates a type context with all canonical builtin types initialized.
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    /// Creates a type context and interns every builtin type in deterministic order.
    #[must_use]
    pub fn new() -> Self {
        let mut context = Self { types: Vec::new(), builtin_ids: HashMap::new(), interned: HashMap::new() };
        for builtin in BuiltinType::ALL {
            context.intern_builtin(builtin);
        }
        context
    }

    fn intern_builtin(&mut self, builtin: BuiltinType) -> TypeId {
        let id = self.intern(builtin.syntax());
        self.builtin_ids.insert(builtin, id);
        id
    }

    /// Interns a type and returns its stable semantic identifier.
    #[must_use]
    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(id) = self.interned.get(&ty).copied() {
            return id;
        }
        let index = u32::try_from(self.types.len()).expect("semantic type arena exceeded u32 capacity");
        let id = TypeId::new(index);
        self.types.push(ty.clone());
        self.interned.insert(ty, id);
        id
    }

    /// Returns the stable identifier assigned to a builtin type.
    #[must_use]
    pub fn builtin(&self, builtin: BuiltinType) -> TypeId {
        self.builtin_ids[&builtin]
    }

    /// Returns the syntax-level type associated with a semantic identifier.
    #[must_use]
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id.index() as usize)
    }

    /// Returns the number of interned types.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.types.len()
    }

    /// Returns whether the type context contains no interned types.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
