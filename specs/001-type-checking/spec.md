# Feature Specification: Type Checking

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Type-safe expressions (Priority: P1)

A compiler user expects expressions to be type-checked before code generation so that incompatible operations are rejected with actionable diagnostics.

**Why this priority**: Preventing invalid programs from reaching later compiler stages is the core value of the type-checker.

**Independent Test**: Compile programs containing valid and invalid expressions and verify that valid programs pass type checking while invalid programs produce type errors with source locations and guidance.

**Acceptance Scenarios**:

1. **Given** two operands of compatible primitive types, **When** an operator requiring those types is applied, **Then** type checking succeeds and the expression receives the resolved type.
2. **Given** operands of incompatible primitive types, **When** the operator is applied, **Then** type checking fails with `CompileError::TypeError` and a diagnostic identifying the mismatch and source span.
3. **Given** a function call with declared parameter types, **When** arguments match those types, **Then** type checking succeeds.
4. **Given** a function call with an argument type that does not match its parameter type, **When** type checking runs, **Then** it emits `CompileError::TypeError` with a diagnostic and a fix suggestion.

---

### User Story 2 - Type-safe declarations and assignments (Priority: P1)

A compiler user expects variable declarations and assignments to reject incompatible values rather than silently coercing them.

**Acceptance Scenarios**:

1. **Given** a variable declaration with a compatible initializer, **When** type checking runs, **Then** the declaration is accepted.
2. **Given** a variable declaration with an incompatible initializer, **When** type checking runs, **Then** it emits `CompileError::TypeError` with a diagnostic and a fix suggestion.
3. **Given** an assignment to an existing variable with a compatible value, **When** type checking runs, **Then** the assignment is accepted.
4. **Given** an assignment with an incompatible value, **When** type checking runs, **Then** it emits `CompileError::TypeError` with a diagnostic and a fix suggestion.

---

### User Story 3 - Scope-aware name and type resolution (Priority: P1)

A compiler user expects names and types to resolve according to lexical scope, including nested blocks and forward references supported by the language design.

**Acceptance Scenarios**:

1. **Given** a declaration in the current scope, **When** an identifier references it, **Then** the current declaration is selected.
2. **Given** no declaration in the current scope but a matching declaration in an enclosing scope, **When** an identifier references it, **Then** the enclosing declaration is selected.
3. **Given** an identifier with no declaration in any visible scope, **When** type checking runs, **Then** it emits `CompileError::TypeError` with the relevant source span and fix suggestion.
4. **Given** functions or types declared later in the same scope, **When** type checking runs, **Then** forward references resolve successfully through multi-pass analysis.

---

### User Story 4 - Typed AST for later compiler stages (Priority: P2)

A compiler implementation needs resolved type information attached to AST nodes so later compiler stages can consume it without repeating type inference or compatibility checks.

**Acceptance Scenarios**:

1. **Given** a successfully type-checked AST node, **When** type checking completes, **Then** its `type_info` contains the resolved type and relevant source information.
2. **Given** a type-checking failure, **When** the AST is inspected, **Then** already-resolved nodes retain their type information where safe, while invalid nodes are not treated as successfully typed.

---

### Edge Cases

- Empty blocks and functions with no return expression.
- Nested scopes that shadow outer bindings.
- Forward references between functions and types in the same scope.
- Recursive function references.
- Null pointer literals in pointer-compatible contexts.
- Arrays with constant versus non-constant sizes.
- Vector declarations where no fixed size is permitted.
- Invalid operators applied to otherwise valid operand types.
- Missing identifiers and types.
- Duplicate declarations where the existing compiler diagnostic conventions apply.

## Requirements *(mandatory)*

- **FR-001**: System MUST type-check all expressions before code generation.
- **FR-002**: System MUST attach resolved type information to AST nodes using a `type_info` field without changing unrelated AST semantics.
- **FR-003**: System MUST validate binary and unary operators against operand types and reject unsupported combinations.
- **FR-004**: System MUST validate function argument types against declared parameter types and validate function return expressions against declared return types.
- **FR-005**: System MUST validate variable declarations and assignments for exact type compatibility.
- **FR-006**: System MUST resolve identifiers through lexical scopes, searching the current scope before enclosing scopes.
- **FR-007**: System MUST support forward references for functions and types within the same scope through multi-pass analysis.
- **FR-008**: Type information attached to AST nodes MUST preserve enough source location information to support accurate diagnostics.
- **FR-009**: System MUST not modify unrelated components of compiler pipeline (lexer, parser) beyond attaching type information to AST nodes.
- **FR-010**: System MUST support core primitive types (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, string, bool), plus Void and NullPtr literals, and composite types Array and Vector. Typing rules: Array requires element type and constant size expression; Vector requires element type; Void usable only as function return type; NullPtr literal represents null pointer value.

  **Acceptance Scenarios for FR-010**:
  1. **Array**: declaration `var a: i32[10] = {1,2,3}` accepted if size constant and element type valid. Mismatched size or non‑constant size triggers `CompileError::TypeError` with diagnostic and fix suggestion.
  2. **Vector**: declaration `var v: vector<string> = {"a","b"}` accepted; any size spec on vector results in type error.
  3. **Void**: function `fun foo() : void { }` accepted; variable declaration `var x: void = ...` rejected with `CompileError::TypeError` and appropriate diagnostic.
  4. **NullPtr**: literal `nullptr` accepted; assigning to non‑pointer context triggers `CompileError::TypeError` with diagnostic.

- **FR-011**: System MUST not allow implicit type casting (coercion) between primitive types; explicit conversion MUST be required.
- **FR-012**: System MUST operate as a multi-pass AST visitor to support forward references for functions and types within the same scope; no flow-sensitive analysis (e.g., definite assignment) is required.
- **FR-013**: All type‑checking errors must be represented by the `TypeError` variant of `CompileError` defined in `src/error/compile_error.rs`
- **FR-014**: System MUST resolve identifiers using a stack of symbol tables in inner-to-outer order (current scope, then parent, up to global) to support nested scopes (blocks, functions). If no declaration is found after searching the global scope, the system MUST emit a `CompileError::TypeError` with the relevant source location, diagnostic details, and a fix suggestion.

### Key Entities

- **AST Node**: Represents syntactic elements; extended with `type_info` field after type checking.
- **TypeInfo**: Holds resolved type, constraints, and source location for diagnostics.
- **Diagnostic**: A type-checking failure MUST be returned directly as `CompileError::TypeError { code, message, span, help }`. When passed to `ErrorReporter::report_errors`, these fields MUST map directly to the reporter's corresponding inputs: `code` to the optional error-code prefix, `message` to the diagnostic message, `span` to the source location and underline, and `help` to the optional fix guidance. The reporter MUST format the error under the `TYPE` category.
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