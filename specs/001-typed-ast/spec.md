# Feature Specification: Full Typed AST

**Feature Branch**: `001-typed-ast`

**Created**: 2026-10-04

**Status**: Draft

**Input**: User description: "Implementare un full typed AST completo e coerente per il linguaggio previsto dalla grammatica del progetto, in modo da rappresentare esplicitamente tutte le principali categorie sintattiche e le relative informazioni tipologiche. L'AST deve modellare attraverso nodi distinti e semanticamente significativi tutti i costrutti previsti dalla grammatica, incluse espressioni, istruzioni, dichiarazioni, riferimenti, definizioni di tipo e ogni ulteriore categoria sintattica prevista."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Grammar-Complete Node Coverage (Priority: P1)

A compiler developer reads a parsed Descar program. The developer finds one distinct AST node for each syntactic construct in the grammar.

- Expressions: binary, unary, literal, grouping, variable reference, assignment, call, array access, and array literal.
- Statements: expression statement, variable declaration, function declaration, main function, if/else, while, for, block, return, break, and continue.
- Type references: all primitive types, array type, vector type, void, nullptr, and custom (user-defined) type.

**Why this priority**: Without complete node coverage, the AST cannot represent an arbitrary valid program. This blocks each downstream phase: type checking, semantic analysis, IR generation, and printing.

**Independent Test**: Parse the example file `large_toy_program.dr` or `sccp_test.dr`. The parser returns a `Vec<Stmt>`. Each statement and each sub-expression maps to a concrete, named node variant. No construct uses a generic catch-all variant.

**Acceptance Scenarios**:

1. **Given** a `.dr` source file that contains all statement kinds, **When** the parser builds the AST,**Then** each statement maps to a node variant with a unique name. Two different constructs do not use the same variant.
2. **Given** an expression that uses all binary operators (arithmetic, comparison, logical, bitwise, shift, and compound assignment), **When** the parser parses the expression, **Then** each operator is a separate `BinaryOp` variant inside an `Expr::Binary` node. The node keeps the left operand and the right operand.
3. **Given** a function declaration with parameters and a typed return annotation, **When** the parser parses the declaration, **Then** the `Stmt::Function` node contains the function name, the complete ordered list of `Parameter` nodes, the declared return type, and the body block. Each `Parameter` node contains the parameter name and type.
4. **Given** a `for` loop with an initializer, a condition, and an increment clause, **When** the parser parses the loop, **Then** the `Stmt::For` node keeps the initializer, the condition, and the increment clause as separate optional fields. The node also keeps the body.

---

### User Story 2 - Uniform and Complete Type Model (Priority: P2)

A type checker or a semantic analysis pass reads a declared type from the AST. The type is on a variable, a parameter, a return position, or a type annotation. The pass does not infer the type. The pass does not rebuild the type from raw tokens. Each position where the grammar permits a type annotation has a first-class `Type` value.

**Why this priority**: Downstream semantic phases (type checking, type inference, and diagnostics) need a complete and uniform type model. If the model has gaps, the type checker must parse again or guess. This brings back ambiguity.

**Independent Test**: Use a `.dr` source file with variables of all primitive types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, `bool`). The file also has arrays (`i32[10]`), vectors (`vector<f64>`), custom types, `void`, and `nullptr`. Each type annotation position in the AST holds the exact `Type` variant, with no ambiguity.

**Acceptance Scenarios**:

1. **Given** the variable declaration `var x: i32 = 0`, **When** you examine the AST node, **Then** the `type_annotation` field contains `Type::I32` and the `is_mutable` flag is `true`.
2. **Given** the constant declaration `const y: bool = true`, **When** you examine the AST node, **Then** the `type_annotation` field contains `Type::Bool` and the `is_mutable` flag is `false`.
3. **Given** the array type annotation `i32[10]`, **When** you examine the AST node, **Then** the `Type::Array` node contains `element_type: Type::I32` and a `size` expression that evaluates to the literal `10`.
4. **Given** the function parameter `n: f64`, **When** you examine the AST node, **Then** the `Parameter` node contains `name = "n"` and `type_annotation = Type::F64`.
5. **Given** a function with no return type annotation, **When** the parser parses the function, **Then** the `return_type` field has the default value `Type::Void`.

---

### User Story 3 - Structural Integrity and Source Provenance (Priority: P2)

Each AST node has a `SourceSpan`. The span gives the position of the node in the source text. A node with a direct text origin (a source token or a range of tokens) has a span that reproduces the exact source fragment. The parser makes some nodes to fill an implicit default. An example is `Type::Void` for a function with no declared return type. The span of such a node has zero width and is at the last consumed token. This span is correct for position-based diagnostics. It does not reproduce a source fragment. The structure of the nodes is explicit. For example, the else branch of an `if` is an `ElseBranch` variant. It is not a raw `Option<Box<Stmt>>`. A program can reach each node without a hidden rule.

**Why this priority**: Error reporting, diagnostic highlighting, and AST pretty-printing need precise source positions. An ambiguous structure causes a silent loss of information.

