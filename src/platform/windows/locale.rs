//! Windows locale detection.

#![allow(unsafe_code)]

use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

/// Get the primary language ID of the current Windows user.
pub fn get_user_language_id() -> u16 {
    unsafe { GetUserDefaultUILanguage() }
}
