// crates/descar-core/src/syntax/ast/else_branch.rs

use crate::syntax::ast::stmt::Stmt;

/// The `else` part of an `if` statement.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ElseBranch {
    /// The `if` statement has no `else` part.
    None,
    /// The `else` part is a block.
    Block(Box<Stmt>),
    /// The `else` part is a nested `if` statement (`else if`).
    ElseIf(Box<Stmt>),
}