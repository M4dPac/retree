//! Shared helpers for tree traversal engines.
//!
//! Contains filtering, classification, and utility functions
//! used by both `OrderedEngine` and `StreamingEngine`.

use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::core::entry::Entry;
use crate::core::sorter::sort_entries;
use crate::core::tree::Tree;
use crate::error::TreeError;

/// Maximum internal recursion depth to prevent stack overflow.
/// Protects both sequential and parallel (8 MiB stack ≈ ~7 000 frames) modes.
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

/// Collect directory entries with optional `--filelimit` early exit.
///
/// When `file_limit` is `Some(limit)`, reads at most `limit + 1` entries
/// into memory.  If more entries exist, counts the remainder via the
/// iterator (O(1) memory) and returns `Err(total_count)`.
/// When `file_limit` is `None`, collects everything (existing behaviour).
pub fn collect_with_filelimit(
    read_dir: ReadDir,
    file_limit: Option<usize>,
) -> Result<Vec<DirEntry>, usize> {
    if let Some(limit) = file_limit {
        let mut iter = read_dir.filter_map(|e| e.ok());
        let check_count = limit.saturating_add(1);
        let entries: Vec<_> = iter.by_ref().take(check_count).collect();
        if entries.len() > limit {
            // Count remaining without storing — each DirEntry created & dropped immediately.
            let total = entries.len() + iter.count();
            return Err(total);
        }
        Ok(entries)
    } else {
        Ok(read_dir.filter_map(|e| e.ok()).collect())
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

// ──────────────────────────────────────────────
// Directory traversal shared helpers
// ──────────────────────────────────────────────

/// Result of checking whether to descend into a directory.
pub enum DescendCheck {
    /// Entry should be returned as a leaf node.
    Leaf,
    /// Same as Leaf, but also carries an error for the caller to report.
    LeafWithError(TreeError),
    /// All checks passed — caller should descend.
    Proceed,
}

/// Check junction, max_depth, and `--one-fs` conditions.
///
/// Call after creating the entry, before inserting into the visited set.
pub fn check_descend(
    entry: &Entry,
    path: &Path,
    depth: usize,
    config: &Config,
    root_device: Option<u64>,
) -> DescendCheck {
    if matches!(
        entry.entry_type,
        crate::core::entry::EntryType::Junction { .. }
    ) && !config.show_junctions
    {
        return DescendCheck::Leaf;
    }

    if let Some(max) = config.max_depth {
        if depth >= max {
            return DescendCheck::Leaf;
        }
    }

    match check_one_fs(root_device, path) {
        OnefsCheck::DifferentDevice => DescendCheck::Leaf,
        OnefsCheck::Unknown => DescendCheck::LeafWithError(TreeError::Io(
            path.to_path_buf(),
            std::io::Error::other("cannot determine volume for --one-fs"),
        )),
        OnefsCheck::Proceed => DescendCheck::Proceed,
    }
}

/// Result of reading a directory's children.
pub enum ReadDirResult {
    /// Successfully read, filtered for I/O errors, and sorted.
    Entries(Vec<DirEntry>),
    /// `read_dir` failed.
    ReadError(std::io::Error),
    /// `--filelimit` exceeded (carries total entry count).
    FilelimitExceeded(usize),
}

/// Read directory, apply `--filelimit`, and sort entries.
///
/// Combines `to_long_path` → `read_dir` → `collect_with_filelimit` → `sort_entries`.
pub fn read_sorted_children(path: &Path, config: &Config) -> ReadDirResult {
    let long_path = crate::platform::to_long_path(path, config.long_paths);
    match std::fs::read_dir(&long_path) {
        Err(e) => ReadDirResult::ReadError(e),
        Ok(rd) => match collect_with_filelimit(rd, config.file_limit) {
            Err(total) => ReadDirResult::FilelimitExceeded(total),
            Ok(mut entries) => {
                sort_entries(&mut entries, &config.sort_config);
                ReadDirResult::Entries(entries)
            }
        },
    }
}

/// Enumerate NTFS Alternate Data Streams for `path` and return them as
/// child tree nodes at the given `depth`.
///
/// On non-Windows platforms `crate::platform::get_alternate_streams`
/// returns an empty `Vec` — zero runtime cost, no `#[cfg]` needed here.
pub fn collect_ads_children(path: &Path, depth: usize) -> Vec<Tree> {
    crate::platform::get_alternate_streams(path)
        .into_iter()
        .map(|stream| Tree {
            entry: Entry::from_ads(path, stream.name, stream.size, depth),
            children: Vec::new(),
        })
        .collect()
}

/// Create a tree node for a file entry, with optional ADS children.
pub fn make_file_node(
    dir_entry: &DirEntry,
    depth: usize,
    needs_file_id: bool,
    show_permissions: bool,
    show_streams: bool,
) -> Result<Tree, TreeError> {
    let entry = Entry::from_dir_entry(
        dir_entry,
        depth,
        false,
        vec![],
        needs_file_id,
        show_permissions,
    )?;
    let stream_children = if show_streams {
        collect_ads_children(&dir_entry.path(), depth + 1)
    } else {
        Vec::new()
    };
    Ok(Tree {
        entry,
        children: stream_children,
    })
}

/// Create a leaf node marked as a recursive symlink.
pub fn make_recursive_link_node(
    dir_entry: &DirEntry,
    depth: usize,
    needs_file_id: bool,
    show_permissions: bool,
) -> Result<Tree, TreeError> {
    let mut entry = Entry::from_dir_entry(
        dir_entry,
        depth,
        false,
        vec![],
        needs_file_id,
        show_permissions,
    )?;
    entry.recursive_link = true;
    Ok(leaf_node(entry))
}
