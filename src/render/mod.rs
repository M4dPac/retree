//! Render layer — output backends using the Strategy pattern.
//!
//! Provides `dispatch()` as the single entry point for rendering.

pub mod helpers;

mod html;
mod json;
mod text;
mod xml;

pub mod traits;

use html::HtmlRenderer;
use json::JsonRenderer;
pub use text::TextRenderer;
pub use traits::Renderer;
use xml::XmlRenderer;

use std::io::Write;

use crate::config::{Config, OutputFormat};
use crate::core::entry::Entry;
use crate::core::tree::Tree;
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

/// Mutable state for tree-based rendering (truncation tracking).
///
/// Shared across text, HTML, and XML renderers.
pub(crate) struct RenderState {
    pub(crate) max_entries: Option<usize>,
    pub(crate) count: usize,
    pub(crate) truncated: bool,
}

/// Generic depth-first tree walker with layout tracking.
///
/// Computes `is_last` and `ancestors_last` on the fly, handles
/// `max_entries` truncation and statistics counting.
/// Used by text and HTML renderers to avoid duplicating traversal logic.
pub(crate) fn walk_tree<F>(
    node: &Tree,
    ancestors_last: &[bool],
    stats: &mut TreeStats,
    state: &mut RenderState,
    emit: &mut F,
) -> Result<(), TreeError>
where
    F: FnMut(&Entry, bool, &[bool]) -> Result<(), TreeError>,
{
    let num_children = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        if state.max_entries.is_some_and(|max| state.count >= max) {
            state.truncated = true;
            return Ok(());
        }
        let is_last = i == num_children - 1;
        emit(&child.entry, is_last, ancestors_last)?;
        helpers::count_stats(&child.entry, stats);
        state.count += 1;
        if !child.children.is_empty() {
            let mut new_ancestors = ancestors_last.to_vec();
            new_ancestors.push(is_last);
            walk_tree(child, &new_ancestors, stats, state, emit)?;
            if state.truncated {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Dispatch rendering to the appropriate backend based on configuration.
///
/// Creates the appropriate renderer based on `config.output_format`
/// and delegates rendering to it via the `Renderer` trait.
pub fn dispatch<W: Write>(
    result: &BuildResult,
    config: &Config,
    writer: &mut W,
    stats: &mut TreeStats,
) -> Result<(), TreeError> {
    match config.output_format {
        OutputFormat::Text => {
            let renderer = TextRenderer::new();
            renderer.render(result, config, writer, stats)
        }
        OutputFormat::Html => {
            let renderer = HtmlRenderer::new(config);
            renderer.render(result, config, writer, stats)
        }
        OutputFormat::Xml => {
            let renderer = XmlRenderer::new();
            renderer.render(result, config, writer, stats)
        }
        OutputFormat::Json => {
            let renderer = JsonRenderer::new();
            renderer.render(result, config, writer, stats)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entry::{Entry, EntryType};
    use crate::core::tree::Tree;
    use crate::core::walker::TreeStats;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn leaf(name: &str, depth: usize) -> Tree {
        Tree {
            entry: Entry {
                path: PathBuf::from(name),
                name: OsString::from(name),
                entry_type: EntryType::File,
                metadata: None,
                depth,
                is_last: false,
                ancestors_last: vec![],
                filelimit_exceeded: None,
                recursive_link: false,
            },
            children: vec![],
        }
    }

    fn dir(name: &str, depth: usize, children: Vec<Tree>) -> Tree {
        Tree {
            entry: Entry {
                path: PathBuf::from(name),
                name: OsString::from(name),
                entry_type: EntryType::Directory,
                metadata: None,
                depth,
                is_last: false,
                ancestors_last: vec![],
                filelimit_exceeded: None,
                recursive_link: false,
            },
            children,
        }
    }

    #[test]
    fn walk_empty_children() {
        let tree = leaf("root", 0);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        let mut calls = vec![];
        walk_tree(
            &tree,
            &[],
            &mut stats,
            &mut state,
            &mut |entry, is_last, ancestors| {
                calls.push((entry.name_str().to_string(), is_last, ancestors.to_vec()));
                Ok(())
            },
        )
        .unwrap();
        assert!(calls.is_empty());
    }

    #[test]
    fn walk_single_child_is_last() {
        let tree = dir("root", 0, vec![leaf("a.txt", 1)]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        let mut calls = vec![];
        walk_tree(
            &tree,
            &[],
            &mut stats,
            &mut state,
            &mut |entry, is_last, ancestors| {
                calls.push((entry.name_str().to_string(), is_last, ancestors.to_vec()));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], ("a.txt".into(), true, vec![]));
    }

    #[test]
    fn walk_two_children_is_last_flag() {
        let tree = dir("root", 0, vec![leaf("a", 1), leaf("b", 1)]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        let mut calls: Vec<(String, bool)> = vec![];
        walk_tree(
            &tree,
            &[],
            &mut stats,
            &mut state,
            &mut |entry, is_last, _| {
                calls.push((entry.name_str().to_string(), is_last));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(calls, vec![("a".into(), false), ("b".into(), true)]);
    }

    #[test]
    fn walk_nested_ancestors_propagated() {
        let tree = dir("root", 0, vec![dir("sub", 1, vec![leaf("file.txt", 2)])]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        let mut calls: Vec<(String, Vec<bool>)> = vec![];
        walk_tree(
            &tree,
            &[],
            &mut stats,
            &mut state,
            &mut |entry, _is_last, ancestors| {
                calls.push((entry.name_str().to_string(), ancestors.to_vec()));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(calls[0], ("sub".into(), vec![]));
        assert_eq!(calls[1], ("file.txt".into(), vec![true]));
    }

    #[test]
    fn walk_complex_tree_layout() {
        // root/
        //   a/        (not last → ancestors [false])
        //     x       (not last in a)
        //     y       (last in a)
        //   b/        (last → ancestors [true])
        //     z       (last in b)
        let tree = dir(
            "root",
            0,
            vec![
                dir("a", 1, vec![leaf("x", 2), leaf("y", 2)]),
                dir("b", 1, vec![leaf("z", 2)]),
            ],
        );
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        let mut calls: Vec<(String, bool, Vec<bool>)> = vec![];
        walk_tree(
            &tree,
            &[],
            &mut stats,
            &mut state,
            &mut |entry, is_last, ancestors| {
                calls.push((entry.name_str().to_string(), is_last, ancestors.to_vec()));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(calls.len(), 5);
        assert_eq!(calls[0], ("a".into(), false, vec![]));
        assert_eq!(calls[1], ("x".into(), false, vec![false]));
        assert_eq!(calls[2], ("y".into(), true, vec![false]));
        assert_eq!(calls[3], ("b".into(), true, vec![]));
        assert_eq!(calls[4], ("z".into(), true, vec![true]));
    }

    #[test]
    fn walk_max_entries_truncates() {
        let tree = dir("root", 0, vec![leaf("a", 1), leaf("b", 1), leaf("c", 1)]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: Some(2),
            count: 0,
            truncated: false,
        };
        let mut names = vec![];
        walk_tree(&tree, &[], &mut stats, &mut state, &mut |entry, _, _| {
            names.push(entry.name_str().to_string());
            Ok(())
        })
        .unwrap();
        assert_eq!(names, vec!["a", "b"]);
        assert!(state.truncated);
    }

    #[test]
    fn walk_max_entries_exact_no_truncation() {
        let tree = dir("root", 0, vec![leaf("a", 1), leaf("b", 1)]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: Some(2),
            count: 0,
            truncated: false,
        };
        let mut names = vec![];
        walk_tree(&tree, &[], &mut stats, &mut state, &mut |entry, _, _| {
            names.push(entry.name_str().to_string());
            Ok(())
        })
        .unwrap();
        assert_eq!(names, vec!["a", "b"]);
        assert!(!state.truncated);
    }

    #[test]
    fn walk_max_entries_nested_truncation() {
        // root/a/x, root/a/y, root/b — max_entries=2 → a + x only
        let tree = dir(
            "root",
            0,
            vec![dir("a", 1, vec![leaf("x", 2), leaf("y", 2)]), leaf("b", 1)],
        );
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: Some(2),
            count: 0,
            truncated: false,
        };
        let mut names = vec![];
        walk_tree(&tree, &[], &mut stats, &mut state, &mut |entry, _, _| {
            names.push(entry.name_str().to_string());
            Ok(())
        })
        .unwrap();
        assert_eq!(names, vec!["a", "x"]);
        assert!(state.truncated);
    }

    #[test]
    fn walk_counts_stats() {
        let tree = dir("root", 0, vec![dir("sub", 1, vec![]), leaf("f.txt", 1)]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        walk_tree(&tree, &[], &mut stats, &mut state, &mut |_, _, _| Ok(())).unwrap();
        assert_eq!(stats.directories, 1);
        assert_eq!(stats.files, 1);
        assert_eq!(state.count, 2);
    }

    #[test]
    fn walk_emit_error_propagates() {
        let tree = dir("root", 0, vec![leaf("a", 1)]);
        let mut stats = TreeStats::default();
        let mut state = RenderState {
            max_entries: None,
            count: 0,
            truncated: false,
        };
        let result = walk_tree(&tree, &[], &mut stats, &mut state, &mut |_, _, _| {
            Err(TreeError::Generic("boom".into()))
        });
        assert!(result.is_err());
    }
}
