#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_EA, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::Ioctl::FSCTL_GET_REPARSE_POINT;
use windows_sys::Win32::System::IO::DeviceIoControl;

const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA0000003;
#[allow(dead_code)]
const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000000C;

#[repr(C)]
struct REPARSE_DATA_BUFFER {
    reparse_tag: u32,
    reparse_data_length: u16,
    reserved: u16,
    data: [u8; 16384],
}

pub fn get_junction_target(path: &Path) -> Option<PathBuf> {
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateFileW(
            wide_path.as_ptr(),
            FILE_READ_EA,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut buffer: REPARSE_DATA_BUFFER = std::mem::zeroed();
        let mut bytes_returned: u32 = 0;

        let result = DeviceIoControl(
            handle,
            FSCTL_GET_REPARSE_POINT,
            std::ptr::null(),
            0,
            &mut buffer as *mut _ as *mut _,
            std::mem::size_of::<REPARSE_DATA_BUFFER>() as u32,
            &mut bytes_returned,
            std::ptr::null_mut(),
        );

        CloseHandle(handle);

        if result == 0 {
            return None;
        }

        if buffer.reparse_tag != IO_REPARSE_TAG_MOUNT_POINT {
            return None;
        }

        // Validate that we received enough data to parse the mount point header.
        // bytes_returned includes ReparseTag(4) + ReparseDataLength(2) + Reserved(2) = 8 bytes
        // plus the mount point header: SubstituteNameOffset(2) + SubstituteNameLength(2)
        //                            + PrintNameOffset(2) + PrintNameLength(2) = 8 bytes
        let header_overhead = 8u32; // reparse header before data[]
        let mount_header = 8u32; // 4 x u16 fields in data[]
        if bytes_returned < header_overhead + mount_header {
            return None;
        }
        let valid_data_len = (bytes_returned - header_overhead) as usize;

        let data = &buffer.data;

        let substitute_name_offset = u16::from_le_bytes([data[0], data[1]]) as usize;
        let substitute_name_length = u16::from_le_bytes([data[2], data[3]]) as usize;

        let start = 8 + substitute_name_offset;
        let end = start + substitute_name_length;

        if end > valid_data_len || end > data.len() {
            return None;
        }

        let name_data = &data[start..end];
        let name: Vec<u16> = name_data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        let target = String::from_utf16_lossy(&name);

        // Remove \??\ prefix if present
        let target = target.strip_prefix("\\??\\").unwrap_or(&target);

        Some(PathBuf::from(target))
    }
}

#[allow(dead_code)]
pub fn is_reparse_point(path: &Path) -> bool {
    if let Ok(attrs) = super::attributes::get_file_attributes(path) {
        attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_get_junction_target_regular_dir_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(get_junction_target(dir.path()).is_none());
    }

    #[test]
    fn test_get_junction_target_regular_file_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("file.txt");
        fs::write(&file, b"x").expect("write");
        assert!(get_junction_target(&file).is_none());
    }

    #[test]
    fn test_get_junction_target_nonexistent() {
        assert!(get_junction_target(Path::new(r"C:\__no_such_junction_42__")).is_none());
    }

    #[test]
    fn test_get_junction_target_real_junction() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("target_dir");
        let junction = dir.path().join("my_junction");
        fs::create_dir(&target).expect("mkdir");

        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .output()
            .expect("mklink");
        assert!(status.status.success(), "mklink /J failed: {:?}", status);

        let resolved = get_junction_target(&junction);
        assert!(resolved.is_some(), "must detect junction");

        let resolved = resolved.expect("some");
        let canon_target = fs::canonicalize(&target).expect("canonicalize target");
        let canon_resolved = fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());
        assert_eq!(canon_target, canon_resolved);
    }

    #[test]
    fn test_is_reparse_point_regular_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("plain.txt");
        fs::write(&file, b"x").expect("write");
        assert!(!is_reparse_point(&file));
    }

    #[test]
    fn test_is_reparse_point_regular_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(!is_reparse_point(dir.path()));
    }

    #[test]
    fn test_is_reparse_point_junction() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("tgt");
        let junction = dir.path().join("jnc");
        fs::create_dir(&target).expect("mkdir");

        let out = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .output()
            .expect("mklink");
        assert!(out.status.success());

        assert!(is_reparse_point(&junction), "junction is a reparse point");
    }

    #[test]
    fn test_junction_handle_closed_on_non_junction() {
        // Stress: call many times to ensure handles are properly closed
        let dir = tempfile::tempdir().expect("tempdir");
        for i in 0..100 {
            let sub = dir.path().join(format!("dir_{}", i));
            fs::create_dir(&sub).expect("mkdir");
            assert!(get_junction_target(&sub).is_none());
        }
        // If handles leaked, we'd hit the process handle limit
    }
}
