# Descar Fully Typed Abstract Syntax Tree Specification

**Status:** Normative design specification for the implemented Descar language subset.

**Revision:** 2026-10-22. ADR-0001 amends literal typing, numeric promotion, shift typing, and compound assignment. Decision DEC-016 records the change. ADR-0003 limits implicit numeric conversion to conversions that preserve every value of the source type. Decision DEC-017 records the change.

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
| SourceSpan | A record `{ source_id: SourceId, start_byte: u64, end_byte: u64 }`. The range is half-open. `start_byte <= end_byte` and `end_byte` is within the source file. |
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
| BAS-010 | Specified | The type rules follow the current checker, except for the confirmed place, compound-assignment, and array-length corrections in DEC-006 through DEC-008, the literal-typing and promotion corrections in DEC-016, and the numeric-conversion correction in DEC-017. |
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
| GAP-008 | The non-void return check uses a recursive heuristic. It does not prove that every runtime path returns. | Section 8 records the exact current rule. This limitation MUST remain visible in diagnostics and future control-flow work. |
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

| ID | Requirement | Verification |
|---|---|---|
| SR-013 | SymbolTable, ScopeRecord, SymbolRecord, LoopRecord, TypeTable, and TypeEntry MUST use the fields and links listed above. | Validate one test table with each record kind, then corrupt every required ID link and verify rejection. |

### 5.4 Source locations

SourceSpan uses a half-open byte range [start_byte, end_byte) in one source file. Every concrete node MUST have a span. The span MUST refer to a registered source file. The span of a composite expression or statement MUST cover its complete source construct. A point span is valid when start_byte equals end_byte.

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
              | expression postfix-op | expression "(" arguments? ")"
              | expression "[" expression "]"
~~~

In this grammar, `block = "{" statement* "}"`; `parameter = identifier ":" type`; `parameters = parameter ("," parameter)*`; `identifiers = identifier ("," identifier)*`; `initializers = expression ("," expression)*`; `expressions = expression ("," expression)*`; and `arguments = expression ("," expression)*`. An array literal may have zero expressions. The parser reports E2001 if a declaration has different counts of identifiers and initializers.

For a for-statement, `initializer = declaration | expression`; `condition = expression`; and `increment = expression`. `break = "break"` and `continue = "continue"`. The `initializer`, `condition`, and `increment` clauses can be absent.

`literal` includes numeric, string, character, Boolean, and nullptr forms. `primitive-type` includes the type names in N-ST-01. `prefix-op` includes `!`, unary `-`, `~`, `++`, and `--`. `infix-op` includes the operators in the precedence table, including `=`, but excludes dot. `postfix-op` includes `++` and `--`. The parser permits any expression as a call callee or index base. Semantic rules then restrict calls and mutable places.

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
| n/a | Call, index, postfix update | (), [], ++, -- | n/a | 27 | 26 |

The parser's binding-power table also lists dot at power 27, but the parser has no dot-expression handler. A dot token in expression position is a syntax error. Member access is outside this subset.

| ID | Requirement | Verification |
| --- | --- | --- |
| SYN-001 | The Surface AST grammar and documented clause constraints MUST match the syntax profile in this section. | Parser-conformance tests cover every production, each optional clause, and each stated parser restriction. |
| SYN-002 | Expression AST shape MUST follow the binding powers and associativity in this table. | For every left-associative row, parse x op y op z and verify the shape (x op y) op z. For the assignment row, parse x op y op z and verify the shape x op (y op z). Verify that every infix table entry satisfies the power derivation rule in this section. |
| SYN-003 | A `fun` or `main` declaration MUST be a direct child of the program root. The parser MUST reject either declaration in a block or other statement position with syntax diagnostic E1004. The SurfaceProgram status MUST be Incomplete. | Parse declarations at the root and inside blocks, branches, and loop bodies. Verify root declarations pass, nested declarations report E1004, and semantic analysis does not run. |

