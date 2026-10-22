# ADR-0001: Literal typing and compound assignment

**Status:** Accepted. Amended in part by ADR-0003: the I32 and U32 to F32 rows and the I64 and U64 to F64 rows leave TY-13, and the pair counts in D6 become 68 and 32.
**Date:** 2026-10-18
**Deciders:** Descar language owner; implementer of the parser and type checker
**Applies to:** Descar Fully Typed Abstract Syntax Tree Specification
**Amends:** BAS-010, DEC-003, TR-001, TR-009, TR-010, TR-012, TR-013, TY-13, TY-14, Section 7.3, Section 7.4, Section 8.1, Section 8.4, Section 8.8, Section 10.1, N-SE-01, N-TE-01, N-TE-05, N-TE-06, INV-016, ACC-007, ERR-009, OPEN-001, Example 14.2, Section 13.2 tests 1 to 3 and 16

## Context

TR-001 gives every unsuffixed numeric literal a 64-bit type: I64, U64 or F64. TY-13 has no edge from I64 to a narrower integer, from I64 to F32, or from F64 to F32. TR-014, TR-015, TR-017 and TR-023 require the value type to be assignable to the target type. DEC-006 and TR-013 require the result of a compound operation to be assignable to its place. That last rule is correct and stays. Together these rules reject ordinary programs and accept programs that change values.

### Programs rejected by the current rules

| Program | Rule chain | Result |
|---|---|---|
| `var a: i32 = 5` | TR-001 types `5` as I64. TY-13 has no I64 to I32 edge. TR-015 needs S ⪯ T. | E2002 |
| `var f: f32 = 1.5` | `1.5` is F64. TY-13 has no F64 to F32 edge. | E2002 |
| `var v: i32[3] = {1, 2, 3}` | TR-018 gives Array(I64,3). The array rule needs I64 ⪯ I32. | E2002 |
| `i += 1`, `i: i32` | TR-013 promotes I32 and I64 to I64. The store needs I64 ⪯ I32. | E2002 |
| `n += 1`, `n: u32` | Promote(U32,I64) is I64. The store needs I64 ⪯ U32. | E2002 |
| `x *= 2.0`, `x: f32` | Promote(F32,F64) is F64. The store needs F64 ⪯ F32. | E2002 |
| `var m: u64 = 1 << 40` | TR-012 gives I64. TY-13 has no I64 to U64 edge. | E2002 |
| `b <<= k`, `b: i8`, `k: u64` | TR-012 promotes both operands to U64. The store needs U64 ⪯ I8. | E2002 |

### Programs accepted by the current rules that change values

| Program | Rule chain |
|---|---|
| `-1i32 < 1u32` | Promote(I32,U32) is U32. The operand -1 becomes 4294967295. OPEN-001 leaves the conversion undefined. |
| `a + b`, `a: i64`, `b: f32` | Promote(I64,F32) is F32. The I64 operand is rounded to a 24-bit significand. |
| `1i32 + 2u64` (Example 14.2) | Promote(I32,U64) is U64. A negative I32 operand changes value. |

Every numeric literal used as a value in Examples 14.1 to 14.4 carries a width suffix. The specification contains no program in which an unsuffixed literal feeds a narrower type. The Number definition in Section 3 has no I64 or U64 variant, so `Integer(i64)` is the value of both `5` and `5i64`. The Surface AST cannot tell them apart.

### Forces

- Section 7.4 states that explicit cast syntax is not supported. No operation narrows a value. A narrow variable receives a value only from a value of its own type or a narrower type, or from a literal.
- GAP-009 and OPEN-004 record that no typed-AST consumer exists. A change costs specification text and checker code only.
- DEC-007 excludes a constant-expression evaluator. BAS-014 excludes inference state from the typed tree. DEC-014 requires every conversion to be explicit.
- OPEN-001 leaves run-time behavior undefined. A rule that changes the width at which an operation runs changes its overflow behavior, so Consequences lists each such change.

