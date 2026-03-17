use std::io::Write;

use serde::Serialize;

use crate::config::Config;
use crate::core::entry::{Entry, EntryType};
use crate::core::walker::{Node, TreeStats};
use crate::core::BuildResult;
use crate::error::TreeError;

use super::context::RenderContext;
use super::helpers;
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
            entry_type: helpers::entry_type_str(&entry.entry_type).to_string(),
            name: entry.name_str().to_string(),
            size,
            time,
            target,
            contents: Vec::new(),
        }
    }

    /// Recursively convert a Node tree into JsonEntry with stats counting.
    fn node_to_json_entry(
        node: &Node,
        config: &Config,
        stats: &mut TreeStats,
        max_entries: Option<usize>,
        count: &mut usize,
    ) -> JsonEntry {
        let mut json_entry = Self::make_json_entry(&node.entry, config);

        for child in &node.children {
            if max_entries.is_some_and(|max| *count >= max) {
                break;
            }
            helpers::count_stats(&child.entry, stats);
            *count += 1;
            json_entry.contents.push(Self::node_to_json_entry(
                child,
                config,
                stats,
                max_entries,
                count,
            ));
        }

        json_entry
    }

    /// Format JSON in tree-compatible style (compact objects with indented nesting)
    fn format_tree_style(output: &[serde_json::Value]) -> Result<String, TreeError> {
        let mut result = String::new();
        result.push_str("[\n");

        for (i, item) in output.iter().enumerate() {
            if let Some(obj) = item.as_object() {
                if obj.get("type").and_then(|v| v.as_str()) == Some("report") {
                    result.push_str(",\n");
                    let compact = serde_json::to_string(item)
                        .map_err(|e| TreeError::Generic(e.to_string()))?;
                    result.push_str(&format!("  {}\n", compact));
                } else {
                    if i > 0 {
                        result.push_str(",\n");
                    }
                    Self::format_entry(&mut result, item, 1)?;
                    result.push('\n');
                }
            }
        }

        result.push(']');
        Ok(result)
    }

    /// Recursively format a single entry in tree-compatible style
    fn format_entry(
        out: &mut String,
        value: &serde_json::Value,
        depth: usize,
    ) -> Result<(), TreeError> {
        let indent = "  ".repeat(depth);

        if let Some(obj) = value.as_object() {
            out.push_str(&indent);
            out.push('{');

            let mut first = true;
            for (key, val) in obj.iter() {
                if key == "contents" {
                    continue;
                }

                if !first {
                    out.push(',');
                }
                first = false;

                let key_json =
                    serde_json::to_string(key).map_err(|e| TreeError::Generic(e.to_string()))?;
                let val_json =
                    serde_json::to_string(val).map_err(|e| TreeError::Generic(e.to_string()))?;
                out.push_str(&format!("{}:{}", key_json, val_json));
            }

            if let Some(contents) = obj.get("contents") {
                if let Some(arr) = contents.as_array() {
                    if !arr.is_empty() {
                        if !first {
                            out.push(',');
                        }
                        out.push_str("\"contents\":[\n");

                        for (i, child) in arr.iter().enumerate() {
                            if i > 0 {
                                out.push_str(",\n");
                            }
                            Self::format_entry(out, child, depth + 1)?;
                        }

                        out.push_str(&format!("\n{}]", indent));
                    }
                }
            }

            out.push('}');
        } else {
            let json =
                serde_json::to_string(value).map_err(|e| TreeError::Generic(e.to_string()))?;
            out.push_str(&indent);
            out.push_str(&json);
        }

        Ok(())
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

        helpers::count_stats(&result.root, stats);

        let root = if let Some(ref tree) = result.tree {
            let mut count = 0usize;
            Self::node_to_json_entry(tree, config, stats, config.max_entries, &mut count)
        } else {
            Self::make_json_entry(&result.root, config)
        };

        let root_value =
            serde_json::to_value(&root).map_err(|e| TreeError::Generic(e.to_string()))?;
        let mut output = vec![root_value];

        if !config.no_report {
            // GNU tree omits "files" key when dirs_only mode
            let report = if config.dirs_only {
                serde_json::json!({
                    "type": "report",
                    "directories": stats.directories.saturating_sub(1)
                })
            } else {
                serde_json::json!({
                    "type": "report",
                    "directories": stats.directories.saturating_sub(1),
                    "files": stats.files
                })
            };
            output.push(report);
        }

        let json_str = if config.json_pretty {
            serde_json::to_string_pretty(&output).map_err(|e| TreeError::Generic(e.to_string()))?
        } else {
            Self::format_tree_style(&output)?
        };

        writeln!(writer, "{}", json_str)?;

        Ok(())
    }
}