Each declaration name has one initializer. The initializer count must equal the name count. Ordinary statements do not require a semicolon. Semicolons delimit the three for-loop clauses. A for-loop initializer is empty, a declaration, or an expression statement. Function and main declarations are top-level items. A block or other statement cannot contain either declaration.

An array type suffix is applied in source order. For example, i32[2][3] is an array of length 2 whose elements are arrays of length 3 of i32.

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

The rows from U8, U16, and U32 to a wider signed integer are the only conversions from an unsigned type to a signed type. They let a value widen to a signed type without a cast (DEC-016). The conversions from I32 and U32 to F32, and from I64 and U64 to F64, are not in the table because the floating-point target cannot represent every value of the integer source type exactly. The thresholds 2^24 for F32 and 2^53 for F64 mark the ranges within which every integer is exactly representable. Some integers above these thresholds remain exactly representable, but others lose precision. A value of type I64 or U64 has no implicit conversion to any other type, and no operation converts it without a cast, which this specification does not provide (Section 7.4). The exactness test in TR-028a applies the same principle to a literal.

The array and vector rules preserve current checker behavior. They allow recursively assignable element types. They are not invariant.

NullPtr is not assignable to String or primitive types.

### 7.3 Numeric promotion

The checker uses this preference order:

F64, F32, U64, I64, U32, I32, U16, I16, U8, I8.

For two numeric operand types, Promote(T₁,T₂) selects the first type C in this list that is equal to either operand. The promotion is defined only when T₁ ⪯ C and T₂ ⪯ C. Otherwise the operation reports E2036 and has no result type. The current checker has no value-preservation condition (GAP-015).

Promotion uses only the conversions in the assignment table in Section 7.2. The Fully Typed AST records each operand conversion as a NumericAssignable conversion.

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
| CharToString | Source is Char and target is String. |
| NullToAggregate | Source is NullPtr and target is Array, Vector, or Named. |
| ArrayElementConversion | Source and target arrays have equal length, and source element is assignable to target element. |
| VectorElementConversion | Source and target vectors have assignable element types. |

Identity conversion does not create a Convert node. Explicit cast syntax is not supported.

| ID | Requirement | Verification |
|---|---|---|
| TY-15 | A non-identity implicit conversion MUST use one of the conversion kinds above and MUST satisfy that kind's source and target conditions. | Construct each allowed conversion and one invalid source/target pair for every conversion kind. |

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
| TR-028a Leaf adaptation | An adaptable literal that receives a numeric context type T takes type T when its signed value v passes the test for its kind. An integer literal with an integer T passes when v is within the range of T. An integer literal with F32 or F64 passes when v is exactly representable in T. A floating-point literal with F32 or F64 passes when its value is finite after rounding to nearest in T. A floating-point literal with an integer T is not compatible by kind. It keeps its default type, and this rule reports nothing. | E2035 when the literal is compatible by kind and fails the test. |
| TR-028b Passing | Only a flexible expression passes its context type to its children. Grouping passes any numeric T. Negate, BitwiseNot, and the Binary operators `+ - * / % & \| ^` pass an integer T to every operand. A shift passes an integer T to its left operand only. An operator node does not pass F32 or F64. Operand and result types then follow TR-009 to TR-012. | None. |
| TR-028c Peer | For `+ - * / % & \| ^ == != < <= > >=`, if exactly one operand is flexible and the other operand is not flexible and has numeric type S, the flexible operand receives S as its context type. This replaces the context type from the position of the Binary node. If both operands are flexible, both receive the context type of the Binary node. Shifts have no peer rule. | None. |
| TR-028d No context | A flexible expression with no context type takes default types, and operators apply TR-009 to TR-012 to those types. Index expressions, conditions, shift counts, and operands of non-flexible unary operators receive no context type. A non-numeric context type has no effect. | None. |

