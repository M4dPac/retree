//! Shared helpers for tree traversal engines.
//!
//! Contains filtering, classification, and utility functions
//! used by both `OrderedEngine` and `StreamingEngine`.

use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::core::entry::Entry;
use crate::core::tree::Tree;

/// Maximum internal recursion depth to prevent stack overflow.
/// Protects both sequential (8 MB stack ≈ ~7 000 frames) and
/// parallel (rayon 2 MB stack ≈ ~1 700 frames) modes.
pub const MAX_INTERNAL_DEPTH: usize = 4096;

/// Key for the visited-set used by cycle detection.
///
/// Prefers OS file identity (volume serial + file ID / inode) which is
/// immune to path aliasing (junctions, `\\?\` prefix, UNC equivalents).
/// Falls back to canonicalized (then raw) path when identity is unavailable.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum VisitedKey {
    /// File identity from the OS — volume serial number and file ID (or dev + ino).
    FileId { volume: u64, file_id: u64 },
    /// Fallback: canonical or raw path.
    Path(PathBuf),
}

/// Build a visited-set key for cycle detection.
///
/// Uses [`crate::platform::get_file_id_follow`] (which resolves symlinks /
/// reparse points) to obtain an identity that is stable across path aliases.
/// On failure falls back to `canonicalize`, then to the raw path.
pub fn make_visited_key(path: &Path) -> VisitedKey {
    if let Some(info) = crate::platform::get_file_id_follow(path) {
        VisitedKey::FileId {
            volume: info.volume_serial,
            file_id: info.file_id,
        }
    } else {
        VisitedKey::Path(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
    }
}

/// Result of filtering a directory entry.
pub enum FilterResult {
    /// Entry passes all filters; `is_dir` indicates directory status.
    Include { is_dir: bool },
    /// Entry excluded by filter rules (`-a`, `-d`, `-I`, `-P`, `--prune`).
    Exclude,
    /// Entry is a Windows reserved device name — caller may report a warning.
    Reserved,
}

/// Check if configuration requires file ID lookups (inode, device, one-fs).
pub fn needs_file_id(config: &Config) -> bool {
    config.one_fs || config.show_inodes || config.show_device
}

/// Compute root device serial for `--one-fs` boundary checking.
///
/// Returns `None` if `--one-fs` is not enabled.
pub fn compute_root_device(config: &Config, root: &Path) -> Option<u64> {
    if config.one_fs {
        crate::platform::get_file_id(root).map(|info| info.volume_serial)
    } else {
        None
    }
}

/// Unified entry filter: hidden, dirs_only, reserved, -I, -P, prune-symlinks.
///
/// Evaluates all per-entry filter rules from the configuration.
/// Returns `FilterResult` so the caller can decide how to handle
/// reserved names (e.g., push a warning vs silently skip).
///
/// On `DirEntry::file_type()` failure, returns `Exclude`.
pub fn filter_entry(config: &Config, de: &DirEntry, parent_matched: bool) -> FilterResult {
    // Hidden files
    if !config.show_all && (de.file_name().as_encoded_bytes().first() == Some(&b'.')) {
        return FilterResult::Exclude;
    }

    let ft = match de.file_type() {
        Ok(ft) => ft,
        Err(_) => return FilterResult::Exclude,
    };

    let is_dir = ft.is_dir() || (config.follow_symlinks && ft.is_symlink() && de.path().is_dir());

    // dirs_only: include directories and symlinks to directories
    if config.dirs_only {
        let is_symlink_to_dir = ft.is_symlink() && de.path().is_dir();
        if !is_dir && !is_symlink_to_dir {
            return FilterResult::Exclude;
        }
    }

    // prune: symlinks to directories are "empty" when not followed — skip them
    if config.prune && !config.follow_symlinks && ft.is_symlink() && de.path().is_dir() {
        return FilterResult::Exclude;
    }

    let name_os = de.file_name();
    let name = name_os.to_string_lossy();

    // Skip Windows reserved device names (CON, NUL, PRN, …).
    if crate::platform::should_skip_reserved_name(&name) {
        return FilterResult::Reserved;
    }

    // -I always excludes matching entries
    if config.filter.excluded(&name) {
        return FilterResult::Exclude;
    }

    // -P include pattern: files only, unless parent dir matched via --matchdirs
    if !is_dir && !parent_matched && !config.filter.matches(&name, false) {
        return FilterResult::Exclude;
    }

    FilterResult::Include { is_dir }
}

// ──────────────────────────────────────────────
// Helpers extracted from engine / streaming
// ──────────────────────────────────────────────

/// Create a leaf tree node (no children).
///
/// Shorthand for the ubiquitous `Tree { entry, children: Vec::new() }`.
pub fn leaf_node(entry: Entry) -> Tree {
    Tree {
        entry,
        children: Vec::new(),
    }
}

/// Resolve root path with `--long-paths` support.
///
/// When `long_paths` is enabled and the root is relative, canonicalizes it
/// first (\\?\ prefix requires absolute paths and no `.`/`..` components).
/// Then applies `platform::to_long_path`.
pub fn resolve_long_root(root: &Path, long_paths: bool) -> PathBuf {
    let effective = if long_paths && !root.is_absolute() {
        std::fs::canonicalize(root).unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|cwd| cwd.join(root))
                .unwrap_or_else(|_| root.to_path_buf())
        })
    } else {
        root.to_path_buf()
    };
    crate::platform::to_long_path(&effective, long_paths)
}

/// Result of `--one-fs` boundary check.
pub enum OnefsCheck {
    /// Same device (or `--one-fs` not enabled) — descend normally.
    Proceed,
    /// Different device — do not descend.
    DifferentDevice,
    /// Cannot determine device — caller should emit error and not descend.
    Unknown,
}

/// Check whether `path` resides on the same filesystem as the root.
///
/// Returns `Proceed` when `root_device` is `None` (i.e. `--one-fs` disabled).
pub fn check_one_fs(root_device: Option<u64>, path: &Path) -> OnefsCheck {
    match root_device {
        None => OnefsCheck::Proceed,
        Some(root_dev) => match crate::platform::get_file_id(path) {
            Some(info) if info.volume_serial == root_dev => OnefsCheck::Proceed,
            Some(_) => OnefsCheck::DifferentDevice,
            None => OnefsCheck::Unknown,
        },
    }
}

/// Determine whether an empty directory should be pruned.
///
/// Returns `true` when `--prune` is active, the directory has no children,
/// is not the root (`depth > 0`), and does not match a `--matchdirs` pattern.
pub fn should_prune(config: &Config, path: &Path, depth: usize, children_empty: bool) -> bool {
    if !config.prune || !children_empty || depth == 0 {
        return false;
    }
    let dir_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    !config.filter.dir_matches_include(dir_name.as_ref())
}
