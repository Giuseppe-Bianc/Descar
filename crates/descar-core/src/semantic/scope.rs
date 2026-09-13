use std::collections::BTreeMap;
use std::sync::Arc;

use super::ids::{GlobalScopeId, ScopeId, SymbolId};

/// Category of a symbol managed by the semantic symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    BuiltinType,
    BuiltinFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: Arc<str>,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub owner: Option<SymbolId>,
    symbols: BTreeMap<Arc<str>, SymbolId>,
}

impl Scope {
    /// Creates a scope with no declared symbols.
    #[must_use]
    pub const fn new(id: ScopeId, parent: Option<ScopeId>, owner: Option<SymbolId>) -> Self {
        Self { id, parent, owner, symbols: BTreeMap::new() }
    }

    /// Looks up a symbol by its name in this scope.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    /// Inserts or replaces a symbol binding and returns the previous binding.
    pub fn insert(&mut self, name: Arc<str>, symbol: SymbolId) -> Option<SymbolId> {
        self.symbols.insert(name, symbol)
    }

    /// Returns whether the scope contains no symbol bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    scopes: Vec<Scope>,
}

impl Default for SymbolTable {
    /// Creates a symbol table with the canonical root scope.
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Creates a symbol table containing an empty root scope.
    #[must_use]
    pub fn new() -> Self {
        Self { symbols: Vec::new(), scopes: vec![Scope::new(ScopeId::new(0), None, None)] }
    }

    /// Returns the canonical global scope identifier.
    #[must_use]
    pub const fn global_scope(&self) -> GlobalScopeId {
        ScopeId::new(0)
    }

    /// Returns a shared reference to the requested scope.
    #[must_use]
    pub fn scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.index() as usize)
    }

    /// Returns a mutable reference to the requested scope.
    #[must_use]
    pub fn scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.index() as usize)
    }

    /// Returns a shared reference to the requested symbol.
    #[must_use]
    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.index() as usize)
    }

    /// Registers a compiler-provided builtin without adding it to a program scope.
    pub fn insert_builtin(&mut self, name: Arc<str>, kind: SymbolKind) -> SymbolId {
        let index = u32::try_from(self.symbols.len()).expect("semantic symbol arena exceeded u32 capacity");
        let id = SymbolId::new(index);
        self.symbols.push(Symbol { id, name, kind });
        id
    }

    /// Returns the number of symbols stored in the table.
    #[must_use]
    pub const fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}