## Decision

Unsuffixed numeric literals take their type from context. Numeric promotion is limited to pairs that preserve every integer value. A shift takes the type of its left operand. A compound assignment runs in the type of its place.

### D1. Suffix record

N-SE-01 Literal gains the field `width_suffix: bool`. It is true when the source token carries a width suffix. An *adaptable literal* is a numeric Literal with `width_suffix` false.

### D2. Default type

The TR-001 defaults apply when TR-028 supplies no context type or does not adapt the literal. Integer is I64, UnsignedInteger is U64 and floating point is F64. A literal with `width_suffix` true always has the type of its Number variant.

### D3. Context typing of literals (new rule TR-028)

Definitions:

- A *flexible expression* is one of:
  1. an adaptable literal;
  2. a Grouping of a flexible expression;
  3. a prefix Negate or BitwiseNot of a flexible expression;
  4. a Binary node with operator `+ - * / % & | ^` whose two operands are flexible;
  5. a Binary node with operator `<<` or `>>` whose left operand is flexible. The right operand can be any expression.
- A *context type* is a type that the position of an expression supplies to it. The positions are:
  1. the declared type, for a binding initializer;
  2. the place type, for an assignment value;
  3. the function result type, for a return value;
  4. the parameter type, for a call argument;
  5. the element type E, for each element of an array literal that has context type Array(E,n). When E is an array type, it is the context type of nested array literals;
  6. the place type, for the value of a compound assignment other than `<<=` and `>>=`.

Rules:

- **TR-028a. Leaf adaptation.** An adaptable literal that receives a numeric context type T takes type T when its signed value v (after D4) passes the test for its kind:
  - an integer literal and an integer T: v is within the range of T;
  - an integer literal and F32 or F64: v is exactly representable in T;
  - a floating-point literal and F32 or F64: the value is finite after rounding to nearest in T.

  If the literal is compatible with T by kind and fails its test, the checker reports E2035 at the literal. A floating-point literal with an integer T is not compatible by kind. It keeps its default type, and this rule reports nothing.
- **TR-028b. Passing.** Only a flexible expression passes its context type to its children. Grouping passes any numeric T. A Negate, a BitwiseNot and the Binary operators `+ - * / % & | ^` pass an integer T to every operand. A shift passes an integer T to its left operand only. An operator node does not pass F32 or F64. Operand and result types then follow TR-009 to TR-012 as usual.
- **TR-028c. Peer rule.** For `+ - * / % & | ^ == != < <= > >=`, if exactly one operand is flexible and the other operand is not flexible and has numeric type S, the flexible operand receives S as its context type. This replaces any context type from the position of the Binary node. If both operands are flexible, both receive the context type of the Binary node. Shifts have no peer rule.
- **TR-028d. No context.** A flexible expression with no context type takes default types, and operators apply TR-009 to TR-012 to those types. Index expressions, conditions, shift counts and operands of non-flexible unary operators receive no context type. A non-numeric context type has no effect.

Formal form, with `Γ ⊢ e ⇐ T ⇒ S` meaning that e is checked with context type T and has type S:

`e` is an adaptable literal, `T` is numeric, and `v` passes the test of TR-028a `⇒ Γ ⊢ e ⇐ T ⇒ T`.

### D4. Negation folding

A prefix Negate applied directly to an adaptable literal MUST be one Literal node in the typed tree. The node holds the negated value, and its span covers both tokens. The most negative value of a signed type has no positive counterpart in that type, so `-128` for I8 cannot be a Negate over a Literal of type I8. A Negate applied to any other expression, including `-(128)` and `- -5`, is a Unary node. A suffixed literal is not folded.

### D5. Typed literal representation

The Number variant of a typed numeric Literal corresponds to its result_type: I64 to Integer, U64 to UnsignedInteger, F64 to Float64, and F32 to Float32. A Scientific form keeps its form at the new width. The value MUST be representable in result_type. INV-022 states this.

