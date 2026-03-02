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
}

/// Build a directory tree for the given path.
///
/// Single entry point for core domain logic:
/// 1. Creates root entry with required metadata
/// 2. Traverses directory tree (filtering and sorting applied internally by engine)
/// 3. Returns structured result
///
/// This function performs NO I/O output — all errors are collected
/// and returned in `BuildResult::errors` for the caller to handle.
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
    })
}