A prefix Negate applied directly to an adaptable literal MUST be one Literal node in the typed tree. The node holds the negated value, and its span covers both tokens. The most negative value of a signed type has no positive counterpart in that type, so `-128` for I8 cannot be a Negate over an I8 literal. A Negate applied to any other expression, including `-(128)` and `- -5`, is a Unary node. A literal with `width_suffix` true is not folded.

The Number variant of a typed numeric Literal corresponds to its result_type, and its value MUST be representable in that type (INV-022).

### 8.3 Unary rules

| Rule ID | Preconditions | Result |
| --- | --- | --- |
| TR-005 Negate | Operand is numeric. | Operand type. |
| TR-006 Logical not | Operand type is Bool. | Bool. |
| TR-007 Bitwise not | Operand is an integer type. | Operand type. |
| TR-008 Increment or decrement | Operand is a mutable Place and has numeric type. A variable place or index-place chain must be rooted in a mutable variable. | Place type. Prefix or postfix form does not change the type. |

The checker reports E2018 for invalid numeric or integer operands, E2019 for invalid logical-not operands, and E2024 for an immutable target. Under DEC-008, a non-place operand also reports E2033.

### 8.4 Binary rules

Let Num be the ten integer and floating-point types. Let Int be the eight integer types. Let Compatible(A,B) mean A ⪯ B or B ⪯ A.

| Rule ID | Operators | Preconditions | Result type |
| --- | --- | --- | --- |
| TR-009 Arithmetic | +, -, *, /, % | Both operands are in Num. Promote both operands to C = Promote(T₁,T₂). Both operands MUST be assignable to C, otherwise E2036. A flexible operand takes its type from TR-028c before promotion. | Promoted type. |
| TR-010 Comparison | ==, !=, <, <=, >, >= | If both operands are numeric, promote them under the TR-009 condition. Otherwise Compatible(T₁,T₂) must be true. Convert operands to the common type defined below. A flexible operand takes its type from TR-028c. | Bool. |
| TR-011 Logical | &&, \|\| | Both operands are Bool. | Bool. |
| TR-012 Bitwise | &, \|, ^, <<, >> | Both operands are in Int. For `&`, `\|`, and `^`, promote both operands to C = Promote(T₁,T₂) under the TR-009 condition. For `<<` and `>>`, do not promote. The right operand keeps its own integer type and receives no conversion. | For `&`, `\|`, and `^`, the promoted type. For `<<` and `>>`, the type of the left operand. |
| TR-013 Compound assignment | +=, -=, *=, /=, %=, &=, \|=, ^=, <<=, >>= | The target is a mutable place of type T. For `+=`, `-=`, `*=`, `/=`, and `%=`, T is in Num. For `&=`, `\|=`, and `^=`, T is in Int. The value is checked with context type T under TR-028, and its type S satisfies S ⪯ T. The value receives a conversion to T when S is not T. The operation runs in T. For `<<=` and `>>=`, T and S are integer types and the value is not converted. | Place type. |

The current checker permits modulo for every numeric type. This specification preserves that rule. Comparisons use Compatible for both equality and ordering when operands are not numeric.

For a non-numeric comparison, choose the common operand type as follows: if T₁ ≡ T₂, use T₁; otherwise, if T₁ ⪯ T₂, convert the left operand to T₂; otherwise, if T₂ ⪯ T₁, convert the right operand to T₁. The comparison is invalid if neither direction holds. For numeric comparisons, use Promote(T₁,T₂) and convert both operands to that type. These conversion nodes make the current checker's compatibility result explicit in the Fully Typed AST. They do not add accepted operand pairs.

The current checker accepts compatible non-numeric operands for ordering operators as well as equality operators. This specification records that type-checking behavior. It does not define runtime ordering behavior for strings or aggregate values.

TR-013 requires the value to be assignable to the place type. For example, an I16 value cannot be applied to an I8 target. It also requires a valid mutable place.

