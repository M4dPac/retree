//! Windows file owner, group, and permission helpers.
//!
//! Uses the Windows Security API:
//! - GetFileSecurityW          → obtain security descriptor
//! - GetSecurityDescriptorOwner/Group → extract SID
//! - LookupAccountSidW        → SID → account name

#![allow(unsafe_code)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

#[allow(dead_code)]
pub struct WinPermissions {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub permissions: String,
}

// ─────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────

/// Convert a `Path` to a null-terminated UTF-16 string for Win32 APIs.
fn to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Call `GetFileSecurityW` twice (size-probe, then fill) and return
/// the raw security-descriptor bytes.
fn get_security_descriptor(path: &Path, info: u32) -> Option<Vec<u8>> {
    use windows_sys::Win32::Security::GetFileSecurityW;

    let wide = to_wide(path);
    let mut needed: u32 = 0;

    unsafe {
        // 1st call — query required buffer size
        GetFileSecurityW(wide.as_ptr(), info, std::ptr::null_mut(), 0, &mut needed);

        if needed == 0 {
            return None;
        }

        // 2nd call — fill the buffer
        let mut buf = vec![0u8; needed as usize];
        let ok = GetFileSecurityW(
            wide.as_ptr(),
            info,
            buf.as_mut_ptr() as *mut _,
            needed,
            &mut needed,
        );

        if ok == 0 {
            None
        } else {
            Some(buf)
        }
    }
}

/// Resolve a SID pointer to a human-readable account name via
/// `LookupAccountSidW`.  Returns `None` when the SID cannot be mapped
/// (deleted account, orphaned SID, etc.).
fn lookup_sid_name(sid: *mut core::ffi::c_void) -> Option<String> {
    use windows_sys::Win32::Security::LookupAccountSidW;

    if sid.is_null() {
        return None;
    }

    unsafe {
        let mut name_len: u32 = 0;
        let mut domain_len: u32 = 0;
        let mut sid_type: i32 = 0;

        // 1st call — get buffer sizes
        LookupAccountSidW(
            std::ptr::null(),
            sid,
            std::ptr::null_mut(),
            &mut name_len,
            std::ptr::null_mut(),
            &mut domain_len,
            &mut sid_type,
        );

        if name_len == 0 {
            return None;
        }

        let mut name_buf = vec![0u16; name_len as usize];
        let mut domain_buf = vec![0u16; domain_len as usize];

        // 2nd call — fill name & domain
        let ok = LookupAccountSidW(
            std::ptr::null(),
            sid,
            name_buf.as_mut_ptr(),
            &mut name_len,
            domain_buf.as_mut_ptr(),
            &mut domain_len,
            &mut sid_type,
        );

        if ok == 0 {
            return None;
        }

        // After success `name_len` = chars written (excluding null terminator)
        Some(String::from_utf16_lossy(&name_buf[..name_len as usize]))
    }
}

// ─────────────────────────────────────────────────────────────
// Public API — called from platform/mod.rs
// ─────────────────────────────────────────────────────────────

/// Get the file owner name (e.g. `"Administrators"`, `"User"`).
pub fn get_file_owner(path: &Path) -> Option<String> {
    use windows_sys::Win32::Security::{GetSecurityDescriptorOwner, OWNER_SECURITY_INFORMATION};

    let mut sd = get_security_descriptor(path, OWNER_SECURITY_INFORMATION)?;

    unsafe {
        let mut owner_sid: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut defaulted: i32 = 0;

        let ok =
            GetSecurityDescriptorOwner(sd.as_mut_ptr() as *mut _, &mut owner_sid, &mut defaulted);

        if ok == 0 || owner_sid.is_null() {
            return None;
        }

        lookup_sid_name(owner_sid)
    }
}

/// Get the primary group name for a file.
pub fn get_file_group(path: &Path) -> Option<String> {
    use windows_sys::Win32::Security::{GetSecurityDescriptorGroup, GROUP_SECURITY_INFORMATION};

    let mut sd = get_security_descriptor(path, GROUP_SECURITY_INFORMATION)?;

    unsafe {
        let mut group_sid: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut defaulted: i32 = 0;

        let ok =
            GetSecurityDescriptorGroup(sd.as_mut_ptr() as *mut _, &mut group_sid, &mut defaulted);

        if ok == 0 || group_sid.is_null() {
            return None;
        }

        lookup_sid_name(group_sid)
    }
}

// ─────────────────────────────────────────────────────────────
// Legacy stubs (kept for future ACL-based permission strings)
// ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_permissions(_path: &Path) -> Option<WinPermissions> {
    // TODO: Full ACL → POSIX-style permission string
    None
}

#[allow(dead_code)]
pub fn format_posix_style(read: bool, write: bool, execute: bool) -> String {
    let mut s = String::with_capacity(3);
    s.push(if read { 'r' } else { '-' });
    s.push(if write { 'w' } else { '-' });
    s.push(if execute { 'x' } else { '-' });
    s
}
