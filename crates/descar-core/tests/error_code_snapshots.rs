use descar_core::error::error_code::{CompilerPhase, ErrorCode, Severity};
use insta::assert_snapshot;

const ALL_ERROR_CODES: &[ErrorCode] = &[
    ErrorCode::E0001,
    ErrorCode::E0002,
    ErrorCode::E0003,
    ErrorCode::E0004,
    ErrorCode::E0005,
    ErrorCode::E0006,
    ErrorCode::E0007,
    ErrorCode::E0008,
    ErrorCode::E0009,
    ErrorCode::E0010,
    ErrorCode::E1001,
    ErrorCode::E1002,
    ErrorCode::E1003,
    ErrorCode::E1004,
    ErrorCode::E1005,
    ErrorCode::E1006,
    ErrorCode::E1007,
    ErrorCode::E1008,
    ErrorCode::E1009,
    ErrorCode::E1010,
    ErrorCode::E1011,
    ErrorCode::E1012,
    ErrorCode::E1013,
    ErrorCode::E1014,
    ErrorCode::E1015,
    ErrorCode::E2001,
    ErrorCode::E2002,
    ErrorCode::E2003,
    ErrorCode::E2004,
    ErrorCode::E2005,
    ErrorCode::E2006,
    ErrorCode::E2007,
    ErrorCode::E2008,
    ErrorCode::E2009,
    ErrorCode::E2010,
    ErrorCode::E2011,
    ErrorCode::E2012,
    ErrorCode::E2013,
    ErrorCode::E2014,
    ErrorCode::E2015,
    ErrorCode::E2016,
    ErrorCode::E2017,
    ErrorCode::E2018,
    ErrorCode::E2019,
    ErrorCode::E2020,
    ErrorCode::E2021,
    ErrorCode::E2022,
    ErrorCode::E2023,
    ErrorCode::E2024,
    ErrorCode::E2025,
    ErrorCode::E2026,
    ErrorCode::E2027,
    ErrorCode::E2028,
    ErrorCode::E2029,
    ErrorCode::E2030,
    ErrorCode::E2031,
    ErrorCode::E2032,
    ErrorCode::E3001,
    ErrorCode::E3002,
    ErrorCode::E3003,
    ErrorCode::E3004,
    ErrorCode::E3005,
    ErrorCode::E3006,
    ErrorCode::E3007,
    ErrorCode::E3008,
    ErrorCode::E4001,
    ErrorCode::E4002,
    ErrorCode::E4003,
    ErrorCode::E4004,
    ErrorCode::E4005,
    ErrorCode::E5001,
    ErrorCode::E5002,
    ErrorCode::E5003,
    ErrorCode::E5004,
    ErrorCode::E5005,
];

#[test]
fn snapshots_all_error_code_mappings() {
    let rendered = ALL_ERROR_CODES
        .iter()
        .map(|error| {
            format!(
                "{debug}\ncode={code}\nnumeric={numeric}\nphase={phase:?}\nseverity={severity:?}\nmessage={message}\ndisplay={display}",
                debug = format_args!("{error:?}"),
                code = error.code(),
                numeric = error.numeric_code(),
                phase = error.phase(),
                severity = error.severity(),
                message = error.message(),
                display = error,
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("all_error_code_mappings", rendered);
}

#[test]
fn snapshots_enum_representations() {
    let severities = [Severity::Note, Severity::Warning, Severity::Error, Severity::Fatal];
    let phases = [
        CompilerPhase::Lexer,
        CompilerPhase::Parser,
        CompilerPhase::Semantic,
        CompilerPhase::IrGeneration,
        CompilerPhase::CodeGeneration,
        CompilerPhase::System,
    ];

    let rendered = format!(
        "severities:\n{}\nphases:\n{}",
        severities
            .iter()
            .map(|severity| format!("{severity:?} => {severity}"))
            .collect::<Vec<_>>()
            .join("\n"),
        phases
            .iter()
            .map(|phase| format!("{phase:?} => {phase}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    assert_snapshot!("enum_representations", rendered);
}

#[test]
fn snapshots_explanations_and_suggestions() {
    let selected = [
        ErrorCode::E0001,
        ErrorCode::E0002,
        ErrorCode::E0003,
        ErrorCode::E0004,
        ErrorCode::E0005,
        ErrorCode::E0008,
        ErrorCode::E0009,
        ErrorCode::E0010,
        ErrorCode::E1001,
        ErrorCode::E1002,
        ErrorCode::E1003,
        ErrorCode::E2023,
        ErrorCode::E2024,
        ErrorCode::E2027,
        ErrorCode::E2028,
        ErrorCode::E2009,
        ErrorCode::E2010,
        ErrorCode::E5005,
    ];

    let rendered = selected
        .iter()
        .map(|error| {
            format!(
                "{code}\nexplanation={explanation:?}\nsuggestions={suggestions:?}",
                code = error.code(),
                explanation = error.explanation(),
                suggestions = error.suggestions(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("explanations_and_suggestions", rendered);
}

#[test]
fn snapshots_boundary_phase_and_severity_behavior() {
    let values = [
        ErrorCode::E0010,
        ErrorCode::E1001,
        ErrorCode::E1015,
        ErrorCode::E2001,
        ErrorCode::E2032,
        ErrorCode::E3001,
        ErrorCode::E3008,
        ErrorCode::E4001,
        ErrorCode::E4005,
        ErrorCode::E5001,
        ErrorCode::E5005,
        ErrorCode::E1013,
    ];

    let rendered = values
        .iter()
        .map(|error| {
            format!(
                "{code}: numeric={numeric}, phase={phase}, severity={severity}",
                code = error.code(),
                numeric = error.numeric_code(),
                phase = error.phase(),
                severity = error.severity(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("boundary_phase_and_severity", rendered);
}
