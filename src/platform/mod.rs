//! Platform abstraction layer.
//!
//! ALL OS-specific code is isolated here.
//! The rest of the codebase calls these functions without any `#[cfg]` guards.

#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unix;

use std::path::{Path, PathBuf};

/// File identification info (inode/file-id, volume serial, link count)
#[derive(Debug, Clone)]
pub struct FileIdInfo {
    pub file_id: u64,
    pub volume_serial: u32,
    pub number_of_links: u32,
}

// ═══════════════════════════════════════
// Console
// ═══════════════════════════════════════

/// Enable ANSI escape sequences (Windows-specific, no-op elsewhere)
pub fn enable_ansi() {
    #[cfg(windows)]
    windows::console::enable_ansi();
}

/// Check if stdout is a TTY
pub fn is_tty() -> bool {
    #[cfg(windows)]
    {
        return windows::console::is_tty();
    }
    #[cfg(not(windows))]
    {
        unix::is_tty()
    }
}

// ═══════════════════════════════════════
// Filesystem
// ═══════════════════════════════════════

/// Get junction point target (Windows NTFS only, always None on other platforms)
pub fn get_junction_target(path: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        return windows::reparse::get_junction_target(path);
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

/// Get file ID info (inode, volume serial, link count)
pub fn get_file_id(path: &Path) -> Option<FileIdInfo> {
    #[cfg(windows)]
    {
        return windows::attributes::get_file_id(path)
            .ok()
            .map(|info| FileIdInfo {
                file_id: info.file_id,
                volume_serial: info.volume_serial,
                number_of_links: info.number_of_links,
            });
    }
    #[cfg(not(windows))]
    {
        unix::get_file_id(path)
    }
}

/// Get raw Windows file attributes (always None on non-Windows)
pub fn get_file_attributes_raw(path: &Path) -> Option<u32> {
    #[cfg(windows)]
    {
        return windows::attributes::get_file_attributes(path).ok();
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

/// Get POSIX file mode (Unix only, returns None on Windows)
pub fn get_file_mode(path: &Path) -> Option<u32> {
    #[cfg(windows)]
    {
        let _ = path;
        None
    }
    #[cfg(not(windows))]
    {
        unix::get_file_mode(path)
    }
}

/// Convert path to long path format (\\?\) on Windows, identity on other platforms
pub fn to_long_path(path: &Path, use_long_paths: bool) -> PathBuf {
    #[cfg(windows)]
    {
        if use_long_paths {
            let path_str = path.to_string_lossy();
            if !path_str.starts_with("\\\\?\\") && path.is_absolute() {
                if let Some(stripped) = path_str.strip_prefix("\\\\") {
                    let mut long_path = String::from("\\\\?\\UNC\\");
                    long_path.push_str(stripped);
                    return PathBuf::from(long_path);
                }
                let mut long_path = String::from("\\\\?\\");
                long_path.push_str(&path_str);
                return PathBuf::from(long_path);
            }
        }
        return path.to_path_buf();
    }
    #[cfg(not(windows))]
    {
        let _ = use_long_paths;
        path.to_path_buf()
    }
}

// ═══════════════════════════════════════
// Locale
// ═══════════════════════════════════════

/// Detect Windows UI language (returns primary language ID, e.g. 0x19 = Russian).
/// Returns None on non-Windows platforms.
pub fn detect_system_language_id() -> Option<u16> {
    #[cfg(windows)]
    {
        return Some(windows::locale::get_user_language_id());
    }
    #[cfg(not(windows))]
    {
        None
    }
}
