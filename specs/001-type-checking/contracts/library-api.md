# Contract: Semantic Type Checker Library API

**Feature**: `001-type-checking` | **Date**: 2026-09-15 | **Spec**: [spec.md](../spec.md)

This contract specifies the public interface of the semantic type checker exposed by `descar-core`.

---

## 1. Top-Level Module Interface (`descar_core::semantic`)

```rust
// crates/descar-core/src/semantic/mod.rs

pub mod binder;
pub mod check;
pub mod ctx;
pub mod diag;
pub mod globals;
pub mod resolve;
pub mod sig;

pub use ctx::{CheckerCtx, Phase};
pub use diag::Diagnostic;
pub use check::Expectation;

/// Represents a successfully verified program with full type annotations.
#[derive(Debug, Clone)]
pub struct CheckedProgram {
    pub ast: Vec<crate::syntax::ast::Stmt>,
    pub symbols: ctx::SymbolTable,
    pub types: ctx::TypeTable,
    pub node_types: std::collections::HashMap<usize, ctx::TypeId>,
}

/// Orchestrates the semantic type checking pipeline across all six phases in order:
/// 1. `binder::bind`: Builds scope tree and registers symbols.
/// 2. `resolve::resolve`: Links identifiers to symbols.
/// 3. `sig::process_signatures`: Computes declared types and checks infinite cycles.
/// 4. `check::check_bodies`: Validates function and method bodies against expectations.
/// 5. `globals::check_globals`: Performs whole-program linting and visibility checks.
///
/// Returns `Ok(CheckedProgram)` if no blocking errors occur, or `Err(Vec<Diagnostic>)`
/// containing all collected diagnostics across all phases.
pub fn check_program(
    ast: &[crate::syntax::ast::Stmt],
) -> Result<CheckedProgram, Vec<Diagnostic>> {
    let mut ctx = CheckerCtx::new();
    
    // Phase 1: Binder
    binder::bind(ast, &mut ctx);
    if ctx.has_blocking_errors() {
        return Err(ctx.take_diagnostics());
    }

    // Phase 2: Resolve
    resolve::resolve(ast, &mut ctx);
    if ctx.has_blocking_errors() {
        return Err(ctx.take_diagnostics());
    }

    // Phase 3: Signatures & Cycle Detection
    sig::process_signatures(ast, &mut ctx);
    if ctx.has_blocking_errors() {
        return Err(ctx.take_diagnostics());
    }

    // Phase 4: Body Checking
    check::check_bodies(ast, &mut ctx);
    if ctx.has_blocking_errors() {
        return Err(ctx.take_diagnostics());
    }

    // Phase 5: Global Program Checks
    globals::check_globals(ast, &mut ctx);
    if ctx.has_blocking_errors() {
        return Err(ctx.take_diagnostics());
    }

    Ok(ctx.into_checked_program(ast.to_vec()))
}
```

---

## 2. Phase Entry Point Signatures

Every phase exposes a standalone public entry function and an isolated error type:

### 2.1 Phase 1: Binder (`descar_core::semantic::binder`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinderError {
    DuplicateDeclaration {
        name: String,
        first_span: crate::location::source_span::SourceSpan,
        duplicate_span: crate::location::source_span::SourceSpan,
    },
}

pub fn bind(
    ast: &[crate::syntax::ast::Stmt],
    ctx: &mut CheckerCtx,
);
```

### 2.2 Phase 2: Resolve (`descar_core::semantic::resolve`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    UndeclaredIdentifier {
        name: String,
        span: crate::location::source_span::SourceSpan,
    },
    UnknownType {
        name: String,
        span: crate::location::source_span::SourceSpan,
    },
}

pub fn resolve(
    ast: &[crate::syntax::ast::Stmt],
    ctx: &mut CheckerCtx,
);
```

### 2.3 Phase 3: Signatures (`descar_core::semantic::sig`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigError {
    InfiniteSizeRecursiveType {
        name: String,
        cycle: Vec<String>,
        span: crate::location::source_span::SourceSpan,
    },
    InvalidVoidUsage {
        span: crate::location::source_span::SourceSpan,
    },
    InvalidArraySize {
        span: crate::location::source_span::SourceSpan,
    },
}

pub fn process_signatures(
    ast: &[crate::syntax::ast::Stmt],
    ctx: &mut CheckerCtx,
);
```

### 2.4 Phase 4: Body Checking (`descar_core::semantic::check`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    TypeMismatch {
        expected: ctx::TypeId,
        found: ctx::TypeId,
        span: crate::location::source_span::SourceSpan,
    },
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        span: crate::location::source_span::SourceSpan,
    },
    NonBooleanCondition {
        found: ctx::TypeId,
        span: crate::location::source_span::SourceSpan,
    },
    InvalidOperatorOperands {
        op: crate::syntax::ast::binary_op::BinaryOp,
        left: ctx::TypeId,
        right: ctx::TypeId,
        span: crate::location::source_span::SourceSpan,
    },
}

pub fn check_bodies(
    ast: &[crate::syntax::ast::Stmt],
    ctx: &mut CheckerCtx,
);
```

### 2.5 Phase 5: Global Checks (`descar_core::semantic::globals`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalsWarning {
    UnusedVariable {
        name: String,
        span: crate::location::source_span::SourceSpan,
    },
    UnreachableCode {
        span: crate::location::source_span::SourceSpan,
    },
}

pub fn check_globals(
    ast: &[crate::syntax::ast::Stmt],
    ctx: &mut CheckerCtx,
);
```

---

## 3. Diagnostic Bridge to `CompileError`

To seamlessly integrate with `descar-core`'s existing `ErrorReporter`, `Diagnostic` implements conversion to `CompileError`:

```rust
impl Diagnostic {
    pub fn to_compile_error(&self) -> crate::error::compile_error::CompileError {
        crate::error::compile_error::CompileError::TypeError {
            code: Some(self.code),
            message: self.message.clone().into(),
            span: self.span.clone(),
            help: self.help.clone(),
        }
    }
}
```
