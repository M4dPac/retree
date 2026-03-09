//! Shared helpers for all render backends.
//!
//! Eliminates code duplication across text/xml/json/html renderers.

use crate::core::entry::{Entry, EntryType};
use crate::core::walker::TreeStats;

/// Count an entry in the tree statistics.
///
/// Increments directories/files/symlinks counters as appropriate.
/// Used by all renderers during tree traversal.
pub fn count_stats(entry: &Entry, stats: &mut TreeStats) {
    match &entry.entry_type {
        EntryType::Directory => {
            stats.directories += 1;
        }
        EntryType::Symlink { target, broken } => {
            stats.symlinks += 1;
            // GNU tree counts symlinks to directories as directories
            if !broken
                && entry
                    .path
                    .parent()
                    .map(|p| p.join(target))
                    .map(|resolved| resolved.is_dir())
                    .unwrap_or(false)
            {
                stats.directories += 1;
            } else {
                stats.files += 1;
            }
        }
        _ => {
            stats.files += 1;
        }
    }
}
/// Format size in human-readable form.
///
/// Uses IEC units (KiB, MiB, ...) by default,
/// or SI units (KB, MB, ...) when `si` is true.
pub fn format_human_size(size: u64, si: bool) -> String {
    let (divisor, units) = if si {
        (1000.0, &["B", "KB", "MB", "GB", "TB", "PB"][..])
    } else {
        (1024.0, &["B", "KiB", "MiB", "GiB", "TiB", "PiB"][..])
    };

    let mut val = size as f64;
    let mut unit_idx = 0;

    while val >= divisor && unit_idx < units.len() - 1 {
        val /= divisor;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}{}", val, units[unit_idx])
    } else {
        format!("{:.1}{}", val, units[unit_idx])
    }
}

/// Escape special characters for XML output.
pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape special characters for HTML output.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Get entry type as a static string (for JSON/XML output).
pub fn entry_type_str(entry_type: &EntryType) -> &'static str {
    match entry_type {
        EntryType::File => "file",
        EntryType::Directory => "directory",
        EntryType::Symlink { .. } => "link",
        EntryType::Junction { .. } => "link",
        EntryType::HardLink { .. } => "file",
        EntryType::Ads { .. } => "stream",
        EntryType::Other => "other",
    }
}
