use std::fmt;

macro_rules! semantic_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u32);

        impl $name {
            #[must_use]
            pub const fn new(index: u32) -> Self {
                Self(index)
            }

            #[must_use]
            pub const fn index(self) -> u32 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

semantic_id!(SymbolId);
semantic_id!(ScopeId);
semantic_id!(TypeId);
semantic_id!(AstNodeId);

pub type GlobalScopeId = ScopeId;
