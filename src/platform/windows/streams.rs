#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard, WIN32_FIND_STREAM_DATA,
};

#[derive(Debug, Clone)]
pub struct AlternateDataStream {
    pub name: String,
    pub size: u64,
}

/// Enumerate NTFS Alternate Data Streams for the given path.
///
/// Returns only *alternate* streams — the default `::$DATA` is filtered out.
/// On non-NTFS volumes (FAT32, exFAT, network) returns an empty `Vec`
/// without errors (the Win32 call simply fails with `INVALID_HANDLE_VALUE`).
///
/// # Safety
///
/// Uses `FindFirstStreamW` / `FindNextStreamW` — safe Win32 API that does
/// not require elevated privileges.  The returned handle is always closed
/// via `FindClose` (RAII-style guard at the end of the function).
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
            if let Some(info) = parse_stream_data(&find_data) {
                streams.push(info);
            }

            if FindNextStreamW(handle, &mut find_data as *mut _ as *mut _) == 0 {
                break;
            }
        }

        FindClose(handle);
    }

    streams
}

/// Parse a single `WIN32_FIND_STREAM_DATA`, filtering out the default
/// `::$DATA` stream and sanitising the name.
fn parse_stream_data(data: &WIN32_FIND_STREAM_DATA) -> Option<AlternateDataStream> {
    let nul_pos = data
        .cStreamName
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(data.cStreamName.len());

    let raw_name = String::from_utf16_lossy(&data.cStreamName[..nul_pos]);

    // Default data stream — skip
    if raw_name == "::$DATA" {
        return None;
    }

    // Stream names come as ":name:$DATA" — strip surrounding markers
    let clean = raw_name
        .trim_start_matches(':')
        .trim_end_matches(":$DATA")
        .to_string();

    // Guard against empty names after stripping
    if clean.is_empty() {
        return None;
    }

    Some(AlternateDataStream {
        name: clean,
        size: data.StreamSize.max(0) as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_no_ads_on_plain_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("plain.txt");
        fs::write(&file, b"hello").unwrap();

        let streams = get_alternate_streams(&file);
        assert!(streams.is_empty(), "plain file should have no ADS");
    }

    #[test]
    fn test_enumerate_single_ads() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("with_ads.txt");
        fs::write(&file, b"main content").unwrap();

        // Create ADS via NTFS path syntax  file.txt:stream_name
        let ads_path = format!("{}:secret", file.display());
        let mut f = fs::File::create(&ads_path).expect("failed to create ADS (is this NTFS?)");
        f.write_all(b"hidden payload").unwrap();
        drop(f);

        let streams = get_alternate_streams(&file);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].name, "secret");
        assert_eq!(streams[0].size, 14); // b"hidden payload".len()
    }

    #[test]
    fn test_enumerate_multiple_ads() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("multi.txt");
        fs::write(&file, b"body").unwrap();

        for name in &["alpha", "beta", "gamma"] {
            let p = format!("{}:{}", file.display(), name);
            fs::write(&p, format!("data-{}", name)).unwrap();
        }

        let streams = get_alternate_streams(&file);
        assert_eq!(streams.len(), 3);

        let names: Vec<&str> = streams.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert!(names.contains(&"gamma"));
    }

    #[test]
    fn test_nonexistent_path_returns_empty() {
        let streams = get_alternate_streams(Path::new(r"C:\__nonexistent_42__"));
        assert!(streams.is_empty());
    }

    #[test]
    fn test_parse_stream_data_filters_default() {
        let mut data: WIN32_FIND_STREAM_DATA = unsafe { std::mem::zeroed() };
        // "::$DATA" in UTF-16
        let name: Vec<u16> = "::$DATA".encode_utf16().chain(std::iter::once(0)).collect();
        data.cStreamName[..name.len()].copy_from_slice(&name);
        data.StreamSize = 100;

        assert!(parse_stream_data(&data).is_none());
    }

    #[test]
    fn test_parse_stream_data_extracts_name() {
        let mut data: WIN32_FIND_STREAM_DATA = unsafe { std::mem::zeroed() };
        let name: Vec<u16> = ":Zone.Identifier:$DATA"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        data.cStreamName[..name.len()].copy_from_slice(&name);
        data.StreamSize = 26;

        let info = parse_stream_data(&data).unwrap();
        assert_eq!(info.name, "Zone.Identifier");
        assert_eq!(info.size, 26);
    }
}
