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

Each AST node represents one syntactic construct. Each node stores the information that is required to represent that construct. The AST does not use a generic node for different syntactic constructs.

Binary operators are represented by a dedicated `BinaryOp` variant. Unary operators are represented by a dedicated `UnaryOp` variant. Operator precedence and associativity are preserved by the AST structure.

**Why this priority*: Without complete node coverage, the AST cannot represent every valid program in the grammar. This blocks type checking, semantic analysis, intermediate representation generation, and AST printing.

**Independent Test**: Parse the example file `large_toy_program.dr` or `sccp_test.dr`. The parser returns a `Vec<Stmt>`. Each statement and each sub-expression maps to a concrete, named node variant. No construct uses a generic catch-all variant. The AST preserves the source structure required by later compiler phases.

**Acceptance Scenarios**:

1. **Given** a `.dr` source file that contains all statement kinds, **When** the parser builds the AST, **Then** each statement maps to a distinct node variant with a unique name. Two different grammar constructs do not map to the same statement variant.
2. **Given** an expression that uses all binary operators (arithmetic, comparison, logical, bitwise, shift, and compound assignment), **When** the parser parses the expression, **Then** each operator maps to a separate `BinaryOp` variant inside an `Expr::Binary` node. The node stores the left operand and the right operand. The AST preserves the operator precedence and associativity defined by the grammar.
3. **Given** a function declaration with parameters and a typed return annotation, **When** the parser parses the declaration, **Then** the `Stmt::Function` node contains the function name, the complete ordered list of `Parameter` nodes, the declared return type, and the body block. Each `Parameter` node contains the parameter name and type. The order of the parameters is preserved. If the function name is `main` and the ordinary function declaration syntax is used, the node is still `Stmt::Function`.
4. **Given** a `for` loop with an initializer, a condition, and an increment clause, **When** the parser parses the loop, **Then** the `Stmt::For` node stores the initializer, the condition, and the increment clause in separate optional fields. The initializer is either a variable declaration or an expression statement. The condition and increment are expressions. The node also stores the loop body. The AST preserves the order of these clauses.
5. **Given** an expression with nested binary and unary operators, **When** the parser builds the AST, **Then** each operator is represented by the correct operator variant. The tree structure reflects the grammar precedence and associativity. Parentheses produce a grouping node.
6. **Given** an array access or array literal, **When** the parser builds the AST, **Then** the array access node stores the array expression and the index expression. The array literal node stores its elements in source order. Nested array expressions are represented by nested AST nodes.
7. **Given** a type reference for any type supported by the grammar, **When** the parser builds the AST, **Then** the type reference maps to a distinct type variant. Array, vector, void, nullptr, primitive, and custom types are represented explicitly. A custom type stores its type name.
8. **Given** a valid `.dr` program that contains nested statements and expressions, **When** the parser builds the AST, **Then** every child construct is represented by the AST node required by its grammar production. No valid construct is discarded, merged with an unrelated construct, or represented by a generic catch-all node.
9. **Given** the dedicated `main { ... }` syntax, **When** the parser builds the AST, **Then** the statement is represented by `Stmt::MainFunction`. The node contains the main body and its source span. It does not contain a function name, parameter list, or return type.
10. **Given** an ordinary function declaration whose name is `main`, **When** the parser builds the AST, **Then** the statement is represented by `Stmt::Function`. The function name is stored as `"main"`. It is not represented by `Stmt::MainFunction`.
11. **Given** a return statement with or without an expression, **When** the parser builds the AST, **Then** `Stmt::Return` contains an optional return value. `None` represents a return statement without a value.
12. **Given** a break or continue statement, **When** the parser builds the AST, **Then** the parser creates `Stmt::Break` or `Stmt::Continue`. Each node stores its source span.
13. **Given** a call expression, **When** the parser builds the AST, **Then** `Expr::Call` contains the callee expression and an ordered list of argument expressions. The callee is not restricted to an identifier.
14. **Given** an assignment expression, **When** the parser builds the AST, **Then** `Expr::Assign` contains a target expression and a value expression. A valid target is an `Expr::Variable` or an `Expr::ArrayAccess`.
15. **Given** a parenthesized expression, **When** the parser builds the AST, **Then** `Expr::Grouping` contains the inner expression and its complete source span.

---

### User Story 2 - Uniform and Complete Type Model (Priority: P2)

A type checker or a semantic analysis pass reads a declared type from the AST. The type checker does not infer the declared type. The type checker does not parse the source tokens again. Each grammar position that permits a type annotation has one first-class `Type` value. The `Type` model represents every type that the grammar can declare. The model represents primitive types, array types, vector types, custom types, `void`, and `nullptr`. The AST stores the type structure. The AST does not store only the source spelling of a type. The semantic pass can inspect the `Type` value without reading the source text again.

**Why this priority**: Downstream semantic phases need one complete type model. A type checker must use the same representation for variables, parameters, return types, and other type annotations. If the AST stores incomplete or different type information, later phases must parse source text again or infer missing information. This creates duplicated parsing logic and can create different results for the same type.

**Independent Test**: Use a `.dr` source file with variables of all primitive types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, `bool`). The file also has arrays (`i32[10]`), vectors (`vector<f64>`), custom types, `void`, and `nullptr`. Use the types in variable declarations, function parameters, function return types, and type annotations. Examine the AST without reading the source tokens. Each type annotation position contains one `Type` value with the correct variant and its required data.

**Acceptance Scenarios**:

1. **Given** the variable declaration `var x: i32 = 0`, **When** you examine the AST node, **Then** the `type_annotation` field contains `Type::I32` and the `is_mutable` flag is `true`.
2. **Given** the constant declaration `const y: bool = true`, **When** you examine the AST node, **Then** the `type_annotation` field contains `Type::Bool` and the `is_mutable` flag is `false`.
3. **Given** the array type annotation `i32[10]`, **When** you examine the AST node, **Then** the type is `Type::Array`, the `element_type` is `Type::I32`, and the `size` field contains an expression node for the literal `10`.
4. **Given** the vector type annotation `vector<f64>`, **When** you examine the AST node, **Then** the type is `Type::Vector` and the `element_type` is `Type::F64`.
5. **Given** a custom type annotation `MyType`, **When** you examine the AST node, **Then** the type is `Type::Custom` and the custom type name is stored in the `Type` value.
6. **Given** the function parameter `n: f64`, **When** you examine the AST node, **Then** the `Parameter` node contains `name = "n"` and `type_annotation = Type::F64`.
7. **Given** a function with the return type annotation `: i32`, **When** you examine the AST node, **Then** the `return_type` field contains `Type::I32`.
8. **Given** a function with no return type annotation, **When** the parser parses the function, **Then** the `return_type` field contains `Type::Void`.
9. **Given** a type annotation for each primitive type (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, `bool`), **When** you examine the AST node, **Then** each annotation contains its matching `Type` variant.
10. **Given** the type annotations `void` and `nullptr`, **When** you examine the AST node, **Then** `void` is represented by `Type::Void` and `nullptr` is represented by `Type::Nullptr`.
11. **Given** any grammar position that permits a type annotation, **When** the parser builds the AST, **Then** the position contains a first-class `Type` value and does not contain only raw type tokens or a source string.
12. **Given** a declared type in the source, **When** a semantic analysis pass reads the AST, **Then** the pass can identify the complete declared type from the `Type` value without reparsing the source tokens.

---

### User Story 3 - Structural Integrity and Source Provenance (Priority: P2)

Each `Expr` node, each `Stmt` node, and each `Parameter` has a `SourceSpan`. The `SourceSpan` identifies the exact range of source text that produced the node. A node with a direct source origin has a `SourceSpan`. The source origin can be one source token or a continuous range of source tokens. The text identified by the `SourceSpan` is the exact source fragment that produced the node. The parser preserves the syntactic structure of the source program in the AST. Each syntactic alternative has an explicit AST variant. The AST does not use a nullable field to represent a distinct syntactic construct. The `else` part of an `if` statement is represented by the `ElseBranch` enum. It is not represented by a raw `Option<Box<Stmt>>`. An `ElseBranch::ElseIf(...)` contains the nested `Stmt::If` node produced by the `else if` syntax. An `ElseBranch::Block(...)` contains the block produced by the `else` syntax. `ElseBranch::None` represents the absence of an `else` part. `Type`, `VarBinding`, and `ElseBranch` do not have a `SourceSpan` of their own. Their source information is represented by the nodes or tokens that contain them. The parser creates `Type::Void` when a function has no declared return type. This `Type::Void` is an inferred AST value. It has no source token and no `SourceSpan`. Every `Expr` and every `Stmt` has a `span()` method. The method returns the `SourceSpan` of the complete syntactic construct represented by the node. For a compound node, the span covers the complete source range represented by the node, including all child syntax that belongs to the construct. The child nodes keep their own spans. The order of children in a collection follows the source order. No parser rule may require a hidden rule to locate a child node. If parsing fails for a syntactic construct, the parser reports the required error and does not store source statements that the parser rejected as valid AST children.

**Why this priority**: Precise source positions are required for diagnostics, source highlighting, AST inspection, and later compiler phases. An AST must preserve the syntactic structure of the program. A nullable or implicit representation can lose information about which grammar alternative produced a construct.

