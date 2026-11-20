//! Source positions, spans, and line tracking.
//!
//! * [`line_tracker`]: Conversion of byte offsets to source locations, and access to source lines.
//! * [`source_location`]: One position in the source text.
//! * [`source_span`]: The extent of a token or construct in a source file.

/// Tracks UTF-8, UTF-16, and line coordinates of the source text.
pub mod line_tracker;
/// A single source location.
pub mod source_location;
pub mod source_span;
