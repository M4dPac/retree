//! Unified tree engine (sequential + parallel)
//! Deterministic structure, no duplicates, identical output in both modes.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::config::Config;
use crate::core::entry::Entry as TreeEntry;
use crate::core::sorter::sort_entries;
use crate::error::TreeError;

pub use crate::core::tree::Tree as Node;

/// Result of tree traversal: entries + any errors encountered
#[derive(Default)]
pub struct TraversalResult {
    pub entries: Vec<TreeEntry>,
    pub errors: Vec<TreeError>,
}

//
// ==============================
// Public Engine
// ==============================
//

pub struct OrderedEngine {
    #[allow(dead_code)]
    threads: usize,
    parallel: bool,
}

impl OrderedEngine {
    pub fn new(config: &Config) -> Self {
        Self {
            threads: config.threads.unwrap_or_else(num_cpus::get),
            parallel: config.parallel,
        }
    }

    pub fn traverse<P: AsRef<Path>>(&self, root: P, config: &Config) -> TraversalResult {
        let mut errors = Vec::new();

        let mut visited = HashSet::new();
        let root_node = if self.parallel {
            build_node_parallel(root.as_ref(), 0, config, &mut errors, visited, false)
        } else {
            build_node_sequential(root.as_ref(), 0, config, &mut errors, &mut visited, false)
        };

        let root_node = match root_node {
            Some(node) => node,
            None => {
                return TraversalResult {
                    entries: Vec::new(),
                    errors,
                };
            }
        };

        let mut entries = Vec::new();
        flatten_tree(&root_node, &[], &mut entries);

        TraversalResult { entries, errors }
    }
}

//
// ==============================
// Sequential Builder
// ==============================
//

