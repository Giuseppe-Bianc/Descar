# Descar

[![Rust CI](https://github.com/Giuseppe-Bianc/Descar/actions/workflows/rust.yml/badge.svg)](https://github.com/Giuseppe-Bianc/Descar/actions/workflows/rust.yml)

## Description

Modern compiler for the Descar language. Converts `.dr` files into tokens, offers colorized output and detailed diagnostics, can emit intermediate representation (IR), supports three optimization levels (`none`, `basic`, `aggressive`), and provides CLI commands for compilation (`cargo run -- compile`) and syntax checking (`cargo run -- check`).

## Main features

- Compilation of `.dr` files
- Syntax checking without generating output (`check`)
- Optimization levels: `none`, `basic`, `aggressive`
- Extended diagnostics (`--diagnostics`)
- Verbosity control (`-v`, `-vv`, `-vvv`) and silent mode (`-q`)
- Colorized output for tokens and spans

## Prerequisites

- Rust toolchain (stable, beta, or nightly) with `cargo`
- `rustup` to manage toolchains

## Installation

```sh
git clone https://github.com/Giuseppe-Bianc/Descar.git
cd Descar
cargo build --workspace          # compila tutti i crate
# oppure installa il binario
cargo install --path crates/descar
```

Alternatively, run directly without installing:

```sh
cargo run -- <comando>
```

## Basic usage

```sh
cargo run -- compile dr_files/simple_test.dr -v
```

Compiles and prints tokens. Options:

- `-O <none|basic|aggressive>` select optimization level
- `--diagnostics` enables detailed statistics
- `-q` silences non-essential output

```sh
cargo run -- check dr_files/simple_test.dr -v
```

Verifies syntax without producing output.

## Language syntax

Explicit casts use the Rust-style `expression as type` form. Targets may be numeric types, `char`, `bool`, `string`, array types such as `i32[4]`, vector types such as `vector<i32>`, or a custom type identifier. The supported conversions are numeric-to-numeric, numeric-to-`char` or `bool`, `char` or `bool` to numeric, `char` to `string`, `nullptr` to arrays, vectors, or custom types, and identity casts. Arbitrary string conversions, unrelated aggregate conversions, and pointer reinterpretation are rejected.

```text
main {
    var wide: i64 = 300
    var small: i8 = wide as i8
    var precise: f64 = 16777217i32 as f64
    var character: char = 65i32 as char
    var text: string = character as string
    var items: vector<i32> = nullptr as vector<i32>
    var record: Record = nullptr as Record
    var same_text: string = text as string
}
```

Casts are explicit. They do not change the rules for implicit numeric conversions. A cast to an array type includes the length, for example `nullptr as i32[4]`. Runtime behavior for narrowing, floating-point conversions, numeric-to-character or Boolean conversions, and null-to-aggregate casts still requires an execution-level definition.

## Practical examples

### Language examples

- `dr_files/simple_test.dr` – basic hello‑world style program.
- `dr_files/input.dr` – demonstrates input handling.
- `dr_files/bitwise_type_mismatch.dr` – type mismatch error for bitwise ops.
- `dr_files/return_missing_value.dr` – missing return value error.

- Compilation with basic optimization:

  ```sh
  cargo run -- compile dr_files/float_test.dr -O basic
  ```

- Silent-mode check:

  ```sh
  cargo run -- check dr_files/return_missing_value.dr -q
  ```

## Repository structure

- `crates/descar-core` → compiler core (tokens, error handling, file utilities)
- `crates/descar-cli` → CLI wrapper based on **clap**
- `crates/descar` → binary entry point linking CLI to core
- `dr_files/` → example `.dr` files used in tests
- `.github/workflows/rust.yml` → multi‑OS CI on GitHub Actions
- `Cargo.toml`, `Cargo.lock` → Cargo workspace configuration

## Test, build, CI

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

CI runs on Linux, macOS, and Windows for stable, beta, and nightly toolchains.

## Troubleshooting

| Symptom | Possible cause | Action |
|---|---|---|
| `unsupported source file extension` | file does not end with `.dr` | rename file with correct extension |
| `failed to read source file` | permissions or incorrect path | check path and read permissions |
| `error: aborting due to previous error` | lint failed | run `cargo fmt --check` and `cargo clippy` |
| `cargo test` fails | failing tests | inspect files in `dr_files/` and fix syntax |

## Additional Documentation

- [`CLAUDE.md`](./CLAUDE.md) contains development guidance, common Cargo commands, workspace architecture, CI details, and project contribution conventions.
- [`SECURITY.md`](./SECURITY.md) describes the vulnerability reporting process, supported versions, security incident handling, and security practices.
- [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md) defines the project's community standards and expected behavior.
- [`.github/ISSUE_TEMPLATE/`](./.github/ISSUE_TEMPLATE/) contains templates for bug reports and feature requests.
- [`.github/pull_request_template.md`](./.github/pull_request_template.md) provides the pull request template used by the repository.

## License

Descar is licensed under the Apache License 2.0.

See [`LICENSE`](./LICENSE) for the complete license text.
