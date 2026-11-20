//! Representation of a span (extent) within a source file.
//!
//! A [`SourceSpan`] identifies a portion of source text. A span has an inclusive
//! start position, an exclusive end position, and the path of the file that it
//! refers to. The compiler uses spans to associate tokens, AST nodes, and
//! diagnostics with a precise location in the source code.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::location::source_location::SourceLocation;

/// Extent (span) of a token or construct within the source text.
///
/// The span is defined by a half-open interval `[start, end)`:
/// - `start` is the **inclusive** start position;
/// - `end` is the **exclusive** end position.
///
/// This convention allows empty spans (`start == end`, see
/// [`SourceSpan::point`]). It also simplifies the computation of the length and the
/// merge of spans (see [`SourceSpan::length`] and [`SourceSpan::merge`]).
///
/// Each span holds the path of the source file (`file_path`). The path is shared
/// through [`Arc<str>`], so many spans that refer to the same file do not duplicate
/// the string.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default, Hash)]
pub struct SourceSpan {
    /// Path of the source file this span refers to.
    file_path: Arc<str>,
    /// Start position, inclusive.
    start: SourceLocation,
    /// End position, exclusive.
    end: SourceLocation,
}

impl SourceSpan {
    /// Creates a new span given the file path and the start and end positions.
    ///
    /// The method does not validate the positions. The caller must make sure that
    /// `start` precedes `end` or has the same offset as `end`.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the source file.
    /// * `start` - The start position (inclusive).
    /// * `end` - The end position (exclusive).
    #[must_use]
    pub const fn new(file_path: Arc<str>, start: SourceLocation, end: SourceLocation) -> Self {
        Self { file_path, start, end }
    }

    /// Creates a zero-length span at the given position.
    ///
    /// A "point" span is always valid, because `start` and `end` have the same value.
    /// Use a point span to represent a position, for example an error location,
    /// without covering any text.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the source file.
    /// * `location` - The position that is both the start and the end of the span.
    #[must_use]
    pub const fn point(file_path: Arc<str>, location: SourceLocation) -> Self {
        // A point span is always valid: start == end.
        Self { file_path, start: location, end: location }
    }

    /// Returns the start position (inclusive) of the span.
    #[must_use]
    pub const fn start(&self) -> SourceLocation {
        self.start
    }

    /// Returns the end position (exclusive) of the span.
    #[must_use]
    pub const fn end(&self) -> SourceLocation {
        self.end
    }

    /// Returns the path of the source file associated with this span.
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns the length of the span in UTF-8 bytes.
    ///
    /// The length is the end offset minus the start offset. The offsets are the
    /// 0-based byte offsets that [`SourceLocation::offset`] returns. If the span is
    /// reversed, the length is 0.
    ///
    /// Note: the length follows the semantics of the Rust method `str::len()`. A
    /// multi-byte character such as `'€'` (U+20AC) adds **3** to the length, not 1.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.end.offset().saturating_sub(self.start.offset())
    }

    /// Returns `true` if the start offset is less than or equal to the end offset.
    ///
    /// A span that returns `false` is reversed.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.start.offset() <= self.end.offset()
    }

    /// Returns `true` if the span has zero length, that is, if the start offset is
    /// equal to the end offset.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start.offset() == self.end.offset()
    }

    /// Returns `true` if the span covers more than one line.
    ///
    /// The method compares the line of `start` with the line of `end`.
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.start.line() != self.end.line()
    }

    /// Returns `true` if the given location is in the half-open interval
    /// `[start, end)` of this span.
    ///
    /// The method compares only the byte offsets. It does not compare file paths.
    #[must_use]
    pub const fn contains(&self, location: SourceLocation) -> bool {
        location.offset() >= self.start.offset() && location.offset() < self.end.offset()
    }

    /// Returns `true` if two spans share at least one character, that is, if their
    /// `[start, end)` intervals intersect.
    ///
    /// Spans from different source files never overlap. A reversed span and an empty
    /// span never overlap another span.
    ///
    /// Two adjacent spans (for example `[0, 3)` and `[3, 5)`) do not overlap.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.is_valid()
            && other.is_valid()
            && self.file_path == other.file_path
            && self.start.offset().max(other.start.offset()) < self.end.offset().min(other.end.offset())
    }

    /// Returns the smallest span that contains both spans (`self` and `other`).
    ///
    /// The new span starts at the smaller of the two start positions and ends at the
    /// larger of the two end positions.
    ///
    /// If the spans belong to different files, the method returns a clone of `self`.
    /// Offsets from different source files cannot form a meaningful combined span.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to merge with `self`.
    ///
    /// # Panics
    ///
    /// Panics if either span has a reversed start and end offset. The method checks
    /// this before it compares the file paths.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        assert!(self.is_valid() && other.is_valid(), "cannot merge invalid source spans");
        if self.file_path != other.file_path {
            return self.clone();
        }

        let merged_start = if self.start.offset() <= other.start.offset() { self.start } else { other.start };
        let merged_end = if self.end.offset() >= other.end.offset() { self.end } else { other.end };

        Self { file_path: self.file_path.clone(), start: merged_start, end: merged_end }
    }
}