The checker reports E2011 for invalid bitwise operands, E2012 or E2017 for invalid logical operands, E2013 or E2016 for invalid arithmetic operands, and E2014 for incompatible comparison operands. It reports E2024 for an immutable compound-assignment target, E2033 for a non-place target under DEC-008, E2002 when the value cannot convert to the place type, E2035 when a literal does not fit its context type, and E2036 when numeric operands have no common type.

### 8.5 Assignment, declaration, call, and index rules

| Rule ID | Preconditions | Postcondition | Possible errors |
| --- | --- | --- | --- |
| TR-014 Assignment | Target is a mutable variable or an index-place chain rooted in a mutable array/vector variable. The value is checked with context type T₂, and its type T₁ is assignable to T₂. | Insert a conversion if needed. Result type is T₂. | E1003 if parsing rejects a target outside variable/index syntax; E2024 immutable target; E2025 undefined variable; E2033 parsed expression is not a mutable place; E2002 incompatible value. |
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
| TR-025 Non-void function return check | The checker applies the current function_has_return rule described below. | A non-void function passes when the current heuristic returns true. | E2003 when the heuristic returns false. |
| TR-026 Array length | Length is a non-negative integer literal that fits u64. An inferred array-literal length is its element count and must also fit u64. | The resolved array type stores that u64 length. | E2034 for a non-literal, negative, or out-of-range length. |

Assignment, compound assignment, increment, and decrement require a place. The place must be a variable or an index chain rooted in a mutable variable. An array literal, call result, or other temporary expression cannot be a place.

The current function_has_return rule is:

1. A Return statement returns true.
2. A Block returns true if any contained statement returns true.
3. A While or For returns the result for its body.
4. An If returns true only if its then branch and present else branch return true.
5. An If without an else returns false.
6. Other statements return false.

This rule does not prove that all execution paths return. It is recorded to match current checker behavior. It MUST NOT be described as path-sensitive proof.

### 8.7 Type syntax resolution and array lengths

The Fully Typed AST stores array length as a u64 constant. The source form must be a non-negative integer literal. The parser may retain other expressions in an incomplete Surface AST, but semantic analysis must report an invalid array-length diagnostic for them.

This rule reflects the confirmed array-length decision in DEC-007. It resolves the current parser/checker mismatch.

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
| TR-010 | `Γ ⊢ e₁:T₁`, `Γ ⊢ e₂:T₂`, `Common(T₁,T₂)=C ⇒ Γ ⊢ e₁ cmp e₂:Bool`, for `cmp∈{==,!=,<,<=,>,>=}`. `Common` uses promotion under the TR-009 condition for two numeric operands and assignability compatibility for other operands. |
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
| TR-025 | If a function result is Void, no return-path condition applies. If its result is non-Void, `HasReturn(body)=true ⇒ Γ ⊢ functionDecl:ok`. `HasReturn` follows the six cases listed in Section 8.6. |
| TR-026 | For array type syntax with length literal `k`, `0≤k≤MAX_U64 ⇒ Length(k)=k`. For an inferred array literal with `n` elements, `0≤n≤MAX_U64 ⇒ Length(elements)=n`. |
| TR-027 | `Γ ⊢ primitiveSyntax ⇒ its canonical primitive TypeId`; `Γ ⊢ namedSyntax(x) ⇒ Named(x)`; `Γ ⊢ τ⇒T` and `T≠Void ⇒ Γ ⊢ Vector(τ)⇒Vector(T)`; `Γ ⊢ τ⇒T` and `Length(k)=n ⇒ Γ ⊢ Array(τ,k)⇒Array(T,n)`. Named(`"void"`) is not the built-in Void type. |
| TR-028 | `e` is an adaptable literal, `T` is numeric, and the signed value `v` of `e` passes the TR-028a test for its kind `⇒ Γ ⊢ e ⇐ T ⇒ T`. A flexible expression passes `T` to its operands under TR-028b. A flexible operand next to a non-flexible operand of numeric type `S` is checked with `S` under TR-028c. |

