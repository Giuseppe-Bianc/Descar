use console::set_colors_enabled;
use descar_core::error::compile_error::CompileError;
use descar_core::error::error_code::ErrorCode;
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::location::line_tracker::LineTracker;
use descar_core::location::source_location::SourceLocation;
use descar_core::location::source_span::SourceSpan;
use std::io;
use std::sync::Arc;

fn make_span(file: &str, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> SourceSpan {
    let end_offset = usize::from(!(start_line == end_line && start_col == end_col));
    SourceSpan::new(
        Arc::from(file),
        SourceLocation::new(start_line, start_col, 0, 0, usize::MAX, usize::MAX),
        SourceLocation::new(end_line, end_col, end_offset, end_offset, usize::MAX, usize::MAX),
    )
}

// =========================================================================
// Unit Tests: Empty and Basic Error Reporting
// =========================================================================

#[test]
fn test_report_errors_empty() {
    set_colors_enabled(false);
    let tracker = LineTracker::new("test.dr", "let a = 1;".to_string());
    let reporter = ErrorReporter::new(tracker);

    let output = reporter.report_errors(vec![]);
    assert_eq!(output, "");
}

#[test]
fn test_report_single_lexer_error_with_code_and_help() {
    set_colors_enabled(false);
    let source = "let a = @invalid;\nlet b = 2;";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let span = make_span("test.dr", 1, 9, 1, 10);
    let error = CompileError::LexerError {
        code: Some(ErrorCode::E0001),
        message: Arc::from("unexpected token '@'"),
        span,
        help: Some("remove the '@' symbol".to_string()),
    };

    let report = reporter.report_errors(vec![error]);

    assert!(report.contains("ERROR [E0001] LEX: unexpected token '@'"));
    assert!(report.contains("Location: test.dr:line 1:column 9-line 1:column 10"));
    assert!(report.contains("1 │ let a = @invalid;"));
    assert!(report.contains("│         ^"));
    assert!(report.contains("help: remove the '@' symbol"));
}

#[test]
fn test_report_syntax_error_without_code_or_help() {
    set_colors_enabled(false);
    let source = "fun foo() {";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let span = make_span("test.dr", 1, 11, 1, 12);
    let error =
        CompileError::SyntaxError { code: None, message: Arc::from("expected '}' to close block"), span, help: None };

    let report = reporter.report_errors(vec![error]);

    assert!(report.contains("ERROR SYNTAX: expected '}' to close block"));
    assert!(report.contains("Location: test.dr:line 1:column 11-line 1:column 12"));
    assert!(report.contains("1 │ fun foo() {"));
    assert!(!report.contains("help:"));
}

#[test]
fn test_report_type_error() {
    set_colors_enabled(false);
    let source = "let x: i32 = \"hello\";";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let span = make_span("test.dr", 1, 14, 1, 21);
    let error = CompileError::TypeError {
        code: Some(ErrorCode::E2001),
        message: Arc::from("mismatched types: expected i32, found string"),
        span,
        help: Some("use an integer literal instead".to_string()),
    };

    let report = reporter.report_errors(vec![error]);

    assert!(report.contains("ERROR [E2001] TYPE: mismatched types: expected i32, found string"));
    assert!(report.contains("Location: test.dr:line 1:column 14-line 1:column 21"));
    assert!(report.contains("1 │ let x: i32 = \"hello\";"));
    // Underline length should be 21 - 14 = 7
    assert!(report.contains("│              ^^^^^^^"));
    assert!(report.contains("help: use an integer literal instead"));
}

#[test]
fn test_report_io_error() {
    set_colors_enabled(false);
    let tracker = LineTracker::new("test.dr", String::new());
    let reporter = ErrorReporter::new(tracker);

    let io_err = io::Error::new(io::ErrorKind::NotFound, "file 'main.dr' not found");
    let error = CompileError::IoError(io_err);

    let report = reporter.report_errors(vec![error]);

    assert!(report.contains("ERROR I/O: file 'main.dr' not found"));
    assert!(!report.contains("Location:"));
}

// =========================================================================
// Corner & Edge Case Tests
// =========================================================================

#[test]
fn test_report_multiline_span_underline() {
    set_colors_enabled(false);
    let source = "/* unclosed\n   multiline\n   comment";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let span = make_span("test.dr", 1, 1, 3, 11);
    let error = CompileError::LexerError {
        code: Some(ErrorCode::E0005),
        message: Arc::from("unterminated comment block"),
        span,
        help: Some("close the comment with '*/'".to_string()),
    };

    let report = reporter.report_errors(vec![error]);

    // When start_line != end_line, underline is just a single caret at start_offset
    assert!(report.contains("1 │ /* unclosed"));
    assert!(report.contains("│ ^"));
    assert!(report.contains("(error spans lines 1-3)"));
}

#[test]
fn test_report_error_with_zero_length_span() {
    set_colors_enabled(false);
    let source = "let a = ;";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    // start_col == end_col (length 0 -> clamped to .max(1))
    let span = make_span("test.dr", 1, 9, 1, 9);
    let error = CompileError::SyntaxError {
        code: Some(ErrorCode::E1004),
        message: Arc::from("expected expression before ';'"),
        span,
        help: None,
    };

    let report = reporter.report_errors(vec![error]);

    // Should underline with at least 1 '^'
    assert!(report.contains("│         ^"));
}

#[test]
fn test_report_line_not_found_in_tracker() {
    set_colors_enabled(false);
    // LineTracker has only 1 line, but error points to line 10
    let source = "let a = 1;";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let span = make_span("test.dr", 10, 5, 10, 10);
    let error = CompileError::SyntaxError {
        code: None,
        message: Arc::from("phantom error outside file boundaries"),
        span,
        help: None,
    };

    let report = reporter.report_errors(vec![error]);

    // Header and location should still be printed, but no source line snippet or underline
    assert!(report.contains("ERROR SYNTAX: phantom error outside file boundaries"));
    assert!(report.contains("Location: test.dr:line 10:column 5-line 10:column 10"));
    assert!(!report.contains("10 │"));
    assert!(!report.contains("│ ^"));
}

#[test]
fn test_report_empty_source_line() {
    set_colors_enabled(false);
    let source = "\n\n";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    // Line 1 is empty
    let span = make_span("test.dr", 1, 1, 1, 1);
    let error = CompileError::LexerError { code: None, message: Arc::from("unexpected newline"), span, help: None };

    let report = reporter.report_errors(vec![error]);

    // Empty source lines skip snippet rendering
    assert!(report.contains("ERROR LEX: unexpected newline"));
    assert!(report.contains("Location: test.dr:line 1:column 1\n"));
    assert!(!report.contains("1 │"));
}

#[test]
fn test_report_column_zero_saturating_sub() {
    set_colors_enabled(false);
    let source = "bad token";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    // Column 0 should saturate to 0 offset rather than underflow
    let span = make_span("test.dr", 1, 0, 1, 3);
    let error = CompileError::LexerError { code: None, message: Arc::from("malformed span column"), span, help: None };

    let report = reporter.report_errors(vec![error]);

    assert!(report.contains("1 │ bad token"));
    assert!(report.contains("│ ^^^"));
}

#[test]
fn test_report_inverted_columns_in_same_line() {
    set_colors_enabled(false);
    let source = "xyz = 123;";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    // end_col < start_col: saturating_sub ensures no panic and .max(1) yields 1 '^'
    let span = make_span("test.dr", 1, 5, 1, 2);
    let error = CompileError::SyntaxError { code: None, message: Arc::from("inverted span"), span, help: None };

    let report = reporter.report_errors(vec![error]);

    assert!(report.contains("1 │ xyz = 123;"));
    assert!(report.contains("│     ^"));
}

#[test]
fn test_report_multiple_errors_sequential() {
    set_colors_enabled(false);
    let source = "let a = @;\nlet b: i32 = false;";
    let tracker = LineTracker::new("test.dr", source.to_string());
    let reporter = ErrorReporter::new(tracker);

    let err1 = CompileError::LexerError {
        code: Some(ErrorCode::E0001),
        message: Arc::from("unexpected character '@'"),
        span: make_span("test.dr", 1, 9, 1, 10),
        help: None,
    };
    let err2 = CompileError::TypeError {
        code: Some(ErrorCode::E2001),
        message: Arc::from("type mismatch"),
        span: make_span("test.dr", 2, 14, 2, 19),
        help: Some("expected i32".to_string()),
    };
    let err3 = CompileError::IoError(io::Error::other("disk error"));

    let report = reporter.report_errors(vec![err1, err2, err3]);

    assert!(report.contains("ERROR [E0001] LEX: unexpected character '@'"));
    assert!(report.contains("ERROR [E2001] TYPE: type mismatch"));
    assert!(report.contains("ERROR I/O: disk error"));
}
