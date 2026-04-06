use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::i18n::{self, get_message, MessageKey};

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("{}", fmt_path(MessageKey::ErrAccessDenied, .0))]
    AccessDenied(PathBuf),

    #[error("Maximum internal depth exceeded at: {}", .0.display())]
    MaxDepthExceeded(PathBuf),

    #[error("{}", fmt_path(MessageKey::ErrNotFound, .0))]
    NotFound(PathBuf),

    #[error("{}", fmt_path(MessageKey::ErrNotDirectory, .0))]
    NotDirectory(PathBuf),

    #[error("{}", fmt_path(MessageKey::ErrSymlinkLoop, .0))]
    SymlinkLoop(PathBuf),

    #[error("{}", fmt_path_io(MessageKey::ErrSymlinkError, .0, .1))]
    SymlinkError(PathBuf, std::io::Error),

    #[error("{}", fmt_path(MessageKey::ErrPathTooLong, .0))]
    PathTooLong(PathBuf),

    #[error("{}", fmt_path(MessageKey::ErrReservedName, .0))]
    ReservedName(PathBuf),

    #[error("{}", fmt_path_io(MessageKey::ErrIo, .0, .1))]
    Io(PathBuf, #[source] std::io::Error),

    #[error("{}", fmt_str(MessageKey::ErrInvalidPattern, .0))]
    InvalidPattern(String),

    #[error("{0}")]
    Generic(String),
}

/// Format a localized error message with a single path placeholder.
fn fmt_path(key: MessageKey, path: &Path) -> String {
    get_message(i18n::current(), key).replace("{}", &path.display().to_string())
}

/// Format a localized error message with path + io::Error placeholders.
fn fmt_path_io(key: MessageKey, path: &Path, error: &std::io::Error) -> String {
    get_message(i18n::current(), key)
        .replacen("{}", &path.display().to_string(), 1)
        .replacen("{}", &error.to_string(), 1)
}

/// Format a localized error message with a single string placeholder.
fn fmt_str(key: MessageKey, msg: &str) -> String {
    get_message(i18n::current(), key).replace("{}", msg)
}

impl From<std::io::Error> for TreeError {
    fn from(err: std::io::Error) -> Self {
        TreeError::Generic(err.to_string())
    }
}

impl TreeError {
    /// Whether this error should affect the process exit code.
    ///
    /// `ReservedName` is an informational warning — it does not count
    /// as a hard error for exit-code purposes.
    pub fn is_hard_error(&self) -> bool {
        !matches!(self, TreeError::ReservedName(_) | TreeError::SymlinkLoop(_))
    }

    /// Map an `io::Error` with path context to the most specific variant.
    ///
    /// - `PermissionDenied` → [`AccessDenied`](TreeError::AccessDenied) (localized message)
    /// - OS "path too long"  → [`PathTooLong`](TreeError::PathTooLong) (localized message)
    /// - everything else     → [`Io`](TreeError::Io) (original OS message preserved)
    pub fn from_io(path: PathBuf, err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => TreeError::AccessDenied(path),
            _ if Self::is_path_too_long(&err) => TreeError::PathTooLong(path),
            _ => TreeError::Io(path, err),
        }
    }

    /// Detect OS-specific "path too long" / "name too long" errors.
    fn is_path_too_long(err: &std::io::Error) -> bool {
        match err.raw_os_error() {
            #[cfg(windows)]
            Some(206) => true, // ERROR_FILENAME_EXCED_RANGE
            #[cfg(not(windows))]
            Some(36) => true, // ENAMETOOLONG
            _ => false,
        }
    }
}

// ═══════════════════════════════════════
// Diagnostic output helpers
// ═══════════════════════════════════════

/// Print a diagnostic error to stderr: `retree: <message>`.
pub fn diag_error(msg: impl std::fmt::Display) {
    eprintln!("retree: {}", msg);
}

/// Print a diagnostic warning to stderr: `retree: warning: <message>`.
pub fn diag_warn(msg: impl std::fmt::Display) {
    eprintln!("retree: warning: {}", msg);
}

