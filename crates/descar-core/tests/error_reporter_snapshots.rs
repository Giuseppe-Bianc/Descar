use console::set_colors_enabled;
use descar_core::error::compile_error::CompileError;
use descar_core::error::error_code::ErrorCode;
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::location::line_tracker::LineTracker;
use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::SourceSpan;
use insta::assert_snapshot;
use std::io;
use std::sync::Arc;

fn make_span(file: &str, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> SourceSpan {
    SourceSpan::new(
        Arc::from(file),
        SourceLocation::new(start_line, start_col, 0, 0, usize::MAX, usize::MAX),
        SourceLocation::new(end_line, end_col, 0, 0, usize::MAX, usize::MAX),
    )
}

#[test]
fn snapshots_error_categories_and_codes() {
    set_colors_enabled(false);

    let source = "let x = @;\nfun test() {\n    let y: i32 = \"str\";\n}\n";
    let tracker = LineTracker::new("sample.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let errors = vec![
        CompileError::LexerError {
            code: Some(ErrorCode::E0001),
            message: Arc::from("unexpected character '@'"),
            span: make_span("sample.dr", 1, 9, 1, 10),
            help: Some("replace with a valid literal or identifier".to_string()),
        },
        CompileError::SyntaxError {
            code: Some(ErrorCode::E1004),
            message: Arc::from("unexpected token in block"),
            span: make_span("sample.dr", 2, 5, 2, 9),
            help: None,
        },
        CompileError::TypeError {
            code: Some(ErrorCode::E2001),
            message: Arc::from("type mismatch: expected i32, found string"),
            span: make_span("sample.dr", 3, 18, 3, 23),
            help: Some("consider parsing the string as integer".to_string()),
        },
        CompileError::IoError(io::Error::new(io::ErrorKind::PermissionDenied, "access denied to imported file")),
    ];

    let rendered = reporter.report_errors(errors);
    assert_snapshot!("error_categories_and_codes", rendered);
}

#[test]
fn snapshots_multiline_and_boundary_formatting() {
    set_colors_enabled(false);

    let source = "struct Foo {\n    a: i32,\n    b: string,\n}\n";
    let tracker = LineTracker::new("sample.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let errors = vec![
        // Multiline error
        CompileError::SyntaxError {
            code: Some(ErrorCode::E1002),
            message: Arc::from("struct definition incomplete or invalid"),
            span: make_span("sample.dr", 1, 1, 4, 2),
            help: Some("check opening and closing braces".to_string()),
        },
        // Single character span
        CompileError::LexerError {
            code: None,
            message: Arc::from("invalid character"),
            span: make_span("sample.dr", 2, 5, 2, 6),
            help: None,
        },
        // Zero length / same start and end column
        CompileError::SyntaxError {
            code: Some(ErrorCode::E1013),
            message: Arc::from("missing semicolon"),
            span: make_span("sample.dr", 3, 14, 3, 14),
            help: Some("insert ';' here".to_string()),
        },
    ];

    let rendered = reporter.report_errors(errors);
    assert_snapshot!("multiline_and_boundary_formatting", rendered);
}