**Independent Test**: Use any source file. Each AST node (expression and statement) has a `span()` method. The method returns a valid `SourceSpan` that maps to the original source text.

**Acceptance Scenarios**:

1. **Given** a statement at line 5, columns 3 to 20 in the source, **When** you check the `span()` of the AST node, **Then** the returned `SourceSpan` identifies this source range correctly.
2. **Given** an `if` statement with an `else if` continuation, **When** you check the AST, **Then** the `else_branch` field is `ElseBranch::ElseIf(...)`. The payload of `ElseIf` is the nested `Stmt::If` node. The field is not `ElseBranch::Block` and it is not `ElseBranch::None`.
3. **Given** a block with several statements, **When** you read the `Stmt::Block` node, **Then** each child statement is available by index in the `statements` vector, in source order.
4. **Given** the source `if (x > 0) { return x; } else return 0;`, **When** the parser parses the statement, **Then** the parser reports the error E1004. The `else_branch` field is `ElseBranch::None`. The AST does not store the statement `return 0;`.

---

### User Story 4 - Typed AST Readiness for Downstream Phases (Priority: P3)

A semantic analysis pass or an IR generator builds a typed tree from the parsed AST. The pass records a resolved type on each expression node at each depth. The `Expr` enum does not change. The typed tree has the same structure as the untyped tree. `TypedExpr` and `TypedStmt` correspond to `Expr` and `Stmt` node by node. Each `TypedExpr` owns its child `TypedExpr` nodes. To read the type of a sub-expression, go from its parent node to the child node. Parsing phases use plain `Expr` and `Stmt` trees. Semantic phases use `TypedExpr` and `TypedStmt` trees.

**Why this priority**: Downstream passes (type inference, IR generation, and optimization) need the type of each sub-expression, not only the type of the outer expression. One `Option<Type>` next to an unchanged `Expr` cannot hold the types of the operands in that `Expr`. For this reason, the typed tree repeats the structure of the untyped tree, and each expression node has its own type slot. `Expr` stays unchanged. Parser-side code does not handle a type field before type checking starts.

**Independent Test**: Build the typed tree from a fully parsed program. Run the type-checking pass on the typed tree. After the pass, each `TypedExpr` node at each depth that stands for a typed construct shows its resolved type in `TypedExpr::resolved_type`. A typed construct is a literal, a variable, a call, a binary expression, or a unary expression. This includes operands inside other expressions. It also includes conditions, initializers, and return values inside statements. The caller does not derive a type again.

**Acceptance Scenarios**:

1. **Given** the expression `1 + 2`, **When** the type checker has processed the `TypedExpr` tree, **Then** the root node has `kind` equal to `TypedExprKind::Binary` and `resolved_type` equal to `Some(Type::I32)` (or the promoted numeric type). The left operand and the right operand are `TypedExpr` nodes with `kind` equal to `TypedExprKind::Literal`. Each operand has `resolved_type` equal to `Some(Type::I32)`.
2. **Given** a `TypedExpr` with `kind` equal to `TypedExprKind::Variable` for a declared `var x: f64`, **When** the type checker completes, **Then** `resolved_type` is `Some(Type::F64)`. This is true if the node is a complete statement expression. It is also true if the node is an operand of `x * 2.0`.
3. **Given** an expression node with an unknown type, for example a node that uses an undeclared variable, **When** the type checker completes, **Then** `resolved_type` is `None` for that node. Other expressions with a known type keep `Some(T)`. An error in one expression does not stop the annotation of other expressions. In `undeclared + 1`, the literal `1` has `Some(Type::I32)` and the undeclared variable has `None`.
4. **Given** the code `if (x > 0) { return x + 1; } else if (x < 0) { return x - 1; }` and `var x: i32` in scope, **When** the type checker completes,   **Then** the condition of the `TypedStmt::If` node has `Some(Type::Bool)`. Each operand of the condition has `Some(Type::I32)`. The returned expression and its operands have `Some(Type::I32)`. The `TypedElseBranch::ElseIf` payload holds a `TypedStmt::If`. The condition and the operands of that `TypedStmt::If` have annotations in the same way.
5. **Given** a parsed `Expr` tree `e`, **When** the pass builds the typed tree with `TypedExpr::from_expr(&e)`, **Then** `e` does not change. Each `resolved_type` in the result is `None`. The call `erase()` on the result returns a tree that is structurally equal to `e`.
6. **Given** the declaration `var a: i32[2 + 3]`,  **When** the pass builds the typed tree, **Then** the type annotation in the `TypedStmt::VarDeclaration` node is the same `Type::Array` value as in the untyped tree. Its `size` field is a plain `Expr` with no resolved type.

---

### Edge Cases

