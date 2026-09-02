use clap::Parser;
use descar_cli::cli::{Args, Command, OptimizationLevel};
use insta::assert_snapshot;

#[test]
fn parses_root_command_without_subcommand() {
    let args = Args::try_parse_from(["descar"]).expect("root command should parse");
    assert!(args.command.is_none());
}

#[test]
fn parses_compile_command() {
    let args = Args::try_parse_from([
        "descar",
        "compile",
        "program.dr",
        "--output",
        "program",
        "--optimize",
        "aggressive",
        "--emit-ir",
        "--diagnostics",
        "-vv",
    ])
    .expect("compile command should parse");

    let Some(Command::Compile(args)) = args.command else {
        panic!("expected compile command");
    };

    assert_eq!(args.input, "program.dr".into());
    assert_eq!(args.output, Some("program".into()));
    assert_eq!(args.optimize, OptimizationLevel::Aggressive);
    assert!(args.emit_ir);
    assert!(args.diagnostics);
    assert_eq!(args.logging.verbose, 2);
    assert!(!args.logging.quiet);
}

#[test]
fn parses_check_command_with_quiet() {
    let args = Args::try_parse_from(["descar", "check", "program.dr", "-vvv", "--quiet"])
        .expect("check command should parse");

    let Some(Command::Check(args)) = args.command else {
        panic!("expected check command");
    };

    assert_eq!(args.input, "program.dr".into());
    assert_eq!(args.logging.verbose, 3);
    assert!(args.logging.quiet);
}

#[test]
fn rejects_invalid_source_extension() {
    let result = Args::try_parse_from(["descar", "check", "program.txt"]);
    let error = result.expect_err("non-.dr input must be rejected");

    assert_snapshot!(error.to_string(), @"
error: invalid value 'program.txt' for 'FILE': expected a path to a .dr file

For more information, try '--help'.
");
}

#[test]
fn accepts_dr_extension_case_insensitively() {
    for path in ["program.dr", "program.DR", "PROGRAM.Dr"] {
        Args::try_parse_from(["descar", "check", path])
            .expect(".dr extension should be accepted case-insensitively");
    }
}

#[test]
fn snapshots_compile_configuration() {
    let args = Args::try_parse_from([
        "descar",
        "compile",
        "examples/hello.dr",
        "--output",
        "build/hello",
        "--optimize",
        "basic",
        "--emit-ir",
        "--diagnostics",
        "-vv",
    ])
    .expect("compile command should parse");

    let Some(Command::Compile(args)) = args.command else {
        panic!("expected compile command");
    };

    assert_snapshot!(format!(
        "input={}\noutput={}\noptimize={:?}\nemit_ir={}\ndiagnostics={}\nverbose={}\nquiet={}",
        args.input.display(),
        args.output.as_deref().map_or("<none>".to_string(), |path| path.display().to_string()),
        args.optimize,
        args.emit_ir,
        args.diagnostics,
        args.logging.verbose,
        args.logging.quiet,
    ), @"
input=examples/hello.dr
output=build/hello
optimize=Basic
emit_ir=true
diagnostics=true
verbose=2
quiet=false
");
}
