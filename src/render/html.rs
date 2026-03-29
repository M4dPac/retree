use std::fs;
use std::io::Write;

use crate::config::Config;
use crate::core::entry::Entry;
use crate::core::tree::Tree;
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;
use crate::i18n::{self, format_report, get_message, MessageKey};

use super::helpers;
use super::traits::Renderer;
use super::RenderState;

pub struct HtmlRenderer {
    base_url: Option<String>,
    title: String,
    no_links: bool,
    intro: Option<String>,
    outro: Option<String>,
}

impl HtmlRenderer {
    pub fn new(config: &Config) -> Self {
        let default_title = get_message(i18n::current(), MessageKey::HtmlTitle);

        let intro = config
            .html_intro
            .as_ref()
            .and_then(|path| match fs::read_to_string(path) {
                Ok(content) => Some(content),
                Err(e) => {
                    eprintln!(
                        "rtree: warning: cannot read --html-intro '{}': {}",
                        path.display(),
                        e
                    );
                    None
                }
            });
        let outro = config
            .html_outro
            .as_ref()
            .and_then(|path| match fs::read_to_string(path) {
                Ok(content) => Some(content),
                Err(e) => {
                    eprintln!(
                        "rtree: warning: cannot read --html-outro '{}': {}",
                        path.display(),
                        e
                    );
                    None
                }
            });

        // Reject dangerous URL schemes in base URL
        let base_url = config.html_base.as_ref().map(|url| {
            let lower: String = url
                .trim()
                .to_lowercase()
                .chars()
                .filter(|c| !c.is_control())
                .collect();
            if lower.starts_with("javascript:")
                || lower.starts_with("data:")
                || lower.starts_with("vbscript:")
            {
                eprintln!("rtree: warning: unsafe URL scheme in -H ignored, using '.'");
                ".".to_string()
            } else {
                url.clone()
            }
        });

        HtmlRenderer {
            base_url,
            title: config
                .html_title
                .clone()
                .unwrap_or_else(|| default_title.to_string()),
            no_links: config.no_links,
            intro,
            outro,
        }
    }

    fn write_header<W: Write>(&self, writer: &mut W) -> Result<(), TreeError> {
        if let Some(ref intro) = self.intro {
            writer.write_all(intro.as_bytes())?;
            return Ok(());
        }

        writeln!(writer, "<!DOCTYPE html>")?;
        writeln!(writer, "<html>")?;
        writeln!(writer, "<head>")?;
        writeln!(writer, "  <meta charset=\"UTF-8\">")?;
        writeln!(
            writer,
            "  <title>{}</title>",
            helpers::escape_html(&self.title)
        )?;
        writeln!(writer, "  <style>")?;
        writeln!(
            writer,
            "    body {{ font-family: monospace; white-space: pre; }}"
        )?;
        writeln!(writer, "    a {{ text-decoration: none; }}")?;
        writeln!(
            writer,
            "    .directory {{ font-weight: bold; color: #0066cc; }}"
        )?;
        writeln!(writer, "    .file {{ color: #333; }}")?;
        writeln!(writer, "    .symlink {{ color: #00aa88; }}")?;
        writeln!(writer, "  </style>")?;
        writeln!(writer, "</head>")?;
        writeln!(writer, "<body>")?;
        writeln!(writer, "<h1>{}</h1>", helpers::escape_html(&self.title))?;

        Ok(())
    }

    fn write_entry_with_layout<W: Write>(
        &self,
        writer: &mut W,
        entry: &Entry,
        is_last: bool,
        ancestors_last: &[bool],
    ) -> Result<(), TreeError> {
        let mut prefix = String::new();
        for &ancestor_last in ancestors_last {
            if ancestor_last {
                prefix.push_str("    ");
            } else {
                prefix.push_str("│   ");
            }
        }

        if entry.depth > 0 {
            if is_last {
                prefix.push_str("└── ");
            } else {
                prefix.push_str("├── ");
            }
        }

        let name = helpers::escape_html(&entry.name_str());
        let class = if entry.entry_type.is_directory() {
            "directory"
        } else if entry.entry_type.is_symlink() {
            "symlink"
        } else {
            "file"
        };

        write!(writer, "{}", prefix)?;

        if !self.no_links {
            let path_for_url = entry.path.display().to_string().replace('\\', "/");
            let encoded_path = helpers::encode_uri_path(&path_for_url);
            let href = if let Some(ref base) = self.base_url {
                format!("{}/{}", base, encoded_path)
            } else {
                encoded_path
            };
            writeln!(
                writer,
                "<a href=\"{}\" class=\"{}\">{}</a><br>",
                helpers::escape_html(&href),
                class,
                name
            )?;
        } else {
            writeln!(writer, "<span class=\"{}\">{}</span><br>", class, name)?;
        }

        Ok(())
    }

    fn write_footer<W: Write>(
        &self,
        writer: &mut W,
        stats: &TreeStats,
        config: &Config,
    ) -> Result<(), TreeError> {
        if let Some(ref outro) = self.outro {
            writer.write_all(outro.as_bytes())?;
            return Ok(());
        }

        if !config.no_report {
            writeln!(writer, "<br>")?;
            let report = format_report(
                i18n::current(),
                stats.directories.saturating_sub(1),
                stats.files,
            );
            writeln!(writer, "<p>{}</p>", helpers::escape_html(&report))?;
        }

        writeln!(writer, "</body>")?;
        writeln!(writer, "</html>")?;

        Ok(())
    }

    /// Recursively render children of a tree node (depth-first).
    fn render_children<W: Write>(
        &self,
        node: &Tree,
        ancestors_last: &[bool],
        writer: &mut W,
        stats: &mut TreeStats,
        state: &mut RenderState,
    ) -> Result<(), TreeError> {
        let num_children = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            if state.max_entries.is_some_and(|max| state.count >= max) {
                state.truncated = true;
                return Ok(());
            }

            let is_last = i == num_children - 1;

            self.write_entry_with_layout(writer, &child.entry, is_last, ancestors_last)?;
            helpers::count_stats(&child.entry, stats);
            state.count += 1;

            if !child.children.is_empty() {
                let mut new_ancestors = ancestors_last.to_vec();
                new_ancestors.push(is_last);
                self.render_children(child, &new_ancestors, writer, stats, state)?;
                if state.truncated {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

impl Renderer for HtmlRenderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        config: &Config,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        self.write_header(writer)?;

        // Root entry
        self.write_entry_with_layout(writer, &result.root, false, &[])?;
        helpers::count_stats(&result.root, stats);

        // Children from tree
        if let Some(ref tree) = result.tree {
            let mut state = RenderState {
                max_entries: config.max_entries,
                count: 0,
                truncated: false,
            };
            self.render_children(tree, &[], writer, stats, &mut state)?;
        }

        self.write_footer(writer, stats, config)?;

        Ok(())
    }
}
