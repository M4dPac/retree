//! rtree - GNU tree compatible directory listing utility.
//!
//! This crate provides both a library API and CLI for displaying
//! directory structures in a tree-like format.

pub mod app;
pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod i18n;
pub mod render;
pub mod style;

#[cfg(windows)]
pub mod windows;

// Re-export main entry point for convenience
pub use app::run;
