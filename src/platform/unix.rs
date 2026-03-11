//! Unix/POSIX platform implementations.

use std::fs::Metadata;
use std::io::IsTerminal;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::platform::FileIdInfo;

pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Get file ID info (inode, device, link count) on Unix
pub fn get_file_id(path: &Path) -> Option<FileIdInfo> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    Some(get_file_id_from_metadata(&metadata))
}

/// Extract file ID info from existing metadata
pub fn get_file_id_from_metadata(metadata: &Metadata) -> FileIdInfo {
    FileIdInfo {
        file_id: metadata.ino(),
        volume_serial: metadata.dev(),
        number_of_links: metadata.nlink() as u32,
    }
}

/// Get POSIX file mode (permissions bits)
pub fn get_file_mode(path: &Path) -> Option<u32> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    Some(metadata.mode())
}

/// Get file owner name (or UID if name lookup fails)
pub fn get_file_owner(path: &Path) -> Option<String> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    let uid = metadata.uid();

    // Try to get username, fall back to UID
    match uzers::get_user_by_uid(uid) {
        Some(user) => Some(user.name().to_string_lossy().into_owned()),
        None => Some(uid.to_string()),
    }
}

/// Get file group name (or GID if name lookup fails)
pub fn get_file_group(path: &Path) -> Option<String> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    let gid = metadata.gid();

    // Try to get group name, fall back to GID
    match uzers::get_group_by_gid(gid) {
        Some(group) => Some(group.name().to_string_lossy().into_owned()),
        None => Some(gid.to_string()),
    }
}
