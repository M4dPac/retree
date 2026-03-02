//! Rendering context containing display settings.

use crate::config::Config;

/// Rendering context — clean boundary between app config and render layer.
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub config: &'a Config,
}

impl<'a> RenderContext<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}