**Independent Test**: Use any valid source file. For each `Expr` node and each `Stmt` node, call `span()`. The returned `SourceSpan` identifies a valid source range. Extract the source text for each range. The extracted text corresponds to the syntactic construct represented by the node. Check compound nodes and their children. Each child has its own valid span and appears in source order.

**Acceptance Scenarios**:

1. **Given** a statement at line 5, columns 3 to 20 in the source, **When** you call `span()` on the corresponding AST node, **Then** the returned `SourceSpan` identifies line 5, columns 3 to 20 and maps to the source fragment for that statement.
2. **Given** an `if` statement with an `else if` continuation, **When** you inspect the AST, **Then** the `else_branch` field is `ElseBranch::ElseIf(...)`. The payload of `ElseIf` is the nested `Stmt::If` node. The field is not `ElseBranch::Block(...)` and it is not `ElseBranch::None`.
3. **Given** an `if` statement with an `else` block, **When** you inspect the AST, **Then** the `else_branch` field is `ElseBranch::Block(...)`. The payload contains the block represented by the `else` part. The field is not `ElseBranch::ElseIf(...)` and it is not `ElseBranch::None`.
4. **Given** an `if` statement without an `else` part, **When** you inspect the AST, **Then** the `else_branch` field is `ElseBranch::None`.
5. **Given** a block with several statements, **When** you read the `Stmt::Block` node, **Then** each child statement is available by index in the `statements` vector and the vector preserves source order.
6. **Given** a block containing statements at different source positions, **When** you call `span()` on the block and on each child statement, **Then** the block span covers the complete block and each child span covers its own syntactic construct. The child spans are ordered according to the source order.
7. **Given** a function with no declared return type, **When** the parser builds the function AST node, **Then** its return type is `Type::Void`. `Type::Void` has no source token and no `SourceSpan`.
8. **Given** a function with an explicit return type, **When** the parser builds the function AST node, **Then** the declared type is represented by the corresponding `Type` value. The source span of the function and the source spans of its expression and statement nodes identify their own source ranges. The `Type` value itself does not require a `SourceSpan`.
9. **Given** an expression produced from a source token or from a continuous range of source tokens, **When** you call `span()` on the expression, **Then** the returned `SourceSpan` identifies the exact source range that produced the expression.
10. **Given** a compound expression containing child expressions, **When** you inspect the AST and the spans, **Then** the compound expression has a span for the complete syntactic construct and each child expression has its own span for its own syntactic construct.
11. **Given** a parameter declared in a function, **When** you inspect the parameter, **Then** the parameter has a `SourceSpan` that identifies the source range of the parameter declaration.
12. **Given** an `if` statement with the source `if (x > 0) { return x } else {return 0 }`, **When** the parser processes the statement, **Then** the parser reports error `E1004`. The `else_branch` field is `ElseBranch::None`. The AST does not store the statement `return 0` as a valid child statement of the `if` node.
13. **Given** the parser reports `E1004` for an invalid `else` construct, **When** the AST is inspected after error recovery, **Then** no rejected statement from the invalid `else` construct is represented as a valid AST node. The parser does not silently convert rejected syntax into a valid `ElseBranch::Block(...)`.
14. **Given** any `Expr` or `Stmt` node returned by the parser without a parser error for that node, **When** its `span()` method is called, **Then** the method returns a valid `SourceSpan` that refers to the original source text. No valid `Expr` or `Stmt` node has a missing source span.
15. **Given** an AST node with several child nodes, **When** the children are traversed, **Then** every syntactic child defined by the grammar is reachable through an explicit AST field or collection. No child may depend on an implicit parser rule, a positional convention that is not defined by the AST, or a hidden nullable state.

---

### User Story 4 - Typed AST Readiness for Downstream Phases (Priority: P3)

A semantic analysis pass or an IR generator builds a typed tree from the parsed AST. The pass decorates each expression node with its resolved type. The `Expr` enum does not change. The typed tree has the same structure as the untyped tree. `TypedExpr` and `TypedStmt` correspond to `Expr` and `Stmt` node by node. Each `TypedExpr` owns its child `TypedExpr` nodes. To read the type of a sub-expression, go from its parent node to the child node. Parsing phases use plain `Expr` and `Stmt` trees. Semantic phases use `TypedExpr` and `TypedStmt` trees.

**Why this priority**: Downstream passes, such as type checking, IR generation, and optimization, need the type of each expression node. A single `Option<Type>` next to an unchanged `Expr` cannot store the types of all sub-expressions in that `Expr`. The typed tree therefore decorates each expression node with its own type slot and preserves the AST structure. This follows the compiler design principle that semantic analysis adds information to the syntax tree without changing the source structure needed by later phases. `Expr` stays unchanged. Parser-side code does not handle a type field before semantic analysis starts.

**Independent Test**: Build the typed tree from a fully parsed program. Run the type-checking pass on the typed tree. After the pass, each `TypedExpr` node at each depth that stands for a typed construct has its resolved type in `TypedExpr::resolved_type`. A typed construct is a literal, a variable, a call, a binary expression, or a unary expression. This includes operands inside other expressions. It also includes conditions, initializers, and return values inside statements. The type checker visits child expressions before or while it computes the type of the parent expression. The caller does not derive a type again.

**Acceptance Scenarios**:

1. **Given** the expression `1 + 2`, **When** the type checker has processed the `TypedExpr` tree, **Then** the root node has `kind` equal to `TypedExprKind::Binary` and `resolved_type` equal to `Some(Type::I32)` (or the promoted numeric type). The left operand and the right operand are `TypedExpr` nodes with `kind` equal to `TypedExprKind::Literal`. Each operand has `resolved_type` equal to `Some(Type::I32)`. The type of the binary expression is computed from the types of its operand nodes.
2. **Given** a `TypedExpr` with `kind` equal to `TypedExprKind::Variable` for a declared `var x: f64`, **When** the type checker completes, **Then** `resolved_type` is `Some(Type::F64)`. This is true if the node is a complete statement expression. It is also true if the node is an operand of `x * 2.0`. The type annotation is stored on the variable expression node and does not depend on the parent expression.
3. **Given** an expression node with an unknown type, for example a node that uses an undeclared variable, **When** the type checker completes, **Then** `resolved_type` is `None` for that node. Other expressions with a known type keep `Some(T)`. An error in one expression does not stop the annotation of other expressions. In `undeclared + 1`, the literal `1` has `Some(Type::I32)` and the undeclared variable has `None`. The parent expression may also have `None` when its type cannot be determined from its operands. The type checker continues to visit the remaining child and sibling nodes.
4. **Given** the code `if (x > 0) { return x + 1 } else if (x < 0) { return x - 1 }` and `var x: i32` in scope, **When** the type checker completes, **Then** the condition of the `TypedStmt::If` node has `Some(Type::Bool)`. Each operand of the condition has `Some(Type::I32)`. The returned expression and its operands have `Some(Type::I32)`. The `TypedElseBranch::ElseIf` payload holds a `TypedStmt::If`. The condition and the operands of that `TypedStmt::If` have annotations in the same way. Each nested statement and expression remains at the same structural position as in the untyped tree.
5. **Given** a parsed `Expr` tree `e`, **When** the pass builds the typed tree with `TypedExpr::from_expr(&e)`, **Then** `e` does not change. Each `resolved_type` in the result is `None`. The call `erase()` on the result returns a tree that is structurally equal to `e`. `TypedExpr::from_expr(&e)` performs no type inference and does not report a semantic type error. It creates the typed tree structure and leaves semantic type information unset until the type-checking pass runs.
6. **Given** the declaration `var a: i32[2 + 3]`, **When** the pass builds the typed tree, **Then** the type annotation in the `TypedStmt::VarDeclaration` node is the same `Type::Array` value as in the untyped tree. Its `size` field is a plain `Expr` with no resolved type. Expressions that are part of a type annotation are not converted into `TypedExpr` nodes by `TypedStmt::from_stmt` and do not receive a `resolved_type` field. This preserves the distinction between expressions that are part of the program expression tree and expressions stored inside type syntax.

---

### Edge Cases

