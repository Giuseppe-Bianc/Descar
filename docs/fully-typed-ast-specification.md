# Descar Fully Typed Abstract Syntax Tree Specification

**Status:** Normative design specification for the implemented Descar language subset.

**Revision:** 2026-10-26. This revision expands phase-boundary contracts, symbol resolution, type-checking and typed-tree finalization, deterministic diagnostic ordering and source locations, recursive canonical type resolution, AST validation and failure reporting, UTF-8 span validity, complexity expectations, traceability, and acceptance criteria while retaining the established language scope and leaving runtime and IR semantics unspecified. ADR-0001 amends literal typing, numeric promotion, shift typing, and compound assignment. Decision DEC-016 records the change. ADR-0002 defines explicit numeric casts. Decision DEC-020 records the change. ADR-0003 limits implicit numeric conversion to conversions that preserve every value of the source type. Decision DEC-017 records the change. ADR-0004 clarifies operator context propagation. Decision DEC-018 records the change. DEC-019 defines conservative non-void return-path validation.

Repository facts are marked **Observed**. Requirements for the new model are marked **Specified**. A requirement marked **Gap** requires an implementation change. Open issues are listed in Section 15.

## 1. Purpose and Scope

This document defines the Descar Surface Abstract Syntax Tree (Surface AST) and Fully Typed Abstract Syntax Tree (Fully Typed AST).

The Surface AST records source syntax. The Fully Typed AST records the same program after symbol resolution and type checking. It contains a resolved type for every expression and a resolved declaration identity for every reference.

The Fully Typed AST is the input contract for later intermediate representation (IR) lowering. It is not IR. IR lowering is outside this specification.

The scope is the language that the repository parser and type checker implement:

- Primitive numeric, Boolean, character, and string values.
- Null-pointer literals.
- Variables, functions, parameters, and the main function.
- Unary and binary operators.
- Explicit casts using `expression as type` across the supported conversion families.
- Assignment and compound assignment.
- Calls.
- Fixed arrays, array literals, vectors, and indexing.
- Blocks, conditionals, loops, returns, break, and continue.

This specification does not add structure declarations, enumerations, generics, unions, optional values, pointer operations, closures, modules, method calls, or function values.

### 1.1 Requirement language

The words MUST, MUST NOT, SHOULD, and MAY state requirements.

### 1.2 Design basis

The compiler concepts in this document come from Aho, Lam, Sethi, and Ullman. The book describes syntax trees, symbol tables, declarations, types, type checking, and intermediate code as related compiler concepts. It does not prescribe the project-specific records or IDs in this document.

The separate Surface AST and Fully Typed AST, node identifiers, type identifiers, symbol identifiers, conversion nodes, and acceptance rules are project design decisions.

### 1.3 Analysis of available requirements

The confirmed target is an internal Fully Typed AST for the current Descar subset. The typed representation is separate from parser output. It follows current checker behavior, with the confirmed mutable-place, compound-assignment, array-length, literal-typing, and promotion rules. Nested functions are not part of the language. Parser diagnostics stay outside the AST. Serialization is out of scope.

## 2. Applicable Standards and References

### 2.1 Theoretical reference

The following concepts from the theoretical reference apply:

| Concept | Application in this specification | Reference |
| --- | --- | --- |
| Syntax trees represent program structure | Surface AST node hierarchy and child order | Aho et al., Chapter 6, Section 6.1 |
| Symbol tables associate names with declarations | Lexical scope and symbol resolution | Aho et al., Chapter 2, Section 2.7; Chapter 6, Section 6.3 |
| Type checking validates operations | Typing rules and semantic diagnostics | Aho et al., Chapter 6, Section 6.5 |
| Syntax-directed translation can build later representations | Construction of the Fully Typed AST from the Surface AST | Aho et al., Chapter 5; Chapter 6, Sections 6.1 and 6.4 |
| Intermediate code is a later representation | Fully Typed AST precedes IR lowering | Aho et al., Chapter 6 |

These references support the compiler architecture. They do not define Descar syntax or Descar type rules.

### 2.2 Writing standard

This document uses the writing goals supplied in the task and *Simplified Technical English (STE)*. The Aerospace, Security and Defence Industries Association of Europe (ASD) publishes the standard as ASD-STE100. The current official issue is Issue 9, dated January 2025. This document uses short sentences, defined terms, stable terminology, numbered procedures, and verifiable requirements.

This document does not claim certification against the complete ASD-STE100 controlled dictionary.

## 3. Terms and Definitions

| Term | Definition |
| --- | --- |
| AST | Abstract syntax tree. A tree representation of source constructs. |
| ID | Identifier. In an ID field, it is an opaque identity value. |
| API | Application programming interface. |
| CLI | Command-line interface. |
| UTF-8 | Unicode Transformation Format, 8-bit. It encodes source text. |
| ASD | Aerospace, Security and Defence Industries Association of Europe. |
| STE | Simplified Technical English. |
| ISBN | International Standard Book Number. |
| iN, uN | Signed integer with N bits; unsigned integer with N bits. These are Rust scalar type names when N is 8, 16, 32, or 64. |
| f32, f64 | 32-bit and 64-bit floating-point scalar types. |
| u64 | Unsigned 64-bit integer. |
| T, T₁, T₂, S, R | Type metavariables. Each denotes one resolved type. |
| n, i | A non-negative element or parameter count; an index in a finite sequence. |
| op | An operator permitted by the rule where it appears. |
| Surface AST | The parser output. It contains syntax and source locations. |
| Fully Typed AST | An AST with resolved types, resolved symbol references, and explicit conversions. |
| Node | One concrete record or enum variant in an AST. |
| Node ID | Identity of one node within one AST instance. |
| Symbol ID | Identity of one declaration within one compilation unit. |
| Type ID | Identity of one canonical type within one compilation unit. |
| AstId | Opaque identity for one AST instance. |
| SourceId | Opaque identity for one source file in one compilation unit. |
| Option<T> | A value of type T or no value. It represents an optional field. |
| Vec<T> | An ordered list of zero or more values of type T. |
| Result<T,E> | Either a successful value of type T or an error value of type E. |
| DiagnosticId | Opaque identity for one diagnostic in one compiler run. |
| SourceMap | A map from SourceId to a source record with `path: String` and `text: String`. |
| Phase | One of Lexer, Parser, Resolver, TypeChecker, TypedValidator, Lowerer, or input/output. |
| Severity | One of Error, Warning, or Note. Errors block successful typed-AST output. |
| ScopeId | Opaque identity for one lexical scope. |
| LoopId | Opaque identity for one loop context. |
| Identifier | A non-empty name token emitted by the lexer. Name comparison uses exact spelling. It does not normalize Unicode. |
| Declared type | Type written for a variable or parameter, or the function result type. An omitted function result annotation means void. |
| Inferred type | Type assigned by semantic analysis to an expression. Variables do not have inferred declarations in this language subset. |
| Result type | Type assigned to an expression after its operand conversions. It can be Void when a call returns no value. |
| Conversion | A typed operation that changes a value from one type to another. |
| Incomplete AST | A parser result that contains recovered syntax or unresolved data. It is not valid input to type checking or IR lowering. |
| Semantic error | A source program error found during name resolution, type checking, or semantic validation. |
| Internal error | A compiler defect or violated implementation invariant. It is not a source program error. |
| Place | A mutable storage location. A variable or an index into a mutable collection can be a place. |
| Type equivalence | The relation used to decide whether two types are the same type. |
| Assignability | The relation used to decide whether an expression of one type can be converted to a destination type. |
| Adaptable literal | A numeric Literal whose `width_suffix` is false. Its type can come from its context (TR-028). |
| Flexible expression | An expression whose type can come from its context. Section 8.2.1 defines it. |
| Context type | A type that the position of an expression supplies to it, such as the declared type of a binding. Section 8.2.1 lists the positions. |
| Default type | The type that TR-001 gives an unsuffixed numeric literal when no context type adapts it: I64, U64, or F64. |

Formal notation:

- Γ is the active lexical environment.
- Γ(x) is the nearest visible declaration of name x.
- Γ ⊢ e : T means that expression e has type T in environment Γ.
- T₁ ≡ T₂ means that T₁ and T₂ are equivalent.
- T₁ ⪯ T₂ means that a value of T₁ is assignable to T₂.
- Promote(T₁,T₂) is the numeric promotion operation in Section 7.3.
- Γ ⊢ e ⇐ T ⇒ S means that e is checked with context type T and has type S.

The AST-specific record definitions are:

