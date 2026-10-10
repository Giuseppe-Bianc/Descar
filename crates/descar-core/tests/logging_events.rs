//! Tests for the `tracing` events emitted by `descar-core`.
//!
//! The events are captured with a subscriber scoped to one thread, so the tests do not depend on
//! the binary and can run in parallel.
//!
//! Contract under test:
//! - diagnostics are logged with their error code and position;
//! - source text, identifiers and file paths never appear in an event;
//! - fine-grained events appear only at `TRACE`, anomalies at `WARN`.

use descar_core::error::compile_error::CompileError;
use descar_core::error::error_code::ErrorCode;
use descar_core::error::error_reporter::ErrorReporter;
use descar_core::lex::lexer::{Lexer, lexer_tokenize_with_errors};
use descar_core::location::line_tracker::LineTracker;
use descar_core::semantic::symbol_table::{ScopeKind, SymbolTable};
use descar_core::semantic::type_checker::TypeChecker;
use descar_core::syntax::parser::JsavParser;
use descar_core::utils::create_span;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::Level;

/// The parser limits its recursion to 1000 levels. Deep inputs need more stack than the default
/// stack of a test thread.
const STACK_SIZE: usize = 16 * 1024 * 1024;

const FILE_PATH: &str = "logging_probe_path.dr";

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl io::Write for Capture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("the capture buffer should not be poisoned").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Runs `run` on a thread with a large stack and returns the events it emitted up to `level`.
fn capture(level: Level, run: impl FnOnce() + Send + 'static) -> String {
    let buffer = Capture::default();
    let writer = buffer.clone();

    thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(level)
                .with_writer(move || writer.clone())
                .with_ansi(false)
                .without_time()
                .finish();
            tracing::subscriber::with_default(subscriber, run);
        })
        .expect("the capture thread should start")
        .join()
        .expect("the capture thread should not panic");

    let bytes = buffer.0.lock().expect("the capture buffer should not be poisoned").clone();
    String::from_utf8(bytes).expect("events should be UTF-8")
}

/// Runs the frontend stages like the binary does, stopping at the first stage with diagnostics.
fn run_frontend(source: String) -> impl FnOnce() + Send + 'static {
    move || {
        let mut lexer = Lexer::new(FILE_PATH, &source);
        let (tokens, lexer_errors) = lexer_tokenize_with_errors(&mut lexer);
        if !lexer_errors.is_empty() {
            return;
        }

        let (statements, parser_errors) = JsavParser::new(&tokens).parse();
        if !parser_errors.is_empty() {
            return;
        }

        let _errors = TypeChecker::new().check(&statements);
    }
}

fn event_lines(events: &str) -> Vec<&str> {
    events.lines().collect()
}

#[test]
fn lexical_errors_log_code_and_position_but_not_the_offending_text() {
    let source = "main {\n    var s: string = \"secret-text-1\\q\"\n}\n".to_owned();

    let events = capture(Level::DEBUG, run_frontend(source));

    assert!(events.contains("lexical error"), "events were: {events}");
    assert!(events.contains("code=\"E0007\""), "events were: {events}");
    assert!(events.contains("line=2"), "events were: {events}");
    assert!(!events.contains("secret-text-1"), "source text leaked into events: {events}");
    assert!(!events.contains(FILE_PATH), "the file path leaked into events: {events}");
}

#[test]
fn syntax_errors_log_code_position_and_recovery_at_trace() {
    let source = "main {\n    var s: string = \"secret-text-2\"\n    )\n}\n".to_owned();

    let events = capture(Level::TRACE, run_frontend(source));

    assert!(events.contains("syntax error"), "events were: {events}");
    assert!(events.contains("code=\"E1004\""), "events were: {events}");
    assert!(events.contains("line=3 column=5"), "events were: {events}");
    assert!(events.contains("skipping token after a failed statement"), "events were: {events}");
    assert!(!events.contains("secret-text-2"), "source text leaked into events: {events}");
    assert!(!events.contains(FILE_PATH), "the file path leaked into events: {events}");
}

#[test]
fn syntax_error_recovery_is_hidden_below_trace() {
    let source = "main {\n    )\n}\n".to_owned();

    let events = capture(Level::DEBUG, run_frontend(source));

    assert!(events.contains("syntax error"), "events were: {events}");
    assert!(!events.contains("skipping token"), "recovery must be a TRACE event, events were: {events}");
}

#[test]
fn recursion_limit_is_a_warning_with_depth_and_position() {
    let depth = 1100;
    let source = format!("main {{\n    var x: i32 = {}1{}\n}}\n", "(".repeat(depth), ")".repeat(depth));

    let events = capture(Level::WARN, run_frontend(source));

    assert!(events.contains("WARN"), "events were: {events}");
    assert!(events.contains("maximum recursion depth exceeded"), "events were: {events}");
    assert!(events.contains("limit=1000"), "events were: {events}");
    assert!(events.contains("line=2"), "events were: {events}");
    assert!(!events.contains("syntax error"), "diagnostics must be hidden at WARN, events were: {events}");
}

