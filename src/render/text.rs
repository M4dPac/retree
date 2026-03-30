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

/// Push a value wrapped in brackets: `[value]  `
/// Used by format_info for all metadata fields.
fn push_bracketed(out: &mut String, content: &str) {
    out.push_str("  [");
    out.push_str(content);
    out.push_str("]  ");
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
                    push_bracketed(&mut name, broken_msg);
                }
                if entry.recursive_link {
                    let recursive_msg = get_message(i18n::current(), MessageKey::RecursiveLink);
                    push_bracketed(&mut name, recursive_msg);
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
            push_bracketed(&mut name, &formatted);
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
                push_bracketed(&mut info, &size_str);
            }

            if config.show_date {
                if let Some(modified) = meta.modified {
                    // Text output uses local time — matches user's terminal context.
                    // Machine-readable formats (XML, JSON) use UTC instead.
                    use chrono::{DateTime, Local};
                    let dt: DateTime<Local> = modified.into();
                    push_bracketed(&mut info, &dt.format(&config.time_fmt).to_string());
                }
            }

            if config.show_permissions {
                push_bracketed(&mut info, &format_permissions(meta, config.perm_mode));
            }

            if config.show_inodes {
                push_bracketed(&mut info, &format!("{:>10}", meta.inode));
            }

            if config.show_device {
                push_bracketed(&mut info, &format!("{:>8x}", meta.device));
            }

            if config.show_owner {
                if let Some(ref owner) = meta.owner {
                    push_bracketed(&mut info, &format!("{:<8}", owner));
                }
            }

            if config.show_group {
                if let Some(ref group) = meta.group {
                    push_bracketed(&mut info, &format!("{:<8}", group));
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
    use crate::render::test_util::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

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
        assert_eq!(
            format_permissions(&meta, crate::cli::PermMode::Windows),
            "rwxr-xr-x"
        );
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
        assert_eq!(
            format_permissions(&meta, crate::cli::PermMode::Windows),
            "R--A--"
        );
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
    // sanitize_for_terminal
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
        assert_eq!(CP437_CHARS.space, b"    ");
    }

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

    #[test]
    fn all_chars_have_equal_branch_widths() {
        fn col_width(bytes: &[u8]) -> usize {
            match std::str::from_utf8(bytes) {
                Ok(s) => s.chars().count(),
                // CP437 is not valid UTF-8 — each byte is exactly one terminal column
                Err(_) => bytes.len(),
            }
        }
        for chars in [&ANSI_CHARS, &CP437_CHARS, &ASCII_CHARS] {
            let w = col_width(chars.branch);
            assert_eq!(
                col_width(chars.last_branch),
                w,
                "last_branch width must equal branch"
            );
            assert_eq!(
                col_width(chars.vertical),
                w,
                "vertical width must equal branch"
            );
            assert_eq!(col_width(chars.space), w, "space width must equal branch");
        }
    }

    // ══════════════════════════════════════════════
    // write_prefix
    // ══════════════════════════════════════════════

    fn prefix_bytes(
        depth: usize,
        is_last: bool,
        ancestors_last: &[bool],
        config: &Config,
    ) -> Vec<u8> {
        let renderer = TextRenderer::new();
        let mut buf = Vec::new();
        renderer
            .write_prefix(&mut buf, depth, is_last, ancestors_last, config)
            .unwrap();
        buf
    }

    fn prefix_str(depth: usize, is_last: bool, ancestors_last: &[bool], config: &Config) -> String {
        String::from_utf8(prefix_bytes(depth, is_last, ancestors_last, config)).unwrap()
    }

    #[test]
    fn prefix_root_depth_zero() {
        assert_eq!(prefix_str(0, false, &[], &Config::default()), "");
    }

    #[test]
    fn prefix_depth_one_not_last() {
        assert_eq!(prefix_str(1, false, &[], &Config::default()), "├── ");
    }

    #[test]
    fn prefix_depth_one_is_last() {
        assert_eq!(prefix_str(1, true, &[], &Config::default()), "└── ");
    }

    #[test]
    fn prefix_depth_two_ancestor_not_last() {
        assert_eq!(
            prefix_str(2, false, &[false], &Config::default()),
            "│   ├── "
        );
    }

    #[test]
    fn prefix_depth_two_ancestor_is_last() {
        assert_eq!(prefix_str(2, true, &[true], &Config::default()), "    └── ");
    }

    #[test]
    fn prefix_deep_mixed_ancestors() {
        assert_eq!(
            prefix_str(4, true, &[false, true, false], &Config::default()),
            "│       │   └── "
        );
    }

    #[test]
    fn prefix_no_indent_writes_nothing() {
        let config = Config {
            no_indent: true,
            ..Config::default()
        };
        assert_eq!(prefix_bytes(3, false, &[false, true], &config).len(), 0);
    }

    #[test]
    fn prefix_ascii_branch() {
        let config = Config {
            line_style: LineStyle::Ascii,
            ..Config::default()
        };
        assert_eq!(prefix_str(1, false, &[], &config), "|-- ");
        assert_eq!(prefix_str(1, true, &[], &config), "`-- ");
    }

    #[test]
    fn prefix_ascii_vertical_ancestor() {
        let config = Config {
            line_style: LineStyle::Ascii,
            ..Config::default()
        };
        assert_eq!(prefix_str(2, true, &[false], &config), "|   `-- ");
    }

    #[test]
    fn prefix_cp437_raw_bytes() {
        let config = Config {
            line_style: LineStyle::Cp437,
            ..Config::default()
        };
        assert_eq!(prefix_bytes(1, false, &[], &config), b"\xc3\xc4\xc4 ");
        assert_eq!(prefix_bytes(1, true, &[], &config), b"\xc0\xc4\xc4 ");
    }

    #[test]
    fn prefix_cp437_vertical_ancestor() {
        let config = Config {
            line_style: LineStyle::Cp437,
            ..Config::default()
        };
        let result = prefix_bytes(2, true, &[false], &config);
        let mut expected = Vec::new();
        expected.extend_from_slice(b"\xb3   ");
        expected.extend_from_slice(b"\xc0\xc4\xc4 ");
        assert_eq!(result, expected);
    }

    #[test]
    fn prefix_cp437_differs_from_ansi() {
        let ansi = prefix_bytes(
            1,
            false,
            &[],
            &Config {
                line_style: LineStyle::Ansi,
                ..Config::default()
            },
        );
        let cp437 = prefix_bytes(
            1,
            false,
            &[],
            &Config {
                line_style: LineStyle::Cp437,
                ..Config::default()
            },
        );
        assert_ne!(ansi, cp437);
    }

    // ══════════════════════════════════════════════
    // format_info
    // ══════════════════════════════════════════════

    #[test]
    fn info_empty_without_metadata() {
        let renderer = TextRenderer::new();
        let entry = file_entry("test.txt", 0);
        assert_eq!(renderer.format_info(&entry, &Config::default()), "");
    }

    #[test]
    fn info_empty_when_no_show_flags() {
        let renderer = TextRenderer::new();
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            size: 1000,
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &Config::default()), "");
    }

    #[test]
    fn info_size_raw_right_aligned() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_size: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            size: 12345,
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [     12345]  ");
    }

    #[test]
    fn info_size_human_readable() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_size: true,
            human_readable: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            size: 1536,
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [1.5KiB]  ");
    }

    #[test]
    fn info_size_human_si() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_size: true,
            human_readable: true,
            si_units: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            size: 1500,
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [1.5KB]  ");
    }

    #[test]
    fn info_inode() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_inodes: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            inode: 42,
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [        42]  ");
    }

    #[test]
    fn info_device_hex() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_device: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            device: 0xff,
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [      ff]  ");
    }

    #[test]
    fn info_owner() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_owner: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            owner: Some("alice".into()),
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [alice   ]  ");
    }

    #[test]
    fn info_group() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_group: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            group: Some("staff".into()),
            ..Default::default()
        });
        assert_eq!(renderer.format_info(&entry, &config), "  [staff   ]  ");
    }

    #[test]
    fn info_multiple_fields_ordered() {
        let renderer = TextRenderer::new();
        let config = Config {
            show_size: true,
            show_inodes: true,
            ..Config::default()
        };
        let mut entry = file_entry("test.txt", 0);
        entry.metadata = Some(EntryMetadata {
            size: 999,
            inode: 7,
            ..Default::default()
        });
        let info = renderer.format_info(&entry, &config);
        // Size comes before inodes in format_info
        let size_pos = info.find("999").unwrap();
        let inode_pos = info.find("7").unwrap();
        assert!(size_pos < inode_pos, "size should appear before inode");
        assert_eq!(info.matches('[').count(), 2);
    }

    // ══════════════════════════════════════════════
    // format_name
    // ══════════════════════════════════════════════

    #[test]
    fn name_simple_file() {
        let renderer = TextRenderer::new();
        assert_eq!(
            renderer.format_name(&file_entry("hello.txt", 1), &Config::default()),
            "hello.txt"
        );
    }

    #[test]
    fn name_directory() {
        let renderer = TextRenderer::new();
        assert_eq!(
            renderer.format_name(&dir_entry("src", 1), &Config::default()),
            "src"
        );
    }

    #[test]
    fn name_full_path() {
        let renderer = TextRenderer::new();
        let config = Config {
            full_path: true,
            ..Config::default()
        };
        let mut entry = file_entry("src/test.rs", 1);
        entry.path = PathBuf::from("src/test.rs");
        let name = renderer.format_name(&entry, &config);
        assert!(name.contains("src") && name.contains("test.rs"));
    }

    #[test]
    fn name_classify_directory_appends_slash() {
        let renderer = TextRenderer::new();
        let config = Config {
            classify: true,
            ..Config::default()
        };
        assert_eq!(
            renderer.format_name(&dir_entry("mydir", 1), &config),
            "mydir/"
        );
    }

    #[test]
    fn name_symlink_shows_target() {
        let renderer = TextRenderer::new();
        let entry = symlink_entry("link", 1, "/target/path", false);
        let name = renderer.format_name(&entry, &Config::default());
        assert!(name.contains("->") && name.contains("/target/path"));
    }

    #[test]
    fn name_broken_symlink_annotation() {
        let renderer = TextRenderer::new();
        let entry = symlink_entry("broken", 1, "/gone", true);
        let name = renderer.format_name(&entry, &Config::default());
        assert!(name.contains("->") && name.contains('['));
    }

    #[test]
    fn name_recursive_link_annotation() {
        let renderer = TextRenderer::new();
        let mut entry = symlink_entry("loop", 1, "/loop", false);
        entry.recursive_link = true;
        let name = renderer.format_name(&entry, &Config::default());
        assert!(name.contains('['));
    }

    #[test]
    fn name_junction_shows_target() {
        let renderer = TextRenderer::new();
        let mut entry = dir_entry("junc", 1);
        entry.entry_type = EntryType::Junction {
            target: PathBuf::from("C:\\target"),
        };
        let name = renderer.format_name(&entry, &Config::default());
        assert!(name.contains("=>") && name.contains("C:\\target"));
    }

    #[test]
    fn name_filelimit_exceeded() {
        let renderer = TextRenderer::new();
        let mut entry = dir_entry("big", 1);
        entry.filelimit_exceeded = Some(5000);
        let name = renderer.format_name(&entry, &Config::default());
        assert!(name.contains('[') && name.contains("5000"));
    }

    #[test]
    fn name_safe_print_sanitizes() {
        let renderer = TextRenderer::new();
        let config = Config {
            safe_print: true,
            ..Config::default()
        };
        let mut entry = file_entry("clean", 1);
        entry.name = OsString::from("evil\x1bname");
        entry.path = PathBuf::from("evil\x1bname");
        let name = renderer.format_name(&entry, &config);
        assert!(name.contains('?') && !name.contains('\x1b'));
    }

    #[test]
    fn name_no_safe_print_preserves_control() {
        let renderer = TextRenderer::new();
        let config = Config {
            safe_print: false,
            ..Config::default()
        };
        let mut entry = file_entry("raw", 1);
        entry.name = OsString::from("file\x07bell");
        entry.path = PathBuf::from("file\x07bell");
        assert!(renderer.format_name(&entry, &config).contains('\x07'));
    }
}