### D6. Promotion limit

TR-009, the numeric branch of TR-010, and TR-012 for `& | ^` select C = Promote(T1,T2) as in Section 7.3 and then require T1 ⪯ C and T2 ⪯ C. If either relation fails, the checker reports E2036 and the operation has no result type. When both hold, each operand whose type is not C receives a NumericAssignable conversion to C.

TY-13 receives six rows that preserve every value: U8 to I16, I32 and I64; U16 to I32 and I64; U32 to I64. Without them no assignment or promotion converts an unsigned value to a signed type.

After D6 these twelve pairs have no promotion, in either operand order: F32 with I64; F32 with U64; U64 with I64, I32, I16 or I8; U32 with I32, I16 or I8; U16 with I16 or I8; U8 with I8. The other 76 ordered pairs of the 100 have the result that Section 7.3 selects. Every promotion is now a pair in TY-13, so NumericPromotion is removed from Section 7.4.

### D7. Shifts

For `<<` and `>>`, both operands MUST have integer types. The result type is the type of the left operand. The right operand keeps its own integer type and receives no conversion.

### D8. Compound assignment

TR-013 is restated. For `+= -= *= /= %= &= |= ^=`:

1. The target is a mutable Place of type T. T is in Num for the arithmetic operators and in Int for the bitwise operators.
2. The value is checked with context type T under TR-028.
3. The value type S satisfies S ⪯ T. The value receives a conversion to T when S is not T.
4. The operation runs in T. The expression type is T.

For `<<=` and `>>=`, T and S are integer types, the value is not converted, and the expression type is T.

Formal form:

`Γ ⊢ p:Place(T)`, `Mutable(p)`, `T∈Num` (`T∈Int` for the bitwise operators), `Γ ⊢ e ⇐ T ⇒ S`, and `S⪯T ⇒ Γ ⊢ p op= e : T`. For `<<=` and `>>=`: `T∈Int`, `Γ ⊢ e:S`, and `S∈Int ⇒ Γ ⊢ p op= e : T`.

Diagnostics: E2002 when S ⪯ T fails, E2013 or E2011 for operands of the wrong class as in the matching binary rule, E2024 for an immutable target, and E2033 for a non-place target.

Under D6, the earlier procedure (promote, then check that the result converts back to T) accepts exactly the pairs with S ⪯ T. This was checked over all 100 ordered pairs of numeric types. D8 therefore removes the target conversion, the store conversion and the separate operation type without changing which non-literal programs are accepted.

## Options Considered

### Option A: Keep the rules and require suffixes

| Dimension | Assessment |
|---|---|
| Complexity | Low. No change. |
| Cost | None to the specification. Every narrow literal needs a suffix. |
| Scalability | `i += 1i32` and `x *= 2.0f32` work. Array literals need a suffix on every element. |
| Precedent | The examples in Section 14 already follow it. |

**Pros:** No checker change. A missing suffix produces an error, not a wrong value.
**Cons:** An unsuffixed literal can never initialize or combine with a type narrower than 64 bits. The suffix set must contain every width.

### Option B: Contextual typing of unsuffixed literals (chosen)

| Dimension | Assessment |
|---|---|
| Complexity | Medium. One new predicate, one context parameter in the checker, one peer rule. |
| Cost | Four new rule paragraphs, one new field, two diagnostics. |
| Scalability | Works for declarations, assignments, returns, arguments and array elements. Needs no new type in the TypeTable. |
| Precedent | Rust and Go reject an out-of-range literal at compile time. |

**Pros:** Removes every rejection in the first table without an inference state. Keeps the typed tree free of untyped types.
**Cons:** The type of a literal depends on its position. `x + 1` changes width for narrow `x`.

### Option C: Untyped constant types