- Multiple bindings (`var a, b: i32 = 1, 2`): Each `VarBinding` is a separate entry with its own initializer. In a syntactically valid program, each initializer is `Some` (FR-017). All entries share the same declared type annotation. In the typed tree, the `TypedStmt::VarDeclaration` node has one `TypedVarBinding` for each binding. Each initializer is a `TypedExpr` with its own `resolved_type`.
- A declaration with a different number of names and initializers (`var a, b: i32 = 1`): The parser reports the error E2001 (FR-017). The AST has one `VarBinding` for each name. The `VarBinding` for `b` has `None` as its initializer. The `TypedVarBinding` for `b` also has `None`. In `var a: i32 = 1, 2`, the AST does not store the initializer `2`.
- A declaration with no initializer (`var a: i32;` or `const c: i32;`): The grammar does not permit it. The declaration has one name and zero initializers. The parser reports the error E2001 (FR-017). It also reports the errors for the missing `=` and the missing expression. The AST has one `VarBinding` with `None` as its initializer.
- A declaration with no name (`var : i32 = 1`): The parser reports the error E1008 (FR-017). The parser does not make a `Stmt::VarDeclaration`.
- A function with no parameters: The `parameters` field is an empty `Vec<Parameter>`. The field is not absent.
- `nullptr`: The AST represents `nullptr` as `Expr::Literal { value: LiteralValue::NullPtr, .. }`. The corresponding type is `Type::NullPtr`.
- An array type with a non-literal size expression: The `Type::Array { size, .. }` node holds the complete expression tree, not only an integer constant. This expression is a plain `Expr` in the untyped tree and in the typed tree. It has no `resolved_type` (FR-022).
- Numeric literals with a non-decimal base (binary `#b...`, octal `#o...`, hexadecimal `#x...`): The AST holds them as `LiteralValue::Numeric(Number)`. This is the same variant as for decimal literals. The lexer resolves the base.
- A `for` loop with all three clauses absent: `initializer`, `condition`, and `increment` are each `None`, independent of each other. The body is still present.
- An expression statement with no trailing semicolon: The `Stmt::Expression` node wraps the expression. The lexer and the parser handle the missing semicolon. The AST does not. The `Stmt::Expression` variant has its own `span` field. The span covers the full statement, with the trailing semicolon if there is one. All other `Stmt` variants do the same.
- `vector<T>` and a user-defined type named `vector`: The parser makes `Type::Vector` when `vector` is followed by `<T>` in a type position. In all other cases, the identifier resolves to `Type::Custom`.
- `1 + 2` in the typed tree: The tree has a `TypedExpr` with `kind` equal to `TypedExprKind::Binary`. This node owns two `TypedExpr` operands with `kind` equal to `TypedExprKind::Literal`. Each of the three nodes has its own `resolved_type`.
- A grouping `(a + b)` in the typed tree: The tree has a `TypedExpr` with `kind` equal to `TypedExprKind::Grouping`. This node owns the `TypedExpr` for `a + b`. The grouping node and the inner node have independent `resolved_type` values.
- An `else if` chain in the typed tree: A `TypedElseBranch::ElseIf` holds a `TypedStmt::If`. That `TypedStmt::If` holds its own `TypedElseBranch`. The structure is the same as in `ElseBranch`.
- An `else` that is followed by a statement that is not an `if` and not a block: The parser reports the error E1004 (FR-007). The `else_branch` field is `ElseBranch::None`. The AST does not store the statement.
- An `ElseBranch` that code makes by hand with a payload of another kind, for example an `ElseIf` with a `Stmt::Break`: The type of the payload is `Box<Stmt>`, so the AST type permits this value. The parser does not make it. Code that reads the value accepts the payload as a `Stmt` (FR-007). The build functions and the `erase` functions keep the payload kind (FR-021).

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The AST must have a separate node variant for each of these expression kinds: binary operation, unary operation, grouping (parenthesized expression), literal, array literal, variable reference, assignment, function or method call, and array index access. A literal is numeric, string, character, boolean, or nullptr. The assignment variant covers only the simple assignment operator `=`. Compound-assignment operators, for example `+=` and `-=`, are `Expr::Binary` nodes with the applicable `BinaryOp` compound-assignment variant.
- **FR-002**: The AST must have a separate node variant for each of these statement kinds: expression statement, variable declaration with a mutability flag, function declaration, main function declaration, if/else-if/else, while loop, three-clause for loop, block, return, break, and continue.
- **FR-003**: The AST must have a type model. The type model includes these types:
    - all primitive scalar types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, and `bool`);
    - the fixed-size array type;
    - the dynamic vector type;
    - `void` and `nullptr`;
    - user-defined custom named types.
- **FR-004**: Each expression node and each statement node must have a `SourceSpan` value.
    - A node with a direct source text origin must have a span that maps the node to the exact source range of the node.
    - A synthesized node must have a zero-width `SourceSpan`. A synthesized node is a node that the parser makes for an implicit default, with no source token.
    - The span of a synthesized node must be at the position of the last consumed token before the synthesis point.
    - The `Stmt::Expression` variant must have its own explicit `span: SourceSpan` field.
    - This span must cover the full statement, with the trailing terminator if there is one.
    - The `Stmt::Expression` variant must not use only the span of the inner `Expr`.
