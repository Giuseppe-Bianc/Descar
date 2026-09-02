use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::{Span, SpanError};

fn location(line: i32, column: i32, offset: i64) -> SourceLocation {
    SourceLocation::create(line, column, offset).expect("test location should be valid")
}

#[test]
fn creation_rejects_end_before_start() {
    let start = location(1, 5, 10);
    let end = location(1, 4, 9);
    assert_eq!(Span::create(start, end), Err(SpanError::EndBeforeStart { start_offset: 10, end_offset: 9 }));
}

#[test]
fn allows_equal_offsets_and_point_spans() {
    let loc = location(2, 3, 7);
    let created = Span::create(loc, loc).unwrap();
    let point = Span::point(loc);
    assert_eq!(created, point);
    assert_eq!(created.length(), 0);
    assert!(created.is_empty());
    assert!(!created.is_multiline());
}

#[test]
fn length_and_multiline_use_location_offsets_and_lines() {
    let start = location(2, 8, 4);
    let end = location(4, 2, 13);
    let span = Span::create(start, end).unwrap();
    assert_eq!(span.start(), start);
    assert_eq!(span.end(), end);
    assert_eq!(span.length(), 9);
    assert!(span.is_multiline());
}

#[test]
fn contains_uses_half_open_interval() {
    let span = Span::create(location(1, 1, 10), location(1, 6, 15)).unwrap();
    assert!(!span.contains(location(1, 1, 9)));
    assert!(span.contains(location(1, 2, 10)));
    assert!(span.contains(location(1, 5, 14)));
    assert!(!span.contains(location(1, 6, 15)));
}

#[test]
fn overlap_handles_partial_containment_and_touching_boundaries() {
    let base = Span::create(location(1, 1, 10), location(1, 1, 20)).unwrap();
    let left = Span::create(location(1, 1, 5), location(1, 1, 12)).unwrap();
    let contained = Span::create(location(1, 1, 12), location(1, 1, 18)).unwrap();
    let right = Span::create(location(1, 1, 18), location(1, 1, 25)).unwrap();
    let touching_left = Span::create(location(1, 1, 0), location(1, 1, 10)).unwrap();
    let touching_right = Span::create(location(1, 1, 20), location(1, 1, 30)).unwrap();
    let point = Span::point(location(1, 1, 10));
    assert!(base.overlaps(&left));
    assert!(base.overlaps(&contained));
    assert!(base.overlaps(&right));
    assert!(!base.overlaps(&touching_left));
    assert!(!base.overlaps(&touching_right));
    assert!(base.overlaps(&base));
    assert!(!point.overlaps(&base));
    assert!(!base.overlaps(&point));
}

#[test]
fn merge_returns_minimum_covering_span() {
    let first = Span::create(location(2, 1, 10), location(2, 5, 20)).unwrap();
    let second = Span::create(location(1, 8, 5), location(3, 2, 25)).unwrap();
    assert_eq!(first.merge(&second).start(), second.start());
    assert_eq!(first.merge(&second).end(), second.end());
    let nested = Span::create(location(2, 2, 12), location(2, 3, 18)).unwrap();
    assert_eq!(first.merge(&nested), first);
}

#[test]
fn extracts_ascii_and_empty_ranges() {
    let source = "hello world";
    let span = Span::create(location(1, 1, 0), location(1, 1, 5)).unwrap();
    assert_eq!(span.extract_from(source), Ok("hello"));
    let empty = Span::point(location(1, 1, 5));
    assert_eq!(empty.extract_from(source), Ok(""));
}

#[test]
fn extracts_utf8_only_at_valid_byte_boundaries() {
    let source = "a€b";
    let euro = Span::create(location(1, 1, 1), location(1, 1, 4)).unwrap();
    assert_eq!(euro.extract_from(source), Ok("€"));
    let invalid_boundary = Span::create(location(1, 1, 2), location(1, 1, 4)).unwrap();
    assert_eq!(invalid_boundary.extract_from(source), Err(SpanError::OffsetOutOfRange(4)));
}

#[test]
fn extract_rejects_end_beyond_source() {
    let span = Span::create(location(1, 1, 0), location(1, 1, 100)).unwrap();
    assert_eq!(span.extract_from("short"), Err(SpanError::OffsetOutOfRange(100)));
}