- Multiple bindings (`var a, b: i32 = 1, 2`): Each `VarBinding` is a separate entry. Each entry has one initializer. In a valid program, the initializer of every binding is `Some` (FR-017). The `Option<Expr>` field is an AST representation choice. It does not permit a declaration without an initializer. All bindings use the same declared type annotation. During semantic analysis, the declared type applies to each binding. The typed tree contains one `TypedVarBinding` for each binding in the `TypedStmt::VarDeclaration` node. Each valid initializer is a separate `TypedExpr`. Each `TypedExpr` contains its own `resolved_type`.
- A declaration has a different number of names and initializers (`var a, b: i32 = 1`): The parser reports error E2001 (FR-017). The AST contains one `VarBinding` for each name. The `VarBinding` for `a` has `Some(1)`. The `VarBinding` for `b` has `None`. The corresponding `TypedVarBinding` for `b` also has `None`. This `None` value represents parser recovery from the invalid declaration. It does not represent a valid uninitialized variable. In `var a: i32 = 1, 2`, the AST contains one `VarBinding` with `Some(1)`. The initializer `2` is parsed but is not stored.
- A declaration has no initializer (`var a: i32;` or `const c: i32;`): The grammar does not define this as a valid declaration. A declaration without `=` has zero initializers. The parser reports the errors required for the invalid syntax and applies the initializer-count rule in FR-017. The AST can contain one `VarBinding` with `None` as a recovery value. No valid program can contain such a `VarBinding`.
- A declaration has no name (`var : i32 = 1`): The parser reports error E1008 (FR-017). The parser does not create a `Stmt::VarDeclaration`.
- A function has no parameters: The `parameters` field contains an empty `Vec<Parameter>`. The field is present.
- A function declaration has the name `main`: The ordinary function declaration is represented by `Stmt::Function`. The name field contains `"main"`. It is not represented by `Stmt::MainFunction`.
- The dedicated `main { ... }` construct: The AST represents it as `Stmt::MainFunction`. It contains `body` and `span`. It has no name, parameters, or return type field.
- `nullptr`: The AST represents `nullptr` as `Expr::Literal { value: LiteralValue::NullPtr, .. }`. The corresponding type is `Type::NullPtr`.
- An array type has a non-literal size expression: The `Type::Array { size, .. }` node contains the complete expression tree. It does not contain only an integer constant. The expression is a plain `Expr` in both the untyped tree and the typed tree. The expression has no `resolved_type` (FR-022).
- A numeric literal uses a non-decimal base, such as binary `#b...`, octal `#o...`, or hexadecimal `#x...`: The AST represents the literal as `LiteralValue::Numeric(Number)`. This variant is also used for decimal literals. The lexer resolves the numeric base.
- A `for` loop has no initializer, condition, or increment: The `initializer`, `condition`, and `increment` fields are each `None`. Each field is independent of the other fields. The loop body is present.
- A `for` loop has an initializer: The initializer is represented by `Some(Box<Stmt>)`. The contained statement is either `Stmt::VarDeclaration` or `Stmt::Expression`. A function declaration, main function, block, if, while, return, break, or continue is not a valid `for` initializer.
- An expression statement has no trailing semicolon: The `Stmt::Expression` node contains the expression. The lexer and parser handle the missing semicolon. The AST does not store a separate representation of the missing semicolon. The `Stmt::Expression` variant has its own `span` field. The span covers the complete statement. It includes the trailing semicolon when the semicolon is present. All other `Stmt` variants follow the same rule.
- `vector<T>` and a user-defined type named `vector`: In a type position, the parser creates `Type::Vector` when `vector` is followed by `<T>`. In all other cases, the identifier resolves to `Type::Custom`.
- `1 + 2` in the typed tree: The tree contains a `TypedExpr` with `kind` set to `TypedExprKind::Binary`. This node owns two `TypedExpr` operands. Both operands have `kind` set to `TypedExprKind::Literal`. Each of the three nodes has its own `resolved_type`.
- A grouping `(a + b)` in the typed tree: The tree contains a `TypedExpr` with `kind` set to `TypedExprKind::Grouping`. This node owns the `TypedExpr` for `a + b`. The grouping node and the inner node have independent `resolved_type` values.
- An `else if` chain in the typed tree: A `TypedElseBranch::ElseIf` contains a `TypedStmt::If`. This `TypedStmt::If` contains its own `TypedElseBranch`. The structure is recursive and follows the same representation used by `ElseBranch`.
- An assignment target: A valid `Expr::Assign.target` is an `Expr::Variable` or an `Expr::ArrayAccess`. A literal, grouping, call, binary expression, or unary expression is not a valid assignment target.
- A call with an expression as callee: The `Expr::Call.callee` field contains the complete callee expression. The AST does not require a separate callee name field.
- A return without a value: `Stmt::Return.value` is `None`.
- A break statement: `Stmt::Break` contains only its source span.
- A continue statement: `Stmt::Continue` contains only its source span.
- An `else` is followed by a statement that is not an `if` and not a block: The parser reports error E1004 (FR-007). The `else_branch` field is `ElseBranch::None`. The AST does not store the statement.
- Code creates an `ElseBranch` with a payload of another kind, for example an `ElseIf` with a `Stmt::Break`: The payload type is `Box<Stmt>`. The AST type therefore permits this value. The parser does not create this value. Code that reads the value treats the payload as a `Stmt` (FR-007). The build functions and the `erase` functions preserve the payload kind (FR-021).

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The AST must have a separate node variant for each of these expression kinds: binary operation, unary operation, grouping (parenthesized expression), literal, array literal, variable reference, assignment, function or method call, and array index access.

  The fields of each expression variant must be:

    - `Expr::Binary`: `left: Box<Expr>`, `op: BinaryOp`, `right: Box<Expr>`, and `span: SourceSpan`.
    - `Expr::Unary`: `op: UnaryOp`, `side: UnaryOpSide`, `expr: Box<Expr>`, and `span: SourceSpan`.
    - `Expr::Grouping`: `expr: Box<Expr>` and `span: SourceSpan`.
    - `Expr::Literal`: `value: LiteralValue` and `span: SourceSpan`.
    - `Expr::ArrayLiteral`: `elements: Vec<Expr>` and `span: SourceSpan`.
    - `Expr::Variable`: `name: String` and `span: SourceSpan`.
    - `Expr::Assign`: `target: Box<Expr>`, `value: Box<Expr>`, and `span: SourceSpan`.
    - `Expr::Call`: `callee: Box<Expr>`, `arguments: Vec<Expr>`, and `span: SourceSpan`.
    - `Expr::ArrayAccess`: `array: Box<Expr>`, `index: Box<Expr>`, and `span: SourceSpan`.

  A literal is numeric, string, character, boolean, or nullptr. The assignment variant covers only the simple assignment operator `=`. Compound-assignment operators, for example `+=` and `-=`, are `Expr::Binary` nodes with the applicable `BinaryOp` compound-assignment variant.

  `Expr::Grouping` represents explicit parentheses. The inner expression is stored in `expr`. The node span covers the complete parenthesized expression.

  `Expr::Call` stores the callee as an `Expr`. The callee is not stored as a separate name field. A call can therefore contain any expression allowed by the grammar as its callee.

  `Expr::ArrayAccess` stores the expression being indexed and the index expression as separate children.

  `Expr::ArrayLiteral` stores zero or more element expressions in source order.

  `Expr::Assign` stores the assignment target and the assigned value. For valid parser output, the target must be `Expr::Variable` or `Expr::ArrayAccess`.

  The AST must preserve the expression structure defined by operator precedence and associativity.

- **FR-002**: The AST SHALL have a separate node variant for each of these statement kinds: expression statement, variable declaration with a mutability flag, function declaration, main function declaration, if statement with optional else-if and else branches, while loop, three-clause for loop, block, return statement, break statement, and continue statement.

  The fields of each statement variant must be:

    - `Stmt::Expression`: `expr: Box<Expr>` and `span: SourceSpan`.
    - `Stmt::VarDeclaration`: `bindings: Vec<VarBinding>`, `type_annotation: Type`, `is_mutable: bool`, and `span: SourceSpan`.
    - `Stmt::Function`: `name: String`, `parameters: Vec<Parameter>`, `return_type: Type`, `body: Box<Stmt>`, and `span: SourceSpan`.
    - `Stmt::MainFunction`: `body: Box<Stmt>` and `span: SourceSpan`.
    - `Stmt::If`: `condition: Box<Expr>`, `then_branch: Box<Stmt>`, `else_branch: ElseBranch`, and `span: SourceSpan`.
    - `Stmt::While`: `condition: Box<Expr>`, `body: Box<Stmt>`, and `span: SourceSpan`.
    - `Stmt::For`: `initializer: Option<Box<Stmt>>`, `condition: Option<Expr>`, `increment: Option<Expr>`, `body: Box<Stmt>`, and `span: SourceSpan`.
    - `Stmt::Block`: `statements: Vec<Stmt>` and `span: SourceSpan`.
    - `Stmt::Return`: `value: Option<Expr>` and `span: SourceSpan`.
    - `Stmt::Break`: `span: SourceSpan`.
    - `Stmt::Continue`: `span: SourceSpan`.

  `Stmt::Function` represents the ordinary function declaration syntax. Its `name` field may contain `"main"`.

  `Stmt::MainFunction` represents the dedicated `main` function syntax. It has no name, parameter list, or return type field. Its body must be a `Stmt::Block`.

  A function declaration with the ordinary function syntax and the name `main` is always `Stmt::Function`. It is never converted to `Stmt::MainFunction`.

  The AST must not use one generic statement variant for these constructs.

- **FR-003**: The AST must have a type model. Each `Type` value is a basic type or a type constructor. The type model must contain:
    - the primitive scalar types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, `string`, and `bool`);
    - the fixed-size array type;
    - the dynamic vector type;
    - `void`;
    - `nullptr`;
    - user-defined custom named types.
  
  The `Type` definition must use these forms:
    - `Type::Custom { name: Arc<str> }`;
    - `Type::Array { element_type: Box<Type>, size: Box<Expr> }`;
    - `Type::Vector { element_type: Box<Type> }`;
    - unit variants for all primitive types, `Type::Void`, and `Type::NullPtr`.
  
  A function type `(s1, ..., sn) -> t` is derived from the parameter types and the return type of `Stmt::Function`. It is not a `Type` variant in this feature.