- **FR-005**: The `Stmt::VarDeclaration` node must contain an ordered `bindings` collection, one shared `type_annotation`, and an `is_mutable` flag. The `is_mutable` flag shows if the declaration uses `var` or `const`.
- **FR-006**: The `Stmt::Function` node must contain the function name, an ordered list of `Parameter` nodes, a declared return type, and the body as a `Stmt::Block`. Each `Parameter` node must contain a name and a type annotation. If the source has no return type, the node must use `Type::Void`.
- **FR-007**: The `ElseBranch` model must show three cases: no else clause (`None`), a plain else block (`Block`), and an else-if continuation (`ElseIf`). The model must show chained conditionals explicitly.
    - The payload of `Block` and the payload of `ElseIf` have the type `Box<Stmt>`.
    - The payload of `ElseIf` must be a `Stmt::If` node. The payload of `Block` must be a `Stmt::Block` node.
    - After `else`, the parser must make `ElseIf` if the next statement is an `if`. It must make `Block` if the next statement is a block.
    - If the next statement is not an `if` and not a block, the parser must report the error E1004 (invalid else branch). The parser must then make `ElseBranch::None`. It must not store the statement.
    - The type `ElseBranch` does not prevent a payload of another kind. Existing tests make `ElseIf` and `Block` values with a `Stmt::Break` payload. A new payload type stops the compilation of these tests (SC-006). For this reason, the payload type stays `Box<Stmt>`.
    - Code that reads an `ElseBranch` must accept any `Stmt` as the payload of `Block` or `ElseIf`. The code must not fail if the payload has another kind.
- **FR-008**: The AST must represent binary operators as a closed enumeration. The enumeration covers all arithmetic, comparison, logical, bitwise, shift, and compound-assignment operators in the grammar. Each operator maps to exactly one variant.
- **FR-009**: The AST must represent unary operators as a closed enumeration. The enumeration must contain negation (`-`), logical NOT (`!`), bitwise complement (`~`), increment (`++`), and decrement (`--`). Each operator must have a `UnaryOpSide` value. The `UnaryOpSide` value is `Prefix` or `Postfix`. It shows the side of the operand where the operator is applied.
- **FR-010**: The `Type::Array` variant must contain the element type and the size. The size is an expression node, not a raw integer. This lets the AST represent array types with a size that is a compile-time constant expression. The untyped tree and the typed tree both store the size expression as a plain `Expr` (FR-022).
- **FR-011**: The `Type::Vector` variant must contain its element type. Its syntax must be different from a `Type::Custom` named `vector`.
- **FR-012**: Each `Expr` node must have a `span()` accessor that returns a reference to its `SourceSpan`.
- **FR-013**: Each `Stmt` node must have a `span()` accessor that returns a reference to its `SourceSpan`.
- **FR-014**: The AST must have a typed tree. Each expression node in the typed tree has its own resolved-type slot, at each depth.
    - The typed tree is in the `syntax::typed_ast` module. It has `TypedExpr` and `TypedStmt` (FR-019, FR-020).
    - Semantic analysis passes must store and read resolved types only through the `resolved_type` field of `TypedExpr`.
    - Semantic analysis passes must not use external side tables. A side table is a collection outside the typed tree that is addressed by node identifier, node address, or source span.
    - The core `Expr` enum must not change.
    - The `Expr` enum must stay usable without a resolved type. Parsing and other pre-semantic phases need this.
    - The annotation applies to each expression separately. Each `TypedExpr` node has an independent `resolved_type` value.
    - A write to `resolved_type` on one node must not change any other node.
    - `resolved_type` must be `Some(T)` when the type of that specific expression can be determined. This is true also when other parts of the program have errors.
    - `resolved_type` must be `None` when the type of that specific expression cannot be resolved.
    - `resolved_type` must be `None` on each node of a tree that no type-checking pass has processed.
- **FR-015**: The `Stmt::For` node must store the initializer, the condition, and the increment as independent optional fields. Each field is absent (`None`) or present. The state of one field must not change the state of another field.
- **FR-016**: Numeric literals with a non-decimal base (binary `#b`, octal `#o`, hexadecimal `#x`) must use the same `LiteralValue::Numeric(Number)` variant as decimal literals. The lexer must resolve the base.
- **FR-017**: The AST must represent a multi-binding variable declaration (`var a, b: T = e1, e2`) as one `Stmt::VarDeclaration` with several `VarBinding` entries. Each `VarBinding` must pair one name with an optional initializer. The type of the initializer is `Option<Expr>`.
    - The grammar requires one initializer for each name. In a syntactically valid program, the initializer of each `VarBinding` is `Some`.
    - The grammar has no declaration without `=`. This rule is the same for `var` and for `const`. A declaration with no initializer has zero initializers, and the next rule applies.
    - If the declaration has at least one name and the number of names is not equal to the number of initializers, the parser must report the error E2001 (initializer count mismatch).
    - After this error, the parser must still make one `Stmt::VarDeclaration`. It must make one `VarBinding` for each name, in source order.
    - The n-th name takes the n-th initializer. A name that has no n-th initializer has `None`.
    - If there are more initializers than names, the AST does not store the extra initializers.
    - If the declaration has no name, the parser must report the error E1008 (missing variable name). The parser must not make a `Stmt::VarDeclaration`. The rules above do not apply.
    - Code that reads the AST must accept `None` as the initializer of a `VarBinding`.
