//! File size formatting types and display implementations.

use std::fmt;

use super::size_system::SizeSystem;

/// Represents a size value together with the unit suffix selected by a [`SizeSystem`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormattedSize {
    pub value: f64,
    pub suffix: &'static str,
}

impl FormattedSize {
    #[must_use]
    pub const fn new(value: f64, suffix: &'static str) -> Self {
        Self { value, suffix }
    }
}

impl fmt::Display for FormattedSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.suffix)
    }
}

/// Pairs the SI and IEC [`FormattedSize`]s of the same byte count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormattedSizePair {
    pub si_size: FormattedSize,
    pub iec_size: FormattedSize,
}

impl FormattedSizePair {
    #[must_use]
    pub const fn new(si_size: FormattedSize, iec_size: FormattedSize) -> Self {
        Self { si_size, iec_size }
    }
}

impl fmt::Display for FormattedSizePair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let si_str = self.si_size.to_string();
        let iec_str = self.iec_size.to_string();
        write!(f, "{si_str:<20} {iec_str:<20}")
    }
}

/// Carries a raw byte count and provides formatting against a [`SizeSystem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileSizeInfo {
    pub bytes: u64,
}

impl FileSizeInfo {
    /// Maximum reachable index in the prefixes array (0..5).
    pub const MAX_PREFIX_INDEX: usize = 5;

    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self { bytes }
    }

    /// Formats the byte count using the given size system.
    ///
    /// The value starts in bytes and is repeatedly divided by the system base while it is at
    /// least the base. At most five divisions are performed, so the selected prefix never exceeds
    /// index `5`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn format(&self, sys: &SizeSystem) -> FormattedSize {
        let mut value = self.bytes as f64;
        let mut index = 0;

        while index < Self::MAX_PREFIX_INDEX && value >= sys.base() {
            value /= sys.base();
            index += 1;
        }

        FormattedSize::new(value, sys.prefixes()[index])
    }
}

/// Combines a byte-count snapshot with the SI and IEC size systems used to format it.
#[derive(Debug, Clone, PartialEq)]
pub struct FileSizeReport<'a> {
    pub info: FileSizeInfo,
    pub si_sys: &'a SizeSystem,
    pub iec_sys: &'a SizeSystem,
}

impl<'a> FileSizeReport<'a> {
    #[must_use]
    pub const fn new(info: FileSizeInfo, si_sys: &'a SizeSystem, iec_sys: &'a SizeSystem) -> Self {
        Self { info, si_sys, iec_sys }
    }

    /// Builds the SI/IEC formatted pair for `info`.
    #[must_use]
    pub fn make_pair(&self) -> FormattedSizePair {
        FormattedSizePair::new(self.info.format(self.si_sys), self.info.format(self.iec_sys))
    }
}

impl fmt::Display for FileSizeReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pair = self.make_pair();
        let separator = "-".repeat(41);

        writeln!(f, "Bytes : {}", self.info.bytes)?;
        writeln!(f, "{separator}")?;
        writeln!(f, "{:<20} {:<20}", "SI", "IEC")?;
        writeln!(f, "{separator}")?;
        write!(f, "{pair}")
    }
}
