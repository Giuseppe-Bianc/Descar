# Feature Specification: Full Typed AST

**Feature Branch**: `001-typed-ast`

**Created**: 2026-10-04

**Status**: Draft

**Input**: User description: "Implementare un full typed AST completo e coerente per il linguaggio previsto dalla grammatica del progetto, in modo da rappresentare esplicitamente tutte le principali categorie sintattiche e le relative informazioni tipologiche. L'AST deve modellare attraverso nodi distinti e semanticamente significativi tutti i costrutti previsti dalla grammatica, incluse espressioni, istruzioni, dichiarazioni, riferimenti, definizioni di tipo e ogni ulteriore categoria sintattica prevista."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Grammar-Complete Node Coverage (Priority: P1)

A compiler developer traverses a parsed Descar program and finds a distinct, identifiable AST node for every syntactic construct defined by the grammar: expressions (binary, unary, literal, grouping, variable reference, assignment, call, array access, array literal), statements (expression statement, variable declaration, function declaration, main function, if/else, while, for, block, return, break, continue), and type references (all primitive types, array type, vector type, void, nullptr, custom/user-defined type).

**Why this priority**: Without complete node coverage the AST cannot faithfully represent any arbitrary valid program, which blocks every downstream phase (type checking, semantic analysis, IR generation, printing).

**Independent Test**: Given the `large_toy_program.dr` or `sccp_test.dr` example files, parsing produces a `Vec<Stmt>` where every statement and sub-expression maps to a concrete, named node variant; no construct falls back to a generic catch-all.

**Acceptance Scenarios**:

1. **Given** a `.dr` source file containing every statement kind, **When** the parser produces an AST, **Then** each statement maps to a uniquely named node variant with no two structurally distinct constructs sharing the same variant.
2. **Given** an expression using all binary operators (arithmetic, comparison, logical, bitwise, shift, compound assignment), **When** the expression is parsed, **Then** each operator is carried as a distinct `BinaryOp` enum variant inside an `Expr::Binary` node, with left and right operands preserved.
3. **Given** a function declaration with parameters and a typed return annotation, **When** parsed, **Then** the `Stmt::Function` node contains the function name, a complete ordered list of `Parameter` nodes (each with name and type), the declared return type, and the body block.
4. **Given** a `for` loop with an initializer, condition, and increment clause, **When** parsed, **Then** the `Stmt::For` node preserves all three optional clauses independently and the body.

---

### User Story 2 - Uniform and Complete Type Model (Priority: P2)

A type-checker or semantic analysis pass retrieves the declared type of any variable, parameter, return position, or type annotation in the AST without having to infer or reconstruct it from raw tokens. Every position where the grammar allows a type annotation carries a first-class `Type` value.

**Why this priority**: Downstream semantic phases (type checking, type inference, diagnostics) depend on the type model being complete and uniform. Gaps require the type checker to re-parse or guess, reintroducing ambiguity.

**Independent Test**: Given a `.dr` source file declaring variables of all primitive types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, `bool`), arrays (`i32[10]`), vectors (`vector<f64>`), custom types, and `void`/`nullptr`, each type annotation position in the AST contains the exact `Type` variant without ambiguity.

**Acceptance Scenarios**:

1. **Given** a variable declaration `var x: i32 = 0`, **When** the AST node is inspected, **Then** the `type_annotation` field contains `Type::I32`, and the `is_mutable` flag is `true`.
2. **Given** a constant declaration `const y: bool = true`, **When** inspected, **Then** `type_annotation` is `Type::Bool` and `is_mutable` is `false`.
3. **Given** an array type annotation `i32[10]`, **When** inspected, **Then** the `Type::Array` node contains `element_type: Type::I32` and a `size` expression evaluating to the literal `10`.
4. **Given** a function parameter `n: f64`, **When** inspected, **Then** the `Parameter` node holds `name = "n"` and `type_annotation = Type::F64`.
5. **Given** a function with no explicit return type annotation, **When** parsed, **Then** the `return_type` field defaults to `Type::Void`.

---

### User Story 3 - Structural Integrity and Source Provenance (Priority: P2)