- **FR-004**: Each `Expr` node, each `Stmt` node, and each `Parameter` must have a `SourceSpan` value.
    - `Type`, `VarBinding`, `ElseBranch`, `TypedVarBinding`, and `TypedElseBranch` must not have a span field.
    - A node that comes directly from the source text must have a span. The span starts at the start of the first token of the node. The span ends at the end of the last token of the node.
    - The source span of a type annotation belongs to the node that contains the `Type` value. A `Type` value does not carry its own source span.
    - For the return type of `Stmt::Function`, the function `span` is the source span for the complete function. The `return_type` field has no separate span.
    - The parser makes `Type::Void` when a function has no declared return type. This value is synthesized from the absence of the return type annotation. It has no source token and no span.
    - An explicit `void` return type and an omitted return type both produce `Type::Void`. The AST does not preserve whether the `Type::Void` value came from the source token `void` or from an omitted annotation.
    - No zero-width `SourceSpan` is required for a synthesized `Type::Void`.
    - The parser makes a placeholder `Expr` when the expression after a unary operator, a binary operator, `=`, or `[` is missing or is not valid. The placeholder keeps the syntax tree complete after a syntax error.
    - The placeholder is `Expr::Literal` with `LiteralValue::NullPtr`. It has no source token. Its span is the span of the token that comes before the missing expression. This token is the unary operator, the binary operator, `=`, or `[`.
    - The `Stmt::Expression` variant must have its own explicit `span: SourceSpan` field.
    - The span of a `Stmt::Expression` must cover the full statement. If the statement has a trailing terminator, the span must include the terminator.
    - The span of a `Stmt::Expression` must not come only from the inner `Expr`.

- **FR-005**: The `Stmt::VarDeclaration` node must contain an ordered `bindings` collection, one `type_annotation`, and an `is_mutable` flag. The `bindings` collection must keep the order of the identifiers in the source text. Each binding in the `bindings` collection has the type of the shared `type_annotation`. The `is_mutable` flag must be true when the declaration uses `var`. The `is_mutable` flag must be false when the declaration uses `const`.

- **FR-006**: The `Stmt::Function` node must contain the function name, the formal parameters, the return type, and the body.

  The fields must have these types:

    - `name: String`;
    - `parameters: Vec<Parameter>`;
    - `return_type: Type`;
    - `body: Box<Stmt>`;
    - `span: SourceSpan`.

  The formal parameters must be an ordered list of `Parameter` nodes. The order of the list must be the same as the order in the source. Each `Parameter` node must contain a name, a type annotation, and a source span. The body must be a `Stmt::Block`.

  If the source has no return type, the node must use `Type::Void`. An explicit `void` annotation and an omitted return type have the same `Type::Void` value. The AST does not store their source origin in the `Type` value.

  The `Stmt::Function.span` is the source span for the complete function.

  The node must contain enough information to build the function type `(s1, ..., sn) -> t`, where `s1` to `sn` are the parameter types and `t` is the return type.

  A function written with the ordinary function declaration syntax and named `main` is represented by `Stmt::Function`. The name is stored in `name`. It is not represented by `Stmt::MainFunction`.

  `Stmt::MainFunction` is a different node. It represents the dedicated `main` function syntax. It contains `body` and `span` only.

  The node must not contain tokens that have no meaning in the tree, such as parentheses and commas.

- **FR-007**: The `ElseBranch` model must show three cases: no else clause (`None`), a plain else block (`Block`), and an else-if continuation (`ElseIf`). The model must show chained conditionals explicitly.
    - The payload of `Block` and the payload of `ElseIf` have the type `Box<Stmt>`.
    - The payload of `ElseIf` must be a `Stmt::If` node. The payload of `Block` must be a `Stmt::Block` node.
    - The grammar of the else part is: `else_part -> "else" if_stmt | "else" block | empty`. The parser must use this grammar to make an `ElseBranch`.
    - The parser must select the production from the next token only. It must not read more tokens, and it must not go back to a token that it already read.
    - If the next token is not `else`, the parser must make `ElseBranch::None`.
    - After `else`, the parser must make `ElseIf` if the next statement is an `if`. It must make `Block` if the next statement is a block.
    - The parser must attach an `else` to the nearest `if` that does not have an `else`.
    - The parser must not make a node for the keyword `else`. The variant of `ElseBranch` holds this information.
    - In a chained conditional, an `ElseIf` holds a `Stmt::If`. The `ElseBranch` of this `Stmt::If` continues the chain. The chain ends with `None` or `Block`.
    - If the next statement is not an `if` and not a block, the parser must report the error E1004 (invalid else branch). The error report must give the position of the token that follows `else`. The parser must then make `ElseBranch::None`. It must not store the statement. The parser must continue so that it can find later errors.
    - The type `ElseBranch` does not prevent a payload of another kind. Existing tests make `ElseIf` and `Block` values with a `Stmt::Break` payload. A different payload type causes a compilation error in these tests (SC-006). For this reason, the payload type stays `Box<Stmt>`.
    - Code that reads an `ElseBranch` must accept any `Stmt` as the payload of `Block` or `ElseIf`. The code must not fail if the payload has another kind.

- **FR-008**: The AST must represent each binary operator as one variant of a closed enumeration. The enumeration must include one variant for each arithmetic, comparison, logical, bitwise, shift, and compound-assignment operator in the grammar. Each operator must have exactly one variant. Each variant must have exactly one operator. The enumeration must not contain an unknown variant or a default variant. A binary-operator node must have one operator variant, one left operand, and one right operand. The AST must not use a string or an integer to identify an operator. The AST must not store precedence or associativity. The parser must use precedence and associativity only to decide the shape of the tree. The logical AND operator and the logical OR operator must have their own variants. These variants must be different from the bitwise AND variant and the bitwise OR variant.

- **FR-009**: The AST must show each unary operator as a node with one operator and one operand. The AST must show the unary operators as a closed enumeration. The enumeration must contain negation (`-`), logical NOT (`!`), bitwise complement (`~`), increment (`++`), and decrement (`--`). Each unary operator node must have a `UnaryOpSide` value. The `UnaryOpSide` value is `Prefix` or `Postfix`. It shows the side of the operand where the operator is applied. Negation, logical NOT, and bitwise complement must have the `Prefix` value. Increment and decrement can have the `Prefix` value or the `Postfix` value. The operand of an increment or decrement must be an lvalue. For increment and decrement, the `Prefix` value gives the new value of the operand as the value of the expression. The `Postfix` value gives the old value of the operand as the value of the expression.

- **FR-010**: The `Type::Array` variant must contain the element type and the size. The element type is a `Type` value. The size is an `Expr` node.
  
  The size expression must be a compile-time constant expression that evaluates to a non-negative integer. The type checker must evaluate the size expression before it uses the array type in type equivalence or other semantic checks. If the size expression cannot be evaluated to a non-negative integer, the type checker must report an error.
  
  Two array types are equivalent when:
    - their element types are equivalent; and
    - their evaluated size values are equal.
  
  The source form of the size expression does not affect type equivalence after evaluation. For example, `i32[10]` and `i32[5 + 5]` are equivalent when both expressions evaluate to the integer value `10`.
  
  Source spans inside the size expressions do not affect type equivalence. Type equivalence compares the semantic array size value, not the source position.

- **FR-011**: The `Type::Vector` variant must contain one element type. The element type must be a `Type` value. Two `Type::Vector` values are equivalent only if their element types are equivalent. The source syntax of a `Type::Vector` must be different from the source syntax of a `Type::Custom` named `vector`.

- **FR-012**: Each `Expr` node must have a `span()` accessor. The accessor must return a reference to the `SourceSpan` of the node. The `SourceSpan` must identify the first and the last source position of the text that the node represents. The span of a parent node must include the span of each child node.

- **FR-013**: Each `Stmt` node must have a `span()` accessor. The accessor must return a reference to the `SourceSpan` of the node. The `SourceSpan` must identify the first and the last source position of the text that the node represents. The span of a parent node must include the span of each child node.

