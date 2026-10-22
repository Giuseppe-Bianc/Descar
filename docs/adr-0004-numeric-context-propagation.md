# ADR-0004: Numeric literal context propagation through operators

**Status:** Accepted
**Date:** 2026-10-22
**Deciders:** Descar language owner
**Applies to:** Descar Fully Typed Abstract Syntax Tree Specification
**Amends:** TR-028a, TR-028b, TR-028c, DEC-018, OPEN-006, Section 13.2 test 29, Section 14.9

## Context

TR-028a says that a floating-point literal does not adapt to an integer context. In that case, the literal keeps its default type and leaf adaptation does not report an error. The enclosing typing rule must still check assignability.

TR-028b says that an operator does not pass an F32 or F64 context to its operands. The previous wording of TR-028c said that two flexible operands both receive the context type of the Binary node, without explicitly making this subject to TR-028b. That wording could imply a conflicting rule.

The ambiguity obscures these results:

| Source | Required result |
|---|---|
| `var mixed: f64 = i + 1.5`, where `i: i32` | F64. The peer rule gives `1.5` an I32 context, but the floating-point literal remains F64. I32 is assignable to F64, so promotion selects F64 and converts the I32 operand to F64. |
| `var shifted: f64 = 1 << 3` | E2002. The F64 initializer context does not pass through the shift. The shift result is I64, which is not assignable to F64. |
| `var value: i32 = 1.5` | E2002. Leaf adaptation does not diagnose the kind mismatch, but the initializer rule checks assignability. |
| `var f: f64 = 1 + 2.5` | E2036 under the current rules. The two flexible literals do not receive the incoming F64 context through the operator. |

## Decision

TR-028b controls whether a Binary node passes its incoming context to its operands. The peer rule in TR-028c is separate and remains active when exactly one operand is flexible and the other operand is non-flexible and numeric. This peer rule may supply F32 or F64 as the flexible operand's context.

When both operands are flexible, the Binary node passes its context only when TR-028b permits it. An incoming F32 or F64 context is not passed through the operator. This decision clarifies the existing rule and does not add a numeric conversion or change the lossless-conversion requirement in ADR-0003.

The phrase in TR-028a that leaf adaptation reports nothing applies only to the leaf adaptation step. The enclosing expression, declaration, assignment, call, or return rule still checks assignability and emits its defined diagnostic.

## Consequences

- `i + 1.5` has type F64 when `i` is I32 or another integer type assignable to F64. The I32 operand receives a NumericAssignable conversion to F64. It reports E2036 when `i` is I64 or U64, which are not assignable to F64 under ADR-0003.
- `var shifted: f64 = 1 << 3` reports E2002 because the shift result remains I64.
- `var value: i32 = 1.5` reports E2002 from the initializer rule.
- `1 + 2.5` and `var f: f64 = 1 + 2.5` continue to report E2036. Whether to support mixed flexible operands through constant-aware literal adaptation remains OPEN-006.
- No implicit integer-to-floating-point conversion that loses precision is introduced.

## Specification amendments

- Clarify that TR-028a's no-error outcome applies only to leaf adaptation.
- Make TR-028b's prohibition on forwarding an incoming F32 or F64 context explicit and preserve the peer rule.
- Make the both-flexible case in TR-028c conditional on TR-028b.
- Add DEC-018, test 29, and examples in Section 14.9.
- Update OPEN-006 to record the unambiguous current behavior while retaining the future language-design question.
