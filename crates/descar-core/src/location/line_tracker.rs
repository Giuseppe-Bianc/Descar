use crate::location::{source_location::SourceLocation, source_span::SourceSpan};
use std::sync::Arc;

/// Tracks source positions through efficient offset-to-location conversion.
///
/// The tracker keeps the original UTF-8 source text and precomputes:
/// - line starts in UTF-8 byte offsets;
/// - the UTF-8 byte offset of every Unicode scalar value;
/// - the corresponding UTF-16 offset of every Unicode scalar value.
///
/// This allows `location_for` to resolve a valid UTF-8 byte offset in O(log n)
/// time while preserving the coordinate semantics used by Dersco.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineTracker {
    /// Original source code content, shared via Arc.
    source: Arc<str>,

    /// Starting UTF-8 byte offset of every logical line.
    ///
    /// The first element is always 0.
    line_starts: Vec<usize>,

    /// UTF-8 byte offsets of Unicode scalar value boundaries.
    ///
    /// Contains one additional entry for EOF.
    char_starts: Vec<usize>,

    /// UTF-16 code-unit offset corresponding to every `char_starts` entry.
    ///
    /// The final entry is the UTF-16 length of the complete source.
    utf16_offsets: Vec<usize>,

    /// Code-point index corresponding to each line start.
    line_char_starts: Vec<usize>,

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
    /// - Unicode LINE SEPARATOR (`U+2028`)
    /// - Unicode PARAGRAPH SEPARATOR (`U+2029`)
    #[must_use]
    pub fn new(file_path: &str, source: String) -> Self {
        let char_count = source.chars().count();

        let mut line_starts = vec![0];
        let mut line_char_starts = vec![0];

        let mut char_starts = Vec::with_capacity(char_count + 1);
        let mut utf16_offsets = Vec::with_capacity(char_count + 1);

        let mut utf16_offset = 0;
        let mut code_point_offset = 0;

        let mut chars = source.char_indices().peekable();

        while let Some((byte_offset, ch)) = chars.next() {
            char_starts.push(byte_offset);
            utf16_offsets.push(utf16_offset);

            utf16_offset += ch.len_utf16();
            code_point_offset += 1;

            match ch {
                '\n' |'\u{0085}' | '\u{2028}' | '\u{2029}' => {
                    line_starts.push(byte_offset + ch.len_utf8());
                    line_char_starts.push(code_point_offset);
                }

                '\r' // CRLF is one logical line break. The line start is added
                    // when the following LF is consumed.
                    if chars.peek().is_none_or(|(_, next)| *next != '\n') => {
                        line_starts.push(byte_offset + ch.len_utf8());
                        line_char_starts.push(code_point_offset);
                    }

                _ => {}
            }
        }

        // EOF is a valid UTF-8 and UTF-16 position.
        char_starts.push(source.len());
        utf16_offsets.push(utf16_offset);

        Self {
            source: source.into(),
            line_starts,
            char_starts,
            utf16_offsets,
            line_char_starts,
            file_path: Arc::from(file_path),
        }
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
    /// # Panics
    ///
    /// Panics if `offset` is outside the source or is not a UTF-8
    /// character boundary.
    #[must_use]
    pub fn location_for(&self, offset: usize) -> SourceLocation {
        assert!(
            offset <= self.source.len(),
            "Offset {offset} out of bounds for source of length {}",
            self.source.len()
        );

        assert!(self.source.is_char_boundary(offset), "Offset {offset} is not a UTF-8 character boundary");

        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let Ok(code_point_offset) = self.char_starts.binary_search(&offset) else {
            unreachable!("offset was checked to be a valid UTF-8 boundary");
        };

        // Dersco columns are counted in Unicode code points, not UTF-8 bytes.
        let column = code_point_offset - self.line_char_starts[line_index] + 1;

        // Dersco's `index` corresponds to Java's UTF-16 `position`.
        let index = self.utf16_offsets[code_point_offset];

        // In Descar, `offset` is already the UTF-8 byte offset.
        let utf8_offset = offset;

        SourceLocation::new(line_index + 1, column, offset, index, utf8_offset, code_point_offset)
    }

    /// Creates a source span from a byte offset range.
    ///
    /// The range follows the half-open `[start, end)` convention.
    #[inline]
    #[must_use]
    pub fn span_for(&self, range: std::ops::Range<usize>) -> SourceSpan {
        SourceSpan::new(self.file_path.clone(), self.location_for(range.start), self.location_for(range.end))
    }

    /// Gets a specific logical line from the source, 1-based.
    #[must_use]
    pub fn get_line(&self, line_number: usize) -> Option<&str> {
        let line_index = line_number.checked_sub(1)?;
        let start = *self.line_starts.get(line_index)?;

        let end = self.line_starts.get(line_index + 1).copied().unwrap_or(self.source.len());

        let line = &self.source[start..end];

        Some(
            line.strip_suffix("\r\n")
                .or_else(|| line.strip_suffix('\n'))
                .or_else(|| line.strip_suffix('\r'))
                .or_else(|| line.strip_suffix('\u{0085}'))
                .or_else(|| line.strip_suffix('\u{2028}'))
                .or_else(|| line.strip_suffix('\u{2029}'))
                .unwrap_or(line),
        )
    }
}
