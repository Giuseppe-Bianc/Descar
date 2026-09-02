use std::fmt;

/// Sentinel value for optional fields that have not been computed.
pub const UNKNOWN: usize = usize::MAX;

/// Source position within a compilation unit.
///
/// Equivalente immutabile del `record` Java, confrontabile per `offset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default, Hash)]
pub struct SourceLocation {
    line: usize,
    column: usize,
    offset: usize,
    index: usize,
    utf8_offset: usize,
    code_point_offset: usize,
}

impl SourceLocation {
    #[must_use]
    pub const fn new(
        line: usize, column: usize, offset: usize, index: usize, utf8_offset: usize, code_point_offset: usize,
    ) -> Self {
        Self { line, column, offset, index, utf8_offset, code_point_offset }
    }

    /// Numero di riga (1-based).
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Numero di colonna (1-based).
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Offset in byte (0-based).
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Indice di carattere (0-based).
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Offset UTF-8, o `UNKNOWN` se non calcolato.
    #[must_use]
    pub const fn utf8_offset(&self) -> usize {
        self.utf8_offset
    }

    /// Offset in code point, o `UNKNOWN` se non calcolato.
    #[must_use]
    pub const fn code_point_offset(&self) -> usize {
        self.code_point_offset
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}:column {}", self.line, self.column)
    }
}
