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
use self::walker::OrderedEngine;

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
/// Uses `OrderedEngine` for both sequential and parallel modes.
/// `--max-entries` is handled by truncating during tree flattening.
pub fn build_tree(config: &Config, path: &Path) -> Result<BuildResult, TreeError> {
    let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
    let needs_attrs = config.show_permissions;

    let root = Entry::from_path(path, 0, true, vec![], needs_file_id, needs_attrs)?;

    let engine = OrderedEngine::new(config);
    let traversal = engine.traverse(path, config);

    Ok(BuildResult {
        root,
        entries: traversal.entries,
        errors: traversal.errors,
        truncated: traversal.truncated,
    })
}
