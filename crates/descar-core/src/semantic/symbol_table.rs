// crates/descar-core/src/semantic/symbol_table.rs

//! Symbols, lexical scopes, and the current function context.
//!
//! [`SymbolTable`] keeps a stack of [`Scope`] values. The first scope is the global scope.
//! The type checker declares variables and functions in the current scope and resolves a
//! name by a search from the innermost scope outward.

use std::{collections::HashMap, sync::Arc};

use crate::{
    error::{compile_error::CompileError, error_code::ErrorCode},
    location::source_span::SourceSpan,
    syntax::ast::Type,
};

/// A symbol in the symbol table.
///
/// A symbol is a variable, a function, or a type alias. Each kind of symbol has its own
/// data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// A variable and its data.
    Variable(VariableSymbol),
    /// A function and its signature.
    Function(FunctionSymbol),
    /// A type alias. The value is the type that the alias stands for.
    TypeAlias(Type),
}

/// The data of a variable symbol.
///
/// The data contains the type of the variable, whether the variable is mutable, and the
/// source locations that error messages use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableSymbol {
    /// The name of the variable.
    pub name: Arc<str>,
    /// The type of the variable.
    pub ty: Type,
    /// Whether the variable is mutable.
    pub mutable: bool,
    /// The source location of the declaration of the variable.
    pub defined_at: SourceSpan,
    /// The source location of the last assignment, if the variable has one.
    pub last_assignment: Option<SourceSpan>,
}

/// A function parameter with a resolved type.
///
/// The type checker converts the type annotation of the source into a [`Type`] to create
/// this value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedParameter {
    /// The name of the parameter.
    pub name: Arc<str>,
    /// The resolved type of the parameter.
    pub ty: Type,
    /// The source location of the declaration of the parameter.
    pub defined_at: SourceSpan,
}

/// The data of a function symbol.
///
/// The data contains the signature of the function (the parameters and the return type)
/// and the source location of the declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSymbol {
    /// The name of the function.
    pub name: Arc<str>,
    /// The parameters of the function, with their types.
    pub parameters: Vec<ResolvedParameter>,
    /// The return type of the function.
    pub return_type: Type,
    /// The source location of the declaration of the function.
    pub defined_at: SourceSpan,
}

/// The kind of a scope.
///
/// The enum uses `#[repr(u8)]`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// The global scope. It contains the top-level declarations.
    Global,
    /// The scope of a function. It contains the parameters and the local variables.
    Function,
    /// The scope of a block, for example the body of a loop or a block of statements.
    Block,
    // Future: Struct scope for struct member access
    // Struct,
}

/// One scope of the symbol table.
///
/// A scope maps names to symbols. The symbol table nests scopes to apply the rules of
/// lexical scoping.
///
/// The fields of `Scope` are private. A caller reads a scope through its methods and
/// changes it only through [`SymbolTable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    kind: ScopeKind,
    symbols: HashMap<Arc<str>, Symbol>,
    defined_at: Option<SourceSpan>,
}

impl Scope {
    /// Creates an empty scope.
    ///
    /// This constructor is private. [`SymbolTable::new`] and [`SymbolTable::push_scope`]
    /// create scopes.
    fn new(kind: ScopeKind, defined_at: Option<SourceSpan>) -> Self {
        Self { kind, symbols: HashMap::new(), defined_at }
    }

    /// Returns the kind of the scope.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> ScopeKind {
        self.kind
    }

    /// Returns the source location where the scope starts.
    ///
    /// The result is `None` for the global scope.
    #[inline]
    #[must_use]
    pub const fn defined_at(&self) -> Option<&SourceSpan> {
        self.defined_at.as_ref()
    }

    /// Returns the symbol with the name `name` in this scope.
    ///
    /// The method does not search the outer scopes.
    #[inline]
    #[must_use]
    pub fn symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Returns whether this scope has a symbol with the name `name`.
    ///
    /// The method does not search the outer scopes.
    #[inline]
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Returns the number of symbols that this scope declares directly.
    #[inline]
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Returns whether the scope has no symbols.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

