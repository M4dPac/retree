//! Windows path transformations (extended-length paths).

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// Convert an absolute path to extended-length form (`\\?\`).
///
/// Handles three cases:
/// - `\\?\...` or `\\.\...` — already extended or device path, returned as-is
/// - `\\server\share\...` — UNC path, converted to `\\?\UNC\server\share\...`
/// - `C:\...` — regular absolute, converted to `\\?\C:\...`
///
/// Uses OsString operations to preserve non-UTF-8 (WTF-16) path components.
pub fn to_long_path(path: &Path) -> PathBuf {
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
    PathBuf::from(OsString::from_wide(&result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_absolute_path() {
        let result = to_long_path(Path::new(r"C:\Users\test"));
        assert_eq!(result, PathBuf::from(r"\\?\C:\Users\test"));
    }

    #[test]
    fn already_extended() {
        let input = Path::new(r"\\?\C:\Users\test");
        assert_eq!(to_long_path(input), input);
    }

    #[test]
    fn device_path_unchanged() {
        let input = Path::new(r"\\.\PhysicalDrive0");
        assert_eq!(to_long_path(input), input);
    }

    #[test]
    fn unc_path() {
        let result = to_long_path(Path::new(r"\\server\share\dir"));
        assert_eq!(result, PathBuf::from(r"\\?\UNC\server\share\dir"));
    }

    #[test]
    fn drive_root() {
        let result = to_long_path(Path::new(r"D:\"));
        assert_eq!(result, PathBuf::from(r"\\?\D:\"));
    }
}
