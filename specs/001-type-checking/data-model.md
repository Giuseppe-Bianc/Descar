# Data Model: Semantic Type Checking

**Feature**: `001-type-checking` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

This document defines the core data structures, relationships, validation invariants, and state transitions for the semantic type checker in `descar-core`.

---

## 1. Core Context & Phase Model

```mermaid
classDiagram
    class CheckerCtx {
        +SymbolTable symbols
        +TypeTable types
        +Vec~Diagnostic~ diagnostics
        +add_diagnostic(Diagnostic)
        +has_errors() bool
    }

    class Phase {
        <<enumeration>>
        Binder
        Resolve
        Sig
        Check
        Globals
    }

    class Diagnostic {
        +Phase phase
        +ErrorCode code
        +String message
        +SourceSpan span
        +Option~String~ help
        +to_compile_error() CompileError
    }

    CheckerCtx o-- Diagnostic
    Diagnostic --> Phase
```

### 1.1 `Phase`

Represents the compilation phase responsible for an operation or diagnostic:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Binder,
    Resolve,
    Sig,
    Check,
    Globals,
}
```

### 1.2 `Diagnostic`

Carries complete source and diagnostic information tagged with the origin phase:

```rust
pub struct Diagnostic {
    pub phase: Phase,
    pub code: ErrorCode,
    pub message: String,
    pub span: SourceSpan,
    pub help: Option<String>,
}
```

**Invariants**:

- `span` must denote a valid location in source code.
- `help` must provide actionable fix guidance (FR-006).
- Maps directly to `CompileError::TypeError` for consumption by `ErrorReporter` (FR-013).

---

## 2. Symbol Table & Scope Tree (`binder` & `resolve`)

```mermaid
classDiagram
    class SymbolTable {
        +Vec~Scope~ scopes
        +ScopeId current_scope
        +ScopeId root_scope
        +enter_scope(ScopeKind) ScopeId
        +exit_scope()
        +insert_symbol(Symbol) Result~SymbolId, Diagnostic~
        +lookup(String) Option~SymbolId~
        +lookup_local(String) Option~SymbolId~
    }

    class Scope {
        +ScopeId id
        +Option~ScopeId~ parent
        +ScopeKind kind
        +HashMap~String, SymbolId~ bindings
    }

    class ScopeKind {
        <<enumeration>>
        Module
        Function
        Block
    }

    class Symbol {
        +SymbolId id
        +String name
        +SymbolKind kind
        +SourceSpan decl_span
        +ScopeId scope_id
        +Option~TypeId~ declared_type
        +bool is_mutable
    }

    class SymbolKind {
        <<enumeration>>
        Variable
        Parameter
        Function
        Struct
        Enum
    }

    SymbolTable o-- Scope
    Scope --> ScopeKind
    Scope o-- Symbol
    Symbol --> SymbolKind
```

### 2.1 `Scope` and `ScopeTree`

- Scope hierarchy is stored as an indexed arena: `Vec<Scope>` with `ScopeId(usize)`.
- Parent references are `Option<ScopeId>`. No reference counting or interior mutability (`Rc<RefCell<_>>`).
- **Inner-to-outer resolution** (FR-014):
  1. Check `current_scope.bindings`.
  2. If absent, follow `scope.parent` upwards until `root_scope` (Module scope).
  3. If not found in `root_scope`, lookup fails and emits an undeclared identifier diagnostic (FR-014, User Story 6).

### 2.2 Validation Invariants

- **Duplicate Declaration** (User Story 5): Inserting a symbol with name $N$ into scope $S$ when $S$ already contains an entry for $N$ fails immediately with a `Binder` duplicate declaration diagnostic. Shadowing in child scopes is permitted (User Story 4).

---

## 3. Type Table & Type Definitions (`sig` & `check`)

```mermaid
classDiagram
    class TypeTable {
        +Vec~TypeState~ types
        +HashMap~TypeKey, TypeId~ interned
        +HashMap~TypeDeclId, TypeId~ nominal
        +reserve_nominal(TypeDeclId, NominalKind) TypeId
        +complete(TypeId, TypeDefinition)
        +get_or_intern(TypeDefinition) TypeId
        +get(TypeId) TypeDefinition
        +is_assignable(target: TypeId, source: TypeId) bool
    }

    class TypeState {
        <<enumeration>>
        Reserved(TypeDeclId, NominalKind)
        Complete(TypeDefinition)
    }

    class TypeKey {
        <<enumeration>>
        Nominal(TypeDeclId)
        Structural(TypeDefinition)
    }

    class NominalKind {
        <<enumeration>>
        Struct
        Enum
    }

    class TypeId {
        +usize 0
    }

    class TypeDefinition {
        <<enumeration>>
        Primitive(PrimitiveType)
        Array(TypeId, usize)
        Vector(TypeId)
        Struct(String, Vec~FieldDef~)
        Enum(String, Vec~VariantDef~)
        Function(Vec~TypeId~, TypeId)
        Void
        NullPtr
    }

    class FieldDef {
        +String name
        +TypeId type_id
        +SourceSpan span
    }

    class VariantDef {
        +String name
        +Option~TypeId~ payload
        +SourceSpan span
    }

    TypeTable o-- TypeState
    TypeState --> TypeDefinition
    TypeState --> NominalKind
    TypeDefinition --> TypeId
    TypeDefinition o-- FieldDef
    TypeDefinition o-- VariantDef
