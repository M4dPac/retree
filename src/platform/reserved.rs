//! Windows reserved device name detection.
//!
//! Platform-independent pure string logic — usable on any OS
//! for cross-platform archive safety, path validation, etc.

/// Check whether a filename is a Windows reserved device name.
///
/// Reserved: CON, PRN, AUX, NUL, COM1–COM9, LPT1–LPT9.
/// Case-insensitive; ignores extensions (`NUL.txt` → reserved).
pub fn is_reserved_windows_name(name: &str) -> bool {
    // Strip the first extension: "CON.txt" → "CON", "NUL.tar.gz" → "NUL"
    let stem = match name.find('.') {
        Some(pos) => &name[..pos],
        None => name,
    };

    // All reserved names are 3 or 4 ASCII characters
    if !(3..=4).contains(&stem.len()) {
        return false;
    }

    // Stack-allocated uppercase (max 4 bytes)
    let mut buf = [0u8; 4];
    for (i, &b) in stem.as_bytes().iter().enumerate() {
        buf[i] = b.to_ascii_uppercase();
    }
    let upper = std::str::from_utf8(&buf[..stem.len()]).unwrap_or("");

    matches!(
        upper,
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

/// Whether a directory entry should be skipped as a reserved device name.
///
/// Returns `true` on Windows for CON, NUL, PRN, etc.;
/// always `false` on other platforms (these names are valid on Unix).
pub fn should_skip_reserved_name(name: &str) -> bool {
    #[cfg(windows)]
    {
        is_reserved_windows_name(name)
    }
    #[cfg(not(windows))]
    {
        let _ = name;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::is_reserved_windows_name;

    #[test]
    fn basic_reserved() {
        for n in ["CON", "PRN", "AUX", "NUL"] {
            assert!(is_reserved_windows_name(n), "{n}");
        }
    }

    #[test]
    fn com_lpt_range() {
        for i in 1..=9 {
            assert!(is_reserved_windows_name(&format!("COM{i}")));
            assert!(is_reserved_windows_name(&format!("LPT{i}")));
        }
    }

    #[test]
    fn case_insensitive() {
        for n in ["con", "Con", "cON", "nUl", "Lpt1", "com9"] {
            assert!(is_reserved_windows_name(n), "{n}");
        }
    }

    #[test]
    fn with_extension() {
        for n in ["CON.txt", "nul.tar.gz", "AUX.log", "COM1.serial"] {
            assert!(is_reserved_windows_name(n), "{n}");
        }
    }

    #[test]
    fn not_reserved() {
        for n in [
            "",
            "CO",
            "CONNN",
            "CONNECT",
            "console.log",
            "COM10",
            "COM0",
            "LPT0",
            "LPT10",
            "NULLIFY",
            "auxiliary",
            "normal.txt",
            "a",
        ] {
            assert!(!is_reserved_windows_name(n), "{n} should NOT match");
        }
    }
}
