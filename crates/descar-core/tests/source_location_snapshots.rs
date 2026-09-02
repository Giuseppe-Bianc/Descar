use descar_core::location::source_location::{SourceLocation, UNKNOWN};
use insta::assert_snapshot;

#[test]
fn snapshots_representations_and_errors() {
    let complete = SourceLocation::create_full(12, 34, 56, 56, 78, 90).unwrap();
    let unknown = SourceLocation::create(1, 1, 0).unwrap();
    let errors = [
        SourceLocation::create(0, 1, 0).unwrap_err(),
        SourceLocation::create(1, 0, 0).unwrap_err(),
        SourceLocation::create(1, 1, -1).unwrap_err(),
        SourceLocation::create_full(1, 1, 0, -1, UNKNOWN, UNKNOWN).unwrap_err(),
        SourceLocation::create_full(1, 1, 0, 0, -1, UNKNOWN).unwrap_err(),
        SourceLocation::create_full(1, 1, 0, 0, UNKNOWN, -1).unwrap_err(),
        SourceLocation::create(1, 1, i32::MAX as i64 + 1).unwrap_err(),
    ];
    let rendered = format!(
        "complete={complete:?}\ndisplay={complete}\nunknown={unknown:?}\nflags=utf8:{} code_point:{}\nerrors=\n{}",
        complete.has_utf8_offset(),
        complete.has_code_point_offset(),
        errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
    );
    assert_snapshot!("representations_and_errors", rendered);
}
