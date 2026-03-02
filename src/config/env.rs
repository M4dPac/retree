//! Environment variable reading for configuration.
//!
//! Isolates all ENV-related logic in one place.
//! Priority chain (highest → lowest):
//! 1. CLI arguments
//! 2. Environment variables  ← this module
//! 3. Future: TOML config file
//! 4. Compiled defaults

/// Check if stdout is a terminal.
pub fn is_tty() -> bool {
    crate::platform::is_tty()
}
