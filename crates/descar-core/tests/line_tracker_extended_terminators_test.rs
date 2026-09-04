use descar_core::location::line_tracker::LineTracker;

#[test]
fn location_for_extended_unicode_line_terminators() {
    let source = "a\u{000B}b\u{000C}c\u{0085}d\u{2028}e\u{2029}f";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let offsets = [
        "a\u{000B}".len(),
        "a\u{000B}b\u{000C}".len(),
        "a\u{000B}b\u{000C}c\u{0085}".len(),
        "a\u{000B}b\u{000C}c\u{0085}d\u{2028}".len(),
        "a\u{000B}b\u{000C}c\u{0085}d\u{2028}e\u{2029}".len(),
    ];

    for (line, offset) in offsets.into_iter().enumerate() {
        let location = tracker.location_for(offset);
        assert_eq!(location.line(), line + 2);
        assert_eq!(location.column(), 1);
    }
}

#[test]
fn get_line_strips_extended_unicode_line_terminators() {
    let tracker = LineTracker::new(
        "test.lang",
        "one\u{000B}two\u{000C}three\u{0085}four\u{2028}five\u{2029}six".to_string(),
    );

    assert_eq!(tracker.get_line(1), Some("one"));
    assert_eq!(tracker.get_line(2), Some("two"));
    assert_eq!(tracker.get_line(3), Some("three"));
    assert_eq!(tracker.get_line(4), Some("four"));
    assert_eq!(tracker.get_line(5), Some("five"));
    assert_eq!(tracker.get_line(6), Some("six"));
}
