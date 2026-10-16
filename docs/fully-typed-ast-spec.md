# Descar Surface AST and Fully Typed AST Specification

**Status:** Design specification. Repository facts are marked **Confirmed**. Decisions that guide a future implementation are marked **Specified**. Decisions that need language-owner approval are marked **Open**.

## 1. Purpose and Scope

This document specifies two related representations for Descar:

1. The **Surface AST** represents source syntax after parsing.
2. The **Fully Typed AST** represents the same program after name resolution and type checking.

The Fully Typed AST preserves source constructs and source locations. It adds resolved symbol identity, type identity, and explicit conversions. It is the input contract for later intermediate representation (IR) lowering. It is not itself IR.

The specification is repository-grounded. It defines current language constructs and states a controlled extension path. It does not confirm unsupported language features as part of the language. In particular, it does not define structures, enums, generics, unions, optional values, reference or pointer operations, closures, modules, or method calls. The current `NullPtr` type does not by itself confirm pointer values or pointer operations.

## 2. Applicable Standards and References

### 2.1 Theoretical basis

The compiler model follows these principles from Aho, Lam, Sethi, and Ullman:

- A syntax tree represents the hierarchical structure of a source construct. The parser can construct this structure from grammar productions.
- Semantic analysis associates information with syntax-tree constructs. Symbol tables support name and scope information.
- Type checking checks operation constraints and can report semantic errors.
- Syntax-directed translation can construct a later representation from syntax and semantic attributes. The book treats syntax-tree variants, types, declarations, and type checking in Chapter 6.
- Intermediate code is a separate compiler representation. It is appropriate to lower the typed tree into IR as a later phase.

These are theoretical principles. The two-tree design, field names, IDs, type categories, and validation rules below are project-specific decisions.

### 2.2 Bibliographic references

1. Alfred V. Aho, Monica S. Lam, Ravi Sethi, and Jeffrey D. Ullman. *Compilers: Principles, Techniques, and Tools*. 2nd ed. Addison-Wesley, 2006. Chapter 2, “A Simple Syntax-Directed Translator”; Chapter 5, “Syntax-Directed Translation”; Chapter 6, “Intermediate-Code Generation,” especially Sections 6.1 “Variants of Syntax Trees,” 6.3 “Types and Declarations,” and 6.5 “Type Checking”; Chapter 7, “Run-Time Environments”; Chapter 8, “Code Generation.” No page numbers are given.
2. ASD-STE100. *Simplified Technical English*. Issue 9, January 2025. Part 1 contains writing rules. This specification uses short sentences, defined terms, and verifiable requirements. It does not claim formal certification against the complete controlled vocabulary.

## 3. Terms and Definitions

| Term | Definition |
|---|---|
| Node | One record in an AST representation. |
| Node ID | Unique identity for one node within one AST instance. |
| Symbol ID | Unique identity for one declaration within one compilation unit. |
| Type ID | Unique identity for one resolved type within one compilation unit. |
| Surface AST | Parser output that records source constructs and written type annotations. |
| Fully Typed AST | Resolved AST where every expression and declaration with semantic type has a valid type, every reference points to a declaration, and each conversion is explicit. |
| Declared type | Type written in source or assigned by a language-defined implicit declaration rule. |
| Inferred type | Type selected by semantic analysis where source does not write a type. |
| Result type | Type produced by evaluating an expression after operand conversions. |
| Conversion | A typed operation that changes a value from one type to another. |
| Error type | Recovery marker used after a source semantic error. It is not a source-language type. |
| Incomplete AST | AST with parser recovery holes or unresolved semantic data. It is not valid for code generation. |

The symbol `Γ` means the active lexical environment. `Γ(x)` is the nearest visible declaration of name `x`. `Γ ⊢ e : T` means expression `e` has type `T` in environment `Γ`. `T₁ ≡ T₂` means type equivalence. `T₁ → T₂` means conversion from `T₁` to `T₂` is allowed by an explicit conversion rule.

## 4. Assumptions and Constraints