- **FR-014**: The AST must have a typed tree. Each expression node in the typed tree has its own resolved-type slot, at each depth.
    - The typed tree is an annotated syntax tree. An annotated syntax tree is a syntax tree that shows the attribute values at each node.
    - The type attribute of an expression node is its `resolved_type` field. A type-checking pass is a semantic analysis pass that sets this field.
    - The typed tree is in the `syntax::typed_ast` module. It has `TypedExpr` and `TypedStmt` (FR-019, FR-020).
    - Semantic analysis passes must store and read resolved types only through the `resolved_type` field of `TypedExpr`.
    - Semantic analysis passes must not use external side tables. A side table is a collection outside the typed tree. The collection uses a node identifier, a node address, or a source span as key.
    - A symbol table is not a side table if it uses a name and a scope as key. A symbol table must not use a node identifier, a node address, or a source span as key.
    - The core `Expr` enum must not change.
    - The `Expr` enum must stay usable without a resolved type. The parser and the other pre-semantic phases need this.
    - The annotation applies to each expression separately. Each `TypedExpr` node has an independent `resolved_type` value.
    - A write to `resolved_type` on one node must not change any other node.
    - A resolved type is a type expression. A type expression is a basic type, a type name, or a type constructor applied to type expressions.
    - Each variant of the `Expr` enum has one type rule. A type rule gives the resolved type of a node from the inputs of that node.
    - The inputs of a type rule are the data of the node, the `resolved_type` of each child node, and the symbol table.
    - The type rule of a node computes `resolved_type` as a synthesized attribute. A synthesized attribute gets its value from the node and from the child nodes of the node.
    - A type-checking pass must resolve all child nodes of a node before it resolves that node. The order goes from the leaf nodes to the root node.
    - If a type rule needs information from a parent node or a sibling node, the type-checking pass must give this information to the type rule as an inherited attribute.
    - A type-checking pass must not store an inherited attribute in the typed tree. The pass sends the attribute to the child node during the traversal.
    - A type-checking pass can use type inference. Type inference finds the type of an expression from the way the program uses that expression.
    - If a type rule converts the type of an operand implicitly, the `resolved_type` of the operand node must not change.
    - If a node has a type error, the type-checking pass must report the error one time. After the error, the pass must continue to the next node.
    - A type-checking pass must not report a type error for a parent node only because a child node has a `resolved_type` of `None`.
    - `resolved_type` must be `Some(T)` if the type of that specific expression can be determined. This is true also if other parts of the program have errors.
    - `resolved_type` must be `None` if the type of that specific expression cannot be resolved.
    - `resolved_type` must be `None` on each node of a tree that no type-checking pass has processed.

- **FR-015**: The `Stmt::For` node must store the initializer, the condition, and the increment as three independent optional fields.

  The fields must have these types:

    - `initializer: Option<Box<Stmt>>`;
    - `condition: Option<Expr>`;
    - `increment: Option<Expr>`;
    - `body: Box<Stmt>`;
    - `span: SourceSpan`.

  The initializer is a statement position with a restricted set of valid variants. When the initializer is present, it must be either `Stmt::VarDeclaration` or `Stmt::Expression`. It must not be a function declaration, main function declaration, if statement, while loop, for loop, block, return statement, break statement, or continue statement.

  The condition is an optional expression. The increment is an optional expression. Each field is `None` when the source code omits the corresponding clause and `Some` when the source code contains the corresponding clause. The state of one field must not change the state of another field.

  The parser must accept the production `stmt -> for ( for_initializer ; optexpr ; optexpr ) stmt`, where `for_initializer` is empty, a variable declaration, or an expression statement, and where each `optexpr` is empty or `expr`.

  The parser must accept all valid combinations of an absent or present initializer, condition, and increment. The parser must require both semicolons in the `for` header for each combination.

  The AST does not store the two semicolons. They are syntax delimiters and are not AST nodes.

  The code generator must evaluate the initializer one time, before the first test of the condition. The code generator must evaluate the condition before each iteration of the body. The code generator must evaluate the increment after each iteration of the body and before the next test of the condition.

  If the condition is `None`, the code generator must treat the condition as true and must not generate a test. If the initializer or the increment is `None`, the code generator must not generate code for that field. If the language has a `continue` statement, a `continue` in the body must transfer control to the increment.

- **FR-016**: The lexer must read a numeric literal with a radix prefix (binary `#b`, octal `#o`, or hexadecimal `#x`) as one token. The lexer must select the longest lexeme that matches the pattern of the literal. The lexer must convert the digits to a value in the radix that the prefix gives. The lexer must give the token the same `LiteralValue::Numeric(Number)` variant as a decimal literal. The parser must not receive the radix. A digit that is not valid in the radix (for example `#b2`) must cause a lexical error. The lexical error must show the position of the lexeme in the source text.

- **FR-017**: The AST must represent a multi-binding variable declaration (`var a, b: T = e1, e2`) as one `Stmt::VarDeclaration` with several `VarBinding` entries. Each `VarBinding` must pair one name with one initializer. The initializer is mandatory in the language. The type of the `initializer` field is `Option<Expr>`.
    - The `Option<Expr>` type describes the AST representation. It does not make the initializer optional in the language grammar.
    - This requirement is the only place that defines the initializer rule. The other sections refer to FR-017. They do not define a different rule.
    - A declaration is valid only when it has at least one name and the number of initializers is equal to the number of names.
    - Each name must have exactly one initializer in a valid declaration.
    - The initializer-count rule is a context-sensitive condition. A context-free grammar cannot check it. The parser checks it after it parses both lists (Aho et al., Section 4.3.5).
    - In a valid declaration, the `initializer` of every `VarBinding` must be `Some`.
    - `None` is not a valid language form. `None` represents only parser recovery after an initializer-count error or an AST value constructed directly by program code.
    - The source form contains a list of names and a list of initializers.
    - The parser must parse the complete initializer list before it compares the number of names with the number of initializers.
    - If the declaration has at least one name and the number of names is not equal to the number of initializers, the parser must report error E2001 (initializer count mismatch).
    - The parser must report E2001 one time for each declaration.
    - After E2001, the parser must still make one `Stmt::VarDeclaration`.
    - After E2001, the parser must make one `VarBinding` for each name, in source order.
    - The n-th name must receive the n-th initializer.
    - A name with no n-th initializer must receive `None`.
    - If there are more initializers than names, the parser must parse each extra initializer but must not store an extra `VarBinding` or an extra initializer in the AST.
    - The parser must not use one initializer for more than one name.
    - The parser must not move an initializer to a later name when an earlier name has no initializer.
    - A declaration without `=` is invalid. This rule applies to both `var` and `const`.
    - A declaration without `=` has zero initializers. The initializer-count rule therefore reports E2001 when the declaration has at least one name.
    - If the declaration has no name, the parser must report error E1008 (missing variable name). The parser must not make a `Stmt::VarDeclaration`. The rules above do not apply.
    - After E1008, the parser must skip tokens up to the next synchronization token. The parser then continues with the next statement.
    - Code that reads the AST must distinguish a valid `Some` initializer from a recovery `None` initializer. A later semantic phase must not treat `None` as a valid declaration with no initializer.
    - The declared `type_annotation` applies to every `VarBinding` in the declaration.

- **FR-018**: Each node variant of the AST must show one construct of the source language only. Each construct must use one node variant only. Two constructs with different meaning must not use the same node variant, even if their fields are the same.

  Each node variant definition must give a label and a fixed field schema. A field with type `Vec<T>` may contain zero or more children. A field with type `Option<T>` may contain zero or one child. The field type and its cardinality are part of the node definition.

  The following distinctions are mandatory:

    - `Stmt::Function` and `Stmt::MainFunction` are different node variants.
    - An ordinary function declaration named `main` is `Stmt::Function`.
    - The dedicated `main` function syntax is `Stmt::MainFunction`.
    - `Expr::Call.callee` is an expression, not a function name.
    - `Expr::Assign.target` is an expression. Valid parser output restricts it to `Expr::Variable` and `Expr::ArrayAccess`.
    - `Stmt::Return.value` is optional.
    - `Stmt::Break` and `Stmt::Continue` have no child expressions.
    - `Stmt::For.initializer` is an optional statement. Valid parser output restricts it to `Stmt::VarDeclaration` and `Stmt::Expression`.
    - `Expr::Grouping` is a distinct node for explicit parenthesized expressions.
    - `Expr::ArrayAccess` and `Expr::ArrayLiteral` are distinct nodes.
    - `VarBinding.initializer` has exactly one child in valid parser output. Its `Option<Expr>` type represents parser recovery only (FR-017).

  The AST must not contain nodes that exist only for the grammar, such as punctuation, keywords, and chain productions.

  The rule applies to AST nodes. It does not require a distinct `Type` value for the absence of an optional type annotation. In particular, an omitted function return type is not a second `Type` variant. It is the absence of source syntax that the parser represents with the default value `Type::Void`.

  Explicit `void` and an omitted return type are therefore semantically equivalent in the AST. The AST does not preserve this source-level distinction.

- **FR-019**: `TypedExpr` must be a struct with exactly three public fields:
    - `kind`, of type `TypedExprKind`;
    - `span`, of type `SourceSpan`;
    - `resolved_type`, of type `Option<Type>`.

  `TypedExpr` must not contain an `Expr`.

  A `TypedExpr` is a node of the annotated syntax tree. The `resolved_type` field is the synthesized attribute `type` of the node. The type checker calculates it from the `kind` of the node and the `resolved_type` of each child node. For an identifier, the type checker also uses the symbol table. The symbol table is not part of the typed tree. `resolved_type` must be `None` until the type checker calculates the type of the node. After the type checker calculates a type for the node, `resolved_type` must be `Some` with this type. If the type checker cannot calculate a type for the node, `resolved_type` must stay `None` (FR-014).

  `TypedExprKind` must be an enum. It must have one variant for each `Expr` variant (FR-001). Each variant must have the same name as the related `Expr` variant.

  The typed tree must have the same shape as the `Expr` tree. Each `Expr` node must have one related `TypedExpr` node. The typed tree must have no other node.

  Each field of an `Expr` variant becomes a field of the related `TypedExprKind` variant. Use these rules:
    - A field of type `Expr` becomes `TypedExpr`. The same change applies inside `Box`, `Vec`, and `Option`.
    - `TypedExpr::span` stores the `SourceSpan` of the variant one time. The variant must not repeat it.
    - Each other field keeps its type. These fields are operators, `LiteralValue`, identifier names, and `Type`.

  A `TypedExpr` owns its child `TypedExpr` nodes. A child node has exactly one parent node. To get the resolved type of a sub-expression, go from the parent node to the child node and read its `resolved_type`. The typed tree has no identifier, index, or lookup structure.

