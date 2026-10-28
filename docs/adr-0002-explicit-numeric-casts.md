# ADR-0002: Explicit cast syntax and conversion rules

**Status:** Accepted. ADR-0005 defines the runtime value of every cast that this ADR leaves open.  
**Date:** 2026-10-26  
**Deciders:** Descar language owner  
**Applies to:** Descar source syntax and numeric type checking  
**Amends:** Fully Typed AST specification Sections 6.2, 6.4, 7.4, 13, 14, and 15

## Context

The numeric conversion rules preserve every value for implicit conversions. That constraint intentionally rejects potentially lossy conversions such as narrowing I64 to I8 and converting I64 to F64. However, a program sometimes needs to request such a conversion explicitly. The language specification previously recorded explicit casts as unsupported and reserved ADR-0002 for that decision.

## Decision

Introduce the cast expression:

```text
expression as type
```

The target follows Descar's existing type grammar, so targets may be numeric types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`), `char`, `bool`, `string`, fixed arrays, vectors, or named custom types. `nullptr` is a value, not a target type token.

The supported conversion families are:

- Numeric-to-numeric, including narrowing and precision loss.
- Numeric-to-`char` and `bool`, and `char` or `bool` to numeric.
- `char` to `string`, using the existing CharToString conversion.
- `nullptr` to an array, vector, or named custom type, using the existing NullToAggregate conversion.
- Identity casts, where source and target types are equivalent.

String parsing, string-to-numeric casts, unrelated aggregate conversions, arbitrary custom-to-custom conversions, and pointer reinterpretation are rejected. The parser represents casts as a dedicated Cast expression. Unsupported source/target combinations produce E2002; a token that cannot begin a type, such as `nullptr`, produces E1002.

Explicit casts do not expand or weaken implicit assignability. Numeric narrowing and precision loss remain explicit-only. An identity cast does not require a runtime conversion operation. Conversions that already have a defined semantic conversion kind, such as CharToString and NullToAggregate, reuse that conversion kind rather than inventing new behavior.

The cast binds more tightly than multiplicative operators and less tightly than prefix operators. Chained casts associate left-to-right. Therefore, `-value as i32` means `(-value) as i32`, while `left + right as f64` means `left + (right as f64)`.

This decision specifies parsing and front-end type checking only. Runtime behavior for integer overflow or narrowing, out-of-range floating-point-to-integer conversion, floating-point rounding, numeric-to-char conversion, numeric-to-bool conversion, and the runtime representation of nullptr-backed aggregates remains unspecified. The lowering and execution specifications must define these cases before generated code executes value-changing casts.

## Alternatives considered

- **Allow implicit narrowing:** Rejected because it would silently change the values of ordinary assignments and operations, contradicting ADR-0003.
- **Use C-style casts such as `(i8)value`:** Rejected because it adds parsing ambiguity and differs from the expression syntax already used by Descar.
- **Use Rust-style `expression as type`:** Chosen because the target is explicit, readable, and can be integrated into the existing Pratt parser with a defined binding power.

## Consequences

- The lexer reserves `as` as a keyword.
- The full existing type grammar is accepted for cast targets, including strings, arrays, vectors, and custom types.
- String parsing, unrelated aggregate conversions, and pointer reinterpretation remain unsupported.
- The AST, parser, semantic checker, and AST printer represent cast expressions directly.
- Regression tests cover cast parsing, precedence, chaining, numeric/scalar conversions, char-to-string, null-to-aggregate, identity casts, and invalid source or target types.
- The README and Fully Typed AST specification document the syntax.
- Runtime conversion behavior remains an open item for the lowering and execution specifications.