`MAX_U64` means the largest value representable by an unsigned 64-bit integer. A type-resolution error or a false premise has the diagnostic listed for that rule.

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

The current checker visits top-level declarations in source order. It does not perform a whole-unit declaration prepass.

The function body uses a function scope. The checker does not add a second block scope for the outer function body. A standalone block creates a block scope. Each if branch creates the scope of its block. A while condition is checked in the surrounding scope. The checker then creates a loop scope for the body; a block body creates a nested block scope. A for statement creates a loop scope before its initializer. This scope covers the initializer, condition, increment, and body. If the body is a block, that block creates a nested scope.

## 10. Invariants and Validity Criteria

The AST validator must report the first failed invariant category and continue where safe.

| ID | Invariant | Stage | Required action on failure |
| --- | --- | --- | --- |
| INV-001 | Every node ID is unique in its AST instance. | Construction and structural validation | Internal error. Do not run semantic analysis. |
| INV-002 | Owned children form a tree with one parent per non-root node. | Structural validation | Internal error. Do not run later phases. |
| INV-003 | Every required child exists and every child list follows source order. | Structural validation | Internal error for constructed nodes; parser diagnostic for malformed source. |
| INV-004 | Every span refers to one source file and uses valid half-open byte positions. Composite spans cover the full construct. | Parser output and typed AST validation | Internal error if a constructed node violates it. |
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
| INV-022 | A numeric Literal has a Number variant that corresponds to its result_type and a value that is representable in that type. | Typed AST validation | Internal error. |

### 10.1 Validity states

| State | Criteria | Accepted by |
| --- | --- | --- |
| Structurally valid Surface AST | Node and span invariants pass. Parse diagnostics are absent. | Name resolution and type checking. |
| Incomplete Surface AST | Parser recovery occurred or parser diagnostics exist. | Diagnostic display and AST debugging only. |
| Semantically valid | All names, types, operations, declarations, and control constraints pass the rules in Sections 7–9. | Fully Typed AST construction. |
| Fully typed | Semantically valid; all expressions and declarations have resolved type data; all references use IDs; all conversions are explicit; all loop-control references are valid in the current function. | Typed AST validation. |
| Ready for lowering | Fully typed and INV-001 through INV-022 pass. No blocking design question remains for the program's constructs. | A later lowering phase that implements this contract. |
| Invalid AST | Any structural or typed invariant fails. | No compiler phase after the failing validator. |

An AST with an error is not semantically valid. Diagnostics do not convert an invalid source program into a valid AST.

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

Every diagnostic MUST contain:

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

