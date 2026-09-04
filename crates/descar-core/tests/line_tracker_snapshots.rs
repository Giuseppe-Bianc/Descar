use descar_core::location::line_tracker::LineTracker;
use descar_core::location::source_location::SourceLocation;
use insta::assert_snapshot;

fn render_location(label: &str, location: SourceLocation) -> String {
    format!(
        "{label}={{ line: {}, column: {}, offset: {}, index: {}, utf8_offset: {}, code_point_offset: {} }}",
        location.line(),
        location.column(),
        location.offset(),
        location.index(),
        location.utf8_offset(),
        location.code_point_offset()
    )
}

#[test]
fn snapshots_empty_source() {
    let tracker = LineTracker::new("test.lang", String::new());
    let location = tracker.location_for(0);

    assert_snapshot!("empty_source", render_location("location", location));
}

#[test]
fn snapshots_ascii_boundaries() {
    let source = "abc";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let rendered = (0..=source.len())
        .map(|offset| render_location(&format!("offset_{offset}"), tracker.location_for(offset)))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("ascii_boundaries", rendered);
}

#[test]
fn snapshots_unicode_coordinate_spaces() {
    let source = "Aé€😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let boundary_offsets = [0, 1, 3, 6, 10, 11];

    let rendered = boundary_offsets
        .into_iter()
        .map(|offset| render_location(&format!("offset_{offset}"), tracker.location_for(offset)))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("unicode_coordinate_spaces", rendered);
}

#[test]
fn snapshots_unicode_column_semantics() {
    let source = "A😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let before_emoji = tracker.location_for(1);
    let before_b = tracker.location_for(5);

    let rendered = [render_location("before_emoji", before_emoji), render_location("before_b", before_b)].join("\n");

    assert_snapshot!("unicode_column_semantics", rendered);
}

#[test]
fn snapshots_line_endings() {
    let cases = [
        ("lf", "a\nb"),
        ("cr", "a\rb"),
        ("crlf", "a\r\nb"),
        ("line_separator", "a\u{2028}b"),
        ("paragraph_separator", "a\u{2029}b"),
    ];

    let mut rendered = Vec::new();

    for (name, source) in cases {
        let tracker = LineTracker::new("test.lang", source.to_string());
        let offset = source.find('b').expect("test source must contain b");
        let location = tracker.location_for(offset);

        rendered.push(render_location(name, location));
    }

    assert_snapshot!("line_endings", rendered.join("\n"));
}

#[test]
fn snapshots_mixed_line_endings() {
    let source = "one\ntwo\rthree\r\nfour\u{2028}five\u{2029}six";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let line_starts = [
        0,
        "one\n".len(),
        "one\ntwo\r".len(),
        "one\ntwo\rthree\r\n".len(),
        "one\ntwo\rthree\r\nfour\u{2028}".len(),
        "one\ntwo\rthree\r\nfour\u{2028}five\u{2029}".len(),
    ];

    let rendered = line_starts
        .into_iter()
        .map(|offset| render_location(&format!("offset_{offset}"), tracker.location_for(offset)))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("mixed_line_endings", rendered);
}

#[test]
fn snapshots_empty_lines() {
    let source = "\n\n\n";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let offsets = [0, 1, 2, 3];

    let rendered = offsets
        .into_iter()
        .map(|offset| render_location(&format!("offset_{offset}"), tracker.location_for(offset)))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("empty_lines", rendered);
}

#[test]
fn snapshots_unicode_after_newline() {
    let source = "abc\nAé😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let offsets = ["abc\n".len(), "abc\nA".len(), "abc\nAé".len(), "abc\nAé😀".len(), "abc\nAé😀B".len()];

    let rendered = offsets
        .into_iter()
        .map(|offset| render_location(&format!("offset_{offset}"), tracker.location_for(offset)))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("unicode_after_newline", rendered);
}

#[test]
fn snapshots_eof_positions() {
    let cases = [
        ("ascii", "abc"),
        ("unicode_bmp", "Aé€"),
        ("unicode_supplementary", "A😀B"),
        ("lf", "abc\n"),
        ("cr", "abc\r"),
        ("crlf", "abc\r\n"),
        ("line_separator", "abc\u{2028}"),
        ("paragraph_separator", "abc\u{2029}"),
    ];

    let rendered = cases
        .into_iter()
        .map(|(name, source)| {
            let tracker = LineTracker::new("test.lang", source.to_string());
            render_location(name, tracker.location_for(source.len()))
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("eof_positions", rendered);
}

#[test]
fn snapshots_span_with_unicode() {
    let source = "A😀B";
    let tracker = LineTracker::new("test.lang", source.to_string());

    let span = tracker.span_for(1..5);

    let rendered = format!(
        "file_path={}\n\
         length={}\n\
         is_empty={}\n\
         is_multiline={}\n\
         start={:?}\n\
         end={:?}",
        span.file_path(),
        span.length(),
        span.is_empty(),
        span.is_multiline(),
        span.start(),
        span.end(),
    );

    assert_snapshot!("span_with_unicode", rendered);
}
