mod natural;

use crate::cli::SortType;
use std::fs::DirEntry;

pub use natural::natural_cmp;

#[derive(Debug, Clone)]
pub struct SortConfig {
    pub sort_type: SortType,
    pub reverse: bool,
    pub dirs_first: bool,
    pub files_first: bool,
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

    // Level 1: compare only alphanumeric chars, case-insensitive
    let a_alnum: String = a_str
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();
    let b_alnum: String = b_str
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();

    a_alnum
        .cmp(&b_alnum)
        // Level 2: case-insensitive with punctuation
        .then_with(|| {
            let a_lower = a_str.to_lowercase();
            let b_lower = b_str.to_lowercase();
            a_lower.cmp(&b_lower)
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
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();

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
                let a_name = a.file_name().to_string_lossy().to_string();
                let b_name = b.file_name().to_string_lossy().to_string();
                natural_cmp(&a_name, &b_name)
            }
            SortType::Size => {
                let a_size = a.metadata().map(|m| m.len()).unwrap_or(0);
                let b_size = b.metadata().map(|m| m.len()).unwrap_or(0);
                a_size.cmp(&b_size)
            }
            SortType::Mtime => {
                let a_time = a.metadata().ok().and_then(|m| m.modified().ok());
                let b_time = b.metadata().ok().and_then(|m| m.modified().ok());
                b_time.cmp(&a_time)
            }
            SortType::Ctime => {
                let a_time = a.metadata().ok().and_then(|m| m.created().ok());
                let b_time = b.metadata().ok().and_then(|m| m.created().ok());
                b_time.cmp(&a_time)
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
