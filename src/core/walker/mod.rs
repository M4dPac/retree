//! Tree traversal engines.
//!
//! Three execution modes, chosen in `app::run`:
//!
//! | Mode | Engine | When | Memory | Output |
//! |------|--------|------|--------|--------|
//! | Sequential | `OrderedEngine` (pool=None) | default | O(tree) | full tree → render |
//! | Parallel  | `OrderedEngine` (pool=Some) | `--parallel` | O(tree) | full tree → render |
//! | Streaming | `StreamingEngine` | `--streaming` + text + !prune | O(depth) | DFS inline write |
//!
//! **Sequential vs Parallel** share the same `OrderedEngine` struct.
//! The parallel path uses rayon `par_iter` on each directory's sorted children,
//! preserving deterministic (sorted) output order.  `DirReadLimiter` provides
//! backpressure to bound concurrent `read_dir` calls.
//!
//! **Streaming** writes entries as they are discovered (DFS), computing
//! `is_last`/`ancestors_last` on the fly.  It uses `EntryWriter` trait to
//! decouple traversal from rendering.  Memory usage is O(depth) vs O(tree).
//!
//! `--max-entries` semantics differ: streaming stops traversal early;
//! ordered mode builds the full tree and sets a `truncated` flag for renderers.

mod common;
mod engine;
pub mod streaming;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::error::TreeError;

use std::io::Write;

pub use engine::{OrderedEngine, TraversalResult};
pub use streaming::{StreamingEngine, StreamingResult};

/// Statistics gathered during tree traversal.
#[derive(Debug, Default, Clone)]
pub struct TreeStats {
    pub directories: u64,
    pub files: u64,
    pub symlinks: u64,
    pub errors: u64,
}

/// Count an entry in the tree statistics.
///
/// Increments directories/files/symlinks counters as appropriate.
/// Used by all renderers and the streaming engine during tree traversal.
pub fn count_stats(entry: &Entry, stats: &mut TreeStats) {
    match &entry.entry_type {
        EntryType::Directory => {
            stats.directories += 1;
        }
        EntryType::Symlink { target, broken } => {
            stats.symlinks += 1;
            // GNU tree counts symlinks to directories as directories
            let points_to_dir = !broken
                && entry
                    .path
                    .parent()
                    .map(|p| p.join(target).is_dir())
                    .unwrap_or(false);

            if points_to_dir {
                stats.directories += 1;
            } else {
                stats.files += 1;
            }
        }
        _ => {
            stats.files += 1;
        }
    }
}

/// Trait for writing individual tree entries to output.
///
/// Decouples the streaming traversal engine from specific renderers.
/// The only current implementor is `TextRenderer`.
pub trait EntryWriter {
    fn write_entry(
        &self,
        writer: &mut dyn Write,
        entry: &Entry,
        config: &Config,
    ) -> Result<(), TreeError>;
}
