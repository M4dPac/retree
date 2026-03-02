use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::i18n::{self, get_message, MessageKey};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum TreeError {
    #[error("{}", format_access_denied(.0))]
    AccessDenied(PathBuf),

    #[error("{}", format_not_found(.0))]
    NotFound(PathBuf),

    #[error("{}", format_not_directory(.0))]
    NotDirectory(PathBuf),

    #[error("{}", format_symlink_loop(.0))]
    SymlinkLoop(PathBuf),

    #[error("{}", format_symlink_error(.0, .1))]
    SymlinkError(PathBuf, std::io::Error),

    #[error("{}", format_path_too_long(.0))]
    PathTooLong(PathBuf),

    #[error("{}", format_invalid_name(.0))]
    InvalidName(PathBuf),

    #[error("{}", format_io_error(.0, .1))]
    Io(PathBuf, #[source] std::io::Error),

    #[error("{}", format_invalid_pattern(.0))]
    InvalidPattern(String),

    #[error("{}", format_config_error(.0))]
    Config(String),

    #[error("{0}")]
    Generic(String),
}

fn format_access_denied(path: &Path) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrAccessDenied);
    template.replace("{}", &path.display().to_string())
}

fn format_not_found(path: &Path) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrNotFound);
    template.replace("{}", &path.display().to_string())
}

fn format_not_directory(path: &Path) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrNotDirectory);
    template.replace("{}", &path.display().to_string())
}

fn format_symlink_loop(path: &Path) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrSymlinkLoop);
    template.replace("{}", &path.display().to_string())
}

fn format_symlink_error(path: &Path, error: &std::io::Error) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrSymlinkError);
    template
        .replacen("{}", &path.display().to_string(), 1)
        .replacen("{}", &error.to_string(), 1)
}

fn format_path_too_long(path: &Path) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrPathTooLong);
    template.replace("{}", &path.display().to_string())
}

fn format_invalid_name(path: &Path) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrInvalidName);
    template.replace("{}", &path.display().to_string())
}

fn format_io_error(path: &Path, error: &std::io::Error) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrIo);
    template
        .replacen("{}", &path.display().to_string(), 1)
        .replacen("{}", &error.to_string(), 1)
}

fn format_invalid_pattern(pattern: &str) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrInvalidPattern);
    template.replace("{}", pattern)
}

fn format_config_error(msg: &str) -> String {
    let template = get_message(i18n::current(), MessageKey::ErrConfig);
    template.replace("{}", msg)
}

impl From<std::io::Error> for TreeError {
    fn from(err: std::io::Error) -> Self {
        TreeError::Generic(err.to_string())
    }
}
