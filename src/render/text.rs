use std::io::Write;

use crate::config::{Config, LineStyle};
use crate::core::entry::{Entry, EntryType};
use crate::core::tree::Tree;
use crate::core::walker::EntryWriter;
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;
use crate::i18n::{self, format_report, get_message, MessageKey};

use super::helpers;
use super::traits::Renderer;
use super::RenderState;

pub struct TextRenderer {
    line_style: LineStyle,
    color_enabled: bool,
    icons_enabled: bool,
}

struct TreeChars {
    branch: &'static str,
    vertical: &'static str,
    last_branch: &'static str,
    #[allow(dead_code)]
    horizontal: &'static str,
    space: &'static str,
}

const ANSI_CHARS: TreeChars = TreeChars {
    branch: "├── ",
    vertical: "│   ",
    last_branch: "└── ",
    horizontal: "─",
    space: "    ",
};

const CP437_CHARS: TreeChars = TreeChars {
    branch: "├── ",
    vertical: "│   ",
    last_branch: "└── ",
    horizontal: "─",
    space: "    ",
};

const ASCII_CHARS: TreeChars = TreeChars {
    branch: "|-- ",
    vertical: "|   ",
    last_branch: "`-- ",
    horizontal: "-",
    space: "    ",
};

impl TextRenderer {
    pub fn new(config: &Config) -> Self {
        TextRenderer {
            line_style: config.line_style,
            color_enabled: config.color_enabled,
            icons_enabled: config.icons_enabled,
        }
    }

