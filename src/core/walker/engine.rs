//! Unified tree engine (sequential + parallel)
//! Deterministic structure, no duplicates, identical output in both modes.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};

use rayon::prelude::*;

use crate::config::Config;
use crate::core::entry::Entry as TreeEntry;
use crate::core::sorter::sort_entries;
use crate::error::TreeError;

pub use crate::core::tree::Tree as Node;

/// Maximum internal recursion depth to prevent stack overflow.
/// Protects both sequential (8 MB stack ≈ ~7 000 frames) and
/// parallel (rayon 2 MB stack ≈ ~1 700 frames) modes.
const MAX_INTERNAL_DEPTH: usize = 4096;

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
fn check_visited(visited: &Mutex<HashSet<PathBuf>>, key: PathBuf) -> bool {
    match visited.lock() {
        Ok(mut v) => !v.insert(key),
        Err(poisoned) => !poisoned.into_inner().insert(key),
    }
}

//
// ==============================
// ADS helper
// ==============================
//

/// Enumerate NTFS Alternate Data Streams for `path` and return them as
/// child tree nodes at the given `depth`.
///
/// On non-Windows platforms `crate::platform::get_alternate_streams`
/// returns an empty `Vec` — zero runtime cost, no `#[cfg]` needed here.
fn collect_ads_children(path: &Path, depth: usize) -> Vec<Node> {
    crate::platform::get_alternate_streams(path)
        .into_iter()
        .map(|stream| Node {
            entry: TreeEntry::from_ads(path, stream.name, stream.size, depth),
            children: Vec::new(),
        })
        .collect()
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
    visited: &'a Mutex<HashSet<PathBuf>>,
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
    pub entries: Vec<TreeEntry>,
    pub errors: Vec<TreeError>,
    pub truncated: bool,
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

        let root_device = if config.one_fs {
            crate::platform::get_file_id(root.as_ref()).map(|info| info.volume_serial)
        } else {
            None
        };

        let dir_limiter = DirReadLimiter::new(config.queue_cap.unwrap_or(64));
        let root_path = root.as_ref();

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
                    entries: Vec::new(),
                    errors,
                    truncated: false,
                };
            }
        };

        let mut entries = Vec::new();
        let mut truncated = false;
        flatten_tree(
            &root_node,
            &[],
            &mut entries,
            self.max_entries,
            &mut truncated,
        );

        TraversalResult {
            entries,
            errors,
            truncated,
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
    visited: &mut HashSet<PathBuf>,
    parent_matched: bool,
    root_device: Option<u64>,
) -> Option<Node> {
    // Internal depth limit to prevent stack overflow
    if depth >= MAX_INTERNAL_DEPTH {
        errors.push(TreeError::MaxDepthExceeded(path.to_path_buf()));
        return None;
    }

    let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
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

    // Track visited directories for cycle detection (junctions, symlinks, mount points)
    let canon_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canon_key) {
        entry.recursive_link = true;
        return Some(Node {
            entry,
            children: Vec::new(),
        });
    }

    // --one-fs: skip directories on different volumes
    if let Some(root_dev) = root_device {
        if let Some(info) = crate::platform::get_file_id(path) {
            if info.volume_serial != root_dev {
                return Some(Node {
                    entry,
                    children: Vec::new(),
                });
            }
        }
    }

    // Junctions: show in listing but don't descend unless --show-junctions
    if matches!(
        entry.entry_type,
        crate::core::entry::EntryType::Junction { .. }
    ) && !config.show_junctions
    {
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

    let long_path = crate::platform::to_long_path(path, config.long_paths);
    let read_dir = match fs::read_dir(&long_path) {
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
        let name_lossy = name_str.to_string_lossy();
        let name = name_lossy.as_ref();
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
                            needs_file_id,
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
                root_device,
            ) {
                children.push(child);
            }
        } else {
            match TreeEntry::from_dir_entry(
                &dir_entry,
                depth + 1,
                false,
                vec![],
                needs_file_id,
                config.show_permissions,
            ) {
                Ok(entry) => {
                    let stream_children = if config.show_streams {
                        collect_ads_children(&dir_entry.path(), depth + 2)
                    } else {
                        Vec::new()
                    };
                    children.push(Node {
                        entry,
                        children: stream_children,
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
        let dir_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let dir_name = dir_name.as_ref();
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

fn build_node_parallel_inner(
    path: &Path,
    depth: usize,
    ctx: &ParallelCtx<'_>,
    parent_matched: bool,
) -> Option<Node> {
    // Internal depth limit to prevent stack overflow
    if depth >= MAX_INTERNAL_DEPTH {
        push_error(ctx.errors, TreeError::MaxDepthExceeded(path.to_path_buf()));
        return None;
    }

    let needs_file_id = ctx.config.one_fs || ctx.config.show_inodes || ctx.config.show_device;
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

    // Track visited directories for cycle detection (junctions, symlinks, mount points)
    let canon_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if check_visited(ctx.visited, canon_key) {
        entry.recursive_link = true;
        return Some(Node {
            entry,
            children: Vec::new(),
        });
    }

    // --one-fs: skip directories on different volumes
    if let Some(root_dev) = ctx.root_device {
        if let Some(info) = crate::platform::get_file_id(path) {
            if info.volume_serial != root_dev {
                return Some(Node {
                    entry,
                    children: Vec::new(),
                });
            }
        }
    }

    // Junctions: show in listing but don't descend unless --show-junctions
    if matches!(
        entry.entry_type,
        crate::core::entry::EntryType::Junction { .. }
    ) && !ctx.config.show_junctions
    {
        return Some(Node {
            entry,
            children: Vec::new(),
        });
    }

    if let Some(max) = ctx.config.max_depth {
        if depth >= max {
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    }

    // Acquire backpressure permit — limits concurrent read_dir operations.
    let dir_guard = ctx.dir_limiter.acquire();
    let long_path = crate::platform::to_long_path(path, ctx.config.long_paths);
    let read_dir = match fs::read_dir(&long_path) {
        Ok(rd) => rd,
        Err(e) => {
            push_error(ctx.errors, TreeError::Io(path.to_path_buf(), e));
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    };

    let mut dir_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    // Release the permit: directory handle consumed, entries in memory
    drop(dir_guard);

    sort_entries(&mut dir_entries, &ctx.config.sort_config);

    // filelimit: skip directories with too many entries
    if let Some(limit) = ctx.config.file_limit {
        if dir_entries.len() > limit {
            entry.filelimit_exceeded = Some(dir_entries.len());
            return Some(Node {
                entry,
                children: Vec::new(),
            });
        }
    }

    let filter = ctx.config.filter.clone();

    let children: Vec<Node> = dir_entries
        .par_iter()
        .filter_map(|dir_entry| {
            let file_type = match dir_entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    push_error(ctx.errors, TreeError::Io(dir_entry.path(), e));
                    return None;
                }
            };

            let is_dir = file_type.is_dir()
                || (ctx.config.follow_symlinks
                    && file_type.is_symlink()
                    && dir_entry.path().is_dir());

            if !ctx.config.show_all {
                if let Some(name) = dir_entry.file_name().to_str() {
                    if name.starts_with('.') {
                        return None;
                    }
                }
            }

            // dirs_only: include directories and symlinks to directories
            if ctx.config.dirs_only {
                let is_symlink_to_dir = file_type.is_symlink() && dir_entry.path().is_dir();
                if !is_dir && !is_symlink_to_dir {
                    return None;
                }
            }

            // prune: symlinks to directories are "empty" when not followed — skip them
            if ctx.config.prune
                && !ctx.config.follow_symlinks
                && file_type.is_symlink()
                && dir_entry.path().is_dir()
            {
                return None;
            }

            let name_str = dir_entry.file_name();
            let name_lossy = name_str.to_string_lossy();
            let name = name_lossy.as_ref();
            // -I always excludes matching entries
            if filter.excluded(name) {
                return None;
            }
            // Files: -P (include) applies, unless parent dir matched with --matchdirs
            if !is_dir && !parent_matched && !filter.matches(name, false) {
                return None;
            }

            if is_dir {
                // Check for recursive symlink when following — atomic check-and-insert
                if ctx.config.follow_symlinks && file_type.is_symlink() {
                    if let Ok(canon) = dir_entry.path().canonicalize() {
                        // Read-only check: is this symlink target already visited?
                        // Don't insert here — the actual insert happens in
                        // build_node_parallel_inner when the target directory is entered.
                        // This avoids a bug where the pre-check insert prevents the
                        // builder from descending into the target.
                        let is_visited = match ctx.visited.lock() {
                            Ok(v) => v.contains(&canon),
                            Err(poisoned) => poisoned.into_inner().contains(&canon),
                        };
                        if is_visited {
                            return match TreeEntry::from_dir_entry(
                                dir_entry,
                                depth + 1,
                                false,
                                vec![],
                                needs_file_id,
                                ctx.config.show_permissions,
                            ) {
                                Ok(mut entry) => {
                                    entry.recursive_link = true;
                                    Some(Node {
                                        entry,
                                        children: Vec::new(),
                                    })
                                }
                                Err(e) => {
                                    push_error(ctx.errors, e);
                                    None
                                }
                            };
                        }
                    }
                }
                let child_parent_matched = parent_matched || filter.dir_matches_include(name);
                build_node_parallel_inner(&dir_entry.path(), depth + 1, ctx, child_parent_matched)
            } else {
                match TreeEntry::from_dir_entry(
                    dir_entry,
                    depth + 1,
                    false,
                    vec![],
                    needs_file_id,
                    ctx.config.show_permissions,
                ) {
                    Ok(entry) => {
                        let stream_children = if ctx.config.show_streams {
                            collect_ads_children(&dir_entry.path(), depth + 2)
                        } else {
                            Vec::new()
                        };
                        Some(Node {
                            entry,
                            children: stream_children,
                        })
                    }
                    Err(e) => {
                        push_error(ctx.errors, e);
                        None
                    }
                }
            }
        })
        .collect();

    // prune: skip empty directories (except root at depth 0)
    // With --matchdirs, directories matching -P pattern are protected from pruning
    if ctx.config.prune && children.is_empty() && depth > 0 {
        let dir_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let dir_name = dir_name.as_ref();
        if !ctx.config.filter.dir_matches_include(dir_name) {
            return None;
        }
    }

    Some(Node { entry, children })
}

fn flatten_tree(
    node: &Node,
    ancestors_last: &[bool],
    output: &mut Vec<TreeEntry>,
    max_entries: Option<usize>,
    truncated: &mut bool,
) {
    let num_children = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        if max_entries.is_some_and(|max| output.len() >= max) {
            *truncated = true;
            return;
        }

        let is_last = i == num_children - 1;

        let mut entry = child.entry.clone();
        entry.is_last = is_last;
        entry.ancestors_last = ancestors_last.to_vec();
        output.push(entry);

        if !child.children.is_empty() {
            let mut new_ancestors = ancestors_last.to_vec();
            new_ancestors.push(is_last);
            flatten_tree(child, &new_ancestors, output, max_entries, truncated);
            if *truncated {
                return;
            }
        }
    }
}
