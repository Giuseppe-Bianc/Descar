# Contract: Diagnostics & Snapshot Testing

**Feature**: `001-type-checking` | **Date**: 2026-09-15 | **Spec**: [spec.md](../spec.md)

This contract defines error codes, diagnostic fields, and snapshot testing conventions for the semantic type checker.

---

## 1. Error Codes (`ErrorCode` Range E2xxx)

All semantic diagnostics map to standardized error codes defined in `descar_core::error::error_code::ErrorCode`:

| Error Code | Name | Phase | Spec Ref | Description |
|------------|------|-------|----------|-------------|
| `E2001` | `TypeMismatch` | `Check` | FR-001, US-1 | Expression type does not match the expected type |
| `E2002` | `UndeclaredVariable` | `Resolve` | FR-014, US-6 | Referenced identifier not found in any accessible scope |
| `E2003` | `DuplicateDeclaration` | `Binder` | FR-014, US-5 | Identifier declared more than once in the same scope |
| `E2004` | `UnknownType` | `Resolve` | FR-007, US-7 | Type name cannot be resolved in symbol table |
| `E2005` | `InfiniteSizeRecursiveType` | `Sig` | FR-007, US-8 | Struct or enum contains itself without indirection |
| `E2006` | `InvalidOperatorOperands` | `Check` | FR-002, US-2 | Binary/unary operator applied to incompatible operand types |
| `E2007` | `NonBooleanCondition` | `Check` | FR-005 | Condition expression in `if`/`while` is not `bool` |
| `E2028` | `ArgumentCountMismatch` | `Check` | FR-003, US-3 | Function call provides fewer or more arguments than declared |
| `E2029` | `ReturnTypeMismatch` | `Check` | FR-004 | Return expression type does not match function return type |
| `E2030` | `InvalidVoidUsage` | `Sig` | FR-010 | `void` used outside of a function return type annotation |
| `E2031` | `InvalidArraySize` | `Sig` | FR-010 | Array size expression is not a constant or non-negative integer |
| `E2032` | `InvalidNullPtrUsage` | `Check` | FR-010 | `nullptr` assigned or passed to a non-pointer target |

---

## 2. Diagnostic Content Structure

Each `Diagnostic` must satisfy:

1. **`phase`**: Populated with the originating `Phase` (`Binder`, `Resolve`, `Sig`, `Check`, `Globals`).
2. **`code`**: An explicit `ErrorCode` from the table above.
3. **`message`**: Clear, descriptive error description stating both what was expected and what was found.
4. **`span`**: The exact `SourceSpan` of the erroneous token/expression.
5. **`help`**: A non-empty actionable fix suggestion (FR-006).

---

## 3. Snapshot Testing Conventions (`insta`)

Integration tests assert the formatted output of `check_program` using `insta`:

```rust
// tests/type_check_snapshots.rs
#[test]
fn test_type_mismatch_snapshot() {
    let source = include_str!("fixtures/type_mismatch.dr");
    let diagnostics = compile_and_check_get_diagnostics(source);
    insta::assert_snapshot!("type_mismatch", diagnostics);
}
```

### Review Workflow

- Any change to diagnostic text, error code, or span location will cause `cargo test` to fail.
- Engineers run `cargo insta review` to inspect and accept or reject the proposed snapshot diff.
