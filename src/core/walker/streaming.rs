//! Streaming tree traversal — renders output during DFS walk.
//!
//! Unlike `OrderedEngine` which builds a full tree in memory,
//! `StreamingEngine` writes entries directly to output as they are visited.
//! This reduces memory usage for large directory trees.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::config::{Config, LineStyle};
use crate::core::entry::{Entry, EntryType};
use crate::core::sorter::sort_entries;
use crate::core::walker::TreeStats;
use crate::error::TreeError;
use crate::i18n;

/// Result of streaming traversal.
pub struct StreamingResult {
    pub errors: Vec<TreeError>,
    pub truncated: bool,
}

/// Mutable traversal state for max_entries tracking.
struct StreamState {
    count: usize,
    max_entries: Option<usize>,
    truncated: bool,
}

impl StreamState {
    fn new(max_entries: Option<usize>) -> Self {
        Self {
            count: 0,
            max_entries,
            truncated: false,
        }
    }

    /// Check if we've reached the entry limit.
    fn should_stop(&self) -> bool {
        self.max_entries.is_some_and(|max| self.count >= max)
    }
}

/// Maximum internal recursion depth to prevent stack overflow.
const MAX_INTERNAL_DEPTH: usize = 4096;

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
    ) -> Result<StreamingResult, TreeError> {
        let config = self.config;
        let mut errors = Vec::new();
        let needs_file_id = config.one_fs || config.show_inodes || config.show_device;

        // Root entry
        let root_entry = Entry::from_path(
            root,
            0,
            false,
            vec![],
            needs_file_id,
            config.show_permissions,
        )?;
        self.write_entry(writer, &root_entry)?;
        stats.directories += 1;

        // DFS children with max_entries tracking
        let mut state = StreamState::new(config.max_entries);
        self.emit_children(
            root,
            1,
            &[],
            false,
            writer,
            stats,
            &mut errors,
            needs_file_id,
            &mut state,
        )?;

        // Report
        if !config.no_report {
            writeln!(writer)?;
            let report = i18n::format_report(
                i18n::current(),
                stats.directories.saturating_sub(1),
                stats.files,
            );
            writeln!(writer, "{}", report)?;
        }

        Ok(StreamingResult {
            errors,
            truncated: state.truncated,
        })
    }

    /// Read, filter, sort, and emit children of a single directory.
    ///
    /// Does NOT recurse into subdirectories (TODO: next phase).
    #[allow(clippy::too_many_arguments)]
    fn emit_children<W: Write>(
        &self,
        dir: &Path,
        depth: usize,
        ancestors_last: &[bool],
        parent_matched: bool,
        writer: &mut W,
        stats: &mut TreeStats,
        errors: &mut Vec<TreeError>,
        needs_file_id: bool,
        state: &mut StreamState,
    ) -> Result<(), TreeError> {
        let config = self.config;

        // Stack overflow protection
        if depth >= MAX_INTERNAL_DEPTH {
            errors.push(TreeError::MaxDepthExceeded(dir.to_path_buf()));
            return Ok(());
        }

        // max_depth: children at `depth` shown only if depth <= max.
        // Engine equivalent: parent at depth-1 checks `(depth-1) >= max`.
        if let Some(max) = config.max_depth {
            if depth > max {
                return Ok(());
            }
        }

        let long_path = crate::platform::to_long_path(dir, config.long_paths);
        let read_dir = match fs::read_dir(&long_path) {
            Ok(rd) => rd,
            Err(e) => {
                errors.push(TreeError::Io(dir.to_path_buf(), e));
                return Ok(());
            }
        };

        let mut dir_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
        sort_entries(&mut dir_entries, &config.sort_config);

        // filelimit: skip children if directory has too many raw entries
        if let Some(limit) = config.file_limit {
            if dir_entries.len() > limit {
                return Ok(());
            }
        }

        // Pre-filter to know total count (needed for is_last)
        let filtered: Vec<&fs::DirEntry> = dir_entries
            .iter()
            .filter(|de| self.should_include(de, parent_matched))
            .collect();

        let total = filtered.len();
        for (i, dir_entry) in filtered.iter().enumerate() {
            // max_entries: stop before writing
            if state.should_stop() {
                state.truncated = true;
                return Ok(());
            }

            let is_last = i == total - 1;

            match Entry::from_dir_entry(
                dir_entry,
                depth,
                is_last,
                ancestors_last.to_vec(),
                needs_file_id,
                config.show_permissions,
            ) {
                Ok(entry) => {
                    self.write_entry(writer, &entry)?;
                    count_entry_stats(&entry, stats);
                    state.count += 1;

                    // Recurse into subdirectories
                    if entry.entry_type.is_directory() {
                        let child_name = dir_entry.file_name();
                        let child_name_str = child_name.to_string_lossy();
                        let child_parent_matched =
                            parent_matched || config.filter.dir_matches_include(&child_name_str);
                        let mut child_ancestors = ancestors_last.to_vec();
                        child_ancestors.push(is_last);
                        self.emit_children(
                            &entry.path,
                            depth + 1,
                            &child_ancestors,
                            child_parent_matched,
                            writer,
                            stats,
                            errors,
                            needs_file_id,
                            state,
                        )?;
                        if state.truncated {
                            return Ok(());
                        }
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        Ok(())
    }

    /// Check whether a directory entry should be included in output.
    fn should_include(&self, de: &fs::DirEntry, parent_matched: bool) -> bool {
        let config = self.config;

        // Hidden files
        if !config.show_all {
            if let Some(name) = de.file_name().to_str() {
                if name.starts_with('.') {
                    return false;
                }
            }
        }

        // dirs_only: keep dirs and symlinks-to-dirs
        if config.dirs_only {
            let ft = match de.file_type() {
                Ok(ft) => ft,
                Err(_) => return false,
            };
            let is_dir =
                ft.is_dir() || (config.follow_symlinks && ft.is_symlink() && de.path().is_dir());
            let is_symlink_to_dir = ft.is_symlink() && de.path().is_dir();
            if !is_dir && !is_symlink_to_dir {
                return false;
            }
        }

        let name_os = de.file_name();
        let name = name_os.to_string_lossy();

        // -I exclude patterns
        if config.filter.excluded(&name) {
            return false;
        }

        // -P include pattern (files only; dirs always pass)
        let is_dir = de
            .file_type()
            .ok()
            .map(|ft| {
                ft.is_dir() || (config.follow_symlinks && ft.is_symlink() && de.path().is_dir())
            })
            .unwrap_or(false);
        if !is_dir && !parent_matched && !config.filter.matches(&name, false) {
            return false;
        }

        true
    }

    /// Write a single entry line to output.
    fn write_entry<W: Write>(&self, writer: &mut W, entry: &Entry) -> Result<(), TreeError> {
        let config = self.config;

        // Root: name (or full path with -f)
        if entry.depth == 0 {
            if config.full_path {
                writeln!(writer, "{}", entry.path.display())?;
            } else {
                writeln!(writer, "{}", entry.name_str())?;
            }
            return Ok(());
        }

        // Tree prefix
        let (branch, last_branch, vertical, space) = match config.line_style {
            LineStyle::Ascii => ("|-- ", "`-- ", "|   ", "    "),
            _ => ("├── ", "└── ", "│   ", "    "),
        };

        let mut line = String::new();
        if !config.no_indent {
            for &ancestor_last in &entry.ancestors_last {
                line.push_str(if ancestor_last { space } else { vertical });
            }
            line.push_str(if entry.is_last { last_branch } else { branch });
        }

        if config.full_path {
            line.push_str(&entry.path.display().to_string());
        } else {
            line.push_str(entry.name_str());
        }

        writeln!(writer, "{}", line)?;
        Ok(())
    }
}

/// Count an entry in traversal statistics.
fn count_entry_stats(entry: &Entry, stats: &mut TreeStats) {
    match &entry.entry_type {
        EntryType::Directory => stats.directories += 1,
        EntryType::Symlink { target, broken } => {
            stats.symlinks += 1;
            if !*broken
                && entry
                    .path
                    .parent()
                    .map(|p| p.join(target).is_dir())
                    .unwrap_or(false)
            {
                stats.directories += 1;
            } else {
                stats.files += 1;
            }
        }
        _ => stats.files += 1,
    }
}
