//! Streaming tree traversal — renders output during DFS walk.
//!
//! Unlike `OrderedEngine` which builds a full tree in memory,
//! `StreamingEngine` writes entries directly to output as they are visited.
//! This reduces memory usage for large directory trees.

use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::core::walker::TreeStats;
use crate::error::TreeError;

/// Streaming tree traversal engine.
///
/// Performs DFS traversal and writes text output inline,
/// computing is_last/ancestors_last on the fly.
pub struct StreamingEngine<'a> {
    config: &'a Config,
}

impl<'a> StreamingEngine<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Traverse directory tree and write text output directly.
    ///
    /// Returns stats and any errors encountered.
    pub fn traverse_and_render<W: Write>(
        &self,
        root: &Path,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<Vec<TreeError>, TreeError> {
        // TODO: Phase 6b — full implementation
        // For now, return empty errors to signal "not implemented, use fallback"
        let _ = (root, writer, stats, self.config);
        Err(TreeError::Generic("streaming not yet implemented".into()))
    }
}
