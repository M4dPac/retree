//! Unified tree engine (sequential + parallel)
//! Deterministic structure, no duplicates, identical output in both modes.

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Condvar, Mutex};

use rayon::prelude::*;

use super::common;
use crate::config::Config;
use crate::core::entry::Entry as TreeEntry;
use crate::error::TreeError;

pub use crate::core::tree::Tree as Node;

//
// ==============================
// Backpressure: DirReadLimiter
// ==============================
//

/// Limits the number of concurrent `read_dir` + collect operations
/// in parallel mode.  Provides backpressure to prevent excessive
/// file-descriptor usage and memory spikes on wide trees.
struct DirReadLimiter {
    state: Mutex<usize>,
    cvar: Condvar,
    max: usize,
}

/// RAII guard — releases one permit on drop.
struct DirReadGuard<'a>(&'a DirReadLimiter);

impl DirReadLimiter {
    fn new(max: usize) -> Self {
        Self {
            state: Mutex::new(0),
            cvar: Condvar::new(),
            max: max.max(1), // at least 1 to prevent deadlock
        }
    }

    fn acquire(&self) -> DirReadGuard<'_> {
        let mut active = self.state.lock().unwrap_or_else(|p| p.into_inner());
        while *active >= self.max {
            active = self.cvar.wait(active).unwrap_or_else(|p| p.into_inner());
        }
        *active += 1;
        DirReadGuard(self)
    }
}

impl Drop for DirReadGuard<'_> {
    fn drop(&mut self) {
        let mut active = self.0.state.lock().unwrap_or_else(|p| p.into_inner());
        *active -= 1;
        self.0.cvar.notify_one();
    }
}

//
// ==============================
// Mutex-poison-safe helpers
// ==============================
//

/// Push an error into a poisonable Mutex<Vec<TreeError>>,
/// recovering from poison instead of losing the error.
fn push_error(errors: &Mutex<Vec<TreeError>>, error: TreeError) {
    match errors.lock() {
        Ok(mut errs) => errs.push(error),
        Err(poisoned) => poisoned.into_inner().push(error),
    }
}

/// Atomically check-and-insert into the visited set.
/// Returns `true` if the path was **already** present (= cycle).
/// On poison: recovers and still inserts.
fn check_visited(visited: &Mutex<HashSet<common::VisitedKey>>, key: common::VisitedKey) -> bool {
    match visited.lock() {
        Ok(mut v) => !v.insert(key),
        Err(poisoned) => !poisoned.into_inner().insert(key),
    }
}

//
// ==============================
// Parallel traversal context
// ==============================
//

/// Shared state for parallel directory traversal.
/// Bundles references that would otherwise require 8+ function arguments.
struct ParallelCtx<'a> {
    config: &'a Config,
    errors: &'a Mutex<Vec<TreeError>>,
    visited: &'a Mutex<HashSet<common::VisitedKey>>,
    dir_limiter: &'a DirReadLimiter,
    root_device: Option<u64>,
}

//
// ==============================
// Result
// ==============================
//

/// Result of tree traversal: entries + any errors encountered
#[derive(Default)]
pub struct TraversalResult {
    pub errors: Vec<TreeError>,
    pub truncated: bool,
    /// Hierarchical tree for rendering
    pub tree: Option<Node>,
}

//
// ==============================
// Public Engine
// ==============================
//

pub struct OrderedEngine {
    parallel: bool,
    pool: Option<rayon::ThreadPool>,
    max_entries: Option<usize>,
}

impl OrderedEngine {
    pub fn new(config: &Config) -> Self {
        let pool = if config.parallel {
            let mut builder = rayon::ThreadPoolBuilder::new().stack_size(8 * 1024 * 1024); // Match main thread stck (8 MiB)
            if let Some(n) = config.threads {
                builder = builder.num_threads(n);
            }
            builder.build().ok()
        } else {
            None
        };

        Self {
            parallel: config.parallel,
            pool,
            max_entries: config.max_entries,
        }
    }

