use std::sync::Arc;

use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::SourceSpan as Span;

const fn location(line: usize, column: usize, offset: usize) -> SourceLocation {
    SourceLocation::new(line, column, offset, 0, usize::MAX, usize::MAX)
}

#[test]
fn allows_equal_offsets_and_point_spans() {
    let loc = location(2, 3, 7);
    let created = Span::new(Arc::from("asd/dd.dr"), loc, loc);
    let point = Span::point(Arc::from("asd/dd.dr"), loc);
    assert_eq!(created, point);
    assert_eq!(created.length(), 0);
    assert!(created.is_empty());
    assert!(!created.is_multiline());
}

#[test]
fn length_and_multiline_use_location_offsets_and_lines() {
    let start = location(2, 8, 4);
    let end = location(4, 2, 13);
    let span = Span::new(Arc::from("asd/dd.dr"), start, end);
    assert_eq!(span.start(), start);
    assert_eq!(span.end(), end);
    assert_eq!(span.length(), 9);
    assert!(span.is_multiline());
}

#[test]
fn contains_uses_half_open_interval() {
    let span = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 10), location(1, 6, 15));
    assert!(!span.contains(location(1, 1, 9)));
    assert!(span.contains(location(1, 2, 10)));
    assert!(span.contains(location(1, 5, 14)));
    assert!(!span.contains(location(1, 6, 15)));
}

#[test]
fn overlap_handles_partial_containment_and_touching_boundaries() {
    let base = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 10), location(1, 1, 20));
    let left = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 5), location(1, 1, 12));
    let contained = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 12), location(1, 1, 18));
    let right = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 18), location(1, 1, 25));
    let touching_left = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 0), location(1, 1, 10));
    let touching_right = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 20), location(1, 1, 30));
    let point = Span::point(Arc::from("asd/dd.dr"), location(1, 1, 10));
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
    let first = Span::new(Arc::from("asd/dd.dr"), location(2, 1, 10), location(2, 5, 20));
    let second = Span::new(Arc::from("asd/dd.dr"), location(1, 8, 5), location(3, 2, 25));
    assert_eq!(first.merge(&second).start(), second.start());
    assert_eq!(first.merge(&second).end(), second.end());
    let nested = Span::new(Arc::from("asd/dd.dr"), location(2, 2, 12), location(2, 3, 18));
    assert_eq!(first.merge(&nested), first);
}

#[test]
fn extracts_ascii_and_empty_ranges() {
    let source = "hello world";
    let span = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 0), location(1, 1, 5));
    assert_eq!(span.extract_from(source), "hello");
    let empty = Span::point(Arc::from("asd/dd.dr"), location(1, 1, 5));
    assert_eq!(empty.extract_from(source), "");
}

#[test]
fn length_is_utf8_byte_count_for_non_ascii_span() {
    // "a€b": 'a' = 1 byte, '€' (U+20AC) = 3 bytes, 'b' = 1 byte → total 5 bytes.
    let source = "a€b";
    assert_eq!(source.len(), 5, "sanity-check: source is 5 UTF-8 bytes");

    // Whole string: offsets 0..5 → length 5.
    let whole = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 0), location(1, 4, 5));
    assert_eq!(whole.length(), 5);
    assert_eq!(whole.extract_from(source), "a€b");

    // Euro sign only: offsets 1..4 → length 3.
    let euro = Span::new(Arc::from("asd/dd.dr"), location(1, 2, 1), location(1, 3, 4));
    assert_eq!(euro.length(), 3);
    assert_eq!(euro.extract_from(source), "€");
}
