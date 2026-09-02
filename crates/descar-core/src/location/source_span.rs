//! Extent of a token in the source text.

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

use crate::location::source_location::SourceLocation;

/// Errore sollevato durante la costruzione o l'uso di uno [`Span`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanError {
    /// `end` precede `start`.
    EndBeforeStart { start_offset: i64, end_offset: i64 },
    /// Un offset non entra in un `usize` (per l'estrazione del testo).
    OffsetOutOfRange(i64),
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndBeforeStart { start_offset, end_offset } => {
                write!(f, "end offset ({end_offset}) must not precede start offset ({start_offset})")
            }
            Self::OffsetOutOfRange(v) => write!(f, "offset {v} does not fit into usize"),
        }
    }
}

impl std::error::Error for SpanError {}

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
    pub fn try_new(start: SourceLocation, end: SourceLocation) -> Result<Self, SpanError> {
        if end.offset() < start.offset() {
            return Err(SpanError::EndBeforeStart { start_offset: start.offset(), end_offset: end.offset() });
        }
        Ok(Self { start, end })
    }

    /// Crea un nuovo span.
    pub fn create(start: SourceLocation, end: SourceLocation) -> Result<Self, SpanError> {
        Self::try_new(start, end)
    }

    /// Crea uno span di lunghezza zero nella posizione data.
    pub fn point(location: SourceLocation) -> Self {
        // Uno span punto è sempre valido: start == end.
        Self { start: location, end: location }
    }

    /// Posizione di inizio (inclusiva).
    pub fn start(&self) -> SourceLocation {
        self.start
    }

    /// Posizione di fine (esclusiva).
    pub fn end(&self) -> SourceLocation {
        self.end
    }

    /// Lunghezza in unità di codice UTF-16 (coerente con `String::len` di Java,
    /// non con `str::len()` di Rust che è in byte).
    pub fn length(&self) -> i64 {
        self.end.offset() - self.start.offset()
    }

    /// Indica se lo span ha lunghezza zero.
    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Indica se lo span si estende su più righe.
    pub fn is_multiline(&self) -> bool {
        self.start.line() != self.end.line()
    }

    /// Indica se la posizione data cade nell'intervallo `[start, end)`.
    pub fn contains(&self, location: SourceLocation) -> bool {
        location.offset() >= self.start.offset() && location.offset() < self.end.offset()
    }

    /// Indica se due span condividono almeno un carattere.
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start.offset() < other.end.offset() && other.start.offset() < self.end.offset()
    }

    /// Restituisce lo span minimo che contiene entrambi gli span.
    pub fn merge(&self, other: &Span) -> Span {
        let merged_start = if self.start.offset() <= other.start.offset() { self.start } else { other.start };
        let merged_end = if self.end.offset() >= other.end.offset() { self.end } else { other.end };
        // L'ordinamento è garantito dalla monotonia di start/end originali.
        Span { start: merged_start, end: merged_end }
    }

    /// Estrae il testo coperto da questo span dalla sorgente data.
    ///
    /// # Errori
    /// Ritorna [`SpanError::OffsetOutOfRange`] se gli offset non entrano in `usize`
    /// o non cadono su un confine di carattere UTF-8 valido.
    pub fn extract_from<'a>(&self, source: &'a str) -> Result<&'a str, SpanError> {
        let start =
            usize::try_from(self.start.offset()).map_err(|_| SpanError::OffsetOutOfRange(self.start.offset()))?;
        let end = usize::try_from(self.end.offset()).map_err(|_| SpanError::OffsetOutOfRange(self.end.offset()))?;

        source.get(start..end).ok_or(SpanError::OffsetOutOfRange(self.end.offset()))
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() { write!(f, "{}", self.start) } else { write!(f, "{}-{}", self.start, self.end) }
    }
}
