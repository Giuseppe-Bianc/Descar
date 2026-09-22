use std::{collections::HashMap, sync::Arc};

use crate::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    location::source_span::SourceSpan,
    syntax::ast::{Parameter, Type},
};

/// Represents a symbol in the symbol table.
///
/// Symbols can be variables, functions, or type aliases. Each symbol type
/// carries specific metadata relevant to semantic analysis and code generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// A variable symbol with associated metadata
    Variable(VariableSymbol),
    /// A function symbol with signature information
    Function(FunctionSymbol),
    /// A type alias mapping to an underlying type
    TypeAlias(Type),
}

/// Metadata for a variable symbol.
///
/// Tracks all information necessary for type checking and mutability analysis,
/// including the variable's type, mutability status, and location information
/// for error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableSymbol {
    /// The name of the variable
    pub name: Arc<str>,
    /// The type of the variable
    pub ty: Type,
    /// Whether the variable is mutable
    pub mutable: bool,
    /// Source location where the variable was defined
    pub defined_at: SourceSpan,
    /// Source location of the last assignment (if any)
    pub last_assignment: Option<SourceSpan>,
}

/// Metadata for a function symbol.
///
/// Contains the function signature including parameters and return type,
/// along with location information for error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSymbol {
    /// The name of the function
    pub name: Arc<str>,
    /// The function's parameters with their types
    pub parameters: Vec<Parameter>,
    /// The return type of the function
    pub return_type: Type,
    /// Source location where the function was defined
    pub defined_at: SourceSpan,
}

/// Represents the different kinds of scopes in the program.
///
/// The scope kind determines visibility rules and what operations are valid.
/// Using `#[repr(u8)]` ensures a compact memory representation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// The global scope containing top-level declarations
    Global,
    /// A function scope containing parameters and local variables
    Function,
    /// A block scope (e.g., inside if, while, for, or explicit blocks)
    Block,
    // Future: Struct scope for struct member access
    // Struct,
}

/// Represents a single scope in the symbol table hierarchy.
///
/// Each scope maintains its own symbol mappings and can be nested within
/// other scopes to implement lexical scoping rules.
///
/// The fields of `Scope` are intentionally private. Access to scope state
/// must be performed through controlled methods so that callers cannot
/// bypass symbol table invariants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    kind: ScopeKind,
    symbols: HashMap<Arc<str>, Symbol>,
    defined_at: Option<SourceSpan>,
}

impl Scope {
    /// Creates a new empty scope.
    ///
    /// This constructor is private to the symbol table module. Scope creation
    /// is controlled by `SymbolTable::new()` and `SymbolTable::push_scope()`.
    fn new(kind: ScopeKind, defined_at: Option<SourceSpan>) -> Self {
        Self { kind, symbols: HashMap::new(), defined_at }
    }

    /// Returns the kind of this scope.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> ScopeKind {
        self.kind
    }

    /// Returns the source location where this scope was created.
    #[inline]
    #[must_use]
    pub const fn defined_at(&self) -> Option<&SourceSpan> {
        self.defined_at.as_ref()
    }

    /// Returns the symbol associated with `name` in this scope only.
    ///
    /// This method does not search outer scopes.
    #[inline]
    #[must_use]
    pub fn symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Returns whether this scope contains a symbol with `name`.
    ///
    /// This method checks this scope only and does not perform lexical lookup.
    #[inline]
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Returns the number of symbols declared directly in this scope.
    #[inline]
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Returns whether this scope contains no symbols.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

