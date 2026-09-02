use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::Span;
use insta::assert_snapshot;

fn location(line: i32, column: i32, offset: i64) -> SourceLocation {
    SourceLocation::create(line, column, offset).expect("test location should be valid")
}

#[test]
fn representations_are_stable() {
    let point = Span::point(location(2, 3, 7));
    let regular = Span::create(location(1, 1, 0), location(2, 4, 5)).unwrap();
    let nested = Span::create(location(1, 2, 1), location(1, 3, 4)).unwrap();
    let merged = regular.merge(&nested);
    let rendered = format!(
        "point={point:?}\npoint_display={point}\npoint_length={}\nregular={regular:?}\nregular_display={regular}\nregular_length={}\nregular_multiline={}\nmerged={merged:?}",
        point.length(),
        regular.length(),
        regular.is_multiline()
    );
    assert_snapshot!(rendered, @r###"point=Span { start: SourceLocation { line: 2, column: 3, offset: 7, index: 7, utf8_offset: -1, code_point_offset: -1 }, end: SourceLocation { line: 2, column: 3, offset: 7, index: 7, utf8_offset: -1, code_point_offset: -1 } }
point_display=line 2:column 3
point_length=0
regular=Span { start: SourceLocation { line: 1, column: 1, offset: 0, index: 0, utf8_offset: -1, code_point_offset: -1 }, end: SourceLocation { line: 2, column: 4, offset: 5, index: 5, utf8_offset: -1, code_point_offset: -1 } }
regular_display=line 1:column 1-line 2:column 4
regular_length=5
regular_multiline=true
merged=Span { start: SourceLocation { line: 1, column: 1, offset: 0, index: 0, utf8_offset: -1, code_point_offset: -1 }, end: SourceLocation { line: 2, column: 4, offset: 5, index: 5, utf8_offset: -1, code_point_offset: -1 } }"###);
}

#[test]
fn extraction_and_errors_are_stable() {
    let reversed = Span::create(location(1, 1, 8), location(1, 1, 3)).unwrap_err();
    let source = "a€b";
    let valid = Span::create(location(1, 1, 1), location(1, 1, 4)).unwrap();
    let invalid_boundary = Span::create(location(1, 1, 2), location(1, 1, 4)).unwrap();
    let out_of_range = Span::create(location(1, 1, 0), location(1, 1, 99)).unwrap();
    let rendered = format!(
        "reversed={reversed:?}\nreversed_display={reversed}\nvalid_extract={:?}\ninvalid_boundary={:?}\nout_of_range={:?}",
        valid.extract_from(source),
        invalid_boundary.extract_from(source),
        out_of_range.extract_from(source)
    );
    assert_snapshot!(rendered, @r###"reversed=EndBeforeStart { start_offset: 8, end_offset: 3 }
reversed_display=end offset (3) must not precede start offset (8)
valid_extract=Ok("€")
invalid_boundary=Err(OffsetOutOfRange(4))
out_of_range=Err(OffsetOutOfRange(99))"###);
}