- **FR-020**: `TypedStmt` must be an enum. It has one variant for each `Stmt` variant (FR-002). Each variant has the same name as the `Stmt` variant and has a `SourceSpan` (FR-004).

  The fields of a `Stmt` variant carry over to the `TypedStmt` variant with these rules. Each rule also applies inside `Box`, `Vec`, and `Option`.
    - A field of type `Expr` becomes `TypedExpr`.
    - A field of type `Stmt` becomes `TypedStmt`.
    - A field of type `ElseBranch` becomes `TypedElseBranch`.
    - A field of type `VarBinding` becomes `TypedVarBinding`.
    - Each other field keeps its type. These fields are `is_mutable`, names, `Parameter`, `Type`, and `SourceSpan`.

  `TypedElseBranch` must have the same three cases as `ElseBranch` (FR-007). Its `Block` payload and its `ElseIf` payload have the type `Box<TypedStmt>`. For parser output, the `ElseIf` payload is a `TypedStmt::If` and the `Block` payload is a `TypedStmt::Block`.

  `TypedVarBinding` must pair the name with an `Option<TypedExpr>` initializer, in the same way as `VarBinding`. The rule of FR-017 applies: the initializer is mandatory in a valid tree, and `None` is a recovery value only.

  `TypedStmt` has no `resolved_type` of its own. A statement does not have a value. In the typed tree, only a `TypedExpr` node holds a type attribute.

- **FR-021**: The build functions `TypedExpr::from_expr(&Expr) -> TypedExpr` and `TypedStmt::from_stmt(&Stmt) -> TypedStmt` must build the typed tree from the untyped tree.
    - A build function must make a tree with the same structure as its input. It keeps the variants, their order, the values of copied fields, and the spans.
    - A build function must set `resolved_type` to `None` on each `TypedExpr`. The field `resolved_type` is the type attribute of an expression node (Aho et al., Section 5.1.1). The value `None` means that no pass has computed this attribute. A build function does not compute it.
    - A build function must not change its input.
    - A build function must make a new node for each node of its input. The typed tree must be a tree and not a directed acyclic graph (Aho et al., Section 6.1.1). Two nodes must not share a child, even if their subtrees are equal. Each node holds its own `resolved_type`, and the type of an expression can be different at each position in the tree.
    - A build function must be an exhaustive `match` over the `Expr` variants or the `Stmt` variants. It must not have a wildcard arm. When a developer adds a variant to `Expr` or `Stmt`, the compilation of `descar-core` fails until the typed tree handles the new variant.

  The functions `TypedExpr::erase(&self) -> Expr` and `TypedStmt::erase(&self) -> Stmt` must rebuild the untyped tree. They do not use any `resolved_type`.
    - For each `Expr` value `e`, `TypedExpr::from_expr(&e).erase()` must be structurally equal to `e`, node by node, with spans.
    - The same rule applies to `Stmt`.
    - Two nodes are structurally equal if they have the same variant, the same field values, the same children in the same order, and the same span. Structural equality is not type equivalence (Aho et al., Section 6.3.2).
    - The result of `erase` must be the same for a typed tree with any `resolved_type` values. A pass that sets `resolved_type` must not change the result of `erase`.
    - The build functions and the `erase` functions must each process each node once.
    - The build functions and the `erase` functions must keep the kind of each `ElseBranch` payload. They must not check it and they must not change it (FR-007).

- **FR-022**: `TypedExpr` and `TypedStmt` must satisfy FR-004, FR-012, and FR-013.
    - `TypedExpr` and each `TypedStmt` variant must have a `span()` accessor.
    - The span of a typed node must be equal to the span of the untyped node that it comes from.

  The type annotation positions in the typed tree are variable declarations, `Parameter`, and return types.
    - Each such position must hold the same `Type` value as the untyped tree.
    - The two trees share `Type`. `Type` has no typed counterpart. A `Type` value is a type expression (Aho et al., Section 6.3.1). It is not an expression node.
    - No `Type` value in the typed tree can hold a `TypedExpr`.

  The `size` expression inside `Type::Array` (FR-010) is a plain `Expr` in both trees.
    - It has no `resolved_type`. It is in a type position and it is not an expression node of the typed tree.
    - A type-checking pass can build a `TypedExpr` from the size expression with `TypedExpr::from_expr` during the check. The pass must not store the result.

- **FR-023**: `TypedExpr`, `TypedExprKind`, `TypedStmt`, `TypedElseBranch`, and `TypedVarBinding` must be in the `syntax::typed_ast` sub-module of `descar-core`. The untyped AST must stay in `syntax::ast`. The module `syntax::typed_ast` can import from `syntax::ast`. The module `syntax::ast` must not import from `syntax::typed_ast`. The two modules must stay separate.
    - The module `syntax::ast` must not have a field for an attribute that a pass computes, such as `resolved_type`. These attributes belong only in `syntax::typed_ast`.

- **FR-024**: The `TypeChecker` must keep the function `check(&mut self, statements: &[Stmt]) -> Vec<CompileError>`. The function must keep its signature and its results (SC-006).
    - The `TypeChecker` must have the new function `check_typed(&mut self, statements: &mut [TypedStmt]) -> Vec<CompileError>`.
    - The function `check_typed` must set `resolved_type` on each `TypedExpr` node of the typed tree. It must follow the rules of FR-014.
    - The function `check_typed` must change only `resolved_type`. It must not change the structure of the typed tree.
    - The type rule of each `Expr` variant in `check_typed` must be the same as the type rule in `check`. The `resolved_type` of a node is `Some(T)` where `check` finds the type T for the related `Expr` node. The `resolved_type` of a node is `None` where `check` finds no type for the related `Expr` node.
    - For each program, `check_typed` must report the same errors as `check`, in the same order.
    - Before this feature, no code and no test uses a resolved type (Assumptions). The function `check_typed` is the first function that sets `resolved_type`.

### Key Entities

- **`Expr`**: The sum type of all expression node variants. Each variant has a `SourceSpan`. The type has a `span()` accessor.

  The expression variants and their fields are:

    - `Binary`: `left`, `op`, `right`, `span`.
    - `Unary`: `op`, `side`, `expr`, `span`.
    - `Grouping`: `expr`, `span`.
    - `Literal`: `value`, `span`.
    - `ArrayLiteral`: `elements`, `span`.
    - `Variable`: `name`, `span`.
    - `Assign`: `target`, `value`, `span`.
    - `Call`: `callee`, `arguments`, `span`.
    - `ArrayAccess`: `array`, `index`, `span`.

- **`Stmt`**: The sum type of all statement node variants. Each variant has a `SourceSpan`. No variant is an exception. This satisfies FR-004.

  The statement variants and their fields are:

    - `Expression`: `expr`, `span`.
    - `VarDeclaration`: `bindings`, `type_annotation`, `is_mutable`, `span`.
    - `Function`: `name`, `parameters`, `return_type`, `body`, `span`.
    - `MainFunction`: `body`, `span`.
    - `If`: `condition`, `then_branch`, `else_branch`, `span`.
    - `While`: `condition`, `body`, `span`.
    - `For`: `initializer`, `condition`, `increment`, `body`, `span`.
    - `Block`: `statements`, `span`.
    - `Return`: `value`, `span`.
    - `Break`: `span`.
    - `Continue`: `span`.

  `Stmt::Function` may have `name = "main"` when the ordinary function declaration syntax is used.

  `Stmt::MainFunction` represents the dedicated `main` function syntax. It has no name, parameters, or return type.

  `Stmt::For.initializer` is `Option<Box<Stmt>>`. Parser output uses only `Stmt::VarDeclaration` and `Stmt::Expression` in this field.

- **`Type`**: The sum type of all type annotation values. `Type` has no `SourceSpan`.
  
  The primitive and special unit variants are `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`, `F32`, `F64`, `Char`, `String`, `Bool`, `Void`, and `NullPtr`.
  
  `Type::Custom` contains `name: Arc<str>`.
  
  `Type::Array` contains `element_type: Box<Type>` and `size: Box<Expr>`.
  
  `Type::Vector` contains `element_type: Box<Type>`.
  
  A `Type::Void` value can come from an explicit `void` annotation or from an omitted function return annotation. The `Type` value does not preserve this origin.
  
  Type equivalence is a semantic rule. It does not use `SourceSpan`. Two array types are equivalent when their element types are equivalent and their evaluated non-negative integer sizes are equal.

- **`BinaryOp`**: A closed enumeration of all binary operators.

- **`UnaryOp`**: A closed enumeration of all unary operators.

