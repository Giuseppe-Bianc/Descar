# Quickstart & Validation Guide: Semantic Type Checking

**Feature**: `001-type-checking` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

This guide provides step-by-step instructions to validate the semantic type checking stage in Descar locally and in CI.

---

## 1. Prerequisites & Environment Setup

1. **Rust Toolchain**: Rust 1.98.1 stable (edition 2024), pinned via `rust-toolchain.toml`:

   ```toml
   [toolchain]
   channel = "1.98.1"
   ```

2. **Cargo Snapshot Tooling** (for local snapshot review):
   ```bash
   cargo install cargo-insta
   ```

---

## 2. Running Verification Scenarios

### Scenario A: Successful Semantic Verification

Verify a well-typed program compiles and passes type checking cleanly:

```bash
cargo run -p descar -- check tests/fixtures/valid_simple.dr
```

**Expected Outcome**: Process exits with code `0` and reports no errors.

---

### Scenario B: Type Mismatch Detection (User Story 1 / E2001)

Verify detection of invalid assignment:

```bash
cargo run -p descar -- check tests/fixtures/type_mismatch.dr
```

**Expected Outcome**: Process exits with code `1`, printing:

```text
ERROR: Type error: [E2001] expected type 'i32', found 'bool' at tests/fixtures/type_mismatch.dr:1:14
help: change the expression or type annotation to match
```

---

### Scenario C: Operator Operand Validation (User Story 2 / E2006)

Verify detection of mismatched arithmetic operands (e.g. string + integer):

```bash
cargo run -p descar -- check tests/fixtures/invalid_binary_op.dr
```

**Expected Outcome**: Process exits with code `1`, reporting operator and operand types.

---

### Scenario D: Undeclared Variable Detection (User Story 6 / E2002)

Verify detection of an identifier absent from symbol tables:

```bash
cargo run -p descar -- check tests/fixtures/undeclared_var.dr
```

**Expected Outcome**: Process exits with code `1`, reporting undeclared identifier with line/column location and fix suggestion.

---

### Scenario E: Recursive Type Cycle Detection (User Story 8 / E2005)

Verify detection of infinite-size recursive structs:

```bash
cargo run -p descar -- check tests/fixtures/recursive_struct.dr
```

**Expected Outcome**: Process exits with code `1`, reporting infinite recursive type layout.

---

## 3. Automated Test Execution

### 3.1 Unit Tests (Synthetic AST Isolation)

Execute isolated phase tests without invoking the parser:

```bash
cargo test -p descar-core semantic::
```

### 3.2 Snapshot Integration Tests

Execute end-to-end integration tests using `insta`:

```bash
cargo test -p descar --test type_check_snapshots
```

### 3.3 Interactive Snapshot Review

If diagnostic formatting was updated intentionally:

```bash
cargo insta review
```

---

## 4. CI Pipeline Conformance Checks

Execute the mandatory CI gates in order:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All three commands must pass without warnings or errors.
