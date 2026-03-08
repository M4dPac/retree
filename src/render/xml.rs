use std::io::Write;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;
use super::helpers;
use super::traits::Renderer;

pub struct XmlRenderer {
    depth_stack: Vec<usize>,
    pending_dir: Option<(usize, String)>,
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
                tag.push_str(&String::from_utf8(meta_buf).unwrap_or_default());
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

        // Root entry
        self.write_entry(writer, &result.root, config)?;
        helpers::count_stats(&result.root, stats);

        // Child entries
        for entry in &result.entries {
            self.write_entry(writer, entry, config)?;
            helpers::count_stats(entry, stats);
        }

        // Close any pending empty directory
        self.close_pending_empty(writer)?;

        // Close remaining open elements
        while let Some(depth) = self.depth_stack.pop() {
            writeln!(writer, "{}</directory>", Self::indent(depth + 1))?;
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
