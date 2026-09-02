use std::path::PathBuf;

use super::source_id::SourceId;
use insta::assert_snapshot;

#[test]
fn representations_are_stable() {
    let values = [
        SourceId::file_path(PathBuf::from("src/main.dr")),
        SourceId::virtual_resource("jar:file:///lib/foo.jar!/Foo.dr".to_owned()).unwrap(),
        SourceId::in_memory_module("repl::session_1".to_owned()).unwrap(),
        SourceId::generated("macro expansion #42".to_owned()).unwrap(),
    ];
    let rendered = values
        .iter()
        .map(|id| format!("identifier={}\ndescribe={}\ndisplay={}", id.identifier(), id.describe(), id))
        .collect::<Vec<_>>()
        .join("\n---\n");
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