Every AST node carries a `SourceSpan` that locates it in the original source text. For nodes with a direct textual origin (a source token or token range), the span reproduces the verbatim source fragment. For synthesised nodes created by the parser to fill implicit defaults (e.g., `Type::Void` for a function with no declared return type), the span is a zero-width anchor at the last consumed token — valid for positional diagnostics but not expected to reproduce a source fragment. Structural relationships between nodes are explicit (e.g., the else branch of an `if` is a distinct `ElseBranch` variant, not a raw `Option<Box<Stmt>>`), and every node can be traversed without implicit interpretation.

**Why this priority**: Error reporting, diagnostic highlighting, and AST pretty-printing all require precise source positions. Ambiguous structural relationships create silent information loss.

**Independent Test**: Given any source file, every AST node (expression and statement) exposes a `span()` method returning a valid `SourceSpan` that maps back to the original source text.

**Acceptance Scenarios**:

1. **Given** a statement at line 5 columns 3–20 in the source, **When** the corresponding AST node's `span()` is inspected, **Then** the returned `SourceSpan` correctly identifies that source range.
2. **Given** an `if` statement with an `else if` continuation, **When** the AST is inspected, **Then** the `else_branch` field is `ElseBranch::ElseIf(...)` containing the nested `Stmt::If` node, not `ElseBranch::Block` or `ElseBranch::None`.
3. **Given** a block with multiple statements, **When** the `Stmt::Block` node is traversed, **Then** each child statement is accessible by index in the `statements` vector in source order.

---

### User Story 4 - Typed AST Readiness for Downstream Phases (Priority: P3)

A semantic analysis pass or IR generator receives the AST and can, without modifying the `Expr` enum itself, associate resolved type information with expression nodes via a `TypedExpr` wrapper struct. The `TypedExpr` wrapper pairs an `Expr` with an `Option<Type>` resolved-type field, allowing parsing phases to work with plain `Expr` trees while semantic phases operate on `TypedExpr` trees. The core `Expr` enum remains unchanged and usable without annotation.

**Why this priority**: Downstream passes (type inference, IR generation, optimisation) need to attach resolved types to expression nodes. Embedding the annotation in `Expr` variants would force every parser-side consumer to handle the type field even before type checking runs; a wrapper cleanly separates the two concerns.

**Independent Test**: After a type-checking pass runs over a fully parsed program, every `TypedExpr` node that corresponds to a typed construct (literals, variables, calls, binary/unary expressions) exposes its resolved type through `TypedExpr::resolved_type`, without requiring the caller to re-derive the type from scratch.

**Acceptance Scenarios**:

1. **Given** a typed expression `1 + 2`, **When** the type checker produces a `TypedExpr` tree, **Then** the `TypedExpr` wrapping the binary node carries `resolved_type: Some(Type::I32)` (or the promoted numeric type).
2. **Given** a `TypedExpr` wrapping a `Expr::Variable` node for a declared `var x: f64`, **When** the type checker has run, **Then** `resolved_type` is `Some(Type::F64)`.
3. **Given** an expression node whose type cannot be resolved (e.g., it references an undeclared variable), **When** inspected post-checking, **Then** `resolved_type` is `None` for that specific expression, while other expressions in the same program whose types are determinable still carry `Some(T)` — errors are local and do not suppress annotation of unaffected nodes.

---

### Edge Cases

