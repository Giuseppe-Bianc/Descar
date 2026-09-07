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

## License

Descar is licensed under the Apache License 2.0.

See [`LICENSE`](./LICENSE) for the complete license text.