- **FR-018**: The AST model must be internally consistent. Two structurally different constructs must not use the same node variant. Each node variant must stand for one semantic concept only.
- **FR-019**: `TypedExpr` must be a struct with exactly three public fields:
    - `kind`, of type `TypedExprKind`;
    - `span`, of type `SourceSpan`;
    - `resolved_type`, of type `Option<Type>`.

  `TypedExpr` must not contain an `Expr`.

  `TypedExprKind` must be an enum. It has one variant for each `Expr` variant (FR-001). Each variant has the same name as the `Expr` variant.

  The fields of an `Expr` variant carry over to the `TypedExprKind` variant with these rules:
    - A field of type `Expr` becomes `TypedExpr`. The same change applies inside `Box`, `Vec`, and `Option`.
    - The `SourceSpan` of the variant is stored once, in `TypedExpr::span`. The variant does not repeat it.
    - Each other field keeps its type. These fields are operators, `LiteralValue`, identifier names, and `Type`.

  A `TypedExpr` owns its child `TypedExpr` nodes. To read the resolved type of a sub-expression, go from the parent node to the child node. The typed tree has no identifier, index, or lookup structure.
- **FR-020**: `TypedStmt` must be an enum. It has one variant for each `Stmt` variant (FR-002). Each variant has the same name as the `Stmt` variant and has a `SourceSpan` (FR-004).

  The fields of a `Stmt` variant carry over to the `TypedStmt` variant with these rules. Each rule also applies inside `Box`, `Vec`, and `Option`.
    - A field of type `Expr` becomes `TypedExpr`.
    - A field of type `Stmt` becomes `TypedStmt`.
    - A field of type `ElseBranch` becomes `TypedElseBranch`.
    - A field of type `VarBinding` becomes `TypedVarBinding`.
    - Each other field keeps its type. These fields are `is_mutable`, names, `Parameter`, `Type`, and `SourceSpan`.

  `TypedElseBranch` must have the same three cases as `ElseBranch` (FR-007). Its `Block` payload and its `ElseIf` payload have the type `Box<TypedStmt>`. For parser output, the `ElseIf` payload is a `TypedStmt::If` and the `Block` payload is a `TypedStmt::Block`.

  `TypedVarBinding` must pair the name with an `Option<TypedExpr>` initializer, in the same way as `VarBinding`.

  `TypedStmt` has no `resolved_type` of its own.
- **FR-021**: The build functions `TypedExpr::from_expr(&Expr) -> TypedExpr` and `TypedStmt::from_stmt(&Stmt) -> TypedStmt` must build the typed tree from the untyped tree.
    - A build function must make a tree with the same structure as its input. It keeps the variants, their order, the values of copied fields, and the spans.
    - A build function must set `resolved_type` to `None` on each `TypedExpr`.
    - A build function must not change its input.
    - A build function must be an exhaustive `match` over the `Expr` variants or the `Stmt` variants. It must not have a wildcard arm. When a developer adds a variant to `Expr` or `Stmt`, the compilation of `descar-core` fails until the typed tree handles the new variant.

  The functions `TypedExpr::erase(&self) -> Expr` and `TypedStmt::erase(&self) -> Stmt` must rebuild the untyped tree. They do not use any `resolved_type`.
    - For each `Expr` value `e`, `TypedExpr::from_expr(&e).erase()` must be structurally equal to `e`, node by node, with spans.
    - The same rule applies to `Stmt`.
    - The build functions and the `erase` functions must each process each node once.
    - The build functions and the `erase` functions must keep the kind of each `ElseBranch` payload. They must not check it and they must not change it (FR-007).
- **FR-022**: `TypedExpr` and `TypedStmt` must satisfy FR-004, FR-012, and FR-013.
    - `TypedExpr` and each `TypedStmt` variant must have a `span()` accessor.
    - The span of a typed node must be equal to the span of the untyped node that it comes from.

  The type annotation positions in the typed tree are variable declarations, `Parameter`, and return types.
    - Each such position must hold the same `Type` value as the untyped tree.
    - The two trees share `Type`. `Type` has no typed counterpart.

  The `size` expression inside `Type::Array` (FR-010) is a plain `Expr` in both trees.
    - It has no `resolved_type`. It is in a type position and it is not an expression node of the typed tree.
    - A type-checking pass can build a `TypedExpr` from the size expression with `TypedExpr::from_expr` for the time of the check. The pass must not store the result.
