<!--
Sync Impact Report
Version change: 1.0.0 → 1.1.0
Modified Principles:
  - "Stack Tecnologico e Build" → expanded into Section 1 (Stack Tecnologico e Build)
  - "Architettura" → expanded into Section 2 (Architettura)
  - "Principi di Sviluppo Rust" → expanded into Section 3 (Principi di Sviluppo Rust)
  - "Formattazione, Linting e Testing" → split into Section 4 (Formattazione e Linting) and Section 5 (Testing)
  - "Governance, CI e Dipendenze" → expanded and split into Section 15 (CI) and Section 16 (Dipendenze)
Added Sections:
  - Section 6: Completezza della Strategia di Testing
  - Section 7: Testing delle Fasi del Compilatore
  - Section 8: Test-Driven Development
  - Section 9: Bug Fixing
  - Section 10: Test Deterministici e Isolamento
  - Section 11: CLI Testing
  - Section 12: Error Handling
  - Section 13: Snapshot Testing
  - Section 14: Regressioni
  - Section 17: Manutenibilità
  - Section 18: Verificabilità dei Requisiti
  - Section 19: Criteri di Completamento
  - Section 20: Regola di Precedenza
  - Section 21: Conformità ASD-STE100
Removed Sections: none
Follow-up TODOs: none
-->
# Descar Constitution

## Core Principles

### 1. Stack Tecnologico e Build

The primary language of the project is Rust.

Use Cargo to manage the project, dependencies, compilation, test execution, and development
activities.

The project uses a Cargo workspace with the following crates:

- `crates/descar-core`: core compiler functions and shared structures for source management,
  positions, and spans.
- `crates/descar-cli`: definition and management of the command-line interface.
- `crates/descar`: the binary entry point and the link between the CLI and the compiler.

The Rust version declared by the workspace MUST be respected. Do not lower it without a documented
technical reason.

Dependencies MUST be managed through Cargo and MUST respect the versions and configurations
defined by the workspace.

Do not introduce Maven, Gradle, Java, JUnit, or tools from ecosystems other than Rust, unless an
explicit and documented requirement makes it necessary.

### 2. Architettura

The project architecture MUST maintain a clear separation of responsibilities between the crates
of the workspace.

`descar-core` MUST remain the layer responsible for core compiler functions. It MUST NOT have
unnecessary dependencies on user interface or CLI-specific components.

`descar-cli` MUST be responsible for the definition of the command-line interface, its arguments,
options, and operational modes.

`descar` MUST act primarily as the entry point and as the composition of components. It MUST NOT
concentrate domain logic or compiler logic directly in the binary crate.

Dependencies between crates MUST follow a direction consistent with this separation of
responsibilities. Circular couplings or dependencies toward higher architectural levels are
prohibited.

### 3. Principi di Sviluppo Rust

Code MUST follow idiomatic Rust conventions and established best practices of the Rust ecosystem.

Code MUST prefer:

- correct ownership and borrowing;
- explicit and appropriate handling of `Result` and `Option`;
- idiomatic error handling;
- simple and consistent APIs;
- strongly expressive types;
- absence of unnecessary duplication;
- separation of responsibilities;
- readability and maintainability;
- absence of `unsafe` unless a concrete technical need exists and is documented.

Prefer idiomatic and simple solutions over excessively complex implementations.

Do not introduce abstractions, traits, wrappers, macros, or layers of indirection that lack a
concrete technical justification.

Changes MUST maintain compatibility with the conventions and architecture already present in the
repository, unless the requirement explicitly calls for an architectural evolution.

### 4. Formattazione e Linting

Rust code MUST be formatted with `rustfmt`.

Formatting MUST be verifiable with:

```
cargo fmt --all -- --check
```

The project uses Clippy as the static analysis tool.

Code MUST be compatible with:

