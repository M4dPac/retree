use std::io::Write;

use crate::config::{Config, LineStyle};
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;
use crate::i18n::{self, format_report, get_message, MessageKey};

use super::context::RenderContext;
use super::helpers;
use super::traits::Renderer;

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

    fn format_prefix(&self, entry: &Entry, config: &Config) -> String {
        if config.no_indent {
            return String::new();
        }

        let chars = self.get_chars();
        let mut prefix = String::new();

        for &is_last in &entry.ancestors_last {
            if is_last {
                prefix.push_str(chars.space);
            } else {
                prefix.push_str(chars.vertical);
            }
        }

        if entry.depth > 0 {
            if entry.is_last {
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
            name.push_str(&entry.path.display().to_string());
        } else {
            name.push_str(entry.name_str());
        }

        if config.classify {
            match &entry.entry_type {
                EntryType::Directory => name.push('/'),
                EntryType::Symlink { .. } | EntryType::Junction { .. } => {}
                EntryType::File => {
                    if is_executable(&entry.path) {
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

    fn write_entry<W: Write>(
        &self,
        writer: &mut W,
        entry: &Entry,
        config: &Config,
    ) -> Result<(), TreeError> {
        let prefix = self.format_prefix(entry, config);
        let info = self.format_info(entry, config);
        let name = self.format_name(entry, config);
        let colored_name = self.apply_color(entry, &name, config);

        writeln!(writer, "{}{}{}", prefix, info, colored_name)?;

        Ok(())
    }
}

impl Renderer for TextRenderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        ctx: &RenderContext,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        let config = ctx.config;

        // Root entry
        self.write_entry(writer, &result.root, config)?;
        helpers::count_stats(&result.root, stats);

        // Child entries
        for entry in &result.entries {
            self.write_entry(writer, entry, config)?;
            helpers::count_stats(entry, stats);
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

fn is_executable(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(
            ext.as_str(),
            "exe" | "com" | "bat" | "cmd" | "ps1" | "vbs" | "js" | "msi"
        )
    } else {
        false
    }
}
