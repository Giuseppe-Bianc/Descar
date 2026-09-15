# Contract: Command-Line Interface (CLI) Integration

**Feature**: `001-type-checking` | **Date**: 2026-09-15 | **Spec**: [spec.md](../spec.md)

This contract defines the CLI behavior for commands invoking semantic type checking in `descar`.

---

## 1. Commands

### 1.1 `descar check <input.dr>`

Runs lexical analysis, syntax parsing, and semantic type checking on the specified file without generating code or executing the program.

#### Input Arguments

- `<input.dr>`: Path to the Descar source file.
- Flags:
    - `-q`, `--quiet`: Suppress non-error output.
    - `-v`, `--verbose`: Show detailed compiler phases and diagnostics.

#### Exit Codes & Outputs

- **Exit Code `0`**:
    - Semantic type checking succeeded with zero errors.
    - Output: Empty if quiet, or status message `Checked <path> successfully` if verbose.
- **Exit Code `1`**:
    - File I/O error, syntax error, or semantic type error encountered.
    - Output printed to `stderr` formatted via `ErrorReporter`:

    ```text
    ERROR: Type error: [E2001] expected type 'i32', found 'bool' at example.dr:3:15
      |
    3 | let x: i32 = true;
      |              ^^^^
    help: change the expression or type annotation to match
    ```

### 1.2 `descar compile <input.dr>`

Compiles the input file. With the introduction of semantic type checking:

1. Performs lexing (`lexer_tokenize_with_errors`). If errors exist, report and exit `1`.
2. Performs parsing (`JsavParser::parse`). If errors exist, report and exit `1`.
3. Performs semantic type checking (`descar_core::semantic::check_program`).
    - If any `Diagnostic` is returned, formats all diagnostics via `ErrorReporter`, writes to `stderr`, and terminates with exit code `1`.
    - Halts the compilation pipeline before AST printing or code generation.

---

## 2. Invariants

1. `descar check` must never print the AST or attempt code generation.
2. `descar compile` must not proceed past semantic analysis if any blocking type error occurs.
3. Diagnostic formatting across `check` and `compile` must be identical, relying exclusively on `ErrorReporter::report_errors`.