- What happens when a variable declaration has multiple bindings (`var a, b: i32 = 1, 2`)? Each `VarBinding` must be preserved as a separate entry with its own initializer, all sharing the same declared type annotation.
- What happens when a function has no parameters? The `parameters` field must be an empty `Vec<Parameter>`, not absent.
- How does the AST represent `nullptr`? As `Expr::Literal { value: LiteralValue::NullPtr, .. }` with corresponding type `Type::NullPtr`.
- What happens when an array type has a non-literal size expression? The `Type::Array { size, .. }` node must hold the full expression tree, not just an integer constant.
- How are numeric literals with non-decimal bases (binary `#b...`, octal `#o...`, hexadecimal `#x...`) represented? They must be carried as `LiteralValue::Numeric(Number)` with the same variant as decimal literals — the base is resolved at lex time.
- What happens for a `for` loop with all three clauses absent? Each of `initializer`, `condition`, `increment` is independently `None`; the body is still present.
- How is an expression statement with no trailing semicolon handled? The `Stmt::Expression` node wraps the expression; the absence of a semicolon is a lexer/parser concern, not an AST concern. The `Stmt::Expression` variant carries its own `span` field covering the full statement extent (including a trailing semicolon when present), consistent with all other `Stmt` variants.
- How is `vector<T>` distinguished from a user-defined type named `vector`? The `Type::Vector` variant is produced when `vector` is followed by `<T>` in the type position; otherwise the identifier resolves to `Type::Custom`.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The AST MUST define a separate node variant for each of the following expression kinds: binary operation, unary operation, grouping (parenthesised expression), literal (numeric, string, character, boolean, nullptr), array literal, variable reference, assignment, function/method call, and array index access. The Assignment variant covers only the simple assignment operator =. Compound-assignment operators (+=, -=, …) are not covered by this variant. They are represented as Expr::Binary nodes with the corresponding BinaryOp compound-assignment variant.
- **FR-002**: The AST MUST define a separate node variant for each of the following kinds of statements: expression statement, variable declaration with a mutability flag, function declaration, main function declaration, if/else-if/else, while loop, three-clause for loop, block, return, break, and continue.
- **FR-003**: The AST shall define a type model that includes all primitive scalar types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, `bool`), the fixed-size array type, the dynamic vector type, `void`, `nullptr`, and user-defined custom named types.
- **FR-004**: Every expression node and every statement node MUST have a `SourceSpan` value. For nodes with a direct source text origin, the span MUST map the node to the exact source range that it occupies. For synthesized nodes, that is, nodes created by the parser for implicit defaults with no corresponding source token, the span MUST be a zero-width `SourceSpan`. The span MUST be anchored at the position of the last consumed token before the synthesis point. The `Stmt::Expression` variant MUST have its own explicit `span: SourceSpan` field. The span MUST cover the full statement, including any trailing terminator. The `Stmt::Expression` variant MUST NOT rely only on the span of the inner `Expr`.
- **FR-005**: The `Stmt::VarDeclaration` node MUST contain an ordered `bindings` collection, one shared `type_annotation`, and an `is_mutable` flag. The `is_mutable` flag MUST identify whether the declaration uses `var` or `const`.
- **FR-006**: The `Stmt::Function` node shall contain the function name, an ordered list of `Parameter` nodes, a declared return type, and the body as a `Stmt::Block`. Each `Parameter` node shall contain a name and a type annotation. If the return type is omitted, the node shall use `Type::Void`.
- **FR-007**: The `ElseBranch` model MUST distinguish three cases: no else clause (`None`), a plain else block (`Block`), or an else-if continuation (`ElseIf`). Chained conditionals MUST be explicit in the model.
- **FR-008**: Binary operators MUST be represented by a closed enumeration covering all arithmetic, comparison, logical, bitwise, shift, and compound-assignment operators defined by the grammar. Each operator MUST map to exactly one variant.
- **FR-009**: Unary operators MUST be represented as a closed enumeration. The enumeration MUST contain negation (`-`), logical NOT (`!`), bitwise complement (`~`), increment (`++`), and decrement (`--`). Each operator MUST be combined with a `UnaryOpSide` value. The `UnaryOpSide` value MUST be `Prefix` or `Postfix`. The value MUST define the application side of the operator.
- **FR-010**: The `Type::Array` variant MUST contain the element type and the size as an expression node, not as a raw integer, to correctly represent array types with a size that is a compile-time constant expression.
- **FR-011**: The `Type::Vector` variant MUST contain its element type and MUST have syntax that is different from a `Type::Custom` named `vector`.
- **FR-012**: Every `Expr` node MUST have a `span()` accessor that returns a reference to its `SourceSpan`.
- **FR-013**: Every `Stmt` node MUST provide a `span()` accessor that returns a reference to its `SourceSpan`.
- **FR-014**: The AST MUST provide one uniform method to attach and read a resolved type annotation on `Expr` nodes. Semantic analysis passes MUST use this method to store resolved type information. Semantic analysis passes MUST NOT use external side tables. The method MUST use a `TypedExpr` wrapper struct. The struct MUST contain an `Expr` and an `Option<Type>` field named `resolved_type`. The `resolved_type` field MUST contain the resolved type annotation. The core `Expr` enum MUST NOT change. The `Expr` enum MUST remain fully usable without a resolved type. This MUST support parsing and other pre-semantic phases. The AST MUST provide a corresponding `TypedStmt` wrapper for statement nodes that own typed sub-expressions. Both `TypedExpr` and `TypedStmt` MUST be in a new `syntax::typed_ast` sub-module within `descar-core`. The untyped AST MUST remain in `syntax::ast`. The typed AST MUST remain in `syntax::typed_ast`. These modules MUST stay clearly separate.The annotation MUST apply to each expression separately. Each expression MUST have an independent `resolved_type` value. `resolved_type` MUST be `Some(T)` when the type of that specific expression can be determined. This MUST be true even when other parts of the program contain errors.`resolved_type` MUST be `None` only when the type of that specific expression cannot be resolved.
- **FR-015**: The `Stmt::For` node MUST represent the initializer, the condition, and the increment as independent optional fields. Each field MUST be either absent (`None`) or present. The state of one field MUST NOT affect the state of another field.
- **FR-016**: Numeric literals with non-decimal bases (binary `#b`, octal `#o`, hexadecimal `#x`) MUST use the same `LiteralValue::Numeric(Number)` variant as decimal literals; the lexer MUST resolve the base.
- **FR-017**: Multi-binding variable declarations (`var a, b: T = e1, e2`) MUST be represented as a single `Stmt::VarDeclaration` with multiple `VarBinding` entries. Each `VarBinding` MUST pair one name with its initializer expression.
- **FR-018**: The AST model MUST be internally consistent: no two structurally different constructs SHALL use the same node variant, and each node variant SHALL be used for one semantic concept only.

