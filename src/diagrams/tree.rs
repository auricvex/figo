//! Hierarchical tree diagrams.
//!
//! Provides both a free function ([`draw_tree`]) for simple usage and a
//! builder ([`Tree`]) for complex configurations.

use std::fmt;

use crate::error::{FigoError, Result};
use crate::style::Charset;

/// A node in a tree structure.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// The label displayed for this node.
    pub label: String,
    /// Child nodes nested under this node.
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a new leaf node.
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), children: Vec::new() }
    }
}

/// Draw a tree diagram.
pub fn draw_tree(
    root: Option<&str>,
    nodes: &[TreeNode],
    _width: usize,
    charset: Charset,
) -> Result<String> {
    Tree::new(charset).root(root).nodes(nodes).build()
}

/// Builder for tree diagrams.
pub struct Tree {
    charset: Charset,
    root: Option<String>,
    nodes: Vec<TreeNode>,
}

impl Tree {
    /// Create a new tree builder.
    pub fn new(charset: Charset) -> Self {
        Self { charset, root: None, nodes: Vec::new() }
    }

    /// Set the root label.
    pub fn root(mut self, root: Option<&str>) -> Self {
        self.root = root.map(String::from);
        self
    }

    /// Set all nodes at once (replaces any previously added nodes).
    pub fn nodes(mut self, nodes: &[TreeNode]) -> Self {
        self.nodes = nodes.to_vec();
        self
    }

    /// Add a single node to the tree.
    pub fn add_node(mut self, node: TreeNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Render the tree and return it as a `String`.
    ///
    /// This is the primary rendering method. For `Display`-based output,
    /// format the builder directly.
    pub fn build(&self) -> Result<String> {
        if self.root.is_none() && self.nodes.is_empty() {
            return Err(FigoError::MissingFields("tree must have at least a root or nodes".into()));
        }

        let mut lines: Vec<String> = Vec::new();

        if let Some(ref root_label) = self.root {
            lines.push(root_label.clone());
        }

        // Compute last-sibling state at the top level; descendants are
        // handled recursively by `render_node`.
        let last_root_idx = self.nodes.len().saturating_sub(1);
        for (i, node) in self.nodes.iter().enumerate() {
            let is_last = i == last_root_idx;
            self.render_node(node, &mut lines, "", is_last);
        }

        Ok(lines.join("\n") + "\n")
    }

    fn render_node(&self, node: &TreeNode, lines: &mut Vec<String>, prefix: &str, is_last: bool) {
        let (branch, continuation) = self.branch_chars();

        let marker = if is_last { continuation } else { branch };

        lines.push(format!("{prefix}{marker} {}", node.label));

        let child_prefix =
            if is_last { format!("{prefix}    ") } else { format!("{prefix}│   ") };

        for (i, child) in node.children.iter().enumerate() {
            let last_child = i == node.children.len() - 1;
            self.render_node(child, lines, &child_prefix, last_child);
        }
    }

    fn branch_chars(&self) -> (&'static str, &'static str) {
        match self.charset {
            Charset::Unicode => ("├──", "└──"),
            Charset::Ascii => ("+--", "\\--"),
        }
    }

    /// Render and return as a `String`.
    ///
    /// Alias for [`build`](Self::build).
    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tree() {
        let nodes = vec![
            TreeNode {
                label: "src/".into(),
                children: vec![TreeNode::new("main.rs"), TreeNode::new("lib.rs")],
            },
            TreeNode::new("Cargo.toml"),
        ];
        let out = draw_tree(Some("Project Root"), &nodes, 60, Charset::Unicode).unwrap();
        assert!(out.contains("Project Root"));
        assert!(out.contains("src/"));
        assert!(out.contains("main.rs"));
        assert!(out.contains("Cargo.toml"));
    }

    #[test]
    fn test_ascii_tree() {
        let nodes = vec![TreeNode::new("file.rs")];
        let out = draw_tree(None, &nodes, 60, Charset::Ascii).unwrap();
        assert!(out.contains("\\--"));
    }

    #[test]
    fn test_empty_tree_errors() {
        assert!(draw_tree(None, &[], 60, Charset::Unicode).is_err());
    }

    #[test]
    fn test_deeply_nested() {
        let nodes = vec![TreeNode {
            label: "a".into(),
            children: vec![TreeNode { label: "b".into(), children: vec![TreeNode::new("c")] }],
        }];
        let out = draw_tree(None, &nodes, 60, Charset::Unicode).unwrap();
        assert!(out.contains("a"));
        assert!(out.contains("b"));
        assert!(out.contains("c"));
    }

    /// Regression for the top-level `is_last` bug: every root child
    /// must get `├──` (or `+--` in ASCII) unless it is the literal final
    /// sibling, which gets `└──` (or `\--` in ASCII). Previously the
    /// dispatch loop hardcoded `is_last = true`, which gave every root
    /// child the last-sibling glyph and left the rendered diagram
    /// looking like a flat list of leaves hanging off the same spine.
    #[test]
    fn test_root_branch_glyph_is_only_for_last_sibling() {
        let nodes = vec![TreeNode::new("a/"), TreeNode::new("b/"), TreeNode::new("c")];
        let out = draw_tree(Some("/"), &nodes, 60, Charset::Unicode).unwrap();
        assert!(out.contains("├── a/"), "non-last sibling must use ├──\n{out}");
        assert!(out.contains("├── b/"), "non-last sibling must use ├──\n{out}");
        assert!(out.contains("└── c"), "last sibling must use └──\n{out}");
        assert!(!out.contains("└── a/"), "non-last sibling must NOT use └──\n{out}");
        assert!(!out.contains("└── b/"), "non-last sibling must NOT use └──\n{out}");
    }

    /// Regression: a non-last root child must propagate a `│` continuation
    /// rail to every row beneath it, so sibling content printed under
    /// the rail stays visually anchored to the parent root.
    #[test]
    fn test_continuation_rail_for_non_last_root_child() {
        let nodes = vec![
            TreeNode {
                label: "src/".into(),
                children: vec![TreeNode::new("main.rs"), TreeNode::new("lib.rs")],
            },
            TreeNode::new("Cargo.toml"),
        ];
        let out = draw_tree(Some("project/"), &nodes, 60, Charset::Unicode).unwrap();
        assert!(out.contains("├── src/"), "non-last root child must use ├──\n{out}");
        assert!(
            out.contains("│   ├── main.rs"),
            "non-last parent's children must carry the │ rail\n{out}",
        );
        assert!(out.contains("│   └── lib.rs"), "\n{out}");
        assert!(out.contains("└── Cargo.toml"));
    }

    /// ASCII charset must mirror the Unicode branching semantics: only
    /// the last sibling gets `\--`; non-last siblings get `+--`. The
    /// legacy behaviour used `\--` for every sibling in ASCII too.
    #[test]
    fn test_ascii_root_branch_glyph_is_only_for_last_sibling() {
        let nodes = vec![TreeNode::new("a.txt"), TreeNode::new("b.txt"), TreeNode::new("c.txt")];
        let out = draw_tree(Some("/"), &nodes, 60, Charset::Ascii).unwrap();
        assert!(out.contains("+-- a.txt"));
        assert!(out.contains("+-- b.txt"));
        assert!(out.contains("\\-- c.txt"));
        assert!(!out.contains("\\-- a.txt"));
        assert!(!out.contains("\\-- b.txt"));
    }
}
