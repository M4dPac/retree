use std::io::Write;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;
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

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn indent(depth: usize) -> String {
        "  ".repeat(depth)
    }

    fn write_entry<W: Write>(&mut self, writer: &mut W, entry: &Entry) -> Result<(), TreeError> {
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
        let name = Self::escape_xml(entry.name_str());

        match &entry.entry_type {
            EntryType::Directory => {
                write!(writer, "{}<directory name=\"{}\"", indent, name)?;

                if let Some(ref meta) = entry.metadata {
                    if let Some(modified) = meta.modified {
                        use chrono::{DateTime, Utc};
                        let dt: DateTime<Utc> = modified.into();
                        write!(writer, " time=\"{}\"", dt.format("%Y-%m-%dT%H:%M:%S"))?;
                    }
                }

                writeln!(writer, ">")?;
                self.depth_stack.push(entry.depth);
            }
            EntryType::File | EntryType::HardLink { .. } => {
                write!(writer, "{}<file name=\"{}\"", indent, name)?;

                if let Some(ref meta) = entry.metadata {
                    write!(writer, " size=\"{}\"", meta.size)?;

                    if let Some(modified) = meta.modified {
                        use chrono::{DateTime, Utc};
                        let dt: DateTime<Utc> = modified.into();
                        write!(writer, " time=\"{}\"", dt.format("%Y-%m-%dT%H:%M:%S"))?;
                    }
                }

                writeln!(writer, "/>")?;
            }
            EntryType::Symlink { target, .. } => {
                writeln!(
                    writer,
                    "{}<link name=\"{}\" target=\"{}\"/>",
                    indent,
                    name,
                    Self::escape_xml(&target.display().to_string())
                )?;
            }
            EntryType::Junction { target } => {
                writeln!(
                    writer,
                    "{}<junction name=\"{}\" target=\"{}\"/>",
                    indent,
                    name,
                    Self::escape_xml(&target.display().to_string())
                )?;
            }
            _ => {
                writeln!(writer, "{}<file name=\"{}\"/>", indent, name)?;
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
        self.write_entry(writer, &result.root)?;
        if result.root.entry_type.is_directory() {
            stats.directories += 1;
        } else {
            stats.files += 1;
        }

        // Child entries
        for entry in &result.entries {
            self.write_entry(writer, entry)?;
            if entry.entry_type.is_directory() {
                stats.directories += 1;
            } else {
                stats.files += 1;
            }
            if entry.entry_type.is_symlink() {
                stats.symlinks += 1;
            }
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