/// The symbol table. It manages lexical scopes and the resolution of names.
///
/// The table has a stack of scopes. A search for a name starts in the innermost scope and
/// continues in the outer scopes. The table also stores the function that the type checker
/// is currently analyzing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    /// The stack of scopes. The current scope is the last element.
    scopes: Vec<Scope>,
    /// The current function, or `None` outside a function.
    current_function: Option<FunctionSymbol>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Creates a symbol table.
    ///
    /// # Returns
    ///
    /// A symbol table with one empty global scope and no current function.
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

    /// Opens a scope and makes it the current scope.
    ///
    /// The method does nothing if `kind` is [`ScopeKind::Global`].
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the new scope.
    /// * `defined_at` - The source location where the scope starts, if known.
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
        if kind == ScopeKind::Global {
            return;
        }

        self.scopes.push(Scope::new(kind, defined_at));
    }

    /// Closes the current scope.
    ///
    /// The method does nothing if the current scope is the global scope. The symbol table
    /// always keeps at least one scope.
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

    /// Returns the number of scopes that are open.
    ///
    /// # Returns
    ///
    /// The number of scopes in the stack. The number is at least 1 because the global scope
    /// is always open.
    #[inline]
    #[must_use]
    pub const fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    /// Returns the current (innermost) scope.
    ///
    /// The reference is read-only. Use methods of [`SymbolTable`], for example
    /// [`SymbolTable::declare`], to change a scope.
    ///
    /// # Returns
    ///
    /// `Some` with a reference to the current scope. The result is `None` only if the scope
    /// stack is empty, which does not occur.
    #[inline]
    #[must_use]
    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    /// Returns the kind of the current scope.
    ///
    /// # Returns
    ///
    /// `Some` with the kind of the current scope, or `None` if the scope stack is empty.
    #[inline]
    #[must_use]
    pub fn current_scope_kind(&self) -> Option<ScopeKind> {
        self.current_scope().map(Scope::kind)
    }

    /// Returns the symbol with the name `name` in the current scope.
    ///
    /// The method does not search the outer scopes.
    #[inline]
    #[must_use]
    pub fn current_symbol(&self, name: &str) -> Option<&Symbol> {
        self.current_scope().and_then(|scope| scope.symbol(name))
    }

    /// Returns whether the current scope has a symbol with the name `name`.
    ///
    /// The method does not search the outer scopes.
    #[inline]
    #[must_use]
    pub fn contains_current(&self, name: &str) -> bool {
        self.current_scope().is_some_and(|scope| scope.contains(name))
    }

    /// Declares a symbol in the current scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol.
    /// * `symbol` - The symbol to store under `name`.
    /// * `declared_at` - The source span of the declaration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the method stored the symbol.
    ///
    /// # Errors
    ///
    /// Returns a [`CompileError::TypeError`] with the code `E2032` if the current scope
    /// already has a symbol with the name `name`. The span of the error is `declared_at`.
    /// If the existing symbol is a variable or a function, the help text of the error gives
    /// the location of the previous declaration.
    ///
    /// # Panics
    ///
    /// Panics if the scope stack is empty. This does not occur, because the symbol table
    /// always keeps the global scope.
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

    /// Finds the innermost symbol with a name and applies a filter to it.
    ///
    /// The method searches from the innermost scope outward and applies the filter to the
    /// first symbol with the name `name`. A symbol in an inner scope hides a symbol with the
    /// same name in an outer scope, also if the filter rejects the inner symbol.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The filter. It extracts data from a symbol.
    /// * `T` - The type of the data that the filter returns.
    fn find_symbol<F, T>(&self, name: &str, filter: F) -> Option<T>
    where
        F: Fn(&Symbol) -> Option<T>,
    {
        let symbol = self.scopes.iter().rev().find_map(|scope| scope.symbol(name));

        symbol.and_then(filter)
    }

    /// Finds the innermost symbol with the name `name`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol.
    ///
    /// # Returns
    ///
    /// `Some` with a clone of the symbol, or `None` if no scope has a symbol with this name.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<Symbol> {
        self.find_symbol(name, |symbol| Some(symbol.clone()))
    }

    /// Finds the innermost symbol with the name `name` and returns it as a function.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function.
    ///
    /// # Returns
    ///
    /// `Some` with a clone of the function symbol. `None` if no scope has a symbol with this
    /// name, or if the innermost symbol with this name is not a function.
    #[must_use]
    pub fn lookup_function(&self, name: &str) -> Option<FunctionSymbol> {
        self.find_symbol(name, |symbol| match symbol {
            Symbol::Function(function) => Some(function.clone()),
            _ => None,
        })
    }

    /// Finds the innermost symbol with the name `name` and returns it as a variable.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable.
    ///
    /// # Returns
    ///
    /// `Some` with a clone of the variable symbol. `None` if no scope has a symbol with this
    /// name, or if the innermost symbol with this name is not a variable.
    #[must_use]
    pub fn lookup_variable(&self, name: &str) -> Option<VariableSymbol> {
        self.find_symbol(name, |symbol| match symbol {
            Symbol::Variable(variable) => Some(variable.clone()),
            _ => None,
        })
    }

    /// Sets the current function.
    ///
    /// The method replaces the previous current function. It does not open a scope.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to set as the current function.
    pub fn enter_function(&mut self, func: FunctionSymbol) {
        self.current_function = Some(func);
    }

    /// Clears the current function.
    ///
    /// The method does not close a scope.
    pub fn exit_function(&mut self) {
        self.current_function = None;
    }

    /// Returns the current function.
    ///
    /// # Returns
    ///
    /// `Some` with a reference to the current function, or `None` if no current function is set.
    #[must_use]
    pub const fn current_function(&self) -> Option<&FunctionSymbol> {
        self.current_function.as_ref()
    }

    /// Returns the return type of the current function.
    ///
    /// # Returns
    ///
    /// `Some` with a clone of the return type, or `None` if no current function is set.
    #[must_use]
    pub fn current_function_return_type(&self) -> Option<Type> {
        self.current_function().map(|function| function.return_type.clone())
    }
}
