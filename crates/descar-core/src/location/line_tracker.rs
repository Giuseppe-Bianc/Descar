use crate::location::{source_location::SourceLocation, source_span::SourceSpan};
use std::sync::Arc;

/// Tracks source positions through efficient offset-to-location conversion.
///
/// The tracker keeps the original UTF-8 source text and uses bottom-up tabulation
/// to precompute the prefix state at every Unicode scalar boundary:
/// - the UTF-8 byte offset;
/// - the corresponding UTF-16 code-unit offset;
/// - the starting byte and code-point offset of every logical line.
///
/// For the UTF-16 coordinate, the recurrence is
/// `utf16[i + 1] = utf16[i] + char[i].len_utf16()`, with `utf16[0] = 0`.
/// The resulting tables let `location_for` resolve a valid UTF-8 byte offset
/// using binary search while preserving the coordinate semantics used by Dersco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourcePosition {
    /// UTF-8 byte offset of this character boundary.
    byte_offset: usize,
    /// UTF-16 code-unit offset of this character boundary.
    utf16_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineStart {
    /// UTF-8 byte offset immediately after the preceding line terminator.
    byte_offset: usize,
    /// Absolute Unicode code-point offset at the beginning of this line.
    code_point_offset: usize,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineTracker {
    /// Original source code content, shared via Arc.
    source: Arc<str>,

    /// Starting byte and code-point offset of every logical line.
    ///
    /// The first element is always (0, 0).
    line_starts: Vec<LineStart>,

    /// Prefix position state for every UTF-8 character boundary, including EOF.
    ///
    /// The vector index is the absolute Unicode code-point offset. Each entry
    /// contains the byte and UTF-16 offsets for that boundary.
    positions: Vec<SourcePosition>,

    /// Path to source file, shared via Arc.
    file_path: Arc<str>,
}

impl LineTracker {
    /// Creates a new line tracker for the given source code.
    ///
    /// Line terminators follow the same semantics as Dersco:
    /// - LF (`\n`)
    /// - CR (`\r`)
    /// - CRLF (`\r\n`) as one logical line break
    /// - VT (`U+000B`)
    /// - FF (`U+000C`)
    /// - Unicode NEXT LINE (`U+0085`)
    /// - Unicode LINE SEPARATOR (`U+2028`)
    /// - Unicode PARAGRAPH SEPARATOR (`U+2029`)
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the source file. The tracker stores the path and uses it for spans.
    /// * `source` - The source text. The tracker takes ownership of the text.
    #[must_use]
    pub fn new(file_path: &str, source: String) -> Self {
        // Count once so the per-character prefix table can reserve its exact capacity.
        let char_count = source.chars().count();

        // Base state: zero bytes, zero Unicode code points, and zero UTF-16 units.
        let mut positions = Vec::with_capacity(char_count + 1);
        positions.push(SourcePosition { byte_offset: 0, utf16_offset: 0 });

        let mut line_starts = vec![LineStart { byte_offset: 0, code_point_offset: 0 }];

        let mut utf16_offset = 0;
        let mut code_point_offset = 0;
        let mut chars = source.char_indices().peekable();

        while let Some((byte_offset, ch)) = chars.next() {
            // Bottom-up DP transition from the previous character boundary.
            // D[i + 1] = D[i] + len_utf16(ch[i]).
            utf16_offset += ch.len_utf16();
            code_point_offset += 1;

            let next_byte_offset = byte_offset + ch.len_utf8();

            match ch {
                '\n' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}' => {
                    line_starts.push(LineStart {
                        byte_offset: next_byte_offset,
                        code_point_offset,
                    });
                }

                '\r' // CRLF is one logical line break. The line start is added
                    // when the following LF is consumed.
                    if chars.peek().is_none_or(|(_, next)| *next != '\n') => {
                    line_starts.push(LineStart {
                        byte_offset: next_byte_offset,
                        code_point_offset,
                    });
                }

                _ => {}
            }

            // Store the completed prefix state for the next character boundary.
            positions.push(SourcePosition {
                byte_offset: next_byte_offset,
                utf16_offset,
            });
        }

        Self {
            source: source.into(),
            line_starts,
            positions,
            file_path: Arc::from(file_path),
        }
    }

    /// Returns the complete source text of the tracker.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Converts a UTF-8 byte offset to its complete source location.
    ///
    /// The returned coordinates are:
    /// - `line`: 1-based logical line;
    /// - `column`: 1-based Unicode code-point column;
    /// - `offset`: UTF-8 byte offset;
    /// - `index`: UTF-16 code-unit offset;
    /// - `utf8_offset`: UTF-8 byte offset;
    /// - `code_point_offset`: absolute Unicode code-point offset.
    ///
    /// The offset can be equal to the length of the source text. This offset is the
    /// end-of-file position.
    ///
    /// # Arguments
    ///
    /// * `offset` - The UTF-8 byte offset in the source text.
    ///
    /// # Returns
    ///
    /// The [`SourceLocation`] of the offset.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is outside the source or is not a UTF-8
    /// character boundary.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::location::line_tracker::LineTracker;
    /// let tracker = LineTracker::new("main.dr", String::from("ab\ncd"));
    /// let location = tracker.location_for(4);
    /// assert_eq!(location.line(), 2);
    /// assert_eq!(location.column(), 2);
    /// assert_eq!(location.offset(), 4);
    /// ```
    #[must_use]
    pub fn location_for(&self, offset: usize) -> SourceLocation {
        assert!(
            offset <= self.source.len(),
            "Offset {offset} out of bounds for source of length {}",
            self.source.len()
        );

        assert!(self.source.is_char_boundary(offset), "Offset {offset} is not a UTF-8 character boundary");

        let line_index = match self
            .line_starts
            .binary_search_by_key(&offset, |line| line.byte_offset)
        {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let code_point_offset = match self
            .positions
            .binary_search_by_key(&offset, |position| position.byte_offset)
        {
            Ok(index) => index,
            Err(_) => unreachable!("offset was checked to be a valid UTF-8 boundary"),
        };

        let line_start = self.line_starts[line_index];
        let position = self.positions[code_point_offset];

        // The position table index is the absolute Unicode code-point offset.
        let column = code_point_offset - line_start.code_point_offset + 1;

        // Dersco's `index` corresponds to Java's UTF-16 `position`.
        let index = position.utf16_offset;
        // In Descar, `offset` is already the UTF-8 byte offset.
        let utf8_offset = offset;

        SourceLocation::new(line_index + 1, column, offset, index, utf8_offset, code_point_offset)
    }

    /// Creates a source span from a byte offset range.
    ///
    /// The range follows the half-open `[start, end)` convention.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of UTF-8 byte offsets in the source text.
    ///
    /// # Returns
    ///
    /// A [`SourceSpan`] with the file path of the tracker. The span starts at the location of
    /// `range.start` and ends at the location of `range.end`.
    ///
    /// # Panics
    ///
    /// Panics if `range.start` or `range.end` is not a valid offset. See
    /// [`LineTracker::location_for`].
    #[inline]
    #[must_use]
    pub fn span_for(&self, range: std::ops::Range<usize>) -> SourceSpan {
        SourceSpan::new(self.file_path.clone(), self.location_for(range.start), self.location_for(range.end))
    }

    /// Gets a specific logical line from the source, 1-based.
    ///
    /// The returned text does not include the line terminator. A source text that ends
    /// with a line terminator has an empty last line.
    ///
    /// # Arguments
    ///
    /// * `line_number` - The line number, starting at 1.
    ///
    /// # Returns
    ///
    /// * `Some` with the text of the line.
    /// * `None` if `line_number` is 0 or is larger than the number of lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::location::line_tracker::LineTracker;
    /// let tracker = LineTracker::new("main.dr", String::from("first\r\nsecond\n"));
    /// assert_eq!(tracker.get_line(1), Some("first"));
    /// assert_eq!(tracker.get_line(2), Some("second"));
    /// assert_eq!(tracker.get_line(3), Some(""));
    /// assert_eq!(tracker.get_line(4), None);
    /// assert_eq!(tracker.get_line(0), None);
    /// ```
    #[must_use]
    pub fn get_line(&self, line_number: usize) -> Option<&str> {
        let line_index = line_number.checked_sub(1)?;
        let start = self.line_starts.get(line_index)?.byte_offset;

        let end = self
            .line_starts
            .get(line_index + 1)
            .map_or(self.source.len(), |line| line.byte_offset);

        let line = &self.source[start..end];

        Some(
            line.strip_suffix("\r\n")
                .or_else(|| line.strip_suffix('\n'))
                .or_else(|| line.strip_suffix('\r'))
                .or_else(|| line.strip_suffix('\u{000B}'))
                .or_else(|| line.strip_suffix('\u{000C}'))
                .or_else(|| line.strip_suffix('\u{0085}'))
                .or_else(|| line.strip_suffix('\u{2028}'))
                .or_else(|| line.strip_suffix('\u{2029}'))
                .unwrap_or(line),
        )
    }
}
