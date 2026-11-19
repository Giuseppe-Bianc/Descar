//! Source positions, spans, and line tracking.

/// Efficient UTF-8, UTF-16, and line-coordinate tracking.
pub mod line_tracker;
/// A single logical source location.
pub mod source_location;
pub mod source_span;
