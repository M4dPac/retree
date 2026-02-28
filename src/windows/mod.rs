#[cfg(windows)]
pub mod attributes;
#[cfg(windows)]
pub mod console;
#[cfg(windows)]
pub mod permissions;
#[cfg(windows)]
pub mod reparse;
#[cfg(windows)]
pub mod streams;

#[cfg(not(windows))]
pub mod console {
    pub fn enable_ansi() {}
    pub fn is_tty() -> bool {
        true
    }
}
