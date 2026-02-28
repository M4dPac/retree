mod html;
mod json;
mod text;
mod xml;

pub use html::HtmlFormatter;
pub use json::JsonFormatter;
pub use text::TextFormatter;
pub use xml::XmlFormatter;

use std::io::Write;

use crate::config::Config;
use crate::error::TreeError;
use crate::walker::{TreeEntry, TreeStats};

pub trait TreeOutput {
    fn begin<W: Write>(&mut self, writer: &mut W) -> Result<(), TreeError>;
    fn write_entry<W: Write>(
        &mut self,
        writer: &mut W,
        entry: &TreeEntry,
        config: &Config,
    ) -> Result<(), TreeError>;
    fn end<W: Write>(
        &mut self,
        writer: &mut W,
        stats: &TreeStats,
        config: &Config,
    ) -> Result<(), TreeError>;
}
