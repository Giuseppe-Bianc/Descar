use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::{SourceSpan, truncate_path};
use descar_core::utils::create_span;
use std::path::Path;
use std::sync::Arc;

const fn location(line: usize, column: usize, offset: usize) -> SourceLocation {
    SourceLocation::new(line, column, offset, 0, usize::MAX, usize::MAX)
}

/*#[test]
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
}*/

macro_rules! truncate_test {
    ($name:ident, $path:expr, $depth:expr) => {
        #[test]
        fn $name() {
            let path = Path::new($path);
            let truncated = truncate_path(path, $depth);
            let snapshot_name =
                if cfg!(unix) { concat!(stringify!($name), "_unix") } else { concat!(stringify!($name), "_windows") };
            insta::assert_snapshot!(snapshot_name, truncated);
        }
    };
}

macro_rules! span_str_test {
    ($name:ident, $file:expr, $sl:expr, $sc:expr, $el:expr, $ec:expr) => {
        #[test]
        fn $name() {
            let span = create_span($file, $sl, $sc, $el, $ec);

            let snapshot_name =
                if cfg!(unix) { concat!(stringify!($name), "_unix") } else { concat!(stringify!($name), "_windows") };
            insta::assert_snapshot!(snapshot_name, span.to_string());
        }
    };
}

// Test di troncamento percorso
truncate_test!(longer_than_depth, "a/b/c/d", 2);
truncate_test!(exact_depth, "a/b/c", 3);
truncate_test!(shorter_than_depth, "a", 2);
truncate_test!(depth_zero, "/usr/project/src/main.vn", 0);
truncate_test!(single_component, "file.vn", 2);
truncate_test!(absolute_path, "/usr/project/src/main.vn", 2);

// Test formattazione stringa span
span_str_test!(same_line, "project/src/main.vn", 5, 3, 5, 10);
span_str_test!(different_lines, "src/module/file.vn", 2, 1, 4, 5);
span_str_test!(single_component_path, "file.vn", 1, 1, 1, 1);
span_str_test!(same_start_end, "a/b/c/d/file.vn", 3, 2, 3, 2);
span_str_test!(minimal_coordinates, "f.vn", 0, 0, 0, 0);

#[test]
fn absolute_path_span() {
    let path = if cfg!(unix) { "/usr/project/src/main.vn" } else { "C:\\project\\src\\main.vn" };
    let span = SourceSpan::new(Arc::from(path), location(5, 3, 20), location(5, 10, 30));
    let snapshot_name = if cfg!(unix) { "absolute_path_span_unix" } else { "absolute_path_span_windows" };
    insta::assert_snapshot!(snapshot_name, span.to_string());
}

#[test]
fn merge_same_file_expands_span() {
    let span1 = create_span("file.vn", 2, 3, 5, 10);
    let span2 = create_span("file.vn", 1, 1, 6, 5);
    let merged = span1.merge(&span2);
    insta::assert_debug_snapshot!(merged);
}

#[test]
fn merge_different_files_no_change() {
    let span1 = create_span("file1.vn", 1, 1, 2, 2);
    let span2 = create_span("file2.vn", 3, 3, 4, 4);
    let merged = span1.merge(&span2);
    insta::assert_debug_snapshot!(merged);
}

#[test]
fn source_span_default() {
    let span = SourceSpan::default();
    insta::assert_debug_snapshot!(span);
}
