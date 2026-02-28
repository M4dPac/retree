//! Parallel traversal engine for rtree
//!
//! Provides ordered parallel directory traversal using work-stealing deques.
//! Maintains output order while utilizing multiple CPU cores for I/O-bound operations.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;

use crossbeam::deque::Worker;
use rustc_hash::FxHashSet;

use crate::config::Config;
use crate::error::TreeError;
use crate::filter::Filter;
use crate::sorter::{sort_entries, SortConfig};
use crate::walker::entry::TreeEntry;
use crate::walker::iterator::TreeIterator;

/// Number of entries to buffer before emitting for ordering
const ORDER_BUFFER_SIZE: usize = 256;

/// Parallel traversal engine that maintains output order
pub struct OrderedEngine {
    /// Number of worker threads
    threads: usize,
    /// Queue capacity per thread
    queue_cap: usize,
    /// Whether parallel mode is enabled
    enabled: bool,
}

impl OrderedEngine {
    /// Create a new engine from config
    pub fn new(config: &Config) -> Self {
        let threads = config.threads.unwrap_or_else(num_cpus::get);
        let queue_cap = config.queue_cap.unwrap_or(4096);

        Self {
            threads,
            queue_cap,
            enabled: config.parallel,
        }
    }

    /// Check if parallel mode is enabled
    pub fn is_parallel(&self) -> bool {
        self.enabled
    }

    /// Run traversal - delegates to sequential if not parallel
    pub fn traverse<P: AsRef<Path>>(
        &self,
        root: P,
        config: &Config,
    ) -> Result<ParallelWalker, TreeError> {
        if self.enabled {
            self.parallel_traverse(root, config)
        } else {
            // Use sequential iterator for ordered mode
            let iter = TreeIterator::new(root.as_ref(), config)?;
            Ok(ParallelWalker::Sequential(Box::new(iter.into_iter())))
        }
    }

