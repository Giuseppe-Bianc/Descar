# ADR-0002: Explicit numeric casts

**Status:** Accepted  
**Date:** 2026-10-26  
**Deciders:** Descar language owner  
**Applies to:** Descar source syntax and numeric type checking  
**Amends:** Fully Typed AST specification Sections 6.2, 6.4, 7.4, 13, 14, and 15

## Context

The numeric conversion rules preserve every value for implicit conversions. That constraint intentionally rejects potentially lossy conversions such as narrowing I64 to I8 and converting I64 to F64. However, a program sometimes needs to request such a conversion explicitly. The language specification previously recorded explicit casts as unsupported and reserved ADR-0002 for that decision.

## Decision

Introduce the cast expression:

```text
expression as numeric_type
```

The target type may be one of `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`, or `bool`.

Numeric-to-numeric casts are supported, including narrowing and precision loss. Numeric-to-char, char-to-numeric, numeric-to-bool, and bool-to-numeric conversions are also supported as explicit casts. String and pointer conversions are rejected. The parser represents the syntax as a dedicated Cast expression. Unsupported source and target combinations produce E2002; unsupported target tokens produce E1002.

Explicit casts do not expand or weaken the implicit assignability table. Numeric narrowing and precision loss are only expressible through this cast syntax. An identity cast is valid and does not require a runtime conversion operation.

The cast binds more tightly than multiplicative operators and less tightly than prefix operators. Chained casts associate left-to-right. Therefore, `-value as i32` means `(-value) as i32`, while `left + right as f64` means `left + (right as f64)`.

This decision specifies parsing and front-end type checking only. Runtime behavior for integer overflow or narrowing, out-of-range floating-point-to-integer conversion, floating-point rounding, numeric-to-char conversion, and numeric-to-bool conversion remains unspecified. The lowering and execution specifications must define those cases before generated code executes explicit casts.

## Alternatives considered

- **Allow implicit narrowing:** Rejected because it would silently change the values of ordinary assignments and operations, contradicting ADR-0003.
- **Use C-style casts such as `(i8)value`:** Rejected because it adds parsing ambiguity and differs from the expression syntax already used by Descar.
- **Use Rust-style `expression as type`:** Chosen because the target is explicit, readable, and can be integrated into the existing Pratt parser with a defined binding power.

## Consequences

- The lexer reserves `as` as a keyword.
- `char` and `bool` are supported cast targets, but do not make string or pointer casts valid.
- The AST, parser, semantic checker, and AST printer represent cast expressions directly.
- Regression tests cover cast parsing, precedence, chaining, numeric conversions, and invalid source or target types.
- The README and Fully Typed AST specification document the syntax.
- Runtime conversion behavior remains an open item for the lowering and execution specifications.
