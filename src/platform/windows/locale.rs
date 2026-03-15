//! Windows locale detection.

#![allow(unsafe_code)]

use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

/// Get the primary language ID of the current Windows user.
pub fn get_user_language_id() -> u16 {
    unsafe { GetUserDefaultUILanguage() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_language_id_nonzero() {
        let id = get_user_language_id();
        // Windows always has a configured language; 0 = failure.
        assert_ne!(id, 0, "language ID must be non-zero");
    }

    #[test]
    fn test_get_user_language_id_stable() {
        let a = get_user_language_id();
        let b = get_user_language_id();
        assert_eq!(a, b, "language ID must be stable across calls");
    }
}
