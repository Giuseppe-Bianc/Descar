use super::stmt::Stmt;

/// The complete set of top-level declarations in one source file.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub struct CompilationUnit {
    /// Declarations in source order. Only global variables, functions, and main
    /// are admitted by the parser at this level.
    pub declarations: Vec<Stmt>,
}

impl CompilationUnit {
    /// Creates a compilation unit from already parsed top-level declarations.
    #[must_use]
    pub const fn new(declarations: Vec<Stmt>) -> Self {
        Self { declarations }
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
