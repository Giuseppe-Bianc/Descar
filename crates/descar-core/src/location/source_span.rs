use std::fmt;

use crate::location::source_location::SourceLocation;

/// Extent of a token in the source text.
///
/// `start` è inclusivo, `end` è esclusivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default, Hash)]
pub struct Span {
    start: SourceLocation,
    end: SourceLocation,
}

impl Span {
    /// Compact constructor equivalente: valida l'ordinamento start/end.
    ///
    /// # Errors
    ///
    /// Returns [`SpanError::EndBeforeStart`] if `end.offset() < start.offset()`.
    #[must_use]
    pub const fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    /// Crea uno span di lunghezza zero nella posizione data.
    #[must_use]
    pub const fn point(location: SourceLocation) -> Self {
        // Uno span punto è sempre valido: start == end.
        Self { start: location, end: location }
    }

    /// Posizione di inizio (inclusiva).
    #[must_use]
    pub const fn start(&self) -> SourceLocation {
        self.start
    }

    /// Posizione di fine (esclusiva).
    #[must_use]
    pub const fn end(&self) -> SourceLocation {
        self.end
    }

    /// Length of the span in UTF-8 bytes, consistent with [`SourceLocation::offset`]
    /// (which is a 0-based byte offset) and with the byte ranges used by
    /// [`Span::extract_from`].
    ///
    /// Note: this is Rust's `str::len()` semantics — a multi-byte character
    /// such as `'€'` (U+20AC) contributes **3** to the length, not 1.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.end.offset() - self.start.offset()
    }

    /// Indica se lo span ha lunghezza zero.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Indica se lo span si estende su più righe.
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.start.line() != self.end.line()
    }

    /// Indica se la posizione data cade nell'intervallo `[start, end)`.
    #[must_use]
    pub const fn contains(&self, location: SourceLocation) -> bool {
        location.offset() >= self.start.offset() && location.offset() < self.end.offset()
    }

    /// Indica se due span condividono almeno un carattere.
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start.offset() < other.end.offset() && other.start.offset() < self.end.offset()
    }

    /// Restituisce lo span minimo che contiene entrambi gli span.
    #[must_use]
    pub const fn merge(&self, other: &Self) -> Self {
        let merged_start = if self.start.offset() <= other.start.offset() { self.start } else { other.start };
        let merged_end = if self.end.offset() >= other.end.offset() { self.end } else { other.end };
        // L'ordinamento è garantito dalla monotonia di start/end originali.
        Self { start: merged_start, end: merged_end }
    }

    /// Estrae il testo coperto da questo span dalla sorgente data.
    ///
    /// # Panics
    ///
    /// Panics if the span's bounds are outside `source` or do not correspond to
    /// valid UTF-8 character boundaries.
    #[must_use]
    pub fn extract_from<'a>(&self, source: &'a str) -> &'a str {
        let start = self.start.offset();

        let end = self.end.offset();

        source.get(start..end).unwrap_or_else(|| panic!("Span bounds are out of range"))
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() { write!(f, "{}", self.start) } else { write!(f, "{}-{}", self.start, self.end) }
    }
}
