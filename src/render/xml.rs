use std::io::Write;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::tree::Tree;
use crate::core::walker::TreeStats;

use crate::core::BuildResult;
use crate::error::TreeError;

use super::helpers;
use super::traits::Renderer;
use super::RenderState;

pub struct XmlRenderer;

impl XmlRenderer {
    pub fn new(_config: &Config) -> Self {
        XmlRenderer
    }

    fn indent(depth: usize) -> String {
        "  ".repeat(depth)
    }

    /// Write common metadata attributes based on config flags
    fn write_meta_attrs<W: Write>(
        writer: &mut W,
        entry: &Entry,
        config: &Config,
    ) -> Result<(), TreeError> {
        if let Some(ref meta) = entry.metadata {
            // Size: -s or -h flag
            if config.show_size {
                if config.human_readable {
                    write!(
                        writer,
                        " size=\"{}\"",
                        helpers::format_human_size(meta.size, config.si_units)
                    )?;
                } else {
                    write!(writer, " size=\"{}\"", meta.size)?;
                }
            }

            // Date: -D flag
            if config.show_date {
                if let Some(modified) = meta.modified {
                    // XML uses UTC for machine-readable, timezone-unambiguous timestamps.
                    use chrono::{DateTime, Utc};
                    let dt: DateTime<Utc> = modified.into();
                    write!(
                        writer,
                        " time=\"{}\"",
                        helpers::escape_xml(&dt.format(&config.time_fmt).to_string())
                    )?;
                }
            }

            // Permissions: -p flag
            if config.show_permissions {
                write!(writer, " mode=\"{:o}\"", meta.mode.unwrap_or(0))?;
            }

            // Owner: -u flag
            if config.show_owner {
                if let Some(ref owner) = meta.owner {
                    write!(writer, " user=\"{}\"", helpers::escape_xml(owner))?;
                }
            }

            // Group: -g flag
            if config.show_group {
                if let Some(ref group) = meta.group {
                    write!(writer, " group=\"{}\"", helpers::escape_xml(group))?;
                }
            }

            // Inode: --inodes flag (u64, show if non-zero)
            if config.show_inodes && meta.inode != 0 {
                write!(writer, " inode=\"{}\"", meta.inode)?;
            }

            // Device: --device flag (u32, show if non-zero)
            if config.show_device && meta.device != 0 {
                write!(writer, " dev=\"{}\"", meta.device)?;
            }
        }
        Ok(())
    }

    /// Recursively render children as XML (tree-based, no pending_dir needed).
    fn render_tree_children<W: Write>(
        writer: &mut W,
        node: &Tree,
        config: &Config,
        stats: &mut TreeStats,
        state: &mut RenderState,
    ) -> Result<(), TreeError> {
        for child in &node.children {
            if state.max_entries.is_some_and(|max| state.count >= max) {
                state.truncated = true;
                return Ok(());
            }

            helpers::count_stats(&child.entry, stats);
            state.count += 1;

            let entry = &child.entry;
            let indent = Self::indent(entry.depth + 1);
            let name = helpers::escape_xml(&entry.name_str());

            match &entry.entry_type {
                EntryType::Directory => {
                    write!(writer, "{}<directory name=\"{}\"", indent, name)?;
                    Self::write_meta_attrs(writer, entry, config)?;

                    if child.children.is_empty() {
                        writeln!(writer, "></directory>")?;
                    } else {
                        writeln!(writer, ">")?;
                        Self::render_tree_children(writer, child, config, stats, state)?;
                        writeln!(writer, "{}</directory>", indent)?;
                        if state.truncated {
                            return Ok(());
                        }
                    }
                }
                EntryType::File | EntryType::HardLink { .. } => {
                    write!(writer, "{}<file name=\"{}\"", indent, name)?;
                    Self::write_meta_attrs(writer, entry, config)?;
                    writeln!(writer, "></file>")?;
                }
                EntryType::Symlink { target, .. } => {
                    write!(
                        writer,
                        "{}<link name=\"{}\" target=\"{}\"",
                        indent,
                        name,
                        helpers::escape_xml(&target.display().to_string())
                    )?;
                    Self::write_meta_attrs(writer, entry, config)?;
                    writeln!(writer, "></link>")?;
                }
                EntryType::Junction { target } => {
                    write!(
                        writer,
                        "{}<link name=\"{}\" target=\"{}\"",
                        indent,
                        name,
                        helpers::escape_xml(&target.display().to_string())
                    )?;
                    Self::write_meta_attrs(writer, entry, config)?;
                    writeln!(writer, "></link>")?;
                }
                _ => {
                    write!(writer, "{}<file name=\"{}\"", indent, name)?;
                    Self::write_meta_attrs(writer, entry, config)?;
                    writeln!(writer, "/>")?;
                }
            }
        }
        Ok(())
    }
}

impl Renderer for XmlRenderer {
    fn render<W: Write>(
        &self,
        result: &BuildResult,
        config: &Config,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        // Header
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(writer, "<tree>")?;

        // Root directory
        helpers::count_stats(&result.root, stats);
        let root_entry = &result.root;
        let indent = Self::indent(root_entry.depth + 1);
        let name = helpers::escape_xml(&root_entry.name_str());
        write!(writer, "{}<directory name=\"{}\"", indent, name)?;
        Self::write_meta_attrs(writer, root_entry, config)?;

        match result.tree {
            Some(ref tree) if !tree.children.is_empty() => {
                writeln!(writer, ">")?;
                let mut state = RenderState {
                    max_entries: config.max_entries,
                    count: 0,
                    truncated: false,
                };
                Self::render_tree_children(writer, tree, config, stats, &mut state)?;
                writeln!(writer, "{}</directory>", indent)?;
            }
            _ => {
                writeln!(writer, "></directory>")?;
            }
        }

        if !config.no_report {
            writeln!(writer, "  <report>")?;
            writeln!(
                writer,
                "    <directories>{}</directories>",
                stats.directories.saturating_sub(1)
            )?;

            if !config.dirs_only {
                writeln!(writer, "    <files>{}</files>", stats.files)?;
            }
            writeln!(writer, "  </report>")?;
        }

        writeln!(writer, "</tree>")?;

        Ok(())
    }
}
