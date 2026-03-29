//! Render layer — output backends using the Strategy pattern.
//!
//! Provides `dispatch()` as the single entry point for rendering.

pub mod helpers;

mod html;
mod json;
mod text;
mod xml;

pub mod traits;

use html::HtmlRenderer;
use json::JsonRenderer;
pub use text::TextRenderer;
pub use traits::Renderer;
use xml::XmlRenderer;

use std::io::Write;

use crate::config::{Config, OutputFormat};
use crate::core::entry::Entry;
use crate::core::tree::Tree;
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

/// Mutable state for tree-based rendering (truncation tracking).
///
/// Shared across text, HTML, and XML renderers.
pub(crate) struct RenderState {
    pub(crate) max_entries: Option<usize>,
    pub(crate) count: usize,
    pub(crate) truncated: bool,
}

/// Generic depth-first tree walker with layout tracking.
///
/// Computes `is_last` and `ancestors_last` on the fly, handles
/// `max_entries` truncation and statistics counting.
/// Used by text and HTML renderers to avoid duplicating traversal logic.
pub(crate) fn walk_tree<F>(
    node: &Tree,
    ancestors_last: &[bool],
    stats: &mut TreeStats,
    state: &mut RenderState,
    emit: &mut F,
) -> Result<(), TreeError>
where
    F: FnMut(&Entry, bool, &[bool]) -> Result<(), TreeError>,
{
    let num_children = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        if state.max_entries.is_some_and(|max| state.count >= max) {
            state.truncated = true;
            return Ok(());
        }
        let is_last = i == num_children - 1;
        emit(&child.entry, is_last, ancestors_last)?;
        helpers::count_stats(&child.entry, stats);
        state.count += 1;
        if !child.children.is_empty() {
            let mut new_ancestors = ancestors_last.to_vec();
            new_ancestors.push(is_last);
            walk_tree(child, &new_ancestors, stats, state, emit)?;
            if state.truncated {
                return Ok(());
            }
        }
    }
    Ok(())
}

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
    match config.output_format {
        OutputFormat::Text => {
            let mut renderer = TextRenderer::new(config);
            renderer.render(result, config, writer, stats)
        }
        OutputFormat::Html => {
            let mut renderer = HtmlRenderer::new(config);
            renderer.render(result, config, writer, stats)
        }
        OutputFormat::Xml => {
            let mut renderer = XmlRenderer::new(config);
            renderer.render(result, config, writer, stats)
        }
        OutputFormat::Json => {
            let mut renderer = JsonRenderer::new(config);
            renderer.render(result, config, writer, stats)
        }
    }
}