    pub fn traverse<P: AsRef<Path>>(&self, root: P, config: &Config) -> TraversalResult {
        let mut errors = Vec::new();
        let visited = HashSet::new();

        let long_root_buf = common::resolve_long_root(root.as_ref(), config.long_paths);
        let root_path = long_root_buf.as_path();

        let root_device = common::compute_root_device(config, root_path);

        if config.one_fs && root_device.is_none() {
            errors.push(TreeError::Io(
                root_path.to_path_buf(),
                std::io::Error::other(
                    "--one-fs: cannot determine root volume; cross-device check skipped",
                ),
            ));
        }

        let dir_limiter = DirReadLimiter::new(config.queue_cap.unwrap_or(64));

        let root_node = if self.parallel {
            match &self.pool {
                Some(pool) => {
                    // Create mutex-wrapped shared state for parallel workers
                    let errors_mutex = Mutex::new(Vec::new());
                    let visited_mutex = Mutex::new(visited);

                    let result = {
                        let ctx = ParallelCtx {
                            config,
                            errors: &errors_mutex,
                            visited: &visited_mutex,
                            dir_limiter: &dir_limiter,
                            root_device,
                        };
                        pool.install(|| build_node_parallel_inner(root_path, 0, &ctx, false))
                    };

                    // Recover errors from mutex (handles poison)
                    match errors_mutex.into_inner() {
                        Ok(errs) => errors.extend(errs),
                        Err(poisoned) => errors.extend(poisoned.into_inner()),
                    }
                    result
                }
                // Pool creation failed — fall back to sequential
                None => {
                    let mut visited = visited;
                    build_node_sequential(
                        root_path,
                        0,
                        config,
                        &mut errors,
                        &mut visited,
                        false,
                        root_device,
                    )
                }
            }
        } else {
            let mut visited = visited;
            build_node_sequential(
                root_path,
                0,
                config,
                &mut errors,
                &mut visited,
                false,
                root_device,
            )
        };

        let root_node = match root_node {
            Some(node) => node,
            None => {
                return TraversalResult {
                    errors,
                    truncated: false,
                    tree: None,
                };
            }
        };

        let truncated = self
            .max_entries
            .is_some_and(|max| root_node.count_nodes().saturating_sub(1) > max);

        TraversalResult {
            errors,
            truncated,
            tree: Some(root_node),
        }
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
    visited: &mut HashSet<common::VisitedKey>,
    parent_matched: bool,
    root_device: Option<u64>,
) -> Option<Node> {
    if depth >= common::MAX_INTERNAL_DEPTH {
        errors.push(TreeError::MaxDepthExceeded(path.to_path_buf()));
        return None;
    }

    let needs_file_id = common::needs_file_id(config);
    let mut entry = match TreeEntry::from_path(
        path,
        depth,
        false,
        vec![],
        needs_file_id,
        config.show_permissions,
    ) {
        Ok(e) => e,
        Err(e) => {
            errors.push(e);
            return None;
        }
    };

    match common::check_descend(&entry, path, depth, config, root_device) {
        common::DescendCheck::Leaf => return Some(common::leaf_node(entry)),
        common::DescendCheck::LeafWithError(e) => {
            errors.push(e);
            return Some(common::leaf_node(entry));
        }
        common::DescendCheck::Proceed => {}
    }

    // Track visited directories for cycle detection.
    let canon_key = common::make_visited_key(path);
    if !visited.insert(canon_key) {
        entry.recursive_link = true;
        return Some(common::leaf_node(entry));
    }

    let dir_entries = match common::read_sorted_children(path, config) {
        common::ReadDirResult::Entries(entries) => entries,
        common::ReadDirResult::ReadError(e) => {
            errors.push(TreeError::Io(path.to_path_buf(), e));
            return Some(common::leaf_node(entry));
        }
        common::ReadDirResult::FilelimitExceeded(total) => {
            entry.filelimit_exceeded = Some(total);
            return Some(common::leaf_node(entry));
        }
    };

    let mut children = Vec::new();

    for dir_entry in dir_entries {
        let is_dir = match common::filter_entry(config, &dir_entry, parent_matched) {
            common::FilterResult::Include { is_dir } => is_dir,
            common::FilterResult::Reserved => {
                errors.push(TreeError::ReservedName(dir_entry.path()));
                continue;
            }
            common::FilterResult::Exclude => continue,
        };

        if is_dir {
            // Check for recursive symlink when following
            if config.follow_symlinks && dir_entry.file_type().is_ok_and(|ft| ft.is_symlink()) {
                let symlink_key = common::make_visited_key(&dir_entry.path());
                if visited.contains(&symlink_key) {
                    match common::make_recursive_link_node(
                        &dir_entry,
                        depth + 1,
                        needs_file_id,
                        config.show_permissions,
                    ) {
                        Ok(node) => children.push(node),
                        Err(e) => errors.push(e),
                    }
                    continue;
                }
            }
            let child_name = dir_entry.file_name();
            let child_name_str = child_name.to_string_lossy();
            let child_parent_matched =
                parent_matched || config.filter.dir_matches_include(&child_name_str);

            if let Some(child) = build_node_sequential(
                &dir_entry.path(),
                depth + 1,
                config,
                errors,
                visited,
                child_parent_matched,
                root_device,
            ) {
                children.push(child);
            }
        } else {
            match common::make_file_node(
                &dir_entry,
                depth + 1,
                needs_file_id,
                config.show_permissions,
                config.show_streams,
            ) {
                Ok(node) => children.push(node),
                Err(e) => errors.push(e),
            }
        }
    }

    if common::should_prune(config, path, depth, children.is_empty()) {
        return None;
    }

    Some(Node { entry, children })
}

//
// ==============================
// Parallel Builder
// ==============================
//

fn build_node_parallel_inner(
    path: &Path,
    depth: usize,
    ctx: &ParallelCtx<'_>,
    parent_matched: bool,
) -> Option<Node> {
    if depth >= common::MAX_INTERNAL_DEPTH {
        push_error(ctx.errors, TreeError::MaxDepthExceeded(path.to_path_buf()));
        return None;
    }

    let needs_file_id = common::needs_file_id(ctx.config);
    let mut entry = match TreeEntry::from_path(
        path,
        depth,
        false,
        vec![],
        needs_file_id,
        ctx.config.show_permissions,
    ) {
        Ok(e) => e,
        Err(e) => {
            push_error(ctx.errors, e);
            return None;
        }
    };

    match common::check_descend(&entry, path, depth, ctx.config, ctx.root_device) {
        common::DescendCheck::Leaf => return Some(common::leaf_node(entry)),
        common::DescendCheck::LeafWithError(e) => {
            push_error(ctx.errors, e);
            return Some(common::leaf_node(entry));
        }
        common::DescendCheck::Proceed => {}
    }

    // Track visited directories for cycle detection.
    let canon_key = common::make_visited_key(path);
    if check_visited(ctx.visited, canon_key) {
        entry.recursive_link = true;
        return Some(common::leaf_node(entry));
    }

    // Acquire backpressure permit — limits concurrent read_dir operations.
    let dir_guard = ctx.dir_limiter.acquire();
    let dir_entries = match common::read_sorted_children(path, ctx.config) {
        common::ReadDirResult::Entries(entries) => {
            drop(dir_guard);
            entries
        }
        common::ReadDirResult::ReadError(e) => {
            push_error(ctx.errors, TreeError::Io(path.to_path_buf(), e));
            return Some(common::leaf_node(entry));
        }
        common::ReadDirResult::FilelimitExceeded(total) => {
            drop(dir_guard);
            entry.filelimit_exceeded = Some(total);
            return Some(common::leaf_node(entry));
        }
    };

    let children: Vec<Node> = dir_entries
        .par_iter()
        .filter_map(|dir_entry| {
            let is_dir = match common::filter_entry(ctx.config, dir_entry, parent_matched) {
                common::FilterResult::Include { is_dir } => is_dir,
                common::FilterResult::Reserved => {
                    push_error(ctx.errors, TreeError::ReservedName(dir_entry.path()));
                    return None;
                }
                common::FilterResult::Exclude => return None,
            };

            if is_dir {
                // Check for recursive symlink when following — atomic check
                if ctx.config.follow_symlinks
                    && dir_entry.file_type().is_ok_and(|ft| ft.is_symlink())
                {
                    let symlink_key = common::make_visited_key(&dir_entry.path());
                    let is_visited = match ctx.visited.lock() {
                        Ok(v) => v.contains(&symlink_key),
                        Err(poisoned) => poisoned.into_inner().contains(&symlink_key),
                    };
                    if is_visited {
                        return match common::make_recursive_link_node(
                            dir_entry,
                            depth + 1,
                            needs_file_id,
                            ctx.config.show_permissions,
                        ) {
                            Ok(node) => Some(node),
                            Err(e) => {
                                push_error(ctx.errors, e);
                                None
                            }
                        };
                    }
                }
                let child_name = dir_entry.file_name();
                let child_name_str = child_name.to_string_lossy();
                let child_parent_matched =
                    parent_matched || ctx.config.filter.dir_matches_include(&child_name_str);
                build_node_parallel_inner(&dir_entry.path(), depth + 1, ctx, child_parent_matched)
            } else {
                match common::make_file_node(
                    dir_entry,
                    depth + 1,
                    needs_file_id,
                    ctx.config.show_permissions,
                    ctx.config.show_streams,
                ) {
                    Ok(node) => Some(node),
                    Err(e) => {
                        push_error(ctx.errors, e);
                        None
                    }
                }
            }
        })
        .collect();

    if common::should_prune(ctx.config, path, depth, children.is_empty()) {
        return None;
    }

    Some(Node { entry, children })
}
