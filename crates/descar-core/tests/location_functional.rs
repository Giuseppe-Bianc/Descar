use std::path::PathBuf;

use descar_core::location::source_id::SourceId;
use descar_core::location::source_location::{SourceLocation, SourceLocationError, UNKNOWN};
use descar_core::location::source_span::{Span, SpanError};

fn location(line: i32, column: i32, offset: i64) -> SourceLocation {
    SourceLocation::create(line, column, offset).expect("test location should be valid")
}

#[test]
fn source_id_file_path_preserves_path() {
    let id = SourceId::file_path(PathBuf::from("src/main.dr"));
    assert_eq!(id.identifier(), "src/main.dr");
    assert_eq!(id.describe(), "file: src/main.dr");
    assert_eq!(id.to_string(), "file: src/main.dr");
}

#[test]
fn source_id_virtual_resource_rejects_blank_input() {
    for uri in ["", "   ", "\t\n"] {
        assert_eq!(SourceId::virtual_resource(uri.to_owned()), Err("uri must not be blank"));
    }
    let id = SourceId::virtual_resource("jar:file:///lib/foo.jar!/Foo.dr".to_owned()).unwrap();
    assert_eq!(id.identifier(), "jar:file:///lib/foo.jar!/Foo.dr");
    assert_eq!(id.describe(), "virtual: jar:file:///lib/foo.jar!/Foo.dr");
}

#[test]
fn source_id_in_memory_module_rejects_blank_input() {
    for name in ["", "  ", "\n"] {
        assert_eq!(SourceId::in_memory_module(name.to_owned()), Err("moduleName must not be blank"));
    }
    let id = SourceId::in_memory_module("repl::session_1".to_owned()).unwrap();
    assert_eq!(id.identifier(), "repl::session_1");
    assert_eq!(id.describe(), "in-memory module: repl::session_1");
}

#[test]
fn source_id_generated_rejects_blank_input() {
    for description in ["", "  ", "\t"] {
        assert_eq!(SourceId::generated(description.to_owned()), Err("description must not be blank"));
    }
    let id = SourceId::generated("macro expansion #42".to_owned()).unwrap();
    assert_eq!(id.identifier(), "<generated:macro expansion #42>");
    assert_eq!(id.describe(), "generated: macro expansion #42");
}

#[test]
fn source_id_accepts_non_blank_strings_without_normalizing_them() {
    let id = SourceId::virtual_resource("  urn:example:test  ".to_owned()).unwrap();
    assert_eq!(id.identifier(), "  urn:example:test  ");
}

#[test]
fn source_id_variants_are_distinct() {
    let file = SourceId::file_path(PathBuf::from("same"));
    let virtual_resource = SourceId::virtual_resource("same".to_owned()).unwrap();
    let module = SourceId::in_memory_module("same".to_owned()).unwrap();
    let generated = SourceId::generated("same".to_owned()).unwrap();
    assert_ne!(file, virtual_resource);
    assert_ne!(virtual_resource, module);
    assert_ne!(module, generated);
}

#[test]
fn source_location_create_derives_index_and_marks_optional_offsets_unknown() {
    let location = SourceLocation::create(3, 7, 42).unwrap();
    assert_eq!(location.line(), 3);
    assert_eq!(location.column(), 7);
    assert_eq!(location.offset(), 42);
    assert_eq!(location.index(), 42);
    assert_eq!(location.utf8_offset(), UNKNOWN);
    assert_eq!(location.code_point_offset(), UNKNOWN);
    assert!(!location.has_utf8_offset());
    assert!(!location.has_code_point_offset());
}

#[test]
fn source_location_accepts_minimum_and_maximum_representable_values() {
    let minimum = SourceLocation::create(1, 1, 0).unwrap();
    assert_eq!(minimum.index(), 0);
    let maximum_index = SourceLocation::create(i32::MAX, i32::MAX, i32::MAX as i64).unwrap();
    assert_eq!(maximum_index.index(), i32::MAX);
    let full = SourceLocation::create_full(1, 1, 0, 0, 0, i64::MAX).unwrap();
    assert_eq!(full.utf8_offset(), 0);
    assert_eq!(full.code_point_offset(), i64::MAX);
}

#[test]
fn source_location_rejects_invalid_required_fields() {
    assert_eq!(SourceLocation::create(0, 1, 0), Err(SourceLocationError::InvalidLine(0)));
    assert_eq!(SourceLocation::create(1, 0, 0), Err(SourceLocationError::InvalidColumn(0)));
    assert_eq!(SourceLocation::create(1, 1, -1), Err(SourceLocationError::InvalidOffset(-1)));
    assert_eq!(SourceLocation::create_full(1, 1, 0, -1, UNKNOWN, UNKNOWN), Err(SourceLocationError::InvalidIndex(-1)));
}

#[test]
fn source_location_rejects_invalid_optional_offsets_but_accepts_unknown() {
    assert_eq!(SourceLocation::create_full(1, 1, 0, 0, -2, UNKNOWN), Err(SourceLocationError::InvalidUtf8Offset(-2)));
    assert_eq!(SourceLocation::create_full(1, 1, 0, 0, UNKNOWN, -2), Err(SourceLocationError::InvalidCodePointOffset(-2)));
    let unknown = SourceLocation::create_full(1, 1, 0, 0, UNKNOWN, UNKNOWN).unwrap();
    assert!(!unknown.has_utf8_offset());
    assert!(!unknown.has_code_point_offset());
}