The existing CompileError model provides a phase-specific variant, optional code, message, span, and optional help. Related spans and NodeId are specified additions.

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
| N-ROOT-01–N-ROOT-02 | Root construction tests for empty and non-empty source units. |
| N-SE-01–N-SE-09 | One constructor and one structural-validation test per expression variant. |
| N-SS-01–N-SS-11 | One constructor and one structural-validation test per statement variant. |
| N-SD-01–N-SD-02 | Constructor, ownership, source-span, and ordering tests for bindings and parameters. |
| N-ST-01–N-ST-04 | Type-syntax tests for each primitive, named type, nested array, and vector. |
| N-TE-01–N-TE-11 | Typed-node tests. Verify result_type and all linked IDs. |
| N-TD-01–N-TD-02 | Typed declaration-node tests. Verify declared types, symbol links, source spans, and owner links. |
| N-TP-01–N-TP-03 | Place tests for mutable variables, grouped places, mutable arrays, immutable targets, and invalid indexes. |
| TY-01–TY-17 | Type identity, canonicalization, assignability, promotion, conversion, and invalid-category tests. |
| TR-001–TR-028 | Positive and negative tests for every typing rule, including primitive, named, vector, nested-array, and invalid array type syntax. |
| SR-001–SR-012 | Scope, shadowing, duplicate, recursion, forward-reference, custom type, and symbol-kind tests. |
| SR-013 | Record-shape, link-integrity, and type-canonicalization tests. |
| SYN-002 | AST-shape tests for left associativity, assignment right associativity, and precedence for every operator class. |
| INV-001–INV-022 | One validator test per invariant. Each negative case must identify the violated invariant. |
| ERR-001–ERR-011 | Parser recovery, blocking errors, accumulated diagnostics, diagnostic fields, error codes, severity, CLI phase boundary, and internal-error separation tests. |
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
21. Primitive, named, vector, and nested-array type syntax. Verify canonical type identity and reject malformed type syntax.
22. Operator associativity. Parse each source below and verify the exact node shape: 10 - 3 - 2 as (10 - 3) - 2; 2 - 3 + 4 as (2 - 3) + 4; 100 / 10 / 5 as (100 / 10) / 5; 1 << 2 << 3 as (1 << 2) << 3; a - b *c - d as (a - (b* c)) - d; a = b = c as a = (b = c); -a *b as (-a)* b.
23. The minimum, the maximum, and each value one outside the range, for all eight integer types, as initializers and as peer operands. Verify E2035 outside the range.
24. F32 overflow, and integer exactness at 2^24 for F32 and 2^53 for F64 with the neighboring values. Verify E2035 for an inexact integer literal.
25. Negation folding: `-128` for I8, `-9223372036854775808` for I64, `-(128)` for I8, and a suffixed literal. Verify that a folded literal is one Literal node.
26. The peer rule with the literal on either side of a non-literal operand, with a flexible operator tree on one side, and with a float literal next to an integer operand.
27. Context passing through array literals, nested array literals, call arguments, return values, and shifts with a non-literal count. Verify that an F32 or F64 context does not pass through an operator.
28. Integer-to-float conversion of a non-literal value. Verify that an I32 or U32 variable assigned to F32, and an I64 or U64 variable assigned to F64, reports E2002, including a variable that holds 16777217. Verify that `x + y` with `x: i32` and `y: f32` reports E2036, and that `x + y` with `x: i32` and `y: f64` has type F64 with a NumericAssignable conversion on `x`. Verify that `var f: f64 = 7 / 2` reports E2002, because the expression has type I64.

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
| ACC-007 | The typed validator rejects each single-invariant mutation in INV-001 through INV-022. |
| ACC-008 | The complete test suite covers every operator, type category, semantic reference, and error category in this document. |
| ACC-009 | No unsupported type category or language feature can be inserted into a successful Fully Typed AST. |
| ACC-010 | The final document review confirms that each requirement has an ID and a verification method. |

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

## 15. Open Issues and Design Decisions

### 15.1 Decided architecture

