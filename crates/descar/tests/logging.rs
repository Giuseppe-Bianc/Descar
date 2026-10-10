//! End-to-end tests for the logging contract of the `descar` binary.
//!
//! Contract under test:
//! - log events go to stderr, never to stdout;
//! - stdout carries only functional output (`--version`, help);
//! - `-v`, `-vv`, `-vvv` and `-q` select the maximum level;
//! - `-q` silences log events but not compiler diagnostics or fatal errors;
//! - source text is never copied into log events.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const LEVELS: [&str; 5] = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

fn fixture(name: &str) -> String {
    format!("{}/../../dr_files/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_descar"))
        .args(args)
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .output()
        .expect("the descar binary should start")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn has_level(text: &str, level: &str) -> bool {
    text.lines().any(|line| line.split_whitespace().next() == Some(level))
}

#[test]
fn default_run_is_silent_on_success() {
    let output = run(&["compile", &fixture("simple_test.dr")]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn single_v_shows_warnings_and_hides_lower_levels() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-O", "basic", "-v"]);
    let err = stderr(&output);

    assert!(output.status.success());
    assert!(has_level(&err, "WARN"), "stderr was: {err}");
    for hidden in ["INFO", "DEBUG", "TRACE"] {
        assert!(!has_level(&err, hidden), "{hidden} must be hidden at -v, stderr was: {err}");
    }
    assert_eq!(stdout(&output), "");
}

#[test]
fn double_v_shows_info_and_debug_and_hides_trace() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-vv"]);
    let err = stderr(&output);

    assert!(output.status.success());
    assert!(has_level(&err, "INFO"), "stderr was: {err}");
    assert!(has_level(&err, "DEBUG"), "stderr was: {err}");
    assert!(!has_level(&err, "TRACE"), "stderr was: {err}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn triple_v_shows_trace_with_source_location() {
    let output = run(&["check", &fixture("simple_test.dr"), "-vvv"]);
    let err = stderr(&output);

    assert!(output.status.success());
    assert!(has_level(&err, "TRACE"), "stderr was: {err}");
    assert!(err.contains("main.rs:"), "trace lines must carry file:line, stderr was: {err}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn more_than_three_v_saturates_at_trace() {
    let output = run(&["check", &fixture("simple_test.dr"), "-vvvvvv"]);

    assert!(output.status.success());
    assert!(has_level(&stderr(&output), "TRACE"));
}

#[test]
fn quiet_silences_log_events_even_with_verbose() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-q", "-vvv"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn quiet_keeps_compiler_diagnostics_and_exit_code() {
    let output = run(&["compile", &fixture("break_outside_loop.dr"), "-q"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("E2009"), "stderr was: {err}");
    for level in ["WARN", "INFO", "DEBUG", "TRACE"] {
        assert!(!has_level(&err, level), "no log event may appear with -q, stderr was: {err}");
    }
    assert_eq!(stdout(&output), "");
}

#[test]
fn quiet_keeps_fatal_io_errors() {
    let output = run(&["compile", "this_file_does_not_exist.dr", "-q"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("ERROR: I/O"), "stderr was: {err}");
    assert_eq!(stdout(&output), "");
}

#[test]
fn failing_source_never_writes_to_stdout_at_any_level() {
    let path = fixture("break_outside_loop.dr");

    for flags in [&[][..], &["-v"], &["-vv"], &["-vvv"]] {
        let mut args = vec!["compile", path.as_str()];
        args.extend_from_slice(flags);

        let output = run(&args);

        assert_eq!(output.status.code(), Some(1), "flags: {flags:?}");
        assert_eq!(stdout(&output), "", "flags: {flags:?}");
        assert!(stderr(&output).contains("E2009"), "flags: {flags:?}");
    }
}

#[test]
fn debug_trace_records_each_frontend_stage_and_its_failure() {
    let output = run(&["compile", &fixture("break_outside_loop.dr"), "-vv"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("lexing finished"), "stderr was: {err}");
    assert!(err.contains("parsing finished"), "stderr was: {err}");
    assert!(err.contains("type checking finished"), "stderr was: {err}");
    assert!(err.contains("frontend stopped"), "stderr was: {err}");
}

#[test]
fn version_goes_to_stdout_only() {
    let output = run(&["--version"]);

    assert!(output.status.success());
    assert!(stdout(&output).contains(env!("CARGO_PKG_VERSION")));
    assert_eq!(stderr(&output), "");
}

#[test]
fn help_without_command_goes_to_stdout_only() {
    let output = run(&[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("Usage"));
    assert_eq!(stderr(&output), "");
}

#[test]
fn redirected_stderr_has_no_ansi_escape_codes() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-O", "basic", "-vvv"]);
    let err = stderr(&output);

    assert_ne!(err, "");
    assert!(!err.contains('\u{1b}'), "stderr must be plain text when it is not a terminal: {err:?}");
}

#[test]
fn every_log_line_is_plain_text_starting_with_a_level() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-O", "basic", "-vvv"]);
    let err = stderr(&output);

    assert_ne!(err, "");
    for line in err.lines() {
        let first = line.split_whitespace().next().unwrap_or_default();
        assert!(LEVELS.contains(&first), "line does not start with a level: {line:?}");
    }
}

#[test]
fn log_events_carry_command_context() {
    let output = run(&["compile", &fixture("simple_test.dr"), "-vv"]);
    let err = stderr(&output);

    assert!(err.contains("compile{"), "events must carry the command span, stderr was: {err}");
    assert!(err.contains("simple_test.dr"), "stderr was: {err}");
}

#[test]
fn source_text_is_never_written_to_logs() {
    let secret = "super-secret-token-12345";
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(&dir).expect("the target tmp directory should be creatable");
    let path = dir.join("logging_secret_probe.dr");
    fs::write(&path, format!("main {{\n    var token: string = \"{secret}\"\n}}\n"))
        .expect("the probe should be written");

    let output = run(&["compile", path.to_str().expect("UTF-8 path"), "-vvv"]);
    let err = stderr(&output);

    assert!(output.status.success(), "stderr was: {err}");
    assert_ne!(err, "");
    assert!(!err.contains(secret), "source text leaked into logs: {err}");
    assert_eq!(stdout(&output), "");
}

#[cfg(unix)]
#[test]
fn control_characters_in_paths_are_escaped_in_logs() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(&dir).expect("the target tmp directory should be creatable");
    let path = dir.join("evil\n INFO forged line \u{1b}[31mred\u{1b}[0m.dr");
    fs::copy(fixture("simple_test.dr"), &path).expect("the probe should be copied");

    let output = run(&["check", path.to_str().expect("UTF-8 path"), "-vv"]);
    let err = stderr(&output);

    assert!(output.status.success(), "stderr was: {err}");
    assert!(!err.is_empty(), "the probe needs log events to inspect");
    assert!(!err.contains('\u{1b}'), "an escape character reached stderr: {err:?}");
    for line in err.lines() {
        let first = line.split_whitespace().next().unwrap_or_default();
        assert!(LEVELS.contains(&first), "a forged or split line reached stderr: {line:?}");
    }
}

/// Log events of a command. Compiler diagnostics share stderr with them but carry no command span.
fn events(text: &str) -> Vec<&str> {
    text.lines().filter(|line| line.contains("compile{") || line.contains("check{")).collect()
}

fn write_probe(name: &str, content: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(&dir).expect("the target tmp directory should be creatable");
    let path = dir.join(name);
    fs::write(&path, content).expect("the probe should be written");
    path
}

#[test]
fn failing_source_emits_no_events_at_the_default_level() {
    let output = run(&["compile", &fixture("break_outside_loop.dr")]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("E2009"), "stderr was: {err}");
    assert_eq!(events(&err), Vec::<&str>::new(), "events must stay below ERROR, stderr was: {err}");
}

#[test]
fn type_errors_are_logged_with_code_and_position() {
    let output = run(&["compile", &fixture("break_outside_loop.dr"), "-vv"]);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        events(&err)
            .iter()
            .any(|line| line.contains("type error") && line.contains("code=\"E2009\"") && line.contains("line=5")),
        "stderr was: {err}"
    );
}

#[test]
fn lexical_error_events_carry_code_and_position_but_not_source_text() {
    let secret = "secret-lexical-probe-1";
    let path = write_probe("logging_lexical_probe.dr", &format!("main {{\n    var s: string = \"{secret}\\q\"\n}}\n"));

    let output = run(&["compile", path.to_str().expect("UTF-8 path"), "-vvv"]);
    let err = stderr(&output);
    let logged = events(&err);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("E0007"), "the diagnostic must still be printed, stderr was: {err}");
    assert!(
        logged
            .iter()
            .any(|line| line.contains("lexical error") && line.contains("code=\"E0007\"") && line.contains("line=2")),
        "stderr was: {err}"
    );
    assert!(logged.iter().all(|line| !line.contains(secret)), "source text leaked into events: {logged:?}");
}

