# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common commands

- Build: `cargo build --workspace`
- Check formatting: `cargo fmt --all -- --check`
- Lint: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Test all: `cargo test --workspace --all-features`
- Run single test: `cargo test <test_name>`
- Run compile: `cargo run -- compile <file>.dr [options]`
- Run check: `cargo run -- check <file>.dr [options]`
- Verbosity: add `-v` (repeat) or `-q` to quiet

Compile options:

- `-O <none|basic|aggressive>` (default none)
- `--output <FILE>`
- `--emit-ir`
- `--diagnostics`

## Architecture

Workspace three crates:

- `descar-core`: core compiler infrastructure, location handling (`SourceId`, `SourceLocation`, `Span`).
- `descar-cli`: CLI definitions using Clap, custom help styles, logging args.
- `descar`: binary entry point, wires CLI to `main`, uses `descar_cli::cli`.

Core crate pure Rust, no external deps. CLI crate depends on `clap` and `descar-core`. Binary crate depends on `descar-cli`.

## CI

GitHub workflow `rust.yml` builds on Linux/macOS/Windows, runs fmt check, `cargo check`, tests, and clippy with warnings as errors.

## Commit style

Follow Conventional Commits. See `.github/copilot-commit-message-instructions.md` for details. Use scopes like `core`, `cli`, `build`, `test`.

## Import other configs

Run `/import` to scan for MCP servers, slash commands, skills, instructions. Then `/import --yes=<digest>` to apply.
