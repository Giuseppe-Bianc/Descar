use std::path::PathBuf;

use descar_core::location::source_id::SourceId;
use insta::assert_snapshot;

#[test]
fn snapshots_representations() {
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
    assert_snapshot!("representations", rendered);
}
