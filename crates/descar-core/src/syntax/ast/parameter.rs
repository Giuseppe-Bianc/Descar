// crates/descar-core/src/syntax/ast/parameter.rs

use crate::{location::source_span::SourceSpan, syntax::ast::type_syntax::TypeSyntax};

/// Parameter definition in a function declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter {
    /// Parameter name.
    pub name: String,

    /// Annotated type of the parameter.
    pub type_annotation: TypeSyntax,

    /// Source extent from the parameter name to the end of its type.
    pub span: SourceSpan,
}

impl Parameter {
    /// Creates a new parameter definition.
    #[must_use]
    pub const fn new(name: String, type_annotation: TypeSyntax, span: SourceSpan) -> Self {
        Self { name, type_annotation, span }
    }
}
