use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use rustc_hash::FxHashSet;

use crate::config::Config;
use crate::core::entry::{Entry as TreeEntry, EntryType, WinAttributes};
use crate::core::sorter;
use crate::error::TreeError;

pub struct TreeIterator {
    stack: Vec<WalkState>,
    config: Config,
    visited: FxHashSet<u64>,
    root_device: Option<u64>,
}

struct WalkState {
    entries: Vec<DirEntry>,
    index: usize,
    depth: usize,
    #[allow(dead_code)]
    parent_path: PathBuf,
    ancestors_last: Vec<bool>,
}

impl TreeIterator {
    pub fn new(root: &Path, config: &Config) -> Result<Self, TreeError> {
        // Convert root to long path if needed (no-op on non-Windows)
        let root = crate::platform::to_long_path(root, config.long_paths);

        let root_device = if config.one_fs {
            crate::platform::get_file_id(&root).map(|info| info.volume_serial)
        } else {
            None
        };

        let mut visited = FxHashSet::default();
        visited.reserve(1024);

        let mut iterator = TreeIterator {
            stack: Vec::new(),
            config: config.clone(),
            visited,
            root_device,
        };

        if root.is_dir() {
            if let Ok(entries) = iterator.read_and_sort_dir(&root) {
                if !entries.is_empty() {
                    iterator.stack.push(WalkState {
                        entries,
                        index: 0,
                        depth: 1,
                        parent_path: root.to_path_buf(),
                        ancestors_last: vec![],
                    });
                }
            }
        }

        Ok(iterator)
    }

    fn read_and_sort_dir(&self, path: &Path) -> Result<Vec<DirEntry>, TreeError> {
        let long_path = crate::platform::to_long_path(path, self.config.long_paths);

        let read_dir =
            fs::read_dir(&long_path).map_err(|e| TreeError::Io(path.to_path_buf(), e))?;

        let mut entries: Vec<DirEntry> = read_dir
            .filter_map(|e| e.ok())
            .filter(|e| self.should_include_entry(e))
            .collect();

        sorter::sort_entries(&mut entries, &self.config.sort_config);

        Ok(entries)
    }

    fn dir_has_visible_entries(&self, path: &Path) -> bool {
        if let Ok(read_dir) = fs::read_dir(path) {
            read_dir
                .filter_map(|e| e.ok())
                .any(|e| self.should_include_entry(&e))
        } else {
            false
        }
    }

    fn should_include_entry(&self, entry: &DirEntry) -> bool {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if !self.config.show_all {
            if name_str.starts_with('.') {
                return false;
            }

            // Check Windows hidden/system attributes via platform facade
            if self.config.hide_system {
                if let Some(attrs_raw) = crate::platform::get_file_attributes_raw(&entry.path()) {
                    let attrs = WinAttributes::from_raw(attrs_raw);
                    if attrs.hidden || attrs.system {
                        return false;
                    }
                }
            }
        }

        if self.config.dirs_only {
            if let Ok(ft) = entry.file_type() {
                // Include directories and symlinks pointing to directories
                let is_dir_like = ft.is_dir() || (ft.is_symlink() && entry.path().is_dir());
                if !is_dir_like {
                    return false;
                }
            }
        }

        if !self.config.filter.matches(&name_str, entry.path().is_dir()) {
            return false;
        }

        true
    }

    fn should_descend(&self, path: &Path, depth: usize) -> bool {
        if let Some(max) = self.config.max_depth {
            if depth >= max {
                return false;
            }
        }

        if let Some(limit) = self.config.file_limit {
            if let Ok(read_dir) = fs::read_dir(path) {
                if read_dir.count() > limit {
                    return false;
                }
            }
        }

        if self.config.prune && !self.dir_has_visible_entries(path) {
            return false;
        }

        // Check filesystem boundary via platform facade
        if let Some(root_dev) = self.root_device {
            if let Some(info) = crate::platform::get_file_id(path) {
                if info.volume_serial != root_dev {
                    return false;
                }
            }
        }

        true
    }
}

impl Iterator for TreeIterator {
    type Item = Result<TreeEntry, TreeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let len = self.stack.len();
            if len == 0 {
                return None;
            }

            let idx = len - 1;

            if self.stack[idx].index >= self.stack[idx].entries.len() {
                self.stack.pop();
                continue;
            }

            let state = &mut self.stack[idx];

            let entry = &state.entries[state.index];
            let is_last = state.index == state.entries.len() - 1;
            state.index += 1;

            let depth = state.depth;
            let ancestors = state.ancestors_last.clone();

            let path = entry.path();

            let needs_file_id =
                self.config.one_fs || self.config.show_inodes || self.config.show_device;
            let needs_attrs = self.config.show_permissions;

            let tree_entry = match TreeEntry::from_dir_entry(
                entry,
                state.depth,
                is_last,
                ancestors.clone(),
                needs_file_id,
                needs_attrs,
            ) {
                Ok(e) => e,
                Err(e) => return Some(Err(e)),
            };

            if path.is_dir() && self.should_descend(&path, depth) {
                if let Some(ref meta) = tree_entry.metadata {
                    if meta.inode != 0 && !self.visited.insert(meta.inode) {
                        return Some(Ok(tree_entry));
                    }
                }

                let should_follow = match tree_entry.entry_type {
                    EntryType::Symlink { .. } => self.config.follow_symlinks,
                    EntryType::Junction { .. } => self.config.show_junctions,
                    _ => true,
                };

                if should_follow {
                    if let Ok(children) = self.read_and_sort_dir(&path) {
                        if !children.is_empty() {
                            let mut new_ancestors = ancestors;
                            new_ancestors.push(is_last);

                            self.stack.push(WalkState {
                                entries: children,
                                index: 0,
                                depth: depth + 1,
                                parent_path: path,
                                ancestors_last: new_ancestors,
                            });
                        }
                    }
                }
            }

            return Some(Ok(tree_entry));
        }
    }
}