| ID | Status | Requirement or assumption |
|---|---|---|
| BAS-001 | Confirmed | The implementation language is Rust and the compiler core is in `descar-core`. |
| BAS-002 | Confirmed | Source currently has literals, variables, assignments, calls, array literals and indexing, unary and binary expressions, declarations, functions, `main`, blocks, conditionals, loops, returns, `break`, and `continue`. |
| BAS-003 | Confirmed | The internal type model has numeric types `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, and `f64`; also `char`, `string`, `bool`, custom names, fixed arrays, vectors, `void`, and `nullptr`. The current parser does not accept every internal type as source syntax. |
| BAS-004 | Confirmed | Current semantic analysis uses nested lexical scopes and reports semantic errors with source spans and error codes. |
| BAS-005 | Confirmed | Current numeric promotion uses the ordered list `f64 > f32 > u64 > i64 > u32 > i32 > u16 > i16 > u8 > i8`. This order does not establish overflow, precision, or runtime conversion behavior. |
| BAS-006 | Specified | Surface AST and Fully Typed AST are separate types. A successful semantic pass produces the latter. |
| BAS-007 | Specified | Error recovery may produce an incomplete Surface AST. Error nodes are not accepted by the Fully Typed AST success state. |
| BAS-008 | Specified | Forward-looking features remain outside the confirmed language until their grammar and semantics are approved. The representation must use closed enums or explicit extension points so unsupported categories cannot silently enter the typed tree. |
| BAS-009 | Open | Confirm whether type aliases and custom types have declarations in the language grammar. The parser accepts a custom identifier as a type name. The symbol table has a `TypeAlias` kind. The inspected AST has no type declaration node and the checker does not resolve custom type declarations. |
| BAS-010 | Open | Confirm whether the current compatibility of `nullptr` with arrays, vectors, and custom names is intended. The checker accepts those assignments, although the language has no pointer type or pointer operation. |
| BAS-011 | Open | Confirm whether numeric promotion order and the separate assignment-conversion table are intended language rules. Define signed/unsigned mixing, float precision, overflow, narrowing, and runtime conversion behavior. |
| BAS-012 | Open | Confirm whether fixed-array length must be a compile-time integer constant and whether length is part of type identity. The parser stores an expression. The checker can evaluate only a non-negative integer literal for array compatibility. |
| BAS-013 | Open | Confirm whether function declarations are visible before their source position. The checker visits declarations in source order and registers each function before checking its body. This permits self-recursion, but does not make later functions visible to earlier function bodies. |
| BAS-014 | Open | Confirm whether function parameters are mutable and whether a function call accepts only a named function. The checker registers parameters as mutable variable symbols and accepts only a variable expression as a callee. |
| BAS-015 | Specified | The Fully Typed AST is the semantic input to IR lowering. The README describes IR output, but the inspected `descar-core` source has no IR or lowering module. This is a target architecture decision, not a confirmed current pipeline. |

### 4.1 Repository behavior that informs the design

The following facts describe current code. They do not automatically define the future language contract.

| ID | Current behavior | Design consequence |
|---|---|---|
| OBS-001 | Variable declarations require a type annotation and an initializer in the parser. The AST stores one shared annotation and a list of name/initializer pairs. | `TypeOrigin::Inferred` is a future extension. The current Surface AST should preserve a required annotation until inference syntax is approved. |
| OBS-002 | Function parameters require annotations. A missing function return annotation becomes `void`. | Keep return annotation presence in the Surface AST so omission remains distinguishable from explicit syntax. Resolve omission to `void` in the typed function signature. |
| OBS-003 | The parser accepts primitive numeric, `char`, `string`, and `bool` type tokens; identifier-named types; array suffixes; and `vector<T>`. Its type parser does not accept `void` or `nullptr` tokens. | Do not claim every `Type` enum variant is currently expressible in source. Reject impossible source forms or mark them as synthetic types. |
| OBS-004 | Array type syntax stores an arbitrary expression as its size. `get_size` evaluates only non-negative integer literals. | The typed type table must store a validated constant length. A non-literal expression currently cannot establish array assignability. |
| OBS-005 | Array literals use braces, must be non-empty for inference, and require exactly equal element types. | The current rule does not promote mixed numeric elements. TYP-015 shall require identical element types unless a future language decision changes this behavior. |
| OBS-006 | Numeric binary operations choose the earliest type in the promotion hierarchy. Assignment, return, and call arguments use a separate explicit assignability relation. | The type checker must not treat promotion and assignment compatibility as one relation. |
| OBS-007 | Assignment compatibility accepts numeric widening pairs, `char` to `string`, `nullptr` to custom/array/vector types, recursively assignable vectors, and arrays with assignable element types and equal evaluable lengths. | These are current implementation rules. Confirm them before treating them as stable language semantics. |
| OBS-008 | Logical and arithmetic compound assignments are represented as binary expressions. Simple assignment has a separate `Assign` node. | The typed tree must preserve this distinction or normalize compound assignment only with an explicit, semantics-preserving rewrite. |
| OBS-009 | Parser recovery can substitute a null-pointer literal when a subexpression fails. Top-level parsing can also skip a token. It does not currently create explicit error AST variants. | `ErrorExpr` and `ErrorStmt` are specified improvements. A recovery null literal must carry recovery status or it can be mistaken for valid source. |
| OBS-010 | The current non-void return check asks whether some return is present; its recursive check does not prove that every control-flow path returns. | TYP-024 and INV-007 require path-sensitive control-flow analysis for a sound Fully Typed AST. The current check is insufficient for that guarantee. |
| OBS-011 | The current checker permits numeric `%` operands, including floating-point types. It also permits comparisons when either direction of assignment compatibility holds. | TYP-006 and TYP-007 define the required operation families. The language owner must confirm whether these broad current cases are intended. |
| OBS-012 | The parser permits prefix increment/decrement on a general expression. The checker verifies numeric type and checks mutability only when it can find a variable name. | The Fully Typed AST rule TYP-011 requires a mutable place. This closes a current semantic gap. |
| OBS-013 | Compound assignment checks target mutability and returns the promoted binary result type. It does not explicitly convert that result back to the target type. | TYP-013 defines a target-type conversion rule. The language owner must confirm narrowing and overflow semantics before implementation. |

Open decisions do not invalidate the structural requirements. A compiler release must not claim the affected behavior as confirmed until the owner resolves the relevant ID.

## 5. AST Architecture

### 5.1 Pipeline

The mandatory sequence is:

1. Parse tokens into a Surface AST and parser diagnostics.
2. Validate Surface AST shape.
3. Resolve declarations and references.
4. Check types and semantic constraints.
5. If no blocking errors remain, construct a Fully Typed AST.
6. Validate Fully Typed AST invariants.
7. Lower the Fully Typed AST to IR.

The semantic pass may use temporary side tables. The returned Fully Typed AST must own or immutably reference all information that later phases need. Later phases must not depend on a live mutable symbol table.

### 5.2 Abstract and concrete nodes

`Node`, `Expr`, `Stmt`, `TypeSyntax`, and `TypedNode` are abstract model categories. They are not directly instantiated. Each listed enum variant or record below is concrete. Rust enum variants are concrete node forms. A declaration is a statement node with declaration metadata.

### 5.3 Identity, ownership, and sharing

| ID | Requirement |
|---|---|
| ARC-001 | Each node has `NodeId(u32)` unique within one AST. IDs are assigned once during construction and remain stable through annotation-only semantic passes. |
| ARC-002 | A node ID must not be reused for a different node while that AST exists. IDs from different ASTs are not comparable unless paired with an `AstId`. |
| ARC-003 | Surface syntax has one owning parent per non-root node. Child edges form an acyclic tree. A node may not appear as two owned children. |
| ARC-004 | Typed AST expression and statement child edges also form an acyclic tree. References to declarations, types, and source-map entries are non-owning IDs and may be shared. |
| ARC-005 | `SymbolId` refers to exactly one declaration in the same compilation unit. `TypeId` refers to exactly one canonical type entry in the same compilation unit. |
| ARC-006 | Implementations may use Rust `Box`, arena indices, or another ownership-safe representation. Public semantics must match these ID and tree rules. |
| ARC-007 | Structural equality compares node kind, ordered fields, and values. It ignores allocation addresses. A separate semantic comparison compares resolved IDs and types. |

An AST is not a valid tree if it has a cycle, a repeated owned child, a missing required child, a dangling ID, or child order that differs from source order.

## 6. Node Model

All spans use half-open byte offsets `[start, end)` in one source file. A span has a source-file ID and valid start and end positions. The source map owns file text and line mapping. Spans do not own AST nodes.

### 6.1 Surface AST common fields

| ID | Requirement |
|---|---|
| SUR-001 | Every concrete Surface AST node has `NodeId` and `SourceSpan`. |
| SUR-002 | Every required child is present. Child vectors preserve lexical source order. |
| SUR-003 | Optional syntax is represented by `Option<T>`, not a fabricated placeholder. |
| SUR-004 | A node produced by parser recovery has `RecoveryStatus::Recovered` and at least one linked diagnostic. Normal nodes have `RecoveryStatus::Complete`. |
| SUR-005 | Surface nodes do not contain inferred types or resolved symbol IDs. Written identifier text and written type syntax remain available. |

### 6.2 Surface concrete nodes

`SExpr` is an abstract category. The following variants are concrete. Each variant also has the common fields in SUR-001.

| Node ID and name | Required fields and types | Semantic function and validity |
|---|---|---|
| S-EX-01 `Literal` | `value: LiteralValue` | Numeric, string, character, Boolean, or null-pointer literal. Literal payload must be valid for its token spelling and source span. |
| S-EX-02 `Variable` | `name: Identifier` | Names a variable or function candidate. Name is non-empty and retains source spelling. |
| S-EX-03 `Grouping` | `inner: SExpr` | Preserves explicit grouping. Exactly one child. |
| S-EX-04 `Unary` | `op: UnaryOp`, `side: Prefix | Postfix`, `operand: SExpr` | Unary operator application. Postfix is valid only for increment and decrement. |
| S-EX-05 `Binary` | `left: SExpr`, `op: BinaryOp`, `right: SExpr` | Binary operation. Child order is left, then right. Assignment operators use this form only where parser grammar does so; simple assignment uses S-EX-08. |
| S-EX-06 `ArrayLiteral` | `elements: Vec<SExpr>` | Array construction. Empty list is structurally valid syntax but semantically invalid until an element type can be determined. |
| S-EX-07 `Call` | `callee: SExpr`, `arguments: Vec<SExpr>` | Function call. Argument order is source order. |
| S-EX-08 `Assign` | `target: SExpr`, `value: SExpr` | Simple assignment. Target must be a variable or index expression. |
| S-EX-09 `Index` | `base: SExpr`, `index: SExpr` | Array or vector indexing. |
| S-EX-10 `ErrorExpr` | `diagnostic_ids: NonEmptyVec<DiagnosticId>` | Parser recovery only. Not a valid source construct. |

`SStmt` is abstract. All statements have a span. `S-STM-01` through `S-STM-11` are concrete:

| Node ID and name | Required fields and types | Semantic function and validity |
|---|---|---|
| S-STM-01 `ExpressionStmt` | `expression: SExpr` | Evaluates an expression as a statement. |
| S-STM-02 `VarDecl` | `bindings: NonEmptyVec<Binding>`, `annotation: TypeSyntax`, `mutability: Mutable | Immutable` | Current grammar requires a type annotation and exactly one initializer per declared name. Each binding has `name: Identifier`, `initializer: SExpr`, and its own span. Binding order is source order. Future annotation-free declarations require an approved grammar change. |
| S-STM-03 `FunctionDecl` | `name: Identifier`, `parameters: Vec<Parameter>`, `return_annotation: Option<TypeSyntax>`, `body: SStmt` | Declares a function. Each parameter has name, required `TypeSyntax`, and span. Duplicate parameter names are semantic errors. |
| S-STM-04 `MainDecl` | `body: SStmt` | Entry-point body. Exactly one is required only if the selected compilation mode requires an entry point. |
| S-STM-05 `If` | `condition: SExpr`, `then_branch: SStmt`, `else_branch: Option<ElseBranch>` | Conditional. `ElseBranch` is `Block(SStmt)` or `ElseIf(SStmt)`. |
| S-STM-06 `While` | `condition: SExpr`, `body: SStmt` | Pre-test loop. |
| S-STM-07 `For` | `initializer: Option<SStmt>`, `condition: Option<SExpr>`, `increment: Option<SExpr>`, `body: SStmt` | C-style loop. Optional fields represent omitted clauses. |
| S-STM-08 `Block` | `statements: Vec<SStmt>` | Ordered lexical statement sequence. Empty block is valid. |
| S-STM-09 `Return` | `value: Option<SExpr>` | Returns from the current function. |
| S-STM-10 `Break` | no variant-specific fields | Exits nearest loop. |
| S-STM-11 `Continue` | no variant-specific fields | Continues nearest loop. |
| S-STM-12 `ErrorStmt` | `diagnostic_ids: NonEmptyVec<DiagnosticId>` | Parser recovery only. |

`TypeSyntax` is abstract, with the following concrete forms:

| Node ID and name | Fields | Validity |
|---|---|---|
| S-TY-01 `PrimitiveType` | `kind: I8 | I16 | I32 | I64 | U8 | U16 | U32 | U64 | F32 | F64 | Char | String | Bool` | These are the primitive type forms accepted by the current type parser. `void` is currently synthesized as the default function result. `nullptr` is currently a literal type, not a parsed type annotation. |
| S-TY-02 `NamedType` | `name: Identifier` | Must resolve to a declared type name. |
| S-TY-03 `ArrayType` | `element: TypeSyntax`, `length: SExpr` | Surface syntax preserves the expression. Typed conversion requires a non-negative integer constant length under the current checker behavior. |
| S-TY-04 `VectorType` | `element: TypeSyntax` | Element type cannot be `void`. |

`LiteralValue` is a tagged value: numeric token value with suffix and radix, UTF-8 string contents, one Unicode scalar for a character, Boolean value, or null-pointer marker. A character payload must contain exactly one Unicode scalar after escape decoding.

### 6.3 Fully Typed AST nodes

The Fully Typed AST has the same source constructs, plus semantic fields. Its construction is atomic: no public `FullyTypedAst` value can be created unless the postconditions in Section 10 hold. The syntax shape below is normative. A language implementation may use different Rust records if it preserves every field and invariant.

`TExpr` is abstract. Every concrete typed expression has `node_id`, `span`, and `result_type: TypeId` unless the row gives a more specific rule.

| Node ID and name | Required fields | Typing and validity |
|---|---|---|
| T-EX-01 `Literal` | `value: LiteralValue`, `result_type` | Type is fixed by literal kind and numeric suffix/defaulting rules. |
| T-EX-02 `VariableRef` | `symbol: SymbolId`, `result_type` | `symbol` resolves to one variable declaration. Function references use T-EX-03. |
| T-EX-03 `FunctionRef` | `symbol: SymbolId`, `result_type` | `result_type` is the function type of the referenced declaration. |
| T-EX-04 `Grouping` | `inner: TExpr`, `result_type` | Result type equals the child result type. |
| T-EX-05 `Unary` | `op`, `side`, `operand: TExpr`, `result_type` | Operand conversions are represented inside `operand`. |
| T-EX-06 `Binary` | `left: TExpr`, `op`, `right: TExpr`, `result_type` | Numeric or other required conversions are child `Convert` nodes. |
| T-EX-07 `ArrayLiteral` | `elements: NonEmptyVec<TExpr>`, `result_type` | All elements have the same element type after explicit conversions. |
| T-EX-08 `Call` | `callee: TExpr`, `arguments: Vec<TExpr>`, `function: SymbolId`, `result_type` | Argument count and types match the resolved function signature. |
| T-EX-09 `Assign` | `target: TPlace`, `value: TExpr`, `result_type` | `TPlace` is a mutable resolved variable or mutable indexed location. Result is the destination type. |
| T-EX-10 `Index` | `base: TExpr`, `index: TExpr`, `result_type` | Base is array/vector; index is integer after allowed conversion. Result is the element type. |
| T-EX-11 `Convert` | `kind: ConversionKind`, `operand: TExpr`, `from: TypeId`, `to: TypeId`, `result_type=to` | Conversion is approved by the conversion relation and records whether it is implicit or explicit. |

`TPlace` is an abstract assignable location. Concrete forms are `VariablePlace { symbol, place_type }` and `IndexPlace { base, index, place_type }`. Places are not general values. They do not have expression node IDs, but their source span is required. An immutable variable cannot form a mutable place.

`TStmt` is abstract. Every concrete statement has `node_id` and `span`. A statement has no expression type field because it does not produce a value. Its semantic result is represented by control-flow facts, not by a value type.

| Node ID and name | Required fields | Validity |
|---|---|---|
| T-STM-01 `ExpressionStmt` | `expression: TExpr` | Child expression is fully typed. |
| T-STM-02 `VarDecl` | `bindings: NonEmptyVec<TBinding>`, `declared_type: TypeId`, `mutability` | Each binding has `symbol`, optional initializer, and span. Inferred declarations store the final type and `TypeOrigin::Inferred`. |
| T-STM-03 `FunctionDecl` | `symbol`, `parameters: Vec<TParameter>`, `function_type: TypeId`, `body` | Parameters and return type agree with function type. |
| T-STM-04 `MainDecl` | `symbol`, `function_type`, `body` | Signature follows the selected target entry-point ABI. |
| T-STM-05 `If` | `condition: TExpr`, `then_branch`, `else_branch: Option<TStmt>` | Condition result type is `bool`. |
| T-STM-06 `While` | `condition: TExpr`, `body` | Condition result type is `bool`. |
| T-STM-07 `For` | typed optional clauses and body | Present condition has type `bool`. Initializer scope encloses condition, increment, and body. |
| T-STM-08 `Block` | `statements: Vec<TStmt>`, `scope: ScopeId` | Statements preserve source order. Scope parent and children are valid. |
| T-STM-09 `Return` | `value: Option<TExpr>`, `function: SymbolId` | Return value presence and type match function return type. |
| T-STM-10 `Break` | `loop_target: NodeId` | Target is an enclosing loop. |
| T-STM-11 `Continue` | `loop_target: NodeId` | Target is an enclosing loop. |

`TParameter` fields are `symbol: SymbolId`, `declared_type: TypeId`, `type_origin: TypeOrigin::Declared`, and `span`. `TBinding` fields are `symbol`, `declared_type`, `type_origin`, optional initializer, and span. `TypeOrigin` is `Declared` or `Inferred`. A conversion result is never stored in `TypeOrigin`; it is a `Convert` node.

No `ErrorExpr`, `ErrorStmt`, unresolved reference, or `Error` type is allowed in a successful Fully Typed AST. Recovery state exists only in a diagnostic-bearing `PartialTypedAst`, which is explicitly not codegen-ready.

## 7. Type System

### 7.1 Type records

`TypeId` indexes an immutable canonical `TypeRecord` table. The following are current supported categories:

| Type form | Representation | Identity and constraints |
|---|---|---|
| Primitive | `I8` through `F64`, `Char`, `String`, `Bool`, `Void`, `NullPtr` | Each primitive kind has one canonical identity. `Void` is a return-only type. `NullPtr` constraints are open under BAS-010. |
| Fixed array | `Array { element: TypeId, length: ConstInt }` | Structural identity: equivalent element types and equal length. Length is non-negative and within implementation limits. |
| Vector | `Vector { element: TypeId }` | Structural identity by equivalent element type. |
| Custom nominal type | `Nominal { symbol: SymbolId }` | Identity is declaration identity, not spelling. This category remains conditional on a type-declaration form being approved. |
| Function | `Function { parameters: Vec<TypeId>, result: TypeId }` | Structural signature identity in a compilation unit. Calling convention is separate target metadata. |
| Error | `Error { diagnostic: DiagnosticId }` | Recovery-only sentinel. Never a language type; forbidden in valid typed output. |
| Inference variable | `Infer { id: InferId }` | Temporary checker state only. It must be solved or rejected before typed output. |

No reference, union, optional, generic, or tuple type is defined. The compiler must reject these forms until an approved language extension defines syntax and typing rules.

### 7.2 Equivalence and assignability

`T₁ ≡ T₂` if both are the same primitive, both are arrays with equivalent element types and equal lengths, both are vectors with equivalent element types, or both are function types with equivalent ordered parameter types and equivalent result types. Nominal types are equivalent only when their `SymbolId` values match.

`T₁` is assignable to destination `T₂` when either `T₁ ≡ T₂` or an allowed conversion `T₁ → T₂` is inserted explicitly in the typed tree. Assignability never silently changes the type field of an expression.

The current numeric promotion ordering is recorded as confirmed implementation behavior, not as a safe or final language rule. For a numeric pair, the current checker chooses the earlier type in the hierarchy. The implementation specification for `Convert` requires a conversion node for each operand promoted to the selected common type. Runtime conversion semantics, overflow behavior, and loss of precision remain open under BAS-011.

### 7.3 Type identity

Two IDs in the same type table identify equivalent types only if their canonical `TypeRecord` is equivalent. IDs from different compilations are not directly comparable. Serialized types must use stable structural keys or declaration keys, not process-local addresses or raw table indices.

## 8. Typing Rules

### 8.1 General judgment

Typing is a partial function. `Γ ⊢ e : T` succeeds only when each required child judgment succeeds and all operator constraints hold. On failure, the checker emits a diagnostic and may continue with an internal error sentinel for recovery. The sentinel cannot escape into a successful Fully Typed AST.

### 8.2 Literals and names

| ID | Rule |
|---|---|
| TYP-001 | Boolean literal: `Γ ⊢ true : bool` and `Γ ⊢ false : bool`. |
| TYP-002 | String literal has type `string`; character literal has type `char`; null literal has type `nullptr`, subject to BAS-010. |
| TYP-003 | A numeric suffix fixes the literal type. Unsuffixed integer and float defaults follow current `Number` mapping: integer to `i64`, unsigned integer to `u64`, ordinary float to `f64`; explicit narrow or `f32` suffixes select their stated type. Range failure is a lexical or literal-validation error. |
| TYP-004 | If `Γ(x)=d` and `d` is a variable, `Γ ⊢ x : declaredType(d)` and the typed node stores `symbol=d.id`. If `d` is a function, a value use is a `FunctionRef`; a variable-only use is an error. If no declaration exists, report undefined name. |
| TYP-005 | Name lookup searches the innermost lexical scope first. A nearer declaration hides an outer declaration. Duplicate declaration in one scope is an error. |

### 8.3 Operators

| ID | Rule and result |
|---|---|
| TYP-006 | Arithmetic `+ - * / %`: operands must be numeric. Promote both operands to common numeric type `N`, then result is `N`. The current checker permits `%` on floating-point types. The language owner must confirm this behavior. |
| TYP-007 | Comparisons `== != < <= > >=`: operands must be compatible. Numeric operands are promoted to common type. Result is `bool`. The exact non-numeric comparison set requires BAS-011 and must be explicit in implementation. |
| TYP-008 | Logical `&& ||`: both operands must be `bool`; result is `bool`. Short-circuit evaluation is required at runtime if the language defines these operators as logical operators. |
| TYP-009 | Bitwise `& | ^ << >>`: both operands must be integer types. Apply the confirmed common integer promotion. Result is the common type. Shift count conversion and out-of-range behavior are open under BAS-011. |
| TYP-010 | Unary `-`: operand must be numeric; result is operand type. Unary `!`: operand and result are `bool`. Unary `~`: operand must be integer; result is operand type. |
| TYP-011 | Prefix/postfix `++ --`: operand must be a mutable place of numeric type. Result type is the place type. The typed node preserves prefix/postfix form. |
| TYP-012 | Simple assignment `x = e`: target must be a mutable variable or index place. Type-check `e`, insert approved conversion to place type, and set assignment result type to place type. |
| TYP-013 | Compound assignment `x op= e`: target must be mutable. Apply the corresponding binary operator rule. Convert the computed result to target type only if the language conversion relation allows it. This operation's exact narrowing behavior is open under BAS-011. |

### 8.4 Composite expressions and calls

| ID | Rule |
|---|---|
| TYP-014 | Grouping has the type of its inner expression. It does not create a conversion. |
| TYP-015 | A non-empty array literal has type `[T; n]` where `n` is element count. Under current behavior, every element must have exactly equivalent type `T`; mixed types are errors and are not promoted. An empty literal is an error. Contextual empty-array inference is not specified. |
| TYP-016 | Indexing requires an array or vector base and an integer index. Result is the element type. Bounds checking is a runtime or optimization concern. |
| TYP-017 | Call requires a function type and a resolved function declaration. Argument count must equal parameter count. Each argument is converted to its parameter type under the conversion rule. Result is the declared function result type. |
| TYP-018 | Function type is the ordered parameter type list plus result type. Each call stores the callee symbol ID and function type. |

### 8.5 Declarations and control flow

| ID | Rule |
|---|---|
| TYP-019 | An annotated variable binding has its written type. Its initializer, if present, must be assignable to that type. A declaration without initializer is valid only if the language permits it for the type and mutability. |
| TYP-020 | An unannotated variable binding infers its type from its initializer. An initializer is required. An initializer that does not produce a type is an error. |
| TYP-021 | Each binding in a multi-binding declaration has its own initializer and symbol ID. All declarations share the written annotation and mutability. Initializer-count syntax must follow the grammar; it must not be guessed from the current `Vec<VarBinding>` representation. |
| TYP-022 | Function parameters have declared types. Function body is checked in a child scope containing parameters. Function symbol and signature must be available according to the function visibility rule; BAS-013 decides forward references and recursion. |
| TYP-023 | `if`, `while`, and present `for` conditions must have type `bool`. |
| TYP-024 | A return statement must be inside a function. A `void` function has no return value. A non-void function returns an assignable value. Every non-void control-flow path must return. This requires path-sensitive control-flow analysis; the current `function_has_return` helper does not establish this postcondition. |
| TYP-025 | `break` and `continue` must resolve to an enclosing loop. The typed nodes store the target loop node ID. |
| TYP-026 | `main` is represented as a function declaration with target-defined entry signature. Current checker treats it as `void` with no parameters. Target ABI and duplicate-entry rules require BAS-015 confirmation. |

## 9. Symbol Resolution and Semantic References

| ID | Requirement |
|---|---|
| SYM-001 | Every declaration receives one `SymbolId`. Symbol records contain kind, source name, declaring node ID, scope ID, declared type or function signature, mutability where applicable, and declaration span. |
| SYM-002 | Every variable reference resolves to one variable declaration. Every call resolves to one function declaration. Every custom type use resolves to one type declaration if custom types are enabled. |
| SYM-003 | Typed references store IDs, not copied names as their semantic identity. They may also preserve source spelling for diagnostics. |
| SYM-004 | The symbol table is lexical. Scope parent links are acyclic. A declaration is visible only in scopes allowed by the language's declaration-order rule. |
| SYM-005 | Parameters are symbols in the function scope. The current checker treats parameters as mutable. The Fully Typed AST records the observed mutability; BAS-014 must confirm it as language behavior. |
| SYM-006 | A symbol ID must resolve in the owning compilation unit. A dangling or cross-unit symbol ID is an internal error. |

## 10. Invariants and Validity Criteria

An AST has independent classifications. An implementation must report each classification separately.

| ID | Classification and invariant | Check stage | Action on failure |
|---|---|---|---|
| INV-001 | **Structurally valid:** all node variants have required fields and valid spans. | Construction and Surface validation | Reject node construction or report internal construction error. |
| INV-002 | **Structurally valid:** owned children form one acyclic tree with unique parent, except the root. | Surface and typed validation | Reject the AST; classify violation as internal error. |
| INV-003 | **Structurally valid:** child vectors preserve source order; optional syntax is represented explicitly. | Parser and Surface validation | Report parser error for malformed input; internal error for malformed in-memory AST. |
| INV-004 | **Structurally valid:** every node and span belongs to the same source map. | Validation and deserialization | Reject AST and report malformed input or internal error. |
| INV-005 | **Semantically valid:** every name reference resolves; duplicate declarations and invalid scope use are absent. | Name resolution | Report source semantic error; do not produce successful typed AST. |
| INV-006 | **Semantically valid:** every expression satisfies its typing rule and all inserted conversions are permitted. | Type checking | Report type error; do not produce successful typed AST. |
| INV-007 | **Semantically valid:** control-flow constraints for return, break, continue, and conditions hold. | Semantic analysis | Report source semantic error; do not produce successful typed AST. |
| INV-008 | **Fully typed:** every expression has exactly one valid `TypeId`; every declaration with semantic type has a resolved `TypeId`. | Typed AST validation | Reject as incomplete or erroneous; do not pass to IR. |
| INV-009 | **Fully typed:** no unresolved inference variable, error type, parser recovery node, unresolved name, or dangling ID exists. | Typed AST validation | Reject; report internal error if the pass claimed success. |
| INV-010 | **Fully typed:** each conversion's `from` equals operand type, its `to` equals result type, and the conversion relation allows it. | Typed AST validation | Reject as internal semantic-pass defect. |
| INV-011 | **Fully typed:** variable references and calls agree with symbol declarations and function signatures. | Typed AST validation | Reject as internal semantic-pass defect. |
| INV-012 | **IR-ready:** all `INV-001` through `INV-011` pass; target-specific entry and ABI checks pass; no blocking diagnostic exists. | Before IR lowering | Stop IR lowering and report diagnostics. |

**Structurally valid** does not imply **semantically valid**. **Semantically valid** does not imply **fully typed** until all inferred types and conversions are materialized. An AST containing errors is never fully typed or IR-ready.

## 11. Error Handling

### 11.1 Categories

| Category | Cause | AST effect | Compilation action |
|---|---|---|---|
| Lexical error | Invalid token, malformed literal, or unterminated text | No node or recovered literal node | Continue scanning where possible. |
| Syntax error | Invalid grammar, missing token, invalid target | `ErrorExpr` or `ErrorStmt` may preserve source location | Parser may recover. Do not type invalid recovered node as valid. |
| Name error | Undefined name, duplicate declaration, wrong symbol kind | Temporary unresolved/error type during recovery | Continue diagnostics; block successful typed AST. |
| Type error | Invalid operator, argument mismatch, assignment mismatch | Temporary `Error` type may suppress cascades | Continue independent checks; block successful typed AST. |
| Semantic flow error | Invalid return, loop control, or missing return path | No valid control-flow annotation | Continue where safe; block successful typed AST. |
| Serialization error | Invalid data, version mismatch, invalid IDs | No accepted AST | Reject input data. |
| Internal error | Broken AST invariant, impossible checker state, dangling internal ID | No recovery node | Stop the affected compiler operation. Record internal context. |

### 11.2 Diagnostic record

Every diagnostic contains `DiagnosticId`, category, stable code where available, severity, primary `SourceSpan`, message, zero or more secondary spans and labels, optional help text, and optional affected `NodeId`. A source error must point to the smallest source span that identifies the violation. An internal error must identify the phase and violated invariant. Diagnostic IDs are stable only within one compilation result.

### 11.3 Error nodes and recovery

1. Only the parser may create `ErrorExpr` or `ErrorStmt`.
2. A recovery node must have a non-empty list of diagnostic IDs and a valid source span.
3. The semantic checker may consume a partial Surface AST to find independent errors. It must not interpret an error node as a valid expression or declaration.
4. The checker may use an internal `Error` type to prevent cascading diagnostics. It must not serialize this type as a language type or return it in a successful typed AST.
5. Type errors stop typed-AST publication, but they do not require the checker to stop collecting diagnostics.
6. Internal errors stop the affected phase. The compiler must not continue to code generation with a violated invariant.

Current error code families are lexical `E0001`–`E0010`, parser `E1001`–`E1015`, semantic `E2001`–`E2032`, IR `E3001`–`E3008`, code generation `E4001`–`E4005`, and I/O `E5001`–`E5005`. Implementations must preserve existing code meanings. A new code requires a unique stable identifier and a test.

## 12. Operations and Interfaces

All operations below are mandatory unless marked optional. The names are conceptual API names. Rust signatures can differ while preserving inputs, outputs, preconditions, and effects.

| ID and operation | Inputs and output | Preconditions, postconditions, effects, and errors |
|---|---|---|
| OPR-001 `construct_surface` | Tokens and source map → `SurfaceAst` plus diagnostics | Parser owns new nodes and IDs. Returns a partial AST only with recovery nodes linked to diagnostics. May return syntax errors. |
| OPR-002 `traverse` | AST and visitor/folder → visit result | Visits each owned node once in deterministic source order. Does not follow semantic reference IDs as child edges. |
| OPR-003 `resolve_symbols` | Surface AST → resolution result and symbol table | Builds lexical scopes and symbol IDs. Result records unresolved references and duplicate declarations as diagnostics. |
| OPR-004 `validate_surface` | Surface AST → validation report | Checks structural invariants without type inference. Invalid in-memory structure is an internal error; source syntax errors remain diagnostics. |
| OPR-005 `type_check` | Surface AST plus resolver data → `Result<FullyTypedAst, Diagnostics>` | Success means every INV-005 through INV-011 holds. Failure returns diagnostics and no successful typed AST. |
| OPR-006 `validate_typed` | Fully Typed AST → validation report | Checks typed invariants and ID consistency. Any failure is an internal compiler error. |
| OPR-007 `transform` | Fully Typed AST and typed rewrite → Fully Typed AST | Optional optimization/rewrite. Must preserve observable behavior, source mappings, and typing invariants. Every output is revalidated. |
| OPR-008 `clone` | AST → independent or shared immutable AST | Clone must preserve node and semantic IDs within the same AST identity, or remap all IDs consistently into a new identity. Mutation aliasing is forbidden. |
| OPR-009 `serialize` | Valid AST and format version → bytes or text | Optional. Must include schema version, node forms, source spans, and required semantic tables. Must not serialize process addresses. |
| OPR-010 `deserialize` | Bytes or text → validated AST or error | Optional. Reject unknown required node kinds, duplicate IDs, dangling references, invalid spans, and unsupported schema versions. Validate before publication. |
| OPR-011 `compare` | Two ASTs and comparison mode → boolean or difference set | Structural mode compares tree shape and source values. Semantic mode also compares resolved symbols and types. Comparison across source maps requires source identity policy. |
| OPR-012 `diagnose` | Diagnostics and source map → rendered diagnostics | Must preserve code, severity, primary span, labels, and help. Rendering must not mutate the AST. |

## 13. Verification and Acceptance Criteria

Each requirement ID in this document is normative. Verification methods below define observable pass conditions. Unit and integration tests are required before implementation acceptance. No specific testing framework is mandated by this specification.

| Verification ID | Covers | Acceptance condition |
|---|---|---|
| VER-001 | SUR-001–SUR-005; OPR-001 | Parse each concrete surface node fixture. Assert node kind, fields, spans, child order, and recovery status. |
| VER-002 | S-EX-01–S-EX-10; S-STM-01–S-STM-12; S-TY-01–S-TY-04 | Unit test valid construction for every variant and rejection/reporting for missing required fields or invalid variant constraints. |
| VER-003 | ARC-001–ARC-007; INV-001–INV-004 | Generate valid trees, then inject duplicate ownership, cycle, bad span, duplicate ID, and dangling child. Validator accepts all valid trees and rejects each corrupted tree with the specified category. |
| VER-004 | TYP-001–TYP-026 | Table-driven tests cover each rule's valid and invalid cases, result type, conversion nodes, and diagnostic category/code. |
| VER-005 | SYM-001–SYM-006; INV-005, INV-011 | Test lexical shadowing, duplicate declaration, unresolved name, wrong symbol kind, recursive/forward function calls per approved BAS-013 rule, and ID-to-declaration consistency. |
| VER-006 | INV-006–INV-012 | Validate that valid programs pass all stages; source type errors fail semantic validity; inference variables and error nodes fail typed validity; target-incomplete trees fail IR readiness. |
| VER-007 | Error handling 11.1–11.3 | For each error class, assert code/category, primary span, affected node when applicable, recovery behavior, and whether a Fully Typed AST is returned. |
| VER-008 | OPR-002–OPR-008 | Assert deterministic traversal, no traversal through symbol links, scope resolution, type checking, and invariant preservation after each allowed transform or clone. |
| VER-009 | OPR-009–OPR-010, if implemented | Round-trip each node category. Assert semantic and structural equality. Reject malformed input and unknown incompatible versions. |
| VER-010 | OPR-011–OPR-012, if implemented | Comparison detects a changed operator, child order, symbol target, or type. Diagnostic rendering preserves source locations and IDs. |
| VER-011 | BAS-009–BAS-015 | Each open decision has an owner-approved answer recorded in this specification or an approved language document before the affected implementation behavior is accepted. |
| VER-012 | ASD-STE100 writing requirement | Editorial review confirms defined terms are consistent, requirements use testable “shall” wording, abbreviations are expanded at first use, and examples are labeled. Full Issue 9 rule checking is required only if the project claims formal STE compliance. |
| VER-013 | OBS-001–OBS-013 | Repository behavior tests confirm every recorded observation. Tests for specified improvements also demonstrate that the typed-AST validator rejects the current gap cases listed in OBS-009 through OBS-013. |

Acceptance requires all mandatory verification rows to pass. Optional serialization and comparison rows apply only when those operations are implemented. Open decision rows must be resolved before acceptance of any implementation that depends on them.

## 14. Technical Examples

### 14.1 Typed numeric expression

Source:

```text
var total: i64 = small + 1;
```

Assume `small` is declared as `i8`. The Surface AST contains a declaration, a binary `Add`, a variable name `small`, and an integer literal. The Fully Typed AST resolves `small` to a `SymbolId`. If the current promotion rule is selected, the `i8` operand is wrapped in a conversion to `i64`. The literal has type `i64`. The binary expression has result type `i64`. The declaration records `TypeOrigin::Declared`.

Invalid typed form: the binary node claims result type `i64`, but its `i8` child has no conversion and the operator rule requires a common `i64` operand type.

### 14.2 Invalid condition

Source:

```text
if (count) { work(); }
```

If `count` has type `i32`, the Surface AST is structurally valid. Type checking reports a Boolean-condition error. The checker may continue to analyze `work()`. It must not return a successful Fully Typed AST.

### 14.3 Unresolved name

Source:

```text
result = missing + 1;
```

If `missing` has no visible declaration, name resolution emits an undefined-name diagnostic at the identifier span. A partial checker may mark the parent expression with an internal error type to avoid follow-on diagnostics. The error type is not part of the language and cannot appear in a successful typed AST.

### 14.4 Shared reference and tree edge

Two `VariableRef` nodes may both store the same `SymbolId`. This is valid sharing because the symbol is a non-owning semantic reference. One expression node used as the owned child of two parents is invalid. The latter creates a graph, not the specified AST tree.

## 15. Open Issues and Design Decisions

### 15.1 Required language-owner decisions

| Issue | Affected requirements | Decision required |
|---|---|---|
| Type declaration grammar and nominal type declarations | BAS-009, S-TY-02, Type System, SYM-002 | Confirm whether custom type names resolve to declared nominal types, aliases, or are not currently supported. |
| Null-pointer meaning | BAS-010, primitive type rules, conversion rules | Confirm legal contexts and compatible destination types. |
| Numeric conversions | BAS-011, TYP-003, TYP-006–TYP-013 | Define pairwise promotion, narrowing, overflow, signed/unsigned rules, and float precision behavior. |
| Array length | BAS-012, S-TY-03, fixed-array identity | Define constant-expression grammar, integer range, zero-length behavior, and type identity. |
| Declaration order | BAS-013, TYP-022, SYM-004 | Define visibility before declaration, mutual recursion, and function recursion. |
| Parameter mutability and calls | BAS-014, T-STM-03, TYP-017, SYM-005 | Confirm parameter mutability and whether indirect calls are permitted. |
| IR boundary and entry point | BAS-015, T-STM-04, TYP-026, INV-012 | Confirm next-stage input, target ABI, and entry-point constraints. |
| Inference grammar | S-STM-02, TYP-019–TYP-021 | Confirm whether annotations are optional in source. Current parser and AST appear to require an annotation. |
| Empty arrays and mixed array elements | TYP-015 | Confirm whether expected-type inference is supported and whether numeric promotion applies to elements. |
| Assignment and comparison semantics | TYP-007, TYP-012–TYP-013 | Define assignment expression value, compound assignment narrowing, equality on strings/arrays, and comparison on non-numeric types. |
| Function return completeness | TYP-024 | Define whether current syntactic `function_has_return` behavior is sufficient or whether path-sensitive control-flow analysis is required. |

### 15.2 Provisional decisions

1. Use stable integer IDs per AST and semantic tables. This allows compact Rust representations and deterministic reference validation. Verify by round-trip and dangling-ID tests.
2. Keep one owner for each syntax node. Use IDs for declarations and types. This avoids shared mutable syntax subtrees and supports later transformations.
3. Represent implicit conversions as explicit typed nodes. This makes the type checker result unambiguous and gives IR lowering a complete operation sequence. This follows syntax-directed translation and explicit intermediate-representation principles; this exact representation is project-specific.
4. Keep the Fully Typed AST separate from IR. IR can change for optimization and target needs without changing source-facing semantic data.
5. Do not add generic, union, optional, reference, or object types without approved syntax and typing rules.

## 16. Bibliographic References

- Aho, Alfred V.; Lam, Monica S.; Sethi, Ravi; Ullman, Jeffrey D. *Compilers: Principles, Techniques, and Tools*. 2nd ed. Addison-Wesley, 2006. Relevant chapters: 2, 4, 6, 7, and 8. The concepts cited in Section 2 are theoretical guidance; this document does not attribute project-specific node definitions to the book.
- ASD. *ASD-STE100 Simplified Technical English*. Issue 9, January 2025. Official information and downloads: [ASD-STE100](https://www.asd-ste100.org/) and [Issue 9 download page](https://asd-ste100.org/STE_downloads.html).
- Descar repository sources inspected for this specification: `crates/descar-core/src/syntax/ast/expr.rs`, `stmt.rs`, `ast_type.rs`, `parameter.rs`, `else_branch.rs`, `binary_op.rs`, `unary_op.rs`, `semantic/type_checker.rs`, `semantic/symbol_table.rs`, `error/error_code.rs`, and `syntax/parser.rs`.