- **FR-023**: `TypedExpr`, `TypedExprKind`, `TypedStmt`, `TypedElseBranch`, and `TypedVarBinding` must be in the `syntax::typed_ast` sub-module of `descar-core`. The untyped AST must stay in `syntax::ast`. The module `syntax::typed_ast` can import from `syntax::ast`. The module `syntax::ast` must not import from `syntax::typed_ast`. The two modules must stay separate.

### Key Entities

- **`Expr`**: The sum type of all expression node variants. Each variant has a `SourceSpan`. The type has a `span()` accessor.
- **`Stmt`**: The sum type of all statement node variants. Each variant has a `SourceSpan`. No variant is an exception. This satisfies FR-004 and the clarification that `Stmt::Expression` has its own explicit `span` field.
- **`Type`**: The sum type of all type annotation variants. The variants are primitives, array, vector, void, nullptr, and custom. The typed tree and the untyped tree share this type.
- **`BinaryOp`**: A closed enumeration of all binary operators.
- **`UnaryOp`**: A closed enumeration of all unary operators.
- **`UnaryOpSide`**: An enumeration with two variants. It shows prefix or postfix application.
- **`LiteralValue`**: The sum type of all literal value kinds. The kinds are numeric, string, char, bool, and nullptr.
- **`Parameter`**: A named tuple with `name`, `type_annotation`, and `span` for function parameters. The typed tree and the untyped tree share this type.
- **`VarBinding`**: A named tuple with `name` and an `Option<Expr>` initializer, for multi-binding declarations. In a syntactically valid program, the initializer is `Some` (FR-017).
- **`ElseBranch`**: An enumeration with three variants, for the else clause of an if statement. The `Block` payload and the `ElseIf` payload have the type `Box<Stmt>`. In parser output, the `ElseIf` payload is a `Stmt::If` and the `Block` payload is a `Stmt::Block` (FR-007).
- **`TypedExpr`**: A struct in `syntax::typed_ast`. It has the fields `kind: TypedExprKind`, `span: SourceSpan`, and `resolved_type: Option<Type>`. It does not contain an `Expr`. Its children are `TypedExpr` nodes. The type-checking pass sets `resolved_type`. `None` means that the type of the specific expression cannot be resolved, or that no type-checking pass has processed the tree.
- **`TypedExprKind`**: An enum in `syntax::typed_ast`. It has one variant for each `Expr` variant. Fields of type `Expr` become `TypedExpr` (FR-019).
- **`TypedStmt`**: An enum in `syntax::typed_ast`. It has one variant for each `Stmt` variant. Fields of type `Expr`, `Stmt`, `ElseBranch`, and `VarBinding` become their typed counterparts (FR-020). It has no `resolved_type` of its own.
- **`TypedElseBranch`**: An enumeration with three variants, in the same way as `ElseBranch`. Its payloads have the type `Box<TypedStmt>` (FR-020).
- **`TypedVarBinding`**: A named tuple with `name` and an optional `TypedExpr` initializer. It corresponds to `VarBinding`.

---

## Clarifications

### Session 2026-10-04

- Q: How does the resolved-type annotation attach to each `Expr` node? The options are an optional field in each expression variant, or a separate wrapper type that holds the `Expr` and its resolved type. A: A wrapper type. A `TypedExpr` struct pairs the `Expr` with an `Option<Type>`. The core `Expr` enum does not change. **Later entries in this session replace the pairing.** The core `Expr` enum still does not change.
- Q: Does the `Stmt::Expression` variant have its own `SourceSpan` field, or does it use the span of the inner `Expr`? This matters for the round-trip span requirement. A: The variant has its own `span: SourceSpan` field. The span covers the full statement, with any terminator.
- Q: A program has type errors in some places. For another expression in the same program, is `TypedExpr::resolved_type` `None`, or is it `Some(...)` with a best-effort type? A: Each expression has its own annotation. The value is `Some(T)` if the type of that specific expression can be determined. The value is `None` only if the type of that specific expression cannot be resolved. An error is local. It does not stop the annotation of other nodes.
- Q: For SC-006, which changes to the `Expr` or `Stmt` enums are breaking? A breaking change needs documentation and a migration path. A: These changes are breaking: renaming a field or variant, removing a field or variant, and changing the type of a field or variant. These changes are not breaking: adding a new optional field and adding a new variant (marked `#[non_exhaustive]`). A `MIGRATION.md` note must describe each breaking change.
- Q: Where are `TypedExpr` and `TypedStmt`? A: They are in a new `syntax::typed_ast` sub-module of `descar-core`. The untyped AST stays in `syntax::ast`. The two modules stay separate.
- Q: A `TypedExpr` with one `Expr` and one `Option<Type>` cannot give the operands of `Expr::Binary` their own resolved type. Four conditions cannot all be true: (1) `TypedExpr` has two members, (2) `Expr` does not change, (3) each sub-expression has an independent `resolved_type`, and (4) side tables are not allowed. Which condition does the spec remove? A: Condition 1. `TypedExpr` and `TypedStmt` are typed trees with the same structure as the untyped trees. The build functions make them from the untyped tree. Each node owns its typed children. `Expr` does not change, no side table exists, and each expression node has its own `resolved_type`.
- Q: Which alternatives does the spec reject? A: There are four. (1) A two-member wrapper that annotates only the root expression. It does not give a type to the operands. (2) An `Expr` that is generic over an annotation parameter. It changes `Expr`. (3) A table that uses node identity, node address, or source span as key. FR-014 does not permit it. (4) A wrapper that contains an `Expr` and also holds typed children. It keeps two trees of the same program, and no rule keeps them consistent.
- Q: How does the spec keep the typed tree and the untyped tree consistent? A: The typed tree does not contain the untyped tree. The build functions make it from the untyped tree (FR-021). The `erase()` functions rebuild the untyped tree. For each `e`, `from_expr(&e).erase()` is structurally equal to `e` (SC-007). The build functions use an exhaustive `match` with no wildcard arm. A new `Expr` or `Stmt` variant stops the compilation until the typed tree handles it.
- Q: Does the `size` expression inside `Type::Array` get a resolved type? A: No. It is in a type position. Both trees share `Type`. The expression stays a plain `Expr` (FR-022). A checking pass that needs its type builds a `TypedExpr` for the time of the check and does not store the result.

