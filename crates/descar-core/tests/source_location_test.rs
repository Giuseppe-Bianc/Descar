use descar_core::location::source_location::{SourceLocation, SourceLocationError, UNKNOWN};

fn location(line: i32, column: i32, offset: i64) -> SourceLocation {
    SourceLocation::create(line, column, offset).expect("test location should be valid")
}

#[test]
fn create_derives_index_and_marks_optional_offsets_unknown() {
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
fn accepts_minimum_and_maximum_representable_values() {
    let minimum = SourceLocation::create(1, 1, 0).unwrap();
    assert_eq!(minimum.index(), 0);
    let maximum_index = SourceLocation::create(i32::MAX, i32::MAX, i64::from(i32::MAX)).unwrap();
    assert_eq!(maximum_index.index(), i32::MAX);
    let full = SourceLocation::create_full(1, 1, 0, 0, 0, i64::MAX).unwrap();
    assert_eq!(full.utf8_offset(), 0);
    assert_eq!(full.code_point_offset(), i64::MAX);
}

#[test]
fn rejects_invalid_required_fields() {
    assert_eq!(SourceLocation::create(0, 1, 0), Err(SourceLocationError::InvalidLine(0)));
    assert_eq!(SourceLocation::create(1, 0, 0), Err(SourceLocationError::InvalidColumn(0)));
    assert_eq!(SourceLocation::create(1, 1, -1), Err(SourceLocationError::InvalidOffset(-1)));
    assert_eq!(SourceLocation::create_full(1, 1, 0, -1, UNKNOWN, UNKNOWN), Err(SourceLocationError::InvalidIndex(-1)));
}

#[test]
fn rejects_invalid_optional_offsets_but_accepts_unknown() {
    assert_eq!(SourceLocation::create_full(1, 1, 0, 0, -2, UNKNOWN), Err(SourceLocationError::InvalidUtf8Offset(-2)));
    assert_eq!(
        SourceLocation::create_full(1, 1, 0, 0, UNKNOWN, -2),
        Err(SourceLocationError::InvalidCodePointOffset(-2))
    );
    let unknown = SourceLocation::create_full(1, 1, 0, 0, UNKNOWN, UNKNOWN).unwrap();
    assert!(!unknown.has_utf8_offset());
    assert!(!unknown.has_code_point_offset());
}

#[test]
fn validation_reports_the_first_invalid_field() {
    assert_eq!(SourceLocation::create(0, 0, -1), Err(SourceLocationError::InvalidLine(0)));
}

#[test]
fn create_rejects_offsets_that_do_not_fit_in_index() {
    assert_eq!(
        SourceLocation::create(1, 1, i64::from(i32::MAX) + 1),
        Err(SourceLocationError::OffsetTooLarge(i64::from(i32::MAX) + 1))
    );
    assert_eq!(SourceLocation::create(1, 1, i64::MAX), Err(SourceLocationError::OffsetTooLarge(i64::MAX)));
}

#[test]
fn ordering_depends_only_on_offset() {
    let low = location(99, 99, 10);
    let same_offset = location(1, 1, 10);
    let high = location(1, 1, 11);
    assert_eq!(low.cmp(&same_offset), std::cmp::Ordering::Equal);
    assert!(low < high);
    assert!(high > low);
}

#[test]
fn with_offsets_preserves_all_other_fields() {
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
