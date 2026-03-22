//! Streaming tree traversal — renders output during DFS walk.
//!
//! Unlike `OrderedEngine` which builds a full tree in memory,
//! `StreamingEngine` writes entries directly to output as they are visited.
//! This reduces memory usage for large directory trees.

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::common;
use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::sorter::sort_entries;
use crate::core::walker::TreeStats;
use crate::error::TreeError;
use crate::i18n;
use crate::render::{helpers::count_stats, TextRenderer};

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
    visited: HashSet<common::VisitedKey>,
    root_device: Option<u64>,
}

impl StreamState {
    fn new(max_entries: Option<usize>, root_device: Option<u64>) -> Self {
        Self {
            count: 0,
            max_entries,
            truncated: false,
            visited: HashSet::new(),
            root_device,
        }
    }

    /// Check if we've reached the entry limit.
    fn should_stop(&self) -> bool {
        self.max_entries.is_some_and(|max| self.count >= max)
    }
}

/// Streaming tree traversal engine.
///
/// Performs DFS traversal and writes text output inline,
/// computing is_last/ancestors_last on the fly.
pub struct StreamingEngine<'a> {
    config: &'a Config,
    text: TextRenderer,
}

impl<'a> StreamingEngine<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            text: TextRenderer::new(config),
        }
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
        let needs_file_id = common::needs_file_id(config);

        // When --long-paths is requested, resolve relative root to absolute
        // first — the \\?\ prefix only works with absolute paths.
        let root_abs;
        let effective_root = if config.long_paths && !root.is_absolute() {
            root_abs = std::env::current_dir()
                .map(|cwd| cwd.join(root))
                .unwrap_or_else(|_| root.to_path_buf());
            root_abs.as_path()
        } else {
            root
        };
        let long_root_buf = crate::platform::to_long_path(effective_root, config.long_paths);

        let root = long_root_buf.as_path();

        // Root entry
        let root_entry = Entry::from_path(
            root,
            0,
            false,
            vec![],
            needs_file_id,
            config.show_permissions,
        )?;
        self.text.write_entry(writer, &root_entry, config)?;
        stats.directories += 1;

        // DFS children with max_entries tracking
        let mut state = StreamState::new(
            config.max_entries,
            common::compute_root_device(config, root),
        );

        if config.one_fs && state.root_device.is_none() {
            errors.push(TreeError::Io(
                root.to_path_buf(),
                std::io::Error::other(
                    "--one-fs: cannot determine root volume; cross-device check skipped",
                ),
            ));
        }

        state.visited.insert(common::make_visited_key(root));
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

    /// Read, filter, sort, and emit children of a single directory,
    /// then recursively descend into subdirectories (DFS).
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
        if depth >= common::MAX_INTERNAL_DEPTH {
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
            .filter(
                |de| match common::filter_entry(config, de, parent_matched) {
                    common::FilterResult::Include { .. } => true,
                    common::FilterResult::Reserved => {
                        errors.push(TreeError::ReservedName(de.path()));
                        false
                    }
                    common::FilterResult::Exclude => false,
                },
            )
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
                Ok(mut entry) => {
                    // Determine if this entry is a traversable directory
                    let should_descend = match &entry.entry_type {
                        EntryType::Directory => true,
                        EntryType::Symlink { broken: false, .. } if config.follow_symlinks => {
                            entry.path.is_dir()
                        }
                        EntryType::Junction { .. } if config.show_junctions => true,
                        _ => false,
                    };

                    // Cycle detection: check visited before writing
                    let descend = if should_descend {
                        let visit_key = common::make_visited_key(&entry.path);
                        if state.visited.insert(visit_key) {
                            true
                        } else {
                            entry.recursive_link = true;
                            false
                        }
                    } else {
                        false
                    };

                    // --one-fs: don't descend into different filesystems.
                    // If volume cannot be determined, conservatively don't descend.
                    let descend = descend
                        && match state.root_device {
                            Some(root_dev) => match crate::platform::get_file_id(&entry.path) {
                                Some(info) => info.volume_serial == root_dev,
                                None => {
                                    errors.push(TreeError::Io(
                                        entry.path.clone(),
                                        std::io::Error::other(
                                            "cannot determine volume for --one-fs",
                                        ),
                                    ));
                                    false
                                }
                            },
                            None => true,
                        };

                    self.text.write_entry(writer, &entry, config)?;
                    count_stats(&entry, stats);
                    state.count += 1;

                    // Emit NTFS Alternate Data Streams for non-descended entries
                    if config.show_streams && !descend {
                        let streams = crate::platform::get_alternate_streams(&entry.path);
                        let num_streams = streams.len();
                        for (si, stream) in streams.into_iter().enumerate() {
                            if state.should_stop() {
                                state.truncated = true;
                                return Ok(());
                            }
                            let mut ads_entry =
                                Entry::from_ads(&entry.path, stream.name, stream.size, depth + 1);
                            ads_entry.is_last = si == num_streams - 1;
                            let mut ads_ancestors = ancestors_last.to_vec();
                            ads_ancestors.push(is_last);
                            ads_entry.ancestors_last = ads_ancestors;
                            self.text.write_entry(writer, &ads_entry, config)?;
                            count_stats(&ads_entry, stats);
                            state.count += 1;
                        }
                    }

                    if descend {
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
}