### Session 2026-10-05

- Q: FR-017 said that each `VarBinding` has an initializer. Key Entities said that the initializer is optional. Which statement is correct? A: The two statements apply to different cases. The type of the initializer is `Option<Expr>`. The initializer is `Some` in a syntactically valid program, because the grammar requires one initializer for each name. The initializer is `None` only after the parser recovers from an initializer-count error (E2001), or when code makes the AST by hand.
- Q: Why does the spec keep `Option<Expr>` and not `Expr`? A: The existing code and the existing tests use a `VarBinding` with `None` as the initializer. A change to `Expr` is a breaking change under SC-006, and the existing tests would not compile.
- Q: What does the parser do when the number of names is not equal to the number of initializers? A: The parser reports the error E2001. It makes one `VarBinding` for each name. The n-th name takes the n-th initializer. A name that has no n-th initializer has `None`. The parser does not store the extra initializers.
- Q: Does the grammar permit a `var` or a `const` declaration with no initializer? A: No. The parser requires `=` and one expression for each name. A declaration with no initializer has zero initializers, and the rule for E2001 applies (FR-017).
- Q: What does the parser do when a declaration has no name? A: The parser reports the error E1008 and does not make a `Stmt::VarDeclaration`. The parser stops before it reads the initializers, so the count rule of FR-017 does not apply.
- Q: What is the type of the `ElseBranch::ElseIf` payload? The spec said that the payload contains a `Stmt::If`, but it did not name the type. A: The type is `Box<Stmt>`, the same as in the existing code. The rule that the payload is a `Stmt::If` applies to parser output (FR-007).
- Q: Why does the spec not use a payload type that can only hold an `if`? A: Existing tests make `ElseIf` and `Block` values with a `Stmt::Break` payload. A new payload type stops the compilation of these tests (SC-006). The spec keeps `Box<Stmt>` and puts the rule on the parser and on the code that reads the value.
- Q: What does the parser do when the statement after `else` is not an `if` and not a block? A: The parser reports the error E1004 and makes `ElseBranch::None`. It does not store the statement. The parser already does this.
- Q: What must code that reads an `ElseBranch` do with a payload of another kind? A: It must accept any `Stmt`. The type checker and the printer already do this. The build functions and the `erase` functions of the typed tree keep the payload kind and do not check it (FR-021).

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For each syntactically valid `.dr` source file, the parsed AST has no unrecognized node and no generic node for a construct in the grammar. The parsed AST includes 100% of the grammar productions that matter for later compiler phases.
- **SC-002**: Each type annotation position in the AST uses a concrete `Type` variant. The positions are variable declarations, parameters, return types, and array sizes. A construct with a specific type keyword does not use `Type::Void` or `Type::Custom` as a default.
- **SC-003**: Each example `.dr` file in `dr_files/` parses to an AST that passes the structural completeness check. Each node variant, span, and child field that the grammar needs is present and has a value.
- **SC-004**: A round-trip property applies to nodes with a direct text origin. Such a node corresponds to one or more source tokens. For each such node:
    - `span()` must return a source range.
    - The text of the original source in that range must be exactly the source fragment that makes the node.

  Synthesized nodes are not subject to the verbatim-reproduction rule. A synthesized node is a node that the parser makes for an implicit default, with no source token. An example is the `Type::Void` return type of a function with no return annotation.
    - A synthesized node must have a zero-width `SourceSpan`.
    - The `SourceSpan` must be at the position of the last consumed source token before the synthesis point. For example, an implicit `Type::Void` has a zero-width `SourceSpan` at the closing `)` of the parameter list.
    - This span is valid for position-based diagnostics. It does not reproduce a source fragment.
