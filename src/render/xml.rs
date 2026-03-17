use std::io::Write;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::{Node, TreeStats};
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;
use super::helpers;
use super::traits::Renderer;

pub struct XmlRenderer {
    depth_stack: Vec<usize>,
    pending_dir: Option<(usize, String)>,
}

/// Mutable state for tree-based rendering (truncation tracking).
struct RenderState {
    max_entries: Option<usize>,
    count: usize,
    truncated: bool,
}

impl XmlRenderer {
    pub fn new(_config: &Config) -> Self {
        XmlRenderer {
            depth_stack: Vec::new(),
            pending_dir: None,
        }
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

    /// Flush any pending (deferred) directory opening tag.
    /// Called when we know the directory has children (next entry is deeper).
    fn flush_pending<W: Write>(&mut self, writer: &mut W) -> Result<(), TreeError> {
        if let Some((depth, tag)) = self.pending_dir.take() {
            writeln!(writer, "{}>", tag)?;
            self.depth_stack.push(depth);
        }
        Ok(())
    }

    /// Close the pending directory as empty (self-closing on one line).
    /// Called when next entry is at the same or higher level.
    fn close_pending_empty<W: Write>(&mut self, writer: &mut W) -> Result<(), TreeError> {
        if let Some((_depth, tag)) = self.pending_dir.take() {
            writeln!(writer, "{}></directory>", tag)?;
        }
        Ok(())
    }

    fn write_entry<W: Write>(
        &mut self,
        writer: &mut W,
        entry: &Entry,
        config: &Config,
    ) -> Result<(), TreeError> {
        // First: if there's a pending directory and new entry is NOT its child,
        // close it as empty.
        if let Some(&(pending_depth, _)) = self.pending_dir.as_ref() {
            if entry.depth <= pending_depth {
                // Next entry is at the same or higher level — pending dir is empty
                self.close_pending_empty(writer)?;
            } else {
                // Next entry is deeper — pending dir has children
                self.flush_pending(writer)?;
            }
        }

        // Close previous elements if we're going back up
        while let Some(&prev_depth) = self.depth_stack.last() {
            if prev_depth >= entry.depth {
                self.depth_stack.pop();
                writeln!(writer, "{}</directory>", Self::indent(prev_depth + 1))?;
            } else {
                break;
            }
        }

        let indent = Self::indent(entry.depth + 1);
        let name = helpers::escape_xml(entry.name_str());

        match &entry.entry_type {
            EntryType::Directory => {
                // Build the opening tag but DON'T write it yet — defer it
                let mut tag = format!("{}<directory name=\"{}\"", indent, name);
                // We need to write meta attrs into the tag string
                let mut meta_buf: Vec<u8> = Vec::new();
                Self::write_meta_attrs(&mut meta_buf, entry, config)?;
                tag.push_str(&String::from_utf8_lossy(&meta_buf));
                self.pending_dir = Some((entry.depth, tag));
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

        Ok(())
    }

    /// Recursively render children as XML (tree-based, no pending_dir needed).
    fn render_tree_children<W: Write>(
        writer: &mut W,
        node: &Node,
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
            let name = helpers::escape_xml(entry.name_str());

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
        &mut self,
        result: &BuildResult,
        ctx: &RenderContext,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        let config = ctx.config;

        // Header
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(writer, "<tree>")?;
        self.depth_stack.clear();
        self.pending_dir = None;

        if let Some(ref tree) = result.tree {
            // Tree-based rendering — no pending_dir/depth_stack needed
            helpers::count_stats(&result.root, stats);

            let root_entry = &result.root;
            let indent = Self::indent(root_entry.depth + 1);
            let name = helpers::escape_xml(root_entry.name_str());

            write!(writer, "{}<directory name=\"{}\"", indent, name)?;
            Self::write_meta_attrs(writer, root_entry, config)?;

            if tree.children.is_empty() {
                writeln!(writer, "></directory>")?;
            } else {
                writeln!(writer, ">")?;
                let mut state = RenderState {
                    max_entries: config.max_entries,
                    count: 0,
                    truncated: false,
                };
                Self::render_tree_children(writer, tree, config, stats, &mut state)?;
                writeln!(writer, "{}</directory>", indent)?;
            }
        } else {
            // Fallback: flat rendering
            self.write_entry(writer, &result.root, config)?;
            helpers::count_stats(&result.root, stats);

            for entry in &result.entries {
                self.write_entry(writer, entry, config)?;
                helpers::count_stats(entry, stats);
            }

            self.close_pending_empty(writer)?;

            while let Some(depth) = self.depth_stack.pop() {
                writeln!(writer, "{}</directory>", Self::indent(depth + 1))?;
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
