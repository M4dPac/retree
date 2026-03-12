//! Core domain layer — unified entry point for business logic modules.
//!
//! Provides `build_tree()` as the single façade for tree construction.
//! The core layer has NO I/O side effects — it only returns data structures.

pub mod entry;
pub mod filter;
pub mod sorter;
pub mod tree;
pub mod walker;

use std::path::Path;

use crate::config::Config;
use crate::error::TreeError;

use self::entry::Entry;
use self::walker::{OrderedEngine, TreeIterator};

/// Result of building a directory tree
#[derive(Debug)]
pub struct BuildResult {
    /// Root entry of the tree
    pub root: Entry,
    /// Flat list of all child entries in depth-first order
    pub entries: Vec<Entry>,
    /// Errors encountered during traversal
    pub errors: Vec<TreeError>,
    /// Whether the output was truncated by --max-entries
    pub truncated: bool,
}

/// Build a directory tree for the given path.
///
/// Uses the streaming `TreeIterator` when `--max-entries` is set
/// (allows early termination without building the full tree).
/// Falls back to `OrderedEngine` for parallel mode or unlimited traversal.
pub fn build_tree(config: &Config, path: &Path) -> Result<BuildResult, TreeError> {
    let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
    let needs_attrs = config.show_permissions;

    let root = Entry::from_path(path, 0, true, vec![], needs_file_id, needs_attrs)?;

    // Use streaming iterator when max_entries is set (avoids full tree in memory)
    if let Some(max) = config.max_entries {
        return build_with_iterator(config, path, root, Some(max));
    }

    // Parallel mode: use OrderedEngine (requires full tree for par_iter)
    if config.parallel {
        let engine = OrderedEngine::new(config);
        let traversal = engine.traverse(path, config);
        return Ok(BuildResult {
            root,
            entries: traversal.entries,
            errors: traversal.errors,
            truncated: false,
        });
    }

    // Sequential without limit: also use streaming iterator (lower memory)
    build_with_iterator(config, path, root, None)
}

/// Build tree using the streaming TreeIterator.
fn build_with_iterator(
    config: &Config,
    path: &Path,
    root: Entry,
    max_entries: Option<usize>,
) -> Result<BuildResult, TreeError> {
    let iter = TreeIterator::new(path, config, max_entries);

    let mut entries = Vec::new();
    let mut errors = Vec::new();

    // Use a wrapper to collect the iterator, then extract errors
    let mut iter = iter;
    for result in &mut iter {
        match result {
            Ok(entry) => entries.push(entry),
            Err(e) => errors.push(e),
        }
    }

    let truncated = iter.was_truncated();
    errors.extend(iter.into_errors());

    Ok(BuildResult {
        root,
        entries,
        errors,
        truncated,
    })
}
