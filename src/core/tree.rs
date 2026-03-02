//! Core domain model for directory tree with hierarchical structure.

use super::entry::Entry;

/// Directory tree with hierarchical children structure
#[derive(Debug, Clone)]
pub struct Tree {
    pub entry: Entry,
    pub children: Vec<Tree>,
}

impl Tree {
    /// Create a new tree node
    pub fn new(entry: Entry) -> Self {
        Self {
            entry,
            children: Vec::new(),
        }
    }

    /// Create tree with children
    pub fn with_children(entry: Entry, children: Vec<Tree>) -> Self {
        Self { entry, children }
    }

    /// Flatten tree into depth-first ordered list of entries
    pub fn flatten(&self) -> Vec<Entry> {
        let mut result = Vec::new();
        flatten_recursive(self, &[], &mut result);
        result
    }

    /// Count total nodes in tree
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }
}

fn flatten_recursive(node: &Tree, ancestors_last: &[bool], output: &mut Vec<Entry>) {
    let num_children = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == num_children - 1;

        let mut entry = child.entry.clone();
        entry.is_last = is_last;
        entry.ancestors_last = ancestors_last.to_vec();
        output.push(entry);

        if !child.children.is_empty() {
            let mut new_ancestors = ancestors_last.to_vec();
            new_ancestors.push(is_last);
            flatten_recursive(child, &new_ancestors, output);
        }
    }
}