- **`UnaryOpSide`**: An enumeration with two variants. It shows prefix or postfix application.

- **`LiteralValue`**: The sum type of all literal value kinds. The kinds are numeric, string, char, bool, and nullptr.

- **`Parameter`**: A struct with the public fields `name: String`, `type_annotation: Type`, and `span: SourceSpan`, for function parameters. The typed tree and the untyped tree share this type.

- **`VarBinding`**: A struct with the public fields `name: String` and `initializer: Option<Expr>`, for multi-binding declarations. `VarBinding` has no `SourceSpan`. The initializer is mandatory in the language (FR-017). The `Option<Expr>` field is part of the AST representation. It does not mean that the language permits a missing initializer. In a valid parsed declaration, `initializer` is always `Some(Expr)`. `None` is allowed only for parser recovery after E2001 or for an AST value constructed directly by program code. The n-th binding corresponds to the n-th name and the n-th initializer. A declaration is valid only when the number of names is equal to the number of initializers.

- **`ElseBranch`**: An enumeration with three variants, for the else clause of an if statement. `ElseBranch` has no `SourceSpan`.

- **`TypedExpr`**: A struct in `syntax::typed_ast`. It has the fields `kind: TypedExprKind`, `span: SourceSpan`, and `resolved_type: Option<Type>`.

- **`TypedExprKind`**: An enum in `syntax::typed_ast`. It has one variant for each `Expr` variant.

- **`TypedStmt`**: An enum in `syntax::typed_ast`. It has one variant for each `Stmt` variant.

- **`TypedElseBranch`**: An enumeration with three variants, in the same way as `ElseBranch`.

- **`TypedVarBinding`**: A struct with the public fields `name: String` and `initializer: Option<TypedExpr>`. The rule of FR-017 applies to its initializer.

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

- Q: Is the initializer optional in the language? A: No. A valid `var` or `const` declaration must have exactly one initializer for each declared name.
- Q: Why is the type of `VarBinding::initializer` `Option<Expr>`? A: `Option<Expr>` represents the AST state for both valid and recovered trees. `Some(expr)` represents an initializer. `None` represents a missing initializer after parser recovery from E2001, or an AST value created directly by program code. `None` does not define a valid source construct.
- Q: What happens when the number of names is not equal to the number of initializers? A: The parser reports E2001 after it parses both lists. The parser creates one `VarBinding` for each name. The n-th name receives the n-th initializer. If a name has no n-th initializer, its initializer is `None`. Extra initializers are parsed but are not stored in the AST.
- Q: Does `var a: i32;` represent an uninitialized variable? A: No. The source form is invalid. The parser reports the required syntax errors and the initializer-count error. A recovered AST can contain `VarBinding { name: "a", initializer: None }`. This value is an error-recovery state, not a valid declaration.
- Q: What does the parser do when the declaration has no name? A: The parser reports the error E1008 and does not make a `Stmt::VarDeclaration`. The parser stops before it reads the initializers, so the count rule of FR-017 does not apply.
- Q: What is the type of the `ElseBranch::ElseIf` payload? The spec said that the payload contains a `Stmt::If`, but it did not name the type. A: The type is `Box<Stmt>`, the same as in the existing code. The rule that the payload is a `Stmt::If` applies to parser output (FR-007).
- Q: Why does the spec not use a payload type that can only hold an `if`? A: Existing tests make `ElseIf` and `Block` values with a `Stmt::Break` payload. A new payload type stops the compilation of these tests (SC-006). The spec keeps `Box<Stmt>` and puts the rule on the parser and on the code that reads the value.
- Q: What does the parser do when the statement after `else` is not an `if` and not a block? A: The parser reports the error E1004 and makes `ElseBranch::None`. It does not store the statement. The parser already does this.
- Q: What must code that reads an `ElseBranch` do with a payload of another kind? A: It must accept any `Stmt`. The type checker and the printer already do this. The build functions and the `erase` functions of the typed tree keep the payload kind and do not check it (FR-021).

### Session 2026-10-06

- Q: Which types have a `SourceSpan`? US3, FR-004, and SC-004 gave three different scopes. A: `Expr`, `Stmt`, and `Parameter` have a span. `TypedExpr` and each `TypedStmt` variant have a span (FR-022). `Type`, `VarBinding`, `ElseBranch`, `TypedVarBinding`, and `TypedElseBranch` have no span of their own. This is the structure of the existing code.
- Q: Does the `Type::Void` that the parser makes for an omitted return type have a span? A: No. `Type` is an enum, and its unit variants (for example `Type::F64`) have no span. `Stmt::Function` has one span, for the whole function. The existing tests make `Stmt::Function` values with a struct literal and print `Type` values with `Debug`. A span field in `Type` or a new span field in `Stmt::Function` stops the compilation of these tests or changes their snapshots (SC-006). For this reason the spec does not require a span for `Type::Void`.
- Q: Does the spec still require a zero-width span at the closing `)` of the parameter list? A: No. The rule needed a place to store the span, and the AST has no such place. The earlier text applied the rule to `Type::Void` only. `SourceSpan::point` can make a zero-width span, but no node in this spec has one.
- Q: Which span does a placeholder `Expr` have? A: The parser makes `Expr::Literal` with `LiteralValue::NullPtr` when the expression after a unary operator, a binary operator, `=`, or `[` is missing or not valid. The span is the span of that operator token or of that `[` token. The parser already does this. The span is not zero-width. It does not reproduce a source fragment of the placeholder.

### Session 2026-10-07

- Q: Does a mechanism that stores a resolved type exist before this feature? A: No. `TypeChecker::check` returns a `Vec<CompileError>` and stores no type on a node. The visit functions of the type checker return an `Option<Type>` for each expression, and `check` does not keep it. No code and no test uses `resolved_type`, `TypedExpr`, or `TypedStmt`. FR-014 adds the first mechanism. It does not replace an existing mechanism.
- Q: SC-005 said that existing tests verify the annotation. Which tests verify it? A: New tests verify it. The existing tests in `type_checker_tests.rs` and `type_checker_snapshot_tests.rs` examine errors only. They call `check`, `is_same_type`, and `get_size`. They pass without change (SC-006). The new tests are in new files, so they do not change an existing test.
- Q: Which function sets `resolved_type`? A: The new function `TypeChecker::check_typed` (FR-024). The function `check` keeps its signature and its results (Assumptions, SC-006).
- Q: How does the spec keep `check` and `check_typed` consistent? A: The type rule of each `Expr` variant is the same in both functions. For each program, both functions report the same errors in the same order (FR-024, SC-005). Only the place of the result is different. `check` uses the type during the traversal and does not keep it. `check_typed` also stores the type in the node (Aho et al., Section 6.5.1).

### Session 2026-10-07-A01

- Q: Where does the span of the synthesized `Type::Void` live?
  
  A: It does not live in `Type`. `Type` has no `SourceSpan`. The parser creates `Type::Void` when a function has no return annotation. This value has no source token and no span. The enclosing `Stmt::Function.span` covers the complete function. No zero-width span is required for `Type::Void`.

- Q: How does the AST distinguish explicit `void` from an omitted return type?
  
  A: It does not distinguish them. Both forms produce `return_type = Type::Void`. The `Type` value does not preserve source origin. The omission is not a type variant and is not a separate AST node.

- Q: Are `i32[10]` and `i32[5 + 5]` equivalent types?
  
  A: Yes, when both size expressions are valid compile-time constant expressions and both evaluate to the same non-negative integer. Array type equivalence compares the evaluated size values. It does not compare the source spans or source spelling of the size expressions.

- Q: Does `Type` equality use source spans?
  
  A: No. `Type` has no source span. Type equivalence does not use source positions. In particular, two equal array sizes at different source positions can still produce equivalent array types.

### Session 2026-10-08

- Q: Is the initializer of a `VarBinding` mandatory or optional? FR-017 required one initializer for each name, and the `Option<Expr>` type of the field could be read as optional. A: It is mandatory in the language. A valid `var` or `const` declaration has exactly one initializer for each declared name. `None` is a recovery value only.
- Q: Why does the spec keep `Option<Expr>` for a mandatory initializer? A: The existing `VarBinding` has the field type `Option<Expr>`. The parser uses `None` for recovery after E2001. A change of the field type to `Expr` is a breaking change under SC-006.
- Q: FR-018 says that an `Option<T>` field has zero or one child. Does this make the initializer optional? A: No. FR-018 lists `VarBinding.initializer` as an exception. In valid parser output it has exactly one child. The `Option` type represents parser recovery only.
- Q: Which rule checks that the number of names is equal to the number of initializers? A: The parser checks it. The initializer-count rule is a context-sensitive condition, and a context-free grammar cannot check it (Aho et al., Section 4.3.5). The parser parses both lists first. Then it compares the counts (FR-017).
- Q: Which section defines the initializer rule? A: FR-017 only. The Key Entities entries, the Edge Cases, FR-018, and FR-020 refer to FR-017.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For each syntactically valid `.dr` source file, the AST has no unknown node and no generic node for a construct in the grammar. A coverage test lists each grammar production. Each production has an AST node variant, or a written reason why it has none. A production that only groups tokens, for example a parenthesized expression, has an AST node variant when the parentheses are an explicit AST construct. The test fails when a grammar production that produces a semantic AST construct is not in the list.
- **SC-002**: Each type annotation position in the AST uses a concrete `Type` value. The positions are variable declarations, parameters, return types, and array sizes. When the source has a type keyword, the parser makes the corresponding `Type` value. The parser does not use `Type::Custom` as a default value.
  
  A function with an explicit `void` return annotation has `return_type = Type::Void`.
  
  A function with no return annotation also has `return_type = Type::Void`.
  
  These two forms are semantically equivalent in the AST. The AST does not preserve their source-level distinction.