| Dimension | Assessment |
|---|---|
| Complexity | High. Adds IntLiteral and FloatLiteral to the type system and a constant evaluator. |
| Cost | Conflicts with DEC-007 and BAS-014. Needs a defaulting pass before the typed tree is built. |
| Scalability | Folds `1 << 40` and `100 + 100` exactly. |
| Precedent | Go. |

**Pros:** Detects overflow of constant expressions.
**Cons:** Adds a type category that must not survive into the typed tree, and a second evaluation model that duplicates what lowering will do.

### Option D: Implicit narrowing in the assignability table

| Dimension | Assessment |
|---|---|
| Complexity | Low. Add I64 to every narrower integer and F64 to F32. |
| Cost | Silent truncation of every non-literal value. |
| Scalability | Constant. |
| Precedent | The C language. |

**Pros:** Smallest change.
**Cons:** Reverses DEC-006 and every narrowing check in the specification.

### Option E: Explicit cast syntax only

| Dimension | Assessment |
|---|---|
| Complexity | Medium. New grammar production, new node, new conversion kinds. |
| Cost | Every narrow literal becomes `5 as i32`. |
| Scalability | Solves narrowing for values, which Option B does not. |
| Precedent | Rust, Go. |

**Pros:** Required eventually, since no non-literal value can be narrowed.
**Cons:** Does not remove the literal burden. It is a separate decision and is listed under Action Items.

### Promotion (D6)

| Option | Effect |
|---|---|
| P1: keep the preference order and define conversion semantics for every pair | Keeps all 100 pairs. The specification must then define sign reinterpretation and float rounding for 24 value-changing pairs. |
| P2: require both operands to convert to the result under TY-13 (chosen) | Rejects 24 ordered pairs. Needs six lossless table rows. Adds no conversion semantics. |
| P3: least upper bound under TY-13 | Accepts I8 with U8 as I16. I32 with U32 has two minimal upper bounds, I64 and F32, so it needs a tie-break rule. |

P2 is chosen. It keeps the Section 7.3 order, adds one precondition, and defines no new run-time conversion.

### Compound assignment (D8)

| Option | Effect |
|---|---|
| K1: keep TR-013 with promote-then-store | Same accepted programs as D8 after D6. Keeps three extra fields and INV-016 complexity. |
| K2: store with implicit narrowing | Accepts `x_i8 += 2i16`. Reverses DEC-006 and GAP-003. |
| K3: operate in the place type (chosen) | Same accepted programs as K1. Removes three fields and the separate operation type. |

## Trade-off Analysis

1. **Width changes.** `x + 1` with `x: i32` now has type I32. It had type I64. OPEN-001 defines no overflow behavior, so no defined behavior is lost. A program that sums values near the I32 limit now overflows at 32 bits when lowering defines overflow. The change appears in the migration list below.
2. **Error, not fallback, for an out-of-range literal.** `x_i8 + 300` reports E2035. The earlier checker accepted it and widened to I64. In Rust and Go the same source is a compile error. A fallback would make the type of `300` depend on whether it fits, which is the behavior D3 removes.
3. **Float contexts do not pass through operators.** With `var f: f64 = 7 / 2`, passing F64 to the literals would turn integer division into float division and give 3.5 where the earlier rules give 3.0. The cost is that `var f: f32 = 0.5 * 2.0` stays rejected, as it is today. A bare literal still adapts, so `var f: f32 = 1` and `x_f32 *= 2` are accepted.
4. **No constant evaluation.** `var x: i8 = 100 + 100` is accepted because each literal fits I8. The sum overflows at run time. Detecting it needs Option C.
5. **Narrowing stays impossible for values.** D6 rejects 24 ordered pairs. A program can widen with an assignment to a wider variable, which works because of the six new table rows. No program can narrow a non-literal value. This limit exists today and needs ADR-0002.
6. **Tree shape.** D4 removes one Unary node per negated literal. `-128i8` stays unwritable because the Number variant I8(i8) cannot hold 128.

## Consequences

**Easier:**