impl std::fmt::Display for SourceSpan {
    /// Formats the span for human-readable output.
    ///
    /// Format: `<path>:line <start_line>:column <start_column>-line <end_line>:column <end_column>`
    ///
    /// If the span is empty (see [`SourceSpan::is_empty`]), the output has only the
    /// start position. The output has no end part.
    ///
    /// The path shows only the last 2 components, to keep the output short
    /// (see [`truncate_path`]).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let truncated_path = truncate_path(Path::new(&*self.file_path), 2);
        if self.is_empty() {
            write!(f, "{truncated_path}:{}", self.start)
        } else {
            write!(f, "{truncated_path}:{}-{}", self.start, self.end)
        }
    }
}

/// Truncates a path to show only the last `depth` components.
///
/// Use this function to show long paths in error messages in a short and
/// readable form.
///
/// # Arguments
/// * `path` - The original file path.
/// * `depth` - The number of trailing components to keep.
///
/// # Returns
/// The text of the truncated path:
/// - the full path, if the number of components is less than or equal to `depth`;
/// - otherwise `..` followed by the last `depth` components.
///
/// # Examples
/// ```
/// use std::path::Path;
/// use descar_core::location::source_span::truncate_path;
/// let path = if cfg!(unix) {
///         "/project/src/module/file.lang"
///     } else {
///         "C:\\project\\src\\module\\file.lang"
///     };
/// let expected = if cfg!(unix) { "../module/file.lang" } else { "..\\module\\file.lang" };
/// assert_eq!(truncate_path(Path::new(path), 2), expected);
/// ```
#[must_use]
pub fn truncate_path(path: &Path, depth: usize) -> String {
    let components: Vec<_> = path.components().collect();
    let len = components.len();

    let truncated = if len <= depth {
        PathBuf::from_iter(&components)
    } else {
        let tail = &components[len - depth..];
        PathBuf::from("..").join(PathBuf::from_iter(tail))
    };

    truncated.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::SourceSpan;
    use crate::location::source_location::SourceLocation;
    use std::sync::Arc;

    /// Creates a span in the file `file` with the given start and end byte offsets.
    fn span(file: &str, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(
            Arc::from(file),
            SourceLocation::new(1, start + 1, start, start, start, start),
            SourceLocation::new(1, end + 1, end, end, end, end),
        )
    }

    #[test]
    fn merge_preserves_complete_bounds_and_file() {
        let left = span("main.dr", 2, 5);
        let right = span("main.dr", 8, 12);
        let merged = left.merge(&right);

        assert_eq!(merged.file_path(), "main.dr");
        assert_eq!(merged.start().offset(), 2);
        assert_eq!(merged.end().offset(), 12);
        assert!(merged.is_valid());
    }

    #[test]
    fn spans_from_different_files_do_not_overlap() {
        let first = span("first.dr", 0, 4);
        let second = span("second.dr", 1, 3);

        assert!(!first.overlaps(&second));
    }

    #[test]
    fn merging_spans_from_different_files_preserves_the_original_span() {
        let original = span("first.dr", 0, 4);
        let other = span("second.dr", 1, 8);
        let merged = original.merge(&other);

        assert_eq!(merged.file_path(), "first.dr");
        assert_eq!(merged.start().offset(), 0);
        assert_eq!(merged.end().offset(), 4);
        assert_eq!(merged, original);
    }

    #[test]
    fn invalid_spans_are_not_empty_or_overlapping() {
        let reversed = span("main.dr", 8, 2);
        let valid = span("main.dr", 1, 10);

        assert!(!reversed.is_empty());
        assert!(!reversed.overlaps(&valid));
    }

    #[test]
    fn reversed_offsets_are_invalid() {
        let reversed = span("main.dr", 8, 2);
        assert!(!reversed.is_valid());
    }
}
