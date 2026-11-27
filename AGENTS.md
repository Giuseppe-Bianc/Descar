# Repository Guidelines

## Project Structure and Architecture

Descar is a Rust 2024 workspace. `crates/descar-core` implements lexing (`src/lex`), parsing (`src/syntax`), type checking (`src/semantic`), diagnostics, and output. `crates/descar-cli` defines the command-line interface; `crates/descar` is the executable entry point. Each crate stores integration tests in `tests/` and Insta snapshots in `tests/snapshots/`; sample Descar programs are in `dr_files/`.

The root `Cargo.toml` defines workspace settings. See `CLAUDE.md` for an architecture overview and `CONTRIBUTING.md` for the full contribution process.

## Build, Test, and Run

- `cargo build --workspace` builds all crates.
- `cargo check --workspace --all-features` checks all crates without producing executables.
- `cargo test --workspace --all-features` runs the full test suite.
- `cargo test <test_name>` runs tests whose names match the filter; for example, `cargo test -p descar-core --test parser_tests` runs the core parser integration tests.
- `cargo fmt --all -- --check` checks formatting; apply it with `cargo fmt --all`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` runs lints and treats warnings as errors.
- `cargo run -- compile dr_files/simple_test.dr` compiles an example; `cargo run -- check dr_files/simple_test.dr` checks it without compilation output.

## Coding Style

Follow idiomatic Rust naming: `snake_case` for modules, functions, and variables; `UpperCamelCase` for types and traits; and `SCREAMING_SNAKE_CASE` for constants. Use the existing module boundaries and error/reporting types when extending compiler behavior. `rustfmt.toml` sets a 120-column maximum and Rust 2024 edition; run the formatting check before submitting.

## Testing Guidelines

Add regression tests in the responsible crate's `tests/` directory with descriptive names such as `parser_tests.rs`. Start with a focused test, then run the workspace suite. Some tests use Insta snapshots in `tests/snapshots/`; when output changes intentionally, update and review the affected snapshots. Exclude unrelated snapshot changes.

## Continuous Integration

GitHub Actions checks formatting, compilation, tests, and Clippy on Linux, macOS, and Windows with stable, beta, and nightly Rust. Keep platform-sensitive behavior portable and review CI results on your pull request.

## Commits and Pull Requests

Use Conventional Commits in `<type>(<scope>): <description>` format, for example `fix(core): handle empty input`. See `.github/copilot-commit-message-instructions.md`; common scopes include `core`, `cli`, `build`, and `test`.

Keep pull requests focused and open them against `main`. Complete `.github/pull_request_template.md`, explain what changed and why, link related issues, and include tests for behavior changes. Before submitting, run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features`.

## Security

Do not report vulnerabilities in public issues or pull requests. Follow `SECURITY.md` for the project's private reporting instructions. Keep credentials and other secrets out of source files, examples, and commits.
