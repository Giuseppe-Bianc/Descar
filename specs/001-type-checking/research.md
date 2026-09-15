# Research & Architectural Decisions: Semantic Type Checking

**Feature**: `001-type-checking` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

This document consolidates architectural investigations, technical decisions, rationales, and rejected alternatives for the Descar semantic type checker.

---

## 1. Module Organization & Phase-Driven Pipeline

### Decision

Organize the semantic type checker as a dedicated library module inside `crates/descar-core/src/semantic/` (and exported through `descar_core::semantic`), structured strictly by **compilation phase** rather than by AST data type:

- `ctx.rs`: `CheckerCtx` and `Phase` definition.
- `binder.rs`: Symbol collection and scope tree construction (Phase 1).
- `resolve.rs`: Identifier and type reference resolution (Phase 2).
- `sig.rs`: Signature evaluation and recursive type cycle detection (Phase 3).
- `check.rs`: Body, expression, and statement type checking with `Expectation` (Phase 4).
- `globals.rs`: Whole-program checks (unused variables/imports, visibility) (Phase 5).
- `diag.rs`: Cross-cutting diagnostic service with explicit `phase` attribution.
- `mod.rs`: Top-level orchestrator exposing `check_program`.

Each phase provides a public entry function and distinct internal error types that map into the unified `Diagnostic` structure.

### Rationale

- **Separation of Concerns**: Each phase has a single responsibility and unambiguous preconditions.
- **Isolated Unit Testability**: Early phases (`binder`, `resolve`, `sig`) can be thoroughly tested with hand-crafted synthetic ASTs without running previous or subsequent phases and without invoking the real parser.
- **Maintainability & Extensibility**: As required by FR-008, adding new statements or expressions touches localized logic in `check.rs` (and corresponding signature calculation in `sig.rs`) without disturbing the scope tree or symbol tables.

### Alternatives Considered

- **Monolithic AST Visitor (`Visitor` pattern covering all tasks in one pass)**:
    - *Rejected*: Makes forward references impossible without complex ad-hoc back-patching. Fails to separate scope creation from type resolution and makes early-phase unit testing impossible without parsing whole programs.
- **Separate Crate (`crates/descar-semantic`)**:
    - *Rejected*: The project constitution establishes three workspace crates (`descar-core`, `descar-cli`, `descar`). AST definitions, tokens, and errors already reside in `descar-core`. Adding a 4th crate would violate the constitution's 3-crate layout without technical necessity.

---

## 2. State Management & Linear Execution Flow (`CheckerCtx`)

### Decision

All state across the type-checking pipeline is encapsulated in a single `CheckerCtx` structure containing:

1. `SymbolTable` (scope tree, symbol records, declaration metadata).
2. `TypeTable` (interned types, type definitions, size/layout rules).
3. `Vec<Diagnostic>` (accumulated warnings and errors).

Each phase takes `&mut CheckerCtx` along with its phase-specific input, and writes its outputs directly into the context or returns them linearly. Interior mutability (`Rc<RefCell<_>>` or `Arc<Mutex<_>>`) is explicitly forbidden.

### Rationale

- **Idiomatic Rust Ownership**: The compilation pipeline is strictly sequential (`binder` -> `resolve` -> `sig` -> `check` -> `globals`). A mutable reference passed sequentially ensures compile-time borrow safety without runtime overhead or risk of `BorrowMutError` panics.
- **Linear Debuggability**: State transformations can be inspected step-by-step between phases without chasing hidden pointer aliases or shared reference cycles.

### Alternatives Considered

- **`Rc<RefCell<Scope>>` Parent/Child Pointers**:
    - *Rejected*: Adds runtime reference-counting and cell-borrowing overhead, complicates lifetime management, and introduces memory leaks if cycle cleanups fail. An arena/index-based `Vec<Scope>` with `ScopeId(usize)` parent indices is safer, cache-friendly, and completely free of reference counting.
- **Pure Functional Pipeline (Passing immutable contexts and returning new states)**:
    - *Rejected*: Imposes excessive allocations and deep copies of symbol and type tables on every phase transition.

---

## 3. Libraries & Dependencies

### Decision

- **Core Compiler (`descar-core`)**: Standard library (`std`) only. Use standard `std::collections::HashMap`, `Vec`, and slice references.
- **Snapshot Testing (`dev-dependencies`)**: `insta = "1.48.0"` (already declared in the root workspace `Cargo.toml`).
- **Error Derivations**: Manual `Display` and `std::error::Error` implementations for phase-specific error enums; do **not** add `thiserror` to new semantic modules.

### Rationale

