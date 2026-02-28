use std::io::Write;

use crate::config::{Config, LineStyle};
use crate::error::TreeError;
use crate::i18n::{self, format_report, get_message, MessageKey};
use crate::walker::{EntryType, TreeEntry, TreeStats};

use super::TreeOutput;

pub struct TextFormatter {
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

impl TextFormatter {
    pub fn new(config: &Config) -> Self {
        TextFormatter {
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

    fn format_prefix(&self, entry: &TreeEntry, config: &Config) -> String {
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

    fn format_name(&self, entry: &TreeEntry, config: &Config) -> String {
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
                EntryType::Symlink { .. } => name.push('@'),
                EntryType::Junction { .. } => name.push('@'),
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
                    // Use localized "broken" message
                    let broken_msg = get_message(i18n::current(), MessageKey::BrokenLink);
                    name.push_str(&format!(" [{}]", broken_msg));
                }
            }
            EntryType::Junction { target } => {
                name.push_str(" => ");
                name.push_str(&target.display().to_string());
            }
            _ => {}
        }

        name
    }

    fn format_info(&self, entry: &TreeEntry, config: &Config) -> String {
        let mut info = String::new();

        if let Some(ref meta) = entry.metadata {
            if config.show_size {
                let size_str = if config.human_readable {
                    format_human_size(meta.size, config.si_units)
                } else {
                    format!("{:>10}", meta.size)
                };
                info.push_str(&format!("[{:>7}]  ", size_str));
            }

            if config.show_date {
                if let Some(modified) = meta.modified {
                    use chrono::{DateTime, Local};
                    let dt: DateTime<Local> = modified.into();
                    let formatted = dt.format(&config.time_fmt).to_string();
                    info.push_str(&format!("[{}]  ", formatted));
                }
            }

            if config.show_permissions {
                let perm_str = match config.perm_mode {
                    crate::cli::PermMode::Posix => {
                        // Generate POSIX-style permissions (rwxr-xr-x format)
                        let mut s = String::new();
                        // Owner permissions
                        s.push(if meta.attributes.readonly { '-' } else { 'r' });
                        s.push(if meta.attributes.readonly { '-' } else { 'w' });
                        s.push(if meta.attributes.readonly { '-' } else { 'x' });
                        // Group permissions
                        s.push(if meta.attributes.readonly { '-' } else { 'r' });
                        s.push(if meta.attributes.readonly { '-' } else { 'w' });
                        s.push(if meta.attributes.readonly { '-' } else { 'x' });
                        // Other permissions
                        s.push(if meta.attributes.readonly { '-' } else { 'r' });
                        s.push(if meta.attributes.readonly { '-' } else { 'w' });
                        s.push(if meta.attributes.readonly { '-' } else { 'x' });
                        s
                    }
                    crate::cli::PermMode::Windows => meta.attributes.to_string_short(),
                };
                info.push_str(&format!("[{}]  ", perm_str));
            }

            if config.show_inodes {
                info.push_str(&format!("[{:>10x}]  ", meta.inode));
            }

            if config.show_device {
                info.push_str(&format!("[{:>8x}]  ", meta.device));
            }
        }

        info
    }

    fn apply_color(&self, entry: &TreeEntry, text: &str, config: &Config) -> String {
        if !self.color_enabled {
            return text.to_string();
        }

        let color_code = config.color_scheme.get_color(entry);
        if color_code.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        }
    }
}

impl TreeOutput for TextFormatter {
    fn begin<W: Write>(&mut self, _writer: &mut W) -> Result<(), TreeError> {
        Ok(())
    }

    fn write_entry<W: Write>(
        &mut self,
        writer: &mut W,
        entry: &TreeEntry,
        config: &Config,
    ) -> Result<(), TreeError> {
        let prefix = self.format_prefix(entry, config);
        let info = self.format_info(entry, config);
        let name = self.format_name(entry, config);
        let colored_name = self.apply_color(entry, &name, config);

        writeln!(writer, "{}{}{}", prefix, info, colored_name)?;

        Ok(())
    }

    fn end<W: Write>(
        &mut self,
        writer: &mut W,
        stats: &TreeStats,
        config: &Config,
    ) -> Result<(), TreeError> {
        if !config.no_report {
            writeln!(writer)?;

            // Use localized report with proper pluralization
            let report = format_report(
                i18n::current(),
                stats.directories.saturating_sub(1), // Subtract root
                stats.files,
            );
            writeln!(writer, "{}", report)?;
        }

        Ok(())
    }
}

fn format_human_size(size: u64, si: bool) -> String {
    let (divisor, units) = if si {
        (1000.0, ["B", "KB", "MB", "GB", "TB", "PB"])
    } else {
        (1024.0, ["B", "KiB", "MiB", "GiB", "TiB", "PiB"])
    };

    let mut size = size as f64;
    let mut unit_idx = 0;

    while size >= divisor && unit_idx < units.len() - 1 {
        size /= divisor;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}{}", size, units[unit_idx])
    } else {
        format!("{:.1}{}", size, units[unit_idx])
    }
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
