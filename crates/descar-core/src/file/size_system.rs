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
    /// Number of unit prefixes in a size system. The prefix indices are 0 to 5.
    pub const PREFIX_COUNT: usize = 6;

    /// Minimum accepted value for `base`.
    pub const EXPECTED_BASE: f64 = 1.0;

    /// Creates a new `SizeSystem` with base validation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the size system.
    /// * `base` - The number by which each unit is larger than the previous unit.
    /// * `prefixes` - The unit prefixes, from the smallest unit to the largest unit.
    ///
    /// # Returns
    ///
    /// A `SizeSystem` with the given name, base, and prefixes.
    ///
    /// # Errors
    ///
    /// Returns `Err` with the message `base must be >= 1.0` if `base` is less than `1.0`.
    pub const fn new(name: &'static str, base: f64, prefixes: [&'static str; 6]) -> Result<Self, &'static str> {
        if base < Self::EXPECTED_BASE {
            return Err("base must be >= 1.0");
        }
        Ok(Self { name, base, prefixes })
    }

    /// Returns the name of the size system.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the base of the size system.
    #[inline]
    #[must_use]
    pub const fn base(&self) -> f64 {
        self.base
    }

    /// Returns the unit prefixes of the size system, from the smallest unit to the largest unit.
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
    ///
    /// The prefixes are `B`, `KB`, `MB`, `GB`, `TB`, and `PB`.
    pub const SI_SYSTEM: SizeSystem =
        SizeSystem { name: "SI", base: 1000.0, prefixes: ["B", "KB", "MB", "GB", "TB", "PB"] };

    /// Binary system (IEC), base 1024.
    ///
    /// The prefixes are `B`, `KiB`, `MiB`, `GiB`, `TiB`, and `PiB`.
    pub const IEC: SizeSystem =
        SizeSystem { name: "IEC", base: 1024.0, prefixes: ["B", "KiB", "MiB", "GiB", "TiB", "PiB"] };
}
