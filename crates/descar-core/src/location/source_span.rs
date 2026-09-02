//! Extent of a token in the source text.

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

use crate::error::internal::SpanError;
use crate::location::source_location::SourceLocation;

/// Extent of a token in the source text.
///
/// `start` è inclusivo, `end` è esclusivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub const fn try_new(start: SourceLocation, end: SourceLocation) -> Result<Self, SpanError> {
        if end.offset() < start.offset() {
            return Err(SpanError::EndBeforeStart { start_offset: start.offset(), end_offset: end.offset() });
        }
        Ok(Self { start, end })
    }

    /// Crea un nuovo span.
    ///
    /// # Errors
    ///
    /// Returns [`SpanError::EndBeforeStart`] if `end.offset() < start.offset()`.
    pub const fn create(start: SourceLocation, end: SourceLocation) -> Result<Self, SpanError> {
        Self::try_new(start, end)
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
    pub const fn length(&self) -> i64 {
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
    /// # Errors
    ///
    /// Returns [`SpanError::OffsetOutOfRange`] if either offset does not fit into
    /// `usize` or does not fall on a valid UTF-8 character boundary.
    pub fn extract_from<'a>(&self, source: &'a str) -> Result<&'a str, SpanError> {
        let start = usize::try_from(self.start.offset())
            .map_err(|_| SpanError::OffsetOutOfRange { offset: self.start.offset() })?;

        let end = usize::try_from(self.end.offset())
            .map_err(|_| SpanError::OffsetOutOfRange { offset: self.end.offset() })?;

        source.get(start..end).ok_or_else(|| SpanError::OffsetOutOfRange { offset: self.end.offset() })
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() { write!(f, "{}", self.start) } else { write!(f, "{}-{}", self.start, self.end) }
    }
}

/// Ordinamento deterministico coerente con [`PartialEq`].
///
/// Delega a [`SourceLocation::cmp`], che confronta tutti i campi nell'ordine
/// di dichiarazione (`line → column → offset → index → utf8_offset →
/// code_point_offset`). Due span distinti secondo `==` non risultano mai
/// uguali secondo `cmp`.
impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start).then_with(|| self.end.cmp(&other.end))
    }
}
