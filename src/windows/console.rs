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
