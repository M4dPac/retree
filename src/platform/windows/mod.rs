//! Windows-specific implementations.
//!
//! This entire module is behind `#[cfg(windows)]` in `platform/mod.rs`.
//! Individual submodules do NOT need their own `#[cfg]` guards.

pub mod attributes;
pub mod console;
pub mod locale;
pub mod paths;
pub mod permissions;
pub mod reparse;
pub mod streams;
