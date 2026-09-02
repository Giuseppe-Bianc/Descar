use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::Span;
use insta::assert_snapshot;

fn location(line: i32, column: i32, offset: i64) -> SourceLocation {
    SourceLocation::create(line, column, offset).expect("test location should be valid")
}

#[test]
fn snapshots_representations() {
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
    assert_snapshot!("representations", rendered);
}

#[test]
fn snapshots_extraction_and_errors() {
    let reversed = Span::create(location(1, 1, 8), location(1, 1, 3)).unwrap_err();
    let source = "a€b";
    let valid = Span::create(location(1, 1, 1), location(1, 1, 4)).unwrap();
    let invalid_boundary = Span::create(location(1, 1, 2), location(1, 1, 4)).unwrap();
    let out_of_range = Span::create(location(1, 1, 0), location(1, 1, 99)).unwrap();
    let rendered = format!(
        "reversed={reversed:?}\n\
     valid_extract={:?}\n\
     invalid_boundary={:?}\n\
     out_of_range={:?}",
        valid.extract_from(source),
        invalid_boundary.extract_from(source),
        out_of_range.extract_from(source)
    );
    assert_snapshot!("extraction_and_errors", rendered);
}
