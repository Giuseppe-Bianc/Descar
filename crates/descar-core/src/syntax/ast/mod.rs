// crates/descar-core/src/syntax/ast/mod.rs

//! Data types that represent the Descar abstract syntax tree.

/// Assignment operators.
pub mod assignment_op;
/// Semantic types used in the AST.
pub mod ast_type;
/// Binary operators.
pub mod binary_op;
/// Compilation units.
pub mod compilation_unit;
/// Else branches.
pub mod else_branch;
/// Expressions.
pub mod expr;
/// Literal values.
pub mod literal_value;
/// Function parameters.
pub mod parameter;
/// Statements.
pub mod stmt;
/// Type syntax nodes.
pub mod type_syntax;
/// Unary operators.
pub mod unary_op;
/// Operand side metadata for unary operators.
pub mod unary_op_side;

pub use assignment_op::AssignmentOp;
pub use ast_type::Type;
pub use compilation_unit::CompilationUnit;
pub use else_branch::ElseBranch;
pub use expr::Expr;
pub use literal_value::LiteralValue;
pub use parameter::Parameter;
pub use stmt::Stmt;
pub use type_syntax::TypeSyntax;
