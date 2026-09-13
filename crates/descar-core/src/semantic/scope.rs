use std::collections::BTreeMap;
use std::sync::Arc;

use super::ids::{GlobalScopeId, ScopeId, SymbolId};

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
    #[must_use]
    pub fn new(id: ScopeId, parent: Option<ScopeId>, owner: Option<SymbolId>) -> Self {
        Self { id, parent, owner, symbols: BTreeMap::new() }
    }

    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<SymbolId> { self.symbols.get(name).copied() }

    pub fn insert(&mut self, name: Arc<str>, symbol: SymbolId) -> Option<SymbolId> { self.symbols.insert(name, symbol) }

    #[must_use]
    pub fn is_empty(&self) -> bool { self.symbols.is_empty() }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    scopes: Vec<Scope>,
}

impl SymbolTable {
    #[must_use]
    pub fn new() -> Self {
        let mut table = Self::default();
        table.scopes.push(Scope::new(ScopeId::new(0), None, None));
        table
    }

    #[must_use]
    pub const fn global_scope(&self) -> GlobalScopeId { ScopeId::new(0) }

    #[must_use]
    pub fn scope(&self, id: ScopeId) -> Option<&Scope> { self.scopes.get(id.index() as usize) }

    #[must_use]
    pub fn scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> { self.scopes.get_mut(id.index() as usize) }

    #[must_use]
    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> { self.symbols.get(id.index() as usize) }

    pub fn insert_builtin(&mut self, name: Arc<str>, kind: SymbolKind) -> SymbolId {
        let id = SymbolId::new(self.symbols.len() as u32);
        self.symbols.push(Symbol { id, name, kind });
        id
    }

    #[must_use]
    pub fn symbol_count(&self) -> usize { self.symbols.len() }
}