| Name | Definition |
| --- | --- |
| AstId, NodeId, SymbolId, TypeId, ScopeId, LoopId, SourceId, CompilationUnitId, DiagnosticId | Distinct newtype wrappers over u64. Zero is a valid ID. An ID is meaningful only in its declared AST, source, compilation-unit, or compiler-run domain. IDs are not interchangeable. |
| CompilationUnitId | Identity of one source file processed as one semantic unit in one compiler run. This subset has no module or multi-file symbol-resolution rule. |
| NodeId | A monotonically allocated u64 unique within one AstId. A node key is the pair (AstId, NodeId). |
| SourceSpan | A record `{ source_id: SourceId, start_byte: u64, end_byte: u64 }`. The range is half-open. `start_byte <= end_byte` and `end_byte` is within the source file. Both offsets MUST be UTF-8 code-point boundaries. A span MUST NOT split a multi-byte Unicode scalar value. |
| Token | Lexer output with `kind: TokenKind` and `span: SourceSpan`. TokenKind belongs to the lexer interface and is outside the AST model. |
| Tokens | `Vec<Token>` in source order. |
| Diagnostics | `Vec<Diagnostic>` in report order. |
| Identifier | The exact non-empty UTF-8 spelling of one lexer identifier token. Equality compares Unicode scalar values without normalization. |
| LiteralValue | `Numeric(Number)`, `StringLit(String)`, `CharLit(String)`, `Bool(bool)`, or `NullPtr`. |
| Number | `I8(i8)`, `I16(i16)`, `I32(i32)`, `Integer(i64)`, `U8(u8)`, `U16(u16)`, `U32(u32)`, `UnsignedInteger(u64)`, `Float32(f32)`, `Float64(f64)`, `Scientific32(f32,i32)`, or `Scientific64(f64,i32)`. The 64-bit variants hold the value of an unsuffixed literal and of a literal with the matching 64-bit suffix. The `width_suffix` field of N-SE-01 tells them apart. |
| UnaryOp | `Negate`, `Not`, `BitwiseNot`, `Increment`, or `Decrement`. |
| BinaryOp | The operators listed in Section 8.4 and the compound operators listed in TR-013. |
| NonAssignmentBinaryOp | The arithmetic, comparison, logical, and bitwise operators in TR-009 through TR-012. It excludes assignment and compound assignment. |
| CompoundOperator | `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `\|=`, `^=`, `<<=`, or `>>=`. |
| UnaryOperand | `Value(TypedExpr)` or `Place(Place)`. Increment and decrement require Place. Other unary operators require Value. |
| NonEmptyVec<T> | An ordered vector with at least one element. |
| ResolvedType | A primitive, composite, opaque named, Void, or NullPtr type listed in Section 7.1. |
| SourceBlock | An alias for the concrete Surface Block node. |
| TypedBlock | An alias for the concrete Typed Block node. |
| ElseBranch | `None`, `Block(SourceBlock)`, or `ElseIf(Surface If)`. |
| TypedElseBranch | `None`, `Block(TypedBlock)`, or `ElseIf(Typed If)`. |

In node tables, fields named `node_id` and `span` have types NodeId and SourceSpan. A field named `result_type` or `place_type` has type TypeId. A field named `operator` has type UnaryOp, NonAssignmentBinaryOp, or CompoundOperator as stated by its node. Fields named `side` have type Prefix or Postfix. An unqualified `else_branch` has type ElseBranch for a Surface If and TypedElseBranch for a Typed If. Every child field has the abstract or concrete node type shown in that field. Every optional field uses `Option<T>`. Every ordered list uses `Vec<T>` unless the field is explicitly `NonEmptyVec<T>`.

## 4. Assumptions and Constraints

| ID | Status | Requirement or observed fact |
| --- | --- | --- |
| BAS-001 | Observed | The compiler is a Rust workspace. The core crate is descar-core. |
| BAS-002 | Observed | The parser returns a list of statements and parser diagnostics. |
| BAS-003 | Observed | The type checker traverses the parsed AST and returns diagnostics. It does not store inferred expression types in the AST. |
| BAS-004 | Observed | The type checker uses nested lexical scopes. |
| BAS-005 | Observed | Numeric promotion preference is f64, f32, u64, i64, u32, i32, u16, i16, u8, i8, in that order. |
| BAS-006 | Specified | The Surface AST and Fully Typed AST are separate representations. |
| BAS-007 | Specified | A failed parse or semantic pass returns diagnostics outside the AST. It does not return a valid Fully Typed AST. |
| BAS-008 | Specified | The AST is an internal compiler model. It has no stable external application programming interface (API) requirement. |
| BAS-009 | Specified | Serialization and deserialization are outside this specification. |
| BAS-010 | Specified | The type rules follow the current checker, except for the confirmed place, compound-assignment, and array-length corrections in DEC-006, [DEC-007](#dec-007), and [DEC-008](#dec-008), defined in Section 15.1, the literal-typing and promotion corrections in DEC-016, and the numeric-conversion correction in DEC-017. |
| BAS-011 | Specified | A source fixed-array length MUST be a non-negative integer literal. The Fully Typed AST stores its value as u64. Zero is valid. |
| BAS-012 | Specified | Named custom types are opaque names. They have no declarations, fields, constructors, or alias rules in this language subset. |
| BAS-013 | Specified | The language has no first-class function type. A function signature is metadata on a function declaration and symbol. |
| BAS-014 | Specified | Error and inference states are not source-language types. They cannot appear in a successful Fully Typed AST. |
| BAS-015 | Observed | Parser recovery can return partial statements with diagnostics. The command-line interface (CLI) stops before type checking when parse diagnostics exist. |
| BAS-016 | Provisional | No maximum AST size, latency, or memory target is confirmed. This specification makes no quantitative performance promise. Verify performance after a representative workload is defined. |
| BAS-017 | Specified | Function and main declarations are valid only as direct children of the program root. The language does not support nested function declarations. |

### 4.1 Repository behavior and gaps

| ID | Observed behavior | Required consequence |
| --- | --- | --- |
| GAP-001 | The parsed AST has no node IDs, type IDs, or resolved symbol IDs. | The Fully Typed AST builder MUST add these fields. |
| GAP-002 | Expression types are returned temporarily by the checker. They are not retained. | The new checker MUST emit a Fully Typed AST with a result type on every expression. |
| GAP-003 | The checker accepts mixed-width compound assignments without checking conversion back to the target type. | The new checker MUST enforce DEC-006 and TR-013. |
| GAP-004 | Array type syntax stores an arbitrary expression. The checker evaluates only non-negative integer literals for size comparison. | The Fully Typed AST MUST use a validated integer length under TR-026 and TR-027. A non-literal length MUST produce a source diagnostic. |
| GAP-005 | Custom type names parse, but the checker does not resolve them to declarations. | The Fully Typed AST MUST preserve them as opaque named types. It MUST NOT imply that named type declarations exist. |
| GAP-006 | Parser recovery can insert a null-pointer literal as an expression placeholder and a built-in Void type as an invalid declaration-type fallback. | An incomplete SurfaceProgram represents a missing initializer or invalid annotation with an absent optional field. It MUST NOT pass a placeholder or fallback to type checking. |
| GAP-007 | Binary expression spans can cover only the operator token. Prefix unary spans can cover only the prefix token. | The new Surface AST and Fully Typed AST MUST use spans for the full source construct. |
| GAP-008 | The non-void return check uses a recursive heuristic. | The checker MUST reject a non-void function when its body can complete normally without returning a value. The analysis is conservative and treats every loop as potentially completing normally. |
| GAP-009 | No typed-AST module or IR-lowering module exists in the inspected core source. | This specification defines a target interface. It does not claim that the target interface is implemented. |
| GAP-010 | The parser permits increment and decrement on any expression. It permits assignment and compound assignment to variables or index expressions, including an index rooted in a temporary. The checker can accept some non-place targets when it finds no immutable variable name. | The new checker MUST require a mutable variable place or an index chain rooted in one. |
| GAP-011 | The checker tracks loop context with a Boolean. It does not reset this state when it enters a nested function. | Nested functions are not supported. The parser MUST reject them under GAP-012. The loop-state leak cannot occur in a valid program. |
| GAP-012 | The parser accepts function and main declarations wherever it parses a statement. The type checker also visits these declarations when they occur inside a block. | The parser MUST reject a function or main declaration that is not a direct program child, as required by SYN-003. |
| GAP-013 | Every binary row has a right power below its left power, so the parser groups repeated operators to the right. For example, 10 - 3 - 2 parses as 10 - (3 - 2). | The parser MUST use the binding powers in Section 6.4. Each left-associative row MUST have a right power equal to its left power plus 1. |
| GAP-014 | TR-001 types every unsuffixed numeric literal with a 64-bit type, and the assignability table has no narrowing edge. An unsuffixed literal cannot initialize a variable of a narrower type or combine with a value of a narrower type. | The new checker MUST apply TR-028. |
| GAP-015 | Promotion selects the first type in the preference order without checking that the operands keep their values. Shift operators promote both operands. The assignability table also contains four numeric conversion rows that cannot preserve every source value: I32 and U32 to F32, and I64 and U64 to F64. | The new checker MUST apply the promotion condition in Section 7.3, the shift rule in TR-012, and the lossless conversion table in Section 7.2 (DEC-017). |

The performance target in BAS-016 is provisional. No unresolved decision affects the confirmed language scope.

## 5. AST Architecture

### 5.1 Required pipeline

The compiler MUST use this sequence:

1. Parse tokens into a Surface AST and parser diagnostics.
2. Reject the Surface AST for semantic processing if parser diagnostics exist.
3. Validate the complete Surface AST structure.
4. Resolve term symbols and canonicalize type syntax. Named custom types remain opaque.
5. Check expression types and statement constraints.
6. Build a Fully Typed AST only if the previous stages produce no blocking errors.
7. Validate the Fully Typed AST.
8. Pass the Fully Typed AST to a later lowering phase.

The type checker MAY use temporary symbol tables and side tables. The returned Fully Typed AST MUST contain or immutably reference all semantic data needed by later phases.

| ID | Requirement | Verification |
| --- | --- | --- |
| ARC-008 | The compiler MUST follow the ordered pipeline in this section. A stage MUST NOT receive a result that failed its preceding validation stage. | Pipeline tests inject one failure at each stage and verify that later stages do not run. |
| ARC-009 | Abstract categories in Section 5.2 MUST NOT be constructible as standalone nodes. Only concrete records and variants in Section 6 can be instantiated. | Compile-time API test or constructor test verifies that no abstract category has a public constructor. |
| ARC-010 | Source locations MUST use the SourceSpan definition in Section 3. A concrete node MUST have one span. | Span validation tests check source identity, range bounds, and zero-length spans. |
| ARC-011 | The Fully Typed AST MUST replace TypeSyntax nodes with canonical TypeIds and resolved constants. It MUST NOT own duplicate Surface TypeSyntax children. Declaration and symbol spans MUST continue to identify each source declaration. | Typed-tree tests verify type annotations resolve to TypeIds, array lengths become u64 constants, and declaration spans remain available. |

### 5.1.1 Phase interface contract

The pipeline in Section 5.1 defines ordering. The following table defines the minimum information contract at each boundary. It does not assert that a corresponding implementation module currently exists.

| Boundary ID | Producer | Consumer | Required input state | Required output or failure behavior |
| --- | --- | --- | --- | --- |
| IF-001 | Lexer | Parser | Source bytes associated with one SourceId. | Tokens are ordered by source position and carry valid half-open spans. Lexical diagnostics prevent successful compilation. |
| IF-002 | Parser | Structural validator | Tokens and lexer status. | A SurfaceProgram preserves source order and full-construct spans. Any parser diagnostic makes the program Incomplete. |
| IF-003 | Structural validator | Resolver | A complete, structurally valid SurfaceProgram. | The resolver must not receive recovery placeholders or invalid required children. Structural implementation defects are internal errors. |
| IF-004 | Resolver | Type checker and typed-AST builder | Surface nodes, lexical scope information, and resolved declaration identities. | Each valid name reference identifies exactly one declaration of the required kind. A blocking name-resolution error prevents successful typed output. |
| IF-005 | Type checker | Typed-AST builder | Resolved names, canonical type information, source constructs, and context types where TR-028 requires them. | Each successfully checked expression has one result TypeId. Each required implicit conversion is explicit in the typed representation. |
| IF-006 | Typed-AST builder | Typed-AST validator | A candidate TypedProgram and its compilation-unit symbol and type tables. | All invariants in Section 10 pass before the candidate is exposed as a successful Fully Typed AST. |
| IF-007 | Typed-AST validator | Later lowering phase | A Fully Typed AST that passes all applicable invariants. | The consumer may rely on resolved references, canonical types, explicit conversions, and valid control-flow references. Lowering semantics remain outside this specification. |
| IF-008 | Any phase | Diagnostic renderer | Diagnostics with phase, severity, source span when applicable, and stable diagnostic identity. | Rendering must not change diagnostic meaning or treat an internal error as a source-language error. |
| IF-009 | Each phase consumer | The preceding phase's output for the applicable boundary. | The consumer MUST check each incoming-boundary precondition in this table that is not guaranteed by the input type. | Reject missing required fields, inconsistent records, and invalid or dangling semantic IDs before invoking phase processing. Classify the failure according to Section 11. Verification: for each applicable invariant, supply a malformed boundary value and assert rejection before processing starts. |
| IF-010 | Each phase boundary | The producer's result, diagnostics, and boundary-validation result. | The result has no blocking diagnostic and passes boundary validation. | A blocking diagnostic or failed validation MUST prevent the result from being passed as valid input to the next phase and MUST prevent successful Fully Typed AST output. Parser recovery is permitted only for diagnostic collection; its Incomplete SurfaceProgram MUST be rejected before resolution under IF-003. Verification: supply a blocking diagnostic and a failed boundary-validation result separately, then assert the downstream phase is not invoked and no successful TypedProgram is returned. |

**Explanatory note:** IF-009 and IF-010 define the common validation and failure-gating rules for the boundaries listed in IF-001 through IF-008.

### 5.2 Abstract and concrete categories

The following are abstract categories. They cannot be instantiated directly:

- AstNode
- SurfaceNode
- SurfaceExpr
- SurfaceStmt
- TypeSyntax
- TypedNode
- TypedExpr
- TypedStmt
- Place

The concrete records and variants in Section 6 can be instantiated.

### 5.3 Identity and sharing

| ID | Requirement | Verification |
| --- | --- | --- |
| ARC-001 | Every AST instance MUST have an AstId. Every node MUST have a NodeId that is unique in that AST. | Build two ASTs with repeated node counts. Verify each node key is unique within its AstId. |
| ARC-002 | A node ID MUST NOT be reused for a different node while its AST exists. | Transform a tree and verify retained nodes keep identity and new nodes receive new IDs. |
| ARC-003 | Every non-root node MUST have exactly one owning parent. Owned child edges MUST form an acyclic tree. | Structural validator must reject a repeated owned child or cycle. |
| ARC-004 | Symbol and type references MAY be shared. They MUST use SymbolId or TypeId. They MUST NOT create AST child edges. | Walk the AST and verify all shared links use typed IDs. |
| ARC-005 | A SymbolId MUST resolve to exactly one declaration in the same compilation unit. A TypeId MUST resolve to exactly one canonical type entry in the same compilation unit. | Referential-integrity test over all symbol and type entries. |
| ARC-006 | If AST comparison is provided, it MUST ignore memory addresses. Structural comparison compares node kind, ordered fields, and literal values. Semantic comparison also compares symbol and type IDs. | If a comparison utility exists, compare separately allocated equivalent trees and semantically different references. |
| ARC-007 | A deep clone MUST create a new AstId and remap all NodeIds. It MAY share immutable symbol and type tables only when both trees retain the same compilation-unit identity. | Clone test checks node identity and reference validity. |

An AST is not a valid tree if it has a cycle, a repeated owned child, a missing required child, an invalid child order, or a dangling semantic ID.

The semantic tables have these required records:

| Record | Required fields and rules |
| --- | --- |
| SymbolTable | `compilation_unit_id: CompilationUnitId`; `global_scope: ScopeId`; `scopes: Vec<ScopeRecord>`; `symbols: Vec<SymbolRecord>`. Each ID occurs once. |
| ScopeRecord | `id: ScopeId`; `parent: Option<ScopeId>`; `kind: Global, Function, or Block`; `owner: Option<NodeId>`; `declarations: Vec<(Identifier, SymbolId)>`. Global has no parent. Other scopes have one parent. Declaration order is source order. |
| SymbolRecord::Variable | `id: SymbolId`; `name: Identifier`; `type_id: TypeId`; `mutable: bool`; `declaration: NodeId`; `span: SourceSpan`; `scope: ScopeId` for the scope that declares it. |
| SymbolRecord::Parameter | `id: SymbolId`; `name: Identifier`; `type_id: TypeId`; `mutable: true`; `declaration: NodeId`; `span: SourceSpan`; `scope: ScopeId` for the function scope; `function: SymbolId`. |
| SymbolRecord::Function | `id: SymbolId`; `name: Identifier`; `parameters: Vec<SymbolId>`; `parameter_types: Vec<TypeId>`; `result_type: TypeId`; `declaration: NodeId`; `span: SourceSpan`; `scope: ScopeId` for the containing declaration scope. The two parameter lists have equal length and matching order. |
| SymbolRecord::Main | A Function record with the exact name `main`, no parameters, and result type Void. |
| LoopRecord | `id: LoopId`; `owner: NodeId`; `function: SymbolId`. Each break or continue refers to the nearest active LoopRecord in the same function. |
| TypeTable | `compilation_unit_id: CompilationUnitId`; `entries: Vec<TypeEntry>`. Each TypeId occurs once. |
| TypeEntry | `id: TypeId`; `kind: ResolvedType`. ResolvedType is one of the categories in Section 7.1. |

Each scope stores only declarations made directly in that scope. Lookup searches the current scope and then each parent, in order. A declaration of the same name in a child scope hides the parent declaration. IDs are never resolved by scanning AST node memory addresses.

TypeTable canonicalization MUST use these keys: primitive and special types use one entry per kind; named types use exact identifier spelling; vector types use the canonical element TypeId; array types use the canonical element TypeId and u64 length. Two equal keys MUST map to one TypeId in a compilation unit. TypeIds are not stable across compilation units or runs.

The required canonicalization procedure is:

~~~text
resolve_type(syntax):
    match syntax:
        PrimitiveType(kind):
            return intern(kind)
        NamedType(name):
            return intern(Named(exact_identifier_spelling(name)))
        VectorType(element):
            element_id = resolve_type(element)
            if element_id is failure:
                return failure
            if resolved_kind(element_id) is built-in Void:
                report an internal type-syntax invariant failure
                return failure
            return intern(Vector(element_id))
        ArrayType(element, length_expr):
            element_id = resolve_type(element)
            length = resolve_array_length(length_expr)  // TR-026
            if either step fails:
                return failure
            return intern(Array(element_id, length))

intern(canonical_kind):
    if TypeTable contains an entry with this canonical kind:
        return that entry's TypeId
    allocate one TypeId
    append TypeEntry { id: TypeId, kind: canonical_kind }
    return TypeId
~~~

`intern` MUST compare keys by the identity rules in Sections 3 and 7.1, not by source span or AST node identity. It MUST return the existing TypeId for an existing canonical key. When a child type or array length fails to resolve, the parent type MUST NOT be inserted. This procedure applies only to TypeSyntax variants the parser is specified to produce. It does not make the built-in Void type available as a source annotation, and it does not turn the identifier `void` into built-in Void.

| ID | Requirement | Verification |
|---|---|---|
| SR-013 | SymbolTable, ScopeRecord, SymbolRecord, LoopRecord, TypeTable, and TypeEntry MUST use the fields and links listed above. | Validate one test table with each record kind, then corrupt every required ID link and verify rejection. |

### 5.4 Source locations

SourceSpan uses a half-open byte range [start_byte, end_byte) in one source file. Every concrete node MUST have a span. The span MUST refer to a registered source file. Both offsets MUST be UTF-8 code-point boundaries, and the range MUST satisfy `start_byte <= end_byte <= source_length`. The span of a composite expression or statement MUST cover its complete source construct. A point span is valid when start_byte equals end_byte.

## 6. Node Model

### 6.1 Root nodes

| Node ID and name | Fields | Cardinality and validity |
| --- | --- | --- |
| N-ROOT-01 SurfaceProgram | ast_id: AstId; node_id: NodeId; source_id: SourceId; statements: Vec<SurfaceStmt>; span: SourceSpan; status: Complete or Incomplete | One root per parsed source unit. Statements are in source order. |
| N-ROOT-02 TypedProgram | ast_id: AstId; node_id: NodeId; source_id: SourceId; compilation_unit_id: CompilationUnitId; statements: Vec<TypedStmt>; symbols: SymbolTable; types: TypeTable; span: SourceSpan | One root per successful semantic unit. It contains no error or unknown type state. The symbol and type tables use this compilation_unit_id. |

The current parser uses Vec<Stmt> as its root. SurfaceProgram is a specified wrapper. It gives the semantic pass one identity and one validity status.

The syntax profile in Section 6.4 is requirement SYN-001. It defines the complete current subset grammar used for AST construction. Parser-conformance verification MUST cover each production and each documented optional clause.

### 6.2 Surface expression nodes

SurfaceExpr is abstract. Each node below is concrete. All required children are owned. All lists preserve source order.

| Node ID and name | Required fields | Semantic function and validity |
| --- | --- | --- |
| N-SE-01 Literal | value: LiteralValue; width_suffix: bool; node_id; span | Numeric, string, character, Boolean, or nullptr literal. `width_suffix` is true when the source token carries a width suffix. A numeric literal with `width_suffix` false is an adaptable literal. |
| N-SE-02 Variable | name: Identifier; node_id; span | Name reference. It is resolved in the semantic pass. |
| N-SE-03 Grouping | expression: SurfaceExpr; node_id; span | Explicit grouping. Exactly one child. |
| N-SE-04 Unary | operator: UnaryOp; side: Prefix or Postfix; operand: SurfaceExpr; node_id; span | Postfix is valid only for increment or decrement. |
| N-SE-05 Binary | left: SurfaceExpr; operator: BinaryOp; right: SurfaceExpr; node_id; span | Binary operation. Left child precedes right child. Compound operators are preserved. For compound assignment, the parser accepts only a variable or index expression as the left child. |
| N-SE-06 ArrayLiteral | elements: Vec<SurfaceExpr>; node_id; span | Array construction. Empty syntax is structurally valid but fails type inference. |
| N-SE-07 Assignment | target: SurfaceExpr; value: SurfaceExpr; node_id; span | Simple assignment. Parser-valid targets are variables or index expressions. An index base can still be a temporary; the semantic place rule rejects it. |
| N-SE-08 Call | callee: SurfaceExpr; arguments: Vec<SurfaceExpr>; node_id; span | Call. Arguments are in source order. |
| N-SE-09 Index | array: SurfaceExpr; index: SurfaceExpr; node_id; span | Array or vector index. |
| N-SE-10 Cast | operand: SurfaceExpr; target_type: TypeSyntax; node_id; span | Explicit conversion using the existing type grammar. The full source extent includes the `as` keyword and target type. Semantic rules constrain which source/target pairs are supported. |

There is no ErrorExpr node. Recovery placeholders do not make a parse result valid. If parsing emits a diagnostic, SurfaceProgram.status is Incomplete.

### 6.3 Surface statement nodes

SurfaceStmt is abstract. All concrete statements have node_id and span.

| Node ID and name | Required fields | Semantic function and validity |
| --- | --- | --- |
| N-SS-01 ExpressionStmt | expression: SurfaceExpr | Evaluates one expression as a statement. Its span is the expression span. |
| N-SS-02 VarDecl | bindings: NonEmptyVec<Binding>; annotation: Option<TypeSyntax>; mutable: bool | Declares one or more variables with one shared annotation. A complete SurfaceProgram requires an annotation and an initializer for every binding. Incomplete parser output may omit either field. Source order is retained. |
| N-SS-03 FunctionDecl | name: Identifier; parameters: Vec<Parameter>; return_annotation: Option<TypeSyntax>; body: Block | Declares a function. It is valid only as a direct child of SurfaceProgram. Each parameter has a name, type annotation, and span. Omitted return annotation means void. |
| N-SS-04 MainDecl | body: Block | Declares the special main function. It is valid only as a direct child of SurfaceProgram. It has no parameters and has result type void. |
| N-SS-05 If | condition: SurfaceExpr; then_block: Block; else_branch: None, Block, or ElseIf | Conditional statement. ElseIf contains another If statement. |
| N-SS-06 While | condition: SurfaceExpr; body: Block | Pre-test loop. |
| N-SS-07 For | initializer: Option<SurfaceStmt>; condition: Option<SurfaceExpr>; increment: Option<SurfaceExpr>; body: SurfaceStmt | C-style loop. Missing clauses remain absent. The current parser permits any statement as the body. |
| N-SS-08 Block | statements: Vec<SurfaceStmt> | Ordered statement sequence. An empty block is valid. |
| N-SS-09 Return | value: Option<SurfaceExpr> | Returns from the current function. |
| N-SS-10 Break | no variant fields | Requests exit from the nearest loop. |
| N-SS-11 Continue | no variant fields | Requests the next iteration of the nearest loop. |

The nested concrete nodes are:

| Node ID and name | Required fields | Relationships and validity |
| --- | --- | --- |
| N-SD-01 Binding | name: Identifier; name_span: SourceSpan; initializer: Option<SurfaceExpr>; node_id: NodeId; span: SourceSpan | Owned by exactly one VarDecl. A complete SurfaceProgram requires an initializer. `span` covers the name and any present initializer. |
| N-SD-02 Parameter | name: Identifier; name_span: SourceSpan; annotation: TypeSyntax; node_id: NodeId; span: SourceSpan | Owned by exactly one FunctionDecl. `span` covers the name and annotation. Parameter order is source order. |

The parser returns an incomplete program with a diagnostic if a declaration has a missing initializer or type annotation. The target Surface AST represents the missing field as None. The typed AST does not represent an uninitialized variable in this subset.

### 6.4 Surface type syntax

TypeSyntax is abstract.

| Node ID and name | Required fields | Validity |
| --- | --- | --- |
| N-ST-01 PrimitiveType | kind: I8, I16, I32, I64, U8, U16, U32, U64, F32, F64, Char, String, or Bool; node_id; span | These are the primitive type tokens accepted by the parser. |
| N-ST-02 NamedType | name: Identifier; node_id; span | Becomes an opaque custom type. |
| N-ST-03 ArrayType | element: TypeSyntax; length: SurfaceExpr; node_id; span | Parser-produced array suffixes apply to primitive or named types and can nest with other array suffixes. The parser does not apply an array suffix after a vector type. Length must be a non-negative integer literal before construction of the typed type. |
| N-ST-04 VectorType | element: TypeSyntax; node_id; span | Element type must be a valid non-void type. |

The parser has no built-in `void` type token. An identifier spelled `void` is a NamedType with that exact name. The built-in Void type is synthesized only when a function has no return annotation. The parser does not accept nullptr as a type annotation. NullPtr is the type of a nullptr literal.

The current syntax profile is:

~~~text
program       = top-level-item*
top-level-item = function | main | statement
statement     = declaration | if-statement | while | for
              | return | break | continue | block | expression
function      = "fun" identifier "(" parameters? ")" (":" type)? block
main          = "main" block
declaration   = ("var" | "const") identifiers ":" type "=" initializers
if-statement  = "if" "(" expression ")" block ("else" (block | if-statement))?
while         = "while" "(" expression ")" block
for           = "for" "(" initializer? ";" condition? ";" increment? ")" statement
return        = "return" expression?
type          = (primitive-type | identifier) array-suffix* | vector-type
array-suffix  = "[" expression "]"
vector-type   = "vector" "<" type ">"
array-literal = "{" expressions? "}"
expression    = literal | identifier | "(" expression ")" | array-literal
              | prefix-op expression | expression infix-op expression
              | expression "as" cast-target
              | expression postfix-op | expression "(" arguments? ")"
              | expression "[" expression "]"
numeric-type  = "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
              | "f32" | "f64"
cast-target   = type
~~~

In this grammar, `block = "{" statement* "}"`; `parameter = identifier ":" type`; `parameters = parameter ("," parameter)*`; `identifiers = identifier ("," identifier)*`; `initializers = expression ("," expression)*`; `expressions = expression ("," expression)*`; and `arguments = expression ("," expression)*`. An array literal may have zero expressions. The parser reports E2001 if a declaration has different counts of identifiers and initializers.

For a for-statement, `initializer = declaration | expression`; `condition = expression`; and `increment = expression`. `break = "break"` and `continue = "continue"`. The `initializer`, `condition`, and `increment` clauses can be absent.

`literal` includes numeric, string, character, Boolean, and nullptr forms. `primitive-type` includes the type names in N-ST-01. `numeric-type` is limited to `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, and `f64`. `cast-target` uses the existing `type` production, so it supports primitive types, array types, vector types, and named custom types. `nullptr` is a value expression, not a type token. `prefix-op` includes `!`, unary `-`, `~`, `++`, and `--`. `infix-op` includes the operators in the precedence table, including `=`, but excludes dot and `as`. The cast production requires a valid `type` after `as`. `postfix-op` includes `++` and `--`. The parser permits any expression as a call callee or index base. Semantic rules then restrict calls and mutable places.

An array suffix applies to a primitive type or identifier type. Multiple suffixes are valid. The parser does not accept an array suffix after a `vector-type`. A vector type can contain an array type or another vector type as its element type.

Expression parsing uses the following binding powers. A larger binding power binds more tightly. The Pratt parser uses a loop condition `left_power > minimum_power` and parses each right operand with the listed right power. Under that rule, a row whose right power is greater than its left power is left-associative. A row whose right power is less than its left power is right-associative. Assignment and compound assignment are right-associative. All other binary rows are left-associative. For each infix row, the left power is 2 multiplied by the level. The right power is the left power plus 1 for a left-associative row and the left power minus 1 for a right-associative row. The prefix left power and the postfix right power are not used.

| Level | Syntax class | Operators | Associativity | Left power | Right power |
| ---: | --- | --- | --- | ---: | ---: |
| 1 | Assignment | =, +=, -=, *=, /=, %=, &=, \|=, ^=, <<=, >>= | Right | 2 | 1 |
| 2 | Logical OR | \|\| | Left | 4 | 5 |
| 3 | Logical AND | && | Left | 6 | 7 |
| 4 | Equality | ==, != | Left | 8 | 9 |
| 5 | Ordering | <, <=, >, >= | Left | 10 | 11 |
| 6 | Bitwise OR | \| | Left | 12 | 13 |
| 7 | Bitwise XOR | ^ | Left | 14 | 15 |
| 8 | Bitwise AND | & | Left | 16 | 17 |
| 9 | Shift | <<, >> | Left | 18 | 19 |
| 10 | Additive | +, - | Left | 20 | 21 |
| 11 | Multiplicative | *, /, % | Left | 22 | 23 |
| n/a | Prefix | !, -, ~, ++, -- | n/a | 24 | 23 |
| n/a | Cast | as cast-target | Left | 23 | 24 |
| n/a | Call, index, postfix update | (), [], ++, -- | n/a | 27 | 26 |

The parser's binding-power table also lists dot at power 27, but the parser has no dot-expression handler. A dot token in expression position is a syntax error. Member access is outside this subset.

| ID | Requirement | Verification |
| --- | --- | --- |
| SYN-001 | The Surface AST grammar and documented clause constraints MUST match the syntax profile in this section. | Parser-conformance tests cover every production, each optional clause, and each stated parser restriction. |
| SYN-002 | Expression AST shape MUST follow the binding powers and associativity in this table. | For every left-associative row, parse x op y op z and verify the shape (x op y) op z. For the assignment row, parse x op y op z and verify the shape x op (y op z). Verify that every infix table entry satisfies the power derivation rule in this section. |
| SYN-003 | A `fun` or `main` declaration MUST be a direct child of the program root. The parser MUST reject either declaration in a block or other statement position with syntax diagnostic E1004. The SurfaceProgram status MUST be Incomplete. | Parse declarations at the root and inside blocks, branches, and loop bodies. Verify root declarations pass, nested declarations report E1004, and semantic analysis does not run. |
| SYN-004 | A cast MUST use the form `expression as cast-target`, where cast-target follows the existing type production. Valid targets include numeric, character, Boolean, string, array, vector, and named custom types. Casts MUST bind more tightly than multiplicative operators and less tightly than prefix operators, and chained casts MUST associate left-to-right. | Parse each supported target family, reject `nullptr` as a type target, and assert AST shape for casts in arithmetic expressions and chained casts. |

Each declaration name has one initializer. The initializer count must equal the name count. Ordinary statements do not require a semicolon. Semicolons delimit the three for-loop clauses. A for-loop initializer is empty, a declaration, or an expression statement. Function and main declarations are top-level items. A block or other statement cannot contain either declaration.

An array type suffix is applied in source order. For example, `i32[2][3]` is an array of length 2 whose elements are arrays of length 3 of i32.

### 6.5 Fully Typed expression nodes

TypedExpr is abstract. Every concrete expression has node_id, span, and result_type: TypeId.

