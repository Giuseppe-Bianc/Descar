# ADR-0005: Runtime value semantics of explicit casts

**Status:** Proposed
**Date:** 2026-10-28
**Deciders:** Descar language owner
**Applies to:** Descar Fully Typed Abstract Syntax Tree Specification, and the lowering and execution specifications that consume it
**Amends:** ADR-0002 (the paragraph that leaves runtime behavior unspecified), DEC-020, OPEN-001, Section 7.4, Section 8.5, Section 14.10, README language syntax
**Numbering:** ADR-0002 stays unchanged except for its status line. This ADR completes it.

## Context

ADR-0002 lets a program cast between numeric types, `char`, `bool`, `string`, and null aggregates. It states that integer narrowing, out-of-range floating-point-to-integer conversion, floating-point rounding, numeric-to-`char`, numeric-to-`bool`, and the representation of null aggregates have no runtime meaning yet. The type checker accepts all of these casts. No later phase can execute them.

ADR-0003 rejected lossy implicit conversions because a rounding conversion needs a runtime definition. Explicit casts need that definition now, for a larger set of pairs.

`char` has no defined value set. TY-05 says only "primitive character type". The lexer accepts one source character or one escape sequence between single quotes, and the source encoding is UTF-8. A rule for converting a number to a `char` cannot be written until the value set is fixed.

## Decision

**Totality.** Every cast that the type checker accepts has exactly one defined result for every source value. No cast traps, aborts, or has undefined behavior. This ADR does not decide arithmetic overflow, division by zero, or shift counts (OPEN-001). Those rules do not affect casts.

**D1. Char.** A `char` holds one Unicode scalar value: U+0000 to U+D7FF or U+E000 to U+10FFFF.

**D2. Integer to integer.** The result is the value of the target type that is congruent to the source value modulo 2^N, where N is the width of the target. Widening from a signed source extends the sign. Widening from an unsigned source extends with zeros. Narrowing keeps the low-order bits. A conversion between signed and unsigned types of equal width keeps the bit pattern.

**D3. Integer to floating point.** The result is the nearest value of the target type. A tie goes to the value with an even significand. No integer type overflows F32 or F64.

**D4. Floating point to floating point.** F32 to F64 is exact. F64 to F32 gives the nearest F32 value, with ties to even. When the rounded magnitude exceeds the largest finite F32 value, the result is infinity with the same sign. Infinity stays infinity. NaN stays NaN. The sign of zero is kept.

**D5. Floating point to integer.** The source is truncated toward zero. A result below the minimum of the target gives the minimum. A result above the maximum gives the maximum. Positive and negative infinity give the maximum and the minimum. NaN gives 0.

**D6. Numeric to `bool`.** The result is false when the source equals zero, including +0.0 and -0.0. Every other value gives true, including NaN.

**D7. `bool` to numeric.** False gives 0 and true gives 1. For a floating-point target the results are 0.0 and 1.0.

**D8. `char` to numeric.** The source is its scalar value, a non-negative integer. An integer target applies D2 to that value, so a narrow target keeps the low-order bits. A floating-point target receives the scalar value exactly, because every scalar value is below 2^24.

**D9. Numeric to `char`.** Let v be the source value. For a floating-point source, v is the source truncated toward zero, and NaN and infinities have no v. When v exists and is a Unicode scalar value, the result is the `char` with that scalar value. Otherwise the result is U+FFFD REPLACEMENT CHARACTER. The conversion does not wrap.

**D10. `char` to `string`.** The result is a string that contains exactly the scalar value. The string encoding belongs to the string runtime specification.

**D11. Null to aggregate.** The result is the null value of the target type, the same value that the NullToAggregate conversion produces. This ADR defines no operation on a null value. Its representation, and the behavior of index, length, and call on it, stay in OPEN-001.

**D12. Identity and widening.** An identity cast does not change the value. Every NumericAssignable pair in Section 7.2 preserves the value, and D2, D3, and D4 give the source value for each of them.

## Options Considered