```
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Warnings MUST NOT be suppressed arbitrarily to obtain a green build.

Exceptions to specific lints MUST be justified by a concrete technical need and MUST be as
narrow as possible.

### 5. Testing

Use the native Rust testing framework via `cargo test`.

When appropriate, also use the snapshot testing tools already present in the project, including
`insta`, following the features and conventions supported by the version declared in the workspace.

Tests MUST follow idiomatic Rust ecosystem conventions and MUST be:

- deterministic;
- reproducible;
- isolated;
- readable;
- specific;
- easy to diagnose when they fail;
- independent of execution order;
- free of accidental dependencies on the local environment.

Do not introduce tests solely to artificially increase coverage. Every test MUST verify
significant system behavior.

## Constraints

- No Maven, Gradle, Java, JUnit, or other non-Rust tools.
- The Rust edition MUST respect the version declared in the workspace.
- Dependencies MUST respect the versions defined in Cargo.toml.
- Avoid unnecessary dependencies; prefer the standard library.
- Use only crates approved after a security evaluation.

## Development Workflow

### 6. Completezza della Strategia di Testing

The test suite MUST provide adequate coverage of the expected behavior of the compiler and its
components.

The testing strategy MUST include at least the following categories.

**Ordinary cases.** Verify the main valid use cases that represent normal operating conditions.
For the compiler, verify when applicable:

- parsing of valid programs;
- correct tokenization;
- handling of supported syntactic structures;
- compilation of valid programs;
- handling of valid CLI options;
- behavior of optimization levels;
- IR production when requested;
- diagnostics requested by the user;
- `compile` mode;
- `check` mode.

**Corner cases.** Verify particular cases and combinations of conditions that, while within the
supported behavior of the system, can produce errors that are difficult to find. Consider when
applicable:

- uncommon combinations of constructs;
- interactions between different compiler features;
- combinations of CLI options;
- combinations of diagnostics, verbosity, and silent mode;
- valid but structurally complex programs;
- inputs that pass through multiple compiler phases;
- cases that can reveal problems in error propagation or position information.

**Edge cases.** Verify the boundaries of the supported domain. Test when applicable:

- minimum values;
- maximum values;
- values immediately before and after the limits;
- empty inputs;
- inputs composed of a single element;
- very small files;
- identifiers or tokens at the limits of the syntactic rules;
- limit lengths;
- limit depths;
- limit combinations of options;
- inclusive and exclusive boundaries.

Also test invalid inputs and error conditions defined by the specification.

### 7. Testing delle Fasi del Compilatore

When the architecture allows it, distribute tests in a way that is consistent with the
different responsibilities of the compiler.

Test individually, when exposed as verifiable components, the functions related to:

- input handling;
- tokenization;
- position and span representation;
- parsing;
- error handling;
- diagnostics;
- intermediate representations;
- optimization;
- compilation;
- CLI interface.

Integration tests MUST also be present when correct behavior depends on the interaction between
multiple components.

For the main flows, prefer tests that verify the observable behavior of the entire pipeline when
an isolated unit test is not sufficient to guarantee the correct functioning of the pipeline.

### 8. Test-Driven Development

Every new feature, significant behavioral change, and bug fix MUST be developed using
Test-Driven Development, when technically applicable.

The process MUST systematically follow the Red → Green → Refactor cycle.

**Red.** Define a test that represents the desired behavior before the implementation. The test
MUST initially fail or MUST reliably reproduce the existing bug. The failure MUST be attributable
to the absence of the required behavior or to the presence of the incorrect behavior.

**Green.** Implement the minimum amount of code necessary to satisfy the test. Do not introduce
features not required by the test or the current requirement.

**Refactor.** After the test passes, refactor the code to improve its structure, clarity,
simplicity, and maintainability. The refactoring MUST NOT modify the verified behavior. After
refactoring, run all relevant tests again. Repeat the cycle until the requirement is complete.

### 9. Bug Fixing

Every bug that can be corrected with a test MUST be converted into a regression test.

The test MUST reproduce the incorrect behavior before the correction.

Implement the correction afterward.

The test MUST demonstrate that the incorrect behavior no longer occurs.

The regression test MUST be kept permanently in the suite, unless a documented technical reason
exists for its removal.

### 10. Test Deterministici e Isolamento

Tests MUST be independent of execution order.

Tests MUST NOT share mutable global state unless that sharing is explicitly the subject of the
test.

Avoid:

- temporal dependencies;
- dependencies on uncontrolled timezone or locale;
- dependencies on non-isolated filesystem;
- network dependencies;
- uncontrolled random values;
- persistent global state;
- assumptions about the developer's local environment.

When a test requires filesystem, processes, or other external resources, those resources MUST be
isolated and cleaned up correctly.

### 11. CLI Testing

Because the project exposes a CLI, tests MUST verify the observable behavior of the main
invocation modes.

Verify when applicable:

- `compile` command;
- `check` command;
- `-O` option;
- `none`, `basic`, and `aggressive` levels;
- `--output`;
- `--emit-ir`;
- `--diagnostics`;
- `-v`, `-vv`, `-vvv`;
- `-q`;
- valid inputs;
- invalid inputs;
- non-existent files;
- unsupported extensions;
- incompatible argument combinations;
- exit codes;
- error and diagnostic messages.

CLI tests MUST verify public behavior without depending unnecessarily on the internal
implementation.

### 12. Error Handling

Errors MUST be treated as part of the public behavior of the compiler.

Tests MUST verify not only that an error is detected, but also, when defined by the
specification:

- the error type is correct;
- the error location is identified correctly;
- the spans are consistent;
- the message is appropriate;
- the diagnostics contain the necessary information;
- the process terminates with the expected behavior.

Errors MUST be represented using idiomatic Rust structures. Use of `panic!` is prohibited for
conditions that belong to the normal domain of application errors.

### 13. Snapshot Testing

When snapshot testing with `insta` is appropriate, snapshots MUST represent significant and
stable system outputs.

Snapshots MUST NOT be updated automatically to mask a regression.

Every change to a snapshot MUST be intentional and MUST correspond to a verified change in the
expected behavior.

Use redactions when the output contains variable values that are not part of the behavior that
the test must verify.

### 14. Regressioni

Every change MUST preserve the already-verified behavior, unless the requirement explicitly
calls for a change to that behavior.

Before a change can be considered complete, run at minimum:

```bash
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