/// Report traversal errors to stderr. Returns count of hard errors.
pub fn report_errors(errors: &[TreeError]) -> u64 {
    for err in errors {
        diag_error(err);
    }
    errors.iter().filter(|e| e.is_hard_error()).count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    // ══════════════════════════════════════════════
    // TreeError construction and Display
    // ══════════════════════════════════════════════

    #[test]
    fn generic_error_display() {
        let err = TreeError::Generic("something failed".into());
        assert_eq!(err.to_string(), "something failed");
    }

    #[test]
    fn not_found_contains_path() {
        let err = TreeError::NotFound(PathBuf::from("/test/missing"));
        let msg = err.to_string();
        assert!(msg.contains("/test/missing"), "got: {msg}");
    }

    #[test]
    fn not_directory_contains_path() {
        let err = TreeError::NotDirectory(PathBuf::from("/test/file.txt"));
        let msg = err.to_string();
        assert!(msg.contains("/test/file.txt"), "got: {msg}");
    }

    #[test]
    fn access_denied_contains_path() {
        let err = TreeError::AccessDenied(PathBuf::from("/secret"));
        let msg = err.to_string();
        assert!(msg.contains("/secret"), "got: {msg}");
    }

    #[test]
    fn max_depth_exceeded_contains_path() {
        let err = TreeError::MaxDepthExceeded(PathBuf::from("/deep/path"));
        let msg = err.to_string();
        assert!(msg.contains("/deep/path"), "got: {msg}");
    }

    #[test]
    fn io_error_contains_path() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = TreeError::Io(PathBuf::from("/locked"), io_err);
        let msg = err.to_string();
        assert!(msg.contains("/locked"), "got: {msg}");
    }

    #[test]
    fn invalid_pattern_contains_pattern() {
        let err = TreeError::InvalidPattern("[invalid".into());
        let msg = err.to_string();
        assert!(msg.contains("[invalid"), "got: {msg}");
    }

    #[test]
    fn reserved_name_contains_path() {
        let err = TreeError::ReservedName(PathBuf::from("CON"));
        let msg = err.to_string();
        assert!(msg.contains("CON"), "got: {msg}");
    }

    #[test]
    fn symlink_loop_contains_path() {
        let err = TreeError::SymlinkLoop(PathBuf::from("/loop/link"));
        let msg = err.to_string();
        assert!(msg.contains("/loop/link"), "got: {msg}");
    }

    #[test]
    fn path_too_long_contains_path() {
        let long = "/a".repeat(200);
        let err = TreeError::PathTooLong(PathBuf::from(&long));
        let msg = err.to_string();
        assert!(msg.contains("/a/a"), "got: {msg}");
    }

    // ══════════════════════════════════════════════
    // From<io::Error>
    // ══════════════════════════════════════════════

    #[test]
    fn from_io_error_creates_generic() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tree_err: TreeError = io_err.into();
        assert!(matches!(tree_err, TreeError::Generic(_)));
        assert!(tree_err.to_string().contains("file not found"));
    }

    #[test]
    fn from_io_error_preserves_message() {
        let io_err = std::io::Error::other("custom message");
        let tree_err: TreeError = io_err.into();
        assert_eq!(tree_err.to_string(), "custom message");
    }

    // ══════════════════════════════════════════════
    // Debug is implemented
    // ══════════════════════════════════════════════

    #[test]
    fn debug_format_works() {
        let err = TreeError::Generic("test".into());
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty());
    }

    // ══════════════════════════════════════════════
    // is_hard_error
    // ══════════════════════════════════════════════

    #[test]
    fn reserved_name_is_not_hard_error() {
        let err = TreeError::ReservedName(PathBuf::from("CON"));
        assert!(!err.is_hard_error());
    }

    #[test]
    fn io_error_is_hard_error() {
        let err = TreeError::Io(PathBuf::from("/x"), std::io::Error::other("fail"));
        assert!(err.is_hard_error());
    }

    #[test]
    fn not_found_is_hard_error() {
        assert!(TreeError::NotFound(PathBuf::from("/x")).is_hard_error());
    }

    #[test]
    fn generic_is_hard_error() {
        assert!(TreeError::Generic("boom".into()).is_hard_error());
    }

    #[test]
    fn max_depth_is_hard_error() {
        assert!(TreeError::MaxDepthExceeded(PathBuf::from("/deep")).is_hard_error());
    }

    // ══════════════════════════════════════════════
    // report_errors
    // ══════════════════════════════════════════════

    #[test]
    fn report_errors_empty() {
        assert_eq!(report_errors(&[]), 0);
    }

    #[test]
    fn report_errors_counts_hard_only() {
        let errors = vec![
            TreeError::Io(PathBuf::from("/a"), std::io::Error::other("x")),
            TreeError::ReservedName(PathBuf::from("CON")),
            TreeError::NotFound(PathBuf::from("/b")),
        ];
        assert_eq!(report_errors(&errors), 2);
    }

    #[test]
    fn report_errors_all_reserved() {
        let errors = vec![
            TreeError::ReservedName(PathBuf::from("CON")),
            TreeError::ReservedName(PathBuf::from("NUL")),
        ];
        assert_eq!(report_errors(&errors), 0);
    }

    // ══════════════════════════════════════════════
    // from_io mapping
    // ══════════════════════════════════════════════

    #[test]
    fn from_io_permission_denied_maps_to_access_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = TreeError::from_io(PathBuf::from("/secret"), io_err);
        assert!(matches!(err, TreeError::AccessDenied(_)));
        assert!(err.to_string().contains("/secret"));
    }

    #[test]
    fn from_io_not_found_stays_as_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err = TreeError::from_io(PathBuf::from("/missing"), io_err);
        assert!(matches!(err, TreeError::Io(_, _)));
    }

    #[test]
    fn from_io_other_stays_as_io() {
        let io_err = std::io::Error::other("something else");
        let err = TreeError::from_io(PathBuf::from("/x"), io_err);
        assert!(matches!(err, TreeError::Io(_, _)));
    }

    #[test]
    fn from_io_access_denied_is_hard_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = TreeError::from_io(PathBuf::from("/x"), io_err);
        assert!(err.is_hard_error());
    }

    #[test]
    fn is_path_too_long_detection() {
        // Synthetic OS error with platform-specific code
        #[cfg(windows)]
        let io_err = std::io::Error::from_raw_os_error(206);
        #[cfg(not(windows))]
        let io_err = std::io::Error::from_raw_os_error(36);

        let err = TreeError::from_io(PathBuf::from("/very/long/path"), io_err);
        assert!(matches!(err, TreeError::PathTooLong(_)));
    }

    #[test]
    fn symlink_loop_is_not_hard_error() {
        let err = TreeError::SymlinkLoop(PathBuf::from("/loop/link"));
        assert!(!err.is_hard_error());
    }

    #[test]
    fn report_errors_symlink_loop_not_counted() {
        let errors = vec![
            TreeError::SymlinkLoop(PathBuf::from("/a")),
            TreeError::Io(PathBuf::from("/b"), std::io::Error::other("x")),
            TreeError::ReservedName(PathBuf::from("CON")),
        ];
        assert_eq!(report_errors(&errors), 1); // only Io
    }
}
