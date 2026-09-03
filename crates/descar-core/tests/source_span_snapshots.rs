use std::sync::Arc;

use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::SourceSpan as Span;
use insta::assert_snapshot;

const fn location(line: usize, column: usize, offset: usize) -> SourceLocation {
    SourceLocation::new(line, column, offset, 0, usize::MAX, usize::MAX)
}
#[test]
fn snapshots_representations() {
    let point = Span::point(Arc::from("asd/dd.dr"), location(2, 3, 7));
    let regular = Span::new(Arc::from("asd/dd.dr"), location(1, 1, 0), location(2, 4, 5));
    let nested = Span::new(Arc::from("asd/dd.dr"), location(1, 2, 1), location(1, 3, 4));
    let merged = regular.merge(&nested);
    let rendered = format!(
        "point={point:?}\npoint_display={point}\npoint_length={}\nregular={regular:?}\nregular_display={regular}\nregular_length={}\nregular_multiline={}\nmerged={merged:?}",
        point.length(),
        regular.length(),
        regular.is_multiline()
    );
    assert_snapshot!("representations", rendered);
}
