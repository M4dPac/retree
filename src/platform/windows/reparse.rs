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

        // Parse the mount point data
        // The structure is complex; this is a simplified version
        let data = &buffer.data;

        // Skip the substitute name offset (2 bytes) and length (2 bytes)
        // Skip the print name offset (2 bytes) and length (2 bytes)
        let substitute_name_offset = u16::from_le_bytes([data[0], data[1]]) as usize;
        let substitute_name_length = u16::from_le_bytes([data[2], data[3]]) as usize;

        let start = 8 + substitute_name_offset;
        let end = start + substitute_name_length;

        if end > data.len() {
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
