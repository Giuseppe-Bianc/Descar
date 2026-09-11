use crate::syntax::ast::stmt::Stmt;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ElseBranch {
    None,
    Block(Box<Stmt>),
    ElseIf(Box<Stmt>),
}
