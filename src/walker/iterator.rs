use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use super::{TreeEntry, WinAttributes};
use crate::config::Config;
use crate::error::TreeError;
use crate::sorter;

pub struct TreeIterator {
    stack: Vec<WalkState>,
    config: Config,
    visited: HashSet<u64>,
    root_device: Option<u32>,
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
    /// Convert path to long path format on Windows if --long-paths is enabled
    #[cfg(windows)]
    fn to_long_path(path: &Path, use_long_paths: bool) -> PathBuf {
        if use_long_paths {
            let path_str = path.to_string_lossy();
            // Only add \\?\ prefix if not already present and path is absolute
            if !path_str.starts_with("\\\\?\\") && path.is_absolute() {
                let mut long_path = String::from("\\\\?\\");
                // Handle UNC paths
                if let Some(stripped) = path_str.strip_prefix("\\\\") {
                    long_path = String::from("\\\\?\\UNC\\");
                    long_path.push_str(stripped);
                    return PathBuf::from(long_path);
                }
                long_path.push_str(&path_str);
                return PathBuf::from(long_path);
            }
        }
        path.to_path_buf()
    }

    #[cfg(not(windows))]
    fn to_long_path(path: &Path, _use_long_paths: bool) -> PathBuf {
        path.to_path_buf()
    }

    pub fn new(root: &Path, config: &Config) -> Result<Self, TreeError> {
        // Convert root to long path if needed
        let root = Self::to_long_path(root, config.long_paths);
        
        let root_device = if config.one_fs {
            #[cfg(windows)]
            {
                crate::windows::attributes::get_file_id(&root)
                    .ok()
                    .map(|info| info.volume_serial)
            }
            #[cfg(not(windows))]
            {
                None
            }
        } else {
            None
        };

        let mut iterator = TreeIterator {
            stack: Vec::new(),
            config: config.clone(),
            visited: HashSet::new(),
            root_device,
        };

        // Initialize with root directory contents
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
        // Convert to long path format if needed
        let long_path = Self::to_long_path(path, self.config.long_paths);
        
        let read_dir = fs::read_dir(&long_path).map_err(|e| TreeError::Io(path.to_path_buf(), e))?;

        let mut entries: Vec<DirEntry> = read_dir
            .filter_map(|e| e.ok())
            .filter(|e| self.should_include_entry(e))
            .collect();

        sorter::sort_entries(&mut entries, &self.config.sort_config);

        Ok(entries)
    }

    /// Check if directory has any visible entries (for prune functionality)
    fn dir_has_visible_entries(&self, path: &Path) -> bool {
        if let Ok(read_dir) = fs::read_dir(path) {
            read_dir.filter_map(|e| e.ok()).any(|e| self.should_include_entry(&e))
        } else {
            false
        }
    }


    fn should_include_entry(&self, entry: &DirEntry) -> bool {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Check hidden files
        if !self.config.show_all {
            if name_str.starts_with('.') {
                return false;
            }

            #[cfg(windows)]
            {
                if let Ok(attrs_raw) =
                    crate::windows::attributes::get_file_attributes(&entry.path())
                {
                    let attrs = WinAttributes::from_raw(attrs_raw);

                    if attrs.hidden {
                        return false;
                    }
                    if self.config.hide_system && attrs.system {
                        return false;
                    }
                }
            }
        }

        // Check dirs_only
        if self.config.dirs_only {
            if let Ok(ft) = entry.file_type() {
                if !ft.is_dir() && !ft.is_symlink() {
                    return false;
                }
            }
        }

        // Apply pattern filters
        if !self.config.filter.matches(&name_str, entry.path().is_dir()) {
            return false;
        }

        true
    }

