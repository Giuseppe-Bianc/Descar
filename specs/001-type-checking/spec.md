# Feature Specification: Semantic Type Checking

**Feature Branch**: `001-type-checking`

**Created**: 2026-09-14

**Status**: Draft

**Input**: User description: "Add a semantic type-checking stage to the existing compiler pipeline that runs after parsing and before [codegen/interpretation]. The type checker validates variable declarations, assignments, expressions, literals, operators, function calls, return statements, and control-flow constructs against the language's type rules. It must integrate using the same architectural patterns, traversal style, and diagnostic-reporting conventions already established by the lexer and parser. The design must allow new types, operators, expressions, and statements to be added with localized changes only, without modifying unrelated parts of the type-checking infrastructure. Type errors must report precise source locations, the incompatible types involved, and the expected type or constraint, formatted consistently with existing lexer/parser diagnostics."

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-09-14

- Q: Which types must the semantic type checker support? -> A: Core primitives (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, string, bool) and user-defined structs/enums
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
- Q: In what order should the type checker search the symbol table stack for an identifier? -> A: Inner-to-outer; search current scope, then parent, up to global
- Q: Which `CompileError` variant should be used for duplicate declarations in the same scope? -> A: Reuse `CompileError::TypeError`
- Q: Does language support forward references for functions and types? -> A: Allow forward references within same scope (multi-pass)
- Q: How is "medium-sized project" defined for the performance target in SC-003? -> A: By line count (1k-10k LOC)
- Q: Are user-defined structs and enums nominally typed or structurally typed? -> A: Nominally typed
- Q: Does language support generic types (e.g., `List<T>`) or only concrete types? -> A: Concrete types only (generics not supported yet)
- Q: Should the type checker handle constant expressions (e.g., `const X = 1 + 2;`) by evaluating them during type checking, or just check their types? -> A: Just check types (no evaluation)
- Q: What happens when identifier lookup reaches global scope without a declaration? -> A: Emit `CompileError::TypeError` with location, diagnostic details, and fix suggestion.
- Q: Are recursive structs and enums permitted in the language? -> A: Not permitted (Emit `CompileError::TypeError`)

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

**Independent Test**: Define function `fn foo(i: i32) {}` and call `foo("s");`. Expect error indicating expected `i32` vs provided `string`.

**Acceptance Scenarios**:

1. **Given** function call with correct argument types, **When** compiler runs, **Then** no type error.
2. **Given** function call with one argument wrong type, **When** compiler runs, **Then** error shows expected type and provided type.
3. **Given** function call with multiple arguments mismatched, **When** compiler runs, **Then** error reports each mismatched argument's expected vs provided types.

---

### User Story 4 - Variable Shadowing (Priority: P2)

Developer declares variable in nested scope with same name as outer scope. Compiler resolves use of variable to the innermost declaration.

**Why this priority**: Essential for block-scoped language semantics.

**Independent Test**: Define `let x: i32 = 1; { let x: string = "a"; let y = x; } let z = x;`. Expect `y` to be type `string` (inner `x`) and `z` to be type `i32` (outer `x`).

**Acceptance Scenarios**:

1. **Given** nested scopes with same identifier, **When** identifier is used, **Then** compiler resolves to most local definition.

---

### User Story 5 - Duplicate Declaration (Priority: P2)

Developer declares variable twice in same scope. Compiler reports duplicate declaration error.

**Why this priority**: Prevent ambiguous variable resolution.

**Independent Test**: Define `let x: i32 = 1; let x: i32 = 2;` in same block. Expect error reporting duplicate identifier `x`.

**Acceptance Scenarios**:

1. **Given** same scope with duplicate identifier, **When** compiler runs, **Then** error reports duplicate declaration.

---

### User Story 6 - Undeclared Variable (Priority: P1)

Developer uses variable that is not declared in any accessible scope. Compiler reports error.

**Why this priority**: Essential for correctness; prevents use of undefined identifiers.

**Independent Test**: Compile file with `let y = x + 1;` where `x` is undeclared. Expect `CompileError::TypeError` with location, "undeclared variable 'x'", and fix suggestion.

**Acceptance Scenarios**:

1. **Given** use of undeclared identifier, **When** compiler runs, **Then** error includes `CompileError::TypeError`, precise location, diagnostic details, and fix suggestion.

---

### User Story 7 - Unknown Type (Priority: P1)

Developer uses a type name that is not defined in the current or global scope. Compiler reports error.

**Why this priority**: Essential for type safety; prevents use of undefined types.

**Independent Test**: Define `let x: UnknownType = 1;`. Expect `CompileError::TypeError` with location, "unknown type 'UnknownType'", and fix suggestion.

**Acceptance Scenarios**:

1. **Given** use of unknown type, **When** compiler runs, **Then** error includes `CompileError::TypeError`, precise location, diagnostic details, and fix suggestion.

---

### User Story 8 - Recursive Type Detection (Priority: P2)

Developer defines a struct or enum that refers to itself directly or indirectly. Compiler reports error.

