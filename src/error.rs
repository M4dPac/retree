use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::i18n::{self, get_message, MessageKey};

#[derive(Error, Debug)]
#[allow(dead_code)]
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

    #[error("{}", fmt_path(MessageKey::ErrInvalidName, .0))]
    InvalidName(PathBuf),

    #[error("{}", fmt_path(MessageKey::ErrReservedName, .0))]
    ReservedName(PathBuf),

    #[error("{}", fmt_path_io(MessageKey::ErrIo, .0, .1))]
    Io(PathBuf, #[source] std::io::Error),

    #[error("{}", fmt_str(MessageKey::ErrInvalidPattern, .0))]
    InvalidPattern(String),

    #[error("{}", fmt_str(MessageKey::ErrConfig, .0))]
    Config(String),

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
    fn config_error_contains_message() {
        let err = TreeError::Config("bad value".into());
        let msg = err.to_string();
        assert!(msg.contains("bad value"), "got: {msg}");
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
}
