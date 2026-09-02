use clap::Parser;
use descar_cli::cli::{Args, Command, OptimizationLevel};
use std::path::PathBuf;

#[test]
fn parses_root_command_without_subcommand() {
    let args = Args::try_parse_from(["descar"]).expect("root command should parse");
    assert!(args.command.is_none());
}

#[test]
fn parses_compile_command_with_all_options() {
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

    assert_eq!(args.input, PathBuf::from("program.dr"));
    assert_eq!(args.output, Some(PathBuf::from("program")));
    assert_eq!(args.optimize, OptimizationLevel::Aggressive);
    assert!(args.emit_ir);
    assert!(args.diagnostics);
    assert_eq!(args.logging.verbose, 2);
    assert!(!args.logging.quiet);
}

#[test]
fn parses_compile_command_with_minimum_options() {
    let args = Args::try_parse_from(["descar", "compile", "program.dr"]).expect("minimal compile command should parse");

    let Some(Command::Compile(args)) = args.command else {
        panic!("expected compile command");
    };

    assert_eq!(args.input, PathBuf::from("program.dr"));
    assert_eq!(args.output, None);
    assert_eq!(args.optimize, OptimizationLevel::None);
    assert!(!args.emit_ir);
    assert!(!args.diagnostics);
    assert_eq!(args.logging.verbose, 0);
    assert!(!args.logging.quiet);
}

#[test]
fn parses_all_optimization_levels() {
    for (value, expected) in [
        ("none", OptimizationLevel::None),
        ("basic", OptimizationLevel::Basic),
        ("aggressive", OptimizationLevel::Aggressive),
    ] {
        let args = Args::try_parse_from(["descar", "compile", "program.dr", "--optimize", value])
            .expect("optimization level should parse");

        let Some(Command::Compile(args)) = args.command else {
            panic!("expected compile command");
        };

        assert_eq!(args.optimize, expected);
    }
}

#[test]
fn parses_check_command_with_quiet() {
    let args =
        Args::try_parse_from(["descar", "check", "program.dr", "-vvv", "--quiet"]).expect("check command should parse");

    let Some(Command::Check(args)) = args.command else {
        panic!("expected check command");
    };

    assert_eq!(args.input, PathBuf::from("program.dr"));
    assert_eq!(args.logging.verbose, 3);
    assert!(args.logging.quiet);
}

#[test]
fn parses_check_command_with_minimum_options() {
    let args = Args::try_parse_from(["descar", "check", "program.dr"]).expect("minimal check command should parse");

    let Some(Command::Check(args)) = args.command else {
        panic!("expected check command");
    };

    assert_eq!(args.input, PathBuf::from("program.dr"));
    assert_eq!(args.logging.verbose, 0);
    assert!(!args.logging.quiet);
}

#[test]
fn supports_repeated_short_verbosity_flags() {
    for (argv, expected) in [
        (["descar", "check", "program.dr", "-v"], 1),
        (["descar", "check", "program.dr", "-vv"], 2),
        (["descar", "check", "program.dr", "-vvv"], 3),
        (["descar", "check", "program.dr", "-vvvv"], 4),
    ] {
        let args = Args::try_parse_from(argv).expect("verbosity flags should parse");

        let Some(Command::Check(args)) = args.command else {
            panic!("expected check command");
        };

        assert_eq!(args.logging.verbose, expected);
    }
}

#[test]
fn accepts_long_flag_forms() {
    let args = Args::try_parse_from(["descar", "check", "program.dr", "--verbose", "--verbose", "--quiet"])
        .expect("long flag forms should parse");

    let Some(Command::Check(args)) = args.command else {
        panic!("expected check command");
    };

    assert_eq!(args.logging.verbose, 2);
    assert!(args.logging.quiet);
}

#[test]
fn accepts_dr_extension_case_insensitively() {
    for path in ["program.dr", "program.DR", "PROGRAM.Dr"] {
        Args::try_parse_from(["descar", "check", path]).expect(".dr extension should be accepted case-insensitively");
    }
}

#[test]
fn accepts_nested_and_dotted_source_paths() {
    for path in ["examples/hello.dr", "src/program.main.dr", "../shared/test-file.dr", "./program.dr"] {
        let args = Args::try_parse_from(["descar", "check", path]).expect("valid .dr source path should parse");

        let Some(Command::Check(args)) = args.command else {
            panic!("expected check command");
        };

        assert_eq!(args.input, PathBuf::from(path));
    }
}

#[test]
fn preserves_output_path_without_normalization() {
    let args = Args::try_parse_from(["descar", "compile", "src/program.dr", "--output", "build/../dist/program"])
        .expect("compile command with output path should parse");

    let Some(Command::Compile(args)) = args.command else {
        panic!("expected compile command");
    };

    assert_eq!(args.output, Some(PathBuf::from("build/../dist/program")));
}

#[test]
fn rejects_invalid_source_extensions() {
    for path in ["program.txt", "program.dr.txt", "program"] {
        let result = Args::try_parse_from(["descar", "check", path]);
        assert!(result.is_err(), "expected invalid source path to be rejected: {path}");
    }
}

#[test]
fn rejects_missing_source_path() {
    for command in ["compile", "check"] {
        let result = Args::try_parse_from(["descar", command]);
        assert!(result.is_err(), "expected missing input to be rejected: {command}");
    }
}

#[test]
fn rejects_unknown_command() {
    let result = Args::try_parse_from(["descar", "build", "program.dr"]);
    assert!(result.is_err());
}

#[test]
fn rejects_unknown_compile_option() {
    let result = Args::try_parse_from(["descar", "compile", "program.dr", "--unknown"]);
    assert!(result.is_err());
}

#[test]
fn rejects_invalid_optimization_level() {
    let result = Args::try_parse_from(["descar", "compile", "program.dr", "--optimize", "turbo"]);
    assert!(result.is_err());
}