#[test]
fn type_errors_log_code_and_position_but_not_identifiers() {
    let source = "main {\n    var value: i32 = undefined_secret_7\n}\n".to_owned();

    let events = capture(Level::DEBUG, run_frontend(source));

    assert!(events.contains("type error"), "events were: {events}");
    assert!(events.contains("code=\"E2023\""), "events were: {events}");
    assert!(events.contains("line=2"), "events were: {events}");
    assert!(!events.contains("undefined_secret_7"), "an identifier leaked into events: {events}");
    assert!(!events.contains("value"), "an identifier leaked into events: {events}");
}

#[test]
fn duplicate_declarations_are_logged_as_type_errors() {
    let source = "main {\n    var twice: i32 = 1i32\n    var twice: i32 = 2i32\n}\n".to_owned();

    let events = capture(Level::DEBUG, run_frontend(source));

    assert!(events.contains("code=\"E2032\""), "events were: {events}");
    assert!(!events.contains("twice"), "an identifier leaked into events: {events}");
}

#[test]
fn function_span_gives_context_to_the_events_of_its_body() {
    let source = "fun first(a: i32): i32 {\n    return a\n}\n\nmain {\n    break\n}\n".to_owned();

    let events = capture(Level::TRACE, run_frontend(source));

    let type_error = event_lines(&events)
        .into_iter()
        .find(|line| line.contains("type error"))
        .unwrap_or_else(|| panic!("no type error event, events were: {events}"));
    assert!(type_error.contains("function{parameters=0 line=5 column=1}"), "line was: {type_error}");
}

#[test]
fn scopes_are_logged_only_at_trace() {
    let run = || {
        let mut table = SymbolTable::new();
        table.push_scope(ScopeKind::Function, None);
        table.push_scope(ScopeKind::Block, None);
        table.pop_scope();
        table.pop_scope();
    };

    let trace = capture(Level::TRACE, run);
    let debug = capture(Level::DEBUG, run);

    assert!(trace.contains("scope opened kind=Function depth=2"), "events were: {trace}");
    assert!(trace.contains("scope opened kind=Block depth=3"), "events were: {trace}");
    assert!(trace.contains("scope closed kind=Block symbols=0 depth=2"), "events were: {trace}");
    assert!(trace.contains("scope closed kind=Function symbols=0 depth=1"), "events were: {trace}");
    assert_eq!(debug, "", "scope events must be hidden at DEBUG");
}

#[test]
fn global_scope_misuse_is_a_warning_and_changes_nothing() {
    let events = capture(Level::WARN, || {
        let mut table = SymbolTable::new();

        table.push_scope(ScopeKind::Global, None);
        table.pop_scope();

        assert_eq!(table.scope_count(), 1);
    });

    assert!(events.contains("ignored a request to open a second global scope"), "events were: {events}");
    assert!(events.contains("ignored a request to close the global scope"), "events were: {events}");
}

#[test]
fn line_index_is_logged_at_trace() {
    let events = capture(Level::TRACE, || {
        let _tracker = LineTracker::new(FILE_PATH, String::from("one\ntwo\n"));
    });

    assert!(events.contains("line index built lines=3 chars=8 bytes=8"), "events were: {events}");
    assert!(!events.contains(FILE_PATH), "the file path leaked into events: {events}");
}

#[test]
fn diagnostic_without_a_source_line_is_reported_and_logged() {
    let reporter = ErrorReporter::new(LineTracker::new(FILE_PATH, String::from("only line")));
    let error = CompileError::SyntaxError {
        code: Some(ErrorCode::E1004),
        message: "secret-text-3".into(),
        span: create_span(FILE_PATH, 99, 1, 99, 2),
        help: None,
    };

    let events = capture(Level::DEBUG, move || {
        let report = reporter.report_errors(vec![error]);
        assert!(report.contains("E1004"), "the diagnostic must still be reported: {report}");
    });

    assert!(events.contains("source line not found"), "events were: {events}");
    assert!(events.contains("line=99"), "events were: {events}");
    assert!(!events.contains("secret-text-3"), "a message leaked into events: {events}");
}

#[test]
fn nothing_is_logged_when_the_source_is_valid_and_the_level_is_debug() {
    let source = "fun add(a: i32, b: i32): i32 {\n    return a + b\n}\n\nmain {\n}\n".to_owned();

    let events = capture(Level::DEBUG, run_frontend(source));

    assert_eq!(events, "", "valid source must not produce diagnostics events");
}
