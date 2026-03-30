//! Platform abstraction layer.
//!
//! ALL OS-specific code is isolated here.
//! The rest of the codebase calls these functions without any `#[cfg]` guards.

#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unix;

mod reserved;

pub use reserved::{is_reserved_windows_name, should_skip_reserved_name};
use std::path::{Path, PathBuf};

/// File identification info (inode/file-id, volume serial, link count)
#[derive(Debug, Clone)]
pub struct FileIdInfo {
    pub file_id: u64,
    pub volume_serial: u64,
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
        windows::console::is_tty()
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
        windows::reparse::get_junction_target(path)
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
        windows::attributes::get_file_id(path).ok()
    }
    #[cfg(not(windows))]
    {
        unix::get_file_id(path)
    }
}

/// Get file ID, **following** symlinks and reparse points.
///
/// Unlike [`get_file_id`] (which uses `symlink_metadata` on Unix),
/// this variant resolves through symlinks so the returned identity
/// corresponds to the *target* — required for cycle detection.
///
/// On Windows `CreateFileW` already follows reparse points, so this
/// delegates to [`get_file_id`].
pub fn get_file_id_follow(path: &Path) -> Option<FileIdInfo> {
    #[cfg(windows)]
    {
        //TODO: CreateFileW without FILE_FLAG_OPEN_REPARSE_POINT follows junctions/symlinks
        get_file_id(path)
    }
    #[cfg(not(windows))]
    {
        unix::get_file_id_follow(path)
    }
}

/// Get raw Windows file attributes (always None on non-Windows)
pub fn get_file_attributes_raw(path: &Path) -> Option<u32> {
    #[cfg(windows)]
    {
        windows::attributes::get_file_attributes(path).ok()
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

// ═══════════════════════════════════════
// Alternate Data Streams
// ═══════════════════════════════════════

/// NTFS Alternate Data Stream info (name + size).
#[derive(Debug, Clone)]
pub struct AdsStreamInfo {
    pub name: String,
    pub size: u64,
}

/// Enumerate NTFS Alternate Data Streams for the given path.
///
/// Returns only alternate streams — the default `::$DATA` is filtered out.
/// Always returns empty `Vec` on non-Windows platforms or non-NTFS volumes.
pub fn get_alternate_streams(path: &Path) -> Vec<AdsStreamInfo> {
    #[cfg(windows)]
    {
        windows::streams::get_alternate_streams(path)
            .into_iter()
            .map(|s| AdsStreamInfo {
                name: s.name,
                size: s.size,
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Vec::new()
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

/// Convert path to long path format (\\?\) on Windows, identity on other platforms.
///
/// Handles three cases:
/// - `\\?\...` or `\\.\...` — already extended or device path, returned as-is
/// - `\\server\share\...` — UNC path, converted to `\\?\UNC\server\share\...`
/// - `C:\...` — regular absolute, converted to `\\?\C:\...`
///
/// Uses OsString operations to preserve non-UTF-8 (WTF-16) path components.
pub fn to_long_path(path: &Path, use_long_paths: bool) -> PathBuf {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        if use_long_paths && path.is_absolute() {
            let wide: Vec<u16> = path.as_os_str().encode_wide().collect();

            // Already extended-length or device path — return as-is
            // \\?\ = [5C 5C 3F 5C], \\.\ = [5C 5C 2E 5C]
            if wide.len() >= 4
                && wide[0] == b'\\' as u16
                && wide[1] == b'\\' as u16
                && (wide[2] == b'?' as u16 || wide[2] == b'.' as u16)
                && wide[3] == b'\\' as u16
            {
                return path.to_path_buf();
            }

            // UNC path: \\server\share → \\?\UNC\server\share
            if wide.len() >= 2 && wide[0] == b'\\' as u16 && wide[1] == b'\\' as u16 {
                let prefix: Vec<u16> = "\\\\?\\UNC\\".encode_utf16().collect();
                let mut result = prefix;
                result.extend_from_slice(&wide[2..]); // skip leading \\
                return PathBuf::from(OsString::from_wide(&result));
            }

            // Regular absolute path: C:\... → \\?\C:\...
            let prefix: Vec<u16> = "\\\\?\\".encode_utf16().collect();
            let mut result = prefix;
            result.extend_from_slice(&wide);
            return PathBuf::from(OsString::from_wide(&result));
        }
        path.to_path_buf()
    }
    #[cfg(not(windows))]
    {
        let _ = use_long_paths;
        path.to_path_buf()
    }
}

/// Get file owner (Unix: UID as string, Windows: None)
pub fn get_file_owner(path: &Path) -> Option<String> {
    #[cfg(windows)]
    {
        windows::permissions::get_file_owner(path)
    }
    #[cfg(not(windows))]
    {
        unix::get_file_owner(path)
    }
}

/// Get file group (Unix: GID as string, Windows: None)
pub fn get_file_group(path: &Path) -> Option<String> {
    #[cfg(windows)]
    {
        windows::permissions::get_file_group(path)
    }
    #[cfg(not(windows))]
    {
        unix::get_file_group(path)
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
        Some(windows::locale::get_user_language_id())
    }
    #[cfg(not(windows))]
    {
        None
    }
}

// ═══════════════════════════════════════
// Executable detection
// ═══════════════════════════════════════

/// Check if a file is executable.
///
/// - **Unix**: checks permission bits (`mode & 0o111 != 0`)
/// - **Windows**: checks file extension (exe, com, bat, cmd, ps1, vbs, js, msi)
pub fn is_executable(path: &Path) -> bool {
    #[cfg(windows)]
    {
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
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::symlink_metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
}
