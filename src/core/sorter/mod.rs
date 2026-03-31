mod natural;

use clap::ValueEnum;
use std::fs::DirEntry;

pub use natural::natural_cmp;

#[derive(Debug, Clone)]
pub struct SortConfig {
    pub sort_type: SortType,
    pub reverse: bool,
    pub dirs_first: bool,
    pub files_first: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortType {
    Name,
    Size,
    Mtime,
    Ctime,
    Version,
    None,
}

impl Default for SortConfig {
    fn default() -> Self {
        SortConfig {
            sort_type: SortType::Name,
            reverse: false,
            dirs_first: false,
            files_first: false,
        }
    }
}

/// Locale-aware comparison matching GNU tree's `strcoll()` behavior.
///
/// Under en_US.UTF-8, `strcoll()` uses multi-level comparison:
/// 1. Primary: only alphanumeric characters, case-insensitive
/// 2. Secondary: full string, case-insensitive
/// 3. Tertiary: full string, case-sensitive (deterministic tie-break)
///
/// This explains why tree sorts "file10.txt" before "file1.txt":
/// stripping punctuation gives "file10txt" vs "file1txt",
/// and '0' < 't'.
fn name_cmp_locale(a: &std::ffi::OsStr, b: &std::ffi::OsStr) -> std::cmp::Ordering {
    let a_str = a.to_string_lossy();
    let b_str = b.to_string_lossy();

    // Level 1: compare only alphanumeric chars, case-insensitive.
    // Iterator::cmp avoids collecting into intermediate Strings —
    // eliminates 2–4 heap allocations per comparison.
    a_str
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .cmp(
            b_str
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(|c| c.to_lowercase()),
        )
        // Level 2: case-insensitive with punctuation
        .then_with(|| {
            a_str
                .chars()
                .flat_map(|c| c.to_lowercase())
                .cmp(b_str.chars().flat_map(|c| c.to_lowercase()))
        })
        // Level 3: case-sensitive tie-break
        .then_with(|| a_str.cmp(&b_str))
}

pub fn sort_entries(entries: &mut [DirEntry], config: &SortConfig) {
    // Skip sorting if:
    // 1. SortType::None (unsorted)
    // 2. 0 or 1 entries (nothing to sort)
    if matches!(config.sort_type, SortType::None) || entries.len() <= 1 {
        return;
    }

    entries.sort_unstable_by(|a, b| {
        // Handle dirs_first / files_first
        if config.dirs_first || config.files_first {
            // Use cached file_type() to avoid stat() on every comparison.
            // For symlinks, fall back to path().is_dir() to follow the link
            // (matches GNU tree behavior: symlinks to dirs count as dirs).
            let a_is_dir = a
                .file_type()
                .map(|ft| ft.is_dir() || (ft.is_symlink() && a.path().is_dir()))
                .unwrap_or(false);
            let b_is_dir = b
                .file_type()
                .map(|ft| ft.is_dir() || (ft.is_symlink() && b.path().is_dir()))
                .unwrap_or(false);

            if a_is_dir != b_is_dir {
                return if config.dirs_first {
                    b_is_dir.cmp(&a_is_dir)
                } else {
                    a_is_dir.cmp(&b_is_dir)
                };
            }
        }

        let cmp = match config.sort_type {
            SortType::Name => {
                let a_name = a.file_name();
                let b_name = b.file_name();
                name_cmp_locale(&a_name, &b_name)
            }
            SortType::Version => {
                let a_name = a.file_name();
                let b_name = b.file_name();
                natural_cmp(&a_name.to_string_lossy(), &b_name.to_string_lossy())
            }
            SortType::Size => {
                let a_size = a.metadata().map(|m| m.len()).unwrap_or(0);
                let b_size = b.metadata().map(|m| m.len()).unwrap_or(0);
                a_size.cmp(&b_size)
            }
            SortType::Mtime => {
                let a_time = a.metadata().ok().and_then(|m| m.modified().ok());
                let b_time = b.metadata().ok().and_then(|m| m.modified().ok());
                a_time.cmp(&b_time)
            }
            SortType::Ctime => {
                let a_time = a.metadata().ok().and_then(|m| m.created().ok());
                let b_time = b.metadata().ok().and_then(|m| m.created().ok());
                a_time.cmp(&b_time)
            }

            SortType::None => std::cmp::Ordering::Equal,
        };

        if config.reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });
}