```

### 3.1 `TypeId` Constants & Primitive Types

`TypeTable` pre-populates standard primitive `TypeId`s at initialization:

- Signed Integers: `I8`, `I16`, `I32`, `I64`
- Unsigned Integers: `U8`, `U16`, `U32`, `U64`
- Floating Point: `F32`, `F64`
- Others: `Char`, `String`, `Bool`, `Void`, `NullPtr`

### 3.2 Nominal Type Reservation

Nominal declarations receive their stable `TypeId` before any field or variant type is resolved. `TypeTable::reserve_nominal` creates a `Reserved(TypeDeclId, kind)` slot and records the declaration-identity-to-`TypeId` binding in `nominal`. `complete` replaces that slot with the fully resolved `TypeDefinition`; it must preserve the original `TypeId` and reject completion of an already completed or unrelated reservation.

`TypeTable::interned` uses a `TypeKey` discriminator: `Nominal(TypeDeclId)` for struct/enum declarations and `Structural(TypeDefinition)` for non-nominal type values. `get_or_intern` therefore interns structural definitions by their value while nominal declarations are keyed by their unique declaration identity rather than by their names or field layouts. This prevents two shadowed declarations with the same name and shape from collapsing to the same `TypeId`; distinct `SymbolId` or `TypeDeclId` values produce distinct nominal entries even when the declared fields are otherwise identical.

### 3.3 `sig` Type Resolution Flow

Phase 3 (`sig`) resolves nominal type declarations in two stages:

1. **Pre-register** each struct or enum declaration by reserving its `TypeId` before resolving any field or variant payload types. The declaration name is immediately bound to that reserved `TypeId` for type-name lookup.
2. Add the reserved `TypeId` to the DFS `in_progress` set, resolve all fields or variant payloads using the normal type-resolution path, then call `complete` with the resulting `TypeDefinition` and remove the `TypeId` from `in_progress`.

A type-name lookup that refers to a declaration already in `nominal` returns the reserved `TypeId`, even while its `TypeState` is `Reserved`. This makes a direct self-reference such as `struct Node { next: Node }` resolve to the same `TypeId` instead of being reported as an unknown type or requiring a completed definition first.

When resolving a referenced nominal type, `sig` consults `in_progress` before descending into that type's definition. If the referenced `TypeId` is already present, the current DFS path contains a cycle and `sig` emits the blocking recursive-type diagnostic. Because every declaration is reserved before its members are resolved, the same check detects transitive cycles such as `A -> B -> A` as well as direct cycles such as `Node -> Node`.

Indirection-capable field types such as pointers or dynamic vectors are handled by the type graph according to the existing infinite-size rule. Only edges that contribute to the enclosing type's inline layout participate in the blocking cycle check.

### 3.4 Type Validation Rules

1. **Nominal Typing** (Clarification 2026-09-15): Structs and enums are compared nominally by their `TypeId` / unique declaration identity, never structurally.
2. **Cycle & Infinite Size Detection** (User Story 8, FR-007):
   - A struct or enum field whose type directly or transitively contains the declaring type without indirection (e.g. pointer or dynamic vector) creates an infinite size layout.
   - Detected during Phase 3 (`sig`) by tracking a DFS `in_progress` set of types. Cycles trigger a blocking `Sig` diagnostic.
3. **Void Placement** (FR-010):
   - `Void` is strictly valid as a function return type.
   - Variable declarations with type `Void` (`let x: void`) are rejected with `CompileError::TypeError`.
4. **No Implicit Coercion** (FR-011):
   - Explicit casts required for mixed integer/float operations (`i32` + `f64` rejected).
   - Array literals must match declared array length and homogeneous element type.

---

## 4. Expression Checking & Expectations (`check`)

### 4.1 `Expectation` Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expectation {
    /// Infer type purely from expression syntax.
    Infer,
    /// Validate expression against an expected type.
    Check(TypeId),
}
```

### 4.2 Numeric Contextual Typing Rules (FR-001, FR-002)

- **Unsuffixed Numeric Literals** (e.g., `42`):

    - Under `Expectation::Check(t)` where `t` is integer/float: Adopt type `t` if the literal value falls within `t`'s min/max range.
    - Under `Expectation::Infer`: Defaults to `I32` for integers or `F64` for decimals.

- **Suffixed Numeric Literals** (e.g., `42u8`, `3.14f32`):

    - Retain their explicit type regardless of expectation. If checked against incompatible `Expectation::Check(t)`, emit mismatch diagnostic.

---

## 5. Pipeline State Transitions

```mermaid
stateDiagram-v2
    [*] --> UncheckedAST: Parser output

    UncheckedAST --> ScopedAST: binder::bind(AST, &mut ctx)
    note right of ScopedAST: Top-level decls registered,\nScope hierarchy constructed

    ScopedAST --> ResolvedAST: resolve::resolve(AST, &mut ctx)
    note right of ResolvedAST: Variable/Type usages linked to SymbolIds,\nUndeclared names flagged

    ResolvedAST --> SignedAST: sig::process_signatures(AST, &mut ctx)
    note right of SignedAST: Nominal TypeIds reserved before member resolution,\nSignatures completed in TypeTable,\nInfinite-size cycles detected

    SignedAST --> TypedAST: check::check_bodies(AST, &mut ctx)
    note right of TypedAST: Bodies checked against Expectation,\nExpressions annotated with TypeId

    TypedAST --> CheckedProgram: globals::check_globals(AST, &mut ctx)
    note right of CheckedProgram: Whole-program checks (unused items, visibility)

    CheckedProgram --> [*]: Success: Ok(CheckedProgram)
    CheckedProgram --> [*]: Errors: Err(Vec<Diagnostic>)
```
