use descar_core::location::line_tracker::LineTracker;

#[test]
fn location_for_zero_offset() {
    let tracker = LineTracker::new("test.lang", "abc".to_string());

    let location = tracker.location_for(0);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 1);
    assert_eq!(location.offset(), 0);
    assert_eq!(location.index(), 0);
    assert_eq!(location.utf8_offset(), 0);
    assert_eq!(location.code_point_offset(), 0);
}

#[test]
fn location_for_eof_after_single_character() {
    let tracker = LineTracker::new("test.lang", "a".to_string());

    let location = tracker.location_for(1);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 2);
    assert_eq!(location.offset(), 1);
    assert_eq!(location.index(), 1);
    assert_eq!(location.utf8_offset(), 1);
    assert_eq!(location.code_point_offset(), 1);
}

#[test]
fn location_for_eof_after_trailing_newline() {
    let tracker = LineTracker::new("test.lang", "abc\n".to_string());

    let location = tracker.location_for(4);

    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 1);
    assert_eq!(location.offset(), 4);
    assert_eq!(location.index(), 4);
    assert_eq!(location.utf8_offset(), 4);
    assert_eq!(location.code_point_offset(), 4);
}

#[test]
fn location_for_eof_after_trailing_cr() {
    let tracker = LineTracker::new("test.lang", "abc\r".to_string());

    let location = tracker.location_for(4);

    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 1);
}

#[test]
fn location_for_eof_after_trailing_crlf() {
    let tracker = LineTracker::new("test.lang", "abc\r\n".to_string());

    let location = tracker.location_for(5);

    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 1);
}

#[test]
fn location_for_empty_lines() {
    let tracker = LineTracker::new("test.lang", "\n\n\n".to_string());

    assert_eq!(tracker.location_for(0).line(), 1);
    assert_eq!(tracker.location_for(1).line(), 2);
    assert_eq!(tracker.location_for(2).line(), 3);
    assert_eq!(tracker.location_for(3).line(), 4);
}

#[test]
fn location_for_empty_source_only_has_eof_position() {
    let tracker = LineTracker::new("test.lang", String::new());

    let location = tracker.location_for(0);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 1);
    assert_eq!(location.offset(), 0);
    assert_eq!(location.index(), 0);
    assert_eq!(location.utf8_offset(), 0);
    assert_eq!(location.code_point_offset(), 0);
}

#[test]
fn location_for_only_crlf_sequences() {
    let tracker = LineTracker::new("test.lang", "\r\n\r\n\r\n".to_string());

    let location = tracker.location_for(6);

    assert_eq!(location.line(), 4);
    assert_eq!(location.column(), 1);
}

#[test]
fn location_for_mixed_line_endings() {
    let source = "a\nb\rc\r\nd\u{2028}e\u{2029}f";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let second_line_start = tracker.location_for("a\n".len());
    let third_line_start = tracker.location_for("a\nb\r".len());
    let fourth_line_start = tracker.location_for("a\nb\rc\r\n".len());
    let fifth_line_start = tracker.location_for("a\nb\rc\r\nd\u{2028}".len());
    let sixth_line_start = tracker.location_for("a\nb\rc\r\nd\u{2028}e\u{2029}".len());

    assert_eq!((second_line_start.line(), second_line_start.column()), (2, 1));
    assert_eq!((third_line_start.line(), third_line_start.column()), (3, 1));
    assert_eq!((fourth_line_start.line(), fourth_line_start.column()), (4, 1));
    assert_eq!((fifth_line_start.line(), fifth_line_start.column()), (5, 1));
    assert_eq!((sixth_line_start.line(), sixth_line_start.column()), (6, 1));
}

#[test]
fn location_for_multiple_consecutive_unicode_line_separators() {
    let source = "\u{2028}\u{2028}\u{2029}\u{2029}";
    let tracker = LineTracker::new("test.lang", source.to_string());

    assert_eq!(tracker.location_for(0).line(), 1);
    assert_eq!(tracker.location_for(3).line(), 2);
    assert_eq!(tracker.location_for(6).line(), 3);
    assert_eq!(tracker.location_for(9).line(), 4);
    assert_eq!(tracker.location_for(12).line(), 5);
}

