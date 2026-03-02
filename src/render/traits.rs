//! Renderer trait — Strategy pattern for output backends.

use std::io::Write;

use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;

/// Trait for render backends implementing the Strategy pattern.
///
/// Each renderer handles the complete rendering pipeline for one tree:
/// begin → write root → write entries → finalize (with stats report).
pub trait Renderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        ctx: &RenderContext,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError>;
}