- **Zero-Dependency Core**: With a nominal, concrete type system lacking generics and type variables, `HashMap` and indexed `Vec` lookups provide predictable $O(1)$ performance.
- **Premature String Interning Avoided**: Identifiers in Descar's AST are already stored as `String` or `Arc<str>`. An interning crate (such as `lasso` or `string-interner`) introduces unsafe blocks or external dependency weight without proven profiling evidence that string comparisons are a compilation bottleneck.
- **Why `thiserror` was Evaluated and Rejected for Semantic Modules**:
  Each phase defines only 2–5 distinct error variants (e.g. `DuplicateDeclaration`, `UndeclaredIdentifier`, `InfiniteSizeRecursiveType`, `TypeMismatch`). Hand-writing `Display` requires fewer than 40 lines per phase, avoiding macro expansion overhead and dependency churn. Can be re-evaluated if variant count exceeds 15.

### Alternatives Considered

- **String Interning (`lasso`, `string-interner`)**:
    - *Rejected*: Spec SC-003 requires type-checking overhead to stay below 15% for 1k–10k LOC. Standard string comparisons on small identifiers easily meet this target. Interning can be considered later via profiling.
- **`thiserror` in semantic submodules**:
    - *Rejected*: Handcrafted `Display` ensures precise diagnostic formatting and zero macro overhead.

---

## 4. Testing & Anti-Regression Strategy

### Decision

Adopt a two-tier testing strategy:

1. **Isolated Unit Tests**:
   - Reside within `crates/descar-core/src/semantic/tests/` (or inline module tests).
   - Test `binder`, `resolve`, and `sig` using manually constructed AST nodes (e.g., `Stmt::VarDeclaration`, `Stmt::Function`).
   - Do **not** invoke `JsavParser` or `Lexer` during unit tests, ensuring phase isolation.
2. **End-to-End Integration & Snapshot Tests**:
   - Reside in `crates/descar/tests/` with sample files in `tests/fixtures/*.dr`.
   - Snapshot diagnostic outputs and error messages using `insta` (`insta::assert_snapshot!`).
   - Developer review workflow: Any intentional or unintentional modification to error messages, error codes, or spans triggers snapshot mismatches, requiring explicit approval via `cargo insta review`.

### Rationale

- Guarantees that parser bugs or lexer changes do not cascade into false failures in semantic unit tests.
- Prevents subtle error message regressions (e.g., a change in `check.rs` inadvertently modifying an error message emitted by `resolve.rs`).

### Alternatives Considered

- **String Matching in Unit Tests (`assert_eq!(err.to_string(), "...")`)**:
    - *Rejected*: Hard to maintain across hundreds of tests; does not provide diff review tooling like `cargo insta`.
- **Integration Tests Only**:
    - *Rejected*: Fails to pinpoint which phase caused a bug and requires valid parser syntax for all edge cases.

---

## 5. Toolchain & CI Governance

### Decision

- Pin Rust compiler version strictly in `rust-toolchain.toml`:

  ```toml
  [toolchain]
  channel = "1.98.1"
  ```

- Toolchain updates must be performed manually in dedicated review pull requests, never automated in CI.
- CI pipeline in GitHub Actions runs:
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo test`

### Rationale

- Rust releases a stable compiler version every 6 weeks. Automatic CI updates risk breaking builds due to newly introduced lints or compiler regressions simultaneously with functional PRs. Manual pinning guarantees build reproducibility.

---

## 6. Type Inference, Contextual Typing & Extensibility Triggers

### Decision

- Function type checking uses an `Expectation` enum:

  ```rust
  pub enum Expectation {
      Infer,
      Check(TypeId),
  }
  ```

- **Current Behavior**:
    - `Infer`: Applies direct syntactic typing rules (e.g., integer/float literal defaults, string literals, boolean literals, declared function return types).
    - Contextual typing for unsuffixed numeric literals: If checked under `Expectation::Check(t)` where `t` is a numeric type, the unsuffixed literal adopts type `t` if its value fits within range; otherwise rejects with `CompileError::TypeError`.
    - No implicit type coercion across primitives (FR-011).
- **Future Extension Point**:
    - If the language acquires generics or full Hindley-Milner type inference, a constraint generation pass and a unification pass will be inserted between `sig` and `check`.
    - The `Expectation` signature remains unchanged; only the internal handling of `Expectation::Infer` will produce type variables and constraints.

---

## 7. Cycle Detection in Type Definitions

### Decision

During Phase 3 (`sig`), the type checker scans struct and enum definitions to ensure all types have a statically finite memory layout.

- If a `struct S` contains a field of type `S` (directly or transitively) without an indirection layer (like a pointer or heap vector), a cycle is detected via a DFS visited-set and an error (`CompileError::TypeError` / `InfiniteSizeRecursiveType`) is emitted.

### Rationale

- Prevents infinite recursion during type layout calculation and ensures types can be allocated in memory. Meets spec requirement FR-007 / Clarification 2026-09-14.

---

## 8. Data Persistence & Incremental Checking

### Decision

- All symbol tables, type tables, and diagnostics are strictly in-memory data structures valid solely for the lifetime of a single compiler invocation.
- Incremental compilation and query systems (such as `salsa`) are explicitly deferred and declared out of scope until language server (LSP) requirements are prioritized.
