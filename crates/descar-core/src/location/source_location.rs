use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

/// Sentinel value for optional fields that have not been computed.
pub const UNKNOWN: i64 = -1;

/// Minimum value for 1-based fields.
const MIN_1_BASED: i32 = 1;

/// Minimum value for 0-based fields.
const MIN_0_BASED: i64 = 0;

/// Error raised when constructing an invalid [`SourceLocation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceLocationError {
    InvalidLine(i32),
    InvalidColumn(i32),
    InvalidOffset(i64),
    InvalidIndex(i32),
    InvalidUtf8Offset(i64),
    InvalidCodePointOffset(i64),
    OffsetTooLarge(i64),
}

impl fmt::Display for SourceLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLine(v) => write!(f, "line must be >= 1 (1-based), got: {v}"),
            Self::InvalidColumn(v) => write!(f, "column must be >= 1 (1-based), got: {v}"),
            Self::InvalidOffset(v) => write!(f, "offset must be >= 0, got: {v}"),
            Self::InvalidIndex(v) => write!(f, "index must be >= 0, got: {v}"),
            Self::InvalidUtf8Offset(v) => write!(f, "utf8Offset must be >= 0 or UNKNOWN, got: {v}"),
            Self::InvalidCodePointOffset(v) => {
                write!(f, "codePointOffset must be >= 0 or UNKNOWN, got: {v}")
            }
            Self::OffsetTooLarge(v) => write!(f, "offset {v} does not fit into an i32 index"),
        }
    }
}

impl std::error::Error for SourceLocationError {}

/// Source position within a compilation unit.
///
/// Equivalente immutabile del `record` Java, confrontabile per `offset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    line: i32,
    column: i32,
    offset: i64,
    index: i32,
    utf8_offset: i64,
    code_point_offset: i64,
}

impl SourceLocation {
    /// Equivalente del "compact constructor" Java: valida tutti i campi.
    pub fn try_new(
        line: i32, column: i32, offset: i64, index: i32, utf8_offset: i64, code_point_offset: i64,
    ) -> Result<Self, SourceLocationError> {
        if line < MIN_1_BASED {
            return Err(SourceLocationError::InvalidLine(line));
        }
        if column < MIN_1_BASED {
            return Err(SourceLocationError::InvalidColumn(column));
        }
        if offset < MIN_0_BASED {
            return Err(SourceLocationError::InvalidOffset(offset));
        }
        if (index as i64) < MIN_0_BASED {
            return Err(SourceLocationError::InvalidIndex(index));
        }
        if utf8_offset != UNKNOWN && utf8_offset < MIN_0_BASED {
            return Err(SourceLocationError::InvalidUtf8Offset(utf8_offset));
        }
        if code_point_offset != UNKNOWN && code_point_offset < MIN_0_BASED {
            return Err(SourceLocationError::InvalidCodePointOffset(code_point_offset));
        }

        Ok(Self { line, column, offset, index, utf8_offset, code_point_offset })
    }

    /// Crea una posizione minimale con line, column e offset soltanto.
    ///
    /// `index` viene derivato da `offset`, come `Math.toIntExact` in Java.
    pub fn create(line: i32, column: i32, offset: i64) -> Result<Self, SourceLocationError> {
        let index = i32::try_from(offset).map_err(|_| SourceLocationError::OffsetTooLarge(offset))?;
        Self::try_new(line, column, offset, index, UNKNOWN, UNKNOWN)
    }

    /// Crea una posizione completa con tutte le varianti di offset.
    pub fn create_full(
        line: i32, column: i32, offset: i64, index: i32, utf8_offset: i64, code_point_offset: i64,
    ) -> Result<Self, SourceLocationError> {
        Self::try_new(line, column, offset, index, utf8_offset, code_point_offset)
    }

    /// Numero di riga (1-based).
    pub fn line(&self) -> i32 {
        self.line
    }

    /// Numero di colonna (1-based).
    pub fn column(&self) -> i32 {
        self.column
    }

    /// Offset in byte (0-based).
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// Indice di carattere (0-based).
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Offset UTF-8, o `UNKNOWN` se non calcolato.
    pub fn utf8_offset(&self) -> i64 {
        self.utf8_offset
    }

    /// Offset in code point, o `UNKNOWN` se non calcolato.
    pub fn code_point_offset(&self) -> i64 {
        self.code_point_offset
    }

    /// Restituisce una copia con il nuovo offset UTF-8.
    pub fn with_utf8_offset(&self, new_utf8_offset: i64) -> Self {
        Self { utf8_offset: new_utf8_offset, ..*self }
    }

    /// Restituisce una copia con il nuovo offset in code point.
    pub fn with_code_point_offset(&self, cp_offset: i64) -> Self {
        Self { code_point_offset: cp_offset, ..*self }
    }

    /// Indica se l'offset UTF-8 è stato calcolato.
    pub fn has_utf8_offset(&self) -> bool {
        self.utf8_offset != UNKNOWN
    }

    /// Indica se l'offset in code point è stato calcolato.
    pub fn has_code_point_offset(&self) -> bool {
        self.code_point_offset != UNKNOWN
    }
}

impl PartialOrd for SourceLocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceLocation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}:column {}", self.line, self.column)
    }
}
