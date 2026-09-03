//! Representation of a span (extent) within a source file.
//!
//! A [`SourceSpan`] identifies a portion of source text via an inclusive
//! start position and an exclusive end position, together with the path of
//! the file it refers to. It is used to associate tokens, AST nodes, and
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
/// This convention allows representing empty spans (`start == end`, see
/// [`SourceSpan::point`]) and simplifies computing length and merging spans
/// (see [`SourceSpan::length`] and [`SourceSpan::merge`]).
///
/// Each span also holds the path of the source file (`file_path`), shared
/// via [`Arc<str>`] to avoid string duplication when many spans reference
/// the same file.
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
    /// No validation is performed: it is the caller's responsibility to
    /// ensure that `start` precedes (or coincides with) `end`.
    ///
    /// # Parameters
    /// * `file_path` - path of the source file.
    /// * `start` - start position (inclusive).
    /// * `end` - end position (exclusive).
    #[must_use]
    pub const fn new(file_path: Arc<str>, start: SourceLocation, end: SourceLocation) -> Self {
        Self { file_path, start, end }
    }

    /// Creates a zero-length span at the given position.
    ///
    /// A "point" span is always valid, since `start` and `end` coincide.
    /// Useful for representing positions (e.g. an error location) without
    /// covering any text.
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

    /// Length of the span in UTF-8 bytes, consistent with [`SourceLocation::offset`]
    /// (which is a 0-based byte offset) and with the byte ranges used by
    /// [`SourceSpan::extract_from`].
    ///
    /// Note: this follows Rust's `str::len()` semantics — a multi-byte
    /// character such as `'€'` (U+20AC) contributes **3** to the length,
    /// not 1.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.end.offset().saturating_sub(self.start.offset())
    }

    /// Indicates whether the span has zero length (i.e. `start == end`).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Indicates whether the span spans multiple lines, by comparing the
    /// line of `start` with the line of `end`.
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.start.line() != self.end.line()
    }

    /// Indicates whether the given location falls within the half-open
    /// interval `[start, end)` of this span.
    #[must_use]
    pub const fn contains(&self, location: SourceLocation) -> bool {
        location.offset() >= self.start.offset() && location.offset() < self.end.offset()
    }

    /// Indicates whether two spans share at least one character, i.e.
    /// whether their `[start, end)` intervals intersect.
    ///
    /// Two adjacent spans (e.g. `[0, 3)` and `[3, 5)`) are not considered
    /// overlapping.
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start.offset() < other.end.offset() && other.start.offset() < self.end.offset()
    }

    /// Returns the smallest span that contains both spans (`self` and
    /// `other`), by taking the minimum of the start positions and the
    /// maximum of the end positions.
    ///
    /// The resulting `file_path` is that of `self`: it is assumed that both
    /// spans belong to the same file.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let merged_start = if self.start.offset() <= other.start.offset() { self.start } else { other.start };
        let merged_end = if self.end.offset() >= other.end.offset() { self.end } else { other.end };

        Self { file_path: self.file_path.clone(), start: merged_start, end: merged_end }
    }
}

impl std::fmt::Display for SourceSpan {
    /// Formats the span for human-readable output.
    ///
    /// Format: `[truncated_path]:line [start_line]:column [start_col] - line [end_line]:column [end_col]`
    ///
    /// If the span is empty (`is_empty()`), only the `start` position is
    /// shown, omitting the `end` part.
    ///
    /// Paths are truncated to show only the last 2 components, for brevity
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
/// Useful for displaying long paths in error messages in a compact and
/// readable way.
///
/// # Arguments
/// * `path` - original file path.
/// * `depth` - number of trailing components to preserve.
///
/// # Returns
/// String representation of the truncated path:
/// - the full path, if the component count is <= `depth`;
/// - otherwise `..` followed by the last `depth` components.
///
/// # Examples
/// ```
/// use std::path::Path;
/// use descar_core::location::source_span::truncate_path;
/// let path = if cfg!(unix) {
///         "/project/src/module/file.lang"
///     } else {
///         "C:\\project\\src.\\module\\file.lang"
///     };
/// let path = Path::new("/project/src/module/file.lang");
/// let expected = if cfg!(unix) { "../module/file.lang" } else { "..\\module\\file.lang" };
/// assert_eq!(truncate_path(path, 2), expected);
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
