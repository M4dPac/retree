use std::io::Write;

use serde::Serialize;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;
use super::traits::Renderer;

pub struct JsonRenderer;

#[derive(Serialize, Clone)]
struct JsonEntry {
    #[serde(rename = "type")]
    entry_type: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    contents: Vec<JsonEntry>,
}

impl JsonRenderer {
    pub fn new(_config: &Config) -> Self {
        JsonRenderer
    }

    fn entry_type_str(entry_type: &EntryType) -> String {
        match entry_type {
            EntryType::File => "file".to_string(),
            EntryType::Directory => "directory".to_string(),
            EntryType::Symlink { .. } => "symlink".to_string(),
            EntryType::Junction { .. } => "junction".to_string(),
            EntryType::HardLink { .. } => "file".to_string(),
            EntryType::Ads { .. } => "stream".to_string(),
            EntryType::Other => "other".to_string(),
        }
    }

    fn make_json_entry(entry: &Entry, config: &Config) -> JsonEntry {
        let target = match &entry.entry_type {
            EntryType::Symlink { target, .. } => Some(target.display().to_string()),
            EntryType::Junction { target } => Some(target.display().to_string()),
            _ => None,
        };

        let time = if config.show_date {
            entry.metadata.as_ref().and_then(|m| m.modified).map(|t| {
                use chrono::{DateTime, Utc};
                let dt: DateTime<Utc> = t.into();
                dt.format("%Y-%m-%dT%H:%M:%S").to_string()
            })
        } else {
            None
        };

        let size = if config.show_size || config.human_readable {
            entry.metadata.as_ref().map(|m| m.size)
        } else {
            None
        };

        JsonEntry {
            entry_type: Self::entry_type_str(&entry.entry_type),
            name: entry.name_str().to_string(),
            size,
            time,
            target,
            contents: Vec::new(),
        }
    }
}

impl Renderer for JsonRenderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        ctx: &RenderContext,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        let config = ctx.config;

        // Count stats for root
        if result.root.entry_type.is_directory() {
            stats.directories += 1;
        } else {
            stats.files += 1;
        }

        // Stack-based tree building from flat entries.
        // Stack represents the current path from root down.
        // stack[0] = root (depth 0), stack[1] = child at depth 1, etc.
        let mut stack: Vec<JsonEntry> = vec![Self::make_json_entry(&result.root, config)];

        for entry in &result.entries {
            // Count stats
            if entry.entry_type.is_directory() {
                stats.directories += 1;
            } else {
                stats.files += 1;
            }
            if entry.entry_type.is_symlink() {
                stats.symlinks += 1;
            }

            // Pop stack until we're at the parent's level.
            // entry.depth tells us how deep this entry is.
            // We want stack to contain exactly `entry.depth` items (the ancestors).
            while stack.len() > entry.depth {
                let child = stack.pop().unwrap();
                stack.last_mut().unwrap().contents.push(child);
            }

            stack.push(Self::make_json_entry(entry, config));
        }

        // Unwind remaining stack into root
        while stack.len() > 1 {
            let child = stack.pop().unwrap();
            stack.last_mut().unwrap().contents.push(child);
        }

        let root = stack.pop().unwrap();

        // Build output array
        let root_value =
            serde_json::to_value(&root).map_err(|e| TreeError::Generic(e.to_string()))?;
        let mut output = vec![root_value];

        // Add report entry unless --noreport
        if !config.no_report {
            output.push(serde_json::json!({
                "type": "report",
                "directories": stats.directories.saturating_sub(1),
                "files": stats.files
            }));
        }

        let json_str =
            serde_json::to_string_pretty(&output).map_err(|e| TreeError::Generic(e.to_string()))?;
        writeln!(writer, "{}", json_str)?;

        Ok(())
    }
}