    fn get_chars(&self) -> &'static TreeChars {
        match self.line_style {
            LineStyle::Ansi => &ANSI_CHARS,
            LineStyle::Cp437 => &CP437_CHARS,
            LineStyle::Ascii => &ASCII_CHARS,
        }
    }

    /// Sanitize string for safe terminal output.
    /// Replaces control characters (except common whitespace) with '?'.
    /// Also replaces Unicode bidi overrides and zero-width characters
    /// that can be used for visual spoofing of filenames.
    fn sanitize_for_terminal(s: &str) -> String {
        s.chars()
            .map(|c| {
                if (c.is_control() && c != '\t' && c != '\n') || helpers::is_bidi_or_zw(c) {
                    '?'
                } else {
                    c
                }
            })
            .collect()
    }

    fn format_prefix(
        &self,
        depth: usize,
        is_last: bool,
        ancestors_last: &[bool],
        config: &Config,
    ) -> String {
        if config.no_indent {
            return String::new();
        }

        let chars = self.get_chars();
        let mut prefix = String::new();

        for &ancestor_last in ancestors_last {
            if ancestor_last {
                prefix.push_str(chars.space);
            } else {
                prefix.push_str(chars.vertical);
            }
        }

        if depth > 0 {
            if is_last {
                prefix.push_str(chars.last_branch);
            } else {
                prefix.push_str(chars.branch);
            }
        }

        prefix
    }

    fn format_name(&self, entry: &Entry, config: &Config) -> String {
        let mut name = String::new();

        if self.icons_enabled {
            let icon = config.icon_set.get_icon(entry);
            name.push_str(&icon);
            name.push(' ');
        }

        if config.full_path {
            if let EntryType::Ads { ref stream_name } = entry.entry_type {
                // Show as /path/to/file.txt:stream_name
                name.push_str(&format!("{}:{}", entry.path.display(), stream_name));
            } else {
                name.push_str(&entry.path.display().to_string());
            }
        } else {
            name.push_str(&entry.name_str());
        }

        if config.classify {
            match &entry.entry_type {
                EntryType::Directory => name.push('/'),
                EntryType::Symlink { .. } | EntryType::Junction { .. } => {}
                EntryType::File => {
                    if crate::platform::is_executable(&entry.path) {
                        name.push('*');
                    }
                }
                _ => {}
            }
        }

        match &entry.entry_type {
            EntryType::Symlink { target, broken } => {
                name.push_str(" -> ");
                name.push_str(&target.display().to_string());
                if *broken {
                    let broken_msg = get_message(i18n::current(), MessageKey::BrokenLink);
                    name.push_str(" [");
                    name.push_str(broken_msg);
                    name.push(']');
                }
                if entry.recursive_link {
                    let recursive_msg = get_message(i18n::current(), MessageKey::RecursiveLink);
                    name.push_str("  [");
                    name.push_str(recursive_msg);
                    name.push(']');
                }
            }
            EntryType::Junction { target } => {
                name.push_str(" => ");
                name.push_str(&target.display().to_string());
            }
            _ => {}
        }

        // filelimit annotation (GNU tree compatible)
        if let Some(count) = entry.filelimit_exceeded {
            let msg = get_message(i18n::current(), MessageKey::ExceedsFileLimit);
            let formatted = msg.replace("{}", &count.to_string());
            name.push_str("  [");
            name.push_str(&formatted);
            name.push(']');
        }

        // Apply safe_print sanitization to entire formatted name
        if config.safe_print {
            name = Self::sanitize_for_terminal(&name);
        }

        name
    }

    fn format_info(&self, entry: &Entry, config: &Config) -> String {
        let mut info = String::new();

        if let Some(ref meta) = entry.metadata {
            if config.show_size {
                let size_str = if config.human_readable {
                    helpers::format_human_size(meta.size, config.si_units)
                } else {
                    format!("{:>10}", meta.size)
                };
                info.push('[');
                info.push_str(&size_str);
                info.push_str("]  ");
            }

            if config.show_date {
                if let Some(modified) = meta.modified {
                    use chrono::{DateTime, Local};
                    let dt: DateTime<Local> = modified.into();
                    let formatted = dt.format(&config.time_fmt).to_string();
                    info.push('[');
                    info.push_str(&formatted);
                    info.push_str("]  ");
                }
            }

            if config.show_permissions {
                let perm_str = format_permissions(meta, config.perm_mode);
                info.push('[');
                info.push_str(&perm_str);
                info.push_str("]  ");
            }

            if config.show_inodes {
                use std::fmt::Write;
                let _ = write!(info, "[{:>10}]  ", meta.inode);
            }

            if config.show_device {
                use std::fmt::Write;
                let _ = write!(info, "[{:>8x}]  ", meta.device);
            }

            if config.show_owner {
                if let Some(ref owner) = meta.owner {
                    // GNU tree format: [username] with 8-char width padding
                    info.push('[');
                    info.push_str(&format!("{:<8}", owner));
                    info.push_str("]  ");
                }
            }

            if config.show_group {
                if let Some(ref group) = meta.group {
                    // GNU tree format: [groupname] with 8-char width padding
                    info.push('[');
                    info.push_str(&format!("{:<8}", group));
                    info.push_str("]  ");
                }
            }
        }

        info
    }

    fn apply_color(&self, entry: &Entry, text: &str, config: &Config) -> String {
        if !self.color_enabled {
            return text.to_string();
        }

        let color_code = config.color_scheme.get_color(entry);
        if color_code.is_empty() {
            text.to_string()
        } else {
            let mut result = String::with_capacity(text.len() + color_code.len() + 10);
            result.push_str("\x1b[");
            result.push_str(&color_code);
            result.push('m');
            result.push_str(text);
            result.push_str("\x1b[0m");
            result
        }
    }

    /// Recursively render children of a tree node (depth-first).
    /// computes is_last/ancestors_last on the fly.
    fn render_children<W: Write>(
        &self,
        node: &Tree,
        ancestors_last: &[bool],
        config: &Config,
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

            self.write_entry_with_layout(writer, &child.entry, is_last, ancestors_last, config)?;
            helpers::count_stats(&child.entry, stats);
            state.count += 1;

            if !child.children.is_empty() {
                let mut new_ancestors = ancestors_last.to_vec();
                new_ancestors.push(is_last);
                self.render_children(child, &new_ancestors, config, writer, stats, state)?;
                if state.truncated {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Write a single entry with explicit layout info (no Entry clone needed).
    fn write_entry_with_layout(
        &self,
        writer: &mut dyn Write,
        entry: &Entry,
        is_last: bool,
        ancestors_last: &[bool],
        config: &Config,
    ) -> Result<(), TreeError> {
        let prefix = self.format_prefix(entry.depth, is_last, ancestors_last, config);
        let info = self.format_info(entry, config);
        let info = if config.safe_print {
            Self::sanitize_for_terminal(&info)
        } else {
            info
        };
        let name = self.format_name(entry, config);
        let colored_name = self.apply_color(entry, &name, config);

        writeln!(writer, "{}{}{}", prefix, info, colored_name)?;
        Ok(())
    }
}

impl EntryWriter for TextRenderer {
    fn write_entry(
        &self,
        writer: &mut dyn Write,
        entry: &Entry,
        config: &Config,
    ) -> Result<(), TreeError> {
        self.write_entry_with_layout(writer, entry, entry.is_last, &entry.ancestors_last, config)
    }
}

impl Renderer for TextRenderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        config: &Config,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        // Root entry
        self.write_entry_with_layout(writer, &result.root, false, &[], config)?;
        helpers::count_stats(&result.root, stats);

        // Children from tree
        if let Some(ref tree) = result.tree {
            let mut state = RenderState {
                max_entries: config.max_entries,
                count: 0,
                truncated: false,
            };
            self.render_children(tree, &[], config, writer, stats, &mut state)?;
        }

        // Report
        if !config.no_report {
            writeln!(writer)?;
            let report = format_report(
                i18n::current(),
                stats.directories.saturating_sub(1),
                stats.files,
            );
            writeln!(writer, "{}", report)?;
        }

        Ok(())
    }
}

