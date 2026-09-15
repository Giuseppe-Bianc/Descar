# Feature Specification: Semantic Type Checking

**Feature Branch**: `001-type-checking`

**Created**: 2026-09-14

**Status**: Draft

**Input**: User description: "Add a semantic type-checking stage to the existing compiler pipeline that runs after parsing and before [codegen/interpretation]. The type checker validates variable declarations, assignments, expressions, literals, operators, function calls, return statements, and control-flow constructs against the language's type rules. It must integrate using the same architectural patterns, traversal style, and diagnostic-reporting conventions already established by the lexer and parser. The design must allow new types, operators, expressions, and statements to be added with localized changes only, without modifying unrelated parts of the type-checking infrastructure. Type errors must report precise source locations, the incompatible types involved, and the expected type or constraint, formatted consistently with existing lexer/parser diagnostics."

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-09-14
- Q: Which types must the semantic type checker support? -> A: Core primitives (i32, f64, bool, str) and user-defined structs/enums
- Q: How should the type checker handle recursive type definitions to prevent infinite loops during validation? -> A: Use visited-set or memoization to detect and handle cycles
- Q: Does the language support implicit type casting (coercion) between primitive types (e.g., i32 to f64)? -> A: No implicit casting; require explicit conversion
- Q: Should the type checker perform any flow-sensitive analysis (e.g., checking for uninitialized variables)? -> A: Simple AST visitor; no flow-sensitive analysis
- Q: How should the type checker handle ambiguous generic constraints if they are introduced in the future? -> A: Treat as type error; require explicit type annotations

### Session 2026-09-15
- Q: Should the type checker reuse the existing `CompileError` enum for its error reporting, matching the pattern used in lexer and parser? -> A: Reuse `CompileError` enum
- Q: Must the type checker use the `TypeError` variant from `CompileError`? -> A: Yes, use `CompileError::TypeError` variant.
- Q: Should diagnostics include fix suggestions? -> A: Include fix suggestions for all type errors.
- Q: How should the type checker resolve variable and type identifiers? -> A: Stack of symbol tables; supports nested blocks and functions
- Q: To what extent should the type checker support type inference? -> A: Local inference for variable declarations

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Type error detection (Priority: P1)

Developer compiles source file containing mismatched types. Compiler reports error with file, line, column, found type, expected type.

**Why this priority**: Core functionality; users need immediate feedback on type errors.

**Independent Test**: Run `cargo run -- check example.dr` with a file containing `let x: i32 = true;`. Expect error message with location and type mismatch.

**Acceptance Scenarios**:

1. **Given** source with invalid assignment, **When** compiler runs, **Then** error output includes precise location and expected type.
2. **Given** source with valid types, **When** compiler runs, **Then** no type errors reported.

---

### User Story 2 - Operator type validation (Priority: P2)

Developer uses arithmetic operator on incompatible types (e.g., adding string to integer). Compiler flags error.

**Why this priority**: Prevent runtime failures; enforce language semantics.

**Independent Test**: Compile file with `let x = "a" + 1;`. Expect type error specifying operator and operand types.

**Acceptance Scenarios**:

1. **Given** source with operator misuse, **When** compiler runs, **Then** error cites operator and operand types.
---

### User Story 3 - Function call type checking (Priority: P3)

Developer calls function with arguments of wrong types. Compiler reports mismatch for each argument.

**Why this priority**: Ensures correct API usage within code.

**Independent Test**: Define function `fn foo(i: i32) {}` and call `foo("s");`. Expect error indicating expected `i32` vs provided `&str`.
---

### Edge Cases

- What happens when type inference fails due to ambiguous generic constraints? System MUST treat this as type error and require explicit type annotations.
- How does system handle recursive type definitions that could cause infinite loops? System MUST use visited-set or memoization to detect and handle cycles.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST traverse AST after parsing and perform local type inference (for variable declarations) and validation for variables, assignments, literals, and expressions.
- **FR-002**: System MUST validate operator operand types according to language rules and emit diagnostic on mismatch.
- **FR-003**: System MUST check function call argument types against parameter signatures and report mismatches.
- **FR-004**: System MUST verify return statement types against function return type annotations.
- **FR-005**: System MUST validate control-flow constructs (if, while, match) for condition expression types being boolean.
- **FR-006**: System MUST produce diagnostics that include source location (file, line, column), found type, expected type or constraint, optional fix suggestion, and follow existing diagnostic format.
- **FR-007**: System MUST allow extension of type system (new primitive types, user-defined structs, enums) with localized changes only.
- **FR-008**: System MUST allow addition of new operators or expressions with minimal impact on existing type-checker components.
- **FR-009**: System MUST not modify unrelated components of compiler pipeline (lexer, parser) beyond attaching type information to AST nodes.
- **FR-010**: System MUST support core primitive types (i32, f64, bool, str) and user-defined structs and enums.
- **FR-011**: System MUST not allow implicit type casting (coercion) between primitive types; explicit conversion MUST be required.
- **FR-012**: System MUST operate as simple AST visitor; no flow-sensitive analysis (e.g., definite assignment) is required.
- **FR-013**: All type‑checking errors must be represented by the `TypeError` variant of `CompileError` defined in `src/error/compile_error.rs`
- **FR-014**: System MUST resolve identifiers using a stack of symbol tables to support nested scopes (blocks, functions).

### Key Entities

- **AST Node**: Represents syntactic elements; extended with `type_info` field after type checking.
- **TypeInfo**: Holds resolved type, constraints, and source location for diagnostics.
- **Diagnostic**: Existing structure used by lexer/parser; type-checker must emit compatible diagnostics.
- **Symbol Table**: Maps identifiers to their types and metadata within a specific scope.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of type errors in test suite are reported with accurate source location and expected type.
- **SC-002**: No false-positive type errors in a suite of valid programs (baseline >= 200 test cases).
- **SC-003**: Type-checking adds at most 15% overhead to overall compilation time on average for medium-sized projects.
- **SC-004**: Stakeholder satisfaction rating ≥ 4/5 in post‑implementation survey regarding clarity of type error messages.

## Assumptions

- Project remains Rust‑only; no external language toolchains introduced.
- Existing AST structure can be extended with a `type_info` field without breaking other passes.
- Diagnostic formatting conventions are defined in `descar-core` and remain unchanged.
- No new syntax introduced; type information attached to existing nodes.
- Developers are familiar with Rust ownership and borrowing semantics, which the type checker respects.
