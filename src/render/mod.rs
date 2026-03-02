//! Render layer — output backends using the Strategy pattern.
//!
//! Provides `dispatch()` as the single entry point for rendering.

mod html;
mod json;
mod text;
mod xml;

pub mod context;
pub mod traits;

pub use context::RenderContext;
pub use html::HtmlRenderer;
pub use json::JsonRenderer;
pub use text::TextRenderer;
pub use traits::Renderer;
pub use xml::XmlRenderer;

use std::io::Write;

use crate::config::{Config, OutputFormat};
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

/// Dispatch rendering to the appropriate backend based on configuration.
///
/// Creates the appropriate renderer based on `config.output_format`
/// and delegates rendering to it via the `Renderer` trait.
pub fn dispatch<W: Write>(
    result: &BuildResult,
    config: &Config,
    writer: &mut W,
    stats: &mut TreeStats,
) -> Result<(), TreeError> {
    let ctx = RenderContext::new(config);

    match config.output_format {
        OutputFormat::Text => {
            let mut renderer = TextRenderer::new(config);
            renderer.render(result, &ctx, writer, stats)
        }
        OutputFormat::Html => {
            let mut renderer = HtmlRenderer::new(config);
            renderer.render(result, &ctx, writer, stats)
        }
        OutputFormat::Xml => {
            let mut renderer = XmlRenderer::new(config);
            renderer.render(result, &ctx, writer, stats)
        }
        OutputFormat::Json => {
            let mut renderer = JsonRenderer::new(config);
            renderer.render(result, &ctx, writer, stats)
        }
    }
}