When a change affects a specific part of the compiler, also run the tests directly relevant to
the modified component.

### 15. CI

Continuous Integration MUST be treated as an integral part of project quality.

Changes MUST remain compatible with the existing CI pipeline, which verifies the project on
multiple operating systems and multiple Rust toolchains.

A change MUST NOT be considered complete if it introduces compilation errors, failing tests,
formatting problems, or Clippy warnings that become errors in the pipeline.

### 16. Dipendenze

Introduce new dependencies only when a concrete technical need exists.

Before adding a dependency, evaluate whether the requirement can be satisfied using:

- standard library features;
- features already present in the workspace;
- abstractions already existing in the project.

Dependencies MUST be consistent with the Cargo workspace.

Do not introduce duplicate or functionally overlapping dependencies without a technical
justification.

### 17. Manutenibilità

Code MUST be maintainable by a developer who knows Rust but does not necessarily know the
specific implementation of the modified component.

Changes MUST be localized and proportional to the requirement.

Avoid extensive refactoring that is not necessary to implement a feature.

When an architectural change is necessary, it MUST be accompanied by a technical justification
and by sufficient tests to demonstrate that the previous behavior is not accidentally broken.

### 18. Verificabilità dei Requisiti

Every significant functional requirement MUST be associated with a concrete verification
strategy.

A requirement MUST NOT be considered implemented only because the code compiles.

The verification MUST demonstrate the expected behavior through appropriate tests.

The test suite MUST provide a clear distinction between:

- correct behavior;
- expected invalid behavior;
- unexpected errors;
- boundary conditions;
- regressions.

### 19. Criteri di Completamento

A feature, change, or correction can be considered complete only when all of the following are
true:

1. The required behavior is defined in verifiable terms.
2. The relevant tests are created or updated.
3. When technically applicable, the TDD Red → Green → Refactor cycle is followed.
4. Ordinary cases are covered.
5. Relevant corner cases are covered.
6. Relevant edge cases are covered.
7. Relevant invalid inputs are verified.
8. Necessary regression tests are present.
9. `cargo test --workspace --all-features` succeeds.
10. `cargo fmt --all -- --check` succeeds.
11. `cargo clippy --workspace --all-targets --all-features -- -D warnings` succeeds.
12. The change remains compatible with the workspace architecture.
13. No unnecessary dependencies or complexity are introduced.

### 20. Regola di Precedenza

When a conflict exists between a new request and the conventions already established by the
repository, apply the following order of priority:

1. Explicit project requirements.
2. Behavior and architecture already defined in the repository.
3. Official Rust and Cargo conventions.
4. Official documentation of the dependencies in use.
5. Established best practices of the Rust ecosystem.

Deviations MUST be justified by a concrete technical requirement.

### 21. Conformità ASD-STE100

All technical documents MUST be written in conformance with the ASD-STE100 standard (Simplified
Technical English).

This requirement applies to the creation, review, and update of all documentation subject to
this rule.

The language used in documents MUST follow the rules, conventions, and restrictions defined by
the applicable standard.

The purpose of ASD-STE100 adoption is to guarantee terminological uniformity, clarity of
exposition, comprehensibility of technical information, and reduction of interpretive ambiguity.

Conformance MUST be maintained throughout the entire lifecycle of the document and MUST be
verified during document control and review activities.

## Governance

- The Constitution supersedes all other project guidelines.
- Amendments require documentation, approval, and a migration plan.
- All PRs MUST verify conformance to each principle before merge.
- Use this document as the authoritative source for project decisions.
- Version increments follow Semantic Versioning:
  - MAJOR: backward-incompatible governance or principle removal or redefinition.
  - MINOR: new principle or section added or materially expanded guidance.
  - PATCH: clarifications, wording, typo fixes, non-semantic refinements.

**Version**: 1.1.0 | **Ratified**: 2026-09-14 | **Last Amended**: 2026-10-04