- Every program in the first table is accepted, with the types shown in the vectors below.
- A compound assignment has one typing rule, and the typed node loses `operation_type`, `target_conversion` and `store_conversion`.
- The lowerer sees one conversion kind for numeric widening instead of two with the same source and target pairs.

**Harder:**

- The checker needs the flexible-expression predicate and a context parameter on every expression visit.
- The lexer must record the suffix and must keep the magnitude of unsuffixed integers up to u64::MAX, so that `-9223372036854775808` can fold.

**Migration, programs whose result changes:**

- Rejected after the change: the 12 pairs in D6; `x_f32 * (2 + 3)`, which reports E2036; `x_i8 + 300`; `var f: f64 = 9007199254740993`; Example 14.2 as written.
- Typed differently after the change: `x_i32 + 1` has type I32; `x_u8 == 5` compares in U8.
- Accepted after the change: every program in the first table.

**To revisit:**

- ADR-0002, explicit conversions, to narrow non-literal values.
- OPEN-001, run-time arithmetic, after the lowering specification exists.
- Negated suffixed literals such as `-128i8`, if the lexer is changed to keep magnitudes for suffixed literals.

## Spec Amendments

Apply these when the ADR is accepted.

**Section 3.** Add terms: *adaptable literal*, *flexible expression*, *context type*, *default type*, each as defined in D1 to D3.

**N-SE-01 Literal.** Fields: `value: LiteralValue; width_suffix: bool; node_id; span`.

**N-TE-01 Literal.** Semantic function: "The Number variant of value corresponds to result_type (D5). A prefix Negate applied directly to an adaptable literal is this node with the negated value (D4)."

**TR-001 Literal.** "A numeric literal with `width_suffix` true has the type of its Number variant. A numeric literal with `width_suffix` false takes the type that TR-028 selects. When TR-028 gives no context type or does not adapt the literal, Integer is I64, UnsignedInteger is U64 and floating point is F64." Possible error: "E2035 from TR-028. An invalid literal is a lexer or parser error."

**TR-028 Literal context.** Add Section 8.2 row with TR-028a to TR-028d from D3. Possible error: E2035.

**TR-009, TR-010, TR-012.** Append to each preconditions cell: "Both operands MUST be assignable to the promoted type. Otherwise E2036." For TR-012 add: "For `<<` and `>>` the result type is the type of the left operand. The right operand has any integer type and is not converted."

**TR-013 Compound assignment.** Replace with D8. Delete "Convert the read target and value to operation_type. The operation result must be assignable to the place type."

**Section 8.4, paragraph after the rules table.** Replace the first three sentences, "TR-013 is a specified correction. It rejects a compound assignment when the promoted result cannot convert to the destination type. For example, an I16 result cannot be stored in an I8 target under the conversion table.", with "TR-013 requires the value to be assignable to the place type. For example, an I16 value cannot be applied to an I8 target." Keep the sentence "It also requires a valid mutable place."

**Section 8.4, diagnostics paragraph.** Replace "E2002 when the operation result cannot convert back to that target" with "E2002 when the value cannot convert to the place type, E2035 when a literal does not fit its context type, and E2036 when numeric operands have no common type".

**Section 8.1 and Section 8.8.** In the text after the TR-009 rule, replace "NumericPromotion" with "NumericAssignable". Add the premises `T₁⪯C` and `T₂⪯C` to the TR-009 rule, to the TR-009, TR-010 and TR-012 rows of Section 8.8, and drop the promotion of the right operand for `<<` and `>>` in the TR-012 row. Replace the TR-013 row with the D8 formal form. Add a TR-028 row with the D3 formal form.

**TY-13 table.** Add rows: U8 to I16, I32, I64; U16 to I32, I64; U32 to I64.