### Key Entities

- **`Expr`**: The sum type of all expression node variants. Each variant carries a `SourceSpan`. The type exposes `span()`.
- **`Stmt`**: The sum type of all statement node variants. Every variant carries a `SourceSpan`. No variant is exempt. This satisfies FR-004 and the clarification that `Stmt::Expression` has its own explicit `span` field.
- **`Type`**: The sum type of all type annotation variants. The variants are primitives, array, vector, void, nullptr, and custom.
- **`BinaryOp`**: A closed enumeration of all binary operators.
- **`UnaryOp`**: A closed enumeration of all unary operators.
- **`UnaryOpSide`**: A two-variant enumeration. It identifies prefix and postfix application.
- **`LiteralValue`**: The sum type of all literal value kinds. The kinds are numeric, string, char, bool, and nullptr.
- **`Parameter`**: A named tuple with `name`, `type_annotation`, and `span` for function parameters.
- **`VarBinding`**: A named tuple with `name` and an optional initializer for multi-binding declarations.
- **`ElseBranch`**: A three-variant enumeration for the else clause of an if statement.
- **`TypedExpr`**: A wrapper struct in `syntax::typed_ast`. It contains an `Expr` and an `Option<Type>` resolved-type field. The type-checker pass populates this field. `None` means that the type of the specific expression cannot be resolved.
- **`TypedStmt`**: A wrapper struct in `syntax::typed_ast`. It contains a `Stmt` and any typed sub-expression context. It mirrors `TypedExpr` at the statement level for phases that operate on a fully annotated tree.

---

## Clarifications

### Session 2026-10-04

