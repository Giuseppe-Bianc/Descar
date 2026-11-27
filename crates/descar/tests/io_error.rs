//! End-to-end test for the text of I/O errors printed by the `descar` binary.

use std::process::Command;

#[test]
fn missing_file_error_keeps_quotes_and_backslashes() {
    let output = Command::new(env!("CARGO_BIN_EXE_descar"))
        .args(["check", r"no\such.dr"])
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE")
        .output()
        .expect("the descar binary should start");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains(r"failed to read 'no\such.dr'"), "stderr was: {stderr}");
}
