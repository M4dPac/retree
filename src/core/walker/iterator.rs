//! Stack-based streaming tree iterator.
//!
//! Yields directory entries one by one without building the full tree
//! in memory.  Used as the sequential backend and when `--max-entries`
//! is set.
//!
//! Compared to the recursive engine:
//! - Memory: O(depth × width) vs O(total_nodes)
//! - No OS stack overflow risk (uses heap-allocated stack)
//! - Supports `--max-entries` for early termination
//! - Prune: single-level check (does not cascade through empty subtrees)

use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::core::entry::Entry as TreeEntry;
use crate::core::sorter;
use crate::error::TreeError;

/// Maximum internal depth to prevent pathological traversal.
/// Stack-based iteration has no OS stack risk, but this prevents
/// runaway traversal of adversarial directory structures.
const MAX_INTERNAL_DEPTH: usize = 4096;

/// Streaming tree iterator that yields entries in depth-first order.
pub struct TreeIterator {
    stack: Vec<WalkState>,
    config: Config,
    visited: HashSet<PathBuf>,
    root_device: Option<u64>,
    errors: Vec<TreeError>,
    emitted: usize,
    max_entries: Option<usize>,
}

/// State for one directory level on the traversal stack.
struct WalkState {
    /// Pre-filtered and sorted entries in this directory.
    entries: Vec<DirEntry>,
    /// Current position within `entries`.
    index: usize,
    /// Depth of entries in this state (1 = root's children).
    depth: usize,
    /// is_last flags of all ancestor entries (for tree drawing).
    ancestors_last: Vec<bool>,
    /// Whether a parent directory matched the -P include pattern.
    parent_matched: bool,
}

impl TreeIterator {
    /// Create a new iterator rooted at `root`.
    ///
    /// `max_entries` limits the total number of entries yielded.
    /// Pass `None` for unlimited traversal.
    pub fn new(root: &Path, config: &Config, max_entries: Option<usize>) -> Self {
        let long_root = crate::platform::to_long_path(root, config.long_paths);

        let root_device = if config.one_fs {
            crate::platform::get_file_id(&long_root).map(|info| info.volume_serial)
        } else {
            None
        };

        let mut visited = HashSet::new();
        let root_canon = long_root
            .canonicalize()
            .unwrap_or_else(|_| long_root.to_path_buf());
        visited.insert(root_canon);

        let mut iter = Self {
            stack: Vec::new(),
            config: config.clone(),
            visited,
            root_device,
            errors: Vec::new(),
            emitted: 0,
            max_entries,
        };

        if long_root.is_dir() {
            match read_and_sort_filtered(&long_root, config, false) {
                Ok((filtered, _total)) => {
                    if !filtered.is_empty() {
                        iter.stack.push(WalkState {
                            entries: filtered,
                            index: 0,
                            depth: 1,
                            ancestors_last: vec![],
                            parent_matched: false,
                        });
                    }
                }
                Err(e) => iter.errors.push(e),
            }
        }

        iter
    }

    /// Consume the iterator and return all collected errors.
    pub fn into_errors(self) -> Vec<TreeError> {
        self.errors
    }

    /// Whether iteration stopped due to `max_entries` limit.
    pub fn was_truncated(&self) -> bool {
        self.max_entries.is_some_and(|max| self.emitted >= max)
    }
}

impl Iterator for TreeIterator {
    type Item = Result<TreeEntry, TreeError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check max_entries limit
        if let Some(max) = self.max_entries {
            if self.emitted >= max {
                return None;
            }
        }