| ID | Decision | Rationale | Requirement affected | Consequence |
| --- | --- | --- | --- | --- |
| DEC-001 | Scope is the implemented Descar language subset. | The user confirmed the current subset as the target. | Sections 1, 6, and 7 | No future language feature is implied. |
| DEC-002 | Surface AST and Fully Typed AST are separate representations. | Syntax and semantic data have different validation needs. | Sections 5 and 6 | Semantic data is stored in a new typed representation. |
| DEC-003 | Typing rules follow the current checker, except for the confirmed mutable-place, compound-assignment, and array-length rules in DEC-006 through DEC-008, the literal-typing and promotion rules in DEC-016, and the numeric-conversion rule in DEC-017. | This preserves the selected checker behavior and isolates each approved correction. | Sections 7 and 8 | Existing promotion, assignability, and operator behavior is recorded, except where DEC-016 or DEC-017 replaces it. |
| DEC-004 | Compile-mode errors remain outside the AST. | A recovered parse tree must not look like a successful semantic result. | Sections 5, 10, and 11 | Error nodes are not valid in either successful AST. Diagnostics remain external. |
| DEC-005 | AST is an internal model. Serialization is out of scope. | No stable external interface was requested or found in the repository. | Section 12 | No external compatibility or encoding contract is required. |
| DEC-006 | A compound assignment value must be assignable to the place type. The operation runs in the place type, and the expression result type is the place type. | The stored value must be valid for the mutable place. | TR-013, GAP-003 | The checker must reject some cases it currently accepts. |
| DEC-007 | Source array lengths are non-negative integer literals. The Fully Typed AST stores the resolved u64 value. | The subset has no constant-expression evaluator for array length. A literal gives a stable type identity. | N-ST-03, TY-07, TR-026, TR-027 | The checker must reject non-literal and negative lengths. |
| DEC-008 | Assignment and update targets must be mutable variable places or index chains rooted in a mutable variable. | Mutation requires a valid mutable storage location. A temporary value is not a place. | N-TP-01–N-TP-03, TR-008, TR-013, TR-014 | The checker must reject some non-place expressions it currently accepts. |
| DEC-009 | TypeTable stores one canonical TypeId per type key. Primitive and special types use kind keys; arrays and vectors use structural keys; named types use exact names. | Canonical IDs make equivalence checks and semantic references unambiguous. | ARC-005, TY-12, TY-17 | TypeIds remain local to one compilation unit. |
| DEC-010 | Identity fields use distinct newtype wrappers over u64. NodeId is unique in one AstId. | Distinct wrappers prevent accidental use of an ID from the wrong domain. Local IDs need no persistent encoding. | ARC-001–ARC-005, ERR-011 | IDs are not stable across ASTs, compilation units, or runs. |
| DEC-011 | SourceSpan uses half-open UTF-8 byte offsets. | Byte offsets support source slicing and do not depend on character width. | ARC-010, INV-004 | A SourceMap is required to render line and column positions. |
| DEC-012 | One source file is one CompilationUnitId in this subset. | The language model has no module or multi-file name-resolution rule. | N-ROOT-02, ARC-005, SR-001 | Multi-file compilation requires a new unit and symbol-resolution design. |
| DEC-013 | Assignment and compound assignment are right-associative. Every other binary operator is left-associative. Binding powers derive from the level and associativity in Section 6.4. | Left associativity gives a - b - c the shape (a - b) - c. Deriving the powers from an explicit associativity prevents a row from having the wrong relation. | SYN-001, SYN-002 | This replaces the previous right-associative grouping of every binary row. A program that relied on the old grouping changes meaning. A later change to a level or an associativity requires a language-syntax revision. |
| DEC-014 | Implicit conversions appear as Convert nodes. | Later phases must not infer hidden conversions from operand types. | N-TE-06, N-TE-11, TY-15, ACC-004 | Every conversion is visible to typed-tree validation and lowering. |
| DEC-015 | Function and main declarations are top-level items only. The parser rejects nested declarations with E1004. | The user confirmed that the language does not support nested functions. The parser currently accepts these declarations in statement positions, so the target parser must enforce the language boundary. | BAS-017, GAP-012, SYN-003, INV-021 | Nested function bodies do not enter name resolution or type checking. Loop-control references always stay within one function. |
| DEC-016 | Unsuffixed numeric literals take a context type. Numeric promotion requires both operands to convert to the result type under TY-13. Shifts take the type of the left operand. A compound assignment runs in the place type. ADR-0001 lists the alternatives considered. | Without context typing no literal reaches a type narrower than 64 bits. Without the promotion condition, promotion changes values for 24 ordered type pairs. | TR-001, TR-009, TR-010, TR-012, TR-013, TR-028, TY-13, TY-14, N-SE-01, N-TE-01, N-TE-05, N-TE-06 | A program can change type or be rejected. ADR-0001 lists the cases. |
| DEC-017 | An implicit numeric conversion MUST preserve every value of the source type. TY-13 loses four rows: I32 and U32 to F32, and I64 and U64 to F64. The exactness test in TR-028a is the literal case of the same rule. ADR-0003 lists the alternatives considered. | The table converted four integer types to a floating-point type that cannot hold all their values. A literal above 2^24 was rejected for F32, while an I32 variable with the same value converted without a diagnostic. ADR-0001 chose promotion that defines no run-time conversion, and a rounding conversion needs a run-time definition. | TY-13, TY-14, TR-009, TR-010, TR-013, TR-028a, GAP-015, OPEN-001, Section 13.2 tests 2, 3 and 28 | 32 ordered numeric pairs have no promotion instead of 24. A program that converts a 32-bit or 64-bit integer to a floating-point type is rejected. No program can convert I64 or U64 to a floating-point type until the explicit conversions in ADR-0002 exist. A mixed integer and floating-point literal expression such as `1 + 2.5` has no promotion (OPEN-006). |

