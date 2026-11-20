//! # Printers Module
//!
//! The printers module renders syntax trees as text in a tree form. Each node of the
//! tree has one line. Branch symbols and indentation show the relation between a node
//! and its children. The output can contain ANSI style sequences for the terminal.
//!
//! * [`ast_printer`]: Renders expressions and statements.
//! * [`branch_type`]: Branch symbols, indentation, and styles for the tree output.

/// Human-readable AST rendering.
///
/// The module renders an expression or a statement as a tree of text lines.
pub mod ast_printer;
pub mod branch_type;