        loop {
            let state = self.stack.last_mut()?;

            if state.index >= state.entries.len() {
                self.stack.pop();
                continue;
            }

            let dir_entry = &state.entries[state.index];
            let depth = state.depth;
            let parent_matched = state.parent_matched;
            let ancestors = state.ancestors_last.clone();
            let is_last = state.index == state.entries.len() - 1;
            state.index += 1;

            // Determine entry type
            let file_type = match dir_entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    self.errors.push(TreeError::Io(dir_entry.path(), e));
                    continue;
                }
            };

            let path = dir_entry.path();
            let is_dir = file_type.is_dir()
                || (self.config.follow_symlinks && file_type.is_symlink() && path.is_dir());

            let needs_file_id =
                self.config.one_fs || self.config.show_inodes || self.config.show_device;

            // Build tree entry
            let mut entry = match TreeEntry::from_dir_entry(
                dir_entry,
                depth,
                is_last,
                ancestors.clone(),
                needs_file_id,
                self.config.show_permissions,
            ) {
                Ok(e) => e,
                Err(e) => return Some(Err(e)),
            };

            // ── File: emit immediately ──
            if !is_dir {
                self.emitted += 1;
                return Some(Ok(entry));
            }

            // ── Directory handling ──

            // Internal depth limit
            if depth >= MAX_INTERNAL_DEPTH {
                self.errors.push(TreeError::MaxDepthExceeded(path.clone()));
                self.emitted += 1;
                return Some(Ok(entry));
            }

            // --level N
            if let Some(max) = self.config.max_depth {
                if depth >= max {
                    self.emitted += 1;
                    return Some(Ok(entry));
                }
            }

            // --one-fs: skip directories on different volumes
            if let Some(root_dev) = self.root_device {
                if let Some(info) = crate::platform::get_file_id(&path) {
                    if info.volume_serial != root_dev {
                        self.emitted += 1;
                        return Some(Ok(entry));
                    }
                }
            }

            // Junction: show in listing but don't descend unless --show-junctions
            if matches!(
                entry.entry_type,
                crate::core::entry::EntryType::Junction { .. }
            ) && !self.config.show_junctions
            {
                self.emitted += 1;
                return Some(Ok(entry));
            }

            // Cycle detection via canonicalized path
            let canon = path.canonicalize().unwrap_or_else(|_| path.clone());

            // Symlink following: read-only pre-check (matches engine.rs parallel logic)
            if self.config.follow_symlinks
                && file_type.is_symlink()
                && self.visited.contains(&canon)
            {
                entry.recursive_link = true;
                self.emitted += 1;
                return Some(Ok(entry));
            }

            // General cycle detection: insert into visited set
            if !self.visited.insert(canon) {
                entry.recursive_link = true;
                self.emitted += 1;
                return Some(Ok(entry));
            }

            // Compute child_parent_matched for --matchdirs
            let name_os = dir_entry.file_name();
            let name = name_os.to_string_lossy();
            let child_parent_matched =
                parent_matched || self.config.filter.dir_matches_include(&name);

            // Read, sort, and filter children
            match read_and_sort_filtered(&path, &self.config, child_parent_matched) {
                Ok((filtered, total)) => {
                    // --filelimit: based on total (unfiltered) count
                    if let Some(limit) = self.config.file_limit {
                        if total > limit {
                            entry.filelimit_exceeded = Some(total);
                            self.emitted += 1;
                            return Some(Ok(entry));
                        }
                    }

                    // --prune: skip directory if no visible children (depth > 0)
                    if self.config.prune
                        && filtered.is_empty()
                        && depth > 0
                        && !self.config.filter.dir_matches_include(&name)
                    {
                        continue; // skip this directory entirely
                    }

                    // Push children onto traversal stack
                    if !filtered.is_empty() {
                        let mut new_ancestors = ancestors;
                        new_ancestors.push(is_last);

                        self.stack.push(WalkState {
                            entries: filtered,
                            index: 0,
                            depth: depth + 1,
                            ancestors_last: new_ancestors,
                            parent_matched: child_parent_matched,
                        });
                    }
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }

            self.emitted += 1;
            return Some(Ok(entry));
        }
    }
}

// ══════════════════════════════════════
// Free helper functions
// ══════════════════════════════════════

/// Read a directory, sort entries, and filter by visibility rules.
///
/// Returns `(filtered_entries, total_readable_count)`.
/// `total_readable_count` is before content filtering (used for `--filelimit`).
fn read_and_sort_filtered(
    path: &Path,
    config: &Config,
    parent_matched: bool,
) -> Result<(Vec<DirEntry>, usize), TreeError> {
    let long_path = crate::platform::to_long_path(path, config.long_paths);
    let read_dir = fs::read_dir(&long_path).map_err(|e| TreeError::Io(path.to_path_buf(), e))?;

    let mut entries: Vec<DirEntry> = read_dir.filter_map(|e| e.ok()).collect();
    let total = entries.len();

    sorter::sort_entries(&mut entries, &config.sort_config);

    let filtered = entries
        .into_iter()
        .filter(|e| should_include(config, e, parent_matched))
        .collect();

    Ok((filtered, total))
}

/// Check if an entry passes all visibility and filter rules.
fn should_include(config: &Config, dir_entry: &DirEntry, parent_matched: bool) -> bool {
    let name_os = dir_entry.file_name();
    let name = name_os.to_string_lossy();

    // Hidden files
    if !config.show_all && name.starts_with('.') {
        return false;
    }

    let file_type = match dir_entry.file_type() {
        Ok(ft) => ft,
        Err(_) => return false,
    };

    let is_dir = file_type.is_dir()
        || (config.follow_symlinks && file_type.is_symlink() && dir_entry.path().is_dir());

    // --dirs-only
    if config.dirs_only {
        let is_symlink_to_dir = file_type.is_symlink() && dir_entry.path().is_dir();
        if !is_dir && !is_symlink_to_dir {
            return false;
        }
    }

    // -I: exclude pattern always applies
    if config.filter.excluded(&name) {
        return false;
    }

    // --prune: skip symlinks-to-dirs when not following
    if config.prune
        && !config.follow_symlinks
        && file_type.is_symlink()
        && dir_entry.path().is_dir()
    {
        return false;
    }

    // -P: include pattern for files (dirs always pass unless --matchdirs)
    if !is_dir && !parent_matched && !config.filter.matches(&name, false) {
        return false;
    }

    true
}