**Why this priority**: Prevent infinite loops during type validation and ensure type size is finite.

**Independent Test**: Define `struct Node { next: Node }`. Expect `CompileError::TypeError` with location, "recursive type definition detected", and fix suggestion.

**Acceptance Scenarios**:

1. **Given** recursive type definition, **When** compiler runs, **Then** error includes `CompileError::TypeError`, precise location, diagnostic details, and fix suggestion.

---

### Edge Cases

- What happens when type inference fails due to ambiguous generic constraints? System MUST treat this as type error and require explicit type annotations.
- How does system handle recursive type definitions that could cause infinite loops? System MUST use visited-set or memoization to detect and handle cycles.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST traverse AST after parsing, perform local type inference for variable declarations, and validate variables, assignments, literals, and expressions. Unsuffixed numeric literals receive contextual typing: they adopt the expected type from variable declarations, assignment targets, or operand positions when compatible; literal values must be within target type's range; explicitly suffixed literals retain their declared type without implicit conversion. Expected types propagate through compound expressions (e.g., arithmetic, logical) without triggering implicit casts, respecting FR‑011.
- **FR-002**: System MUST validate operator operand types according to language rules, including contextual typing for unsuffixed numeric literals and range validation, and emit diagnostic on mismatch.
- **FR-003**: System MUST check function call argument types against parameter signatures and report mismatches.
- **FR-004**: System MUST verify return statement types against function return type annotations.
- **FR-005**: System MUST validate control-flow constructs (if, while, match) for condition expression types being boolean.
- **FR-006**: System MUST produce diagnostics that include source location (file, line, column), found type, expected type or constraint, a fix suggestion for every type error, following existing diagnostic format, with CompileError::TypeError help field set to Some(...).
- **FR-007**: System MUST allow extension of type system (new primitive types, user-defined structs, enums) with localized changes only, defined as modifications limited to ≤3 files and ≤150 lines total, and no changes to core type-checker logic beyond registration hooks.

**Acceptance Scenarios for FR-007**:

1. **New primitive type** `myint`: addition restricted to type definition file and registration in type registry; no other files altered.
2. **User-defined struct** `Point`: changes confined to struct definition module and type registry; other components unchanged.
3. **User-defined enum** `Color`: modifications limited to enum definition file and registry; no impact on existing type-checker code.

- **FR-008**: System MUST allow addition of new operators, expressions, or statements with minimal impact on existing type-checker components, defined as at most 5 files changed and ≤200 lines total across modifications, and no changes to core type-checker logic beyond designated extension points.

**Acceptance Scenarios for FR-008**:

1. **New operator** `**` (exponentiation): only operator definition file, precedence table, and operator type rule modified; other components unchanged.
2. **New expression** `len(expr)`: changes limited to expression handler and type inference for `len`; no other modules altered.
3. **New statement** `assert!`: changes confined to statement parser and type-check rule; no other components affected.

- **FR-009**: System MUST not modify unrelated components of compiler pipeline (lexer, parser) beyond attaching type information to AST nodes.
- **FR-010**: System MUST support core primitive types (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, string, bool) and user-defined nominally typed structs and enums.
- **FR-011**: System MUST not allow implicit type casting (coercion) between primitive types; explicit conversion MUST be required.
- **FR-012**: System MUST operate as a multi-pass AST visitor to support forward references for functions and types within the same scope; no flow-sensitive analysis (e.g., definite assignment) is required.
- **FR-013**: All type‑checking errors must be represented by the `TypeError` variant of `CompileError` defined in `src/error/compile_error.rs`
- **FR-014**: System MUST resolve identifiers using a stack of symbol tables in inner-to-outer order (current scope, then parent, up to global) to support nested scopes (blocks, functions). If no declaration is found after searching the global scope, the system MUST emit a `CompileError::TypeError` with the relevant source location, diagnostic details, and a fix suggestion.

### Key Entities

- **AST Node**: Represents syntactic elements; extended with `type_info` field after type checking.
- **TypeInfo**: Holds resolved type, constraints, and source location for diagnostics.
- **Diagnostic**: Existing structure used by lexer/parser; type-checker must emit compatible diagnostics.
- **Symbol Table**: Maps identifiers to their types and metadata within a specific scope.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of type errors in test suite are reported with accurate source location and expected type.
- **SC-002**: No false-positive type errors in a suite of valid programs (baseline >= 200 test cases).
- **SC-003**: Type-checking adds at most 15% overhead to overall compilation time on average for medium-sized projects (defined as 1k-10k LOC).
- **SC-004**: Stakeholder satisfaction rating ≥ 4/5 in post‑implementation survey regarding clarity of type error messages.

## Assumptions

- Project remains Rust‑only; no external language toolchains introduced.
- Existing AST structure can be extended with a `type_info` field without breaking other passes.
- Diagnostic formatting conventions are defined in `descar-core` and remain unchanged.
- No new syntax introduced; type information attached to existing nodes.
- Developers are familiar with Rust ownership and borrowing semantics, which the type checker respects.
