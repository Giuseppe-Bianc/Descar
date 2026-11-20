//! Branch type printer utilities for tree-like output structures.
//!
//! This module provides types and functions that print hierarchical data with
//! indentation and branch symbols (`├── ` and `└── `).
//!
//! # Key Types
//! - [`BranchType`]: The position of a node among its siblings (`Last` or `Middle`)
//! - [`BranchConfig`]: Holds the branch types of a parent node, a current node, and a child node
//! - [`StyleManager`]: Console styles for the different element types
//!
//! # Usage
//! Use [`print_children`] to go through the child elements with the correct branch symbols.
//! Use [`append_line`] to add a formatted line with the correct indentation.

use console::Style;
use std::fmt::Write;

/// Position of a node among its siblings in a tree output.
///
/// The branch type selects the branch symbol of the node and the indentation of its children.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchType {
    /// The node is the last child of its parent.
    Last,
    /// The node is a child of its parent and has at least one sibling after it.
    Middle,
}

impl BranchType {
    /// Returns the branch symbol that starts the line of a node.
    ///
    /// # Returns
    ///
    /// `"└── "` for [`BranchType::Last`] and `"├── "` for [`BranchType::Middle`].
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Last => "└── ",
            Self::Middle => "├── ",
        }
    }

    /// Returns the text that the indentation of the children of a node uses.
    ///
    /// # Returns
    ///
    /// Four spaces for [`BranchType::Last`]. For [`BranchType::Middle`], a vertical line
    /// (`│`) followed by three spaces. The vertical line connects the children to the
    /// siblings that follow the node.
    #[must_use]
    pub const fn indent_continuation(&self) -> &'static str {
        match self {
            Self::Last => "    ",
            Self::Middle => "│   ",
        }
    }
}

/// Holds the branch types of a parent node, a current node, and a child node.
pub struct BranchConfig {
    /// Branch type of the parent node.
    pub parent_type: BranchType,
    /// Branch type of the current node.
    pub current_type: BranchType,
    /// Branch type of the child node.
    pub child_type: BranchType,
}

impl BranchConfig {
    /// Creates a branch configuration.
    ///
    /// # Arguments
    ///
    /// * `parent_type` - The branch type of the parent node.
    /// * `current_type` - The branch type of the current node.
    /// * `child_type` - The branch type of the child node.
    #[must_use]
    pub const fn new(parent_type: BranchType, current_type: BranchType, child_type: BranchType) -> Self {
        Self { parent_type, current_type, child_type }
    }
}

/// Console styles for the different element types in a tree output.
///
/// [`StyleManager::new`] sets the colors and the text attributes. The derived `Default`
/// implementation uses the `Default` value of `Style` for each field.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleManager {
    /// Style for operators.
    pub operator: Style,
    /// Style for literal values.
    pub literal: Style,
    /// Style for variables and names.
    pub variable: Style,
    /// Style for labels that give the structure of the tree.
    pub structure: Style,
    /// Style for punctuation and grouping elements.
    pub punctuation: Style,
    /// Style for keywords and statements.
    pub keyword: Style,
    /// Style for types.
    pub type_style: Style,
    /// Style for notes, for example an empty block.
    pub metadata: Style,
}

impl StyleManager {
    /// Creates the default styles of the tree output.
    ///
    /// The styles are:
    ///
    /// * `operator`: blue
    /// * `literal`: green
    /// * `variable`: yellow
    /// * `structure`: cyan
    /// * `punctuation`: magenta
    /// * `keyword`: blue
    /// * `type_style`: green
    /// * `metadata`: dim and italic
    #[must_use]
    pub const fn new() -> Self {
        Self {
            operator: Style::new().blue(),
            literal: Style::new().green(),
            variable: Style::new().yellow(),
            structure: Style::new().cyan(),
            punctuation: Style::new().magenta(),
            keyword: Style::new().blue(),
            type_style: Style::new().green(),
            metadata: Style::new().dim().italic(),
        }
    }
}

/// Prints a list of children with correct branch symbols and indentation.
///
/// Iterates through all children, assigning [`BranchType::Middle`] to all but the last child,
/// and [`BranchType::Last`] to the final child. For each child, calls the provided closure
/// with the appropriate branch type.
///
/// If `children` is empty, the function does not call `print_fn`.
///
/// # Arguments
///
/// * `children` - The slice of child elements to print.
/// * `indent` - The current indentation text.
/// * `output` - The string buffer that receives the formatted output.
/// * `styles` - The style configuration for the formatting.
/// * `print_fn` - The closure that the function calls for each child. The arguments of the
///   closure are `(child, indent, branch_type, output, styles)`.
pub fn print_children<T, F>(children: &[T], indent: &str, output: &mut String, styles: &StyleManager, mut print_fn: F)
where
    F: FnMut(&T, &str, BranchType, &mut String, &StyleManager),
{
    if let Some((last, rest)) = children.split_last() {
        for child in rest {
            print_fn(child, indent, BranchType::Middle, output, styles);
        }
        print_fn(last, indent, BranchType::Last, output, styles);
    }
}

/// Builds the indentation text for the children of a node.
///
/// The function appends the continuation text of `branch_type` to `indent`. See
/// [`BranchType::indent_continuation`].
///
/// # Arguments
///
/// * `indent` - The indentation text of the node.
/// * `branch_type` - The branch type of the node.
///
/// # Returns
///
/// A new string with the indentation text of the children.
#[must_use]
pub fn get_indent(indent: &str, branch_type: &BranchType) -> String {
    let continuation = branch_type.indent_continuation();
    let mut result = String::with_capacity(indent.len() + continuation.len());
    result.push_str(indent);
    result.push_str(continuation);
    result
}

/// Appends one line of a tree output to a string.
///
/// The line has the form `<indent><branch symbol><text>` and ends with a line break. The
/// function applies `style` to `text`. The branch symbol comes from [`BranchType::symbol`].
///
/// # Arguments
///
/// * `output` - The string that receives the line.
/// * `indent` - The indentation text of the line.
/// * `branch_type` - The branch type of the node that the line shows.
/// * `style` - The style that the function applies to `text`.
/// * `text` - The text of the node.
pub fn append_line(output: &mut String, indent: &str, branch_type: BranchType, style: &Style, text: &str) {
    let branch = branch_type.symbol();
    let styled_text = style.apply_to(text);
    writeln!(output, "{indent}{branch}{styled_text}").unwrap();
}
