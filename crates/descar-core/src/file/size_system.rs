//! Size systems and unit definitions.

/// Describes a size system by its name, numeric base, and ordered list of unit prefixes.
///
/// The prefix list contains exactly six entries (indices 0 through 5).
/// The base must be at least `1.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct SizeSystem {
    name: &'static str,
    base: f64,
    prefixes: [&'static str; 6],
}

impl SizeSystem {
    /// Expected number of prefixes (indices 0..5).
    pub const PREFIX_COUNT: usize = 6;

    /// Minimum accepted value for `base`.
    pub const EXPECTED_BASE: f64 = 1.0;

    /// Creates a new `SizeSystem` with base validation.
    ///
    /// # Errors
    /// Returns `Err` if `base` is less than `1.0`.
    pub const fn new(name: &'static str, base: f64, prefixes: [&'static str; 6]) -> Result<Self, &'static str> {
        if base < Self::EXPECTED_BASE {
            return Err("base must be >= 1.0");
        }
        Ok(Self { name, base, prefixes })
    }

    #[inline]
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    #[inline]
    #[must_use]
    pub const fn base(&self) -> f64 {
        self.base
    }

    #[inline]
    #[must_use]
    pub const fn prefixes(&self) -> &[&'static str; 6] {
        &self.prefixes
    }
}

/// Catalog of built-in size systems used by the compiler reporting layer.
pub struct SizeSystems;

impl SizeSystems {
    /// Decimal system (SI), base 1000.
    pub const SI_SYSTEM: SizeSystem =
        SizeSystem { name: "SI", base: 1000.0, prefixes: ["B", "KB", "MB", "GB", "TB", "PB"] };

    /// Binary system (IEC), base 1024.
    pub const IEC: SizeSystem =
        SizeSystem { name: "IEC", base: 1024.0, prefixes: ["B", "KiB", "MiB", "GiB", "TiB", "PiB"] };
}