**Section 7.3.** Replace the paragraphs after the preference order with: "For two numeric operand types, Promote(T1,T2) selects the first type C in this list that is equal to either operand. The promotion is defined only when T1 ⪯ C and T2 ⪯ C. Otherwise the operation reports E2036. Promotion uses only conversions in TY-13. The order does not define arithmetic overflow, shift-count range or floating-point rounding."

**TY-14 verification.** "Exhaust all 100 ordered numeric type pairs. Assert the selected TypeId for 76 pairs and E2036 for the 24 ordered pairs listed in D6."

**Section 7.4.** Delete the NumericPromotion row. NumericAssignable: "Source and target are numeric and the pair is in TY-13. The kind is used for assignment conversions and for operand conversions under D6."

**N-TE-05 Binary.** Append: "For `<<` and `>>`, result_type equals left.result_type, and right has an integer type with no Convert node."

**N-TE-06 CompoundAssign.** Fields: `target: Place; operator: CompoundOperator; value: TypedExpr; result_type: TypeId`. Semantic function: "The target is a mutable Place of type T. The operation runs in T. For operators other than `<<=` and `>>=`, value.result_type equals T, and a Convert child exists when the source type differs. For `<<=` and `>>=`, value has an integer type and no Convert child. result_type equals T."

**INV-016.** Replace with: "A CompoundAssign has result_type equal to target.place_type. For operators other than `<<=` and `>>=`, value.result_type equals target.place_type. For `<<=` and `>>=`, value.result_type is an integer type." Stage and action unchanged.

**INV-022 (new).** "A numeric Literal has a Number variant that corresponds to its result_type and a value representable in that type. | Typed AST validation | Internal error."

**INV range.** Extend "INV-001 through INV-021" to "INV-001 through INV-022" in Section 10.1, ACC-007 and Section 13.1.

**ERR-009.** Append: "E2035 identifies an unsuffixed numeric literal whose value does not fit its context type. E2036 identifies numeric operands that have no common type under TR-009, TR-010 or TR-012."

**GAP table.** Add:

| ID | Observed behavior | Required consequence |
|---|---|---|
| GAP-014 | TR-001 types every unsuffixed literal with a 64-bit type, and TY-13 has no narrowing edge. An unsuffixed literal cannot initialize or combine with a narrower type. | The new checker MUST apply TR-028. |
| GAP-015 | Promotion selects the first type in the preference order without checking that the operands keep their values. Shifts promote both operands. | The new checker MUST apply D6 and D7. |

**BAS-010 and DEC-003.** Replace "DEC-006 through DEC-008" with "DEC-006 through DEC-008 and DEC-016". DEC-003 consequence: "Existing promotion, assignability and operator behavior is recorded, except where DEC-016 replaces it."

**DEC-016 (new).**

| ID | Decision | Rationale | Requirement affected | Consequence |
|---|---|---|---|---|
| DEC-016 | Unsuffixed numeric literals take a context type. Promotion requires both operands to convert to the result by TY-13. Shifts take the left operand type. Compound assignment runs in the place type. Alternatives are in ADR-0001. | Without context typing no literal reaches a type narrower than 64 bits. Without the promotion limit, promotion changes values for 24 ordered pairs. | TR-001, TR-009, TR-010, TR-012, TR-013, TR-028, TY-13, TY-14, N-SE-01, N-TE-01, N-TE-05, N-TE-06 | A program can change type or be rejected. ADR-0001 lists the cases. |

**OPEN-001.** Replace the issue text with: "Run-time behavior of arithmetic overflow, division by zero, shift counts outside the operand width, integer-to-float conversion of a non-exact value, and rounding of floating-point literals to F32 is not defined. After ADR-0001, no promotion or compound assignment changes an integer value."

**Example 14.2.** Replace with:

~~~text
main {
    var value: u64 = 1u32 + 2u64
}
~~~

Required semantic result: the operands have types U32 and U64. Promote selects U64, and U32 ⪯ U64 holds. The U32 operand has an explicit NumericAssignable conversion to U64. The binary expression and the initializer have type U64.

