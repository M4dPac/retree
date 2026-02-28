use std::fs;
use std::io::Write;

use crate::config::Config;
use crate::error::TreeError;
use crate::i18n::{self, format_report, get_message, MessageKey};
use crate::walker::{TreeEntry, TreeStats};

use super::TreeOutput;

pub struct HtmlFormatter {
    base_url: Option<String>,
    title: String,
    no_links: bool,
    intro: Option<String>,
    outro: Option<String>,
}

impl HtmlFormatter {
    pub fn new(config: &Config) -> Self {
        // Use localized default title if not specified
        let default_title = get_message(i18n::current(), MessageKey::HtmlTitle);

        // Load custom intro/outro files if specified
        let intro = config.html_intro.as_ref().and_then(|path| {
            fs::read_to_string(path).ok()
        });
        let outro = config.html_outro.as_ref().and_then(|path| {
            fs::read_to_string(path).ok()
        });

        HtmlFormatter {
            base_url: config.html_base.clone(),
            title: config
                .html_title
                .clone()
                .unwrap_or_else(|| default_title.to_string()),
            no_links: config.no_links,
            intro,
            outro,
        }
    }

    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}

impl TreeOutput for HtmlFormatter {
    fn begin<W: Write>(&mut self, writer: &mut W) -> Result<(), TreeError> {
        // Write custom intro if provided
        if let Some(ref intro) = self.intro {
            writer.write_all(intro.as_bytes())?;
            return Ok(());
        }

        // Default HTML template
        writeln!(writer, "<!DOCTYPE html>")?;
        writeln!(writer, "<html>")?;
        writeln!(writer, "<head>")?;
        writeln!(writer, "  <meta charset=\"UTF-8\">")?;
        writeln!(
            writer,
            "  <title>{}</title>",
            Self::escape_html(&self.title)
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
        writeln!(writer, "<h1>{}</h1>", Self::escape_html(&self.title))?;

        Ok(())
    }

    fn write_entry<W: Write>(
        &mut self,
        writer: &mut W,
        entry: &TreeEntry,
        _config: &Config,
    ) -> Result<(), TreeError> {
        let mut prefix = String::new();
        for &is_last in &entry.ancestors_last {
            if is_last {
                prefix.push_str("    ");
            } else {
                prefix.push_str("│   ");
            }
        }

        if entry.depth > 0 {
            if entry.is_last {
                prefix.push_str("└── ");
            } else {
                prefix.push_str("├── ");
            }
        }

        let name = Self::escape_html(entry.name_str());
        let class = if entry.entry_type.is_directory() {
            "directory"
        } else if entry.entry_type.is_symlink() {
            "symlink"
        } else {
            "file"
        };

        write!(writer, "{}", prefix)?;

        if !self.no_links {
            let href = if let Some(ref base) = self.base_url {
                format!("{}/{}", base, entry.path.display())
            } else {
                entry.path.display().to_string()
            };
            writeln!(
                writer,
                "<a href=\"{}\" class=\"{}\">{}</a><br>",
                Self::escape_html(&href),
                class,
                name
            )?;
        } else {
            writeln!(writer, "<span class=\"{}\">{}</span><br>", class, name)?;
        }

        Ok(())
    }

    fn end<W: Write>(
        &mut self,
        writer: &mut W,
        stats: &TreeStats,
        config: &Config,
    ) -> Result<(), TreeError> {
        // Write custom outro if provided
        if let Some(ref outro) = self.outro {
            writer.write_all(outro.as_bytes())?;
            return Ok(());
        }

        // Default ending
        if !config.no_report {
            writeln!(writer, "<br>")?;
            let report = format_report(
                i18n::current(),
                stats.directories.saturating_sub(1),
                stats.files,
            );
            writeln!(writer, "<p>{}</p>", report)?;
        }

        writeln!(writer, "</body>")?;
        writeln!(writer, "</html>")?;

        Ok(())
    }
}