### 15.2 Open issues and pending clarification

| ID | Issue or question | Evidence or rationale | Requirement affected | Consequence |
| --- | --- | --- | --- | --- |
| OPEN-001 | Runtime behavior is not defined for arithmetic overflow, division by zero, shift counts outside the operand width, and rounding of a floating-point literal to F32. After ADR-0001 and ADR-0003, no promotion, compound assignment, or implicit conversion changes the value of a typed operand. | The checker selects types but does not define execution behavior. | Section 7.3 | IR and runtime specifications must define lowering behavior before executable semantics are claimed. |
| OPEN-002 | The current non-void return heuristic does not prove return on all paths. | The checker uses a recursive presence test, not control-flow analysis. | TR-025, INV-015 | This specification records current checker behavior. A sound control-flow rule requires a later design decision. |
| OPEN-003 | Custom names have no declarations or member access. | The parser accepts names, but the checker does not resolve declarations for them. | TY-09, SR-012 | A future user-defined type feature requires type declarations, name resolution, and new tests. |
| OPEN-004 | The repository has no Fully Typed AST implementation or typed-tree consumer. | Source inspection found no typed-AST or IR-lowering module. | OP-005, OP-006 | This document defines a target contract. Integration design remains future work. |
| OPEN-005 | The checker accepts compatible non-numeric operands for equality and ordering, including Void call results. It defines no runtime comparison implementation. | The type checker returns Bool after compatibility validation. It defines no compare operation or runtime value for Void. | TR-010, TY-10 | A later execution specification must define behavior for every accepted non-numeric pair or narrow the type rule. |
| OPEN-006 | An expression with an integer literal operand and a floating-point literal operand, such as `1 + 2.5`, has no promotion. TR-028c passes no F32 or F64 context through an operator, so the integer literal has type I64 and the other operand has type F64. I64 is not assignable to F64 (DEC-017), and the operation reports E2036. | The earlier table accepted this program through the I64 to F64 row. The last sentence of TR-028c, which gives two flexible operands the context type of the Binary node, can be read to conflict with the F32 and F64 exception in TR-028b. | TR-028b, TR-028c, TR-009, TR-010 | A later decision must let an integer literal adapt to a floating-point peer, for example by evaluating a flexible expression as a constant and checking the value against the context type, or require the programmer to write a floating-point literal. |

## 16. Bibliographic References

1. Alfred V. Aho, Monica S. Lam, Ravi Sethi, and Jeffrey D. Ullman. *Compilers: Principles, Techniques, and Tools*. 2nd ed. Addison-Wesley, 2006. ISBN 978-0-321-48681-3. Relevant sections: 2.7, “Symbol Tables”; 6.1, “Variants of Syntax Trees”; 6.3, “Types and Declarations”; 6.5, “Type Checking.” The section titles and publication details are available from [Pearson's book page](https://www.informit.com/store/compilers-principles-techniques-and-tools-9780321486813).
2. Aerospace, Security and Defence Industries Association of Europe. *ASD-STE100 Simplified Technical English*. Issue 9, January 15, 2025. Official issue information is available from the [ASD-STE100 website](https://www.asd-ste100.org/) and its [downloads page](https://www.asd-ste100.org/STE_downloads.html).
