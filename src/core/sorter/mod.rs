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

pub fn sort_entries(entries: &mut [DirEntry], config: &SortConfig) {
    // Skip sorting if:
    // 1. SortType::None (unsorted)
    // 2. 0 or 1 entries (nothing to sort)
    if matches!(config.sort_type, SortType::None) || entries.len() <= 1 {
        return;
    }

    // Use sort_unstable_by for better performance
    // (avoids maintaining stable order, which is faster)
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
                a_name.cmp(&b_name)
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