| Node ID and name | Required fields | Semantic function and validity |
| --- | --- | --- |
| N-TE-01 Literal | value: LiteralValue; result_type | Literal type follows TR-001 and TR-028. The Number variant of a numeric value corresponds to result_type: I64 to Integer, U64 to UnsignedInteger, F64 to Float64, and F32 to Float32. A Scientific form keeps its form at the new width. A prefix Negate applied directly to an adaptable literal is this node with the negated value (Section 8.2.1). |
| N-TE-02 VariableRef | name: Identifier; symbol: SymbolId; result_type | The symbol must identify a variable declaration with the same name. |
| N-TE-03 Grouping | expression: TypedExpr; result_type | Result type equals the child type. |
| N-TE-04 Unary | operator: UnaryOp; side: Prefix or Postfix; operand: UnaryOperand; result_type: TypeId | Increment and decrement use a mutable Place. Other unary operators use a value expression. A Negate applied directly to an adaptable literal does not produce this node (Section 8.2.1). |
| N-TE-05 Binary | left: TypedExpr; operator: NonAssignmentBinaryOp; right: TypedExpr; result_type | Operand conversions are explicit child nodes. For `<<` and `>>`, result_type equals left.result_type, and right has an integer type with no Convert node. |
| N-TE-06 CompoundAssign | target: Place; operator: CompoundOperator; value: TypedExpr; result_type: TypeId | The target is a mutable Place of type T. The operation runs in T. For operators other than `<<=` and `>>=`, value.result_type equals T, and a Convert child exists when the source type differs from T. For `<<=` and `>>=`, value has an integer type and no Convert child. result_type equals T. |
| N-TE-07 ArrayLiteral | elements: NonEmptyVec<TypedExpr>; result_type | Every element has an equivalent type. |
| N-TE-08 Call | callee_name: Identifier; callee_span: SourceSpan; function: SymbolId; arguments: Vec<TypedExpr>; result_type | Symbol identifies a function with the same name. Arguments match its signature. |
| N-TE-09 Assignment | target: Place; value: TypedExpr; result_type | Value converts to place_type. Result type equals place_type. |
| N-TE-10 Index | array: TypedExpr; index: TypedExpr; result_type | Result type equals the indexed element type. |
| N-TE-11 Convert | kind: ConversionKind; operand: TypedExpr; from: TypeId; to: TypeId; result_type: TypeId | result_type equals to. The conversion kind must be allowed by Section 7. |

Place is abstract. It is an AST node category with node_id, span, and place_type: TypeId. It is not a value expression. It can occur only as the target of assignment, compound assignment, increment, or decrement.

| Node ID and name | Required fields | Validity |
| --- | --- | --- |
| N-TP-01 VariablePlace | symbol: SymbolId; place_type | Symbol identifies a mutable variable. |
| N-TP-02 IndexPlace | base: Place; index: TypedExpr; place_type | Base is a mutable array or vector place. The index type is integer. The element type equals place_type. Nested index places form a chain rooted in a mutable variable place. |
| N-TP-03 GroupedPlace | place: Place; place_type | Transparent grouping around a variable or index place. It preserves source grouping and does not change place_type. It cannot make a non-place expression into a place. |

A Place cannot appear as a standalone statement or call argument. An index place owns its base place and index expression exactly once. This prevents a later phase from evaluating either child twice by mistake.

### 6.6 Fully Typed statement nodes

TypedStmt is abstract. Every concrete statement has node_id and span. Statements do not have an expression type. A statement type field is not applicable.

| Node ID and name | Required fields | Semantic function and validity |
| --- | --- | --- |
| N-TS-01 ExpressionStmt | expression: TypedExpr | Contains one typed expression. |
| N-TS-02 VarDecl | bindings: NonEmptyVec<TypedBinding>; declared_type: TypeId; mutable: bool | Every binding has a SymbolId and a typed initializer. Each initializer converts to declared_type. |
| N-TS-03 FunctionDecl | name: Identifier; symbol: SymbolId; parameters: Vec<TypedParameter>; signature: FunctionSignature; body: TypedBlock | Valid only as a direct child of TypedProgram. Symbol and signature agree. Parameter order matches source order. The outer body block uses the function scope. |
| N-TS-04 MainDecl | symbol: SymbolId; signature: FunctionSignature; body: TypedBlock | Valid only as a direct child of TypedProgram. Signature has no parameters and Void result. The outer body block uses the main function scope. |
| N-TS-05 If | condition: TypedExpr; then_block: TypedBlock; else_branch | Condition is bool. |
| N-TS-06 While | condition: TypedExpr; scope: ScopeId; body: TypedBlock | Condition is bool. The loop scope is distinct from the body block scope. |
| N-TS-07 For | initializer: Option<TypedStmt>; condition: Option<TypedExpr>; increment: Option<TypedExpr>; scope: ScopeId; body: TypedStmt | A present condition is bool. The initializer is empty, a declaration, or an expression statement. The body may be any statement, as in the current parser. The loop scope covers all clauses and the body. |
| N-TS-08 Block | statements: Vec<TypedStmt>; scope: ScopeId | Statements are in source order. |
| N-TS-09 Return | value: Option<TypedExpr>; function: SymbolId | Value presence and type agree with the function signature. |
| N-TS-10 Break | loop_context: LoopId | The ID refers to an enclosing loop. |
| N-TS-11 Continue | loop_context: LoopId | The ID refers to an enclosing loop. |

| Node ID and name | Required fields | Relationships and validity |
| --- | --- | --- |
| N-TD-01 TypedBinding | node_id: NodeId; name: Identifier; name_span: SourceSpan; symbol: SymbolId; declared_type: TypeId; initializer: TypedExpr; span: SourceSpan | Owned by exactly one Typed VarDecl. Its symbol type equals declared_type. Initializer result type equals declared_type after any Convert node. |
| N-TD-02 TypedParameter | node_id: NodeId; name: Identifier; name_span: SourceSpan; symbol: SymbolId; type: TypeId; span: SourceSpan | Owned by exactly one Typed FunctionDecl. Its symbol type equals type. |

FunctionSignature has fields `parameter_types: Vec<TypeId>` in parameter source order and `result_type: TypeId`. It is symbol metadata. It is not a first-class TypeId.

## 7. Type System

### 7.1 Type categories

| Type ID and category | Representation | Identity and validity |
| --- | --- | --- |
| TY-01 Signed integer | I8, I16, I32, I64 | Distinct primitive types. |
| TY-02 Unsigned integer | U8, U16, U32, U64 | Distinct primitive types. |
| TY-03 Floating point | F32, F64 | Distinct primitive types. |
| TY-04 Boolean | Bool | Primitive condition and logical type. |
| TY-05 Character | Char | Primitive character type. |
| TY-06 String | String | Primitive string type. |
| TY-07 Fixed array | Array(TypeId, u64 length) | Structural type. Element type and length are part of identity. The source length is a non-negative integer literal. The Fully Typed AST stores its u64 value. An inferred array literal uses its element count. |
| TY-08 Vector | Vector(TypeId) | Structural type. Element type is part of identity. |
| TY-09 Opaque named type | Named(Identifier) | Identity is exact identifier spelling. No declaration lookup or member access is defined. |
| TY-10 Void | Void | Function result and call result. It is not a storable value type. The current checker can still infer Array(Void,n) from an array literal of Void calls. |
| TY-11 Null pointer literal | NullPtr | Type of the nullptr literal. It is not a pointer declaration type. |

The system has no function type, generic type parameter, function overloading, union type, optional type, reference type, or error type.

The identifier `void` is not a built-in type token. An explicit annotation `: void` means Named(`void`). An omitted function result annotation means the built-in Void type. These types are different.

Void can occur as the result_type of a call expression. The checker also permits two Void call results as compatible comparison operands and permits an array literal whose elements have Void type. Such an array type is inferred only; source type syntax cannot declare it. These are current checker behaviors. They do not define runtime values for Void or Array(Void,n).

Error and incomplete states use diagnostic or checker state. They MUST NOT be stored as TypeIds in a successful Fully Typed AST.

### 7.2 Type identity and assignability

Type identity and assignability are different relations.

The relation T₁ ≡ T₂ is defined as follows:

1. Primitive types are equivalent only when their type IDs are equal.
2. Named types are equivalent only when their exact names are equal.
3. Vectors are equivalent when their element types are equivalent.
4. Arrays are equivalent when their element types are equivalent and their resolved lengths are equal.
5. Void and NullPtr are equivalent only to themselves.

Array and Vector equivalence is structural. Named type equivalence is nominal by exact identifier spelling. A named type does not resolve to a type declaration in this language subset.

The current checker also compares unresolved array-size expressions structurally in its is_same_type helper. The Fully Typed AST does not contain unresolved array lengths.

The assignability relation T₁ ⪯ T₂ is directional.

| ID | Requirement | Verification |
| --- | --- | --- |
| TY-12 | Type equivalence MUST follow the five rules above. | Compare equal and unequal primitive, named, vector, array, Void, and NullPtr types. |
| TY-13 | Assignability MUST follow the source-to-target table below. Every numeric row MUST preserve every value of the source type. The target type MUST represent each source value exactly. | Test each listed pair and each adjacent disallowed numeric pair in both directions. For every numeric pair, check the row against the value-preservation test that follows the table. Verify that I32 and U32 to F32, and I64 and U64 to F64, are not assignable. |

| Source T₁ | Target T₂ | Assignable |
| --- | --- | --- |
| Any type | Same type | Yes |
| I8 | I16, I32, I64, F32, F64 | Yes |
| I16 | I32, I64, F32, F64 | Yes |
| I32 | I64, F64 | Yes |
| I64 | None | Not applicable |
| U8 | U16, U32, U64, I16, I32, I64, F32, F64 | Yes |
| U16 | U32, U64, I32, I64, F32, F64 | Yes |
| U32 | U64, I64, F64 | Yes |
| U64 | None | Not applicable |
| F32 | F64 | Yes |
| Char | String | Yes |
| Array(A,n) | Array(B,m) | Yes if n=m and A ⪯ B |
| Vector(A) | Vector(B) | Yes if A ⪯ B |
| NullPtr | Array, Vector, or Named type | Yes |
| Any other pair | Any other pair | No |

Every numeric row preserves every value of the source type (DEC-017). An integer row to a floating-point type exists only when the floating-point significand holds every magnitude of the integer type. F32 has a 24-bit significand and F64 has a 53-bit significand. The value-preservation test for a numeric source S and target T is:

1. If S and T are both signed integers or both unsigned integers, T has at least as many bits as S.
2. If S is an unsigned integer and T is a signed integer, T has more bits than S.
3. If S is a signed integer and T is an unsigned integer, the pair is not assignable.
4. If S is an integer and T is a floating-point type, the number of magnitude bits of S is at most the significand width of T. The magnitude bits are 7, 15, 31, and 63 for I8, I16, I32, and I64, and 8, 16, 32, and 64 for U8, U16, U32, and U64.
5. If S is a floating-point type, T is a floating-point type with a significand at least as wide.
6. If S is a floating-point type and T is an integer type, the pair is not assignable.

The rows from U8, U16, and U32 to a wider signed integer are the only conversions from an unsigned type to a signed type. They let a value widen to a signed type without a cast (DEC-016). The conversions from I32 and U32 to F32, and from I64 and U64 to F64, are not in the table because the floating-point target cannot represent every value of the integer source type exactly. The thresholds 2^24 for F32 and 2^53 for F64 mark the ranges within which every integer is exactly representable. Some integers above these thresholds remain exactly representable, but others lose precision. A value of type I64 or U64 has no implicit conversion to any other type. An explicit numeric cast is available when a conversion is required (Section 7.4, DEC-020). The exactness test in TR-028a applies the same principle to a literal.

The array and vector rules preserve current checker behavior. They allow recursively assignable element types. They are not invariant.

NullPtr is not assignable to String or primitive types.

### 7.3 Numeric promotion

The checker uses this preference order:

F64, F32, U64, I64, U32, I32, U16, I16, U8, I8.

For two numeric operand types, Promote(T₁,T₂) selects the first type C in this list that is equal to either operand. The promotion is defined only when T₁ ⪯ C and T₂ ⪯ C. Otherwise the operation reports E2036 and has no result type. **Observed implementation gap:** the current checker does not yet enforce the specified value-preservation condition (GAP-015). The required behavior in this specification does enforce it.

Promotion uses only the conversions in the assignment table in Section 7.2. The Fully Typed AST records each operand conversion as a NumericAssignable conversion.

The required promotion procedure is deterministic:

~~~text
promote(T1, T2):
    require T1 and T2 are numeric types
    for C in [F64, F32, U64, I64, U32, I32, U16, I16, U8, I8]:
        if (T1 ≡ C or T2 ≡ C) and T1 ⪯ C and T2 ⪯ C:
            return C
    report E2036
    return no type
~~~

The procedure MUST use the assignability relation in Section 7.2, not host-language casts or value-dependent guesses. It MUST NOT inspect runtime values. The ordered table contains ten entries, so the procedure takes O(1) time and O(1) additional space under the specified fixed type set. These bounds describe the algorithm, not measured end-to-end compiler performance. A successful result preserves every value of both operand types because both assignability premises must hold. A failed search produces E2036 and MUST NOT produce a typed binary expression.

Of the 100 ordered pairs of numeric types, 68 have a promoted type. These sixteen pairs have none, in either operand order: F32 with I32, I64, U32, or U64; F64 with I64 or U64; U64 with I64, I32, I16, or I8; U32 with I32, I16, or I8; U16 with I16 or I8; U8 with I8. No promotion rounds an operand, because every conversion in the table in Section 7.2 preserves every value.

The order does not define arithmetic overflow, shift-count range, floating-point rounding, or runtime conversion behavior. Those rules belong to a later execution or lowering specification.

| ID | Requirement | Verification |
|---|---|---|
| TY-14 | Numeric promotion MUST select the earliest matching type in the stated order for every pair of numeric types. It MUST report E2036 when either operand is not assignable to that type. | Exhaust all 10 × 10 ordered numeric type pairs. Assert the selected TypeId for 68 pairs and E2036 for the 32 ordered pairs listed above. |

### 7.4 Conversion nodes

ConversionKind has these variants:

| Kind | Requirement |
| --- | --- |
| NumericAssignable | Source and target are numeric, and the pair is in the assignment table in Section 7.2. The kind covers assignment conversions and the operand conversions of Section 7.3. Every conversion of this kind preserves every value of the source type (DEC-017). |
| ExplicitCast | Numeric-to-numeric, numeric-to-char, char-to-numeric, numeric-to-bool, and bool-to-numeric conversions are allowed only when requested by source syntax `expression as target-type` (DEC-020). Char-to-string and null-pointer-to-array, vector, or named custom type use the corresponding existing conversion rules, and same-type casts are identity operations. String parsing, unrelated aggregate conversions, and pointer reinterpretation are not supported. Runtime value semantics for value-changing casts are not defined by the current front-end specification and MUST be specified before lowering executes them. |
| CharToString | Source is Char and target is String. |
| NullToAggregate | Source is NullPtr and target is Array, Vector, or Named. |
| ArrayElementConversion | Source and target arrays have equal length, and source element is assignable to target element. |
| VectorElementConversion | Source and target vectors have assignable element types. |

Identity conversion does not create a Convert node. An explicit cast becomes an ExplicitCast conversion when source and target types differ and the pair is not covered by an existing conversion kind. Char-to-string uses CharToString; null-pointer-to-array, vector, or named custom type uses NullToAggregate. Supported ExplicitCast pairs are numeric-to-numeric, numeric-to-char, char-to-numeric, numeric-to-bool, and bool-to-numeric. String parsing, unrelated aggregate conversions, and pointer reinterpretation MUST be rejected during type checking. Explicit casts do not modify the implicit assignability table in Section 7.2.

| ID | Requirement | Verification |
|---|---|---|
| TY-15 | A non-identity implicit conversion MUST use one of the conversion kinds above and MUST satisfy that kind's source and target conditions. | Construct each allowed implicit conversion and one invalid source/target pair for every implicit conversion kind. |
| TY-18 | ExplicitCast MUST accept numeric source and target types, numeric-to-char, char-to-numeric, numeric-to-bool, and bool-to-numeric pairs, including narrowing pairs excluded from implicit assignment. Char-to-string and null-pointer-to-array, vector, or custom type MUST use CharToString and NullToAggregate respectively. Same-type casts MUST be accepted as identity operations. String parsing, unrelated aggregate conversions, and pointer reinterpretation MUST remain invalid. | Test numeric pairs, char/string, null-pointer aggregate targets, same-type casts, and invalid conversions. |

### 7.5 Declared, inferred, and result types

| Element | Type source | Fully Typed AST field | Reason |
| --- | --- | --- | --- |
| Variable declaration | Required source annotation | TypedBinding.declared_type and variable symbol type | Variable type inference is not supported. |
| Parameter | Required source annotation | TypedParameter.type and parameter symbol type | Every parameter has an explicit annotation. |
| Function result | Source annotation, or synthesized Void when absent | FunctionSignature.result_type | A function has one result type. |
| Main function | Fixed zero-parameter Void signature | FunctionSignature | Main is a function declaration, not a first-class value. |
| Other function declaration | Ordered parameter and result metadata | FunctionSignature and Function SymbolRecord | Function values and function TypeIds are not supported. |
| Literal expression | Literal kind, numeric suffix, and context type (TR-028), or the default type | TypedExpr.result_type | Every expression has a type after successful checking. |
| Other expression | Typing rule in Section 8 | TypedExpr.result_type | Every expression has a type after successful checking. |
| Place | Variable symbol or indexed element type | Place.place_type | A place denotes storage. It is not a value expression. |
| Converted expression | Conversion target | Convert.result_type and `to` | The result type is the target type. |
| Surface AST node | Not yet resolved | No TypeId | The Surface AST records syntax before semantic analysis. |
| TypeSyntax | Not yet resolved | No TypeId in the Surface AST | TypeSyntax is input to type resolution. |
| Program, block, and statement | No expression value | No type field | These nodes control declarations and execution. |
| Error or incomplete expression | No valid type | No TypeId; diagnostic stays outside the AST | Error states are not source-language types. |

Variable type inference is not supported. Expression type synthesis is supported and is mandatory.

| ID | Requirement | Verification |
| --- | --- | --- |
| TY-16 | Declared types, inferred expression types, result types, and non-applicable statement types MUST use the fields in the table above. | Inspect a typed sample containing every declaration and expression category. Verify statement nodes have no type field. |
| TY-17 | TypeTable MUST use the canonicalization keys in Section 5.3. | Intern each primitive and composite type twice. Verify the same TypeId is returned for equivalent keys and a distinct TypeId is returned for non-equivalent keys. |
| TY-19 | Type resolution MUST recursively resolve child types before interning composite types. An invalid child type or array length MUST prevent insertion of its parent key. `intern` MUST return one canonical TypeId per key in a compilation unit and MUST NOT use source spans or AST node IDs as type identity. | Resolve repeated primitive, named, nested-array, and nested-vector syntax. Verify shared canonical IDs, distinct IDs for non-equivalent keys, and no parent entry after an invalid child or length. |

## 8. Typing Rules

### 8.1 General rule

The type checker MUST visit every expression. It MUST produce one result type for each expression when no error exists. It MUST report errors and continue through independent expressions where safe.

An error result is internal checker state. It is not a type. A failed check cannot produce a Fully Typed AST.

Every rule in this section has the following initial conditions: the Surface AST is complete and structurally valid; parser diagnostics are absent; the active lexical environment is established; each child expression required by the rule has been checked; and the canonical TypeTable is available. The rule-specific tables give the remaining preconditions and errors. A rule succeeds only when all its preconditions hold. Its conclusion is the postcondition. Its result node MUST satisfy the node and invariant requirements in Sections 6 and 10. A failed rule MUST report its listed source diagnostic and MUST NOT produce a successful typed node.

The notation `Γ ⊢ e : T` means expression `e` has type `T`. The notation `Γ ⊢ p : Place(T)` means `p` is a storage place of type `T`. `Mutable(p)` means that the place is rooted in a mutable variable. `Γ ⊢ τ ⇒ T` means type syntax `τ` resolves to TypeId `T`. `Γ;F;L ⊢ s : ok` means statement `s` passes in environment `Γ`, current function `F`, and active loop stack `L`. `x` and `f` are names. `s` in `Variable(s,T)` is a SymbolId. `Num` and `Int` are defined in Section 8.4. `Common(T₁,T₂)` is the common type selected by the comparison rule in Section 8.4. `HasReturn(s)` is the current return heuristic defined below.

The core rules are:

$$
\frac{\Gamma(x)=\operatorname{Variable}(s,T)}
     {\Gamma\vdash x:T}
\quad\text{(TR-003)}
$$

$$
\frac{\Gamma\vdash e_1:T_1 \qquad \Gamma\vdash e_2:T_2 \qquad
      T_1,T_2\in Num \qquad C=\operatorname{Promote}(T_1,T_2) \qquad
      T_1\preceq C \qquad T_2\preceq C}
     {\Gamma\vdash e_1\ op\ e_2:C}
\quad\text{(TR-009)}
$$

The binary rule inserts a NumericAssignable conversion on each operand whose type is not C. If T₁ ⪯ C or T₂ ⪯ C does not hold, the rule reports E2036.

$$
\frac{\Gamma\vdash p:\operatorname{Place}(T) \qquad
      \Gamma\vdash e:S \qquad S\preceq T \qquad p\text{ is mutable}}
     {\Gamma\vdash p=e:T}
\quad\text{(TR-014)}
$$

$$
\frac{\Gamma(f)=\operatorname{Function}(s,[T_1,\ldots,T_n],R)
      \qquad \Gamma\vdash e_i:S_i \qquad S_i\preceq T_i\ (1\le i\le n)}
     {\Gamma\vdash f(e_1,\ldots,e_n):R}