    fn should_descend(&self, path: &Path, depth: usize) -> bool {
        // Check depth limit
        if let Some(max) = self.config.max_depth {
            if depth >= max {
                return false;
            }
        }

        // Check file limit
        if let Some(limit) = self.config.file_limit {
            if let Ok(read_dir) = fs::read_dir(path) {
                if read_dir.count() > limit {
                    return false;
                }
            }
        }

        // Check prune - skip empty directories
        #[allow(clippy::collapsible_if)]
        if self.config.prune {
            if !self.dir_has_visible_entries(path) {
                return false;
            }
        }

        // Check filesystem boundary
        if let Some(root_dev) = self.root_device {
            #[cfg(windows)]
            {
                if let Ok(info) = crate::windows::attributes::get_file_id(path) {
                    if info.volume_serial != root_dev {
                        return false;
                    }
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

            // Берём индекс последнего состояния
            let idx = len - 1;

            // Проверяем границы без удержания mutable borrow
            if self.stack[idx].index >= self.stack[idx].entries.len() {
                self.stack.pop();
                continue;
            }

            // Теперь берём mutable ссылку локально
            let state = &mut self.stack[idx];

            let entry = &state.entries[state.index];
            let is_last = state.index == state.entries.len() - 1;
            state.index += 1;

            let depth = state.depth;
            let ancestors = state.ancestors_last.clone();

            if !ancestors.is_empty() || state.depth > 1 {
                // Add parent's is_last status for tree drawing
            }

            let path = entry.path();

            // Create tree entry
            let tree_entry =
                match TreeEntry::from_dir_entry(entry, state.depth, is_last, ancestors.clone()) {
                    Ok(e) => e,
                    Err(e) => return Some(Err(e)),
                };

            // Handle directory descent
            if path.is_dir() && self.should_descend(&path, depth) {
                // Check for cycles via inode
                #[allow(clippy::collapsible_if)]
                if let Some(ref meta) = tree_entry.metadata {
                    if meta.inode != 0 {
                        if !self.visited.insert(meta.inode) {
                            // Already visited, skip to avoid cycle
                            return Some(Ok(tree_entry));
                        }
                    }
                }

                // Follow symlinks if configured
                let should_follow = match tree_entry.entry_type {
                    crate::walker::EntryType::Symlink { .. } => self.config.follow_symlinks,
                    crate::walker::EntryType::Junction { .. } => self.config.show_junctions,
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

// Helper trait for config cloning (needed because Filter doesn't implement Clone directly)
impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            paths: self.paths.clone(),
            show_all: self.show_all,
            dirs_only: self.dirs_only,
            follow_symlinks: self.follow_symlinks,
            full_path: self.full_path,
            one_fs: self.one_fs,
            max_depth: self.max_depth,
            file_limit: self.file_limit,
            no_report: self.no_report,
            filter: self.filter.clone(),
            prune: self.prune,
            sort_config: self.sort_config.clone(),
            no_indent: self.no_indent,
            line_style: self.line_style,
            color_enabled: self.color_enabled,
            icons_enabled: self.icons_enabled,
            icon_style: self.icon_style,
            show_size: self.show_size,
            human_readable: self.human_readable,
            si_units: self.si_units,
            show_date: self.show_date,
            time_fmt: self.time_fmt.clone(),
            show_permissions: self.show_permissions,
            show_owner: self.show_owner,
            show_group: self.show_group,
            show_inodes: self.show_inodes,
            show_device: self.show_device,
            classify: self.classify,
            safe_print: self.safe_print,
            literal: self.literal,
            perm_mode: self.perm_mode,
            output_format: self.output_format,
            output_file: self.output_file.clone(),
            html_base: self.html_base.clone(),
            html_title: self.html_title.clone(),
            html_intro: self.html_intro.clone(),
            html_outro: self.html_outro.clone(),
            no_links: self.no_links,
            show_streams: self.show_streams,
            show_junctions: self.show_junctions,
            hide_system: self.hide_system,
            long_paths: self.long_paths,
            color_scheme: self.color_scheme.clone(),
            icon_set: self.icon_set.clone(),
        }
    }
}
