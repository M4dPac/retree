//! Unix/POSIX platform implementations.

use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::platform::FileIdInfo;

pub fn is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
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
        volume_serial: metadata.dev() as u32,
        number_of_links: metadata.nlink() as u32,
    }
}

/// Get POSIX file mode (permissions bits)
pub fn get_file_mode(path: &Path) -> Option<u32> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    Some(metadata.mode())
}

/// Get file owner UID as string
pub fn get_file_owner(path: &Path) -> Option<String> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    Some(metadata.uid().to_string())
}

/// Get file group GID as string
pub fn get_file_group(path: &Path) -> Option<String> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    Some(metadata.gid().to_string())
}
