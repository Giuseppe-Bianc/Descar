use descar_core::location::line_tracker::LineTracker;

#[test]
fn locations_stay_exact_past_checkpoints() {
    let source = "é€😀x\n".repeat(100);
    let tracker = LineTracker::new("test.dr", source.clone());
    let (mut line, mut column, mut utf16, mut code_points) = (1, 1, 0, 0);
    for (offset, ch) in source.char_indices() {
        let location = tracker.location_for(offset);
        assert_eq!(
            (location.line(), location.column(), location.index(), location.code_point_offset()),
            (line, column, utf16, code_points),
            "offset {offset}"
        );
        utf16 += ch.len_utf16();
        code_points += 1;
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
}
