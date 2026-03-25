//! Configuration layer.
//!
//! Unified configuration built from multiple sources with priority:
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. Future: TOML config file
//! 4. Compiled defaults (lowest priority)

mod options;

pub use options::{Config, LineStyle, OutputFormat};
