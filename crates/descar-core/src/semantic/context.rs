use std::collections::BTreeMap;
use std::sync::Arc;

use crate::syntax::ast::Stmt;

use super::{
    config::SemanticConfig,
    diagnostics::{Diagnostic, DiagnosticEngine},
    ids::{GlobalScopeId, SymbolId},
    scope::{SymbolKind, SymbolTable},
    types::{BuiltinType, TypeContext},
    validator::{AstValidationError, validate_ast_shape},
};

/// Resulting state of semantic-context initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializationStatus {
    Ready,
    Recovery,
    Failed,
}

#[derive(Debug)]
pub struct BuiltinRegistry {
    type_symbols: BTreeMap<BuiltinType, SymbolId>,
}

impl BuiltinRegistry {
    fn new(symbols: &mut SymbolTable) -> Self {
        let mut type_symbols = BTreeMap::new();
        for builtin in BuiltinType::ALL {
            let name = builtin_name(builtin);
            let symbol = symbols.insert_builtin(Arc::from(name), SymbolKind::BuiltinType);
            type_symbols.insert(builtin, symbol);
        }
        Self { type_symbols }
    }

    /// Returns the symbol assigned to a builtin type.
    #[must_use]
    pub fn type_symbol(&self, builtin: BuiltinType) -> SymbolId {
        self.type_symbols[&builtin]
    }

    /// Returns the complete builtin type-to-symbol registry.
    #[must_use]
    pub const fn type_symbols(&self) -> &BTreeMap<BuiltinType, SymbolId> {
        &self.type_symbols
    }
}

/// Owns the semantic state shared by the Phase 01 analysis pipeline.
#[derive(Debug)]
pub struct SemanticContext {
    config: SemanticConfig,
    types: TypeContext,
    symbols: SymbolTable,
    builtins: BuiltinRegistry,
    diagnostics: DiagnosticEngine,
    status: InitializationStatus,
    frozen: bool,
}

impl SemanticContext {
    /// Validates an AST shape and freezes a successfully initialized context.
    #[allow(clippy::result_large_err)]
    pub fn initialize(ast: &[Stmt], config: SemanticConfig) -> Result<Self, Self> {
        let mut context = Self::new(config);
        if let Err(error) = validate_ast_shape(ast) {
            context.diagnostics.emit(Diagnostic::internal(format_ast_error(&error), None));
            context.status = InitializationStatus::Failed;
            return Err(context);
        }
        context.frozen = true;
        Ok(context)
    }

    /// Creates an unfrozen semantic context with builtin state initialized.
    #[must_use]
    pub fn new(config: SemanticConfig) -> Self {
        let types = TypeContext::new();
        let mut symbols = SymbolTable::new();
        let builtins = BuiltinRegistry::new(&mut symbols);
        Self {
            config,
            types,
            symbols,
            builtins,
            diagnostics: DiagnosticEngine::new(),
            status: InitializationStatus::Ready,
            frozen: false,
        }
    }

    /// Returns the immutable semantic configuration.
    #[must_use]
    pub const fn config(&self) -> &SemanticConfig {
        &self.config
    }

    /// Returns the canonical global scope identifier.
    #[must_use]
    pub const fn global_scope(&self) -> GlobalScopeId {
        self.symbols.global_scope()
    }

    /// Returns the semantic type context.
    #[must_use]
    pub const fn types(&self) -> &TypeContext {
        &self.types
    }

    /// Returns the builtin symbol registry.
    #[must_use]
    pub const fn builtins(&self) -> &BuiltinRegistry {
        &self.builtins
    }

    /// Returns the semantic symbol table.
    #[must_use]
    pub const fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// Returns diagnostics collected during initialization.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticEngine {
        &self.diagnostics
    }

    /// Returns the current initialization status.
    #[must_use]
    pub const fn status(&self) -> InitializationStatus {
        self.status
    }

    /// Returns whether successful initialization has frozen the context.
    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.frozen
    }
}

const fn builtin_name(builtin: BuiltinType) -> &'static str {
    match builtin {
        BuiltinType::I8 => "i8",
        BuiltinType::I16 => "i16",
        BuiltinType::I32 => "i32",
        BuiltinType::I64 => "i64",
        BuiltinType::U8 => "u8",
        BuiltinType::U16 => "u16",
        BuiltinType::U32 => "u32",
        BuiltinType::U64 => "u64",
        BuiltinType::F32 => "f32",
        BuiltinType::F64 => "f64",
        BuiltinType::Char => "char",
        BuiltinType::String => "string",
        BuiltinType::Bool => "bool",
        BuiltinType::Void => "void",
        BuiltinType::NullPtr => "nullptr",
    }
}

fn format_ast_error(error: &AstValidationError) -> String {
    match error {
        AstValidationError::InvalidSpan { file } => format!("invalid AST source span in {file}"),
        AstValidationError::MultipleSourceFiles { first, second } => {
            format!("compilation unit contains spans from multiple source files: {first} and {second}")
        }
    }
}
