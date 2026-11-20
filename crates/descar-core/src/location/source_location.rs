use std::fmt;

/// Sentinel value for optional fields that have not been computed.
pub const UNKNOWN: usize = usize::MAX;

/// Source position within a compilation unit.
///
/// The type is immutable. It has the same role as a Java `record`.
///
/// The derived comparison operators compare the fields in this order: `line`, `column`,
/// `offset`, `index`, `utf8_offset`, and `code_point_offset`. The comparison does not
/// use only `offset`.
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
    /// Creates a source location from its coordinates.
    ///
    /// The method does not validate the values.
    ///
    /// # Arguments
    ///
    /// * `line` - The line number, starting at 1.
    /// * `column` - The column number, starting at 1.
    /// * `offset` - The byte offset, starting at 0.
    /// * `index` - The character index, starting at 0.
    /// * `utf8_offset` - The UTF-8 offset, or [`UNKNOWN`] if it is not computed.
    /// * `code_point_offset` - The code point offset, or [`UNKNOWN`] if it is not computed.
    #[must_use]
    pub const fn new(
        line: usize, column: usize, offset: usize, index: usize, utf8_offset: usize, code_point_offset: usize,
    ) -> Self {
        Self { line, column, offset, index, utf8_offset, code_point_offset }
    }

    /// Returns the line number, starting at 1.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number, starting at 1.
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Returns the byte offset, starting at 0.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the character index, starting at 0.
    ///
    /// [`LineTracker::location_for`](crate::location::line_tracker::LineTracker::location_for)
    /// sets this value to the UTF-16 code-unit offset.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the UTF-8 offset, or [`UNKNOWN`] if it is not computed.
    #[must_use]
    pub const fn utf8_offset(&self) -> usize {
        self.utf8_offset
    }

    /// Returns the code point offset, or [`UNKNOWN`] if it is not computed.
    #[must_use]
    pub const fn code_point_offset(&self) -> usize {
        self.code_point_offset
    }
}

impl fmt::Display for SourceLocation {
    /// Writes the line number and the column number in the form `line <line>:column <column>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}:column {}", self.line, self.column)
    }
}
