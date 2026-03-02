//! Configuration structures and enums.
//!
//! Contains the unified `Config` — the single source of truth for all settings.

use std::path::PathBuf;

use crate::cli::{IconStyle, PermMode};
use crate::core::filter::Filter;
use crate::core::sorter::SortConfig;
use crate::style::colors::ColorScheme;
use crate::style::icons::IconSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Html,
    Xml,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LineStyle {
    Ansi,
    Cp437,
    Ascii,
}

/// Unified application configuration.
///
/// Built from CLI arguments + environment variables.
/// Future: will also incorporate TOML config file values.
#[derive(Debug, Clone)]
pub struct Config {
    pub paths: Vec<PathBuf>,

    // Listing
    pub show_all: bool,
    pub dirs_only: bool,
    pub follow_symlinks: bool,
    pub full_path: bool,
    pub one_fs: bool,
    pub max_depth: Option<usize>,
    pub file_limit: Option<usize>,
    pub no_report: bool,

    // Filtering
    pub filter: Filter,
    pub prune: bool,

    // Sorting
    pub sort_config: SortConfig,

    // Display
    pub no_indent: bool,
    pub line_style: LineStyle,
    pub color_enabled: bool,
    pub icons_enabled: bool,
    pub icon_style: IconStyle,

    // File info
    pub show_size: bool,
    pub human_readable: bool,
    pub si_units: bool,
    pub show_date: bool,
    pub time_fmt: String,
    pub show_permissions: bool,
    pub show_owner: bool,
    pub show_group: bool,
    pub show_inodes: bool,
    pub show_device: bool,
    pub classify: bool,
    pub safe_print: bool,
    pub literal: bool,
    pub perm_mode: PermMode,

    // Output
    pub output_format: OutputFormat,
    pub output_file: Option<PathBuf>,
    pub html_base: Option<String>,
    pub html_title: Option<String>,
    pub html_intro: Option<PathBuf>,
    pub html_outro: Option<PathBuf>,
    pub no_links: bool,

    // Windows
    pub show_streams: bool,
    pub show_junctions: bool,
    pub hide_system: bool,
    pub long_paths: bool,

    // Style
    pub color_scheme: ColorScheme,
    pub icon_set: IconSet,

    // Parallel execution
    pub parallel: bool,
    pub threads: Option<usize>,
    pub queue_cap: Option<usize>,
}
