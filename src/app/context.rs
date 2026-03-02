//! Application context for future extensibility.
//!
//! Currently, `Config` serves as the primary runtime context.
//! This module provides a wrapper for potential future enhancements
//! such as dependency injection, runtime state, or resource management.

use crate::config::Config;

/// Application execution context.
///
/// Wraps configuration and provides a single point for
/// managing runtime state and dependencies.
#[derive(Debug)]
pub struct AppContext {
    pub config: Config,
}

impl AppContext {
    /// Create a new application context from configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Borrow the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Consume context and return owned configuration.
    pub fn into_config(self) -> Config {
        self.config
    }
}

impl From<Config> for AppContext {
    fn from(config: Config) -> Self {
        Self::new(config)
    }
}
