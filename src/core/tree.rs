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

    /// Flatten tree into depth-first ordered list of entries.
    ///
    /// Used only in tests — all renderers traverse `Tree` recursively.
    #[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entry::{Entry, EntryType};
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn file(name: &str) -> Entry {
        Entry {
            path: PathBuf::from(name),
            name: OsString::from(name),
            entry_type: EntryType::File,
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        }
    }

    fn dir(name: &str) -> Entry {
        Entry {
            path: PathBuf::from(name),
            name: OsString::from(name),
            entry_type: EntryType::Directory,
            metadata: None,
            depth: 0,
            is_last: false,
            ancestors_last: vec![],
            filelimit_exceeded: None,
            recursive_link: false,
        }
    }

    // ── Tree::new ────────────────────────────────

    #[test]
    fn new_creates_leaf_node() {
        let t = Tree::new(file("test.txt"));
        assert!(t.children.is_empty());
        assert_eq!(t.entry.name, "test.txt");
    }

    #[test]
    fn new_preserves_entry_type() {
        let t = Tree::new(dir("src"));
        assert!(t.entry.entry_type.is_directory());
    }

    // ── Tree::with_children ──────────────────────

    #[test]
    fn with_children_stores_children() {
        let children = vec![Tree::new(file("a")), Tree::new(file("b"))];
        let t = Tree::with_children(dir("root"), children);
        assert_eq!(t.children.len(), 2);
        assert_eq!(t.children[0].entry.name, "a");
        assert_eq!(t.children[1].entry.name, "b");
    }

    #[test]
    fn with_children_empty_vec() {
        let t = Tree::with_children(dir("root"), vec![]);
        assert!(t.children.is_empty());
    }

    // ── count_nodes ──────────────────────────────

    #[test]
    fn count_nodes_single() {
        assert_eq!(Tree::new(file("x")).count_nodes(), 1);
    }

    #[test]
    fn count_nodes_flat() {
        let t = Tree::with_children(
            dir("root"),
            vec![
                Tree::new(file("a")),
                Tree::new(file("b")),
                Tree::new(file("c")),
            ],
        );
        assert_eq!(t.count_nodes(), 4);
    }

    #[test]
    fn count_nodes_nested() {
        let t = Tree::with_children(
            dir("root"),
            vec![
                Tree::new(file("a")),
                Tree::with_children(dir("sub"), vec![Tree::new(file("b"))]),
            ],
        );
        assert_eq!(t.count_nodes(), 4); // root + a + sub + b
    }

    #[test]
    fn count_nodes_deep_chain() {
        let t = Tree::with_children(
            dir("l0"),
            vec![Tree::with_children(
                dir("l1"),
                vec![Tree::with_children(
                    dir("l2"),
                    vec![Tree::new(file("leaf"))],
                )],
            )],
        );
        assert_eq!(t.count_nodes(), 4);
    }

    // ── flatten ──────────────────────────────────

    #[test]
    fn flatten_empty_root_returns_empty() {
        let t = Tree::new(dir("root"));
        assert!(t.flatten().is_empty());
    }

    #[test]
    fn flatten_single_child() {
        let t = Tree::with_children(dir("root"), vec![Tree::new(file("only.txt"))]);
        let flat = t.flatten();
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].name, "only.txt");
        assert!(flat[0].is_last);
        assert!(flat[0].ancestors_last.is_empty());
    }

    #[test]
    fn flatten_is_last_flags() {
        let t = Tree::with_children(
            dir("root"),
            vec![
                Tree::new(file("first")),
                Tree::new(file("middle")),
                Tree::new(file("last")),
            ],
        );
        let flat = t.flatten();
        assert!(!flat[0].is_last);
        assert!(!flat[1].is_last);
        assert!(flat[2].is_last);
    }

    #[test]
    fn flatten_depth_first_order() {
        let t = Tree::with_children(
            dir("root"),
            vec![
                Tree::with_children(dir("alpha"), vec![Tree::new(file("inside.txt"))]),
                Tree::new(file("beta.txt")),
            ],
        );
        let names: Vec<String> = t
            .flatten()
            .iter()
            .map(|e| e.name_str().into_owned())
            .collect();
        assert_eq!(names, vec!["alpha", "inside.txt", "beta.txt"]);
    }

    #[test]
    fn flatten_ancestors_last_non_last_parent() {
        // root
        // ├── dir_a (not last)
        // │   └── child.txt
        // └── dir_b (last)
        //     └── leaf.txt
        let t = Tree::with_children(
            dir("root"),
            vec![
                Tree::with_children(dir("dir_a"), vec![Tree::new(file("child.txt"))]),
                Tree::with_children(dir("dir_b"), vec![Tree::new(file("leaf.txt"))]),
            ],
        );
        let flat = t.flatten();

        // child.txt inside non-last dir_a → ancestors_last = [false]
        assert_eq!(flat[1].name, "child.txt");
        assert_eq!(flat[1].ancestors_last, vec![false]);

        // leaf.txt inside last dir_b → ancestors_last = [true]
        assert_eq!(flat[3].name, "leaf.txt");
        assert_eq!(flat[3].ancestors_last, vec![true]);
    }

    #[test]
    fn flatten_deep_ancestors_last_chain() {
        // root └── a └── b └── c
        let t = Tree::with_children(
            dir("root"),
            vec![Tree::with_children(
                dir("a"),
                vec![Tree::with_children(dir("b"), vec![Tree::new(file("c"))])],
            )],
        );
        let flat = t.flatten();
        assert_eq!(flat[2].name, "c");
        assert_eq!(flat[2].ancestors_last, vec![true, true]);
    }

    #[test]
    fn flatten_complex_ancestors() {
        // root
        // ├── d1 (not last)
        // │   ├── f1 (not last in d1)
        // │   └── f2 (last in d1)
        // └── d2 (last)
        //     └── f3 (last in d2)
        let t = Tree::with_children(
            dir("root"),
            vec![
                Tree::with_children(
                    dir("d1"),
                    vec![Tree::new(file("f1")), Tree::new(file("f2"))],
                ),
                Tree::with_children(dir("d2"), vec![Tree::new(file("f3"))]),
            ],
        );
        let flat = t.flatten();

        // f1: not last, parent d1 not last → ancestors_last=[false]
        assert_eq!(flat[1].name, "f1");
        assert!(!flat[1].is_last);
        assert_eq!(flat[1].ancestors_last, vec![false]);

        // f2: last in d1, parent d1 not last → ancestors_last=[false]
        assert_eq!(flat[2].name, "f2");
        assert!(flat[2].is_last);
        assert_eq!(flat[2].ancestors_last, vec![false]);

        // f3: last in d2, parent d2 last → ancestors_last=[true]
        assert_eq!(flat[4].name, "f3");
        assert!(flat[4].is_last);
        assert_eq!(flat[4].ancestors_last, vec![true]);
    }

    // ── Clone ────────────────────────────────────

    #[test]
    fn clone_preserves_structure() {
        let t = Tree::with_children(dir("root"), vec![Tree::new(file("child"))]);
        let c = t.clone();
        assert_eq!(c.count_nodes(), t.count_nodes());
        assert_eq!(c.entry.name, t.entry.name);
        assert_eq!(c.children.len(), t.children.len());
        assert_eq!(c.children[0].entry.name, "child");
    }
}
