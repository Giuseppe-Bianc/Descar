//! # Semantic Analysis Module
//!
//! The semantic analysis module handles the verification of syntactic correctness
//! and type checking of the abstract syntax tree.
pub mod symbol_table;
pub mod type_checker;
pub mod typed_ast;

pub use typed_ast::{FullyTypedAst, ResolvedType, TypedElseBranch, TypedExpr, TypedExprKind, TypedStmt, TypedVarBinding};
