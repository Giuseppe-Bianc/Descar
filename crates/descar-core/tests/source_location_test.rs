use descar_core::location::source_location::{SourceLocation, UNKNOWN};

const fn location(line: usize, column: usize, offset: usize) -> SourceLocation {
    SourceLocation::new(line, column, offset, 0, UNKNOWN, UNKNOWN)
}

#[test]
fn create_derives_index_and_marks_optional_offsets_unknown() {
    let location = SourceLocation::new(3, 7, 42, 0, UNKNOWN, UNKNOWN);
    assert_eq!(location.line(), 3);
    assert_eq!(location.column(), 7);
    assert_eq!(location.offset(), 42);
    assert_eq!(location.index(), 0);
    assert_eq!(location.utf8_offset(), UNKNOWN);
    assert_eq!(location.code_point_offset(), UNKNOWN);
}

#[test]
fn ordering_is_consistent_with_equality() {
    // Same offset but different line/column → not equal, not Equal under cmp.
    // line 1 < line 99, so a comes before b.
    let a = location(1, 1, 10);
    let b = location(99, 99, 10);
    assert!(a < b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);

    // Only truly identical locations compare Equal.
    let dup = location(1, 1, 10);
    assert_eq!(a.cmp(&dup), std::cmp::Ordering::Equal);

    // Higher offset on the same line/column is still greater.
    let high = location(1, 1, 11);
    assert!(a < high);
    assert!(high > a);
}