#[test]
fn location_for_bmp_multibyte_character() {
    let source = "éX";
    let tracker = LineTracker::new("test.lang", source.to_string());

    // `é` occupies 2 UTF-8 bytes but one UTF-16 code unit and one code point.
    let location = tracker.location_for(2);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 2);
    assert_eq!(location.offset(), 2);
    assert_eq!(location.index(), 1);
    assert_eq!(location.utf8_offset(), 2);
    assert_eq!(location.code_point_offset(), 1);
}

#[test]
fn location_for_three_byte_bmp_character() {
    let source = "€X";
    let tracker = LineTracker::new("test.lang", source.to_string());

    // `€` occupies 3 UTF-8 bytes.
    let location = tracker.location_for(3);

    assert_eq!(location.column(), 2);
    assert_eq!(location.offset(), 3);
    assert_eq!(location.index(), 1);
    assert_eq!(location.utf8_offset(), 3);
    assert_eq!(location.code_point_offset(), 1);
}

#[test]
fn location_for_supplementary_character() {
    let source = "😀X";
    let tracker = LineTracker::new("test.lang", source.to_string());

    // 😀 occupies 4 UTF-8 bytes and 2 UTF-16 code units.
    let location = tracker.location_for(4);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 2);
    assert_eq!(location.offset(), 4);
    assert_eq!(location.index(), 2);
    assert_eq!(location.utf8_offset(), 4);
    assert_eq!(location.code_point_offset(), 1);
}

#[test]
fn location_for_mixed_bmp_and_supplementary_characters() {
    let source = "Aé😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    // UTF-8 byte offsets:
    // A   = 0
    // é   = 1..2
    // 😀  = 3..6
    // B   = 7
    let location = tracker.location_for(7);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 4);

    // UTF-16:
    // A = 1
    // é = 1
    // 😀 = 2
    // => B starts at index 4
    assert_eq!(location.index(), 4);

    assert_eq!(location.offset(), 7);
    assert_eq!(location.utf8_offset(), 7);
    assert_eq!(location.code_point_offset(), 3);
}

#[test]
fn location_for_unicode_character_at_beginning_of_line() {
    let source = "\n😀X";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let location = tracker.location_for(1);

    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 1);
    assert_eq!(location.code_point_offset(), 1);
    assert_eq!(location.index(), 1);
}

#[test]
fn location_for_unicode_character_after_newline() {
    let source = "abc\n😀X";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let location = tracker.location_for(8);

    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 2);
    assert_eq!(location.offset(), 8);
    assert_eq!(location.code_point_offset(), 5);
    assert_eq!(location.index(), 6);
}

#[test]
fn location_for_many_supplementary_characters() {
    let source = "😀😀😀X";
    let tracker = LineTracker::new("test.lang", source.to_string());

    // Three emojis = 12 UTF-8 bytes and 6 UTF-16 units.
    let location = tracker.location_for(12);

    assert_eq!(location.line(), 1);
    assert_eq!(location.column(), 4);
    assert_eq!(location.offset(), 12);
    assert_eq!(location.index(), 6);
    assert_eq!(location.utf8_offset(), 12);
    assert_eq!(location.code_point_offset(), 3);
}

#[test]
fn location_for_unicode_line_separator_with_unicode_content() {
    let source = "é\u{2028}😀X";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let offset = "é\u{2028}".len();
    let location = tracker.location_for(offset);

    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 1);
    assert_eq!(location.offset(), offset);
    assert_eq!(location.index(), 2);
    assert_eq!(location.utf8_offset(), offset);
    assert_eq!(location.code_point_offset(), 2);
}

#[test]
#[should_panic(expected = "out of bounds")]
fn location_for_offset_greater_than_source_length() {
    let tracker = LineTracker::new("test.lang", "abc".to_string());

    let _ = tracker.location_for(usize::MAX);
}

#[test]
#[should_panic(expected = "not a UTF-8 character boundary")]
fn location_for_offset_inside_two_byte_utf8_character() {
    let tracker = LineTracker::new("test.lang", "é".to_string());

    // `é` occupies bytes 0 and 1. Offset 1 is not a character boundary.
    let _ = tracker.location_for(1);
}

#[test]
#[should_panic(expected = "not a UTF-8 character boundary")]
fn location_for_offset_inside_three_byte_utf8_character() {
    let tracker = LineTracker::new("test.lang", "€".to_string());

    let _ = tracker.location_for(1);
}

