use clap::Parser;
use descar_cli::cli::{Args, Command};
use insta::assert_snapshot;

#[test]
fn snapshots_root_command() {
    let args = Args::try_parse_from(["descar"]).expect("root command should parse");
    assert_snapshot!("root_command", format!("{args:#?}"));
}

#[test]
fn snapshots_compile_minimal_configuration() {
    let args = Args::try_parse_from(["descar", "compile", "program.dr"]).expect("minimal compile command should parse");
    assert_snapshot!("compile_minimal", format!("{args:#?}"));
}

#[test]
fn snapshots_compile_full_configuration() {
    let args = Args::try_parse_from([
        "descar",
        "compile",
        "examples/hello.dr",
        "--output",
        "build/hello",
        "--optimize",
        "aggressive",
        "--emit-ir",
        "--diagnostics",
        "-vvv",
        "--quiet",
    ])
    .expect("fully configured compile command should parse");
    assert_snapshot!("compile_full", format!("{args:#?}"));
}

#[test]
fn snapshots_check_full_configuration() {
    let args = Args::try_parse_from(["descar", "check", "examples/hello.dr", "-vvv", "--quiet"])
        .expect("fully configured check command should parse");
    assert_snapshot!("check_full", format!("{args:#?}"));
}

#[test]
fn snapshots_case_insensitive_source_paths() {
    let paths = ["program.dr", "program.DR", "PROGRAM.Dr"];
    let parsed: Vec<_> = paths
        .into_iter()
        .map(|path| Args::try_parse_from(["descar", "check", path]).expect("path should parse"))
        .collect::<Vec<_>>();

    assert_snapshot!("case_insensitive_source_paths", format!("{parsed:#?}"));
}

#[test]
fn snapshots_invalid_source_extension_error() {
    let error = Args::try_parse_from(["descar", "check", "program.txt"])
        .expect_err("invalid source extension should be rejected");
    assert_snapshot!("invalid_source_extension", error.to_string());
}

#[test]
fn snapshots_missing_input_errors() {
    for command in ["compile", "check"] {
        let error = Args::try_parse_from(["descar", command]).expect_err("missing source path should be rejected");
        assert_snapshot!(format!("missing_input_{command}"), error.to_string());
    }
}

#[test]
fn snapshots_unknown_command_error() {
    let error =
        Args::try_parse_from(["descar", "build", "program.dr"]).expect_err("unknown command should be rejected");
    assert_snapshot!("unknown_command", error.to_string());
}

#[test]
fn snapshots_invalid_optimization_error() {
    let error = Args::try_parse_from(["descar", "compile", "program.dr", "--optimize", "turbo"])
        .expect_err("invalid optimization level should be rejected");
    assert_snapshot!("invalid_optimization", error.to_string());
}

#[test]
fn snapshots_unknown_compile_option_error() {
    let error = Args::try_parse_from(["descar", "compile", "program.dr", "--unknown"])
        .expect_err("unknown compile option should be rejected");
    assert_snapshot!("unknown_compile_option", error.to_string());
}

#[test]
fn snapshots_output_path_edge_case() {
    let args = Args::try_parse_from([
        "descar",
        "compile",
        "src/program.main.dr",
        "--output",
        "build/../dist/program",
        "--optimize",
        "basic",
    ])
    .expect("edge-case output path should parse");

    let Some(Command::Compile(args)) = args.command else {
        panic!("expected compile command");
    };

    assert_snapshot!("output_path_edge_case", format!("{args:#?}"));
}
