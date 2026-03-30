use std::fs;
use std::io::Write;

use crate::config::Config;
use crate::core::entry::Entry;
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

/// Validate and sanitize a base URL for HTML output.
/// Blocks dangerous schemes (javascript:, data:, vbscript:) to prevent XSS.
fn sanitize_base_url(url: &str) -> String {
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
        url.to_string()
    }
}

/// Load an optional HTML fragment file, logging a warning on failure.
fn load_optional_file(path: &std::path::Path, flag_name: &str) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(content) => Some(content),
        Err(e) => {
            eprintln!(
                "rtree: warning: cannot read {} '{}': {}",
                flag_name,
                path.display(),
                e
            );
            None
        }
    }
}

impl HtmlRenderer {
    pub fn new(config: &Config) -> Self {
        let default_title = get_message(i18n::current(), MessageKey::HtmlTitle);

        let intro = config
            .html_intro
            .as_ref()
            .and_then(|path| load_optional_file(path, "--html-intro"));
        let outro = config
            .html_outro
            .as_ref()
            .and_then(|path| load_optional_file(path, "--html-outro"));

        let base_url = config.html_base.as_ref().map(|url| sanitize_base_url(url));

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
}

impl Renderer for HtmlRenderer {
    fn render<W: Write>(
        &self,
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
            super::walk_tree(
                tree,
                &[],
                stats,
                &mut state,
                &mut |entry, is_last, ancestors| {
                    self.write_entry_with_layout(writer, entry, is_last, ancestors)
                },
            )?;
        }

        self.write_footer(writer, stats, config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tree::Tree;
    use crate::core::walker::TreeStats;
    use crate::render::test_util::*;

    fn render_html(result: &BuildResult, config: &Config) -> String {
        let renderer = HtmlRenderer::new(config);
        let mut buf = Vec::new();
        let mut stats = TreeStats::default();
        renderer
            .render(result, config, &mut buf, &mut stats)
            .unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn html_basic_structure() {
        let result = result_with(dir_entry("root", 0), None);
        let config = Config::default();
        let output = render_html(&result, &config);
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<html>"));
        assert!(output.contains("<head>"));
        assert!(output.contains("</head>"));
        assert!(output.contains("<body>"));
        assert!(output.contains("</body>"));
        assert!(output.contains("</html>"));
    }

    #[test]
    fn html_root_entry_linked() {
        let result = result_with(dir_entry("mydir", 0), None);
        let config = Config::default();
        let output = render_html(&result, &config);
        assert!(output.contains("class=\"directory\""));
        assert!(output.contains("mydir"));
    }

    #[test]
    fn html_no_links_uses_span() {
        let tree = Tree {
            entry: dir_entry("root", 0),
            children: vec![Tree {
                entry: file_entry("f.txt", 1),
                children: vec![],
            }],
        };
        let result = result_with(dir_entry("root", 0), Some(tree));
        let config = Config {
            no_links: true,
            ..Default::default()
        };
        let output = render_html(&result, &config);
        assert!(output.contains("<span class=\"file\">f.txt</span>"));
        assert!(!output.contains("<a href="));
    }

    #[test]
    fn html_with_links_uses_anchor() {
        let tree = Tree {
            entry: dir_entry("root", 0),
            children: vec![Tree {
                entry: file_entry("f.txt", 1),
                children: vec![],
            }],
        };
        let result = result_with(dir_entry("root", 0), Some(tree));
        let config = Config::default();
        let output = render_html(&result, &config);
        assert!(output.contains("<a href="));
        assert!(output.contains("f.txt</a>"));
    }

    #[test]
    fn html_escapes_special_chars_in_name() {
        let entry = file_entry("a&b<c", 1);

        let tree = dir(
            "root",
            0,
            vec![Tree {
                entry,
                children: vec![],
            }],
        );
        let result = result_with(dir_entry("root", 0), Some(tree));
        let config = Config {
            no_links: true,
            ..Default::default()
        };
        let output = render_html(&result, &config);
        assert!(output.contains("a&amp;b&lt;c"));
    }

    #[test]
    fn html_no_report() {
        let result = result_with(dir_entry("root", 0), None);
        let config = Config {
            no_report: true,
            ..Default::default()
        };
        let output = render_html(&result, &config);
        assert!(!output.contains("<p>"));
    }

    // ══════════════════════════════════════════════
    // sanitize_base_url
    // ══════════════════════════════════════════════

    #[test]
    fn url_normal_https() {
        assert_eq!(
            sanitize_base_url("https://example.com"),
            "https://example.com"
        );
    }

    #[test]
    fn url_relative_path() {
        assert_eq!(sanitize_base_url("./docs"), "./docs");
    }

    #[test]
    fn url_file_scheme_allowed() {
        assert_eq!(sanitize_base_url("file:///home"), "file:///home");
    }

    #[test]
    fn url_javascript_blocked() {
        assert_eq!(sanitize_base_url("javascript:alert(1)"), ".");
    }

    #[test]
    fn url_data_blocked() {
        assert_eq!(sanitize_base_url("data:text/html,<h1>X</h1>"), ".");
    }

    #[test]
    fn url_vbscript_blocked() {
        assert_eq!(sanitize_base_url("vbscript:MsgBox"), ".");
    }

    #[test]
    fn url_case_insensitive_blocked() {
        assert_eq!(sanitize_base_url("JavaScript:alert(1)"), ".");
        assert_eq!(sanitize_base_url("DATA:text/html,..."), ".");
    }

    #[test]
    fn url_control_chars_stripped_before_check() {
        assert_eq!(sanitize_base_url("java\x00script:foo"), ".");
    }

    #[test]
    fn url_whitespace_trimmed() {
        assert_eq!(
            sanitize_base_url("  https://example.com  "),
            "  https://example.com  "
        );
        // trimming is only for scheme check, original value preserved
    }
}