| Question | Chosen | Rejected options and reason |
|---|---|---|
| Failure policy | Total. Every value has a result. | **Trap on loss:** a cast is how a program accepts a value change. A trap makes `x as u8` unusable for taking the low byte, and no runtime error model exists. **Undefined behavior:** the front end would accept programs with no result. **Separate checked and unchecked casts:** compatible with this ADR. The checked form is deferred. |
| Integer narrowing | Wrap modulo 2^N (D2) | **Saturate:** changes the bit pattern, so the low byte cannot be extracted, and costs a compare and a select. **Trap:** see above. |
| Float to integer | Truncate, saturate, NaN to 0 (D5) | **Trap:** see above. **Wrap:** has no defined value for fractions, NaN, or infinity. **Hardware-defined result:** differs between targets. |
| Rounding | Nearest, ties to even (D3, D4) | **Toward zero:** differs from the default hardware conversion. **Implementation-defined:** differs between targets. The choice also matches TR-028a, which rounds a floating-point literal to nearest. |
| Numeric to `bool` | Not equal to zero (D6) | **Only 0 and 1 valid:** needs a trap or an extra rule for other values. **Reject floating-point sources:** narrows the ADR-0002 families. |
| Value set of `char` | Unicode scalar value (D1) | **Byte:** cannot hold `'é'` or any other non-ASCII literal. **UTF-16 code unit:** a surrogate is not a character. |
| Numeric to `char`, invalid value | U+FFFD (D9) | **Trap:** see above. **Wrap into the valid range:** maps distinct values to the same character without notice. **Allow only `u8` and `u32` sources:** narrows the ADR-0002 families and is the cleaner rule if a diagnostic is preferred to a replacement character. |

## Trade-off Analysis

1. **Lowering cost.** Totality requires a clamp for floating-point-to-integer casts on targets whose conversion instruction does not saturate, and a range check for numeric-to-`char` casts. A trap would be cheaper to lower and is rejected for the reason in the table.
2. **Hidden errors.** `-1 as char` and `1e20 as u64` produce values without a diagnostic. D9 is the weakest rule in this ADR for that reason. A checked conversion is the remedy, and it is listed as an action item.
3. **Alignment with Rust.** Every numeric-to-numeric result in this ADR equals the result of Rust's `as` operator for the same types. Rust has no numeric-to-`bool` cast and allows only `u8` to `char`. D6 and D9 follow Descar's ADR-0002 families.
4. **Independence from arithmetic.** An arithmetic trap or wrap decided later does not change any cast. Rust makes the same separation.

## Consequences

**Easier:**

- Every accepted cast has a result. The lowering of an ExplicitCast depends only on the source and target types.
- OPEN-001 shrinks to arithmetic overflow, division by zero, shift counts, the rounding of an F32 literal, and the representation of null aggregates.
- The vectors below can become execution tests when an evaluator exists.

**Harder:**

- The lowering specification must provide a clamp sequence for floating-point-to-integer casts.
- No program can detect a lossy cast at runtime until a checked conversion exists.

**To revisit:**

- A checked conversion that reports a value change.
- Whether numeric-to-`char` accepts only `u8` and `u32` sources.
- The runtime representation of null aggregates.

## Verification Vectors

A source value has the stated type. An F32 source is the F32 value nearest to the written decimal. NaN is quiet NaN. Every numeric-to-numeric, `bool`-to-numeric, and `char`-to-numeric result equals the result of Rust's `as` operator for the same types, with a `bool` or `char` source reaching a floating-point target through `u8` or `u32`. Numeric-to-`bool` and numeric-to-`char` have no Rust equivalent and follow D6 and D9.

### D2. Integer to integer

| Source type | Source value | Target | Result |
|---|---|---|---|
| i64 | `300` | i8 | `44` |
| i64 | `-1` | u8 | `255` |
| i32 | `-1` | u32 | `4294967295` |
| i32 | `-1` | u64 | `18446744073709551615` |
| u8 | `255` | i8 | `-1` |
| i16 | `128` | i8 | `-128` |
| u64 | `18446744073709551615` | i64 | `-1` |
| i64 | `9223372036854775807` | i32 | `-1` |
| i8 | `-128` | i64 | `-128` |
| u32 | `4294967295` | i64 | `4294967295` |

### D3. Integer to floating point

| Source type | Source value | Target | Result |
|---|---|---|---|
| i32 | `16777217` | f32 | `16777216.0` |
| i32 | `16777219` | f32 | `16777220.0` |
| i32 | `16777217` | f64 | `16777217.0` |
| i64 | `9007199254740993` | f64 | `9007199254740992.0` |
| i64 | `9007199254740995` | f64 | `9007199254740996.0` |
| u64 | `18446744073709551615` | f32 | `18446744073709551616.0` |
| u64 | `18446744073709551615` | f64 | `18446744073709551616.0` |
| i64 | `-9223372036854775808` | f64 | `-9223372036854775808.0` |