\quad\text{(TR-017)}
$$

The call rule also requires exactly n arguments. All symbols in these rules use the definitions in Section 3.

### 8.2 Literal, name, and grouping rules

| Rule ID | Rule and postcondition | Possible error |
| --- | --- | --- |
| TR-001 Literal | A numeric literal with `width_suffix` true has the type of its parsed Number variant. A numeric literal with `width_suffix` false takes the type that TR-028 selects. When TR-028 supplies no context type or does not adapt the literal, the default type applies: Integer is I64, UnsignedInteger is U64, and floating point is F64. | E2035 from TR-028. An invalid literal is a lexer or parser error. |
| TR-002 Literal | String, character, Boolean, and nullptr literals have String, Char, Bool, and NullPtr types. | Invalid literal payload is a construction error. |
| TR-003 Variable | If Γ(name) is a variable symbol with type T, then Γ ⊢ name : T and the node stores that SymbolId. | E2022 if the nearest declaration is a function; E2023 if no variable exists. |
| TR-004 Grouping | If Γ ⊢ e : T, then Γ ⊢ (e) : T. The result type equals the child type. | Child error propagates. |
| TR-028 Literal context | A flexible expression takes its type from its context under Section 8.2.1. | E2035 when a literal does not fit its context type. |

### 8.2.1 Context typing of numeric literals

An adaptable literal is a numeric Literal with `width_suffix` false. A literal with `width_suffix` true always has the type of its Number variant.

A flexible expression is one of the following:

1. An adaptable literal.
2. A Grouping of a flexible expression.
3. A prefix Negate or BitwiseNot of a flexible expression.
4. A Binary node with operator `+`, `-`, `*`, `/`, `%`, `&`, `|`, or `^` whose two operands are flexible expressions.
5. A Binary node with operator `<<` or `>>` whose left operand is a flexible expression. The right operand can be any expression.

A context type is a type that the position of an expression supplies to it. The positions are:

1. The declared type, for the initializer of a binding.
2. The place type, for the value of an assignment.
3. The function result type, for a return value.
4. The parameter type, for a call argument.
5. The element type E, for each element of an array literal that has context type Array(E,n). When E is an array type, it is the context type of a nested array literal.
6. The place type, for the value of a compound assignment other than `<<=` and `>>=`.

| Rule ID | Rule and postcondition | Possible error |
| --- | --- | --- |
| TR-028a Leaf adaptation | An adaptable literal that receives a numeric context type T takes type T when its signed value v passes the test for its kind. An integer literal with an integer T passes when v is within the range of T. An integer literal with F32 or F64 passes when v is exactly representable in T. A floating-point literal with F32 or F64 passes when its value is finite after rounding to nearest in T. A floating-point literal with an integer T is not compatible by kind. It keeps its default type. Leaf adaptation reports no error for this kind mismatch; the enclosing rule still checks assignability. | E2035 when the literal is compatible by kind and fails the test. |
| TR-028b Passing | Only a flexible expression passes its context type to its children. Grouping passes any numeric T. Negate, BitwiseNot, and the Binary operators `+ - * / % & \| ^` pass an integer T to every operand. A shift passes an integer T to its left operand only. A Binary node MUST NOT pass its incoming F32 or F64 context to an operand. This restriction does not override the separate peer rule in TR-028c. Operand and result types then follow TR-009 to TR-012. | None. |
| TR-028c Peer | For `+ - * / % & \| ^ == != < <= > >=`, if exactly one operand is flexible and the other operand is not flexible and has numeric type S, the flexible operand receives S as its context type. This replaces the context type from the position of the Binary node. This peer context may be F32 or F64. If both operands are flexible, both receive the context type of the Binary node only when TR-028b permits that context to pass. Therefore, an incoming F32 or F64 context is not passed through an operator to two flexible operands. Shifts have no peer rule. | None. |
| TR-028d No context | A flexible expression with no context type takes default types, and operators apply TR-009 to TR-012 to those types. Index expressions, conditions, shift counts, and operands of non-flexible unary operators receive no context type. A non-numeric context type has no effect. | None. |

For example, `var value: i32 = 1.5` leaves `1.5` at F64 under TR-028a. Leaf adaptation does not report an error, but TR-015 reports E2002 because F64 is not assignable to I32. The absence of an error from TR-028a does not mean that the enclosing expression is valid.

A prefix Negate applied directly to an adaptable literal MUST be one Literal node in the typed tree. The node holds the negated value, and its span covers both tokens. The most negative value of a signed type has no positive counterpart in that type, so `-128` for I8 cannot be a Negate over an I8 literal. A Negate applied to any other expression, including `-(128)` and `- -5`, is a Unary node. A literal with `width_suffix` true is not folded.

