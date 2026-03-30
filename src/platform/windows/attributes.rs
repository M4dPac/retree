#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, GetFileAttributesW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    INVALID_FILE_ATTRIBUTES, OPEN_EXISTING,
};

use crate::platform::FileIdInfo;

pub fn get_file_id(path: &Path) -> Result<FileIdInfo, std::io::Error> {
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateFileW(
            wide_path.as_ptr(),
            0, // No access needed for metadata
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS, // Required for directories
            std::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        let mut info: BY_HANDLE_FILE_INFORMATION = std::mem::zeroed();
        let result = GetFileInformationByHandle(handle, &mut info);
        let err = if result == 0 {
            Some(std::io::Error::last_os_error())
        } else {
            None
        };
        CloseHandle(handle);

        if let Some(e) = err {
            return Err(e);
        }

        let file_id = ((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64);

        Ok(FileIdInfo {
            file_id,
            volume_serial: info.dwVolumeSerialNumber as u64,
            number_of_links: info.nNumberOfLinks,
        })
    }
}

pub fn get_file_attributes(path: &Path) -> Result<u32, std::io::Error> {
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let attrs = GetFileAttributesW(wide_path.as_ptr());
        if attrs == INVALID_FILE_ATTRIBUTES {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(attrs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_file_id_regular_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test.txt");
        fs::write(&file, b"hello").expect("write");

        let info = get_file_id(&file).expect("get_file_id should succeed");
        assert!(info.file_id > 0, "file_id must be non-zero");
        assert!(info.volume_serial > 0, "volume_serial must be non-zero");
        assert!(info.number_of_links >= 1, "at least 1 link");
    }

    #[test]
    fn test_get_file_id_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let info = get_file_id(dir.path()).expect("get_file_id on dir");
        assert!(info.file_id > 0);
    }

    #[test]
    fn test_get_file_id_nonexistent() {
        let result = get_file_id(Path::new(r"C:\__nonexistent_42__\nope.txt"));
        assert!(result.is_err(), "nonexistent path must fail");
    }

    #[test]
    fn test_get_file_id_hardlinks_share_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let original = dir.path().join("original.txt");
        let hardlink = dir.path().join("hardlink.txt");
        fs::write(&original, b"data").expect("write");
        fs::hard_link(&original, &hardlink).expect("hard_link");

        let a = get_file_id(&original).expect("id original");
        let b = get_file_id(&hardlink).expect("id hardlink");

        assert_eq!(a.file_id, b.file_id, "hard links share file_id");
        assert_eq!(a.volume_serial, b.volume_serial);
        assert!(a.number_of_links >= 2);
        assert!(b.number_of_links >= 2);
    }

    #[test]
    fn test_get_file_id_different_files_differ() {
        let dir = tempfile::tempdir().expect("tempdir");
        let f1 = dir.path().join("one.txt");
        let f2 = dir.path().join("two.txt");
        fs::write(&f1, b"1").expect("write");
        fs::write(&f2, b"2").expect("write");

        let a = get_file_id(&f1).expect("id f1");
        let b = get_file_id(&f2).expect("id f2");
        assert_ne!(
            a.file_id, b.file_id,
            "distinct files must have different ids"
        );
    }

    #[test]
    fn test_get_file_attributes_regular_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("regular.txt");
        fs::write(&file, b"content").expect("write");

        let attrs = get_file_attributes(&file).expect("attrs");
        // FILE_ATTRIBUTE_DIRECTORY = 0x10
        assert_eq!(attrs & 0x10, 0, "regular file is not a directory");
    }

    #[test]
    fn test_get_file_attributes_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let attrs = get_file_attributes(dir.path()).expect("attrs");
        assert_ne!(attrs & 0x10, 0, "must have DIRECTORY attribute");
    }

    #[test]
    fn test_get_file_attributes_nonexistent() {
        let result = get_file_attributes(Path::new(r"C:\__nonexistent_42__"));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_id_is_stable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("stable.txt");
        fs::write(&file, b"data").expect("write");

        let a = get_file_id(&file).expect("first call");
        let b = get_file_id(&file).expect("second call");
        assert_eq!(a.file_id, b.file_id, "repeated calls must return same id");
        assert_eq!(a.volume_serial, b.volume_serial);
    }
}
