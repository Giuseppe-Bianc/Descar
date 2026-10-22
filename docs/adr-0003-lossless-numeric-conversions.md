# ADR-0003: Lossless implicit numeric conversions

**Status:** Accepted
**Date:** 2026-10-22
**Deciders:** Descar language owner
**Applies to:** Descar Fully Typed Abstract Syntax Tree Specification
**Amends:** ADR-0001 D6, DEC-017 (new), GAP-015, TY-13, TY-14, Section 7.2, Section 7.3, Section 7.4, OPEN-001, Section 13.2 tests 2 and 3, and new test 28
**Numbering:** ADR-0001 reserves ADR-0002 for explicit conversions. This ADR does not take that number.

## Context

ADR-0001 D6 limits promotion to pairs where both operands convert to the result type under TY-13. ADR-0001 rejected option P1, which requires the specification to define sign reinterpretation and float rounding. The same ADR says that promotion defines no new run-time conversion.

TY-13 still contains four rows that round the source value:

| Row | Why the row loses values |
|---|---|
| I32 to F32 | F32 has a 24-bit significand. I32 has 31 magnitude bits. |
| U32 to F32 | F32 has a 24-bit significand. U32 has 32 magnitude bits. |
| I64 to F64 | F64 has a 53-bit significand. I64 has 63 magnitude bits. |
| U64 to F64 | F64 has a 53-bit significand. U64 has 64 magnitude bits. |

TR-028a requires a literal to be exactly representable in an F32 or F64 context. The two rules disagree:

| Program | Result before this ADR |
|---|---|
| `var f: f32 = 16777217` | E2035 |
| `var n: i32 = 16777217` then `var f: f32 = n` | Accepted before ADR-0003. The conversion could lose precision; the exact runtime result is deferred to the execution specification (OPEN-001). |
| `a + b`, `a: i64`, `b: f32` | E2036 |
| `a + b`, `a: i32`, `b: f32` | Accepted with type F32. The value of `a` can change. |

The boundary between the last two rows follows no value-preservation rule. OPEN-001 listed the conversion of a non-exact integer to a floating-point type as undefined run-time behavior. That entry records the defect and does not resolve it.

## Decision

An implicit numeric conversion MUST preserve every value of the source type. TY-13 loses the four rows above. Section 7.2 states the test for each source and target category. The exactness test in TR-028a is the same rule applied to one literal, and it does not change.

After this decision:

- TY-13 contains exactly the numeric pairs that pass the test. TY-13 verification checks every ordered numeric pair against the test.
- 68 ordered pairs have a promoted type. 32 have none. The 16 unordered pairs without a promotion are: F32 with I32, I64, U32, or U64; F64 with I64 or U64; U64 with I64, I32, I16, or I8; U32 with I32, I16, or I8; U16 with I16 or I8; U8 with I8.
- I64 and U64 have no outgoing implicit conversion.
- NumericAssignable conversions never round. OPEN-001 no longer lists integer-to-float conversion.

## Options Considered

| Option | Effect |
|---|---|
| L1: remove the four rows (chosen) | One rule covers literals and variables. Lowering defines no integer-to-float rounding. 32 ordered pairs have no promotion. |
| L2: keep the four rows and drop the exactness test in TR-028a | Keeps every program that the current table accepts. Lowering must define the rounding mode of integer-to-float conversion. This is the work that ADR-0001 rejected in option P1. |
| L3: keep both rules as they are | Reports a diagnostic for a literal and none for a variable that holds the same value. Rejected. |

Rust uses the same boundary in its `From` implementations: i8, i16, u8, and u16 convert to f32, and i8, i16, i32, u8, u16, and u32 convert to f64. It has no `From` implementation from i32 to f32 or from i64 to f64.

## Consequences

**Rejected after the change:**

| Program | Before | After |
|---|---|---|
| `var f: f64 = n`, `n: i64` or `n: u64` | Convert to F64 | E2002 |
| `var f: f32 = n`, `n: i32` or `n: u32` | Convert to F32 | E2002 |
| `var f: f64 = 7 / 2` | I64 division, then Convert to F64. Value 3.0. | E2002 |
| `var f: f64 = 2 * 3` | I64 product, then Convert to F64 | E2002 |
| `x * (2 + 3)`, `x: f64` | F64 | E2036 |
| `1 + 2.5`, `2 * 3.14`, `var f: f64 = 1 + 2.5` | F64 | E2036. The integer literal has type I64 before it meets the F64 literal. See OPEN-006. |
| `a + b`, `a: i32`, `b: f32` | F32 | E2036 |
| `a + b`, `a: i64`, `b: f64` | F64 | E2036 |
| `x += y`, `x: f32`, `y: i32` | Convert I32 to F32 | E2002 |

**Unchanged:** `var f: f64 = n` with `n: i32` or `n: u32`; `var f: f32 = n` with `n: i16` or `n: u16`; `x + 1` with `x: f64`; `var f: f64 = 7`; every literal test in ADR-0001.

**Harder:** No source form converts I64 or U64 to a floating-point type. The language already has no source form that narrows a non-literal value. This ADR adds the 64-bit integer to floating-point direction to that limit. ADR-0001 reserves ADR-0002 for explicit conversions, and this ADR makes that decision necessary for any program that mixes a 64-bit integer with a floating-point value. An unsuffixed integer literal that is the whole initializer, argument, or peer operand still adapts to a floating-point type under TR-028a. A suffixed I64 or U64 literal does not, because its type is fixed.

**Known gap, OPEN-006:** Two literal operands of different kinds, such as `1 + 2.5`, have no promotion, because TR-028c passes no floating-point context through an operator and the integer literal defaults to I64. The earlier table accepted this program only through the I64 to F64 row. A complete fix needs the integer literal to adapt to the floating-point literal. Evaluating a flexible expression as a constant, and then checking the value against the context type, does this for every case. This ADR does not change TR-028.

**Checker change:** Remove the four rows from the assignability table. No other checker rule changes, because TR-009, TR-010, TR-013, TR-014, TR-015, TR-017, and TR-023 already refer to TY-13.

## Spec Amendments

Applied in this revision of the specification:

- Revision line, BAS-010, DEC-003: reference DEC-017.
- GAP-015: record the four rounding rows and require the lossless table.
- TY-13: requirement text and verification. Section 7.2: table rows, the value-preservation test, and the explanation of the removed rows.
- Section 7.3: 68 and 16 replace 76 and 12. TY-14: 68 and 32 replace 76 and 24.
- Section 7.4: NumericAssignable states that it preserves every value.
- OPEN-001: remove integer-to-float conversion.
- Section 13.2: extend tests 2 and 3, and add test 28.
- DEC-017: record this decision.
