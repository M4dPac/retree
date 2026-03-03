//! Unified tree engine (sequential + parallel)
//! Deterministic structure, no duplicates, identical output in both modes.

use std::fs;
use std::path::Path;

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

        let root_node = if self.parallel {
            build_node_parallel(root.as_ref(), 0, config, &mut errors)
        } else {
            build_node_sequential(root.as_ref(), 0, config, &mut errors)
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
) -> Option<Node> {
    let entry = match TreeEntry::from_path(
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

        let is_dir = file_type.is_dir();

        if !config.show_all {
            if let Some(name) = dir_entry.file_name().to_str() {
                if name.starts_with('.') {
                    continue;
                }
            }
        }

        if config.dirs_only && !is_dir {
            continue;
        }

        if !filter.matches(dir_entry.file_name().to_str().unwrap_or(""), is_dir) {
            continue;
        }

        if is_dir {
            if let Some(child) = build_node_sequential(&dir_entry.path(), depth + 1, config, errors)
            {
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

    // prune: skip empty directories (exept root at depth 0)
    if config.prune && children.is_empty() && depth > 0 {
        return None;
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
) -> Option<Node> {
    let errors_mutex = Mutex::new(Vec::new());
    let result = build_node_parallel_inner(path, depth, config, &errors_mutex);
    errors.extend(errors_mutex.into_inner().unwrap_or_default());
    result
}

fn build_node_parallel_inner(
    path: &Path,
    depth: usize,
    config: &Config,
    errors: &Mutex<Vec<TreeError>>,
) -> Option<Node> {
    let entry = match TreeEntry::from_path(
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

            let is_dir = file_type.is_dir();

            if !config.show_all {
                if let Some(name) = dir_entry.file_name().to_str() {
                    if name.starts_with('.') {
                        return None;
                    }
                }
            }

            if config.dirs_only && !is_dir {
                return None;
            }

            if !filter.matches(dir_entry.file_name().to_str().unwrap_or(""), is_dir) {
                return None;
            }

            if is_dir {
                build_node_parallel_inner(&dir_entry.path(), depth + 1, config, errors)
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

    // prune: skip empty directories (exept root at depth 0)
    if config.prune && children.is_empty() && depth > 0 {
        return None;
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
