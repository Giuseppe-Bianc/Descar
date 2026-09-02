use std::path::PathBuf;

use descar_core::location::source_id::SourceId;

#[test]
fn file_path_preserves_path() {
    let id = SourceId::file_path(PathBuf::from("src/main.dr"));
    assert_eq!(id.identifier(), "src/main.dr");
    assert_eq!(id.describe(), "file: src/main.dr");
    assert_eq!(id.to_string(), "file: src/main.dr");
}

#[test]
fn virtual_resource_rejects_blank_input() {
    for uri in ["", "   ", "\t\n"] {
        assert_eq!(SourceId::virtual_resource(uri.to_owned()), Err("uri must not be blank"));
    }
    let id = SourceId::virtual_resource("jar:file:///lib/foo.jar!/Foo.dr".to_owned()).unwrap();
    assert_eq!(id.identifier(), "jar:file:///lib/foo.jar!/Foo.dr");
    assert_eq!(id.describe(), "virtual: jar:file:///lib/foo.jar!/Foo.dr");
}

#[test]
fn in_memory_module_rejects_blank_input() {
    for name in ["", "  ", "\n"] {
        assert_eq!(SourceId::in_memory_module(name.to_owned()), Err("moduleName must not be blank"));
    }
    let id = SourceId::in_memory_module("repl::session_1".to_owned()).unwrap();
    assert_eq!(id.identifier(), "repl::session_1");
    assert_eq!(id.describe(), "in-memory module: repl::session_1");
}

#[test]
fn generated_rejects_blank_input() {
    for description in ["", "  ", "\t"] {
        assert_eq!(SourceId::generated(description.to_owned()), Err("description must not be blank"));
    }
    let id = SourceId::generated("macro expansion #42".to_owned()).unwrap();
    assert_eq!(id.identifier(), "<generated:macro expansion #42>");
    assert_eq!(id.describe(), "generated: macro expansion #42");
}

#[test]
fn accepts_non_blank_strings_without_normalizing_them() {
    let id = SourceId::virtual_resource("  urn:example:test  ".to_owned()).unwrap();
    assert_eq!(id.identifier(), "  urn:example:test  ");
}

#[test]
fn variants_are_distinct() {
    let file = SourceId::file_path(PathBuf::from("same"));
    let virtual_resource = SourceId::virtual_resource("same".to_owned()).unwrap();
    let module = SourceId::in_memory_module("same".to_owned()).unwrap();
    let generated = SourceId::generated("same".to_owned()).unwrap();
    assert_ne!(file, virtual_resource);
    assert_ne!(virtual_resource, module);
    assert_ne!(module, generated);
}