**Example 14.2a (new).** `var value: u64 = 1i32 + 2u64` selects U64, and I32 ⪯ U64 does not hold. The checker reports E2036 and returns no Fully Typed AST.

**Example 14.7 (new).**

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

Required semantic result: `a` receives Literal I32(5). `b` receives one Literal I8(-128) and no Unary node. `c` receives Literal F32(1). `d` has type Array(I32,2) with two I32 literals. In `e`, the left literal has type U64, the right literal has type I64, and the Binary node has type U64. In `a += 1` the value is Literal I32(1) with no Convert node. In `c *= 2.0` the value is Literal F32(2).

**Example 14.8 (new).**

~~~text
main {
    var a: i8 = 128
    var b: u8 = -1
    var c: i32 = 1.5
}
~~~

The checker reports E2035 for `a`, E2035 for `b`, and E2002 for `c`. It returns no Fully Typed AST.

**Section 13.2 tests.** Replace tests 1 to 3 and 16, and add tests 23 to 27:

1. Every numeric type in literal typing, with and without a suffix, with and without a context type.
2. Every assignment conversion in TY-13, including the six added rows, and the adjacent disallowed conversion in both directions.
3. All 100 ordered numeric type pairs: the selected type for 76 and E2036 for 24.
16. Compound assignment with each vector in the Compound table below.
23. The minimum, the maximum, and each value one outside the range, for all eight integer types, as initializers and as peer operands.
24. F32 overflow, and integer exactness at 2^24 and 2^53 with the neighboring values.
25. Negation folding: `-128` for I8, `-9223372036854775808` for I64, `-(128)` for I8, and a suffixed literal.
26. The peer rule with the literal on either side, and with a flexible operator tree on one side.
27. Context passing through array literals, nested array literals, call arguments, return values, and shifts with a non-literal count.

## Verification Vectors

Types in the Result column are typed-tree types. A bare diagnostic code means the checker returns no Fully Typed AST.

### Literals

| Source | Result |
|---|---|
| `var a: i32 = 5` | Literal I32(5) |
| `var a: i8 = 127` | Literal I8(127) |
| `var a: i8 = 128` | E2035 |
| `var a: i8 = -128` | one Literal I8(-128) |
| `var a: i8 = -129` | E2035 |
| `var a: u8 = 255` | Literal U8(255) |
| `var a: u8 = 256` | E2035 |
| `var a: u8 = -1` | E2035 |
| `var a: i64 = -9223372036854775808` | one Literal I64 |
| `var a: i64 = 9223372036854775808` | E2035 |
| `var a: u64 = 18446744073709551615` | Literal U64 |
| `var f: f32 = 1.5` | Literal F32(1.5) |
| `var f: f32 = 1e38` | Literal F32 |
| `var f: f32 = 1e39` | E2035 |
| `var f: f32 = 1` | Literal F32(1) |
| `var f: f32 = 16777216` | Literal F32 |
| `var f: f32 = 16777217` | E2035 |
| `var f: f64 = 9007199254740992` | Literal F64 |
| `var f: f64 = 9007199254740993` | E2035 |
| `var i: i32 = 1.5` | E2002 |
| `var s: string = 5` | E2002 |
| `var x: i32 = 1 + 2` | Binary I32 with two I32 literals |
| `var x: i8 = 100 + 100` | accepted, Binary I8 |
| `var x: i8 = 100 + 200` | E2035 at `200` |
| `var m: u64 = 1 << 40` | left U64, right I64, Binary U64 |
| `var m: u64 = 1 << n`, `n: u64` | left U64, Binary U64 |
| `var m: u64 = (1 << n) \| (1 << k)` | Binary U64 |
| `var u: u8 = ~0` | BitwiseNot over Literal U8(0), type U8 |
| `var f: f64 = 7 / 2` | I64 division, then I64 to F64 conversion. Value 3.0 |
| `var f: f32 = 7 / 2` | E2002 |
| `var f: f32 = 0.5 * 2.0` | E2002 |
| `var v: i32[3] = {1, 2, 3}` | Array(I32,3) |
| `var v: i32[2][3] = {{1,2,3},{4,5,6}}` | Array(Array(I32,3),2) |
| `var v: i8[2] = {1, 200}` | E2035 at `200` |
| `{1i8, 2}[0]`, no context type | E2021, unchanged |