- Q: How should the resolved-type annotation be structurally attached to each `Expr` node — as an optional field embedded directly inside every expression variant, or as a separate wrapper type that wraps the entire `Expr` together with its resolved type? → A: Wrapper type — introduce a `TypedExpr` struct pairing `Expr` with `Option<Type>`; the core `Expr` enum stays unchanged.
- Q: Should the `Stmt::Expression` variant carry its own dedicated `SourceSpan` field, or is delegating to the inner `Expr`'s span sufficient to satisfy the round-trip span fidelity requirement? → A: Add a dedicated `span: SourceSpan` field to `Stmt::Expression`, covering the full statement extent including any terminator.
- Q: When the type checker encounters an expression inside a program that contains type errors elsewhere, should `TypedExpr::resolved_type` for that expression be `None` or `Some(...)` with a best-effort inferred type? → A: Per-expression independent annotation — `Some(T)` if that specific expression's type is determinable, `None` only if that specific expression's own type cannot be resolved; errors are local and do not suppress annotation of unaffected nodes.
- Q: For the purpose of SC-006, which changes to `Expr` or `Stmt` enums should be treated as breaking and require explicit documentation and a migration path? → A: Renaming, removing, or changing the type of an existing field/variant is breaking; adding new optional fields or new variants (marked `#[non_exhaustive]`) is non-breaking. Breaking changes MUST be documented in a `MIGRATION.md` note.
- Q: Should `TypedExpr` and `TypedStmt` be defined in the existing `syntax::ast` module or in a new dedicated sub-module within `descar-core`? → A: New `syntax::typed_ast` sub-module within `descar-core` — keeps untyped AST (`syntax::ast`) and typed AST (`syntax::typed_ast`) in clearly separated modules.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Given any syntactically valid `.dr` source file, the parsed AST contains no "unrecognised" or generic catch-all node for any construct defined by the grammar; coverage is 100% of grammar productions relevant to subsequent compiler phases.
- **SC-002**: Every type annotation position in the AST (variable declaration, parameter, return type, array size) is represented by a concrete `Type` variant — no position silently defaults to `Type::Void` or `Type::Custom` for a construct that has a specific type keyword.
- **SC-003**: All example `.dr` files in `dr_files/` are parseable to an AST that passes a structural completeness check (every node variant, span, and child field is non-null/non-empty where the grammar mandates presence).
- **SC-004**: A round-trip property holds for nodes with a direct textual origin: for every AST node that corresponds to one or more source tokens, the `span()` method returns a source range that, when extracted from the original source text, reproduces the verbatim source fragment that produced the node. Synthesised nodes — nodes created by the parser to fill in implicit defaults with no corresponding source token (e.g., a `Type::Void` return type for a function that omits the return annotation) — are explicitly exempt from the verbatim-reproduction clause. Synthesised nodes MUST carry a zero-width `SourceSpan` anchored at the position of the last consumed source token before the synthesis point (e.g., the closing `)` of the parameter list for an implicit `Type::Void`). This anchor span is valid for position-based diagnostics but does not reproduce a source fragment.
- **SC-005**: The resolved-type annotation mechanism on `Expr` nodes is exercised by the existing type-checker tests: after the type checker runs, typed expressions carry their resolved type and the tests continue to pass with no regression.
- **SC-006**: All existing tests in `crates/descar-core/tests/` pass without modification after the AST changes are applied. A "breaking change" is defined as: renaming, removing, or changing the type of an existing public `Expr` or `Stmt` field or variant. Adding new optional fields or new enum variants (marked `#[non_exhaustive]` where appropriate) is non-breaking and does not require a migration path. Any breaking change introduced by this feature MUST be explicitly documented in a `MIGRATION.md` note within the feature directory.

---

## Assumptions

- The grammar of the Descar language is fully captured by the current `JsavParser` and the existing token set in `TokenKind`; no grammar extensions (structs, enums, generics beyond `vector<T>`, lambdas, traits, modules) are in scope for this feature.
- The existing `SourceSpan` and `SourceLocation` types are sufficient to represent all required source ranges; no new location infrastructure is needed.
- The `Number` type in `crates/descar-core/src/tokens/number.rs` is the canonical representation for all numeric literal values and is shared between the lexer and the AST's `LiteralValue::Numeric` variant.
- The `Type::Void` variant serves as the default return type for functions that do not declare one; this default is applied by the parser, not the type checker. The synthesised `Type::Void` node carries a zero-width `SourceSpan` anchored at the last consumed token before synthesis (e.g., the closing `)` of the parameter list), consistent with the synthesised-node span policy in FR-004 and SC-004.
- The resolved-type annotation on `Expr` nodes (FR-014) is implemented as a `TypedExpr` wrapper struct rather than an embedded field on each `Expr` variant. The `Expr` enum itself is not changed, preserving all existing parsing and printing code that constructs or matches on `Expr` variants without type information.
- The `vector<T>` type notation is the only generic/parameterised type constructor in scope; no other parameterised type syntax exists in the grammar.
- All existing public APIs (parser output, printer input, type-checker input) remain stable; the typed AST is additive and backward-compatible where possible, or migration is explicit and documented.
- The `specs/` directory did not previously exist; this is the first feature specification in the project.