fn build_node_sequential(
    path: &Path,
    depth: usize,
    config: &Config,
    errors: &mut Vec<TreeError>,
    visited: &mut HashSet<PathBuf>,
    parent_matched: bool,
) -> Option<Node> {
    let mut entry = match TreeEntry::from_path(
        path,
        depth,
        false,
        vec![],
        config.show_inodes,
        config.show_permissions,
    ) {
        Ok(e) => e,
        Err(e) => {
            errors.push(e);
            return None;
        }
    };

    // Track visited directories for cycle detection (junctions, symlinks, mount points)
    let canon_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canon_key) {
        entry.recursive_link = true;
        return Some(Node {
            entry,
            children: Vec::new(),
        });
    }

    if let Some(max) = config.max_depth {
        if depth >= max {
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            errors.push(TreeError::Io(path.to_path_buf(), e));
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    };

    let mut dir_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    sort_entries(&mut dir_entries, &config.sort_config);

    // filelimit: skip directories with too many entries
    if let Some(limit) = config.file_limit {
        if dir_entries.len() > limit {
            entry.filelimit_exceeded = Some(dir_entries.len());
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    }

    let filter = config.filter.clone();

    let mut children = Vec::new();

    for dir_entry in dir_entries {
        let file_type = match dir_entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                errors.push(TreeError::Io(dir_entry.path(), e));
                continue;
            }
        };

        let is_dir = file_type.is_dir()
            || (config.follow_symlinks && file_type.is_symlink() && dir_entry.path().is_dir());

        if !config.show_all {
            if let Some(name) = dir_entry.file_name().to_str() {
                if name.starts_with('.') {
                    continue;
                }
            }
        }

        // dirs_only: include directories and symlinks to directories
        if config.dirs_only {
            let is_symlink_to_dir = file_type.is_symlink() && dir_entry.path().is_dir();
            if !is_dir && !is_symlink_to_dir {
                continue;
            }
        }

        // prune: symlinks to directories are "empty" when not followed — skip them
        if config.prune
            && !config.follow_symlinks
            && file_type.is_symlink()
            && dir_entry.path().is_dir()
        {
            continue;
        }

        let name_str = dir_entry.file_name();
        let name = name_str.to_str().unwrap_or("");
        // -I always excludes matching entries
        if filter.excluded(name) {
            continue;
        }
        // Files: -P (include) applies, unless parent dir matched with --matchdirs
        if !is_dir && !parent_matched && !filter.matches(name, false) {
            continue;
        }

        if is_dir {
            // Check for recursive symlink when following
            if config.follow_symlinks && file_type.is_symlink() {
                if let Ok(canon) = dir_entry.path().canonicalize() {
                    if visited.contains(&canon) {
                        match TreeEntry::from_dir_entry(
                            &dir_entry,
                            depth + 1,
                            false,
                            vec![],
                            config.show_inodes,
                            config.show_permissions,
                        ) {
                            Ok(mut entry) => {
                                entry.recursive_link = true;
                                children.push(Node {
                                    entry,
                                    children: Vec::new(),
                                });
                            }
                            Err(e) => {
                                errors.push(e);
                            }
                        }
                        continue;
                    }
                }
            }
            let child_parent_matched = parent_matched || filter.dir_matches_include(name);
            if let Some(child) = build_node_sequential(
                &dir_entry.path(),
                depth + 1,
                config,
                errors,
                visited,
                child_parent_matched,
            ) {
                children.push(child);
            }
        } else {
            match TreeEntry::from_dir_entry(
                &dir_entry,
                depth + 1,
                false,
                vec![],
                config.show_inodes,
                config.show_permissions,
            ) {
                Ok(entry) => {
                    children.push(Node {
                        entry,
                        children: Vec::new(),
                    });
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    }

    // prune: skip empty directories (except root at depth 0)
    // With --matchdirs, directories matching -P pattern are protected from pruning
    if config.prune && children.is_empty() && depth > 0 {
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !config.filter.dir_matches_include(dir_name) {
            return None;
        }
    }

    Some(Node { entry, children })
}

//
// ==============================
// Parallel Builder
// ==============================
//

use std::sync::Mutex;

fn build_node_parallel(
    path: &Path,
    depth: usize,
    config: &Config,
    errors: &mut Vec<TreeError>,
    visited: HashSet<PathBuf>,
    parent_matched: bool,
) -> Option<Node> {
    let errors_mutex = Mutex::new(Vec::new());
    let visited_mutex = Mutex::new(visited);
    let result = build_node_parallel_inner(
        path,
        depth,
        config,
        &errors_mutex,
        &visited_mutex,
        parent_matched,
    );
    errors.extend(errors_mutex.into_inner().unwrap_or_default());
    result
}

fn build_node_parallel_inner(
    path: &Path,
    depth: usize,
    config: &Config,
    errors: &Mutex<Vec<TreeError>>,
    visited: &Mutex<HashSet<PathBuf>>,
    parent_matched: bool,
) -> Option<Node> {
    let mut entry = match TreeEntry::from_path(
        path,
        depth,
        false,
        vec![],
        config.show_inodes,
        config.show_permissions,
    ) {
        Ok(e) => e,
        Err(e) => {
            if let Ok(mut errs) = errors.lock() {
                errs.push(e);
            }
            return None;
        }
    };

    // Track visited directories for cycle detection (junctions, symlinks, mount points)
    let canon_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let already_visited = visited
        .lock()
        .map(|mut v| !v.insert(canon_key))
        .unwrap_or(false);
    if already_visited {
        entry.recursive_link = true;
        return Some(Node {
            entry,
            children: Vec::new(),
        });
    }

    if let Some(max) = config.max_depth {
        if depth >= max {
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            if let Ok(mut errs) = errors.lock() {
                errs.push(TreeError::Io(path.to_path_buf(), e));
            }
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    };

    let mut dir_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    sort_entries(&mut dir_entries, &config.sort_config);

    // filelimit: skip directories with too many entries
    if let Some(limit) = config.file_limit {
        if dir_entries.len() > limit {
            entry.filelimit_exceeded = Some(dir_entries.len());
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    }

    let filter = config.filter.clone();

    let children: Vec<Node> = dir_entries
        .par_iter()
        .filter_map(|dir_entry| {
            let file_type = match dir_entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    if let Ok(mut errs) = errors.lock() {
                        errs.push(TreeError::Io(dir_entry.path(), e));
                    }
                    return None;
                }
            };

            let is_dir = file_type.is_dir()
                || (config.follow_symlinks && file_type.is_symlink() && dir_entry.path().is_dir());

            if !config.show_all {
                if let Some(name) = dir_entry.file_name().to_str() {
                    if name.starts_with('.') {
                        return None;
                    }
                }
            }

            // dirs_only: include directories and symlinks to directories
            if config.dirs_only {
                let is_symlink_to_dir = file_type.is_symlink() && dir_entry.path().is_dir();
                if !is_dir && !is_symlink_to_dir {
                    return None;
                }
            }

            // prune: symlinks to directories are "empty" when not followed — skip them
            if config.prune
                && !config.follow_symlinks
                && file_type.is_symlink()
                && dir_entry.path().is_dir()
            {
                return None;
            }

            let name_str = dir_entry.file_name();
            let name = name_str.to_str().unwrap_or("");
            // -I always excludes matching entries
            if filter.excluded(name) {
                return None;
            }
            // Files: -P (include) applies, unless parent dir matched with --matchdirs
            if !is_dir && !parent_matched && !filter.matches(name, false) {
                return None;
            }

            if is_dir {
                // Check for recursive symlink when following
                if config.follow_symlinks && file_type.is_symlink() {
                    if let Ok(canon) = dir_entry.path().canonicalize() {
                        let is_recursive =
                            visited.lock().map(|v| v.contains(&canon)).unwrap_or(true);
                        if is_recursive {
                            return match TreeEntry::from_dir_entry(
                                dir_entry,
                                depth + 1,
                                false,
                                vec![],
                                config.show_inodes,
                                config.show_permissions,
                            ) {
                                Ok(mut entry) => {
                                    entry.recursive_link = true;
                                    Some(Node {
                                        entry,
                                        children: Vec::new(),
                                    })
                                }
                                Err(e) => {
                                    if let Ok(mut errs) = errors.lock() {
                                        errs.push(e);
                                    }
                                    None
                                }
                            };
                        }
                    }
                }
                let child_parent_matched = parent_matched || filter.dir_matches_include(name);
                build_node_parallel_inner(
                    &dir_entry.path(),
                    depth + 1,
                    config,
                    errors,
                    visited,
                    child_parent_matched,
                )
            } else {
                match TreeEntry::from_dir_entry(
                    dir_entry,
                    depth + 1,
                    false,
                    vec![],
                    config.show_inodes,
                    config.show_permissions,
                ) {
                    Ok(entry) => Some(Node {
                        entry,
                        children: Vec::new(),
                    }),
                    Err(e) => {
                        if let Ok(mut errs) = errors.lock() {
                            errs.push(e);
                        }
                        None
                    }
                }
            }
        })
        .collect();

    // prune: skip empty directories (except root at depth 0)
    // With --matchdirs, directories matching -P pattern are protected from pruning
    if config.prune && children.is_empty() && depth > 0 {
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !config.filter.dir_matches_include(dir_name) {
            return None;
        }
    }

    Some(Node { entry, children })
}

fn flatten_tree(node: &Node, ancestors_last: &[bool], output: &mut Vec<TreeEntry>) {
    let num_children = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == num_children - 1;

        let mut entry = child.entry.clone();
        entry.is_last = is_last;
        entry.ancestors_last = ancestors_last.to_vec();
        output.push(entry);

        if !child.children.is_empty() {
            let mut new_ancestors = ancestors_last.to_vec();
            new_ancestors.push(is_last);
            flatten_tree(child, &new_ancestors, output);
        }
    }
}
