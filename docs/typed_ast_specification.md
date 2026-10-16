# Specification: Fully Typed Abstract Syntax Tree for the Descar Compiler

**Document ID:** DESCAR-SPEC-AST-001  
**Version:** 1.0.0  
**Date:** 2026-10-16  
**Status:** Draft  

---

## Table of Contents

1. [Purpose and Scope](#1-purpose-and-scope)
2. [Applicable Standards and References](#2-applicable-standards-and-references)
3. [Terms and Definitions](#3-terms-and-definitions)
4. [Assumptions and Constraints](#4-assumptions-and-constraints)
5. [AST Architecture](#5-ast-architecture)
6. [Node Model](#6-node-model)
7. [Type System](#7-type-system)
8. [Typing Rules](#8-typing-rules)
9. [Symbol Resolution and Semantic References](#9-symbol-resolution-and-semantic-references)
10. [Invariants and Validity Criteria](#10-invariants-and-validity-criteria)
11. [Error Handling](#11-error-handling)
12. [Operations and Interfaces](#12-operations-and-interfaces)
13. [Verification and Acceptance Criteria](#13-verification-and-acceptance-criteria)
14. [Technical Examples](#14-technical-examples)
15. [Open Issues and Design Decisions](#15-open-issues-and-design-decisions)
16. [Bibliographic References](#16-bibliographic-references)

---

## 1. Purpose and Scope

### 1.1 Purpose

This document specifies the structure, type rules, semantic constraints, and validity criteria for the **Fully Typed Abstract Syntax Tree (FTAST)** of the Descar compiler.

The FTAST is a transformation of the parser-produced AST. It annotates each typable node with its resolved `Type`. This makes type information available to all subsequent compilation phases without repeated traversal or re-derivation.

The document gives enough detail for a developer to implement the FTAST. All essential requirements are stated. All architecture decisions are documented. The developer shall not need to infer requirements or make undocumented decisions.

### 1.2 Scope

This specification applies to:

- The node model of the FTAST.
- The type system used by the Descar language.
- The typing rules that assign a type to each typable node.
- The symbol resolution model.
- The invariants that a valid FTAST must satisfy.
- The error model for type checking and AST construction.
- The operations defined over the FTAST.
- The verification criteria for the FTAST.

This specification does **not** cover:

- The lexer (tokenization phase).
- The parser (syntax analysis phase), except where its output forms the input to the FTAST construction process.
- Intermediate representation (IR) generation, which uses the FTAST as input.
- Code generation.
- The command-line interface.

### 1.3 Relationship to Existing AST

The current parser-produced AST (hereafter the **Parser AST**) contains all structural information but does not store resolved types on expression nodes. The FTAST adds type annotations. The FTAST is a separate, enriched representation produced by the type checker pass.

---

## 2. Applicable Standards and References

| ID | Document |
|----|----------|
| REF-01 | *Compilers: Principles, Techniques, and Tools*, Second Edition. A.V. Aho, M.S. Lam, R. Sethi, J.D. Ullman. Addison-Wesley, 2007. (Hereafter: "Dragon Book") |
| REF-02 | ASD-STE100 Issue 7, *Simplified Technical English*, AeroSpace and Defence Industries Association of Europe, 2017. |
| REF-03 | Descar compiler source code, crate `descar-core`, version at commit referenced in `Cargo.lock`. |
| REF-04 | IEEE 754-2019, *Standard for Floating-Point Arithmetic*. |

**Note on REF-01:** This document applies concepts from the Dragon Book as theoretical foundations. Section citations are provided when a concept is directly traceable to a specific chapter or section. Section numbers are not cited unless they can be confirmed from the edition listed above. When a concept is a project-specific design decision, this document states that explicitly.

---

## 3. Terms and Definitions

The following terms are used throughout this document. Each term has one precise meaning.

| Term | Definition |
|------|------------|
| **AST** | Abstract Syntax Tree. A tree-shaped data structure that represents the syntactic structure of a program after parsing. (Dragon Book, Chapter 2) |
| **Parser AST** | The AST produced by the Descar parser. Expression nodes in the Parser AST do not carry a resolved type field. |
| **FTAST** | Fully Typed Abstract Syntax Tree. The enriched AST produced after type checking. Every typable node carries a resolved `Type`. |
| **Node** | One element of the AST or FTAST. A node represents one syntactic or semantic construct. |
| **Expression node** | A node that computes a value. Expression nodes are typable. |
| **Statement node** | A node that represents an action or declaration. Statement nodes are not directly typed, but they contain typed sub-nodes. |
| **Type** | A classification of a value or expression. Types determine which operations are valid on a value. |
| **Declared type** | The type written explicitly in the source code (for example, the annotation in `var x: i32`). |
| **Inferred type** | The type that the type checker derives for an expression that has no explicit annotation. |
| **Resolved type** | The type stored on a node in the FTAST after type checking. For annotated nodes, the resolved type equals the declared type. For unannotated nodes, the resolved type equals the inferred type. |
| **Promoted type** | A type that results from applying numeric promotion to two operands of different numeric types. The promoted type is the type of the result. |
| **Type promotion** | The implicit widening of a numeric type to a wider compatible type. (Dragon Book, Section 6.5) |
| **Type compatibility** | The property of two types that allows one to be used where the other is expected. |
| **Type assignability** | The property that a value of a given type can be stored in a variable of a declared type. Assignability is a project-specific rule defined in Section 8.3. |
| **Symbol** | A named entity declared in the program. A symbol can be a variable, a function, or a type alias. |
| **Symbol table** | A data structure that maps names to their associated symbols within a scope. (Dragon Book, Section 2.7) |
| **Scope** | A region of the program in which a symbol is visible. |
| **Lexical scope** | A scope defined by the static structure of the program text, not by runtime behavior. (Dragon Book, Section 1.6) |
| **Scope kind** | One of three values: `Global`, `Function`, or `Block`. Scope kinds determine visibility rules. |
| **SourceSpan** | A half-open interval `[start, end)` identifying a region of source text by file path, start position, and end position. |
| **Typable node** | A node that can have a meaningful resolved type. All expression nodes are typable. |
| **Untypable node** | A node that does not have a meaningful resolved type. Statement nodes are untypable unless they are expression statements. |
| **Error node** | A concept used to describe a node that the type checker could not fully resolve because of a semantic error. The FTAST uses `Option<Type>` fields on expression nodes to represent this: a `None` value indicates an unresolved type. |
| **Promotion hierarchy** | A total order on numeric types that determines which type is wider. |
| **Compound assignment** | A binary operator that combines arithmetic or bitwise computation with assignment, for example `+=`, `-=`, `*=`. |
| **Void** | A special type that indicates the absence of a value. Functions with no return value have type `Void`. |
| **NullPtr** | A special type for the null pointer literal. `NullPtr` is not assignable to any named type in the current language version. |
| **Custom type** | A user-defined named type. Custom type resolution to a concrete definition is not performed in the current version. |
| **Structural validity** | A property of an AST or FTAST: all required fields are present, all cardinality constraints are met, and the tree has no cycles. |
| **Semantic validity** | A property of an FTAST: all typing rules are satisfied, all symbols are resolved, and all control-flow constraints are met. |

---

## 4. Assumptions and Constraints

### 4.1 Confirmed Assumptions

The following assumptions are based on the source code and are confirmed by the codebase analysis.

| ID | Assumption | Basis |
|----|------------|-------|
| ASM-01 | The Descar language is a statically typed language. All types are known at compile time. | `TypeChecker` performs compile-time type checking. |
| ASM-02 | The FTAST is produced from a structurally valid Parser AST. The type checker does not repair structural errors. | `TypeChecker::check` receives `&[Stmt]` from the parser. |
| ASM-03 | Type checking is performed in a single depth-first pass over the Parser AST. The FTAST is produced after that pass completes. | `TypeChecker::check` calls `visit_statements` once. |
| ASM-04 | Type checking accumulates all errors. It does not stop at the first error. | `TypeChecker.errors: Vec<CompileError>` is populated throughout the pass. |
| ASM-05 | The Descar language uses lexical scoping. Inner scopes can shadow outer scopes. | `SymbolTable::lookup` searches from the innermost scope outward. |
| ASM-06 | The `main` block is the program entry point. It is treated as a void function with no parameters. | `visit_main_function` calls `visit_function` with `Type::Void`. |
| ASM-07 | Numeric type promotion follows a fixed hierarchy: `F64 > F32 > U64 > I64 > U32 > I32 > U16 > I16 > U8 > I8`. | `HIERARCHY` constant in `type_checker.rs`. |
| ASM-08 | The FTAST is consumed by the IR generation phase. IR generation requires all expression types to be resolved. | Pipeline structure in `main.rs` and IR module (E3xxx errors). |
| ASM-09 | Custom types are parsed and stored as `Type::Custom { name }` but are not resolved to a concrete definition during type checking. | `Type::Custom` variant has no definition field; no resolution logic in `type_checker.rs`. |
| ASM-10 | The `NullPtr` type is a valid literal type but cannot be used as a variable annotation or assignment target in the current version. | No assignability rule in `is_assignable` permits `NullPtr` on the right-hand side of a typed assignment. |

### 4.2 Provisional Assumptions

The following assumptions are necessary to complete the specification. They are not confirmed by the source code and may change.

| ID | Assumption | Reason | How to verify |
|----|------------|--------|---------------|
| PASSUM-01 | The FTAST will be a new set of types separate from the Parser AST types. It will not mutate the Parser AST in place. | Mutating the Parser AST requires adding `Option<Type>` fields to `Expr`, which changes the Parser AST type. A separate FTAST avoids this coupling. | Confirmed when the design decision in OI-01 is resolved. |
| PASSUM-02 | The `Vector` type behaves identically to a dynamic-size array for type-checking purposes. Array-specific rules that reference a size expression do not apply to `Vector`. | No size expression is present in `Type::Vector`. | Confirmed by reviewing any future IR generation code that handles `Type::Vector`. |
| PASSUM-03 | Function overloading is not supported in the current version. Each function name maps to exactly one `FunctionSymbol` per scope. | `SymbolTable::declare` rejects redeclaration. The `FunctionSymbol` has no overload list. | Confirmed by attempting to declare two functions with the same name in the same scope. |

### 4.3 Constraints

| ID | Constraint |
|----|------------|
| CON-01 | The FTAST must be producible from the Parser AST without modifying the parser or the lexer. |
| CON-02 | The FTAST must use the `Type` enum defined in `descar-core::syntax::ast::ast_type` without modification. If the `Type` enum must change, the change must be documented in this specification. |
| CON-03 | The FTAST must preserve all `SourceSpan` information from the Parser AST. |
| CON-04 | The FTAST must not use unsafe Rust code. |
| CON-05 | The implementation language is Rust, stable toolchain. |

---

## 5. AST Architecture

### 5.1 Compilation Pipeline Context

The FTAST is produced by the semantic analysis phase and consumed by the IR generation phase. The following table shows the position of the FTAST in the pipeline.

| Phase | Input | Output |
|-------|-------|--------|
| Lexer | Source text (`.dr` file) | Token stream |
| Parser | Token stream | Parser AST (`Vec<Stmt>`) |
| **Type Checker (Semantic Analysis)** | **Parser AST** | **FTAST + `Vec<CompileError>`** |
| IR Generator | FTAST | Intermediate representation |
| Code Generator | IR | Target code |

This design follows the multi-pass compiler model described in Dragon Book, Chapter 1.

### 5.2 FTAST Structure Overview

The FTAST is a tree of nodes. The root is a sequence of top-level `Stmt` nodes (the program). Each node contains:

- A **node kind** that identifies its role.
- **Child nodes** that represent sub-expressions or sub-statements.
- A **SourceSpan** for diagnostic reporting.
- For expression nodes: a **resolved type** (`Option<Type>`).

The FTAST is organized into two primary node categories:

1. **Expression nodes** (`TypedExpr`): Nodes that compute a value. All expression nodes carry a resolved type.
2. **Statement nodes** (`TypedStmt`): Nodes that represent actions, declarations, or control flow. Statement nodes do not carry a direct type but contain typed expression nodes.

Auxiliary data types support the primary nodes:
- `Type`: Represents a Descar type.
- `BinaryOp`, `UnaryOp`, `UnaryOpSide`: Operators.
- `LiteralValue`, `Number`: Literal data.
- `Parameter`, `VarBinding`: Declaration helpers.
- `ElseBranch`: Represents the optional else part of an `if` statement.
- `SymbolRef`: A resolved reference to a symbol in the symbol table (see Section 6.3).

### 5.3 Node Identity and Sharing

Each node in the FTAST is a distinct value. The FTAST does not use shared references between nodes. This is a project-specific decision based on the Rust ownership model. The consequences are:

- Cloning a node is a deep copy.
- There are no cycles.
- Node identity is value identity (structural equality).

This approach is consistent with the current Parser AST design, which uses `#[derive(Clone, PartialEq, Eq)]` on all AST types.

### 5.4 Abstract and Concrete Nodes

The FTAST uses two abstract categories: `TypedExpr` and `TypedStmt`. Both are Rust enums. All variants of these enums are concrete nodes that can be instantiated.

The following types are used as helpers. They are not nodes in the FTAST, but they appear as fields within nodes:

- `Type` (Section 7)
- `BinaryOp`, `UnaryOp`, `UnaryOpSide`
- `LiteralValue`, `Number`
- `Parameter`, `VarBinding`
- `ElseBranch`
- `SourceSpan`
- `SymbolRef`

---

## 6. Node Model

### 6.1 Notation

The following notation is used in this section.

- `field: FieldType` describes a required field.
- `field: Option<FieldType>` describes an optional field.
- `field: Vec<FieldType>` describes an ordered, possibly empty list.
- `field: Box<NodeType>` describes a single child node held by pointer.
- A **validity condition** is a rule that must be true for the node to be structurally valid.
- A **semantic constraint** is a rule that must be true for the node to be semantically valid.
- `ty: Option<Type>` on expression nodes: `Some(T)` means the node has resolved type `T`; `None` means the type could not be resolved because of a semantic error.

### 6.2 Expression Nodes (`TypedExpr`)

All `TypedExpr` variants carry a `ty: Option<Type>` field. This field is `None` if and only if the type checker recorded at least one error while resolving this node.

#### 6.2.1 `TypedExpr::Literal`

**Semantic function:** Represents a literal value.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `value` | `LiteralValue` | Yes | The literal value. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type. Must not be `None` for a well-formed literal. |

**Type assignment rule:** The type is determined by the `LiteralValue` variant.

| `LiteralValue` variant | Resolved type |
|------------------------|---------------|
| `Numeric(Number::I8(_))` | `Type::I8` |
| `Numeric(Number::I16(_))` | `Type::I16` |
| `Numeric(Number::I32(_))` | `Type::I32` |
| `Numeric(Number::Integer(_))` | `Type::I64` |
| `Numeric(Number::U8(_))` | `Type::U8` |
| `Numeric(Number::U16(_))` | `Type::U16` |
| `Numeric(Number::U32(_))` | `Type::U32` |
| `Numeric(Number::UnsignedInteger(_))` | `Type::U64` |
| `Numeric(Number::Float32(_))` | `Type::F32` |
| `Numeric(Number::Scientific32(_, _))` | `Type::F32` |
| `Numeric(Number::Float64(_))` | `Type::F64` |
| `Numeric(Number::Scientific64(_, _))` | `Type::F64` |
| `StringLit(_)` | `Type::String` |
| `CharLit(_)` | `Type::Char` |
| `Bool(_)` | `Type::Bool` |
| `NullPtr` | `Type::NullPtr` |

**Validity conditions:**
- `value` must be a valid `LiteralValue` variant.
- `ty` must not be `None`.

**Structural invariant:** Literal nodes are leaf nodes. They have no child `TypedExpr` nodes.

---

#### 6.2.2 `TypedExpr::Variable`

**Semantic function:** Represents a reference to a declared variable.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Variable name. |
| `span` | `SourceSpan` | Yes | Source location. |
| `symbol_ref` | `Option<SymbolRef>` | Yes | Resolved reference to the variable symbol. `None` if the variable is not declared. |
| `ty` | `Option<Type>` | Yes | Resolved type. `None` if the variable is not declared. |

**Type assignment rule:** If `symbol_ref` is `Some(ref)`, then `ty` is `Some(ref.ty.clone())`. If `symbol_ref` is `None`, then `ty` is `None`.

**Validity conditions:**
- `name` must not be empty.
- `symbol_ref` and `ty` must be consistent: if one is `Some`, the other must be `Some`.

**Semantic constraint:** A `Variable` node where `symbol_ref` is `None` indicates an undefined variable. The type checker records error E2023.

---

#### 6.2.3 `TypedExpr::Assign`

**Semantic function:** Represents an assignment. The assignment evaluates to the type of the target.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target` | `Box<TypedExpr>` | Yes | Assignment target. Must be a `Variable` or `ArrayAccess` node. |
| `value` | `Box<TypedExpr>` | Yes | Value to assign. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type. Equals the resolved type of `target` when valid. |

**Validity conditions:**
- `target` must be a `Variable` or `ArrayAccess` node.
- The resolved type of `value` must be assignable to the resolved type of `target`.

**Semantic constraints:**
- The variable referenced by `target` must be declared as mutable. If it is immutable, the type checker records error E2024.
- If `target.ty` is `None`, then `ty` is `None`.
- If `value.ty` is `None`, then `ty` is `None`.

**Structural invariant:** `target` has exactly one direct child expression.

---

#### 6.2.4 `TypedExpr::Binary`

**Semantic function:** Represents a binary operation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `left` | `Box<TypedExpr>` | Yes | Left operand. |
| `op` | `BinaryOp` | Yes | Binary operator. |
| `right` | `Box<TypedExpr>` | Yes | Right operand. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type of the operation result. |
| `promoted_operand_type` | `Option<Type>` | Yes | The common type to which both operands are promoted before the operation is applied. `None` for non-numeric operators. |

**Note on `promoted_operand_type`:** This field records the result of type promotion (Section 8.4). It distinguishes between the declared type of each operand and the type after implicit widening. This distinction is required by IR generation to insert the correct conversion instructions. This is a project-specific design decision (see OI-02 in Section 15).

**Validity conditions:**
- Both `left` and `right` must not be `None`.

**Type assignment rules by operator class:**

| Operator class | Operators | Result type | Operand constraint |
|----------------|-----------|-------------|-------------------|
| Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `Mod` and their `=` forms | Promoted operand type | Both operands must be numeric. |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` | `Type::Bool` | Both operands must be compatible after promotion. |
| Logical | `&&`, `\|\|` | `Type::Bool` | Both operands must be `Type::Bool`. |
| Bitwise / Shift | `&`, `\|`, `^`, `<<`, `>>` and their `=` forms | Promoted operand type | Both operands must be integer types. |

See Section 8.5 for the full operator typing rules.

---

#### 6.2.5 `TypedExpr::Unary`

**Semantic function:** Represents a unary operation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `op` | `UnaryOp` | Yes | Unary operator. |
| `side` | `UnaryOpSide` | Yes | `Prefix` or `Postfix`. |
| `expr` | `Box<TypedExpr>` | Yes | Operand. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type. |

**Type assignment rules:**

| Operator | Operand constraint | Result type |
|----------|--------------------|-------------|
| `Negate` | Operand must be numeric. | Same as operand type. |
| `Not` | Operand must be `Bool`. | `Type::Bool` |
| `BitwiseNot` | Operand must be an integer type. | Same as operand type. |
| `Increment` | Operand must be numeric and refer to a mutable variable. | Same as operand type. |
| `Decrement` | Operand must be numeric and refer to a mutable variable. | Same as operand type. |

**Validity conditions:**
- `Postfix` is valid only for `Increment` and `Decrement`.
- `Prefix` is valid for all unary operators.

---

#### 6.2.6 `TypedExpr::Grouping`

**Semantic function:** Represents a parenthesized expression. The grouping node exists to preserve source location information. It does not affect the type.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `expr` | `Box<TypedExpr>` | Yes | Inner expression. |
| `span` | `SourceSpan` | Yes | Source location of the parentheses. |
| `ty` | `Option<Type>` | Yes | Resolved type. Equals `expr.ty`. |

**Type assignment rule:** `ty = expr.ty`.

**Validity conditions:** `ty` must equal `expr.ty`.

---

#### 6.2.7 `TypedExpr::ArrayLiteral`

**Semantic function:** Represents an array literal.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `elements` | `Vec<TypedExpr>` | Yes | Element expressions. Must not be empty. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type. Has the form `Type::Array { element_type, size }`. |

**Type assignment rule:** Let `T` be the resolved type of the first element. All elements must have the same type `T`. The resolved type of the array literal is `Type::Array { element_type: Box::new(T), size: Box::new(Literal(Integer(len))) }` where `len` is the number of elements.

**Validity conditions:**
- `elements` must not be empty. If it is empty, the type checker records error E2020 and `ty` is `None`.
- All elements must have the same resolved type. If types differ, the type checker records error E2021 and `ty` is `None`.

---

#### 6.2.8 `TypedExpr::Call`

**Semantic function:** Represents a function call.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `callee` | `Box<TypedExpr>` | Yes | The expression that names the function. Must resolve to a `Variable` node that refers to a `FunctionSymbol`. |
| `arguments` | `Vec<TypedExpr>` | Yes | Argument expressions. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type. Equals the return type of the called function. |
| `function_ref` | `Option<FunctionSymbolRef>` | Yes | Resolved reference to the function symbol. |

**Type assignment rule:** `ty = function_ref.return_type`.

**Validity conditions:**
- `callee` must resolve to a declared function name.
- The number of arguments must match the number of parameters in the function declaration.
- The resolved type of each argument must be assignable to the declared type of the corresponding parameter.

**Semantic constraints:**
- If the callee does not resolve to a function, the type checker records error E2023.
- Argument count mismatch is a semantic error (see OI-03 in Section 15 for the error code status).

---

#### 6.2.9 `TypedExpr::ArrayAccess`

**Semantic function:** Represents an element access in an array.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `array` | `Box<TypedExpr>` | Yes | The array expression. |
| `index` | `Box<TypedExpr>` | Yes | The index expression. |
| `span` | `SourceSpan` | Yes | Source location. |
| `ty` | `Option<Type>` | Yes | Resolved type. Equals the element type of the array. |

**Type assignment rule:** If `array.ty` is `Some(Type::Array { element_type, .. })`, then `ty = Some(*element_type)`. Otherwise, `ty = None`.

**Validity conditions:**
- `array.ty` must be a `Type::Array` or `Type::Vector`. If it is any other type, the type checker records a semantic error.
- `index.ty` must be an integer type.

---

### 6.3 The `SymbolRef` Type

`SymbolRef` is a resolved reference to a `VariableSymbol`. It is stored on `TypedExpr::Variable` nodes.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `Arc<str>` | Symbol name. |
| `ty` | `Type` | Type of the variable at the point of declaration. |
| `mutable` | `bool` | Whether the variable is mutable. |
| `defined_at` | `SourceSpan` | Source location of the declaration. |

**Design rationale:** Including `SymbolRef` in the FTAST allows IR generation to access symbol metadata without querying the symbol table. This avoids rebuilding the symbol table in later phases.

The `FunctionSymbolRef` type used in `TypedExpr::Call` has the same structure as `FunctionSymbol` in the symbol table.

---

### 6.4 Statement Nodes (`TypedStmt`)

Statement nodes do not carry a `ty` field. They represent program structure, not computed values. The following subsections describe each statement node.

#### 6.4.1 `TypedStmt::Expression`

**Semantic function:** Evaluates an expression for its side effects.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `expr` | `Box<TypedExpr>` | Yes | The expression to evaluate. |

**Note:** The resolved type of `expr` is computed and stored on `expr.ty`, but the statement itself has no type.

---

#### 6.4.2 `TypedStmt::VarDeclaration`

**Semantic function:** Declares one or more variables with a common declared type.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bindings` | `Vec<TypedVarBinding>` | Yes | One or more variable name and optional initializer pairs. Must not be empty. |
| `type_annotation` | `Type` | Yes | The declared type of all variables in `bindings`. |
| `is_mutable` | `bool` | Yes | Whether the declared variables are mutable. |
| `span` | `SourceSpan` | Yes | Source location. |

`TypedVarBinding` is:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Variable name. |
| `initializer` | `Option<Box<TypedExpr>>` | No | Optional initializer expression. |

**Validity conditions:**
- `bindings` must not be empty.
- If an initializer is present, its resolved type must be assignable to `type_annotation`. If not, the type checker records error E2002.
- Each name in `bindings` must be unique within the declaration. Names must not already be declared in the current scope. If a name is already declared, the type checker records error E2032.

---

#### 6.4.3 `TypedStmt::Function`

**Semantic function:** Declares and defines a named function.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Function name. |
| `parameters` | `Vec<Parameter>` | Yes | Parameter list. May be empty. |
| `return_type` | `Type` | Yes | Declared return type. |
| `body` | `Box<TypedStmt>` | Yes | Function body. Must be a `Block` node. |
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:**
- `name` must not be empty.
- `body` must be a `Block` node.
- If `return_type` is not `Void`, the body must contain a return path on all control-flow branches. If a return path is missing, the type checker records error E2003.
- Each `Parameter.name` must be unique within the parameter list.

---

#### 6.4.4 `TypedStmt::MainFunction`

**Semantic function:** The program entry point. It is treated as a void function with no parameters.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `body` | `Box<TypedStmt>` | Yes | Main function body. Must be a `Block` node. |
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:** Same as `TypedStmt::Function` with `return_type = Type::Void` and an empty parameter list.

**Design note:** `MainFunction` is a separate node from `Function` because it has a distinct syntactic form (`main { ... }` without a return type) and distinct semantic rules.

---

#### 6.4.5 `TypedStmt::If`

**Semantic function:** Conditional execution.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `condition` | `Box<TypedExpr>` | Yes | The condition expression. |
| `then_branch` | `Box<TypedStmt>` | Yes | The branch executed when the condition is `true`. |
| `else_branch` | `TypedElseBranch` | Yes | The optional else part. |
| `span` | `SourceSpan` | Yes | Source location. |

`TypedElseBranch` is an enum with three variants:
- `None`: No else branch.
- `Block(Box<TypedStmt>)`: An else block.
- `ElseIf(Box<TypedStmt>)`: An else-if chain (another `If` node).

**Validity conditions:**
- `condition.ty` must be `Some(Type::Bool)`. If it is not, the type checker records error E2004.

---

#### 6.4.6 `TypedStmt::While`

**Semantic function:** A loop that repeats while the condition is `true`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `condition` | `Box<TypedExpr>` | Yes | Loop condition. |
| `body` | `Box<TypedStmt>` | Yes | Loop body. |
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:**
- `condition.ty` must be `Some(Type::Bool)`. If it is not, the type checker records error E2004.

---

#### 6.4.7 `TypedStmt::For`

**Semantic function:** A C-style three-part loop.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `initializer` | `Option<Box<TypedStmt>>` | No | Optional initialization statement. |
| `condition` | `Option<Box<TypedExpr>>` | No | Optional loop condition. |
| `increment` | `Option<Box<TypedExpr>>` | No | Optional increment expression. |
| `body` | `Box<TypedStmt>` | Yes | Loop body. |
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:**
- If `condition` is present, `condition.ty` must be `Some(Type::Bool)`. If it is not, the type checker records error E2004.

---

#### 6.4.8 `TypedStmt::Block`

**Semantic function:** A sequence of statements in a new lexical scope.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statements` | `Vec<TypedStmt>` | Yes | Statements in the block. May be empty. |
| `span` | `SourceSpan` | Yes | Source location. |

---

#### 6.4.9 `TypedStmt::Return`

**Semantic function:** Returns a value from the enclosing function.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `value` | `Option<Box<TypedExpr>>` | No | The return value expression. |
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:**
- A `Return` node must appear inside a function body. If it does not, the type checker records error E2005.
- If the enclosing function has `return_type = Void`, `value` must be `None`. If `value` is `Some`, the type checker records error E2006.
- If the enclosing function has `return_type != Void`, `value` must be `Some`. If `value` is `None`, the type checker records error E2008.
- If `value` is `Some(expr)`, `expr.ty` must be assignable to the declared return type. If it is not, the type checker records error E2007.

---

#### 6.4.10 `TypedStmt::Break`

**Semantic function:** Exits the innermost enclosing loop.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:**
- A `Break` node must appear inside a loop body. If it does not, the type checker records error E2009.

---

#### 6.4.11 `TypedStmt::Continue`

**Semantic function:** Skips to the next iteration of the innermost enclosing loop.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `span` | `SourceSpan` | Yes | Source location. |

**Validity conditions:**
- A `Continue` node must appear inside a loop body. If it does not, the type checker records error E2010.

---

### 6.5 Operator Types

#### 6.5.1 `BinaryOp`

`BinaryOp` is an enum with the following 28 variants. The table groups them by class. The class determines the typing rules that apply (Section 8.5).

| Variant | Symbol | Class |
|---------|--------|-------|
| `Add` | `+` | Arithmetic |
| `AddEqual` | `+=` | Compound Arithmetic Assignment |
| `Subtract` | `-` | Arithmetic |
| `SubtractEqual` | `-=` | Compound Arithmetic Assignment |
| `Multiply` | `*` | Arithmetic |
| `MultiplyEqual` | `*=` | Compound Arithmetic Assignment |
| `Divide` | `/` | Arithmetic |
| `DivideEqual` | `/=` | Compound Arithmetic Assignment |
| `Modulo` | `%` | Arithmetic |
| `ModuloEqual` | `%=` | Compound Arithmetic Assignment |
| `Equal` | `==` | Comparison |
| `NotEqual` | `!=` | Comparison |
| `Less` | `<` | Comparison |
| `LessEqual` | `<=` | Comparison |
| `Greater` | `>` | Comparison |
| `GreaterEqual` | `>=` | Comparison |
| `And` | `&&` | Logical |
| `Or` | `\|\|` | Logical |
| `BitwiseAnd` | `&` | Bitwise |
| `BitwiseAndEqual` | `&=` | Compound Bitwise Assignment |
| `BitwiseOr` | `\|` | Bitwise |
| `BitwiseOrEqual` | `\|=` | Compound Bitwise Assignment |
| `BitwiseXor` | `^` | Bitwise |
| `BitwiseXorEqual` | `^=` | Compound Bitwise Assignment |
| `ShiftLeft` | `<<` | Shift |
| `ShiftLeftEqual` | `<<=` | Compound Shift Assignment |
| `ShiftRight` | `>>` | Shift |
| `ShiftRightEqual` | `>>=` | Compound Shift Assignment |

#### 6.5.2 `UnaryOp`

| Variant | Symbol | Valid side | Operand constraint | Result type |
|---------|--------|------------|-------------------|-------------|
| `Negate` | `-` | Prefix | Numeric | Same as operand |
| `Not` | `!` | Prefix | `Bool` | `Bool` |
| `BitwiseNot` | `~` | Prefix | Integer | Same as operand |
| `Increment` | `++` | Prefix or Postfix | Numeric, mutable variable | Same as operand |
| `Decrement` | `--` | Prefix or Postfix | Numeric, mutable variable | Same as operand |

---

## 7. Type System

### 7.1 Overview

The Descar type system is a nominal, static type system. Types are identified by name, not by structure. Type checking is performed at compile time.

The type system supports the following categories:
1. **Primitive integer types** (Section 7.2)
2. **Primitive floating-point types** (Section 7.3)
3. **Primitive non-numeric types** (Section 7.4)
4. **Composite types** (Section 7.5)
5. **User-defined types** (Section 7.6)
6. **Special types** (Section 7.7)

Generic or parameterized types are not supported in the current version.

### 7.2 Primitive Integer Types

| Type | Rust representation | Range |
|------|---------------------|-------|
| `Type::I8` | `i8` | −128 to 127 |
| `Type::I16` | `i16` | −32 768 to 32 767 |
| `Type::I32` | `i32` | −2 147 483 648 to 2 147 483 647 |
| `Type::I64` | `i64` | −9 223 372 036 854 775 808 to 9 223 372 036 854 775 807 |
| `Type::U8` | `u8` | 0 to 255 |
| `Type::U16` | `u16` | 0 to 65 535 |
| `Type::U32` | `u32` | 0 to 4 294 967 295 |
| `Type::U64` | `u64` | 0 to 18 446 744 073 709 551 615 |

All integer types are value types. Integer types are numeric types. All integer types are integer types for the purpose of bitwise operator constraints.

### 7.3 Primitive Floating-Point Types

| Type | Rust representation | Standard |
|------|---------------------|----------|
| `Type::F32` | `f32` | IEEE 754 single-precision (REF-04) |
| `Type::F64` | `f64` | IEEE 754 double-precision (REF-04) |

Floating-point types are numeric types. Floating-point types are **not** integer types. Floating-point types cannot be used with bitwise operators.

### 7.4 Primitive Non-Numeric Types

| Type | Description |
|------|-------------|
| `Type::Bool` | Boolean. Valid values: `true`, `false`. |
| `Type::Char` | A single Unicode character. Represented as a Rust `String` in the AST for Unicode compatibility. |
| `Type::String` | A UTF-8 string. |

`Bool`, `Char`, and `String` are not numeric types and are not integer types.

### 7.5 Composite Types

#### 7.5.1 `Type::Array`

A fixed-size, homogeneous sequence of elements.

| Field | Type | Description |
|-------|------|-------------|
| `element_type` | `Box<Type>` | The type of every element. |
| `size` | `Box<Expr>` | An expression that evaluates to a non-negative integer. This expression must be a constant integer literal at the time of type checking. |

**Validity conditions:**
- `element_type` must not be `Void` or `NullPtr`.
- `size` must evaluate to a positive integer. A size of zero is a semantic error (see OI-04 in Section 15 for the error code status).

**Type identity rule:** Two `Array` types are equal if and only if their `element_type` values are equal and their `size` expressions evaluate to the same non-negative integer value.

**Note on size evaluation:** The type checker calls `get_size` to extract the integer value from a size expression. If the expression is a non-literal, `get_size` returns `None` and array types are compared structurally (pointer equality of the `size` expression). This is a current limitation. See PASSUM-02 and OI-04 in Section 15.

#### 7.5.2 `Type::Vector`

A dynamically-sized, homogeneous sequence of elements.

| Field | Type | Description |
|-------|------|-------------|
| `element_type` | `Box<Type>` | The type of every element. |

**Validity conditions:** `element_type` must not be `Void` or `NullPtr`.

**Type identity rule:** Two `Vector` types are equal if and only if their `element_type` values are equal.

### 7.6 User-Defined Types

| Type | Description |
|------|-------------|
| `Type::Custom { name: Arc<str> }` | A named user-defined type. |

**Current limitation (ASM-09):** Custom types are stored by name but are not resolved to a concrete definition in the current type checker. Two `Custom` types with the same `name` value are considered equal. Assignment compatibility is not verified for custom types in the current version.

### 7.7 Special Types

| Type | Description | Usage |
|------|-------------|-------|
| `Type::Void` | The absence of a value. | Return type of functions that do not return a value. Not valid as a variable type or as an expression type. |
| `Type::NullPtr` | The null pointer type. | Type of the `nullptr` literal. Not assignable to any named type in the current version. |

### 7.8 Type Identity and Equivalence

The Descar type system uses **nominal equivalence**. Two types are equal if and only if they are the same `Type` variant with equal field values. This is the equality defined by `#[derive(PartialEq, Eq)]` on the `Type` enum.

The exception is `Type::Array` and `Type::Vector`, where element type equality is recursive and size evaluation is applied before comparison. This is a project-specific decision that extends structural equality to handle size expressions.

Structural equivalence (where two types are equal if they have the same shape, regardless of name) is not supported in the current version.

### 7.9 Numeric Type Hierarchy and Promotion

The promotion hierarchy is a total order on numeric types. It is used to determine the common type for binary operations with operands of different numeric types.

The hierarchy is (from widest to narrowest):

```
F64 > F32 > U64 > I64 > U32 > I32 > U16 > I16 > U8 > I8
```

**Promotion rule:** Given two numeric types `T1` and `T2`, the promoted type is the one that appears earlier in the hierarchy. If `T1 == T2`, the promoted type is `T1`.

This is a widening-only, implicit promotion. No narrowing conversion is implicit. (Dragon Book, Section 6.5 describes type widening.)

**Example:** `I32` and `F64` are promoted to `F64`.

### 7.10 Type Assignability

Type assignability determines when a value of type `T_src` can be stored in a variable of declared type `T_dst`.

The assignability rules are:

1. If `T_src == T_dst`, the assignment is valid.
2. If `T_src` and `T_dst` are both numeric and `T_src` is narrower than `T_dst` in the promotion hierarchy, the assignment is valid (widening assignment).
3. If `T_src` is `NullPtr`, the assignment is not valid for any declared type in the current version.
4. In all other cases, the assignment is not valid.

**Note:** The term "assignable" is used in `TypeChecker::is_assignable`. The exact implementation in `type_checker.rs` must be treated as the authoritative rule until this specification is confirmed against the implementation. This is provisional assumption PASSUM-01.

---

## 8. Typing Rules

### 8.1 Notation

Typing rules are written in a conventional judgement form:

```
Γ ⊢ e : T
```

This reads: "In environment Γ, expression `e` has type `T`."

- `Γ` is the type environment (the symbol table at the point of evaluation).
- `e` is an expression node.
- `T` is a `Type` value.
- `promote(T1, T2)` returns the promoted type of `T1` and `T2` according to Section 7.9.
- `assignable(T_src, T_dst)` is true when Section 7.10 permits the assignment.
- `integer(T)` is true when `T` is `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, or `U64`.
- `numeric(T)` is true when `T` is any type in the promotion hierarchy.
- `lookup_var(Γ, name)` returns the `VariableSymbol` for `name` in `Γ`, or error E2023.
- `lookup_fn(Γ, name)` returns the `FunctionSymbol` for `name` in `Γ`, or error.

(Dragon Book, Chapter 6 defines attribute grammars and type environments as the formal basis for these rules.)

### 8.2 Literal Typing Rule

```
Γ ⊢ Literal(v) : type_of_literal(v)
```

`type_of_literal` is the function defined in Section 6.2.1.

**Preconditions:** None.  
**Postconditions:** The resolved type is determined solely by the literal variant. No environment lookup is required.  
**Errors:** None. Literal typing always succeeds.

### 8.3 Variable Typing Rule

```
lookup_var(Γ, name) = sym
─────────────────────────────
Γ ⊢ Variable(name) : sym.ty
```

**Preconditions:** `name` must appear in `Γ` as a variable symbol.  
**Postconditions:** `ty = sym.ty`.  
**Errors:**
- E2022: If `name` is found in `Γ` as a function symbol, not a variable symbol.
- E2023: If `name` is not found in `Γ` at all.

### 8.4 Assignment Typing Rule

```
Γ ⊢ target : T_target    lookup_var(Γ, base(target)).mutable = true
Γ ⊢ value : T_value      assignable(T_value, T_target)
────────────────────────────────────────────────────────────────────
Γ ⊢ Assign(target, value) : T_target
```

**Preconditions:**
- `target` must be a `Variable` or `ArrayAccess` node.
- The base variable of `target` must be mutable.
- `T_value` must be assignable to `T_target`.

**Postconditions:** The resolved type of the assignment expression equals `T_target`.  
**Errors:**
- E2024: If the base variable of `target` is immutable.
- E2025: If `target` is a `Variable` and the name is not declared.

### 8.5 Binary Expression Typing Rules

#### 8.5.1 Arithmetic and Compound Arithmetic Assignment

```
Γ ⊢ e1 : T1    Γ ⊢ e2 : T2    numeric(T1)    numeric(T2)
T = promote(T1, T2)
──────────────────────────────────────────────────────────
Γ ⊢ Binary(e1, op_arith, e2) : T
```

where `op_arith` ∈ {`Add`, `AddEqual`, `Subtract`, `SubtractEqual`, `Multiply`, `MultiplyEqual`, `Divide`, `DivideEqual`, `Modulo`, `ModuloEqual`}.

**Preconditions:** Both operands must be numeric.  
**Postconditions:** The result type is the promoted type. `promoted_operand_type = T`.  
**Errors:**
- E2013: If the operands are not compatible after promotion.
- E2016: If an operand is not numeric.
- E2024: For compound assignment operators, if the base variable of `e1` is immutable.

#### 8.5.2 Comparison

```
Γ ⊢ e1 : T1    Γ ⊢ e2 : T2    numeric(T1)    numeric(T2)
T = promote(T1, T2)
──────────────────────────────────────────────────────────
Γ ⊢ Binary(e1, op_cmp, e2) : Bool
```

where `op_cmp` ∈ {`Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`}.

**Preconditions:** Both operands must be compatible (equal types after promotion, or same non-numeric type).  
**Postconditions:** The result type is always `Bool`.  
**Errors:**
- E2014: If the operands are not compatible.

#### 8.5.3 Logical

```
Γ ⊢ e1 : Bool    Γ ⊢ e2 : Bool
────────────────────────────────
Γ ⊢ Binary(e1, op_logic, e2) : Bool
```

where `op_logic` ∈ {`And`, `Or`}.

**Preconditions:** Both operands must be `Bool`.  
**Postconditions:** The result type is `Bool`.  
**Errors:**
- E2012: If an operand is not `Bool`.

#### 8.5.4 Bitwise and Shift

```
Γ ⊢ e1 : T1    Γ ⊢ e2 : T2    integer(T1)    integer(T2)
T = promote(T1, T2)
──────────────────────────────────────────────────────────
Γ ⊢ Binary(e1, op_bit, e2) : T
```

where `op_bit` ∈ {`BitwiseAnd`, `BitwiseAndEqual`, `BitwiseOr`, `BitwiseOrEqual`, `BitwiseXor`, `BitwiseXorEqual`, `ShiftLeft`, `ShiftLeftEqual`, `ShiftRight`, `ShiftRightEqual`}.

**Preconditions:** Both operands must be integer types.  
**Postconditions:** The result type is the promoted integer type.  
**Errors:**
- E2011: If an operand is not an integer type.

### 8.6 Unary Expression Typing Rules

#### 8.6.1 Numeric Negation

```
Γ ⊢ e : T    numeric(T)
────────────────────────
Γ ⊢ Unary(Negate, e) : T
```

**Errors:** E2018 if `T` is not numeric.

#### 8.6.2 Logical Not

```
Γ ⊢ e : Bool
──────────────────────────
Γ ⊢ Unary(Not, e) : Bool
```

**Errors:** E2019 if `T` is not `Bool`.

#### 8.6.3 Bitwise Not

```
Γ ⊢ e : T    integer(T)
────────────────────────
Γ ⊢ Unary(BitwiseNot, e) : T
```

**Errors:** E2018 if `T` is not an integer type.

#### 8.6.4 Increment and Decrement

```
Γ ⊢ e : T    numeric(T)    lookup_var(Γ, base(e)).mutable = true
────────────────────────────────────────────────────────────────
Γ ⊢ Unary(Increment|Decrement, e) : T
```

**Errors:** E2018 if `T` is not numeric. E2024 if the base variable is immutable.

### 8.7 Function Call Typing Rule

```
lookup_fn(Γ, name) = fn_sym
|args| = |fn_sym.parameters|
∀i: Γ ⊢ args[i] : T_i    assignable(T_i, fn_sym.parameters[i].type_annotation)
──────────────────────────────────────────────────────────────────────────────
Γ ⊢ Call(name, args) : fn_sym.return_type
```

**Preconditions:**
- `name` must be a declared function.
- Argument count must match parameter count.
- Each argument must be assignable to the corresponding parameter type.

**Postconditions:** The result type is the function's return type.  
**Errors:**
- E2023: If `name` is not a declared function.

### 8.8 Array Literal Typing Rule

```
|elements| ≥ 1
∀i: Γ ⊢ elements[i] : T
──────────────────────────────────────────────────────────────────────
Γ ⊢ ArrayLiteral(elements) : Array { element_type: T, size: len }
```

where `len` is the number of elements.

**Preconditions:** All elements must have the same type.  
**Errors:**
- E2020: If `elements` is empty.
- E2021: If elements have different types.

### 8.9 Array Access Typing Rule

```
Γ ⊢ arr : Array { element_type: T, size: _ }    Γ ⊢ idx : T_idx    integer(T_idx)
──────────────────────────────────────────────────────────────────────────────────
Γ ⊢ ArrayAccess(arr, idx) : T
```

**Preconditions:**
- `arr.ty` must be `Array` or `Vector`.
- `idx.ty` must be an integer type.

**Postconditions:** The result type is the element type of the array.

### 8.10 Return Typing Rule

```
return_type_stack.last() = T_expected
Γ ⊢ e : T_actual    assignable(T_actual, T_expected)    T_expected ≠ Void
──────────────────────────────────────────────────────────────────────────
Γ ⊢ Return(Some(e)) : ok
```

```
return_type_stack.last() = Void
────────────────────────────────
Γ ⊢ Return(None) : ok
```

**Errors:**
- E2005: Return outside function.
- E2006: Return with a value in a void function.
- E2007: Return type mismatch.
- E2008: Return with no value in a non-void function.

### 8.11 Condition Typing Rule (for `if`, `while`, `for`)

```
Γ ⊢ cond : Bool
────────────────────────────
cond is a valid condition
```

**Errors:** E2004 if `cond.ty` is not `Bool`.

### 8.12 Variable Declaration Typing Rule

```
Γ ⊢ init : T_init    assignable(T_init, T_annotation)
────────────────────────────────────────────────────
VarDeclaration(name, T_annotation, init) is valid
```

**Errors:**
- E2002: If `T_init` is not assignable to `T_annotation`.
- E2032: If `name` is already declared in the current scope.

---

## 9. Symbol Resolution and Semantic References

### 9.1 Symbol Table Structure

The symbol table is a stack of scopes. The bottom of the stack is always the global scope. Each new function or block pushes a new scope. The scope is popped when the function or block ends.

Three scope kinds exist:

| Scope kind | When pushed |
|------------|-------------|
| `Global` | Created at initialization. Never pushed or popped. |
| `Function` | Pushed when a `Function` or `MainFunction` body is entered. |
| `Block` | Pushed when a `Block`, `While`, or `For` body is entered. |

### 9.2 Symbol Lookup

Symbol lookup starts at the innermost (topmost) scope and moves outward to the global scope. The first scope that contains a symbol with the given name provides the result. This is lexical scoping. (Dragon Book, Section 2.7.)

A symbol in an inner scope hides a symbol with the same name in an outer scope. The outer symbol is not accessible while the inner symbol is in scope.

### 9.3 Symbol Declaration

Symbols are declared with `declare(name, symbol, declared_at)`. A declaration fails if a symbol with the same name is already present in the current scope. Declaration failure produces error E2032.

Function parameters are declared in the `Function` scope, not in the outer scope. This means that a parameter name can shadow an outer variable.

### 9.4 Symbol Reference Resolution in the FTAST

Each `TypedExpr::Variable` node in the FTAST carries a `SymbolRef` that contains a copy of the resolved `VariableSymbol`. This copy is made at the time of resolution. If the original symbol is later re-assigned, the `SymbolRef` is not updated. It reflects the state of the symbol at the point of the reference, not the point of any later assignment.

This is a project-specific design decision consistent with the value-based ownership model of Rust. See OI-05 in Section 15 for the open issue on whether `SymbolRef` should also carry the `last_assignment` field.

### 9.5 Function Reference Resolution in the FTAST

Each `TypedExpr::Call` node carries a `FunctionSymbolRef` that contains a copy of the resolved `FunctionSymbol`. This includes the parameter list and the return type.

### 9.6 Type Alias Resolution

`Symbol::TypeAlias(Type)` is supported in the symbol table but is not used by the type checker in the current version. Custom types are stored as `Type::Custom { name }` and are not expanded to their aliased type. This is confirmed assumption ASM-09.

---

## 10. Invariants and Validity Criteria

### 10.1 Structural Validity Invariants

A structurally valid FTAST satisfies all of the following:

| ID | Invariant | Checked at |
|----|-----------|------------|
| SV-01 | Every node has a non-null `SourceSpan`. | FTAST construction. |
| SV-02 | Every `TypedExpr` has a `ty` field (which may be `None` if a type error occurred). | FTAST construction. |
| SV-03 | `TypedStmt::Function.body` is always a `TypedStmt::Block`. | FTAST construction. |
| SV-04 | `TypedStmt::MainFunction.body` is always a `TypedStmt::Block`. | FTAST construction. |
| SV-05 | `TypedStmt::VarDeclaration.bindings` is not empty. | FTAST construction. |
| SV-06 | `TypedExpr::Assign.target` is a `Variable` or `ArrayAccess` node. | FTAST construction. |
| SV-07 | `TypedElseBranch::ElseIf` wraps a `TypedStmt::If` node. | FTAST construction. |
| SV-08 | The FTAST is a tree. There are no shared child references and no cycles. | FTAST construction (guaranteed by Rust ownership). |
| SV-09 | `TypedExpr::ArrayLiteral.elements` is not empty (or `ty` is `None`). | Type checker. |

**Required action when violated:** If a structural invariant is violated during FTAST construction, the compiler must emit an internal error and stop. These violations indicate a bug in the compiler, not an error in the source program.

### 10.2 Semantic Validity Invariants

A semantically valid FTAST satisfies all of the following:

| ID | Invariant | Checked at |
|----|-----------|------------|
| SM-01 | Every `TypedExpr` has a `ty` that is `Some(T)` for a valid `T`. | Type checker. |
| SM-02 | Every `TypedExpr::Variable` has a `symbol_ref` that is `Some(ref)` and `ty = Some(ref.ty)`. | Type checker. |
| SM-03 | Every `TypedExpr::Call` has a `function_ref` that is `Some(ref)` and `ty = Some(ref.return_type)`. | Type checker. |
| SM-04 | Every condition in `If`, `While`, and `For` has `ty = Some(Bool)`. | Type checker. |
| SM-05 | Every `Return` node is inside a function or `MainFunction` body. | Type checker. |
| SM-06 | Every `Break` and `Continue` node is inside a loop body. | Type checker. |
| SM-07 | Every `TypedStmt::VarDeclaration` declares only names that are not already declared in the current scope. | Type checker. |
| SM-08 | The initializer type of each `VarBinding` is assignable to the declared type. | Type checker. |
| SM-09 | Every `TypedStmt::Function` with a non-void return type has a return path on all control-flow branches. | Type checker. |
| SM-10 | Every binary expression with a bitwise or shift operator has operands of integer type. | Type checker. |
| SM-11 | Every binary expression with a logical operator has operands of type `Bool`. | Type checker. |
| SM-12 | Every assignment to a variable targets a mutable variable. | Type checker. |

**Required action when violated:** If a semantic invariant is violated, the type checker records a `CompileError::TypeError` with the appropriate error code and continues checking. The FTAST is produced with `None` in the `ty` field of the affected node. IR generation must not proceed if any `ty` field is `None`.

### 10.3 Fully Typed Invariant

An FTAST is **fully typed** if and only if:

1. It satisfies all structural invariants (Section 10.1).
2. It satisfies all semantic invariants (Section 10.2).
3. Every `TypedExpr.ty` is `Some(T)` for a concrete, non-`Void` `Type` value, with the following exceptions:
   - `TypedExpr::Call` nodes where `function_ref.return_type == Void` have `ty = Some(Void)`.
   - Nodes for which no type can be assigned because of a source error have `ty = None`.

An FTAST is **ready for IR generation** if and only if it is fully typed and no `ty` field is `None`.

### 10.4 Incomplete or Invalid FTAST

An FTAST is **incomplete** if any `ty` field is `None`. An incomplete FTAST indicates that the source program contains at least one semantic error.

An FTAST is **invalid** if any structural invariant is violated. An invalid FTAST indicates a compiler implementation bug.

IR generation must not start if the FTAST is incomplete or invalid.

---

## 11. Error Handling

### 11.1 Error Categories

The Descar compiler uses a single `CompileError` enum with the following variants relevant to FTAST construction and type checking:

| Variant | Phase | Error code range |
|---------|-------|-----------------|
| `LexerError` | Lexical analysis | E0001–E0999 |
| `SyntaxError` | Parsing | E1001–E1999 |
| `TypeError` | Semantic analysis | E2001–E2999 |
| `IoError` | I/O | No code |

This specification covers only `TypeError` errors. `LexerError` and `SyntaxError` occur before the FTAST is constructed.

### 11.2 Semantic Error Codes

The following table lists all confirmed E2xxx error codes that the type checker produces.

| Code | Name | Condition |
|------|------|-----------|
| E2002 | Type assign mismatch | Initializer type is not assignable to declared type. |
| E2003 | Missing return | Non-void function may not return a value on all paths. |
| E2004 | Non-bool condition | Condition expression does not have type `Bool`. |
| E2005 | Return outside function | `return` appears outside a function body. |
| E2006 | Return value in void | `return` with a value appears in a void function. |
| E2007 | Return type mismatch | Return expression type is not assignable to declared return type. |
| E2008 | Missing return value | `return` without a value appears in a non-void function. |
| E2009 | Break outside loop | `break` appears outside a loop body. |
| E2010 | Continue outside loop | `continue` appears outside a loop body. |
| E2011 | Bitwise on non-integer | Bitwise or shift operator applied to a non-integer operand. |
| E2012 | Logical on non-bool | Logical operator `&&` or `\|\|` applied to a non-bool operand. |
| E2013 | Arithmetic type mismatch | Arithmetic operator applied to incompatible types. |
| E2014 | Comparison type mismatch | Comparison operator applied to incompatible types. |
| E2016 | Arithmetic on non-numeric | Arithmetic operator applied to a non-numeric type. |
| E2017 | Logical on non-bool (binary) | Logical operator applied to a non-bool operand (second check). |
| E2018 | Numeric unary mismatch | Unary numeric operator applied to a non-numeric or non-integer type. |
| E2019 | Logical not on non-bool | Unary `!` applied to a non-bool type. |
| E2020 | Empty array literal | Array literal contains zero elements. |
| E2021 | Mixed array elements | Array literal contains elements of different types. |
| E2022 | Function used as variable | A function name is used where a variable is expected. |
| E2023 | Undefined variable | A variable name is not found in any accessible scope. |
| E2024 | Assign to immutable | An assignment targets an immutable variable. |
| E2025 | Undefined variable in assignment | The target variable of an assignment is not declared. |
| E2032 | Identifier redeclaration | A name is declared more than once in the same scope. |

### 11.3 Diagnostic Information

Each `TypeError` carries:

| Field | Type | Description |
|-------|------|-------------|
| `code` | `Option<ErrorCode>` | Standardized error code. |
| `message` | `Arc<str>` | Human-readable description of the error. |
| `span` | `SourceSpan` | Source location where the error was detected. |
| `help` | `Option<String>` | Optional guidance for fixing the error. |

The `span` field must reference the tightest enclosing source region that contains the cause of the error. For example, E2004 must reference the span of the condition expression, not the span of the entire `if` statement.

### 11.4 Error Recovery

The type checker uses the following recovery strategy:

1. When a type error is detected, the checker records the error and continues.
2. The affected expression node receives `ty = None`.
3. Sub-expressions of the affected node are still visited if they are structurally accessible.
4. If an expression has `ty = None`, any parent expression that depends on the type of that expression also receives `ty = None`.

This strategy maximizes the number of errors reported in a single compilation pass. (Dragon Book, Section 4.8 discusses error recovery in parsing; the same principle applies to semantic analysis.)

### 11.5 Conditions That Stop Compilation

Compilation stops before IR generation if either of the following conditions is true:

1. The FTAST contains at least one node with `ty = None` (incomplete FTAST).
2. A structural invariant is violated (invalid FTAST).

Compilation does **not** stop during type checking. The type checker always completes its traversal to collect all errors.

### 11.6 Source Program Errors vs. Internal Errors

A **source program error** is an error caused by invalid source code written by the user. Source program errors produce `CompileError::TypeError` values and are reported to the user with a message, span, and optional help.

An **internal error** is an error caused by a bug in the compiler implementation. Examples include a structural invariant violation or an unexpected `None` in a field that must always be `Some`. Internal errors must be reported separately from source program errors, must not use the E2xxx error code range, and must include information useful for debugging the compiler. The current implementation uses `panic!` or `expect` for internal errors.

---

## 12. Operations and Interfaces

### 12.1 Node Construction

**Operation:** Build a `TypedExpr` or `TypedStmt` node.

**Inputs:** The node variant, field values.  
**Outputs:** A new node value.  
**Preconditions:** All required fields must be provided. Optional fields may be `None`.  
**Postconditions:** The node satisfies all structural validity conditions for its variant.  
**Errors:** If a required field is missing, the construction fails. This is an internal error.

**Mandatory:** Yes.

---

### 12.2 Tree Traversal

**Operation:** Visit every node in the FTAST in a defined order.

Two traversal orders are relevant:

1. **Depth-first, pre-order:** Visit a node before its children. Used for scope management (push scope before visiting children).
2. **Depth-first, post-order:** Visit a node after its children. Used for type inference (compute child types before parent type).

The type checker uses post-order traversal for expression nodes and pre-order traversal for scope management.

**Inputs:** The root node sequence (`Vec<TypedStmt>`), a visitor function or trait object.  
**Outputs:** Depends on the visitor.  
**Preconditions:** The FTAST is structurally valid.  
**Postconditions:** Every node is visited exactly once.  
**Errors:** None, unless the visitor function fails.

**Mandatory:** Yes.

---

### 12.3 Symbol Lookup and Resolution

**Operation:** Resolve a name to a symbol in the current scope chain.

**Inputs:** A symbol name (`&str`), the current `SymbolTable`.  
**Outputs:** `Option<Symbol>`. `None` if not found.  
**Preconditions:** The symbol table is in a valid state (at least one scope exists).  
**Postconditions:** The returned symbol is the innermost matching symbol. The symbol table is not modified.  
**Errors:** None. The caller is responsible for handling `None`.

**Mandatory:** Yes.

---

### 12.4 Structural Validation

**Operation:** Verify that the FTAST satisfies all structural invariants.

**Inputs:** The root node sequence.  
**Outputs:** A list of violated invariants (empty if valid).  
**Preconditions:** None.  
**Postconditions:** If the output is empty, the FTAST satisfies all structural invariants in Section 10.1.  
**Errors:** None. The validator reports violations; it does not stop.

**Mandatory:** Yes. This operation must run before semantic validation.

---

### 12.5 Type Checking

**Operation:** Check all typing rules. Produce a fully typed FTAST or a list of errors.

**Inputs:** A Parser AST (`Vec<Stmt>`).  
**Outputs:** A tuple `(Vec<TypedStmt>, Vec<CompileError>)`. If the error list is empty, the FTAST is complete and ready for IR generation.  
**Preconditions:** The Parser AST is structurally valid.  
**Postconditions:**
- Every `TypedExpr.ty` field is set (to `Some(T)` or `None`).
- Every detected error is recorded in the error list.
- The error list is empty if and only if the FTAST satisfies all semantic invariants.

**Observable effects:** None outside the return value.  
**Errors:** Internal errors are panics. Source errors are returned in the error list.

**Mandatory:** Yes.

---

### 12.6 Node Transformation

**Operation:** Rewrite one or more nodes in the FTAST without changing its meaning.

Examples: constant folding, dead code elimination, normalization.

**Inputs:** A node or subtree, a transformation function.  
**Outputs:** A new node or subtree.  
**Preconditions:** The input node is structurally valid.  
**Postconditions:** The output node is structurally valid and semantically equivalent to the input.  
**Errors:** If a transformation produces a structural invariant violation, it is an internal error.

**Mandatory:** Optional. Defined for use by optimization passes.

---

### 12.7 Diagnostic Generation

**Operation:** Produce a human-readable diagnostic message from a `CompileError`.

**Inputs:** A `CompileError`, the source file text (for context).  
**Outputs:** A formatted diagnostic string.  
**Preconditions:** The `CompileError` has a non-null `span`.  
**Postconditions:** The output contains the error code (if present), the message, and the source location.  
**Errors:** None.

**Mandatory:** Yes.

---

### 12.8 Operations Not in Scope for FTAST v1.0

The following operations are recognized but are not required for FTAST version 1.0:

| Operation | Reason for exclusion |
|-----------|---------------------|
| Serialization and deserialization | No current requirement for persisting the FTAST to disk. |
| AST comparison (structural equality) | `#[derive(PartialEq, Eq)]` provides this for testing. No separate operation is needed. |
| Data cloning | Provided by `#[derive(Clone)]`. |

---

## 13. Verification and Acceptance Criteria

### 13.1 Requirements Identifiers

Requirements defined in this document use the following identifier format:

- `REQ-AST-NNN`: A requirement about the node model.
- `REQ-TYPE-NNN`: A requirement about the type system.
- `REQ-RULE-NNN`: A requirement about a typing rule.
- `REQ-INV-NNN`: A requirement about an invariant.
- `REQ-ERR-NNN`: A requirement about error handling.
- `REQ-OP-NNN`: A requirement about an operation.

### 13.2 Node Model Requirements

| ID | Requirement | Verification method |
|----|-------------|---------------------|
| REQ-AST-001 | Every `TypedExpr` variant carries a field `ty: Option<Type>`. | Inspect the Rust type definition. |
| REQ-AST-002 | Every `TypedExpr` and `TypedStmt` variant carries a field `span: SourceSpan`. | Inspect the Rust type definition. |
| REQ-AST-003 | `TypedExpr::Variable` carries a `symbol_ref: Option<SymbolRef>` field. | Inspect the Rust type definition. |
| REQ-AST-004 | `TypedExpr::Call` carries a `function_ref: Option<FunctionSymbolRef>` field. | Inspect the Rust type definition. |
| REQ-AST-005 | `TypedExpr::Binary` carries a `promoted_operand_type: Option<Type>` field. | Inspect the Rust type definition. |
| REQ-AST-006 | The FTAST is a tree with no cycles and no shared child references. | Verified by the Rust borrow checker: no `Rc<>` or `Arc<>` wrapping node types. |

### 13.3 Type System Requirements

| ID | Requirement | Verification method |
|----|-------------|---------------------|
| REQ-TYPE-001 | All 19 `Type` variants (I8, I16, I32, I64, U8, U16, U32, U64, F32, F64, Char, String, Bool, Custom, Array, Vector, Void, NullPtr, and future additions) are represented. | Inspect the `Type` enum. |
| REQ-TYPE-002 | `Type::Array` contains an `element_type: Box<Type>` and a `size: Box<Expr>`. | Inspect the `Type` enum. |
| REQ-TYPE-003 | Two `Array` types with equal element types and equal size values are equal. | Unit test: construct two `Type::Array` with the same element type and a `Number::Integer(5)` size expression. Assert that `is_same_type` returns `true`. |
| REQ-TYPE-004 | The promotion hierarchy is exactly: F64 > F32 > U64 > I64 > U32 > I32 > U16 > I16 > U8 > I8. | Unit test: for every pair (T1, T2) in the hierarchy, assert that `promote(T1, T2)` returns the wider type. |
| REQ-TYPE-005 | Widening assignment is valid: a value of type `T_narrow` can be assigned to a variable of type `T_wide` where `T_narrow` is narrower in the hierarchy. | Unit test: declare `var x: i64` and assign an `i32` literal. Assert no E2002 error. |
| REQ-TYPE-006 | Narrowing assignment is not valid: a value of type `T_wide` cannot be assigned to a variable of type `T_narrow` where `T_wide` is wider. | Unit test: declare `var x: i8` and assign an `i64` literal. Assert E2002 error. |

### 13.4 Typing Rule Requirements

| ID | Requirement | Verification method |
|----|-------------|---------------------|
| REQ-RULE-001 | Literal nodes are typed without environment lookup. | Unit test: construct `Literal(Bool(true))` and check `ty = Some(Bool)`. |
| REQ-RULE-002 | `Variable` nodes are typed by symbol table lookup. | Unit test: declare a variable `x: i32`, then use `x` in an expression. Assert `ty = Some(I32)`. |
| REQ-RULE-003 | A `Variable` node for an undeclared name has `ty = None` and produces E2023. | Unit test: use an undeclared name. Assert `ty = None` and one E2023 error. |
| REQ-RULE-004 | Arithmetic binary expressions produce the promoted type. | Unit test: `1i8 + 1i32`. Assert result type is `I32`. |
| REQ-RULE-005 | Comparison binary expressions always produce `Bool`. | Unit test: `1i32 < 2i32`. Assert result type is `Bool`. |
| REQ-RULE-006 | Logical binary expressions require `Bool` operands and produce `Bool`. | Unit test: `true && false`. Assert result type is `Bool`. |
| REQ-RULE-007 | Bitwise binary expressions require integer operands. | Unit test: `1.0f32 & 2.0f32`. Assert E2011 error. |
| REQ-RULE-008 | A condition in `if` that is not `Bool` produces E2004. | Unit test: `if (1i32) {}`. Assert E2004 error. |
| REQ-RULE-009 | A `return` outside a function produces E2005. | Test file: `dr_files/return_outside_function.dr`. Assert E2005 error. |
| REQ-RULE-010 | A non-void function without a return path produces E2003. | Test file: `dr_files/return_missing_value.dr`. Assert E2003 error. |
| REQ-RULE-011 | Assignment to an immutable variable produces E2024. | Test file: `dr_files/test_const_assign.dr`. Assert E2024 error. |
| REQ-RULE-012 | Assignment type mismatch produces E2002. | Unit test: `var x: i32 = true`. Assert E2002 error. |

### 13.5 Invariant Requirements

| ID | Requirement | Verification method |
|----|-------------|---------------------|
| REQ-INV-001 | A fully typed FTAST has no `ty = None` fields. | Run the type checker on a valid program. Assert that the returned error list is empty and all `ty` fields are `Some`. |
| REQ-INV-002 | An FTAST with at least one `ty = None` field is not ready for IR generation. | Unit test: produce an FTAST with one E2023 error. Assert that the FTAST is marked as not ready. |
| REQ-INV-003 | `TypedStmt::Function.body` is always a `Block`. | Inspect all parser tests. Assert no other statement kind is used as a function body. |
| REQ-INV-004 | `TypedStmt::VarDeclaration.bindings` is never empty. | Unit test: attempt to construct a `VarDeclaration` with an empty `bindings` list. Assert internal error. |

### 13.6 Error Handling Requirements

| ID | Requirement | Verification method |
|----|-------------|---------------------|
| REQ-ERR-001 | Each error in the E2xxx range maps to exactly one `ErrorCode` variant. | Inspect the `ErrorCode` enum and cross-reference with Section 11.2. |
| REQ-ERR-002 | Each error carries a `span` that references the tightest source region containing the cause. | Unit tests for each error code: assert that the span in the error matches the expected source region. |
| REQ-ERR-003 | The type checker continues after an error and collects all errors in a single pass. | Unit test: write a program with three independent errors. Assert that all three errors are returned. |
| REQ-ERR-004 | IR generation does not start if any error is present. | Integration test: compile a file with a known error. Assert no IR output is produced. |

### 13.7 Operation Requirements

| ID | Requirement | Verification method |
|----|-------------|---------------------|
| REQ-OP-001 | `TypeChecker::check` accepts `&[Stmt]` and returns `Vec<CompileError>`. | Inspect the function signature. |
| REQ-OP-002 | `TypeChecker::check` on a valid program returns an empty error list. | Test file: `dr_files/simple_test.dr`. Assert no errors. |
| REQ-OP-003 | `SymbolTable::lookup` searches from innermost scope outward. | Unit test: declare `x` in outer scope, declare another `x` in inner scope, assert that lookup inside inner scope returns the inner `x`. |
| REQ-OP-004 | `SymbolTable::declare` returns E2032 for a name already declared in the current scope. | Unit test: declare `x` twice in the same scope. Assert E2032 error. |

### 13.8 Acceptance Criteria

A requirement is met when:

1. The implementation passes all stated unit tests without manual interpretation.
2. The test is automated and can be run with `cargo test --workspace`.
3. No test result requires subjective judgment.

A requirement is **not** met if:

- The test fails.
- The test does not exist.
- The test requires manual inspection to determine pass or fail.

---

## 14. Technical Examples

### 14.1 Example: Simple Function with Return Type

**Source (Descar):**

```
fun add(a: i32, b: i32): i32 {
    return a + b
}
```

**FTAST representation (conceptual):**

```
TypedStmt::Function {
    name: "add",
    parameters: [
        Parameter { name: "a", type_annotation: I32, span: ... },
        Parameter { name: "b", type_annotation: I32, span: ... }
    ],
    return_type: I32,
    body: TypedStmt::Block {
        statements: [
            TypedStmt::Return {
                value: Some(
                    TypedExpr::Binary {
                        left: TypedExpr::Variable {
                            name: "a",
                            symbol_ref: Some(SymbolRef { name: "a", ty: I32, mutable: true, ... }),
                            ty: Some(I32),
                            span: ...
                        },
                        op: Add,
                        right: TypedExpr::Variable {
                            name: "b",
                            symbol_ref: Some(SymbolRef { name: "b", ty: I32, mutable: true, ... }),
                            ty: Some(I32),
                            span: ...
                        },
                        promoted_operand_type: Some(I32),
                        ty: Some(I32),
                        span: ...
                    }
                ),
                span: ...
            }
        ],
        span: ...
    },
    span: ...
}
```

**Type derivation:**
1. `a` resolves to `I32` via symbol lookup.
2. `b` resolves to `I32` via symbol lookup.
3. `a + b`: both operands are numeric, `promote(I32, I32) = I32`. Result type: `I32`.
4. `return a + b`: `I32` is assignable to declared return type `I32`. Valid.

---

### 14.2 Example: Type Promotion in Binary Expression

**Source:**

```
var result: f64 = 1i32 + 2.5
```

**Type derivation:**
1. `1i32` has type `I32`.
2. `2.5` (untyped float literal) has type `F64` (default float type is `F64`, parsed as `Number::Float64`).
3. `promote(I32, F64) = F64`.
4. `promoted_operand_type = F64`. Result type of `+` is `F64`.
5. `F64` is assignable to declared type `F64`. Valid.

---

### 14.3 Example: Type Error — Immutable Assignment

**Source:**

```
var x: i32 = 10i32
x = 20i32
```

**Type derivation:**
1. `x` is declared as immutable (`var`, not `mut var`).
2. `x = 20i32`: target is `Variable("x")`. Lookup finds `x` with `mutable = false`.
3. Type checker records E2024.
4. `TypedExpr::Assign.ty = None`.

---

### 14.4 Example: Array Literal

**Source:**

```
var primes: i64[5] = {2, 3, 5, 7, 11}
```

**Type derivation:**
1. All elements are untyped integer literals. Each is parsed as `Number::Integer(_)`, which has type `I64`.
2. All element types are equal: `I64`.
3. Array literal type: `Array { element_type: I64, size: Literal(Integer(5)) }`.
4. Declared type: `I64[5]` = `Array { element_type: I64, size: 5 }`.
5. `is_same_type` evaluates the size expressions: both evaluate to `5`. Types are equal. Valid.

---

### 14.5 Example: Error — Bitwise Operator on Float

**Source (from `dr_files/bitwise_type_mismatch.dr`):**

```
var a: f32 = 1.0
var b: f32 = 2.0
var c: f32 = a & b
```

**Type derivation:**
1. `a` resolves to `F32`.
2. `b` resolves to `F32`.
3. `&` is a bitwise operator. It requires integer operands.
4. `integer(F32) = false`. Type checker records E2011.
5. `TypedExpr::Binary.ty = None`.
6. `c` cannot be assigned. The `VarDeclaration` for `c` records E2002 (or leaves `ty = None`).

---

### 14.6 Example: Scope Shadowing

**Source:**

```
var x: i32 = 1i32
fun foo(): i32 {
    var x: i64 = 2
    return x
}
```

**Symbol resolution:**
1. Outer scope: `x → VariableSymbol { ty: I32, mutable: false }`.
2. `foo` creates a function scope.
3. Inner scope: `x → VariableSymbol { ty: I64, mutable: false }`. This shadows the outer `x`.
4. Inside `foo`, `x` resolves to `I64`.
5. `return x`: `I64` is assignable to declared return type `I64`. Valid.
6. The outer `x: I32` is not affected.

---

## 15. Open Issues and Design Decisions

This section records decisions and issues that affect the specification. Each entry has a unique identifier, a status, and a description.

### 15.1 Open Issues

| ID | Status | Description | Affected requirements |
|----|--------|-------------|----------------------|
| OI-01 | **Open** | Should the FTAST be a separate set of types from the Parser AST, or should the Parser AST types be extended with optional type fields? A separate FTAST avoids changing the parser output type but increases code size. Extending the Parser AST is simpler but couples the parser and the type checker. | REQ-AST-001 through REQ-AST-006, PASSUM-01 |
| OI-02 | **Open** | Should `TypedExpr::Binary` carry a `promoted_operand_type` field? This field is required by IR generation to insert widening conversions. However, it is an architectural assumption. If IR generation can re-derive the promoted type from the operand types, the field may not be needed. | REQ-AST-005 |
| OI-03 | **Open** | There is no confirmed error code for function call argument count mismatch. The type checker may silently produce a wrong result or panic. An E2xxx code must be assigned. | REQ-RULE requirement for `Call` |
| OI-04 | **Open** | What is the behavior for an array with a zero size expression? The specification states it is a semantic error, but no E2xxx code is assigned in the current implementation. | REQ-TYPE-002 |
| OI-05 | **Open** | Should `SymbolRef` carry the `last_assignment: Option<SourceSpan>` field from `VariableSymbol`? This field is useful for definite assignment analysis in future versions. It is not required for the current type checker. | REQ-AST-003 |

### 15.2 Confirmed Design Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| DD-01 | The type checker uses a single-pass depth-first traversal. | This is sufficient for the current language, which has no forward references. Functions must be declared before use. See Section 5.1 and ASM-03. |
| DD-02 | The type checker continues after errors. | This maximizes the number of errors reported in a single compilation. The user does not need to fix errors one at a time. See Section 11.4. |
| DD-03 | `Option<Type>` is used for unresolved types instead of an `ErrorType` variant. | Adding an `ErrorType` variant to `Type` would propagate errors through all code that matches on `Type`. Using `Option<Type>` keeps error representation external to the type. |
| DD-04 | Numeric type promotion is widening-only and implicit. | This follows the principle of least surprise for arithmetic expressions. The developer confirms this. See Section 7.9 and Dragon Book, Section 6.5. |
| DD-05 | Custom type resolution is deferred. | Custom types are parsed and stored, but type resolution (checking that the custom type is declared) is not implemented in the current version. This is ASM-09. |
| DD-06 | `MainFunction` is a distinct node from `Function`. | The `main` block has no declared return type and no parameter list in Descar syntax. A distinct node avoids special-case logic in the type checker and IR generator. |

---

## 16. Bibliographic References

| ID | Reference |
|----|-----------|
| REF-01 | Aho, A.V., Lam, M.S., Sethi, R., Ullman, J.D. *Compilers: Principles, Techniques, and Tools*, Second Edition. Addison-Wesley, 2007. ISBN: 0-321-48681-1. |
| REF-02 | ASD-STE100 Issue 7. *Simplified Technical English: Specification for the Preparation of Maintenance Documentation*. AeroSpace and Defence Industries Association of Europe, 2017. |
| REF-03 | Descar compiler source code. Repository: `https://github.com/Giuseppe-Bianc/Descar`. Crate: `descar-core`. Referenced files: `src/syntax/ast/`, `src/semantic/`, `src/error/`. |
| REF-04 | IEEE 754-2019. *IEEE Standard for Floating-Point Arithmetic*. IEEE, 2019. DOI: 10.1109/IEEESTD.2019.8766229. |

---

*End of document DESCAR-SPEC-AST-001 v1.0.0*