#[test]
fn source_location_validation_reports_the_first_invalid_field() {
    assert_eq!(SourceLocation::create(0, 0, -1), Err(SourceLocationError::InvalidLine(0)));
}

#[test]
fn source_location_create_rejects_offsets_that_do_not_fit_in_index() {
    assert_eq!(SourceLocation::create(1, 1, i32::MAX as i64 + 1), Err(SourceLocationError::OffsetTooLarge(i32::MAX as i64 + 1)));
    assert_eq!(SourceLocation::create(1, 1, i64::MAX), Err(SourceLocationError::OffsetTooLarge(i64::MAX)));
}

#[test]
fn source_location_ordering_depends_only_on_offset() {
    let low = location(99, 99, 10);
    let same_offset = location(1, 1, 10);
    let high = location(1, 1, 11);
    assert_eq!(low.cmp(&same_offset), std::cmp::Ordering::Equal);
    assert_eq!(low, same_offset);
    assert!(low < high);
    assert!(high > low);
}

#[test]
fn source_location_with_offsets_preserves_all_other_fields() {
    let original = SourceLocation::create_full(2, 5, 10, 10, UNKNOWN, UNKNOWN).unwrap();
    let with_utf8 = original.with_utf8_offset(12);
    let with_code_point = with_utf8.with_code_point_offset(8);
    assert_eq!(with_utf8.line(), original.line());
    assert_eq!(with_utf8.column(), original.column());
    assert_eq!(with_utf8.offset(), original.offset());
    assert_eq!(with_utf8.index(), original.index());
    assert_eq!(with_utf8.utf8_offset(), 12);
    assert_eq!(with_code_point.code_point_offset(), 8);
    assert!(with_code_point.has_utf8_offset());
    assert!(with_code_point.has_code_point_offset());
}

#[test]
fn span_creation_rejects_end_before_start() {
    let start = location(1, 5, 10);
    let end = location(1, 4, 9);
    assert_eq!(Span::create(start, end), Err(SpanError::EndBeforeStart { start_offset: 10, end_offset: 9 }));
}

#[test]
fn span_allows_equal_offsets_and_point_spans() {
    let loc = location(2, 3, 7);
    let created = Span::create(loc, loc).unwrap();
    let point = Span::point(loc);
    assert_eq!(created, point);
    assert_eq!(created.length(), 0);
    assert!(created.is_empty());
    assert!(!created.is_multiline());
}

#[test]
fn span_length_and_multiline_use_location_offsets_and_lines() {
    let start = location(2, 8, 4);
    let end = location(4, 2, 13);
    let span = Span::create(start, end).unwrap();
    assert_eq!(span.start(), start);
    assert_eq!(span.end(), end);
    assert_eq!(span.length(), 9);
    assert!(span.is_multiline());
}

#[test]
fn span_contains_uses_half_open_interval() {
    let span = Span::create(location(1, 1, 10), location(1, 6, 15)).unwrap();
    assert!(!span.contains(location(1, 1, 9)));
    assert!(span.contains(location(1, 2, 10)));
    assert!(span.contains(location(1, 5, 14)));
    assert!(!span.contains(location(1, 6, 15)));
}

#[test]
fn span_overlap_handles_partial_containment_and_touching_boundaries() {
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
fn span_merge_returns_minimum_covering_span() {
    let first = Span::create(location(2, 1, 10), location(2, 5, 20)).unwrap();
    let second = Span::create(location(1, 8, 5), location(3, 2, 25)).unwrap();
    assert_eq!(first.merge(&second).start(), second.start());
    assert_eq!(first.merge(&second).end(), second.end());
    let nested = Span::create(location(2, 2, 12), location(2, 3, 18)).unwrap();
    assert_eq!(first.merge(&nested), first);
}

#[test]
fn span_extracts_ascii_and_empty_ranges() {
    let source = "hello world";
    let span = Span::create(location(1, 1, 0), location(1, 1, 5)).unwrap();
    assert_eq!(span.extract_from(source), Ok("hello"));
    let empty = Span::point(location(1, 1, 5));
    assert_eq!(empty.extract_from(source), Ok(""));
}

#[test]
fn span_extracts_utf8_only_at_valid_byte_boundaries() {
    let source = "a€b";
    let euro = Span::create(location(1, 1, 1), location(1, 1, 4)).unwrap();
    assert_eq!(euro.extract_from(source), Ok("€"));
    let invalid_boundary = Span::create(location(1, 1, 2), location(1, 1, 4)).unwrap();
    assert_eq!(invalid_boundary.extract_from(source), Err(SpanError::OffsetOutOfRange(4)));
}

#[test]
fn span_extract_rejects_end_beyond_source() {
    let span = Span::create(location(1, 1, 0), location(1, 1, 100)).unwrap();
    assert_eq!(span.extract_from("short"), Err(SpanError::OffsetOutOfRange(100)));
}