#[test]
#[should_panic(expected = "not a UTF-8 character boundary")]
fn location_for_offset_inside_four_byte_utf8_character() {
    let tracker = LineTracker::new("test.lang", "😀".to_string());

    let _ = tracker.location_for(1);
}

#[test]
fn location_for_all_valid_boundaries_of_unicode_source() {
    let source = "Aé€😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let expected = [
        // offset, line, column, UTF-16 index, code-point offset
        (0, 1, 1, 0, 0),
        (1, 1, 2, 1, 1),
        (3, 1, 3, 2, 2),
        (6, 1, 4, 3, 3),
        (10, 1, 5, 5, 4),
    ];

    for (offset, line, column, index, code_point_offset) in expected {
        let location = tracker.location_for(offset);

        assert_eq!(location.offset(), offset);
        assert_eq!(location.line(), line);
        assert_eq!(location.column(), column);
        assert_eq!(location.index(), index);
        assert_eq!(location.utf8_offset(), offset);
        assert_eq!(location.code_point_offset(), code_point_offset);
    }
}

#[test]
fn location_for_preserves_absolute_code_point_offset_across_lines() {
    let source = "A😀\nBé\n€C";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let c_offset = source.find('C').expect("C must exist");

    let location = tracker.location_for(c_offset);

    assert_eq!(location.line(), 3);
    assert_eq!(location.column(), 2);

    assert_eq!(location.code_point_offset(), source[..c_offset].chars().count());

    assert_eq!(location.index(), source[..c_offset].encode_utf16().count());
}

#[test]
fn span_for_with_unicode_source_returns_complete_locations() {
    let source = "A😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let span = tracker.span_for(1..5);

    assert_eq!(span.file_path(), "test.lang");
    assert_eq!(span.length(), 4);
    assert!(!span.is_empty());
    assert!(!span.is_multiline());

    let start = span.start();
    let end = span.end();

    assert_eq!(start.column(), 2);
    assert_eq!(start.offset(), 1);
    assert_eq!(start.index(), 1);
    assert_eq!(start.utf8_offset(), 1);
    assert_eq!(start.code_point_offset(), 1);

    assert_eq!(end.column(), 3);
    assert_eq!(end.offset(), 5);
    assert_eq!(end.index(), 3);
    assert_eq!(end.utf8_offset(), 5);
    assert_eq!(end.code_point_offset(), 2);
}

#[test]
fn span_for_empty_range_creates_point_span() {
    let tracker = LineTracker::new("test.lang", "abc".to_string());

    let span = tracker.span_for(2..2);

    assert!(span.is_empty());
    assert_eq!(span.length(), 0);
    assert_eq!(span.start(), span.end());
    assert_eq!(span.start().offset(), 2);
}

#[test]
fn get_line_handles_empty_first_line() {
    let tracker = LineTracker::new("test.lang", "\nsecond".to_string());

    assert_eq!(tracker.get_line(1), Some(""));
    assert_eq!(tracker.get_line(2), Some("second"));
}

#[test]
fn get_line_handles_empty_lines_between_content() {
    let tracker = LineTracker::new("test.lang", "first\n\nthird".to_string());

    assert_eq!(tracker.get_line(1), Some("first"));
    assert_eq!(tracker.get_line(2), Some(""));
    assert_eq!(tracker.get_line(3), Some("third"));
}

#[test]
fn get_line_handles_trailing_empty_line() {
    let tracker = LineTracker::new("test.lang", "first\n".to_string());

    assert_eq!(tracker.get_line(1), Some("first"));
    assert_eq!(tracker.get_line(2), Some(""));
}

#[test]
fn get_line_handles_crlf_without_returning_terminator() {
    let tracker = LineTracker::new("test.lang", "first\r\nsecond".to_string());

    assert_eq!(tracker.get_line(1), Some("first"));
    assert_eq!(tracker.get_line(2), Some("second"));
}

#[test]
fn get_line_handles_unicode_line_terminators() {
    let tracker = LineTracker::new("test.lang", "first\u{2028}second\u{2029}third".to_string());

    assert_eq!(tracker.get_line(1), Some("first"));
    assert_eq!(tracker.get_line(2), Some("second"));
    assert_eq!(tracker.get_line(3), Some("third"));
}
