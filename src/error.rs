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