fn format_permissions(
    meta: &crate::core::entry::EntryMetadata,
    perm_mode: crate::cli::PermMode,
) -> String {
    if let Some(mode) = meta.mode {
        return format_posix_mode(mode);
    }

    match perm_mode {
        crate::cli::PermMode::Posix => {
            let mut s = String::new();
            s.push('r');
            s.push(if meta.attributes.readonly { '-' } else { 'w' });
            s.push('-');
            s.push('r');
            s.push(if meta.attributes.readonly { '-' } else { 'w' });
            s.push('-');
            s.push('r');
            s.push(if meta.attributes.readonly { '-' } else { 'w' });
            s.push('-');
            s
        }
        crate::cli::PermMode::Windows => meta.attributes.to_string_short(),
    }
}

fn format_posix_mode(mode: u32) -> String {
    let mut s = String::with_capacity(9);

    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entry::{EntryMetadata, WinAttributes};

    // ══════════════════════════════════════════════
    // format_posix_mode
    // ══════════════════════════════════════════════

    #[test]
    fn posix_mode_000() {
        assert_eq!(format_posix_mode(0o000), "---------");
    }

    #[test]
    fn posix_mode_777() {
        assert_eq!(format_posix_mode(0o777), "rwxrwxrwx");
    }

    #[test]
    fn posix_mode_644() {
        assert_eq!(format_posix_mode(0o644), "rw-r--r--");
    }

    #[test]
    fn posix_mode_755() {
        assert_eq!(format_posix_mode(0o755), "rwxr-xr-x");
    }

    #[test]
    fn posix_mode_400() {
        assert_eq!(format_posix_mode(0o400), "r--------");
    }

    #[test]
    fn posix_mode_200() {
        assert_eq!(format_posix_mode(0o200), "-w-------");
    }

    #[test]
    fn posix_mode_100() {
        assert_eq!(format_posix_mode(0o100), "--x------");
    }

    #[test]
    fn posix_mode_070() {
        assert_eq!(format_posix_mode(0o070), "---rwx---");
    }

    #[test]
    fn posix_mode_007() {
        assert_eq!(format_posix_mode(0o007), "------rwx");
    }

    #[test]
    fn posix_mode_111() {
        assert_eq!(format_posix_mode(0o111), "--x--x--x");
    }

    #[test]
    fn posix_mode_222() {
        assert_eq!(format_posix_mode(0o222), "-w--w--w-");
    }

    #[test]
    fn posix_mode_444() {
        assert_eq!(format_posix_mode(0o444), "r--r--r--");
    }

    #[test]
    fn posix_mode_600() {
        assert_eq!(format_posix_mode(0o600), "rw-------");
    }

    #[test]
    fn posix_mode_length_always_9() {
        for mode in [0, 0o777, 0o644, 0o100, 0o070] {
            assert_eq!(format_posix_mode(mode).len(), 9, "mode={:o}", mode);
        }
    }

    // ══════════════════════════════════════════════
    // format_permissions
    // ══════════════════════════════════════════════

    #[test]
    fn perms_with_unix_mode_ignores_perm_mode() {
        let meta = EntryMetadata {
            mode: Some(0o755),
            ..Default::default()
        };
        let result = format_permissions(&meta, crate::cli::PermMode::Windows);
        assert_eq!(result, "rwxr-xr-x", "Unix mode overrides perm_mode");
    }

    #[test]
    fn perms_posix_readonly_true() {
        let meta = EntryMetadata {
            mode: None,
            attributes: WinAttributes {
                readonly: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            format_permissions(&meta, crate::cli::PermMode::Posix),
            "r--r--r--"
        );
    }

    #[test]
    fn perms_posix_readonly_false() {
        let meta = EntryMetadata {
            mode: None,
            attributes: WinAttributes {
                readonly: false,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            format_permissions(&meta, crate::cli::PermMode::Posix),
            "rw-rw-rw-"
        );
    }

    #[test]
    fn perms_windows_mode_delegates_to_attrs() {
        let meta = EntryMetadata {
            mode: None,
            attributes: WinAttributes::from_raw(0x1 | 0x20), // R + A
            ..Default::default()
        };
        let result = format_permissions(&meta, crate::cli::PermMode::Windows);
        assert_eq!(result, "R--A--");
    }

    #[test]
    fn perms_posix_length_always_9() {
        for readonly in [true, false] {
            let meta = EntryMetadata {
                mode: None,
                attributes: WinAttributes {
                    readonly,
                    ..Default::default()
                },
                ..Default::default()
            };
            assert_eq!(
                format_permissions(&meta, crate::cli::PermMode::Posix).len(),
                9
            );
        }
    }

    // ══════════════════════════════════════════════
    // TextRenderer::sanitize_for_terminal
    // ══════════════════════════════════════════════

    #[test]
    fn sanitize_clean_string() {
        assert_eq!(
            TextRenderer::sanitize_for_terminal("hello world"),
            "hello world"
        );
    }

    #[test]
    fn sanitize_empty_string() {
        assert_eq!(TextRenderer::sanitize_for_terminal(""), "");
    }

    #[test]
    fn sanitize_null_byte() {
        assert_eq!(TextRenderer::sanitize_for_terminal("a\x00b"), "a?b");
    }

    #[test]
    fn sanitize_escape_char() {
        assert_eq!(TextRenderer::sanitize_for_terminal("a\x1bb"), "a?b");
    }

    #[test]
    fn sanitize_bell() {
        assert_eq!(TextRenderer::sanitize_for_terminal("a\x07b"), "a?b");
    }

    #[test]
    fn sanitize_preserves_tab() {
        assert_eq!(TextRenderer::sanitize_for_terminal("a\tb"), "a\tb");
    }

    #[test]
    fn sanitize_preserves_newline() {
        assert_eq!(TextRenderer::sanitize_for_terminal("a\nb"), "a\nb");
    }

    #[test]
    fn sanitize_bidi_rlo() {
        assert_eq!(TextRenderer::sanitize_for_terminal("\u{202E}evil"), "?evil");
    }

    #[test]
    fn sanitize_zwj() {
        assert_eq!(
            TextRenderer::sanitize_for_terminal("join\u{200D}er"),
            "join?er"
        );
    }

    #[test]
    fn sanitize_bom() {
        assert_eq!(TextRenderer::sanitize_for_terminal("\u{FEFF}file"), "?file");
    }

    #[test]
    fn sanitize_multiple_control_chars() {
        assert_eq!(
            TextRenderer::sanitize_for_terminal("\x1b[31mRED\x1b[0m"),
            "?[31mRED?[0m"
        );
    }

    #[test]
    fn sanitize_unicode_content_preserved() {
        assert_eq!(TextRenderer::sanitize_for_terminal("файл.txt"), "файл.txt");
    }

    #[test]
    fn sanitize_mixed_control_and_normal() {
        assert_eq!(
            TextRenderer::sanitize_for_terminal("normal\x01\x02text\x03end"),
            "normal??text?end"
        );
    }
}