- **SC-003**: Each example `.dr` file in `dr_files/` parses to an AST that passes the structural completeness check. The check passes when each node variant, span, and child field that the grammar needs is present and has a value.
- **SC-004**: A round-trip property applies to each `Expr` node, each `Stmt` node, and each `Parameter` with a direct text origin. Each of these nodes comes from one or more source tokens. For each of these nodes:
    - The span of the node is a source range. The range starts with the first lexeme of the node and ends with the last lexeme of the node. The span comes from `span()`, or from the `span` field of `Parameter`.
    - The text of the original source in that range is the same as the source fragment that makes the node.
  
  A node with no direct text origin is not subject to the round-trip property.
    - The `Type::Void` return type of a function with no return annotation has no source token and no span.
    - No zero-width span is required for the synthesized `Type::Void`.
    - The placeholder `Expr` of FR-004 has the span of the token that comes before the missing expression. The text in this span is the text of the previous token. It is not a source fragment of the placeholder.
- **SC-005**: The type checker uses type synthesis. It finds the type of an expression from the types of its subexpressions. The existing type-checker tests are in `type_checker_tests.rs` and `type_checker_snapshot_tests.rs`. They call `TypeChecker::check` and they examine its errors. They do not use a resolved type, because no resolved-type annotation exists before this feature. These tests pass without change (SC-006).
    - New tests examine the resolved-type annotation on `TypedExpr` trees at each depth. They are in new test files in `crates/descar-core/tests/`. They call `TypeChecker::check_typed` (FR-024).
    - After `check_typed` runs on a program, each resolvable `TypedExpr` node has `Some(T)`. Resolvable nodes are literals, variables, calls, binary expressions, and unary expressions. This includes operands inside other expressions. It also includes conditions, initializers, assignment targets, and return values inside statements.
    - For each program that an existing type-checker test gives to `check`, `check_typed` reports the same errors as `check` (FR-024).
- **SC-006**: All existing tests in `crates/descar-core/tests/` pass without change after the AST changes. The only exception is a test that uses an item with a breaking change that `MIGRATION.md` lists. A breaking change is one of these changes to an existing public `Expr` or `Stmt` field or variant: a new name, removal, or a new type. A new optional field or a new enum variant, with `#[non_exhaustive]` where necessary, is not a breaking change. It needs no migration path. The feature directory must contain a `MIGRATION.md` note for each breaking change of this feature.
- **SC-007**: For each `.dr` file in `dr_files/`, build the typed tree from the parsed program. The typed tree is the AST with a type attribute on each expression node.
    - The parsed program does not change.
    - `erase()` on the typed tree returns a tree that is equal to the parsed program in structure and in spans.
    - The number of `TypedExpr` nodes is the same as the number of `Expr` nodes.
    - The number of `TypedStmt` nodes is the same as the number of `Stmt` nodes.
- **SC-008**: For each `.dr` file in `dr_files/`, examine each `ElseBranch` value in the parsed program and in the typed tree.
    - Each `ElseBranch::ElseIf` has a `Stmt::If` payload. Each `ElseBranch::Block` has a `Stmt::Block` payload.
    - Each `TypedElseBranch::ElseIf` has a `TypedStmt::If` payload. Each `TypedElseBranch::Block` has a `TypedStmt::Block` payload.

  A test makes a `Stmt::If` by hand. Its `ElseBranch::ElseIf` has a `Stmt::Break` payload.
    - `TypedStmt::from_stmt` and `erase()` keep the payload kind.
    - The result of `erase()` is the same as the input in structure and in spans.
- **SC-009**: For each `.dr` file in `dr_files/` that has no syntax error, examine each `VarBinding` in the parsed program and each `TypedVarBinding` in the typed tree.
    - Each `VarBinding` has `Some` as its initializer.
    - Each `TypedVarBinding` has `Some` as its initializer.
    - No syntactically valid `.dr` file contains a `VarBinding` or `TypedVarBinding` with `None` as its initializer.
  A separate recovery test uses declarations with an initializer-count error.
    - `var a, b: i32 = 1` gives the error E2001. The AST has `a` with `Some` and `b` with `None`.
    - `var a: i32 = 1, 2` gives the error E2001. The AST has one `VarBinding` with `Some`. It does not store the initializer `2`.
    - `var a: i32;` is invalid. The parser reports the required syntax errors and E2001. The AST can have one `VarBinding` with `None`.
    - `var : i32 = 1;` gives the error E1008. The AST has no `Stmt::VarDeclaration`.
  A `None` initializer in a recovered AST is not a valid declaration state. The parser and later phases must not interpret it as a declaration form allowed by the language.

---

## Assumptions

- `JsavParser` and the existing token set in `TokenKind` fully define the grammar of the Descar language. No grammar extension is in scope. The extensions are structs, enums, generics other than `vector<T>`, lambdas, traits, and modules.
- The existing `SourceSpan` and `SourceLocation` types are sufficient for all required source ranges. The feature needs no new location infrastructure. A `SourceSpan` has an inclusive start and an exclusive end.
- The `Number` type in `crates/descar-core/src/tokens/number.rs` is the canonical representation of all numeric literal values. The lexer and the `LiteralValue::Numeric` variant of the AST share this type.
- `Type` has no `SourceSpan`. The parser represents an omitted function return annotation with `Type::Void`. This synthesized value has no source token and no span. The enclosing `Stmt::Function.span` contains the source range for the function.
- Explicit `void` and an omitted return annotation both produce `Type::Void`. The AST does not preserve the source origin of the value.
- `Type::Array` stores its size as `Box<Expr>`. Type equivalence evaluates the size expression to a non-negative integer and compares the resulting value. Source spans do not affect type equivalence.
- The resolved-type annotation (FR-014) is a typed tree (`TypedExpr` and `TypedStmt`). The typed tree has the same structure as `Expr` and `Stmt`, node by node. The build functions make it (FR-021). The `Expr` enum does not change. All existing parsing and printing code that makes or matches `Expr` variants without type information continues to work.
- The existing `TypeChecker` has no resolved-type annotation. `TypeChecker::check` returns a `Vec<CompileError>` and stores no type on a node. Its internal visit functions return an `Option<Type>` for each expression, and `check` does not keep it. No code and no test before this feature uses `resolved_type`, `TypedExpr`, or `TypedStmt`. The existing type-checker tests call `new`, `default`, `check`, `is_same_type`, and `get_size`. The table `TYPE_PROMOTION_CACHE` uses a pair of types as key. It is not a side table (FR-014).
- The children of `Expr` variants are `Expr` values, directly or inside `Box`, `Vec`, or `Option`. The children of `Stmt` variants are `Expr`, `Stmt`, `ElseBranch`, or `VarBinding` values in the same containers. FR-019 and FR-020 use these shapes. If the source code holds a child in a different shape, the same rule applies to that shape. In that case, the implementation plan must record the exact typed definition before the implementation starts.
- `LiteralValue`, `BinaryOp`, `UnaryOp`, `UnaryOpSide`, `Type`, `SourceSpan`, and identifier names implement `Clone`. The build functions and the `erase` functions need this. To add a missing `Clone` derive is not a breaking change under SC-006.
- After the build function runs and before type checking, each `resolved_type` is `None`. The model has no separate value for "not yet checked". To tell the two states apart, check if the type-checking pass has processed the tree.
- The `vector<T>` notation is the only generic type constructor in scope. The grammar has no other parameterized type syntax.
- `Stmt::Function` and `Stmt::MainFunction` are separate AST variants. The dedicated `main` syntax is represented by `Stmt::MainFunction`. An ordinary function declaration with `name = "main"` is represented by `Stmt::Function`.
- `Stmt::For.initializer` is `Option<Box<Stmt>>`. For parser output, a present initializer is either `Stmt::VarDeclaration` or `Stmt::Expression`. `Stmt::For.condition` and `Stmt::For.increment` are `Option<Expr>`.
- `Expr::Call.callee` is `Box<Expr>`. The AST does not require the callee to be an identifier.
- `Expr::Assign.target` is `Box<Expr>`. Valid parser output restricts the target to `Expr::Variable` and `Expr::ArrayAccess`.
- `Stmt::Return.value` is `Option<Expr>`. `None` represents a return statement without a value.
- `Stmt::Break` and `Stmt::Continue` contain only their `SourceSpan`.
- `Expr::Grouping` contains the parenthesized expression. Explicit grouping is therefore available to later compiler phases.
- All existing public APIs stay stable unless a migration is explicitly documented.
- The `specs/` directory did not exist before. This is the first feature specification in the project.