The Number variant of a typed numeric Literal corresponds to its result_type, and its value MUST be representable in that type ([INV-022](#inv-022), defined in Section 10).

### 8.3 Unary rules

| Rule ID | Preconditions | Result |
| --- | --- | --- |
| TR-005 Negate | Operand is numeric. | Operand type. |
| TR-006 Logical not | Operand type is Bool. | Bool. |
| TR-007 Bitwise not | Operand is an integer type. | Operand type. |
| TR-008 Increment or decrement | Operand is a mutable Place and has numeric type. A variable place or index-place chain must be rooted in a mutable variable. | Place type. Prefix or postfix form does not change the type. |

The checker reports E2018 for invalid numeric or integer operands, E2019 for invalid logical-not operands, and E2024 for an immutable target. Under [DEC-008](#dec-008), defined in Section 15.1, a non-place operand also reports E2033.

### 8.4 Binary rules

Let Num be the ten integer and floating-point types. Let Int be the eight integer types. Let Compatible(A,B) mean A ⪯ B or B ⪯ A.

| Rule ID | Operators | Preconditions | Result type |
| --- | --- | --- | --- |
| TR-009 Arithmetic | +, -, *, /, % | Both operands are in Num. Promote both operands to C = Promote(T₁,T₂). Both operands MUST be assignable to C, otherwise E2036. A flexible operand takes its type from TR-028c before promotion. | Promoted type. |
| TR-010 Comparison | ==, !=, <, <=, >, >= | Ordering operators (<, <=, >, >=) require numeric operands and promote them under the TR-009 condition. Equality operators (==, !=) use numeric promotion for numeric operands. For non-numeric equality operands, Compatible(T₁,T₂) must be true. Convert operands to the common type defined below. A flexible operand takes its type from TR-028c. | Bool. |
| TR-011 Logical | &&, \|\| | Both operands are Bool. | Bool. |
| TR-012 Bitwise | &, \|, ^, <<, >> | Both operands are in Int. For `&`, `\|`, and `^`, promote both operands to C = Promote(T₁,T₂) under the TR-009 condition. For `<<` and `>>`, do not promote. The right operand keeps its own integer type and receives no conversion. | For `&`, `\|`, and `^`, the promoted type. For `<<` and `>>`, the type of the left operand. |
| TR-013 Compound assignment | +=, -=, *=, /=, %=, &=, \|=, ^=, <<=, >>= | The target is a mutable place of type T. For `+=`, `-=`, `*=`, `/=`, and `%=`, T is in Num. For `&=`, `\|=`, and `^=`, T is in Int. The value is checked with context type T under TR-028, and its type S satisfies S ⪯ T. The value receives a conversion to T when S is not T. The operation runs in T. For `<<=` and `>>=`, T and S are integer types and the value is not converted. | Place type. |

The current checker permits modulo for every numeric type. This specification preserves that rule. Equality comparisons use Compatible for non-numeric operands. Ordering operators require numeric operands.

For a non-numeric comparison, choose the common operand type as follows: if T₁ ≡ T₂, use T₁; otherwise, if T₁ ⪯ T₂, convert the left operand to T₂; otherwise, if T₂ ⪯ T₁, convert the right operand to T₁. The comparison is invalid if neither direction holds. For numeric comparisons, use Promote(T₁,T₂) and convert both operands to that type. These conversion nodes make the current checker's compatibility result explicit in the Fully Typed AST. They do not add accepted operand pairs.

Ordering operators accept numeric operands only. The checker MUST reject non-numeric operands for <, <=, >, and >=, even when their types are compatible. This restriction prevents the Fully Typed AST from containing ordering operations without defined runtime semantics. Equality operators retain the existing compatibility rule for non-numeric operands.

TR-013 requires the value to be assignable to the place type. For example, an I16 value cannot be applied to an I8 target. It also requires a valid mutable place.

The checker reports E2011 for invalid bitwise operands, E2012 or E2017 for invalid logical operands, E2013 or E2016 for invalid arithmetic operands, and E2014 for incompatible comparison operands. It reports E2024 for an immutable compound-assignment target, E2033 for a non-place target under [DEC-008](#dec-008), defined in Section 15.1, E2002 when the value cannot convert to the place type, E2035 when a literal does not fit its context type, and E2036 when numeric operands have no common type.

### 8.5 Assignment, declaration, call, and index rules

| Rule ID | Preconditions | Postcondition | Possible errors |
| --- | --- | --- | --- |
| TR-014 Assignment | Target is a mutable variable or an index-place chain rooted in a mutable array/vector variable. The value is checked with context type T₂, and its type T₁ is assignable to T₂. | Insert a conversion if needed. Result type is T₂. | E1003 if parsing rejects a target outside variable/index syntax; E2024 immutable target; E2023 undefined variable; E2033 parsed expression is not a mutable place; E2002 incompatible value. |
| TR-015 Variable declaration | Annotation resolves to T. Each initializer is checked with context type T, and its type Tᵢ is assignable to T. | Each variable symbol has declared type T. The initializer is converted to T. | E2002 incompatible initializer; E2032 duplicate name. |
| TR-016 Function declaration | Parameter types and result type resolve. Each parameter name is unique in the function scope. | Signature is the ordered parameter type list and result type. An omitted result annotation gives Void. | E2032 duplicate parameter; invalid annotation diagnostic. |
| TR-017 Function call | Callee is a simple name that resolves to a function. Argument count equals parameter count. Each argument is checked with its parameter type as context type and is assignable to that type. | Call result type is the function result type. Each needed argument conversion is explicit. | E2026 non-name callee; E2027 unknown function; E2028 wrong count; E2029 incompatible argument. |
| TR-018 Array literal | At least one element. When the literal has context type Array(E,n), each element is checked with context type E. Every element has an equivalent type. | Result is Array(element type, element count). No element promotion occurs. | E2020 empty literal; E2021 mixed element types. |
| TR-019 Index | Base has Array(T,n) or Vector(T). Index has an integer type. | Result type is T. | E2030 non-integer index; E2031 non-indexable base. |

The current parser accepts only a simple variable name as a function call target after semantic checking. Function values and indirect calls are not supported.

### 8.6 Statement and control-flow rules

| Rule ID | Preconditions | Postcondition | Possible errors |
| --- | --- | --- | --- |
| TR-020 If | Condition has Bool type. | Both branches are checked in their lexical scopes. | E2004 non-Boolean condition. |
| TR-021 While | Condition has Bool type. | Body is checked with loop context active. | E2004 non-Boolean condition. |
| TR-022 For | Present condition has Bool type. | Initializer scope covers condition, increment, and body. Loop context is active in the body. | E2004 non-Boolean condition. |
| TR-023 Return | A current function exists. A value is absent only for Void result. A present value is checked with the function result type as context type and is assignable to that type. | Insert a conversion when needed. Return node links to its Function SymbolId. | E2005 outside function; E2006 value from Void; E2007 mismatch; E2008 missing value. |
| TR-024 Break or continue | An active loop exists in the current function. | Node links to the nearest LoopId in that function. | E2009 break outside loop; E2010 continue outside loop. |
| TR-025 Non-void function return check | For a non-void result, the function body MUST NOT be able to complete normally without a return. The analysis MUST process block statements in order, require both branches of an if/else to terminate to prove no fall-through, and treat every while/for loop as potentially completing normally. | Accept only when the body cannot reach its end through normal control flow. | E2003 when fall-through is possible. |
| TR-026 Array length | Length is a non-negative integer literal that fits u64. An inferred array-literal length is its element count and must also fit u64. | The resolved array type stores that u64 length. | E2034 for a non-literal, negative, or out-of-range length. |

Assignment, compound assignment, increment, and decrement require a place. The place must be a variable or an index chain rooted in a mutable variable. An array literal, call result, or other temporary expression cannot be a place.

The non-void return-path rule is a conservative normal-completion analysis:

1. A Return statement cannot complete normally.
2. A Block can complete normally if every statement in sequence can complete normally. Once a statement cannot complete normally, later statements do not restore normal completion.
3. An If can complete normally if either branch can complete normally. An If without an else can always complete normally because its condition can be false.
4. A While or For is always treated as potentially completing normally, regardless of returns in its body. The current checker does not prove that a loop condition is always true or that the loop cannot exit.
5. Other statements can complete normally.

A non-void function MUST produce E2003 when this analysis says its body can complete normally. This rule rejects a return that occurs only inside a loop, while accepting a guaranteed return after a loop or returns in both branches of an if/else. The analysis is conservative rather than a complete proof of runtime control flow.

### 8.7 Type syntax resolution and array lengths

The Fully Typed AST stores array length as a u64 constant. The source form must be a non-negative integer literal. The parser may retain other expressions in an incomplete Surface AST, but semantic analysis must report an invalid array-length diagnostic for them.

This rule reflects the confirmed array-length decision in [DEC-007](#dec-007), defined in Section 15.1. It resolves the current parser/checker mismatch.

| Rule ID | Preconditions | Postcondition | Possible error |
|---|---|---|---|
| TR-027 Type syntax resolution | The TypeSyntax node is structurally valid. Primitive types use the canonical primitive entry. A named type uses its exact name and does not require declaration lookup. Vector and array element syntax resolves recursively. Array length passes TR-026. | Return the canonical TypeId for the resolved primitive, Named, Vector, or Array type. Vector element type MUST NOT be built-in Void. | E2034 for an invalid array length; parser diagnostic for invalid TypeSyntax. |

### 8.8 Formal forms for all typing rules

The following forms state the premises and conclusion for every rule in TR-001 through TR-028. The rule tables in Sections 8.2 through 8.7 define each rule's diagnostics and AST postconditions.

| Rule | Formal premises and conclusion |
| --- | --- |
| TR-001 | `LiteralType(e)=T ⇒ Γ ⊢ e:T`. `LiteralType` uses the parsed numeric variant when `width_suffix` is true. When it is false, `LiteralType` uses the type from TR-028 if a context type adapts the literal, and otherwise the defaults in TR-001. |
| TR-002 | `e` is a string, character, Boolean, or nullptr literal with kind type `T ⇒ Γ ⊢ e:T`. |
| TR-003 | `Γ(x)=Variable(s,T) ⇒ Γ ⊢ x:T`, and the typed reference stores `s`. |
| TR-004 | `Γ ⊢ e:T ⇒ Γ ⊢ (e):T`. |
| TR-005 | `Γ ⊢ e:T` and `T∈Num ⇒ Γ ⊢ -e:T`. |
| TR-006 | `Γ ⊢ e:Bool ⇒ Γ ⊢ !e:Bool`. |
| TR-007 | `Γ ⊢ e:T` and `T∈Int ⇒ Γ ⊢ ~e:T`. |
| TR-008 | `Γ ⊢ p:Place(T)`, `Mutable(p)`, and `T∈Num ⇒ Γ ⊢ ++p:T` or `Γ ⊢ p++:T`; the same rule applies to decrement. |
| TR-009 | `Γ ⊢ e₁:T₁`, `Γ ⊢ e₂:T₂`, `T₁,T₂∈Num`, `C=Promote(T₁,T₂)`, `T₁⪯C`, and `T₂⪯C ⇒ Γ ⊢ e₁ op e₂:C`, for `op∈{+,-,*,/,%}`. |
| TR-010 | `Γ ⊢ e₁:T₁`, `Γ ⊢ e₂:T₂`, `Common(T₁,T₂)=C ⇒ Γ ⊢ e₁ cmp e₂:Bool`, for `cmp∈{==,!=,<,<=,>,>=}`. Ordering operators require `T₁,T₂ ∈ Num` and use promotion under the TR-009 condition. Equality uses promotion for numeric operands and assignability compatibility for non-numeric operands. |
| TR-011 | `Γ ⊢ e₁:Bool` and `Γ ⊢ e₂:Bool ⇒ Γ ⊢ e₁ op e₂:Bool`, for `op∈{&&,\|\|}`. |
| TR-012 | `Γ ⊢ e₁:T₁`, `Γ ⊢ e₂:T₂`, `T₁,T₂∈Int`, `C=Promote(T₁,T₂)`, `T₁⪯C`, and `T₂⪯C ⇒ Γ ⊢ e₁ op e₂:C`, for `op∈{&,\|,^}`. For `op∈{<<,>>}`: `Γ ⊢ e₁:T₁`, `Γ ⊢ e₂:T₂`, and `T₁,T₂∈Int ⇒ Γ ⊢ e₁ op e₂:T₁`. |
| TR-013 | `Γ ⊢ p:Place(T)`, `Mutable(p)`, `T∈Num` (`T∈Int` for `&=`, `\|=`, and `^=`), `Γ ⊢ e ⇐ T ⇒ S`, and `S⪯T ⇒ Γ ⊢ p op= e:T`. For `<<=` and `>>=`: `T∈Int`, `Γ ⊢ e:S`, and `S∈Int ⇒ Γ ⊢ p op= e:T`. |
| TR-014 | `Γ ⊢ p:Place(T)`, `Mutable(p)`, `Γ ⊢ e:S`, and `S⪯T ⇒ Γ ⊢ p=e:T`. |
| TR-015 | `Γ ⊢ τ⇒T`; each initializer `eᵢ` has type `Sᵢ`; every `Sᵢ⪯T`; and the declared names are unique in their scope `⇒ Γ ⊢ varDecl:ok`. |
| TR-016 | Parameter and result syntax resolve to `[T₁,…,Tₙ]` and `R`; parameter names are unique; and the body passes under the function environment `⇒ Γ ⊢ functionDecl:ok` with signature `([T₁,…,Tₙ],R)`. An omitted result annotation resolves to Void. |
| TR-017 | `Γ(f)=Function(s,[T₁,…,Tₙ],R)`; there are exactly `n` arguments; each `Γ ⊢ eᵢ:Sᵢ`; and each `Sᵢ⪯Tᵢ ⇒ Γ ⊢ f(e₁,…,eₙ):R`. The call stores `s`. |
| TR-018 | `n>0`; each `Γ ⊢ eᵢ:Tᵢ`; and every `Tᵢ≡T ⇒ Γ ⊢ {e₁,…,eₙ}:Array(T,n)`. |
| TR-019 | `Γ ⊢ a:Array(T,n)` or `Γ ⊢ a:Vector(T)`; `Γ ⊢ i:I`; and `I∈Int ⇒ Γ ⊢ a[i]:T`. This rule does not perform a compile-time bounds check. |
| TR-020 | `Γ ⊢ c:Bool`; `Γ;F;L ⊢ thenBlock:ok`; and each present else branch passes in its branch scope `⇒ Γ;F;L ⊢ if(c):ok`. |
| TR-021 | `Γ ⊢ c:Bool`; and `Γ;F;L·loop ⊢ body:ok ⇒ Γ;F;L ⊢ while(c,body):ok`. |
| TR-022 | Every present initializer, condition, and increment passes in the for-scope; a present condition has type Bool; and `Γ;F;L·loop ⊢ body:ok ⇒ Γ;F;L ⊢ for(...):ok`. |
| TR-023 | For a Void function `F`, `Γ;F;L ⊢ return:ok`. For a non-Void function with result `R`, `Γ ⊢ e:S` and `S⪯R ⇒ Γ;F;L ⊢ return e:ok`. |
| TR-024 | `L` contains an active loop `l` whose `function` is `F ⇒ Γ;F;L ⊢ break:ok` or `continue:ok`, and the node stores the nearest such `LoopId`. |
| TR-025 | If a function result is Void, no return-path condition applies. If its result is non-Void, `MayCompleteNormally(body)=false ⇒ Γ ⊢ functionDecl:ok`; otherwise report E2003. `MayCompleteNormally` follows Section 8.6 and treats every loop as potentially completing normally. |
| TR-026 | For array type syntax with length literal `k`, `0≤k≤MAX_U64 ⇒ Length(k)=k`. For an inferred array literal with `n` elements, `0≤n≤MAX_U64 ⇒ Length(elements)=n`. |
| TR-027 | `Γ ⊢ primitiveSyntax ⇒ its canonical primitive TypeId`; `Γ ⊢ namedSyntax(x) ⇒ Named(x)`; `Γ ⊢ τ⇒T` and `T≠Void ⇒ Γ ⊢ Vector(τ)⇒Vector(T)`; `Γ ⊢ τ⇒T` and `Length(k)=n ⇒ Γ ⊢ Array(τ,k)⇒Array(T,n)`. Named(`"void"`) is not the built-in Void type. |
| TR-028 | `e` is an adaptable literal, `T` is numeric, and the signed value `v` of `e` passes the TR-028a test for its kind `⇒ Γ ⊢ e ⇐ T ⇒ T`. A flexible expression passes `T` to its operands under TR-028b. A flexible operand next to a non-flexible operand of numeric type `S` is checked with `S` under TR-028c. |

`MAX_U64` means the largest value representable by an unsigned 64-bit integer. A type-resolution error or a false premise has the diagnostic listed for that rule.

### 8.9 Type-checking pass procedure

This section defines the required pass behavior that connects the local rules in Sections 7 and 8. It specifies the observable contract, not a required class hierarchy, visitor framework, or implementation language.

#### 8.9.1 Pass inputs and output states

The type-checking operation accepts only a complete, structurally valid SurfaceProgram with no parser diagnostics, the current CompilationUnitId, the canonical TypeTable, and the symbol-resolution result required by Sections 5 and 9. A type checker MUST reject an Incomplete SurfaceProgram before checking any source expression (TC-001).

The pass has two externally observable outcomes:

- **Success:** a Fully Typed AST whose nodes satisfy the node definitions in Section 6 and all applicable invariants in Section 10.
- **Failure:** diagnostics and no successful Fully Typed AST.

The checker MAY retain private intermediate state while it checks a program. Such state MAY include an internal failure marker, temporary inferred types, a partially built typed node, and a diagnostic accumulator. These are implementation details. A failure marker MUST NOT be represented as a source-language TypeId or exposed as a successful Typed AST node (TC-004, TC-006, INV-012).

#### 8.9.2 Expression checking operation

The following pseudocode illustrates the expression-checking sequence governed by TC-001 through TC-004 and the typing rules in Sections 7 and 8. It is an implementation guide, not a separate interface requirement.

```text
check_expression(surface_expr, optional_context_type, environment):
    validate that surface_expr is a complete, structurally valid node
    determine child contexts only from TR-028 and the applicable typing rule
    check required child expressions in deterministic field/source order
    apply the typing rule's preconditions to the child results
    if a required child failed or a rule precondition is false:
        record the rule's source diagnostic when applicable
        return failure state, not a typed expression
    determine the canonical result TypeId
    insert each conversion required by the rule
    construct the specified typed node with resolved IDs and source span
    validate the constructed node's local invariants
    return the typed expression
```

A rule MUST NOT infer a context merely because a destination type exists elsewhere in the AST (TC-003). Context is passed only in positions identified by TR-028 and the rule tables. In particular, the restriction in TR-028b on forwarding an incoming F32 or F64 context, the distinct peer behavior in TR-028c, and the no-context cases in TR-028d are normative.

A successful expression node MUST have exactly one result TypeId. A conversion node MUST have a source child whose result type equals its `from` type, and its result type MUST equal its `to` type. An identity conversion MUST NOT create a Convert node. A rule that requires a conversion MUST NOT leave that conversion implicit (TC-002).

When an expression cannot be checked, the checker MUST NOT manufacture a plausible result type to allow the parent node to appear valid (TC-004). It MUST continue checking independent sibling expressions when doing so does not depend on the failed result (TC-004). Any such additional diagnostics do not turn the failed expression into a valid node.

#### 8.9.3 Statement and declaration checking

The checker MUST process statements in their specified order and establish lexical scope, function context, and loop context in accordance with Sections 8.6 and 9 (TR-020–TR-025, SR-001–SR-008, TC-004). A statement that does not produce an expression value MUST NOT receive an expression TypeId (TC-002, TY-16).

For each declaration, the checker MUST resolve required type syntax, validate declaration constraints, and check initializers or function bodies under the applicable declared types and lexical environment (TR-015–TR-017, SR-002–SR-004). A variable declaration MUST NOT acquire its own binding before its initializer is checked, as required by SR-004. A function MUST be registered at the point required by SR-002 and SR-013 before its body is checked, so direct recursion can resolve without making later functions visible (SR-002, SR-013).

The checker MUST apply the specified return-path analysis to every non-Void function (TR-025). Finding a Return node somewhere in a function body is not sufficient. The condition for acceptance is the `MayCompleteNormally` result defined by TR-025 and Section 8.6.

#### 8.9.4 Diagnostic accumulation and finalization

The checker MUST accumulate diagnostics from independent expressions when their checks do not depend on a failed result. It MUST preserve a deterministic diagnostic order based on phase order and deterministic traversal or rule order. It MUST NOT depend on hash-table iteration order for externally visible diagnostics (TC-004, TC-005).

At finalization, the checker MUST validate the candidate typed tree against the invariants in Section 10 (TC-006, VAL-002). The checker MUST return no Fully Typed AST if any blocking Error diagnostic exists, if a required type or semantic reference is missing, or if typed validation fails. An internal invariant failure MUST be classified as an internal error rather than rewritten as a source type error.

The finalization contract is:

~~~text
finalize(candidate, diagnostics):
    if diagnostics contains a blocking Error:
        discard candidate
        return Failure(diagnostics)

    validation = validate_typed_ast(candidate)
    if validation failed:
        record an internal validation failure
        discard candidate
        return Failure(diagnostics plus internal failure)

    return Success(candidate, non-blocking diagnostics)
~~~

The candidate is private until finalization succeeds. No API caller or later compiler phase may observe a candidate tree on either failure path. Warnings and Notes may accompany Success because they are non-blocking under ERR-011. A failed validator result is not converted into a warning, and a source diagnostic is not converted into an internal error merely because the compiler continues checking independent constructs. This procedure defines the outcome contract; it does not require a particular result type or allocation strategy.

| Rule ID | Requirement | Verification |
| --- | --- | --- |
| TC-001 | The type checker MUST reject an Incomplete or structurally invalid SurfaceProgram before expression checking. | Supply an Incomplete program with a recovery placeholder and a structurally invalid complete-status tree. Verify that neither enters the expression-checking operation and that no TypedProgram is returned. |
| TC-002 | Every successfully checked expression MUST receive one canonical result TypeId, and every non-identity implicit conversion required by its rule MUST be explicit in the typed tree. | Check representative literals, variable references, binary expressions, assignments, call arguments, and return values. Assert result types and conversion source/target types. |
| TC-003 | Context types MUST be propagated only according to TR-028a through TR-028d and the consuming typing rule. | Verify initializer, argument, return, assignment, array-element, compound-assignment, peer-operand, shift, and non-flexible operand cases. Include the operator boundaries in test 29 of Section 13.2. |
| TC-004 | A failed expression check MUST NOT emit a successful typed expression with a fabricated type. The checker MUST continue checking independent expressions when their checks do not depend on the failed result. | Check a program with two independent type errors and assert both diagnostics are reported, while no successful TypedProgram or error-typed expression is exposed. |
| TC-005 | The checker MUST accumulate diagnostics deterministically and MUST NOT use hash iteration order to determine the externally visible order. | Run the same erroneous source repeatedly and with different hash seeds where supported. Compare the ordered diagnostic phase, code, primary span, and message. |
| TC-006 | The checker MUST return a Fully Typed AST only when no blocking Error diagnostic exists and typed validation succeeds. | Exercise separately a source type error, a missing semantic reference in a candidate tree, and an invalid conversion node. Verify that each case returns failure and no Fully Typed AST. |
| TC-007 | The finalization operation MUST keep the candidate typed tree private until all blocking-error checks and typed validation succeed. It MUST return no typed tree when diagnostics contain a blocking Error or validation fails. Non-blocking Warning and Note diagnostics MAY accompany success. | Verify success with no diagnostics and with only Warning/Note diagnostics. Verify failure with a blocking Error, a missing type reference, and a typed invariant failure. Assert that no candidate tree is observable on any failure path. |



## 9. Symbol Resolution and Semantic References

### 9.1 Symbol records

| Symbol kind | Required fields |
| --- | --- |
| Variable | SymbolId; name; declared TypeId; mutability; declaration NodeId; declaration span; ScopeId |
| Parameter | SymbolId; name; TypeId; declaration NodeId; span; function SymbolId; ScopeId |
| Function | SymbolId; name; ordered parameter SymbolIds and TypeIds; result TypeId; declaration NodeId; span; ScopeId |
| Main function | Function symbol with no parameters and Void result |

There is no type-alias symbol in the supported language. A custom type name is an opaque type name, not a reference to a declaration.

### 9.2 Scope rules

| Rule ID | Requirement |
| --- | --- |
| SR-001 | The symbol table MUST have one global scope. The innermost scope is searched first. |
| SR-002 | Function declarations MUST be registered before their body is checked. This permits direct recursion. |
| SR-003 | A function declared later in source is not visible before its declaration. |
| SR-004 | A variable initializer is checked before the new variable is registered. Self-reference in its initializer is undefined. |
| SR-005 | A declaration MUST be unique in its current scope. An inner scope MAY shadow an outer declaration. |
| SR-006 | Function parameters are mutable local variables. |
| SR-007 | A Block creates a nested lexical scope. A For statement creates a scope that covers all clauses and its body. |
| SR-008 | If branches and While bodies are blocks. A For body may be any statement. A body block creates its own scope. |
| SR-009 | Name lookup MUST select the nearest declaration with the exact name, regardless of symbol kind. A variable reference succeeds only if that declaration is a variable. If it is a function, report E2022. Do not search an outer scope for a variable of the same name. |
| SR-010 | A call reference MUST select the nearest declaration with the exact name. It succeeds only if that declaration is a function. If it is a variable, report E2027. Do not search an outer scope for a function of the same name. |
| SR-011 | The Fully Typed AST MUST retain the resolved SymbolId. Later phases MUST NOT repeat name lookup. |
| SR-012 | A named custom type MUST retain its exact source name as an opaque type identity. It does not resolve to a declaration in this language subset. |
| SR-013 | Mutual recursion between distinct functions is not supported. Function declarations are processed in source order: a function MUST be registered before its body is checked, and a later function MUST NOT be visible to an earlier function. A function MAY call itself recursively. A call from one function to a later function MUST fail name resolution with E2023. | Define `first` before `second`, with `first` calling `second`, and verify E2023 and no Fully Typed AST. Verify direct self-recursion passes. Define `second` before `first`, with `first` calling `second`, and verify the call resolves. |

The current checker visits top-level declarations in source order. It does not perform a whole-unit declaration prepass. Therefore, direct recursion is supported because the current function is registered before its body is checked. Mutual recursion between distinct functions is not supported: the earlier function cannot resolve a call to a later declaration. This restriction is explicit in SR-013. Reordering declarations can make a call to an already declared function valid, but does not enable a mutually recursive pair.

The function body uses a function scope. The checker does not add a second block scope for the outer function body. A standalone block creates a block scope. Each if branch creates the scope of its block. A while condition is checked in the surrounding scope. The checker then creates a loop scope for the body; a block body creates a nested block scope. A for statement creates a loop scope before its initializer. This scope covers the initializer, condition, increment, and body. If the body is a block, that block creates a nested scope.

### 9.3 Required symbol-resolution procedure

The rules in this section define the behavior that a resolver must produce. They do not require a specific map implementation or traversal library. The algorithm uses exact identifier spelling, the lexical scope tree, and source order.

#### 9.3.1 Resolution state

At the start of resolution, the compiler MUST create one SymbolTable and one global ScopeRecord for the current CompilationUnitId (SR-001, DEC-012). The resolver MUST maintain the following state (SR-001, SR-011, SR-017, INV-014):

- An active scope, represented by a ScopeId.
- The source-ordered declaration position currently being processed.
- A mapping from each successfully registered declaration to one SymbolId.
- A mapping from each resolved variable or call reference to its SymbolId.
- The active function context and loop context required by the semantic rules.

The resolver MUST NOT reuse a SymbolId for two declarations in one compilation unit (ARC-005, SR-017). A failed declaration or reference MUST NOT produce a successful Fully Typed AST (ERR-004, TC-006). Error recovery may continue to collect independent diagnostics, but a recovery result is not a resolved program.

#### 9.3.2 Scope lookup

The required name lookup operation is:

```text
lookup(name, starting_scope):
    scope = starting_scope
    while scope exists:
        if scope contains a declaration whose identifier equals name:
            return that declaration
        scope = parent(scope)
    return NOT_FOUND
```

Identifier equality uses the exact spelling rule in Section 3. The lookup operation returns the nearest declaration regardless of symbol kind. The caller then validates the required kind.

The following variable-reference and call cases operationalize SR-014 and are subject to the expression and call rules TR-003 and TR-017.

For a variable reference:
1. Call `lookup` from the active scope.
2. If no declaration is found, report the undefined-variable diagnostic defined by TR-003.
3. If the nearest declaration is not a variable or parameter, report E2022. Do not continue searching an outer scope.
4. Otherwise, attach the declaration's SymbolId to the reference and use its canonical TypeId as the expression type.

For a call:
1. Call `lookup` from the active scope.
2. If no declaration is found, report the undefined-function diagnostic defined by the call rule.
3. If the nearest declaration is not a function, report E2027. Do not continue searching an outer scope.
4. Otherwise, attach the function SymbolId, check argument count and each argument against the ordered signature, and use the signature's result TypeId for the call result.

A source-level identifier reference MUST NOT retain only a name when resolution succeeds. Later phases MUST use the resolved SymbolId and MUST NOT repeat lexical lookup (SR-011, SR-017).

#### 9.3.3 Source-ordered declaration processing

The resolver MUST process root declarations in source order (SR-002, SR-003, SR-013). It MUST NOT pre-register all functions in the compilation unit, because SR-003 and SR-013 prohibit references to later functions. For each function declaration, however, it MUST resolve the function signature and register that function in the containing scope before checking the function body. This order permits direct self-recursion without permitting calls to later declarations.

The required conceptual procedure is:

```text
resolve_program(items):
    create global scope
    for item in source order:
        if item is a function:
            reject a duplicate name in the current scope
            resolve its parameter and result types
            register the function symbol
            enter the function scope
            register parameters in declaration order
            check the body using that function scope
            leave the function scope
        else if item is main:
            validate the required main signature
            register the main symbol according to the root declaration rules
            check the body in the main function scope
        else:
            resolve or validate the item under the root grammar
    return diagnostics and semantic records
```

This pseudocode defines observable ordering, not a required code structure. A main declaration MUST be processed under the root-only and signature constraints in BAS-017, SR-006, and INV-019. Whether main is exposed as a callable source-level name must follow the existing call rules and MUST NOT be inferred from the fact that its internal symbol uses a function record (INV-019, TR-017).

Before a variable declaration becomes visible, its initializer MUST be checked in the environment that exists before that declaration (SR-004, SR-015). The declaration itself MUST NOT be used to resolve a self-reference in its initializer. A declaration that conflicts with a name already present in the same scope MUST produce a duplicate-declaration diagnostic and MUST NOT replace the existing symbol (SR-005, SR-015). An inner-scope declaration may shadow a declaration in an outer scope, subject to the nearest-declaration lookup behavior above.

#### 9.3.4 Scope entry and exit

The scope behavior below elaborates SR-007, SR-008, and SR-016. The resolver MUST enter and leave scopes according to those rules:

| Construct | Scope rule |
| --- | --- |
| Function body | Use the function scope containing its parameters. Do not introduce an additional outer-body block scope. |
| Standalone block | Enter a child block scope before processing its statements. Leave that scope when the block ends. |
| If branch | Process the branch block in its block scope. Declarations in one branch are not visible in the other branch or after the conditional. |
| While statement | Resolve the condition in the surrounding scope. Process the body under the loop context and the required loop/body scope arrangement in SR-007 and SR-008. Leave all scopes introduced for the body when it ends. |
| For statement | Enter one for-scope before its initializer. Keep it active for the initializer, condition, increment, and body. If the body is a block, enter its nested block scope. Leave the for-scope after the complete statement. |
| Nested standalone block | Enter a child scope and leave it at the end of the block. |

Scope entry and exit MUST be balanced on both successful processing and recoverable diagnostic paths (SR-016). A declaration introduced in a scope MUST NOT remain visible after that scope is left (SR-016). Loop context is separate from lexical visibility: entering or leaving a lexical scope MUST NOT by itself change the active loop stack (SR-016, INV-014).

#### 9.3.5 Additional requirements and verification

| Rule ID | Requirement | Verification |
| --- | --- | --- |
| SR-014 | Name lookup MUST search the active scope and then each lexical parent, returning the first exact-name match regardless of symbol kind. A kind mismatch at that match MUST be diagnosed without searching outer scopes. | Reference a top-level function name in variable-expression position and verify E2022. Then shadow a top-level function name with a local variable and call that name; verify E2027 and confirm lookup does not fall back to the outer function. |
| SR-015 | A declaration MUST become visible only at the point specified by SR-002 through SR-004 and the source-ordered procedure in Section 9.3.3. A failed duplicate declaration MUST NOT replace an existing symbol. | Verify variable initializer self-reference does not resolve to the variable being declared, an initializer may resolve an outer binding that the new declaration will shadow, and duplicate declarations leave the original SymbolId intact. |
| SR-016 | Scope entry and exit MUST match Section 9.3.4. No declaration may remain visible after its declaring scope ends. | Verify branch-local and block-local names are unavailable after the block, for-initializer names are visible throughout the for statement but unavailable after it, and nested shadowing does not alter the outer declaration after scope exit. |
| SR-017 | Every successfully resolved variable reference and call MUST store the exact SymbolId selected by lookup. The symbol must belong to the same CompilationUnitId as the typed program. | Resolve shadowed references and direct recursive calls, then assert each reference points to the expected declaration. Mutate a resolved reference to a dangling or cross-unit SymbolId and verify typed validation rejects it. |



## 10. Invariants and Validity Criteria

The AST validator must report the first failed invariant category and continue where safe.

| ID | Invariant | Stage | Required action on failure |
| --- | --- | --- | --- |
| INV-001 | Every node ID is unique in its AST instance. | Construction and structural validation | Internal error. Do not run semantic analysis. |
| INV-002 | Owned children form a tree with one parent per non-root node. | Structural validation | Internal error. Do not run later phases. |
| INV-003 | Every required child exists and every child list follows source order. | Structural validation | Internal error for constructed nodes; parser diagnostic for malformed source. |
| INV-004 | Every span refers to one source file, uses valid half-open byte positions whose boundaries are UTF-8 code-point boundaries, and covers the full construct when composite. | Parser output and typed AST validation | Internal error if a constructed node violates it. |
| INV-005 | A SurfaceProgram is Complete only when parsing produced no diagnostics and no recovery placeholder. | After parsing | Mark Incomplete. Stop before type checking. |
| INV-006 | Every identifier reference resolves to one declaration of the required kind. | Name resolution | Source semantic error. Do not construct a Fully Typed AST. |
| INV-007 | Every expression in the Fully Typed AST has a valid TypeId. | Type checking and typed validation | Source semantic error if the source operation is invalid; internal error if the checker omitted a type. |
| INV-008 | Every declaration and function signature has resolved type metadata. | Type checking | Source semantic error for an invalid type; internal error for missing metadata. |
| INV-009 | Every conversion node obeys the conversion table and its child type equals from. | Typed AST validation | Internal error. |
| INV-010 | Every Place refers to mutable storage rooted in a mutable variable, and its place_type equals the target type. | Type checking and typed validation | Source semantic error for immutability or invalid target; internal error for inconsistent metadata. |
| INV-011 | Every array type has a resolved non-negative integer length. | Type resolution | Source semantic error for a non-literal or invalid length. |
| INV-012 | No Error, Unknown, or unresolved type marker exists in a successful Fully Typed AST. | Typed AST validation | Internal error. The constructor must reject the tree. |
| INV-013 | Every SymbolId and TypeId refers to an entry in the same compilation unit. | Typed AST validation | Internal error. |
| INV-014 | Every break and continue points to the nearest active loop in the same function. | Semantic validation and typed validation | Report E2009 or E2010 for invalid source control flow. Treat a mismatched LoopRecord as an internal error. |
| INV-015 | All required type and mutability checks report no errors. | End of semantic analysis | Return diagnostics. Do not return a Fully Typed AST. |
| INV-016 | A CompoundAssign has result_type equal to target.place_type. For operators other than `<<=` and `>>=`, value.result_type equals target.place_type. For `<<=` and `>>=`, value.result_type is an integer type. | Typed AST validation | Internal error for inconsistent metadata. |
| INV-017 | Every call argument count and converted argument type match the referenced function signature. The call result_type equals the signature result_type. | Type checking and typed validation | Source semantic error for an invalid call; internal error for inconsistent typed metadata. |
| INV-018 | Each TypedBinding and TypedParameter type equals the type in its SymbolRecord. | Typed AST validation | Internal error for inconsistent declaration metadata. |
| INV-019 | Every FunctionDecl signature equals its Function SymbolRecord. Main has no parameters and Void result. | Typed AST validation | Internal error for inconsistent signature metadata. |
| INV-020 | TypeTable contains exactly one TypeId for each canonical type key. | Type resolution and typed validation | Internal error for duplicate or missing canonical entries. |
| INV-021 | Surface and typed FunctionDecl and MainDecl nodes occur only as direct children of their program root. No block or other statement contains either declaration. | Parser output and typed validation | Report E1004 and mark SurfaceProgram Incomplete for source nesting. Treat a nested declaration in a typed tree as an internal error. |
| <a id="inv-022"></a>INV-022 | A numeric Literal has a Number variant that corresponds to its result_type and a value that is representable in that type. | Typed AST validation | Internal error. |
| <a id="inv-023"></a>INV-023 | Every SourceSpan start and end offset is a UTF-8 code-point boundary within its registered source, in addition to satisfying the half-open range bounds. | Structural and typed-AST validation; diagnostic rendering | Internal error for a constructed invalid span or reporting failure if the renderer receives one. |

### 10.1 Validity states

| State | Criteria | Accepted by |
| --- | --- | --- |
| Structurally valid Surface AST | Node and span invariants pass. Parse diagnostics are absent. | Name resolution and type checking. |
| Incomplete Surface AST | Parser recovery occurred or parser diagnostics exist. | Diagnostic display and AST debugging only. |
| Semantically valid | All names, types, operations, declarations, and control constraints pass the rules in Sections 7–9. | Fully Typed AST construction. |
| Fully typed | Semantically valid; all expressions and declarations have resolved type data; all references use IDs; all conversions are explicit; all loop-control references are valid in the current function. | Typed AST validation. |
| Ready for lowering | Fully typed and INV-001 through [INV-023](#inv-023) pass. No blocking design question remains for the program's constructs. | A later lowering phase that implements this contract. |
| Invalid AST | Any structural or typed invariant fails. | No compiler phase after the failing validator. |

An AST with an error is not semantically valid. Diagnostics do not convert an invalid source program into a valid AST.

### 10.2 AST validation procedure

Structural validation and typed-AST validation have separate responsibilities. Structural validation determines whether the node graph is a well-formed instance of its declared AST representation. Typed-AST validation additionally determines whether the semantic metadata and cross-references satisfy the invariants of a successful Fully Typed AST.

#### 10.2.1 Structural validation

The validator MUST visit every owned node and every owned child edge (VAL-001). It MUST check the following conditions before accepting a Surface AST or Typed AST:

1. The root node has the expected root kind and belongs to one AstId.
2. Every NodeId is unique within that AstId.
3. Every non-root node has exactly one owning parent. Owned child edges form an acyclic tree.
4. Every required child exists and has a node kind allowed by its parent field.
5. Ordered child lists preserve the order required by the node definition and source syntax.
6. Every SourceSpan refers to a registered SourceId. Its half-open byte range satisfies `start_byte <= end_byte <= source_length`.
7. A composite node span covers its complete source construct, as required by INV-004.
8. A parser-recovered program or a program with parser diagnostics is Incomplete and cannot be accepted as complete semantic input.

A conceptual traversal that satisfies VAL-001 is:

```text
validate_structure(root):
    if root is absent:
        return invalid-root result
    expected_ast_id = root.ast_id
    pending = [(root, root_parent, root_field)]
    seen_node_ids = empty set
    valid = true

    while pending is not empty:
        node, parent, parent_field = remove next pending item
        if node.ast_id != expected_ast_id:
            record cross-AST ownership failure
            valid = false
            continue
        if node.node_id is in seen_node_ids:
            record duplicate NodeId or repeated-owned-node failure
            valid = false
            continue
        add node.node_id to seen_node_ids
        validate node kind, field types, and required fields
        validate node span against SourceMap
        for each owned child field in declared field order:
            if a required child is absent:
                record missing-child failure
                valid = false
            for each present child in that field:
                add (child, node, field) to pending in reverse order

    return valid only if no structural failure was recorded
```

This is an illustrative iterative walk, not a mandated implementation. The
validity conditions and failure behavior remain those in INV-001 through
INV-005 and VAL-001. An implementation MAY use a different traversal
strategy, but it must detect repeated owned nodes and cycles without
unbounded traversal.

The validator MUST distinguish a malformed source result from a compiler-created invalid node (INV-005, ERR-001–ERR-003, VAL-001, VAL-004). Parser-produced invalidity must be represented by parser diagnostics and an Incomplete state. A constructed node that violates the node model or identity invariants is an internal error.

#### 10.2.2 Typed-AST validation

After structural validation succeeds, the typed validator MUST check all applicable semantic invariants in Section 10 (VAL-002). At minimum, it MUST establish:

- Every expression has one TypeId that resolves to the canonical TypeTable entry in the same CompilationUnitId.
- Every declaration and function signature agrees with its SymbolRecord.
- Every successfully resolved variable and call reference points to the correct SymbolRecord and has the type required by that record.
- Every Convert node uses an allowed ConversionKind, its child type equals `from`, and its result type equals `to`.
- Every compound assignment has a valid mutable Place, uses the operation type required by TR-013, and has result_type equal to the place type.
- Every array node has a resolved length and every function call agrees with the resolved signature's arity and parameter types.
- Every break and continue reference identifies the nearest valid loop in the same function.
- No error marker, placeholder, unresolved type, dangling ID, unsupported type category, or diagnostic sentinel appears in a successful tree.

The validator MUST apply checks in an order that prevents unsafe reference access:

~~~text
validate_typed_ast(program):
    structural_report = validate_structure(program)
    if structural_report has failures:
        return Invalid(structural_report)

    validate SymbolTable and TypeTable record shapes, compilation-unit IDs,
        ID uniqueness, and canonical type-key uniqueness
    record table failures before using affected IDs
    prepare lookups only from entries whose IDs are unique and in-unit

    for node in deterministic preorder, using declared child-field order:
        for each semantic reference field in declared field order:
            if its ID has no unique, valid in-unit entry:
                record the applicable invariant failure
                mark dependent checks as unavailable
            else:
                validate the reference's expected kind and relation
        validate node-specific semantic invariants only when their required
            references have been resolved safely

    if any structural, table, or semantic failure exists:
        return Invalid(failures in deterministic encounter order)
    return Valid
~~~

An ambiguous duplicate ID MUST NOT be resolved by selecting the first or last entry. A dangling, duplicate, or foreign-unit reference MUST NOT be dereferenced. The validator MAY continue with independent records and nodes after recording such a failure, but it MUST skip checks that require the invalid reference. This procedure defines required behavior, not a required map, indexing structure, or error-reporting data type.

Validation MUST use the same canonical type and assignability definitions as semantic analysis (TY-12, TY-13, TY-17, VAL-002). It MUST NOT silently repair a malformed typed tree by inventing a type, rebinding a reference by name, or inserting an unrequested conversion (VAL-003).

#### 10.2.3 Failure behavior and validation result

A validator MUST return a successful validity result only if all required checks pass (VAL-002, VAL-004). It MAY accumulate multiple independent validation failures where this is safe, but it MUST stop following a branch when doing so would require dereferencing an invalid or dangling reference (VAL-002). Any typed-tree invariant failure is an internal error, not a source-language type diagnostic (INV-001–INV-023, VAL-002, ERR-006). An invalid tree MUST NOT be passed to a lowering or other downstream phase (VAL-004).

| Rule ID | Requirement | Verification |
| --- | --- | --- |
| VAL-001 | Structural validation MUST check root kind, node identity, owned-child topology, required child kinds, source order, and source-span validity before accepting an AST. | Mutate a valid tree separately to introduce a duplicate NodeId, repeated child, missing child, invalid child kind, reversed child order, out-of-range span, and a span boundary inside a multi-byte UTF-8 character. Verify each mutation is rejected with the corresponding invariant or internal-error result. |
| VAL-002 | Typed-AST validation MUST verify semantic IDs, canonical types, declaration/signature consistency, conversion metadata, mutable places, array lengths, call signatures, and loop references against the invariants in Section 10. It MUST classify typed-tree invariant failures as internal errors and MUST NOT dereference invalid or dangling references while continuing validation. | Create one invalid typed tree per applicable invariant, including multiple invalid references, and assert rejection as an internal error without a crash or downstream use. |
| VAL-003 | A validator MUST NOT repair an invalid AST by resolving names again, inventing missing type information, inserting conversions, or changing source spans. | Remove a type ID, replace a valid symbol ID with a dangling ID, and alter a conversion target. Verify failure without tree mutation or substituted metadata. |
| VAL-004 | An Incomplete SurfaceProgram and any typed tree that fails validation MUST be rejected as downstream phase input. | Attempt semantic processing on an Incomplete SurfaceProgram and lowering on an invalid TypedProgram. Verify the next phase is not entered. |
| VAL-005 | Each validation failure MUST identify the violated invariant or validation rule and classify the failure as structural or typed-semantic. It MUST identify the affected field and node when they can be established safely. The report MUST NOT dereference a dangling ID or use an ambiguous duplicate NodeId as a unique reference. Multiple independent failures MAY be reported in deterministic traversal order. | Mutate separate nodes to violate different invariants and verify each report identifies the rule and safe location. Include duplicate IDs and dangling references, and verify report generation does not crash or dereference invalid data. |
| VAL-006 | Typed-AST validation MUST establish uniqueness and compilation-unit ownership of semantic table entries before it dereferences their IDs. It MUST NOT resolve an ambiguous duplicate ID by entry order. Checks that depend on a dangling, duplicate, or foreign-unit ID MUST be skipped, while independent checks MAY continue in deterministic traversal order. | Introduce duplicate SymbolIds, duplicate TypeIds, duplicate canonical type keys, a foreign CompilationUnitId, and dangling references. Verify safe rejection, no arbitrary entry selection, no invalid dereference, and deterministic independent findings. |



## 11. Error Handling

### 11.1 Error categories

| Category | Cause | Required behavior |
| --- | --- | --- |
| Lexical error | Invalid character or malformed token. | Report source span and lexer code. Do not build a Fully Typed AST. |
| Syntax error | Token sequence does not match the grammar. | Report source span and parser code. Parser may recover for diagnostics. Mark the result Incomplete. |
| Name-resolution error | Undefined name, wrong symbol kind, or duplicate declaration. | Report the reference or declaration span and related declaration span when available. |
| Type error | Invalid operand, assignment, argument, return, or index type. | Report the affected expression or statement span and expected and actual types. |
| Control error | Invalid break, continue, or return context. | Report the control statement span and the required context. |
| Internal error | Compiler invariant failure, corrupt ID, or impossible checker state. | Stop compilation. Do not report it as a user type error. |
| Input/output error | Source file cannot be read or written. | Report the file operation and operating system error. |

### 11.2 Diagnostic record

The normative diagnostic field set is defined by ERR-008. Every diagnostic MUST contain:

- DiagnosticId.
- Phase.
- Error code when the code system defines one.
- Severity.
- Message.
- Primary SourceSpan when source text caused the error.
- Optional related spans.
- Optional help text.
- Optional NodeId when the affected node exists.

| ID | Requirement | Verification |
| --- | --- | --- |
| ERR-008 | Every diagnostic MUST contain the fields listed above. A source-caused diagnostic MUST include its primary SourceSpan. A diagnostic MUST link to its affected NodeId when that node exists. | Generate one diagnostic from each error category and validate all required fields and the source-node link. |
| ERR-011 | DiagnosticId MUST be unique in one compiler run. Phase and Severity MUST use the enumerations in Section 3. Every Error is blocking. A Warning or Note does not block Fully Typed AST output. | Generate diagnostics in each phase and verify ID uniqueness, enum values, and blocking behavior by severity. |
| ERR-012 | A diagnostic's primary span MUST identify the source construct responsible for the immediate error. An unresolved identifier or wrong-kind reference MUST point to the identifier token. A duplicate declaration MUST point to the later declaration and include the earlier declaration as a related span when available. An invalid operator or operand combination MUST point to the complete expression span. | Trigger each listed error and assert the primary span boundaries and, for duplicates, the related declaration span. |
| ERR-013 | A diagnostic span MUST refer to an existing SourceId in the SourceMap and use valid half-open byte offsets whose start and end are UTF-8 code-point boundaries within that source. A renderer encountering an invalid span record MUST report an internal reporting failure and MUST NOT misclassify the source program as invalid. | Render diagnostics at byte offset zero, at end of file where an empty span is permitted, and on multi-byte UTF-8 text. Inject an out-of-range span, an offset inside a multi-byte scalar, and an unknown SourceId. Verify internal reporting failure. |
| ERR-014 | Diagnostic order MUST follow pipeline phase order and deterministic phase encounter order as defined in Section 11.2. The order MUST NOT depend on hash iteration, diagnostic code values, or DiagnosticId values. | Compile the same input repeatedly and with different hash seeds where supported. Compare the complete ordered diagnostic list, including phase, code, primary span, related spans, and message. Include multiple diagnostics with the same primary span. |

The existing CompileError model provides a phase-specific variant, optional code, message, span, and optional help. Related spans and NodeId are specified additions. The span-selection rules above define the diagnostic contract; the visual formatting of rendered messages remains implementation-dependent.

Diagnostic ordering is observable compiler output and MUST be deterministic. The compiler MUST preserve the phase execution order in Section 5.1. Within one phase, the encounter order MUST follow the phase's deterministic traversal: source/token order for lexical and parser diagnostics, source-ordered declarations and statements for name resolution and type checking, and declared AST field order for validation. When one node triggers more than one diagnostic, the implementation MUST emit them in the order in which the applicable rule checks are performed. Diagnostic codes and DiagnosticId numeric values MUST NOT be used as ordering keys. DiagnosticId MUST remain unique within the compiler run, but the ID value does not convey diagnostic severity or precedence.

The implementation MUST NOT use hash-table iteration order to select which declaration, node, or diagnostic is visited first. If an implementation uses a collection whose iteration order is unspecified, it MUST impose a deterministic order at the point where that collection affects diagnostics or semantic output.

### 11.3 Recovery and stopping rules

| ID | Requirement |
| --- | --- |
| ERR-001 | Parser recovery MAY continue to find later syntax diagnostics. Its result MUST be Incomplete if any parser error occurred. |
| ERR-002 | A null-pointer expression placeholder or a built-in Void type fallback created during parser recovery MUST NOT be treated as source syntax. The parse diagnostics make the whole result Incomplete. |
| ERR-003 | Name resolution and type checking MUST NOT accept an Incomplete Surface AST. |
| ERR-004 | Type checking MAY collect multiple independent source diagnostics. It MUST return no Fully Typed AST when any blocking diagnostic exists. |
| ERR-005 | A successful Fully Typed AST MUST contain no error node and no diagnostic sentinel. |
| ERR-006 | Internal errors MUST stop the current compilation. A source error MUST NOT cause an internal-error classification. |
| ERR-007 | No code-generation or lowering phase may accept an Incomplete or invalid AST. |
| ERR-010 | On a statement parse failure at the root or in a block, the parser MUST advance one token and continue. It MAY return successfully parsed statements with diagnostics. Expression recovery placeholders MUST be nullptr literals. An invalid declaration-type fallback MUST be represented as an absent annotation. Both placeholders MUST remain unusable because the parse result is Incomplete. The CLI MUST stop before type checking when parse diagnostics exist. |

| ID | Requirement | Verification |
|---|---|---|
| ERR-009 | Codes E0001–E0999 identify lexical errors. Codes E1000–E1999 identify syntax errors. Codes E2000–E2999 are normally semantic or type errors. E2001 is an exception: the parser uses it for a variable initializer-count mismatch. The diagnostic phase and error variant define the category. Existing codes E2002–E2032 SHOULD be retained for their current conditions. E2033 identifies a non-place or otherwise invalid storage target. E2034 identifies an invalid fixed-array length. E2035 identifies an unsuffixed numeric literal whose value does not fit its context type. E2036 identifies numeric operands that have no common type under TR-009, TR-010, or TR-012. | Verify code, phase, error variant, and message for each listed diagnostic condition, including parser E2001. |

## 12. Operations and Interfaces

### 12.1 Mandatory operations

| Operation ID | Inputs and output | Preconditions and postconditions | Errors and effects |
| --- | --- | --- | --- |
| OP-001 Construct Surface AST | Tokens → SurfaceProgram and diagnostics | Tokens belong to one SourceId. Output preserves source order and spans. | Lexer or parser diagnostics. No semantic side effect. |
| OP-002 Traverse | AST and visitor → visitor result | AST passes structural validation. Traversal is deterministic depth-first, in field order. | Visitor error is returned to the caller. |
| OP-003 Structural validation | SurfaceProgram or TypedProgram → validation result | AST root is present. | Reports invariant violations. Typed-tree violations are internal errors. |
| OP-004 Resolve symbols | Complete SurfaceProgram → resolved symbol context and diagnostics | Surface tree is structurally valid and has no parser diagnostics. | Source diagnostics. No successful output when any blocking error exists. |
| OP-005 Type check and build | Complete SurfaceProgram plus symbol context → Result<TypedProgram, Diagnostics> | All source names and type syntax are available. | Collects source semantic errors. Returns no TypedProgram on error. |
| OP-006 Validate typed AST | TypedProgram → valid marker or internal error | TypedProgram was constructed by the semantic pass. | Rejects missing types, bad references, bad conversions, and structural errors. |
| OP-007 Insert conversions | Typed expression and expected type → converted TypedExpr | The assignment relation permits the conversion. Numeric promotion uses the same relation. | Type diagnostic when no conversion exists. Rewrites only the typed tree. |
| OP-008 Generate diagnostics | Diagnostic records and source map → rendered output | Span source exists when source span is present. | Rendering error is an internal reporting failure. |
| OP-009 Transform or clone AST | Valid AST plus transformation or clone request → valid AST | Validate output after transformation. A clone gets new AstId and NodeIds. Retained transformed nodes keep IDs. A transformation that changes source meaning, names, or types MUST rerun the required semantic passes before it returns a Fully Typed AST. | Invalid output is an internal error. Do not mutate the input AST. |

### 12.2 Optional operations

| Operation ID | Status | Inputs and output | Preconditions, postconditions, effects, and errors |
| --- | --- | --- | --- |
| Serialization and reconstruction | Out of scope | None. | No serialized format or compatibility promise is required. |
| OP-010 Structural comparison | Optional internal utility | Two ASTs of the same representation → bool. | Both inputs must be structurally valid. Compare node kinds, operators, literal payloads, fields, and ordered owned children. Ignore spans and all IDs. No mutation. Invalid input is an internal error. |
| OP-011 Semantic comparison | Optional internal utility | Two TypedPrograms with the same CompilationUnitId → bool. | Both inputs must pass typed validation. Compare typed node shape, TypeIds, SymbolIds, conversion kinds, and resolved metadata. Ignore memory addresses and NodeIds. No mutation. Different compilation units are invalid input. |
| Persistent shared-node editing | Out of scope | None. | AST child nodes have single ownership. |
| Incremental semantic analysis | Out of scope | None. | IDs are stable only within one AST instance. |

### 12.3 Cloning and transformation

A transformation MUST preserve source spans for retained constructs. A transformation that inserts conversion nodes MUST use the original operand span for the conversion node and retain the original child span. A transformation MUST NOT duplicate an effectful child expression. OP-009 defines the input, output, preconditions, postconditions, effects, and error behavior.

A clone of a Surface AST or Typed AST is a new AST instance. It receives a new AstId and new NodeIds. Symbol and type tables may be shared only as immutable data within the same compilation unit.

### 12.4 Algorithmic complexity and resource expectations

This subsection defines asymptotic expectations for the required operations. These expectations describe algorithmic growth. They are not wall-clock performance guarantees. No maximum source size, latency target, or memory budget is confirmed by BAS-016.

Let:
- V be the number of AST nodes visited by an operation.
- E be the number of owned child edges in the AST.
- D be the maximum lexical-scope nesting depth.
- S be the number of symbol records in the compilation unit.
- T be the number of canonical types in the type table.
- L be the number of source bytes.

| ID | Operation | Complexity requirement | Assumptions and verification |
| --- | --- | --- | --- |
| CMP-001 | Full AST traversal and structural validation | Time O(V + E), auxiliary space O(H) for an iterative traversal, where H is the maximum pending traversal frontier. A recursive implementation may use O(D_ast) call-stack space, where D_ast is maximum AST depth. | Each node and child edge is examined a bounded number of times. Verify with instrumentation on generated trees of increasing size. Do not treat recursion depth as unbounded. |
| CMP-002 | Source-span validation | O(1) per span, excluding source-map lookup. Whole-tree validation is O(V + E). | Span checks compare byte offsets and source identity. Validate boundary spans at byte offset zero and at the source length. |
| CMP-003 | Lexical name lookup | The result must be equivalent to searching the current scope and then its lexical parents. A hash-based scope table commonly gives expected O(1) lookup per scope, and O(D) across D scopes; an ordered map commonly gives O(log S_scope) per scope. The chosen data structure remains implementation-dependent. | Correct shadowing takes precedence over a specific map implementation. Measure the selected implementation before making performance claims. |
| CMP-004 | Type canonicalization and equivalence lookup | The TypeTable must return one canonical TypeId for each type key. Lookup complexity depends on the selected map and key representation and is not fixed by this specification. | Verify uniqueness and repeat lookup identity for primitive, named, array, and vector keys. Do not claim constant-time lookup without an implementation and data-structure basis. |
| CMP-005 | Type checking and typed-AST construction | For a tree with V nodes, the pass must not repeatedly traverse an unchanged subtree solely to recompute information already available from that subtree. A single-pass implementation with constant work per node, excluding symbol/type lookups, has O(V + E) traversal work. | The exact bound includes the costs of symbol and type lookup, diagnostics, and any required contextual rechecking. Profile representative programs before setting a numeric performance threshold. |
| CMP-006 | Diagnostic accumulation | Appending one diagnostic must not require copying all previously accumulated diagnostics. Total accumulation should be O(K), excluding message construction and sorting, for K diagnostics. | Verify that a sequence of K diagnostics does not trigger a quadratic append strategy. Diagnostic ordering must follow the defined phase and traversal behavior, not hash iteration order. |
| CMP-007 | Return-path validation | The conservative analysis in TR-025 must inspect each statement in the function body a bounded number of times, giving O(V_f + E_f) time for a function subtree of V_f nodes and E_f child edges. | Loops are treated as potentially completing normally. The analysis must not recursively reanalyze a loop body without a defined bound. Verify nested blocks, conditionals, and loops. |
| CMP-008 | Resource exhaustion | Allocation failure, excessive nesting, and source-size limits must not be reported as ordinary source type errors. The implementation must use the repository's established failure mechanism or document a new one before claiming recovery behavior. | No specific maximum size or recovery guarantee is established. Tests must distinguish a controlled failure from a panic, process abort, or invalid Fully Typed AST. |

The complexity rows are normative design constraints on avoidable algorithmic work. They do not select hash maps, ordered maps, recursion, or an allocation strategy. A performance acceptance threshold requires a separately approved workload, execution environment, and measurement method.

## 13. Verification and Acceptance Criteria

### 13.1 Verification methods

The implementation MUST provide tests or validator checks for each row below. A test passes only when the observed AST shape, type, reference, or diagnostic equals the stated expectation.

| Requirement IDs | Verification |
| --- | --- |
| BAS-006–BAS-017 | Architecture review plus compile-mode, internal-model, supported-type, recovery, error-state, root-only function, and later workload-performance checks. |
| GAP-001–GAP-010, GAP-013–GAP-015 | Source inspection confirms each observed gap; implementation review confirms the stated required consequence. |
| GAP-011–GAP-012 | Source inspection confirms loop-state behavior and nested declaration acceptance; parser tests verify root-only function and main declarations. |
| ARC-001–ARC-011 | Node ID uniqueness, tree shape, clone identity, semantic reference, pipeline, abstract-node, span, and resolved-type representation tests. |
| SYN-001 | Parser-conformance tests for each grammar production and optional clause. |
| SYN-003 | Parser tests verify root-only `fun` and `main` declarations, E1004 diagnostics, incomplete status, and the semantic phase boundary. |
| IF-001–IF-010 | Interface input-state, output-invariant, diagnostics, validation, and blocking-failure tests at each compiler phase boundary. |
| N-ROOT-01–N-ROOT-02 | Root construction tests for empty and non-empty source units. |
| N-SE-01–N-SE-09 | One constructor and one structural-validation test per expression variant. |
| N-SS-01–N-SS-11 | One constructor and one structural-validation test per statement variant. |
| N-SD-01–N-SD-02 | Constructor, ownership, source-span, and ordering tests for bindings and parameters. |
| N-ST-01–N-ST-04 | Type-syntax tests for each primitive, named type, nested array, and vector. |
| N-TE-01–N-TE-11 | Typed-node tests. Verify result_type and all linked IDs. |
| N-TD-01–N-TD-02 | Typed declaration-node tests. Verify declared types, symbol links, source spans, and owner links. |
| N-TP-01–N-TP-03 | Place tests for mutable variables, grouped places, mutable arrays, immutable targets, and invalid indexes. |
| TY-01–TY-19 | Type identity, canonicalization, assignability, promotion, conversion, and invalid-category tests. |
| TR-001–TR-028 | Positive and negative tests for every typing rule, including primitive, named, vector, nested-array, and invalid array type syntax. |
| SR-001–SR-012 | Scope, shadowing, duplicate, recursion, forward-reference, custom type, and symbol-kind tests. |
| SR-013 | Source-order visibility, direct recursion, and rejection of calls to later declarations and mutual recursion. |
| SR-014–SR-017 | Nearest-scope lookup, declaration visibility, scope exit, and resolved SymbolId identity tests. |
| TC-001–TC-007 | Negative and positive tests for pass input state, result types, contextual typing, diagnostic accumulation, deterministic ordering, typed-tree finalization, and successful-output gating. |
| VAL-001–VAL-006 | Structural topology, source-span, semantic cross-reference, safe table indexing, deterministic failure reporting, no-repair, and downstream-gating tests for AST validators. |
| SYN-002 | AST-shape tests for left associativity, assignment right associativity, and precedence for every operator class. |
| INV-001–[INV-023](#inv-023) | One validator test per invariant. Each negative case must identify the violated invariant. |
| ERR-001–ERR-014 | Parser recovery, blocking errors, accumulated diagnostics, diagnostic fields, deterministic ordering and locations, error codes, severity, CLI phase boundary, and internal-error separation tests. |
| OP-001–OP-009 | Interface tests for each mandatory operation, transformation, and cloning. |
| OP-010–OP-011 | Comparison tests only if an optional comparison utility is implemented. |

### 13.2 Required boundary tests

The test suite MUST include:

1. Every numeric type in literal typing, with and without a width suffix, and with and without a context type.
2. Every permitted numeric assignment conversion, including the six unsigned-to-signed rows, and the adjacent disallowed conversion in both directions. Include the four removed rows: I32 and U32 to F32, and I64 and U64 to F64. Each reports E2002 as an initializer.
3. All 100 ordered numeric type pairs. Verify the promoted type for 68 pairs and E2036 for the 32 ordered pairs listed in Section 7.3, including I32 with F32, I64 with F64, U64 with I64, and I32 with U32.
4. Empty and one-element array literals.
5. Equal and unequal array lengths.
6. Zero and maximum representable array lengths.
7. Non-integer and negative array lengths.
8. Vector element type assignment.
9. NullPtr assignment to each permitted target category and one disallowed primitive target.
10. Zero-argument, correct-arity, too-few, and too-many function calls.
11. Return with and without a value in Void and non-Void functions.
12. Break and continue inside and outside nested loops.
13. Nested function and main declarations in blocks, branches, and loop bodies. Verify E1004 and Incomplete status.
14. Prefix and postfix update of mutable, immutable, and non-place expressions.
15. Assignment through a mutable variable, through a mutable index chain, through an immutable variable, and through an index into a temporary expression.
16. Assignment and compound assignment where the value can and cannot convert to the place type, including `i += 1` for I32, `x *= 2.0` for F32, `x += 2i16` for I8, and `x <<= k` for I8 with a U64 count.
17. A recovered parse result containing a placeholder. Verify it cannot enter type checking.
18. A variable, function, type, or scope ID that is dangling or belongs to another AST. Verify typed validation rejects it.
19. A comparison between two Void-returning calls. Verify the current type rule accepts it and assigns Bool.
20. An array literal whose elements are Void-returning calls. Verify the current checker assigns Array(Void,n).
21. Primitive, named, vector, and nested-array type syntax. Verify canonical type identity and reject malformed type syntax. Resolve repeated canonical keys in different source positions, verify equal keys share one TypeId, verify non-equivalent keys do not share an ID, and verify an invalid child type or length does not create a parent TypeEntry.
22. Operator associativity. Parse each source below and verify the exact node shape: 10 - 3 - 2 as (10 - 3) - 2; 2 - 3 + 4 as (2 - 3) + 4; 100 / 10 / 5 as (100 / 10) / 5; 1 << 2 << 3 as (1 << 2) << 3; a - b *c - d as (a - (b* c)) - d; a = b = c as a = (b = c); -a *b as (-a)* b.
23. The minimum, the maximum, and each value one outside the range, for all eight integer types, as initializers and as peer operands. Verify E2035 outside the range.
24. F32 overflow, and integer exactness at 2^24 for F32 and 2^53 for F64 with the neighboring values. Verify E2035 for an inexact integer literal.
25. Negation folding: `-128` for I8, `-9223372036854775808` for I64, `-(128)` for I8, and a suffixed literal. Verify that a folded literal is one Literal node.
26. The peer rule with the literal on either side of a non-literal operand, with a flexible operator tree on one side, and with a float literal next to an integer operand.
27. Context passing through array literals, nested array literals, call arguments, return values, and shifts with a non-literal count. Verify that an F32 or F64 context does not pass through an operator.
28. Integer-to-float conversion of a non-literal value. Verify that an I32 or U32 variable assigned to F32, and an I64 or U64 variable assigned to F64, reports E2002, including a variable that holds 16777217. Verify that `x + y` with `x: i32` and `y: f32` reports E2036, and that `x + y` with `x: i32` and `y: f64` has type F64 with a NumericAssignable conversion on `x`. Verify that `var f: f64 = 7 / 2` reports E2002, because the expression has type I64.
29. Operator context boundaries. Verify that `var i: i32 = 1; var mixed: f64 = i + 1.5` has type F64 and a NumericAssignable conversion from I32 to F64, because `1.5` remains F64 after receiving the peer context I32 and I32 is assignable to F64. Also verify that `var wide: i64 = 1; var rejected: f64 = wide + 1.5` reports E2036 because I64 is not assignable to F64. Verify that `var shifted: f64 = 1 << 3` reports E2002 because the shift result is I64. Verify the enclosing rule reports E2002 for `var value: i32 = 1.5`, even though TR-028a reports no leaf-adaptation error.
30. Non-void return-path analysis. Verify that `fun f(): i32 { while (false) { return 1 } }` and the equivalent `for` loop report E2003. Verify that a return after a possibly completing loop passes, and that an `if/else` with a return in both branches passes. Verify that an `if` without `else` does not satisfy the check.

31. Nearest-scope name lookup. Declare a top-level function and then shadow its name with a local variable in a function body. Verify a call through that name reports E2027 for a non-function nearest declaration and does not resolve to the outer function. Verify a variable reference whose nearest declaration is a function reports E2022.
32. Initializer visibility and same-scope duplicates. In a nested block, declare an outer variable and declare an inner variable with the same name whose initializer references that name. Verify the initializer resolves to the outer SymbolId and subsequent references in the inner scope resolve to the inner SymbolId. Declare the same name twice in one scope and verify E2032; the rejected duplicate must not replace the first SymbolId.
33. Scope exit. Declare a variable in a nested block and reference it after the block. Verify E2023. Declare a for-loop initializer variable, reference it in the condition, increment, and body, then reference it after the loop. Verify all in-loop references resolve and the post-loop reference reports E2023.
34. Function source order. Verify a function can call itself recursively because its symbol is registered before its body is checked. Verify a call to a function declared later reports E2023. Verify declaration order that places the callee first allows the call, and does not create support for mutual recursion.
35. Diagnostic source locations. Trigger an unresolved identifier, a duplicate declaration, and a numeric expression whose operands have no common promotion type. Verify the unresolved identifier diagnostic points to the identifier token, the duplicate diagnostic points to the later declaration and links the original declaration span, and the invalid binary diagnostic points to the complete expression span. Test source spans at byte offset zero, a permitted empty end-of-file span, and valid boundaries around a multi-byte UTF-8 source sequence. Verify that a boundary inside a multi-byte scalar is rejected.
36. AST validation failures. Starting from a valid typed tree, separately introduce a duplicate NodeId, a missing required child, a dangling SymbolId, an invalid conversion target, an immutable SymbolRecord used as an assignment target, and a break/continue link to a loop in another function. Verify the validator rejects each tree, identifies the violated invariant or validation rule, reports a safe node or field location when available, does not dereference invalid IDs or repair the input, and prevents downstream lowering.
37. Type-checking failure gating. Supply an Incomplete SurfaceProgram, a source program with two independent type errors, and a candidate typed tree with a missing TypeId. Verify the first is rejected before semantic analysis, the second produces both independent diagnostics and no Fully Typed AST, and the third fails typed validation without synthesizing a replacement type.
38. Diagnostic ordering. Run the same source with multiple errors repeatedly and under different hash seeds where supported. Verify phase order, diagnostic code, primary span, and message order remain stable and do not depend on hash-map traversal.
39. Semantic table validation. Mutate a valid TypedProgram to contain duplicate SymbolIds, duplicate TypeIds, duplicate canonical type keys, a table whose CompilationUnitId differs from the root, and a reference to a missing entry. Verify duplicate or foreign entries are rejected before use, dependent checks are skipped rather than dereferencing them, independent findings remain deterministic, and no invalid TypedProgram is returned.

### 13.3 Acceptance criteria

The specification is implemented when all the following conditions hold:

| Acceptance ID | Objective condition |
| --- | --- |
| ACC-001 | Every concrete node in Section 6 has a construction and validation test. |
| ACC-002 | A valid sample program produces a Fully Typed AST with a TypeId on every expression. |
| ACC-003 | Every variable and call reference resolves to exactly one SymbolId. |
| ACC-004 | Every implicit conversion is represented by a Convert node with the same source and target TypeIds. |
| ACC-005 | Invalid source programs produce diagnostics and no Fully Typed AST. |
| ACC-006 | Incomplete parser output is rejected by type checking. |
| ACC-007 | The typed validator rejects each single-invariant mutation in INV-001 through [INV-023](#inv-023). |
| ACC-008 | The complete test suite covers every operator, type category, semantic reference, and error category in this document. |
| ACC-009 | No unsupported type category or language feature can be inserted into a successful Fully Typed AST. |
| ACC-010 | The final document review confirms that each requirement has an ID and a verification method. |
| ACC-011 | Every phase boundary in IF-001 through IF-010 rejects inputs that fail its required preconditions, and no blocking result reaches a downstream phase as valid input. |
| ACC-012 | The type-checking pass satisfies TC-001 through TC-006, including contextual typing under TR-028, explicit required conversions, deterministic diagnostics, and the absence of a successful Fully Typed AST on failure. |
| ACC-013 | Symbol resolution satisfies SR-001 through SR-017, including exact nearest-scope lookup, declaration visibility, scope exit, direct recursion, and rejection of later-function references. |
| ACC-014 | Diagnostic locations satisfy ERR-012 and ERR-013 for unresolved identifiers, duplicate declarations, numeric promotion failures, valid source-map references, and invalid-span handling. |
| ACC-015 | The validators satisfy VAL-001 through VAL-004 by rejecting malformed trees and invalid semantic metadata without silently repairing the input or allowing downstream processing. |
| ACC-016 | The implementation review or an explicit complexity argument addresses CMP-001 through CMP-008. Runtime performance thresholds are not claimed until a representative workload and measurement environment are approved. |
| ACC-017 | The implementation exhausts all 100 ordered numeric type pairs. Exactly 68 pairs produce the specified promoted type, and the other 32 report E2036 without producing a typed binary expression. Every selected conversion satisfies the value-preservation predicate in Section 7.2. Verification: Section 13.2 test 3 and the procedure in Section 7.3. |
| ACC-018 | Context-sensitive literal typing satisfies TR-028a through TR-028d, including integer range boundaries, exact floating-point representability, peer operands, compound assignment, and the rule that F32/F64 context does not pass through an operator to two flexible operands. Verification: Section 13.2 tests 1, 23 to 29, with expected tree shapes and diagnostics. |
| ACC-019 | Diagnostics are reproducible in both contents and order for repeated compilations of the same input. The sequence follows phase and encounter order, including ties on the same primary span. Verification: Section 13.2 test 38, TC-005, and ERR-014. |
| ACC-020 | Every AST validation failure identifies the violated invariant or rule and supplies a safe node or field location when available. Reporting a validation failure never dereferences a dangling ID, repairs the AST, or permits downstream use. Verification: VAL-001 through VAL-005 and Section 13.2 test 36. |
| ACC-021 | A candidate TypedProgram becomes observable only after error gating and typed validation succeed. Blocking diagnostics and validation failures return failure without a TypedProgram; non-blocking diagnostics may accompany success. Verification: TC-006, TC-007, and Section 13.2 test 37. |
| ACC-022 | Each compilation unit has exactly one TypeId for every canonical type key, and failed type resolution does not leave an invalid composite type entry. Verification: TY-17, TY-19, and Section 13.2 test 21. |
| ACC-023 | Typed validation rejects duplicate, dangling, and foreign-unit semantic IDs without dereferencing ambiguous or invalid entries. It preserves deterministic reports for independent failures and never returns the rejected tree. Verification: VAL-005, VAL-006, and Section 13.2 tests 36 and 39. |
| ACC-024 | Every accepted SourceSpan uses in-range UTF-8 code-point boundaries. A span that splits a multi-byte scalar is rejected by structural validation and diagnostic rendering. Verification: INV-004, INV-023, VAL-001, ERR-013, and Section 13.2 test 35. |

## 14. Technical Examples

### 14.1 Valid typed conversion

Source:

~~~text
fun widen(value: i16): i32 {
    return value
}

main {
    var result: i32 = widen(2i16)
}
~~~

Required semantic result:

1. The parameter has declared type I16.
2. The function result has declared type I32.
3. The return value has type I16.
4. The return value contains an explicit I16-to-I32 conversion.
5. The call references the function SymbolId.
6. The call result has type I32.
7. The variable binding has declared type I32.

### 14.2 Numeric promotion

Source:

~~~text
main {
    var value: u64 = 1u32 + 2u64
}
~~~

Required semantic result:

1. The operands have types U32 and U64.
2. The promotion preference selects U64, and U32 ⪯ U64 holds.
3. The U32 operand has an explicit NumericAssignable conversion to U64.
4. The binary expression result type is U64.
5. The initializer already has the declared type U64.

This example specifies type selection. It does not specify runtime overflow or conversion precision.

### 14.2a Promotion with no common type

Source:

~~~text
main {
    var value: u64 = 1i32 + 2u64
}
~~~

The promotion preference selects U64. I32 ⪯ U64 does not hold, because a negative I32 value has no U64 value. The checker reports E2036. It returns no Fully Typed AST.

### 14.3 Invalid compound assignment

Source:

~~~text
main {
    var value: i8 = 1i8
    value += 2i16
}
~~~

The value has type I16. I16 is not assignable to the place type I8. The checker reports E2002. It returns no Fully Typed AST.

### 14.4 Invalid array length

Source:

~~~text
main {
    var values: i32[2 + 3] = {1i32, 2i32, 3i32, 4i32, 5i32}
}
~~~

The parser may preserve the size expression. The semantic pass reports an invalid array length because the required source form is a non-negative integer literal. The pass returns no Fully Typed AST.

### 14.5 Incomplete parse result

If the parser substitutes nullptr for a missing operand and emits a syntax diagnostic, the returned SurfaceProgram has status Incomplete. The placeholder does not have valid source status. If the parser creates a Void fallback for an invalid variable annotation, the Surface AST represents the annotation as absent. Type checking rejects the entire program before name resolution.

### 14.6 Built-in Void and named `void`

Source:

~~~text
fun no_result() {
    return
}

fun named_result(): void {
    return nullptr
}

main {
    named_result()
}
~~~

The result type of `no_result` is built-in Void because its annotation is omitted. The result type of `named_result` is Named(`void`) because `void` is an identifier. The nullptr return converts to that opaque named type under the current assignability rule.

### 14.7 Literal typing

Source:

~~~text
main {
    var a: i32 = 5
    var b: i8 = -128
    var c: f32 = 1
    var d: i32[2] = {1, 2}
    var e: u64 = 1 << 40
    a += 1
    c *= 2.0
}
~~~

Required semantic result:

1. The initializer of `a` is Literal I32(5).
2. The initializer of `b` is one Literal I8(-128) and no Unary node.
3. The initializer of `c` is Literal F32(1).
4. The initializer of `d` has type Array(I32,2) and two I32 literals.
5. In the initializer of `e`, the left literal has type U64, the right literal has type I64, and the Binary node has type U64.
6. In `a += 1`, the value is Literal I32(1) with no Convert node.
7. In `c *= 2.0`, the value is Literal F32(2).

### 14.8 Literal that does not fit

Source:

~~~text
main {
    var a: i8 = 128
    var b: u8 = -1
    var c: i32 = 1.5
}
~~~

The checker reports E2035 for `a` and E2035 for `b`, because each value is outside the range of its context type. It reports E2002 for `c`, because a floating-point literal does not adapt to an integer type. It returns no Fully Typed AST.



### 14.9 Operator context boundaries

#### Integer variable with a floating-point literal

Source:

~~~text
main {
    var i: i32 = 1
    var mixed: f64 = i + 1.5
}
~~~

Required semantic result:

1. The declared type of `i` is I32.
2. TR-028c gives the adaptable literal `1.5` the peer context I32.
3. TR-028a does not adapt a floating-point literal to an integer type. The literal keeps its default type F64.
4. Promote(I32, F64) selects F64 under Section 7.3. I32 is assignable to F64, so the typed AST converts the left operand from I32 to F64.
5. The binary expression has type F64. The F64 initializer context does not need to pass through the operator to determine this result.

#### Floating-point initializer with a shift expression

Source:

~~~text
main {
    var shifted: f64 = 1 << 3
}
~~~

Required semantic result:

1. The F64 initializer context does not pass through the shift operator.
2. The left literal and shift-count literal both use their default type I64. The shift count receives no context under TR-028d.
3. TR-012 assigns the shift expression the left operand type, I64.
4. I64 is not assignable to F64 under Section 7.2. TR-015 reports E2002 for the initializer.


### 14.10 Explicit casts across supported types

Source:

~~~text
main {
    var wide: i64 = 300
    var small: i8 = wide as i8
    var precise: f64 = 16777217i32 as f64
    var narrower_float: f32 = 3.5d as f32
    var character: char = 65i32 as char
    var text: string = character as string
    var values: vector<i32> = nullptr as vector<i32>
    var fixed: i32[4] = nullptr as i32[4]
    var record: Record = nullptr as Record
    var same_text: string = text as string
}
~~~

The parser constructs a Cast node for each `as` expression and parses the target using the existing type grammar. The type checker accepts numeric conversions, numeric-to-char/Boolean and char/Boolean-to-numeric conversions, char-to-string, null-pointer-to-aggregate conversions, and same-type identity casts. This permits explicit narrowing without adding it to implicit assignability. For example, I64 to I8 is accepted only because the source uses `as i8`.

The current front end does not define runtime behavior for integer overflow, out-of-range floating-point-to-integer conversions, floating-point rounding, numeric-to-char conversion, numeric-to-bool conversion, or the runtime representation of null-backed aggregates. The execution and lowering specification must define those cases before generated code can execute them. Char-to-string and nullptr-to-aggregate reuse the existing CharToString and NullToAggregate conversion rules. Same-type casts are identity operations.


## 15. Open Issues and Design Decisions

### 15.1 Decided architecture

| ID | Decision | Rationale | Requirement affected | Consequence |
| --- | --- | --- | --- | --- |
| DEC-001 | Scope is the implemented Descar language subset. | The user confirmed the current subset as the target. | Sections 1, 6, and 7 | No future language feature is implied. |
| DEC-002 | Surface AST and Fully Typed AST are separate representations. | Syntax and semantic data have different validation needs. | Sections 5 and 6 | Semantic data is stored in a new typed representation. |
| DEC-003 | Typing rules follow the current checker, except for the confirmed mutable-place, compound-assignment, and array-length rules in DEC-006, [DEC-007](#dec-007), and [DEC-008](#dec-008), defined in Section 15.1, the literal-typing and promotion rules in DEC-016, and the numeric-conversion rule in DEC-017. | This preserves the selected checker behavior and isolates each approved correction. | Sections 7 and 8 | Existing promotion, assignability, and operator behavior is recorded, except where DEC-016 or DEC-017 replaces it. |
| DEC-004 | Compile-mode errors remain outside the AST. | A recovered parse tree must not look like a successful semantic result. | Sections 5, 10, and 11 | Error nodes are not valid in either successful AST. Diagnostics remain external. |
| DEC-005 | AST is an internal model. Serialization is out of scope. | No stable external interface was requested or found in the repository. | Section 12 | No external compatibility or encoding contract is required. |
| DEC-006 | A compound assignment value must be assignable to the place type. The operation runs in the place type, and the expression result type is the place type. | The stored value must be valid for the mutable place. | TR-013, GAP-003 | The checker must reject some cases it currently accepts. |
| <a id="dec-007"></a>DEC-007 | Source array lengths are non-negative integer literals. The Fully Typed AST stores the resolved u64 value. | The subset has no constant-expression evaluator for array length. A literal gives a stable type identity. | N-ST-03, TY-07, TR-026, TR-027 | The checker must reject non-literal and negative lengths. |
| <a id="dec-008"></a>DEC-008 | Assignment and update targets must be mutable variable places or index chains rooted in a mutable variable. | Mutation requires a valid mutable storage location. A temporary value is not a place. | N-TP-01–N-TP-03, TR-008, TR-013, TR-014 | The checker must reject some non-place expressions it currently accepts. |
| DEC-009 | TypeTable stores one canonical TypeId per type key. Primitive and special types use kind keys; arrays and vectors use structural keys; named types use exact names. | Canonical IDs make equivalence checks and semantic references unambiguous. | ARC-005, TY-12, TY-17, TY-19 | TypeIds remain local to one compilation unit. |
| DEC-010 | Identity fields use distinct newtype wrappers over u64. NodeId is unique in one AstId. | Distinct wrappers prevent accidental use of an ID from the wrong domain. Local IDs need no persistent encoding. | ARC-001–ARC-005, ERR-011 | IDs are not stable across ASTs, compilation units, or runs. |
| DEC-011 | SourceSpan uses half-open UTF-8 byte offsets. | Byte offsets support source slicing and do not depend on character width. | ARC-010, INV-004 | A SourceMap is required to render line and column positions. |
| DEC-012 | One source file is one CompilationUnitId in this subset. | The language model has no module or multi-file name-resolution rule. | N-ROOT-02, ARC-005, SR-001 | Multi-file compilation requires a new unit and symbol-resolution design. |
| DEC-013 | Assignment and compound assignment are right-associative. Every other binary operator is left-associative. Binding powers derive from the level and associativity in Section 6.4. | Left associativity gives a - b - c the shape (a - b) - c. Deriving the powers from an explicit associativity prevents a row from having the wrong relation. | SYN-001, SYN-002 | This replaces the previous right-associative grouping of every binary row. A program that relied on the old grouping changes meaning. A later change to a level or an associativity requires a language-syntax revision. |
| DEC-014 | Implicit conversions appear as Convert nodes. | Later phases must not infer hidden conversions from operand types. | N-TE-06, N-TE-11, TY-15, ACC-004 | Every conversion is visible to typed-tree validation and lowering. |
| DEC-015 | Function and main declarations are top-level items only. The parser rejects nested declarations with E1004. | The user confirmed that the language does not support nested functions. The parser currently accepts these declarations in statement positions, so the target parser must enforce the language boundary. | BAS-017, GAP-012, SYN-003, INV-021 | Nested function bodies do not enter name resolution or type checking. Loop-control references always stay within one function. |
| DEC-016 | Unsuffixed numeric literals take a context type. Numeric promotion requires both operands to convert to the result type under TY-13. Shifts take the type of the left operand. A compound assignment runs in the place type. ADR-0001 lists the alternatives considered. | Without context typing no literal reaches a type narrower than 64 bits. Without the promotion condition, promotion changes values for 24 ordered type pairs. | TR-001, TR-009, TR-010, TR-012, TR-013, TR-028, TY-13, TY-14, N-SE-01, N-TE-01, N-TE-05, N-TE-06 | A program can change type or be rejected. ADR-0001 lists the cases. |
| DEC-017 | An implicit numeric conversion MUST preserve every value of the source type. TY-13 loses four rows: I32 and U32 to F32, and I64 and U64 to F64. The exactness test in TR-028a is the literal case of the same rule. ADR-0003 lists the alternatives considered. | The table converted four integer types to a floating-point type that cannot hold all their values. A literal above 2^24 was rejected for F32, while an I32 variable with the same value converted without a diagnostic. ADR-0001 chose promotion that defines no run-time conversion, and a rounding conversion needs a run-time definition. | TY-13, TY-14, TR-009, TR-010, TR-013, TR-028a, GAP-015, OPEN-001, Section 13.2 tests 2, 3 and 28 | 32 ordered numeric pairs have no promotion instead of 24. The implicit conversions I32 and U32 to F32, and I64 and U64 to F64, are rejected. I64 and U64 can be converted to floating-point types only through an explicit numeric cast under DEC-020. A mixed integer and floating-point literal expression such as `1 + 2.5` has no promotion (OPEN-006). |
| DEC-018 | TR-028b controls whether a Binary node passes its incoming context to operands. The peer rule in TR-028c is separate. When both operands are flexible, the Binary node passes its context only when TR-028b permits it. An incoming F32 or F64 context is not passed through an operator. | The former TR-028c wording could be read to override TR-028b for two flexible operands. Explicit precedence fixes the type and diagnostic for operator expressions without adding a numeric conversion. | TR-028b, TR-028c, TR-009, TR-012, TR-015, Section 13.2 test 29, Section 14.9, OPEN-006 | `x + 1.5` has type F64 when `x` is I32, because I32 is assignable to F64. It reports E2036 when `x` is I64 or U64, because neither type is assignable to F64. `var x: f64 = 1 << 3` reports E2002 because the shift result is I64 and I64 is not assignable to F64. |
| DEC-019 | Non-void functions are rejected when conservative statement analysis says their bodies can complete normally. Blocks are analyzed in order, if/else requires both branches to avoid fall-through, and all loops are treated as potentially completing normally. | A recursive search for a return can accept a return nested inside a loop that may execute zero times or exit. | TR-025, GAP-008, OPEN-002, Section 13.2 test 30 | A return inside a loop is insufficient by itself. A return after the loop or returns in both branches of an if/else satisfy the check. |
| DEC-020 | Explicit casts use `expression as cast-target`, where cast-target follows the existing type production. Supported changing conversions are numeric-to-numeric, numeric-to-char, char-to-numeric, numeric-to-bool, bool-to-numeric, char-to-string, and nullptr-to-array/vector/custom type. Same-type casts are identities; string parsing, unrelated aggregate conversions, and pointer reinterpretation are rejected. The conversion rules do not expand the implicit assignability table. Runtime behavior for value-changing casts is deferred to the execution and lowering specifications. ADR-0002 records this decision. | These conversions cover useful existing value categories without silently expanding implicit conversions or inventing string parsing and pointer reinterpretation semantics. | SYN-004, N-SE-10, TY-18, Section 14.10, OPEN-001 | Casts are parsed and type-checked. Runtime conversion behavior remains to be specified before the compiler executes value-changing casts. |

### 15.2 Open issues and pending clarification

| ID | Issue or question | Evidence or rationale | Requirement affected | Consequence |
| --- | --- | --- | --- | --- |
| OPEN-001 | Runtime behavior is not defined for arithmetic overflow, division by zero, shift counts outside the operand width, rounding of a floating-point literal to F32, narrowing integer conversions, out-of-range floating-point-to-integer casts, floating-point rounding, numeric-to-char and numeric-to-bool conversions, or how nullptr-backed arrays/vectors/custom types are represented at runtime. Implicit conversions remain lossless, but explicit casts may change or lose value information. | The checker selects types but does not define execution behavior or generated instructions for explicit conversions. | Sections 7.3 and 7.4, DEC-020 | IR, lowering, and runtime specifications must define these behaviors before executable cast semantics are claimed. |
| OPEN-002 | The return-path analysis is conservative and does not prove arbitrary loop termination or unreachable-code properties. | Loops are treated as potentially completing normally, even when their conditions may be constant. | TR-025, INV-015 | This may reject some functions whose loop is in fact non-terminating, but it prevents a return inside a possibly skipped or exited loop from satisfying the non-void return requirement. |
| OPEN-003 | Custom names have no declarations or member access. | The parser accepts names, but the checker does not resolve declarations for them. | TY-09, SR-012 | A future user-defined type feature requires type declarations, name resolution, and new tests. |
| OPEN-004 | The repository has no Fully Typed AST implementation or typed-tree consumer. | Source inspection found no typed-AST or IR-lowering module. | OP-005, OP-006 | This document defines a target contract. Integration design remains future work. |
| OPEN-005 | The checker accepts compatible non-numeric operands for equality, including Void call results. It defines no runtime equality implementation. | The type checker returns Bool after compatibility validation. It defines no equality operation or runtime value for Void. | TR-010, TY-10 | A later execution specification must define behavior for accepted non-numeric equality pairs. Ordering on non-numeric operands is rejected by TR-010. |
| OPEN-006 | An expression with two flexible operands of different numeric kinds, such as `1 + 2.5`, has no promotion under the current rules. The integer literal defaults to I64 and the floating-point literal defaults to F64 because an incoming F32 or F64 context is not passed through the operator. I64 is not assignable to F64 (DEC-017), so the operation reports E2036. | ADR-0004 clarifies that TR-028b takes precedence over TR-028c when both operands are flexible. It does not decide whether the language should later add constant-aware adaptation for mixed flexible operands. | TR-028b, TR-028c, TR-009, TR-010 | `1 + 2.5` and `var f: f64 = 1 + 2.5` report E2036 under the current rules. In contrast, `i + 1.5` where `i: i32` has type F64 because I32 is assignable to F64. A later language decision may add constant-aware adaptation for mixed flexible operands. |
| OPEN-007 | Source spans use UTF-8 byte offsets, but the diagnostic renderer has no normative mapping from byte offsets to displayed line and column numbers. The presentation of tabs, line endings, and Unicode columns is also unspecified. | Section 5.4 defines the canonical byte range, and Section 11.2 leaves visual formatting implementation-dependent. No renderer compatibility contract is established. | Sections 5.4 and 11.2, OP-008 | The byte span remains the canonical diagnostic location. Rendered line and column conventions must not be treated as a stable interface until a separate decision defines them. |

## 16. Bibliographic References

1. Alfred V. Aho, Monica S. Lam, Ravi Sethi, and Jeffrey D. Ullman. *Compilers: Principles, Techniques, and Tools*. 2nd ed. Addison-Wesley, 2006. ISBN 978-0-321-48681-3. Relevant sections: 2.7, “Symbol Tables”; 6.1, “Variants of Syntax Trees”; 6.3, “Types and Declarations”; 6.5, “Type Checking.” The section titles and publication details are available from [Pearson's book page](https://www.informit.com/store/compilers-principles-techniques-and-tools-9780321486813).
2. Aerospace, Security and Defence Industries Association of Europe. *ASD-STE100 Simplified Technical English*. Issue 9, January 15, 2025. Official issue information is available from the [ASD-STE100 website](https://www.asd-ste100.org/) and its [downloads page](https://www.asd-ste100.org/STE_downloads.html).

## 17. Traceability and Specification Review

### 17.1 Requirement and decision traceability

The following matrix identifies the principal sources of the normative rules. Requirement identifiers remain the stable references for implementation and tests. A design decision or textbook reference does not replace a requirement's acceptance criteria.

| Specification area | Requirement or decision references | Verification location | Primary theoretical relationship |
| --- | --- | --- | --- |
| Surface and typed tree separation | BAS-006–BAS-009, ARC-001–ARC-011, INV-001–INV-005, VAL-001–VAL-006 | Sections 10.2 and 13.1–13.3 | Aho et al., Chapter 6, Section 6.1, syntax-tree representations |
| Lexical scope and resolved references | SR-001–SR-017, INV-006, INV-013, DEC-009–DEC-012 | Sections 9.3, 13.1–13.2 | Aho et al., Chapter 2, Section 2.7, symbol tables; Chapter 6, Section 6.3, types and declarations |
| Type identity, assignment, and conversions | TY-01–TY-19, TR-001–TR-028, TC-001–TC-007, INV-007–INV-009, INV-016–INV-020 | Sections 8.9, 13.1–13.3 and ADR-0001, ADR-0003, ADR-0004 verification vectors | Aho et al., Chapter 6, Section 6.5, type checking |
| Operator precedence and AST shape | SYN-001–SYN-003, DEC-013, INV-003–INV-004 | Section 13.2, test 22 | Aho et al., Chapter 4, syntax analysis, and Chapter 6, Section 6.1 |
| Syntax-directed semantic construction | ARC-008, OP-004–OP-007, IF-004–IF-006 | Sections 13.1–13.3 | Aho et al., Chapter 5, syntax-directed translation; Chapter 6, Sections 6.3 and 6.5 |
| Phase interface and failure gating | IF-001–IF-010, VAL-001–VAL-006, TC-001, TC-006–TC-007 | Sections 5.1.1, 8.9, 10.2, and 13.1–13.3 | Phase separation and semantic validation; interface identifiers and failure contracts are Descar-specific |
| Control-flow and return validation | TR-025, GAP-008, DEC-019, OPEN-002 | Section 13.2, test 30 | Static semantic analysis associated with syntax-directed translation; the conservative rule itself is Descar-specific |
| Error boundaries and invalid intermediate state | ERR-001–ERR-014, INV-005, INV-012, INV-023, IF-001–IF-010, VAL-001–VAL-006, OPEN-007 | Sections 10.2, 11, and 13.1–13.3 | Compiler phase separation; diagnostic policy and error codes are Descar-specific |
| Algorithmic expectations | CMP-001–CMP-008 | Section 12.4 plus implementation profiling when a workload is approved | Asymptotic analysis is general algorithmic methodology, not a Descar-specific textbook guarantee |
| Numeric language decisions | DEC-016–DEC-018, OPEN-001, OPEN-006 | ADR-0001, ADR-0003, ADR-0004 and Section 13.2 tests 23–29 | The textbook supplies the type-checking framework. The exact numeric rules are project decisions. |

### 17.2 Theoretical scope boundaries

The specification applies compiler theory only where it informs the Fully Typed AST contract. It does not define a complete lexer, source grammar, intermediate representation, runtime system, optimizer, or target-code generator.

| Textbook topic | Applicability to this document | Required follow-up if the project implements that phase |
| --- | --- | --- |
| Chapter 3, lexical analysis | Token and source-span interfaces are referenced, but token patterns and scanner algorithms are not specified here. | Define the lexical grammar, token priority, escape handling, invalid-token behavior, and lexer tests in a lexer specification. |
| Chapter 4, syntax analysis | Parser output, precedence, associativity, and incomplete parse behavior are in scope where recorded in Sections 5, 6, and 11. | Maintain a complete grammar and parser strategy in the parser specification. Do not infer a parser algorithm from this AST contract. |
| Chapter 5, syntax-directed translation | Relevant to constructing semantic information from source constructs. | Specify translation actions and their invariants if construction is implemented through syntax-directed translation. The current contract does not require a particular implementation technique. |
| Chapter 6, intermediate-code generation | The Fully Typed AST is an input boundary before IR lowering. IR form and lowering rules are explicitly out of scope. | Define IR instructions, control-flow representation, temporary values, lowering rules, and semantic-preservation tests before implementing the consumer described by IF-007. |
| Chapter 7, run-time environments | Runtime storage, activation records, parameter passing, and procedure-call implementation are out of scope. | Specify runtime layout and calling conventions when a concrete execution model is selected. |
| Chapter 8, code generation | Target instructions, register allocation, instruction selection, and target calling conventions are out of scope. | Define target architecture assumptions and code-generation correctness criteria in a separate specification. |
| Chapter 9, machine-independent optimizations | Optimization passes and data-flow analyses are out of scope. | Define each optimization's preconditions, semantic-preservation condition, and verification strategy before adding it to the pipeline. |

The chapter mapping identifies topics for further specification. It does not claim that every described technique is required by Descar or that the textbook prescribes the project's data structures.

### 17.3 Final consistency review

A change to this specification is complete only when the author or reviewer has checked each applicable item below.

| Review ID | Review condition | Evidence |
| --- | --- | --- |
| QA-001 | Every normative requirement has a unique identifier and an objective verification method. | Requirement tables and Section 13. |
| QA-002 | Every numeric typing rule agrees with ADR-0001, ADR-0003, and ADR-0004. | Compare TR-001, TR-009–TR-013, TR-028, and DEC-016–DEC-018 against the ADR decisions and verification vectors. |
| QA-003 | Observed implementation facts are distinguishable from specified behavior and proposed future work. | Status labels in Sections 1 and 4, GAP records, and OPEN records. |
| QA-004 | Every interface identifies its required input state and failure behavior. | Sections 5.1.1 and 12. |
| QA-005 | AST identity, type identity, symbol resolution, spans, conversions, and control-flow references satisfy the invariants in Section 10. | Structural and typed-AST validator tests. |
| QA-006 | Tests cover valid programs, invalid programs, boundary values, and cross-phase behavior described by the requirements. | Section 13.2 test vectors and the acceptance criteria in Section 13.3. |
| QA-007 | Complexity statements distinguish asymptotic expectations from measured performance claims. | Section 12.4 and BAS-016. |
| QA-008 | No unsupported runtime or target-code behavior is implied by type checking or the Fully Typed AST. | Sections 1, 7, 12, and OPEN-001, OPEN-004, OPEN-005. |
| QA-009 | Every remaining limitation is recorded rather than silently resolved. | Section 15.2. |
| QA-010 | Any substantive change to established behavior is linked to an explicit design decision and its verification vectors. | Section 15.1 and the applicable ADR. |
| QA-011 | Current numeric behavior agrees across this specification and ADR-0001, ADR-0003, and ADR-0004. Historical ADR examples and counts are clearly distinguished from current normative results. | Review TR-009 through TR-013 and TR-028, DEC-016 through DEC-018, OPEN-006, Section 13.2 tests 3 and 28 to 29, and the ADR consequence and verification tables. |
| QA-012 | The traceability matrix uses current requirement ranges and connects newly added canonicalization, diagnostic, validator, and finalization requirements to their acceptance evidence. | Compare the entries in Section 17.1 with Sections 5, 8, 10, 11, 13, and 15.2. |

### 17.4 Remaining limitations

The document is a target contract, not evidence that all specified components exist. In particular, OPEN-004 records that the Fully Typed AST implementation and its consumer are not present in the inspected repository state. The specification does not establish that the tests in Section 13 have been implemented or pass.

The execution semantics for overflow, division by zero, shift counts outside the operand width, floating-point literal rounding, and accepted non-numeric equality remain unresolved as listed in OPEN-001 and OPEN-005. Mixed flexible numeric operands remain governed by OPEN-006 and ADR-0004. No quantitative resource limits or performance acceptance thresholds have been approved. Diagnostic line/column rendering and Unicode column conventions remain unspecified under OPEN-007; source byte spans are the normative location data.

These limitations must be resolved in the relevant implementation or follow-up specification before the project claims behavior beyond the defined AST and static semantic contract.