### D4. Floating point to floating point

| Source type | Source value | Target | Result |
|---|---|---|---|
| f64 | `1e39` | f32 | `+inf` |
| f64 | `-1e39` | f32 | `-inf` |
| f64 | `0.1` | f32 | `0.10000000149011612` |
| f64 | `3.5` | f32 | `3.5` |
| f64 | `NaN` | f32 | `NaN` |
| f64 | `1e-46` | f32 | `0.0` |
| f64 | `-0.0` | f32 | `-0.0` |
| f32 | `0.1` | f64 | `0.10000000149011612` |

### D5. Floating point to integer

| Source type | Source value | Target | Result |
|---|---|---|---|
| f64 | `3.99` | i32 | `3` |
| f64 | `-3.99` | i32 | `-3` |
| f64 | `300.0` | u8 | `255` |
| f64 | `-1.0` | u8 | `0` |
| f64 | `-129.5` | i8 | `-128` |
| f64 | `NaN` | i32 | `0` |
| f64 | `+inf` | i64 | `9223372036854775807` |
| f64 | `-inf` | i64 | `-9223372036854775808` |
| f64 | `1e20` | u64 | `18446744073709551615` |
| f64 | `-0.0` | i32 | `0` |
| f32 | `2147483648.0` | i32 | `2147483647` |
| f64 | `4294967296.0` | u32 | `4294967295` |

### D6. Numeric to `bool`

| Source type | Source value | Target | Result |
|---|---|---|---|
| i32 | `0` | bool | `false` |
| i32 | `-5` | bool | `true` |
| u64 | `1` | bool | `true` |
| f64 | `0.0` | bool | `false` |
| f64 | `-0.0` | bool | `false` |
| f64 | `NaN` | bool | `true` |
| f64 | `5e-324` | bool | `true` |
| f32 | `0.5` | bool | `true` |

### D7. `bool` to numeric

| Source type | Source value | Target | Result |
|---|---|---|---|
| bool | `true` | i8 | `1` |
| bool | `false` | u64 | `0` |
| bool | `true` | f32 | `1.0` |
| bool | `false` | f64 | `0.0` |

### D8. `char` to numeric

A `char` source is written as its scalar value.

| Source type | Source value | Target | Result |
|---|---|---|---|
| char | `U+0041` | u8 | `65` |
| char | `U+0041` | i8 | `65` |
| char | `U+00E9` | i8 | `-23` |
| char | `U+20AC` | u8 | `172` |
| char | `U+1F600` | u16 | `62976` |
| char | `U+1F600` | i32 | `128512` |
| char | `U+10FFFF` | f32 | `1114111.0` |
| char | `U+0000` | u8 | `0` |

### D9. Numeric to `char`

| Source type | Source value | Target | Result |
|---|---|---|---|
| i32 | `65` | char | `U+0041` |
| i32 | `233` | char | `U+00E9` |
| i64 | `128512` | char | `U+1F600` |
| i32 | `-1` | char | `U+FFFD` |
| u32 | `55296` | char | `U+FFFD` |
| u32 | `57343` | char | `U+FFFD` |
| u32 | `57344` | char | `U+E000` |
| u32 | `1114111` | char | `U+10FFFF` |
| u32 | `1114112` | char | `U+FFFD` |
| u64 | `4294967361` | char | `U+FFFD` |
| f64 | `65.9` | char | `U+0041` |
| f64 | `-0.5` | char | `U+0000` |
| f64 | `NaN` | char | `U+FFFD` |
| f64 | `+inf` | char | `U+FFFD` |

## Action Items

1. [ ] Specification: record DEC-021, trim OPEN-001, and update Sections 7.4, 8.5, 14.10, and the README.
2. [ ] Language owner: review D1 and D9, then change the status to Accepted.
3. [ ] Lowering specification: map each rule to target operations and choose the clamp sequence for D5 and the range check for D9.
4. [ ] Tests: implement the vectors as execution tests when an evaluator exists.
5. [ ] Decide a checked conversion for programs that need a diagnostic on value change.
6. [ ] Decide the runtime representation of null aggregates (OPEN-001).
7. [ ] String runtime specification: define the encoding that D10 relies on.
