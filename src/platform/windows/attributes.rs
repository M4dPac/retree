#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, GetFileAttributesW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    INVALID_FILE_ATTRIBUTES, OPEN_EXISTING,
};

pub struct FileIdInfo {
    pub file_id: u64,
    pub volume_serial: u64,
    pub number_of_links: u32,
}

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
