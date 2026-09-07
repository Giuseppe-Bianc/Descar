# Contributing to Descar

Thank you for taking the time to contribute to Descar.

Descar is a modern compiler for the Descar language, implemented as a Rust workspace. Contributions are welcome when they improve the compiler, CLI, diagnostics, tests, documentation, or developer experience.

Please read this document before opening an issue or pull request. It explains how to prepare your development environment, how to validate changes, and what is expected from contributions.

## Table of Contents

* [Code of Conduct](#code-of-conduct)
* [Before You Contribute](#before-you-contribute)
* [Development Environment](#development-environment)
* [Repository Structure](#repository-structure)
* [Building the Project](#building-the-project)
* [Testing](#testing)
* [Formatting and Linting](#formatting-and-linting)
* [Running Descar](#running-descar)
* [Reporting Bugs](#reporting-bugs)
* [Suggesting Enhancements](#suggesting-enhancements)
* [Your First Code Contribution](#your-first-code-contribution)
* [Documentation Contributions](#documentation-contributions)
* [Pull Requests](#pull-requests)
* [Commit Messages](#commit-messages)
* [Security Issues](#security-issues)
* [License](#license)

## Code of Conduct

All contributors are expected to follow the project's [Code of Conduct](./CODE_OF_CONDUCT.md).

Contributions should remain constructive and focused on improving the project. Disrespectful, discriminatory, or abusive behavior is not acceptable.

If you encounter behavior that violates the Code of Conduct, follow the reporting procedure described in `CODE_OF_CONDUCT.md`.

## Before You Contribute

Before starting significant work:

1. Read the [README](./README.md).
2. Read [`CLAUDE.md`](./CLAUDE.md) for the project's development commands and architecture.
3. Search existing [issues](https://github.com/Giuseppe-Bianc/Descar/issues) to determine whether the problem or proposal has already been discussed.
4. For substantial changes, open or comment on an issue before implementing the change.
5. Check existing pull requests to avoid duplicating ongoing work.
6. Make sure your proposed contribution fits the scope of Descar.

For small documentation fixes or clearly isolated bug fixes, opening an issue first is optional.

## Development Environment

Descar requires:

* Rust with Cargo
* `rustup` for managing Rust toolchains
* A stable, beta, or nightly Rust toolchain

Clone the repository and enter its directory:

```sh
git clone https://github.com/Giuseppe-Bianc/Descar.git
cd Descar
```

Build the complete workspace:

```sh
cargo build --workspace
```

The project is organized as a Cargo workspace containing the compiler core, CLI, and binary crates.

## Repository Structure

The main components are:

```text
crates/
├── descar-core/    # Compiler core and infrastructure
├── descar-cli/     # Command-line interface
└── descar/         # Binary entry point

dr_files/           # Descar language examples and test inputs

.github/
├── ISSUE_TEMPLATE/ # Issue templates
├── pull_request_template.md
└── workflows/
    └── rust.yml    # CI workflow
```

The compiler architecture is documented in [`CLAUDE.md`](./CLAUDE.md).

## Building the Project

Build all workspace crates:

```sh
cargo build --workspace
```

Build in release mode:

```sh
cargo build --workspace --release
```

Install the Descar binary locally:

```sh
cargo install --path crates/descar
```

## Testing

Run the complete test suite:

```sh
cargo test --workspace --all-features
```

Run a specific test:

```sh
cargo test <test_name>
```

When changing compiler behavior, add or update tests that demonstrate the expected behavior.

Changes affecting parsing, diagnostics, type checking, compilation, or optimization should include appropriate regression coverage.

## Formatting and Linting

Before submitting a pull request, run:

```sh
cargo fmt --all -- --check
```

Run Clippy with warnings treated as errors:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

A contribution should not introduce formatting errors, compiler warnings, or new Clippy warnings.

If formatting changes are required, apply them with:

```sh
cargo fmt --all
```

Then run the validation commands again.

## Running Descar

Compile a Descar source file:

```sh
cargo run -- compile dr_files/simple_test.dr
```

Compile with verbose output:

```sh
cargo run -- compile dr_files/simple_test.dr -v
```

Select an optimization level:

```sh
cargo run -- compile dr_files/float_test.dr -O basic
```

The available optimization levels are:

```text
none
basic
aggressive
```

Run syntax checking without producing compilation output:

```sh
cargo run -- check dr_files/simple_test.dr
```

Enable diagnostics:

```sh
cargo run -- compile dr_files/simple_test.dr --diagnostics
```

Use `-v`, `-vv`, or `-vvv` for increased verbosity, or `-q` for quiet mode.

## Reporting Bugs

Before reporting a bug:

1. Make sure you are working with the latest version of the repository.
2. Read the README and available documentation.
3. Search existing issues for the same or a similar problem.
4. Confirm that the behavior can be reproduced.
5. Collect the relevant Rust version, operating system, input file, command, output, and error message.
6. Reduce the example to the smallest reproducible case when possible.

Bug reports should contain enough information for another contributor to reproduce the problem.

Include:

* A clear description of the expected behavior.
* A clear description of the actual behavior.
* Steps to reproduce the problem.
* The relevant `.dr` source file or a minimal reproduction.
* The command used to run Descar.
* The operating system.
* The Rust toolchain version.
* Relevant logs or compiler output.

Open a bug report through the repository's [issue tracker](https://github.com/Giuseppe-Bianc/Descar/issues).

Do not report security vulnerabilities through public issues. Follow the [Security Policy](./SECURITY.md) instead.

## Suggesting Enhancements

Enhancement proposals are welcome when they have a clear relationship to Descar.

Before opening an enhancement issue:

1. Search existing issues and pull requests.
2. Read the README and development documentation.
3. Verify that the proposed functionality is not already available.
4. Consider whether the proposal fits the compiler's architecture and current scope.
5. Explain the problem the proposed change would solve.

A useful enhancement proposal should describe:

* The problem or limitation.
* The proposed behavior.
* Why the change is useful.
* Possible alternatives.
* Examples of the expected CLI or language behavior, where applicable.
* Any compatibility or architectural implications.

Use the repository's [issue tracker](https://github.com/Giuseppe-Bianc/Descar/issues) to discuss proposals.

## Your First Code Contribution

Good starting contributions include:

* Fixing reproducible compiler bugs.
* Adding regression tests.
* Improving compiler diagnostics.
* Improving CLI behavior.
* Improving documentation.
* Adding or improving `.dr` examples.
* Fixing build, formatting, or CI problems.

For a contribution that changes compiler architecture or introduces a substantial language feature, discuss the proposal in an issue before implementation.

### Typical workflow

1. Fork the repository.
2. Clone your fork locally.
3. Create a focused branch from `main`.
4. Make the change.
5. Add or update tests.
6. Run formatting.
7. Run Clippy.
8. Run the complete test suite.
9. Review your changes.
10. Push the branch to your fork.
11. Open a pull request against `main`.

Example:

```sh
git checkout -b fix/compiler-diagnostic
```

Keep each branch focused on one logical change.

Avoid mixing unrelated refactoring, formatting changes, feature work, and bug fixes in the same pull request.

## Documentation Contributions

Documentation changes are treated as valid contributions.

Useful documentation improvements include:

* Fixing inaccurate instructions.
* Improving installation instructions.
* Adding CLI examples.
* Documenting compiler behavior.
* Improving error explanations.
* Adding examples of the Descar language.
* Improving contributor documentation.

When documentation describes behavior that can change, verify the relevant commands against the current implementation.

The main documentation entry point is [`README.md`](./README.md).

## Pull Requests

Before opening a pull request, make sure the change has been tested locally.

The repository provides a [pull request template](./.github/pull_request_template.md). Use it when opening a PR and provide the requested information.

A pull request should:

* Have a clear title.
* Explain what changed.
* Explain why the change is needed.
* Reference the relevant issue when applicable.
* Include tests for behavior changes.
* Include documentation updates when necessary.
* Pass formatting checks.
* Pass Clippy.
* Pass the test suite.
* Avoid unrelated changes.

Use a focused pull request. Large changes that combine unrelated concerns are harder to review and are more likely to be rejected or require significant restructuring.

Maintainers may request changes before a pull request can be merged.

## Commit Messages

Descar follows [Conventional Commits](https://www.conventionalcommits.org/).

Use a commit format such as:

```text
<type>(<scope>): <description>
```

Examples:

```text
feat(core): add source span validation
fix(cli): improve invalid argument diagnostics
test(core): add parser regression tests
docs: update contribution guidelines
build: update workspace configuration
```

Relevant scopes include:

* `core`
* `cli`
* `build`
* `test`

Keep commit messages concise and describe the purpose of the change.

## Security Issues

Do not disclose security vulnerabilities through public GitHub issues or pull requests.

Follow [`SECURITY.md`](./SECURITY.md) for the project's security reporting process.

Security reports should be sent to the address specified in the security policy.

## Validation Checklist

Before submitting a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

If your change affects the build or compilation path, also run:

```sh
cargo build --workspace
```

Confirm that:

* [ ] The change is focused and necessary.
* [ ] Tests were added or updated where appropriate.
* [ ] Formatting passes.
* [ ] Clippy passes without warnings.
* [ ] All workspace tests pass.
* [ ] Documentation was updated when required.
* [ ] The pull request template is complete.
* [ ] No unrelated files were modified.

## License

Descar is licensed under the [Apache License 2.0](./LICENSE).

By submitting a contribution, you agree that your contribution can be distributed under the project's license and that you have the necessary rights to submit the contributed material.
