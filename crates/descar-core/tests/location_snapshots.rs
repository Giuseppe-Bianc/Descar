use std::path::PathBuf;

use descar_core::location::source_id::SourceId;
use descar_core::location::source_location::{SourceLocation, UNKNOWN};
use descar_core::location::source_span::Span;
use insta::assert_snapshot;

fn location(line: i32, column: i32, offset: i64) -> SourceLocation {
    SourceLocation::create(line, column, offset).expect("test location should be valid")
}

#[test]
fn snapshots_source_id_representations() {
    let values = [
        SourceId::file_path(PathBuf::from("src/main.dr")),
        SourceId::virtual_resource("jar:file:///lib/foo.jar!/Foo.dr".to_owned()).unwrap(),
        SourceId::in_memory_module("repl::session_1".to_owned()).unwrap(),
        SourceId::generated("macro expansion #42".to_owned()).unwrap(),
    ];
    let rendered = values.iter().map(|id| format!("identifier={}\ndescribe={}\ndisplay={}", id.identifier(), id.describe(), id)).collect::<Vec<_>>().join("\n---\n");
    assert_snapshot!(rendered, @r###"identifier=src/main.dr
describe=file: src/main.dr
display=file: src/main.dr
---
identifier=jar:file:///lib/foo.jar!/Foo.dr
describe=virtual: jar:file:///lib/foo.jar!/Foo.dr
display=virtual: jar:file:///lib/foo.jar!/Foo.dr
---
identifier=repl::session_1
describe=in-memory module: repl::session_1
display=in-memory module: repl::session_1
---
identifier=<generated:macro expansion #42>
describe=generated: macro expansion #42
display=generated: macro expansion #42"###);
}

#[test]
fn snapshots_source_location_representations_and_errors() {
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

#[test]
fn snapshots_span_representations() {
    let point = Span::point(location(2, 3, 7));
    let regular = Span::create(location(1, 1, 0), location(2, 4, 5)).unwrap();
    let nested = Span::create(location(1, 2, 1), location(1, 3, 4)).unwrap();
    let merged = regular.merge(&nested);
    let rendered = format!("point={point:?}\npoint_display={point}\npoint_length={}\nregular={regular:?}\nregular_display={regular}\nregular_length={}\nregular_multiline={}\nmerged={merged:?}", point.length(), regular.length(), regular.is_multiline());
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
fn snapshots_span_extraction_and_errors() {
    let reversed = descar_core::location::source_span::Span::create(location(1, 1, 8), location(1, 1, 3)).unwrap_err();
    let source = "a€b";
    let valid = Span::create(location(1, 1, 1), location(1, 1, 4)).unwrap();
    let invalid_boundary = Span::create(location(1, 1, 2), location(1, 1, 4)).unwrap();
    let out_of_range = Span::create(location(1, 1, 0), location(1, 1, 99)).unwrap();
    let rendered = format!("reversed={reversed:?}\nreversed_display={reversed}\nvalid_extract={:?}\ninvalid_boundary={:?}\nout_of_range={:?}", valid.extract_from(source), invalid_boundary.extract_from(source), out_of_range.extract_from(source));
    assert_snapshot!(rendered, @r###"reversed=EndBeforeStart { start_offset: 8, end_offset: 3 }
reversed_display=end offset (3) must not precede start offset (8)
valid_extract=Ok("€")
invalid_boundary=Err(OffsetOutOfRange(4))
out_of_range=Err(OffsetOutOfRange(99))"###);
}
