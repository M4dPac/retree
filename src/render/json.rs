use std::io::Write;

use serde::Serialize;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::TreeStats;
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;
use super::traits::Renderer;

pub struct JsonRenderer {
    entries: Vec<JsonEntry>,
    stack: Vec<usize>,
}

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

#[derive(Serialize)]
#[allow(dead_code)]
struct JsonReport {
    #[serde(rename = "type")]
    entry_type: String,
    directories: u64,
    files: u64,
}

impl JsonRenderer {
    pub fn new(_config: &Config) -> Self {
        JsonRenderer {
            entries: Vec::new(),
            stack: Vec::new(),
        }
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

    fn accumulate_entry(&mut self, entry: &Entry) {
        let target = match &entry.entry_type {
            EntryType::Symlink { target, .. } => Some(target.display().to_string()),
            EntryType::Junction { target } => Some(target.display().to_string()),
            _ => None,
        };

        let time = entry.metadata.as_ref().and_then(|m| m.modified).map(|t| {
            use chrono::{DateTime, Utc};
            let dt: DateTime<Utc> = t.into();
            dt.format("%Y-%m-%dT%H:%M:%S").to_string()
        });

        let json_entry = JsonEntry {
            entry_type: Self::entry_type_str(&entry.entry_type),
            name: entry.name_str().to_string(),
            size: entry.metadata.as_ref().map(|m| m.size),
            time,
            target,
            contents: Vec::new(),
        };

        while self.stack.len() > entry.depth {
            self.stack.pop();
        }

        if let Some(&parent_idx) = self.stack.last() {
            self.entries[parent_idx].contents.push(json_entry);
        } else {
            self.entries.push(json_entry);
            if entry.entry_type.is_directory() {
                self.stack.push(self.entries.len() - 1);
            }
        }
    }
}

impl Renderer for JsonRenderer {
    fn render<W: Write>(
        &mut self,
        result: &BuildResult,
        _ctx: &RenderContext,
        writer: &mut W,
        stats: &mut TreeStats,
    ) -> Result<(), TreeError> {
        self.entries.clear();
        self.stack.clear();

        // Root entry
        self.accumulate_entry(&result.root);
        if result.root.entry_type.is_directory() {
            stats.directories += 1;
        } else {
            stats.files += 1;
        }

        // Child entries
        for entry in &result.entries {
            self.accumulate_entry(entry);
            if entry.entry_type.is_directory() {
                stats.directories += 1;
            } else {
                stats.files += 1;
            }
            if entry.entry_type.is_symlink() {
                stats.symlinks += 1;
            }
        }

        // Serialize and write
        let output = self.entries.clone();
        let json =
            serde_json::to_string_pretty(&output).map_err(|e| TreeError::Generic(e.to_string()))?;

        writeln!(writer, "{}", json)?;

        Ok(())
    }
}
