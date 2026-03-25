//! Renderer trait — Strategy pattern for output backends.

use std::io::Write;

use crate::config::Config;
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

/// Trait for render backends implementing the Strategy pattern.
///
/// Each renderer handles the complete rendering pipeline for one tree:
/// begin → write root → write entries → finalize (with stats report).
pub trait Renderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        config: &Config,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError>;
}
