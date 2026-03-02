use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard, WIN32_FIND_STREAM_DATA,
};
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AlternateDataStream {
    pub name: String,
    pub size: u64,
}

#[allow(dead_code)]
pub fn get_alternate_streams(path: &Path) -> Vec<AlternateDataStream> {
    let mut streams = Vec::new();

    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut find_data: WIN32_FIND_STREAM_DATA = std::mem::zeroed();

        let handle = FindFirstStreamW(
            wide_path.as_ptr(),
            FindStreamInfoStandard,
            &mut find_data as *mut _ as *mut _,
            0,
        );

        if handle == INVALID_HANDLE_VALUE {
            return streams;
        }

        loop {
            let name = String::from_utf16_lossy(
                &find_data.cStreamName[..find_data
                    .cStreamName
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(find_data.cStreamName.len())],
            );

            // Skip the default ::$DATA stream
            if name != "::$DATA" {
                streams.push(AlternateDataStream {
                    name: name
                        .trim_start_matches(':')
                        .trim_end_matches(":$DATA")
                        .to_string(),
                    size: find_data.StreamSize as u64,
                });
            }

            if FindNextStreamW(handle, &mut find_data as *mut _ as *mut _) == 0 {
                break;
            }
        }

        FindClose(handle);
    }

    streams
}
