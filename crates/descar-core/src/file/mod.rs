//! File size utilities.
//!
//! The module contains the types that format a byte count in SI and IEC units.
//! The [`size_system`] module defines the size systems and their unit prefixes.
//! The [`file_size`] module defines the types that format a byte count.

pub mod file_size;
pub mod size_system;

pub use file_size::{FileSizeInfo, FileSizeReport, FormattedSize, FormattedSizePair};
pub use size_system::{SizeSystem, SizeSystems};
