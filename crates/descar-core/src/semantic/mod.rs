pub mod config;
pub mod context;
pub mod diagnostics;
pub mod ids;
pub mod scope;
pub mod types;
pub mod validator;

pub use config::{LanguageVersion, SemanticConfig};
pub use context::{BuiltinRegistry, InitializationStatus, SemanticContext};
pub use diagnostics::{Diagnostic, DiagnosticEngine, DiagnosticSeverity};
pub use ids::{AstNodeId, GlobalScopeId, ScopeId, SymbolId, TypeId};
pub use scope::{Scope, Symbol, SymbolKind, SymbolTable};
pub use types::{BuiltinType, TypeContext};
pub use validator::{validate_ast_shape, AstValidationError};
