//! Streaming tree traversal — renders output during DFS walk.
//!
//! Unlike `OrderedEngine` which builds a full tree in memory,
//! `StreamingEngine` writes entries directly to output as they are visited.
//! This reduces memory usage for large directory trees.

use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::core::entry::Entry;
use crate::core::walker::TreeStats;
use crate::error::TreeError;
use crate::i18n;

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
    /// Returns non-fatal traversal errors.
    pub fn traverse_and_render<W: Write>(
        &self,
        root: &Path,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<Vec<TreeError>, TreeError> {
        let config = self.config;
        let errors = Vec::new();

        let needs_file_id = config.one_fs || config.show_inodes || config.show_device;

        // Create and emit root entry
        let root_entry = Entry::from_path(
            root,
            0,
            false,
            vec![],
            needs_file_id,
            config.show_permissions,
        )?;

        writeln!(writer, "{}", root_entry.name_str())?;
        stats.directories += 1;

        // TODO: Phase 6b — DFS traversal of children

        // Report line
        if !config.no_report {
            writeln!(writer)?;
            let report = i18n::format_report(
                i18n::current(),
                stats.directories.saturating_sub(1),
                stats.files,
            );
            writeln!(writer, "{}", report)?;
        }

        Ok(errors)
    }
}

