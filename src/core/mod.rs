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
    /// Errors encountered during traversal
    pub errors: Vec<TreeError>,
    /// Whether the output was truncated by --max-entries
    pub truncated: bool,
    /// Hierarchical tree (for future tree-based rendering)
    pub tree: Option<tree::Tree>,
}

/// Build a directory tree for the given path.
///
/// Accepts a pre-built `OrderedEngine` to allow reuse across multiple
/// paths (avoids recreating the rayon thread pool per path).
///
/// **Note:** the root `Entry` is created twice — once here (for `BuildResult.root`,
/// used by all renderers) and once inside `engine.traverse()` (as `tree.entry`,
/// used only for tree structure).  This is intentional: `root` must be available
/// even when `tree` is `None` (traversal failed after root was validated).
/// The extra `stat()` call is negligible (~1 µs).
pub fn build_tree(
    engine: &OrderedEngine,
    config: &Config,
    path: &Path,
) -> Result<BuildResult, TreeError> {
    let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
    let needs_attrs = config.show_permissions;

    let root = Entry::from_path(path, 0, needs_file_id, needs_attrs)?;

    let traversal = engine.traverse(path, config);

    Ok(BuildResult {
        root,
        errors: traversal.errors,
        truncated: traversal.truncated,
        tree: traversal.tree,
    })
}
