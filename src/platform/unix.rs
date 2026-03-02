//! Unix/POSIX platform implementations.

pub fn is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
}
