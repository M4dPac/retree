#![allow(unsafe_code)]

use windows_sys::Win32::System::Console::*;

pub fn enable_ansi() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle as isize != -1 {
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

pub fn is_tty() -> bool {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle as isize == -1 {
            return false;
        }
        let mut mode: u32 = 0;
        GetConsoleMode(handle, &mut mode) != 0
    }
}

#[allow(dead_code)]
pub fn get_console_width() -> Option<u16> {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle as isize == -1 {
            return None;
        }

        let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if GetConsoleScreenBufferInfo(handle, &mut info) != 0 {
            Some((info.srWindow.Right - info.srWindow.Left + 1) as u16)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_ansi_no_panic() {
        // CI has no real console — must not panic
        enable_ansi();
    }

    #[test]
    fn test_enable_ansi_idempotent() {
        // Calling twice must be safe
        enable_ansi();
        enable_ansi();
    }

    #[test]
    fn test_is_tty_returns_bool_no_panic() {
        let result = is_tty();
        // In CI: typically false (piped stdout).  Just verify no panic.
        let _ = result;
    }

    #[test]
    fn test_get_console_width_no_panic() {
        let width = get_console_width();
        if let Some(w) = width {
            assert!(w > 0, "width must be positive if Some");
        }
        // None is acceptable (no console attached)
    }
}