#[test]
fn syntax_error_events_carry_code_position_and_recovery_but_not_source_text() {
    let secret = "secret-syntax-probe-2";
    let path =
        write_probe("logging_syntax_probe.dr", &format!("main {{\n    var s: string = \"{secret}\"\n    )\n}}\n"));

    let output = run(&["compile", path.to_str().expect("UTF-8 path"), "-vvv"]);
    let err = stderr(&output);
    let logged = events(&err);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        logged.iter().any(|line| line.contains("syntax error")
            && line.contains("code=\"E1004\"")
            && line.contains("line=3 column=5")),
        "stderr was: {err}"
    );
    assert!(logged.iter().any(|line| line.contains("skipping token after a failed statement")), "stderr was: {err}");
    assert!(logged.iter().all(|line| !line.contains(secret)), "source text leaked into events: {logged:?}");
}

#[test]
fn recursion_limit_warning_is_visible_at_single_v_and_hidden_by_default() {
    let depth = 1100;
    let source = format!("main {{\n    var x: i32 = {}1{}\n}}\n", "(".repeat(depth), ")".repeat(depth));
    let path = write_probe("logging_recursion_probe.dr", &source);
    let path = path.to_str().expect("UTF-8 path");

    let verbose = run(&["compile", path, "-v"]);
    let verbose_err = stderr(&verbose);
    let default = run(&["compile", path]);

    assert_eq!(verbose.status.code(), Some(1));
    assert!(has_level(&verbose_err, "WARN"), "stderr was: {verbose_err}");
    assert!(verbose_err.contains("maximum recursion depth exceeded"), "stderr was: {verbose_err}");
    assert_eq!(default.status.code(), Some(1));
    assert_eq!(events(&stderr(&default)), Vec::<&str>::new());
}

#[test]
fn scope_events_appear_only_at_trace_level() {
    let debug = stderr(&run(&["check", &fixture("simple_test.dr"), "-vv"]));
    let trace = stderr(&run(&["check", &fixture("simple_test.dr"), "-vvv"]));

    assert!(!debug.contains("scope opened"), "stderr was: {debug}");
    assert!(trace.contains("scope opened"), "stderr was: {trace}");
    assert!(trace.contains("scope closed"), "stderr was: {trace}");
    assert!(trace.contains("function{parameters=2 line=1 column=1}"), "stderr was: {trace}");
}

#[test]
fn started_event_reports_the_version_and_success_reports_the_elapsed_time() {
    for command in ["check", "compile"] {
        let output = run(&[command, &fixture("simple_test.dr"), "-vv"]);
        let err = stderr(&output);

        assert!(output.status.success(), "{command} failed, stderr was: {err}");
        assert!(err.contains(&format!("version=\"{}\"", env!("CARGO_PKG_VERSION"))), "{command}: {err}");
        assert!(err.contains("succeeded elapsed="), "{command}: {err}");
    }
}
