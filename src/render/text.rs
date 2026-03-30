use std::io::Write;

use crate::config::{Config, LineStyle};
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::EntryWriter;
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;
use crate::i18n::{self, format_report, get_message, MessageKey};

use super::helpers;
use super::traits::Renderer;
use super::RenderState;

pub struct TextRenderer;

struct TreeChars {
    branch: &'static [u8],
    vertical: &'static [u8],
    last_branch: &'static [u8],
    space: &'static [u8],
}

const ANSI_CHARS: TreeChars = TreeChars {
    branch: "├── ".as_bytes(),
    vertical: "│   ".as_bytes(),
    last_branch: "└── ".as_bytes(),
    space: "    ".as_bytes(),
};

const CP437_CHARS: TreeChars = TreeChars {
    branch: b"\xc3\xc4\xc4 ",
    vertical: b"\xb3   ",
    last_branch: b"\xc0\xc4\xc4 ",
    space: b"    ",
};

const ASCII_CHARS: TreeChars = TreeChars {
    branch: b"|-- ",
    vertical: b"|   ",
    last_branch: b"`-- ",
    space: b"    ",
};

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextRenderer {
    pub fn new() -> Self {
        TextRenderer
    }

    fn get_chars(config: &Config) -> &'static TreeChars {
        match config.line_style {
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

    fn write_prefix(
        &self,
        writer: &mut dyn Write,
        depth: usize,
        is_last: bool,
        ancestors_last: &[bool],
        config: &Config,
    ) -> Result<(), TreeError> {
        if config.no_indent {
            return Ok(());
        }

        let chars = Self::get_chars(config);

        for &ancestor_last in ancestors_last {
            if ancestor_last {
                writer.write_all(chars.space)?;
            } else {
                writer.write_all(chars.vertical)?;
            }
        }

        if depth > 0 {
            if is_last {
                writer.write_all(chars.last_branch)?;
            } else {
                writer.write_all(chars.branch)?;
            }
        }

        Ok(())
    }

    fn format_name(&self, entry: &Entry, config: &Config) -> String {
        let mut name = String::new();

        if config.icons_enabled {
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
                    // Text output uses local time — matches user's terminal context.
                    // Machine-readable formats (XML, JSON) use UTC instead.
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
        if !config.color_enabled {
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

    /// Write a single entry with explicit layout info (no Entry clone needed).
    fn write_entry_with_layout(
        &self,
        writer: &mut dyn Write,
        entry: &Entry,
        is_last: bool,
        ancestors_last: &[bool],
        config: &Config,
    ) -> Result<(), TreeError> {
        // write prefix directly as byte
        self.write_prefix(writer, entry.depth, is_last, ancestors_last, config)?;

        let info = self.format_info(entry, config);
        let info = if config.safe_print {
            Self::sanitize_for_terminal(&info)
        } else {
            info
        };
        let name = self.format_name(entry, config);
        let colored_name = self.apply_color(entry, &name, config);

        write!(writer, "{}{}", info, colored_name)?;
        writeln!(writer)?;
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
        &self,
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
            super::walk_tree(
                tree,
                &[],
                stats,
                &mut state,
                &mut |entry, is_last, ancestors| {
                    self.write_entry_with_layout(writer, entry, is_last, ancestors, config)
                },
            )?;
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

    // ══════════════════════════════════════════════
    // TreeChars constants — byte values
    // ══════════════════════════════════════════════

    // ANSI: UTF-8 box-drawing characters
    #[test]
    fn ansi_chars_are_valid_utf8() {
        assert!(std::str::from_utf8(ANSI_CHARS.branch).is_ok());
        assert!(std::str::from_utf8(ANSI_CHARS.vertical).is_ok());
        assert!(std::str::from_utf8(ANSI_CHARS.last_branch).is_ok());
    }

    #[test]
    fn ansi_chars_correct_symbols() {
        assert_eq!(std::str::from_utf8(ANSI_CHARS.branch).unwrap(), "├── ");
        assert_eq!(std::str::from_utf8(ANSI_CHARS.vertical).unwrap(), "│   ");
        assert_eq!(std::str::from_utf8(ANSI_CHARS.last_branch).unwrap(), "└── ");
    }

    // CP437: single-byte DOS values — NOT valid UTF-8
    #[test]
    fn cp437_branch_is_not_utf8() {
        // 0xC3 0xC4 0xC4 is not a valid UTF-8 sequence
        assert!(std::str::from_utf8(CP437_CHARS.branch).is_err());
    }

    #[test]
    fn cp437_chars_correct_bytes() {
        assert_eq!(CP437_CHARS.branch[0], 0xC3); // ├
        assert_eq!(CP437_CHARS.branch[1], 0xC4); // ─
        assert_eq!(CP437_CHARS.branch[2], 0xC4); // ─
        assert_eq!(CP437_CHARS.vertical[0], 0xB3); // │
        assert_eq!(CP437_CHARS.last_branch[0], 0xC0); // └
    }

    #[test]
    fn cp437_space_is_ascii() {
        // Space padding must be plain ASCII in every mode
        assert_eq!(CP437_CHARS.space, b"    ");
    }

    // ASCII: only printable ASCII characters
    #[test]
    fn ascii_chars_are_valid_utf8() {
        assert!(std::str::from_utf8(ASCII_CHARS.branch).is_ok());
        assert!(std::str::from_utf8(ASCII_CHARS.vertical).is_ok());
        assert!(std::str::from_utf8(ASCII_CHARS.last_branch).is_ok());
    }

    #[test]
    fn ascii_chars_correct_symbols() {
        assert_eq!(std::str::from_utf8(ASCII_CHARS.branch).unwrap(), "|-- ");
        assert_eq!(std::str::from_utf8(ASCII_CHARS.vertical).unwrap(), "|   ");
        assert_eq!(
            std::str::from_utf8(ASCII_CHARS.last_branch).unwrap(),
            "`-- "
        );
    }

    // All three sets must have equal branch/vertical/last_branch widths
    // to keep tree columns aligned correctly
    #[test]
    fn all_chars_have_equal_branch_widths() {
        // Column width must be equal for correct tree alignment.
        // Byte length is irrelevant — UTF-8 box-drawing chars are multi-byte.
        fn col_width(bytes: &[u8]) -> usize {
            match std::str::from_utf8(bytes) {
                Ok(s) => s.chars().count(),
                // CP437 is not valid UTF-8 — count raw bytes directly,
                // each CP437 byte is exactly one terminal column
                Err(_) => bytes.len(),
            }
        }

        for chars in [&ANSI_CHARS, &CP437_CHARS, &ASCII_CHARS] {
            let w = col_width(chars.branch);
            assert_eq!(
                col_width(chars.last_branch),
                w,
                "last_branch column width must equal branch"
            );
            assert_eq!(
                col_width(chars.vertical),
                w,
                "vertical column width must equal branch"
            );
            assert_eq!(
                col_width(chars.space),
                w,
                "space column width must equal branch"
            );
        }
    }

    // ══════════════════════════════════════════════
    // write_prefix
    // ══════════════════════════════════════════════

    fn write_prefix_to_vec(
        depth: usize,
        is_last: bool,
        ancestors_last: &[bool],
        line_style: LineStyle,
        no_indent: bool,
    ) -> Vec<u8> {
        let renderer = TextRenderer::new();
        let config = Config {
            line_style,
            no_indent,
            ..Config::default()
        };
        let mut buf = Vec::new();
        renderer
            .write_prefix(&mut buf, depth, is_last, ancestors_last, &config)
            .unwrap();
        buf
    }

    #[test]
    fn prefix_no_indent_produces_empty() {
        let out = write_prefix_to_vec(2, false, &[false, false], LineStyle::Ansi, true);
        assert!(out.is_empty());
    }

    #[test]
    fn prefix_depth_zero_produces_empty() {
        // Root entry — no prefix should be written
        let out = write_prefix_to_vec(0, false, &[], LineStyle::Ansi, false);
        assert!(out.is_empty());
    }

    #[test]
    fn prefix_depth1_not_last_ansi() {
        let out = write_prefix_to_vec(1, false, &[], LineStyle::Ansi, false);
        assert_eq!(out, ANSI_CHARS.branch);
    }

    #[test]
    fn prefix_depth1_is_last_ansi() {
        let out = write_prefix_to_vec(1, true, &[], LineStyle::Ansi, false);
        assert_eq!(out, ANSI_CHARS.last_branch);
    }

    #[test]
    fn prefix_depth1_not_last_ascii() {
        let out = write_prefix_to_vec(1, false, &[], LineStyle::Ascii, false);
        assert_eq!(out, ASCII_CHARS.branch);
    }

    #[test]
    fn prefix_depth1_not_last_cp437() {
        let out = write_prefix_to_vec(1, false, &[], LineStyle::Cp437, false);
        assert_eq!(out, CP437_CHARS.branch);
    }

    #[test]
    fn prefix_ancestor_not_last_adds_vertical() {
        // ancestors_last = [false] → ancestor is not last → draw vertical bar
        let out = write_prefix_to_vec(2, false, &[false], LineStyle::Ansi, false);
        let mut expected = Vec::new();
        expected.extend_from_slice(ANSI_CHARS.vertical); // from ancestor
        expected.extend_from_slice(ANSI_CHARS.branch); // current entry
        assert_eq!(out, expected);
    }

    #[test]
    fn prefix_ancestor_is_last_adds_space() {
        // ancestors_last = [true] → ancestor is last → space instead of vertical
        let out = write_prefix_to_vec(2, false, &[true], LineStyle::Ansi, false);
        let mut expected = Vec::new();
        expected.extend_from_slice(ANSI_CHARS.space); // from ancestor
        expected.extend_from_slice(ANSI_CHARS.branch); // current entry
        assert_eq!(out, expected);
    }

    #[test]
    fn prefix_deep_nesting_correct_order() {
        // Depth 3: ancestors [false, true] → vertical, space, then branch
        let out = write_prefix_to_vec(3, false, &[false, true], LineStyle::Ansi, false);
        let mut expected = Vec::new();
        expected.extend_from_slice(ANSI_CHARS.vertical); // ancestor 1 (not last)
        expected.extend_from_slice(ANSI_CHARS.space); // ancestor 2 (last)
        expected.extend_from_slice(ANSI_CHARS.branch); // current entry
        assert_eq!(out, expected);
    }

    #[test]
    fn prefix_cp437_ancestor_uses_cp437_vertical() {
        let out = write_prefix_to_vec(2, false, &[false], LineStyle::Cp437, false);
        let mut expected = Vec::new();
        expected.extend_from_slice(CP437_CHARS.vertical);
        expected.extend_from_slice(CP437_CHARS.branch);
        assert_eq!(out, expected);
        // Verify this is NOT the UTF-8 vertical bar
        assert_ne!(&out[..CP437_CHARS.vertical.len()], ANSI_CHARS.vertical);
    }

    #[test]
    fn prefix_ascii_and_ansi_differ() {
        let ansi = write_prefix_to_vec(1, false, &[], LineStyle::Ansi, false);
        let ascii = write_prefix_to_vec(1, false, &[], LineStyle::Ascii, false);
        assert_ne!(ansi, ascii);
    }

    #[test]
    fn prefix_cp437_and_ansi_differ() {
        // Regression guard: CP437 and ANSI constants must not be identical
        let ansi = write_prefix_to_vec(1, false, &[], LineStyle::Ansi, false);
        let cp437 = write_prefix_to_vec(1, false, &[], LineStyle::Cp437, false);
        assert_ne!(ansi, cp437);
    }
}
