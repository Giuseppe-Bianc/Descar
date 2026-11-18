use super::stmt::Stmt;
use crate::location::source_span::SourceSpan;

/// The complete set of top-level declarations in one source file.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub struct CompilationUnit {
    /// Top-level statements in source order. For parses without syntax errors,
    /// /// the parser admits only global variables, functions, and main at this level.
    /// Error recovery may preserve invalid top-level statements in a recovered
    /// compilation unit so downstream analysis can report additional diagnostics.
    pub declarations: Vec<Stmt>,
    /// Source extent spanning the first through last declaration.
    pub span: SourceSpan,
}

impl CompilationUnit {
    /// Creates a compilation unit from already parsed top-level declarations.
    #[must_use]
    pub const fn new(declarations: Vec<Stmt>, span: SourceSpan) -> Self {
        Self { declarations, span }
    }

    /// Returns true when the unit contains no declarations.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Returns the number of top-level declarations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.declarations.len()
    }
}

impl std::ops::Deref for CompilationUnit {
    type Target = [Stmt];

    fn deref(&self) -> &Self::Target {
        &self.declarations
    }
}

impl<'a> IntoIterator for &'a CompilationUnit {
    type Item = &'a Stmt;
    type IntoIter = std::slice::Iter<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.declarations.iter()
    }
}