/// The symbol table manages lexical scoping and symbol resolution.
///
/// Implements a stack of scopes to support nested lexical scoping, with
/// symbols resolved by searching from the innermost scope outward.
/// Also tracks the current function context for return type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    /// Stack of scopes, with the current scope at the end
    scopes: Vec<Scope>,
    /// The currently active function (if inside a function)
    current_function: Option<FunctionSymbol>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Creates a new symbol table with a global scope.
    ///
    /// # Returns
    ///
    /// A new `SymbolTable` initialized with an empty global scope.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::semantic::symbol_table::SymbolTable;
    /// let symbol_table = SymbolTable::new();
    /// assert_eq!(symbol_table.scope_count(), 1);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { scopes: vec![Scope::new(ScopeKind::Global, None)], current_function: None }
    }

    /// Pushes a new scope onto the scope stack.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of scope to create
    /// * `defined_at` - Optional source location where the scope begins
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::semantic::symbol_table::SymbolTable;
    /// use descar_core::semantic::symbol_table::ScopeKind;
    /// let mut table = SymbolTable::new();
    /// table.push_scope(ScopeKind::Block, None);
    /// assert_eq!(table.scope_count(), 2);
    /// ```
    pub fn push_scope(&mut self, kind: ScopeKind, defined_at: Option<SourceSpan>) {
        self.scopes.push(Scope::new(kind, defined_at));
    }

    /// Pops the current scope from the scope stack.
    ///
    /// The global scope is never popped to maintain invariant that at least
    /// one scope always exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::semantic::symbol_table::SymbolTable;
    /// use descar_core::semantic::symbol_table::ScopeKind;
    /// let mut table = SymbolTable::new();
    /// table.push_scope(ScopeKind::Block, None);
    /// table.pop_scope();
    /// assert_eq!(table.scope_count(), 1);
    /// ```
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Returns the total number of scopes currently active.
    ///
    /// # Returns
    ///
    /// The count of scopes in the stack (always at least 1 for global scope).
    #[inline]
    #[must_use]
    pub const fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    /// Returns a reference to the current (innermost) scope.
    ///
    /// The returned scope is read-only. Mutation must be performed through
    /// symbol table APIs such as `declare()`.
    ///
    /// # Returns
    ///
    /// An optional reference to the current scope.
    #[inline]
    #[must_use]
    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    /// Returns the kind of the current scope.
    ///
    /// # Returns
    ///
    /// An optional `ScopeKind` indicating the type of the current scope.
    #[inline]
    #[must_use]
    pub fn current_scope_kind(&self) -> Option<ScopeKind> {
        self.current_scope().map(Scope::kind)
    }

    /// Looks up a symbol in the current scope only.
    ///
    /// Outer scopes are intentionally ignored.
    ///
    /// This method provides controlled read-only access to the current scope
    /// without exposing the internal symbol map.
    #[inline]
    #[must_use]
    pub fn current_symbol(&self, name: &str) -> Option<&Symbol> {
        self.current_scope().and_then(|scope| scope.symbol(name))
    }

    /// Returns whether the current scope contains `name`.
    ///
    /// Outer scopes are intentionally ignored.
    #[inline]
    #[must_use]
    pub fn contains_current(&self, name: &str) -> bool {
        self.current_scope().is_some_and(|scope| scope.contains(name))
    }

    /// Declares a new symbol in the current scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol to declare
    /// * `symbol` - The symbol metadata to associate with the name
    /// * `declared_at` - The source span of the declaration being added
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the symbol was successfully declared
    /// * `Err(CompileError)` if a symbol with the same name already exists in the current scope
    ///
    /// # Panics
    ///
    /// Panics if the scope stack is empty. This should never occur in practice as the
    /// symbol table maintains the invariant that at least the global scope always exists.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the identifier is already declared in the current scope,
    /// including the location of the previous declaration for error reporting.
    #[allow(clippy::expect_used, clippy::result_large_err)]
    pub fn declare(&mut self, name: &str, symbol: Symbol, declared_at: SourceSpan) -> Result<(), CompileError> {
        let current_scope = self.current_scope().expect("At least one scope");

        if let Some(existing) = current_scope.symbols.get(name) {
            let help = match existing {
                Symbol::Variable(v) => Some(format!("Previous declaration at {}", v.defined_at)),
                Symbol::Function(f) => Some(format!("Previous declaration at {}", f.defined_at)),
                Symbol::TypeAlias(_) => None,
            };

            return Err(CompileError::TypeError {
                code: Some(ErrorCode::E2032),
                message: Arc::from(format!(
                    "Identifier '{}' already declared in this {:?} scope",
                    name, current_scope.kind
                )),
                span: declared_at,
                help,
            });
        }

        self.scopes.last_mut().expect("At least one scope").symbols.insert(name.into(), symbol);

        Ok(())
    }

    /// Generic helper method to find symbols with a custom filter.
    ///
    /// Searches from the innermost scope outward and applies the filter to the
    /// first symbol with the requested name. A symbol in an inner scope therefore
    /// hides same-named symbols in outer scopes even when the filter rejects it.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Filter function that extracts desired data from a symbol
    /// * `T` - Type of data returned by the filter
    fn find_symbol<F, T>(&self, name: &str, filter: F) -> Option<T>
    where
        F: Fn(&Symbol) -> Option<T>,
    {
        let symbol = self.scopes.iter().rev().find_map(|scope| scope.symbol(name));

        symbol.and_then(filter)
    }

    /// Looks up a symbol by name, searching through all scopes.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol to look up
    ///
    /// # Returns
    ///
    /// An optional clone of the symbol if found, or `None` if not found.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<Symbol> {
        self.find_symbol(name, |sym| Some(sym.clone()))
    }

    /// Looks up the innermost symbol with the given name as a function.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to look up
    ///
    /// # Returns
    ///
    /// A clone of the function symbol, or `None` if the name is not found or its
    /// innermost declaration is not a function.
    #[must_use]
    pub fn lookup_function(&self, name: &str) -> Option<FunctionSymbol> {
        self.find_symbol(name, |sym| match sym {
            Symbol::Function(f) => Some(f.clone()),
            _ => None,
        })
    }

    /// Looks up the innermost symbol with the given name as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to look up
    ///
    /// # Returns
    ///
    /// A clone of the variable symbol, or `None` if the name is not found or its
    /// innermost declaration is not a variable.
    #[must_use]
    pub fn lookup_variable(&self, name: &str) -> Option<VariableSymbol> {
        self.find_symbol(name, |sym| match sym {
            Symbol::Variable(v) => Some(v.clone()),
            _ => None,
        })
    }

    /// Sets the current function context.
    ///
    /// This is used to track which function is currently being analyzed,
    /// enabling proper return type checking.
    ///
    /// # Arguments
    ///
    /// * `func` - The function symbol to set as current
    pub fn enter_function(&mut self, func: FunctionSymbol) {
        self.current_function = Some(func);
    }

    /// Clears the current function context.
    ///
    /// Called when exiting a function scope to reset the function context.
    pub fn exit_function(&mut self) {
        self.current_function = None;
    }

    /// Returns a reference to the current function being analyzed.
    ///
    /// # Returns
    ///
    /// An optional reference to the current function symbol, or `None` if
    /// not currently inside a function.
    #[must_use]
    pub const fn current_function(&self) -> Option<&FunctionSymbol> {
        self.current_function.as_ref()
    }

    /// Returns the return type of the current function.
    ///
    /// # Returns
    ///
    /// An optional clone of the current function's return type, or `None` if
    /// not currently inside a function.
    #[must_use]
    pub fn current_function_return_type(&self) -> Option<Type> {
        self.current_function().map(|f| f.return_type.clone())
    }
}
