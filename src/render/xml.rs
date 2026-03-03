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
}

impl XmlRenderer {
    pub fn new(_config: &Config) -> Self {
        XmlRenderer {
            depth_stack: Vec::new(),
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
                    write!(writer, " time=\"{}\"", dt.format(&config.time_fmt))?;
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

    fn write_entry<W: Write>(
        &mut self,
        writer: &mut W,
        entry: &Entry,
        config: &Config,
    ) -> Result<(), TreeError> {
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
                write!(writer, "{}<directory name=\"{}\"", indent, name)?;
                Self::write_meta_attrs(writer, entry, config)?;
                writeln!(writer, ">")?;
                self.depth_stack.push(entry.depth);
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
                writeln!(writer, "/>")?;
            }
            EntryType::Junction { target } => {
                write!(
                    writer,
                    "{}<junction name=\"{}\" target=\"{}\"",
                    indent,
                    name,
                    helpers::escape_xml(&target.display().to_string())
                )?;
                Self::write_meta_attrs(writer, entry, config)?;
                writeln!(writer, "/>")?;
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

        // Root entry
        self.write_entry(writer, &result.root, config)?;
        helpers::count_stats(&result.root, stats);

        // Child entries
        for entry in &result.entries {
            self.write_entry(writer, entry, config)?;
            helpers::count_stats(entry, stats);
        }

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
            writeln!(writer, "    <files>{}</files>", stats.files)?;
            writeln!(writer, "  </report>")?;
        }

        writeln!(writer, "</tree>")?;

        Ok(())
    }
}

