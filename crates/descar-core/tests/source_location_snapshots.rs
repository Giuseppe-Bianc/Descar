use descar_core::location::source_location::{SourceLocation, UNKNOWN};
use insta::assert_snapshot;

#[test]
fn snapshots_representations_and_errors() {
    let complete = SourceLocation::new(12, 34, 56, 56, 78, 90);
    let unknown = SourceLocation::new(1, 1, 0, 0, 0, 0);
    let errors = [
        SourceLocation::new(0, 1, 0, usize::MAX, UNKNOWN, UNKNOWN),
        SourceLocation::new(1, 0, 0, usize::MAX, UNKNOWN, UNKNOWN),
        SourceLocation::new(1, 1, usize::MAX - 1, usize::MAX, UNKNOWN, UNKNOWN),
        SourceLocation::new(1, 1, 0, usize::MAX, UNKNOWN, UNKNOWN),
        SourceLocation::new(1, 1, 0, 0, usize::MAX, UNKNOWN),
        SourceLocation::new(1, 1, 0, 0, UNKNOWN, usize::MAX),
        SourceLocation::new(1, 1, usize::MAX, usize::MAX, UNKNOWN, UNKNOWN),
    ];
    let rendered = format!(
        "complete={complete:?}\ndisplay={complete}\nunknown={unknown:?}\nerrors=\n{}",
        errors.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join("\n")
    );
    assert_snapshot!("representations_and_errors", rendered);
}
