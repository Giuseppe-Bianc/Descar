use super::source_location::{SourceLocation, UNKNOWN};
use insta::assert_snapshot;

#[test]
fn representations_and_errors_are_stable() {
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
    let rendered = format!("complete={complete:?}\ndisplay={complete}\nunknown={unknown:?}\nflags=utf8:{} code_point:{}\nerrors=\n{}", complete.has_utf8_offset(), complete.has_code_point_offset(), errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"));
    assert_snapshot!(rendered, @r###"complete=SourceLocation { line: 12, column: 34, offset: 56, index: 56, utf8_offset: 78, code_point_offset: 90 }
display=line 12:column 34
unknown=SourceLocation { line: 1, column: 1, offset: 0, index: 0, utf8_offset: -1, code_point_offset: -1 }
flags=utf8:true code_point:true
errors=
line must be >= 1 (1-based), got: 0
column must be >= 1 (1-based), got: 0
offset must be >= 0, got: -1
index must be >= 0, got: -1
utf8Offset must be >= 0 or UNKNOWN, got: -1
codePointOffset must be >= 0 or UNKNOWN, got: -1
offset 2147483648 does not fit into an i32 index"###);
}