- **SC-005**: Type-checker tests verify the resolved-type annotation on `TypedExpr` trees at each depth. After the type checker runs on a program, each resolvable `TypedExpr` node has `Some(T)`. Resolvable nodes are literals, variables, calls, binary expressions, and unary expressions. This includes operands inside other expressions. It also includes expressions in statement conditions, initializers, and return values. The existing type-checker tests pass with no regression.
- **SC-006**: All existing tests in `crates/descar-core/tests/` pass without change after the AST changes. A breaking change is one of these: renaming, removing, or changing the type of an existing public `Expr` or `Stmt` field or variant. Adding a new optional field or a new enum variant, with `#[non_exhaustive]` where necessary, is not a breaking change. It needs no migration path. The feature directory must contain a `MIGRATION.md` note for each breaking change of this feature.
- **SC-007**: For each `.dr` file in `dr_files/`, build the typed tree from the parsed program.
    - The parsed program does not change.
    - `erase()` on the typed tree returns a tree that is structurally equal to the parsed program, with spans.
    - The number of `TypedExpr` nodes is equal to the number of `Expr` nodes.
    - The number of `TypedStmt` nodes is equal to the number of `Stmt` nodes.
- **SC-008**: For each `.dr` file in `dr_files/`, check each `ElseBranch` value in the parsed program and in the typed tree.
    - Each `ElseBranch::ElseIf` has a `Stmt::If` payload. Each `ElseBranch::Block` has a `Stmt::Block` payload.
    - Each `TypedElseBranch::ElseIf` has a `TypedStmt::If` payload. Each `TypedElseBranch::Block` has a `TypedStmt::Block` payload.

  A test makes a `Stmt::If` by hand. Its `ElseBranch::ElseIf` has a `Stmt::Break` payload.
    - `TypedStmt::from_stmt` and `erase()` keep the payload kind.
    - The result of `erase()` is structurally equal to the input, with spans.
- **SC-009**: For each `.dr` file in `dr_files/` that has no syntax error, check each `VarBinding` in the parsed program and each `TypedVarBinding` in the typed tree.
    - Each `VarBinding` has `Some` as its initializer.
    - Each `TypedVarBinding` has `Some` as its initializer.

  A test parses each of these sources.
    - `var a, b: i32 = 1;` gives the error E2001. The AST has `a` with `Some` and `b` with `None`.
    - `var a: i32 = 1, 2;` gives the error E2001. The AST has one `VarBinding` with `Some`. It does not store the initializer `2`.
    - `var a: i32;` gives the error E2001. The AST has one `VarBinding` with `None`.
    - `var : i32 = 1;` gives the error E1008. The AST has no `Stmt::VarDeclaration`.

---

## Assumptions

- `JsavParser` and the existing token set in `TokenKind` fully define the grammar of the Descar language. No grammar extension is in scope. The extensions are structs, enums, generics other than `vector<T>`, lambdas, traits, and modules.
- The existing `SourceSpan` and `SourceLocation` types are sufficient for all required source ranges. The feature needs no new location infrastructure.
- The `Number` type in `crates/descar-core/src/tokens/number.rs` is the canonical representation of all numeric literal values. The lexer and the `LiteralValue::Numeric` variant of the AST share this type.
- The `Type::Void` variant is the default return type for a function that does not declare one. The parser applies this default. The type checker does not. The synthesized `Type::Void` node has a zero-width `SourceSpan` at the last consumed token before the synthesis point. An example is the closing `)` of the parameter list. This agrees with the span policy for synthesized nodes in FR-004 and SC-004.
- The resolved-type annotation (FR-014) is a typed tree (`TypedExpr` and `TypedStmt`). The typed tree has the same structure as `Expr` and `Stmt`, node by node. The build functions make it (FR-021). The `Expr` enum does not change. All existing parsing and printing code that makes or matches `Expr` variants without type information continues to work.
- The children of `Expr` variants are `Expr` values, directly or inside `Box`, `Vec`, or `Option`. The children of `Stmt` variants are `Expr`, `Stmt`, `ElseBranch`, or `VarBinding` values in the same containers. FR-019 and FR-020 use these shapes. If the source code holds a child in a different shape, the same rule applies to that shape. In that case, the implementation plan must record the exact typed definition before the implementation starts.
- `LiteralValue`, `BinaryOp`, `UnaryOp`, `UnaryOpSide`, `Type`, `SourceSpan`, and identifier names implement `Clone`. The build functions and the `erase` functions need this. To add a missing `Clone` derive is not a breaking change under SC-006.
- After the build function runs and before type checking, each `resolved_type` is `None`. The model has no separate value for "not yet checked". To tell the two states apart, check if the type-checking pass has processed the tree.
- The `vector<T>` notation is the only generic type constructor in scope. The grammar has no other parameterized type syntax.
- All existing public APIs (parser output, printer input, and type-checker input) stay stable. The typed AST is additive and backward-compatible where possible. Where it is not, the migration is explicit and documented.
- The `specs/` directory did not exist before. This is the first feature specification in the project.