### Peer operands

| Source | Result |
|---|---|
| `x + 1`, `x: i32` | I32 |
| `x + 300`, `x: i8` | E2035 |
| `x + 1.5`, `x: i32` | F64 |
| `x + -1`, `x: u32` | E2035 |
| `x == 5`, `x: u8` | Bool, compared in U8 |
| `x * 2`, `x: f32` | F32 |
| `x * 2.0`, `x: f32` | F32 |
| `x * (2 + 3)`, `x: f32` | E2036 |
| `1 + 2.5` | F64 |
| `1u32 + 2u64` | U64 with conversion on the left operand |
| `1i32 + 2u64` | E2036 |
| `a + b`, `a: u32`, `b: i64` | I64 with conversion on `a` |
| `a + b`, `a: i64`, `b: f32` | E2036 |
| `a + b`, `a: i8`, `b: u8` | E2036 |
| `a < b`, `a: i32`, `b: u32` | E2036 |

### Compound assignment and shifts

| Source | Result |
|---|---|
| `i += 1`, `i: i32` | value Literal I32(1), no Convert |
| `x *= 2.0`, `x: f32` | value Literal F32(2) |
| `n += 1`, `n: u32` | value Literal U32(1) |
| `n += -1`, `n: u8` | E2035 |
| `x += 2i16`, `x: i8` | E2002 |
| `x += y`, `x: i64`, `y: i32` | Convert I32 to I64 |
| `x += y`, `x: i32`, `y: i64` | E2002 |
| `x += y`, `x: f32`, `y: i32` | Convert I32 to F32 |
| `x += y`, `x: f32`, `y: i64` | E2002 |
| `x += y`, `x: i32`, `y: u16` | Convert U16 to I32 |
| `x += y`, `x: i32`, `y: u32` | E2002 |
| `x += 1.5`, `x: i32` | E2002 |
| `x &= 255`, `x: i8` | E2035 |
| `x &= 255`, `x: i32` | accepted |
| `s += 1`, `s: string` | E2013 or E2016, as for `s + 1` |
| `x <<= k`, `x: i8`, `k: u64` | accepted, no Convert, type I8 |
| `x <<= 3`, `x: i32` | accepted, count Literal I64(3) |
| `x << 3`, `x: u8` | U8 |
| `x << k`, `x: i8`, `k: u64` | I8 |
| `x << 1.5`, `x: i32` | E2011 |
| `x <<= 1`, `x: f32` | E2011 |

## Action Items

1. [ ] Lexer: record `width_suffix`. Confirm that the suffix set contains i64, u64 and f64, because a suffix is the only way to write a wide literal in a narrow context. Keep the magnitude of unsuffixed integers up to u64::MAX.
2. [ ] Surface AST: add `width_suffix` to N-SE-01.
3. [ ] Checker: add the flexible-expression predicate, the context parameter, the peer rule, negation folding, E2035 and E2036.
4. [ ] Checker: add the six TY-13 rows, the Promote precondition and the shift rule. Remove NumericPromotion.
5. [ ] Typed AST: change N-TE-01, N-TE-05 and N-TE-06. Replace INV-016. Add INV-022.
6. [x] Specification: apply the amendments above and renumber nothing. All new IDs append to existing ranges. Applied on 2026-10-18.
7. [ ] Tests: implement the vectors above as checker tests, and the 100-pair promotion test with its two counts.
8. [ ] Write ADR-0002 for explicit conversions, since no non-literal value can be narrowed after this ADR.