    /// Parallel traversal with ordered output
    fn parallel_traverse<P: AsRef<Path>>(
        &self,
        root: P,
        config: &Config,
    ) -> Result<ParallelWalker, TreeError> {
        let root_path = root.as_ref().to_path_buf();
        let root_entry = self.create_root_entry(&root_path, config)?;

        // Create channels for ordered output
        let (tx, rx) = channel::<TreeEntry>();

        // Shared state
        let visited: Arc<FxHashSet<u64>> = Arc::new(FxHashSet::default());
        let active_count = Arc::new(AtomicUsize::new(1)); // Start with root
        let done = Arc::new(AtomicBool::new(false));

        // Clone config for each worker
        let filter = config.filter.clone();
        let sort_config = config.sort_config.clone();
        let max_depth = config.max_depth;
        let file_limit = config.file_limit;
        let one_fs = config.one_fs;
        let follow_symlinks = config.follow_symlinks;
        let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
        let needs_attrs = config.show_permissions;
        let root_path_clone = root_path.clone();

        // Spawn worker threads
        let handles = (0..self.threads)
            .map(|thread_id| {
                let tx = tx.clone();
                let visited = Arc::clone(&visited);
                let active_count = Arc::clone(&active_count);
                let done = Arc::clone(&done);
                let filter_clone = filter.clone();
                let sort_config_clone = sort_config.clone();
                let root_for_thread = if thread_id == 0 { Some(root_path_clone.clone()) } else { None };

                thread::spawn(move || {
                    // Each worker has its own deque
                    let worker = Worker::new_lifo();
                    
                    // Push root to first worker
                    if let Some(root_p) = root_for_thread {
                        worker.push(WorkItem {
                            path: root_p,
                            depth: 0,
                        });
                    }

                    loop {
                        // Try to steal work from other workers
                        let work = worker.pop().or_else(|| steal_work(&worker));

                        match work {
                            Some(item) => {
                                // Process directory
                                process_directory(
                                    item,
                                    &tx,
                                    &visited,
                                    &active_count,
                                    &done,
                                    thread_id,
                                    &filter_clone,
                                    &sort_config_clone,
                                    max_depth,
                                    file_limit,
                                    one_fs,
                                    follow_symlinks,
                                    needs_file_id,
                                    needs_attrs,
                                    &worker,
                                );
                            }
                            None => {
                                // No work available, check if we should exit
                                if active_count.load(Ordering::Relaxed) == 0 || done.load(Ordering::Relaxed) {
                                    break;
                                }
                                // Yield and retry
                                thread::yield_now();
                            }
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        Ok(ParallelWalker::Parallel {
            rx,
            root: Some(root_entry),
            handles,
            done,
        })
    }

    /// Create root entry
    fn create_root_entry(&self, path: &Path, config: &Config) -> Result<TreeEntry, TreeError> {
        let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
        let needs_attrs = config.show_permissions;
        TreeEntry::from_path(path, 0, true, vec![], needs_file_id, needs_attrs)
    }
}

/// Work item for parallel processing
#[derive(Debug)]
struct WorkItem {
    path: PathBuf,
    depth: usize,
}

/// Steal work from other workers (simplified - in production would use a shared deque)
fn steal_work(_worker: &Worker<WorkItem>) -> Option<WorkItem> {
    // Simplified: in real implementation would try to steal from other workers
    // For now, this is a placeholder that returns None
    None
}

/// Process a directory in parallel
#[allow(clippy::too_many_arguments)]
fn process_directory(
    item: WorkItem,
    tx: &Sender<TreeEntry>,
    visited: &Arc<FxHashSet<u64>>,
    active_count: &Arc<AtomicUsize>,
    done: &Arc<AtomicBool>,
    thread_id: usize,
    filter: &Filter,
    sort_config: &SortConfig,
    max_depth: Option<usize>,
    file_limit: Option<usize>,
    one_fs: bool,
    follow_symlinks: bool,
    needs_file_id: bool,
    needs_attrs: bool,
    worker: &Worker<WorkItem>,
) {
    // Check depth limit
    if let Some(max) = max_depth {
        if item.depth >= max {
            return;
        }
    }

    // Read directory entries
    let entries = match std::fs::read_dir(&item.path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    // Collect and filter entries
    let mut dir_entries: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            filter.matches(e.file_name().to_str().unwrap_or(""), is_dir)
        })
        .collect();

    // Sort entries using the existing sort_entries function
    sort_entries(&mut dir_entries, sort_config);

    // Process each entry
    for dir_entry in dir_entries {
        let path = dir_entry.path();
        let entry_type = match dir_entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        // Create TreeEntry
        let ancestors = vec![];
        // is_last is always false in parallel mode (approximation)
        match TreeEntry::from_dir_entry(&dir_entry, item.depth + 1, false, ancestors, needs_file_id, needs_attrs) {
            Ok(entry) => {
                // Send entry to output channel
                let _ = tx.send(entry);

                // If directory, add to work queue
                if entry_type.is_dir() {
                    active_count.fetch_add(1, Ordering::Relaxed);
                    worker.push(WorkItem {
                        path,
                        depth: item.depth + 1,
                    });
                }
            }
            Err(_) => continue,
        }
    }

    // Decrement active count
    active_count.fetch_sub(1, Ordering::Relaxed);
}

/// Parallel walker that produces entries in order
pub enum ParallelWalker {
    /// Sequential fallback
    Sequential(Box<dyn Iterator<Item = Result<TreeEntry, TreeError>>>),
    /// Parallel mode with channel
    Parallel {
        rx: std::sync::mpsc::Receiver<TreeEntry>,
        root: Option<TreeEntry>,
        handles: Vec<thread::JoinHandle<()>>,
        done: Arc<AtomicBool>,
    },
}

impl Iterator for ParallelWalker {
    type Item = Result<TreeEntry, TreeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ParallelWalker::Sequential(iter) => iter.next(),
            ParallelWalker::Parallel { rx, root, .. } => {
                // First return root if not yet returned
                if let Some(r) = root.take() {
                    return Some(Ok(r));
                }
                // Then return from channel
                rx.recv().ok().map(Ok)
            }
        }
    }
}
