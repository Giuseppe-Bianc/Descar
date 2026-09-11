use crate::{location::source_span::SourceSpan, syntax::ast::ast_type::Type};

/// Parameter definition in a function declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter {
    /// Parameter name.
    pub name: String,

    /// Annotated type of the parameter.
    pub type_annotation: Type,

    /// Source location extent.
    pub span: SourceSpan,
}

impl Parameter {
    /// Creates a new parameter definition.
    #[must_use]
    pub const fn new(name: String, type_annotation: Type, span: SourceSpan) -> Self {
        Self { name, type_annotation, span }
    }
}
