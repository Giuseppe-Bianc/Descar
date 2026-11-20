//! File size formatting types and display implementations.

use std::fmt;

use super::size_system::SizeSystem;

/// Represents a size value together with the unit suffix selected by a [`SizeSystem`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormattedSize {
    /// Numeric value in the selected unit.
    pub value: f64,
    /// Unit suffix appended to the value.
    pub suffix: &'static str,
}

impl FormattedSize {
    /// Creates a formatted size.
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value in the selected unit.
    /// * `suffix` - The unit suffix of the value.
    #[must_use]
    pub const fn new(value: f64, suffix: &'static str) -> Self {
        Self { value, suffix }
    }
}

impl fmt::Display for FormattedSize {
    /// Writes the value with two decimal places, a space, and the suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::file::FormattedSize;
    /// let size = FormattedSize::new(1.5, "KB");
    /// assert_eq!(size.to_string(), "1.50 KB");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.suffix)
    }
}

/// Pairs the SI and IEC [`FormattedSize`]s of the same byte count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormattedSizePair {
    /// Size formatted using SI decimal prefixes.
    pub si_size: FormattedSize,
    /// Size formatted using IEC binary prefixes.
    pub iec_size: FormattedSize,
}

impl FormattedSizePair {
    /// Creates a pair of formatted sizes.
    ///
    /// # Arguments
    ///
    /// * `si_size` - The size formatted with SI decimal prefixes.
    /// * `iec_size` - The size formatted with IEC binary prefixes.
    #[must_use]
    pub const fn new(si_size: FormattedSize, iec_size: FormattedSize) -> Self {
        Self { si_size, iec_size }
    }
}

impl fmt::Display for FormattedSizePair {
    /// Writes the SI size and the IEC size on one line.
    ///
    /// Each size is left-aligned in a field of 20 characters. One space separates the two fields.
    /// The output has no line break at the end.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let si_str = self.si_size.to_string();
        let iec_str = self.iec_size.to_string();
        write!(f, "{si_str:<20} {iec_str:<20}")
    }
}

/// Carries a raw byte count and provides formatting against a [`SizeSystem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileSizeInfo {
    /// Raw byte count.
    pub bytes: u64,
}

impl FileSizeInfo {
    /// Highest prefix index that [`FileSizeInfo::format`] can select. The index is 5.
    pub const MAX_PREFIX_INDEX: usize = 5;

    /// Creates a `FileSizeInfo` from a byte count.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw byte count.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self { bytes }
    }

    /// Formats the byte count using the given size system.
    ///
    /// The value starts in bytes. While the value is at least the base of the size system,
    /// the function divides the value by the base. The function divides at most five times.
    /// Thus the selected prefix index never exceeds `5`.
    ///
    /// The conversion of the byte count to `f64` can lose precision for very large byte counts.
    ///
    /// # Arguments
    ///
    /// * `sys` - The size system that gives the base and the unit prefixes.
    ///
    /// # Returns
    ///
    /// A [`FormattedSize`] with the divided value and the prefix at the index that equals the
    /// number of divisions.
    ///
    /// # Examples
    ///
    /// ```
    /// use descar_core::file::{FileSizeInfo, SizeSystems};
    /// let info = FileSizeInfo::new(1500);
    /// assert_eq!(info.format(&SizeSystems::SI_SYSTEM).to_string(), "1.50 KB");
    /// assert_eq!(FileSizeInfo::new(1024).format(&SizeSystems::IEC).to_string(), "1.00 KiB");
    /// ```
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
    /// Raw byte-count information.
    pub info: FileSizeInfo,
    /// SI decimal size system.
    pub si_sys: &'a SizeSystem,
    /// IEC binary size system.
    pub iec_sys: &'a SizeSystem,
}

impl<'a> FileSizeReport<'a> {
    /// Creates a report from byte count and size systems.
    ///
    /// # Arguments
    ///
    /// * `info` - The raw byte-count information.
    /// * `si_sys` - The size system for the SI size.
    /// * `iec_sys` - The size system for the IEC size.
    #[must_use]
    pub const fn new(info: FileSizeInfo, si_sys: &'a SizeSystem, iec_sys: &'a SizeSystem) -> Self {
        Self { info, si_sys, iec_sys }
    }

    /// Builds the SI/IEC formatted pair for `info`.
    ///
    /// The method formats `info` with `si_sys` and with `iec_sys`.
    ///
    /// # Returns
    ///
    /// A [`FormattedSizePair`] with the SI size and the IEC size.
    #[must_use]
    pub fn make_pair(&self) -> FormattedSizePair {
        FormattedSizePair::new(self.info.format(self.si_sys), self.info.format(self.iec_sys))
    }
}

impl fmt::Display for FileSizeReport<'_> {
    /// Writes the report as text with these lines:
    ///
    /// 1. The raw byte count, in the form `Bytes : <count>`.
    /// 2. A separator line of 41 hyphens.
    /// 3. The headings `SI` and `IEC`, each left-aligned in a field of 20 characters.
    /// 4. A second separator line.
    /// 5. The formatted pair from [`FileSizeReport::make_pair`].
    ///
    /// The output has no line break at the end.
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
