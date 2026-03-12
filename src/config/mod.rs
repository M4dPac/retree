//! Configuration layer.
//!
//! Unified configuration built from multiple sources with priority:
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. Future: TOML config file
//! 4. Compiled defaults (lowest priority)

pub mod env;
mod options;

pub use options::{Config, LineStyle, OutputFormat};

use crate::cli::{Args, ColorWhen, IconsWhen, SortType};
use crate::core::filter::Filter;
use crate::core::sorter::SortConfig;
use crate::error::TreeError;
use crate::style::colors::ColorScheme;
use crate::style::icons::IconSet;

impl Config {
    /// Build unified configuration from CLI arguments and environment.
    ///
    /// Priority: CLI args > ENV > (future: TOML) > defaults
    pub fn build(args: Args) -> Result<Self, TreeError> {
        let color_enabled = match args.effective_color() {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto => env::is_tty(),
        };

        let icons_enabled = match args.effective_icons() {
            IconsWhen::Always => true,
            IconsWhen::Never => false,
            IconsWhen::Auto => env::is_tty() && color_enabled,
        };

        let line_style = if args.cp437 {
            LineStyle::Cp437
        } else if let Some(ref charset) = args.charset {
            match charset.to_uppercase().as_str() {
                "IBM437" | "CP437" | "437" => LineStyle::Cp437,
                "ASCII" | "US-ASCII" => LineStyle::Ascii,
                // UTF-8 and anything else → default Ansi (Unicode box-drawing)
                _ => LineStyle::Ansi,
            }
        } else {
            LineStyle::Ansi
        };

        let output_format = if args.json {
            OutputFormat::Json
        } else if args.xml {
            OutputFormat::Xml
        } else if args.html_base.is_some() {
            OutputFormat::Html
        } else {
            OutputFormat::Text
        };

        let filter = Filter::new(
            args.pattern.as_deref(),
            &args.exclude,
            args.match_dirs,
            args.ignore_case,
        )?;

        let sort_config = SortConfig {
            sort_type: args.sort.unwrap_or({
                if args.unsorted {
                    SortType::None
                } else if args.version_sort {
                    SortType::Version
                } else if args.time_sort {
                    SortType::Mtime
                } else if args.ctime_sort {
                    SortType::Ctime
                } else {
                    SortType::Name
                }
            }),
            reverse: args.reverse,
            dirs_first: args.dirs_first,
            files_first: args.files_first,
        };

        // Future: load TOML config and merge here
        // let file_config = toml::load_config()?;
        // Values from CLI args override file_config override defaults.

        let color_scheme = ColorScheme::load();
        let icon_set = IconSet::new(args.icon_style);

        Ok(Config {
            paths: args.paths,

            show_all: args.all,
            dirs_only: args.dirs_only,
            follow_symlinks: args.follow_symlinks,
            full_path: args.full_path,
            one_fs: args.one_fs,
            max_depth: args.max_depth,
            file_limit: args.file_limit,
            no_report: args.no_report,

            filter,
            prune: args.prune,

            sort_config,

            no_indent: args.no_indent,
            line_style,
            color_enabled,
            icons_enabled,
            icon_style: args.icon_style,

            show_size: args.size || args.human_readable,
            human_readable: args.human_readable,
            si_units: args.si_units,
            show_date: args.date,
            time_fmt: args.time_fmt,
            show_permissions: args.permissions,
            show_owner: args.uid,
            show_group: args.gid,
            show_inodes: args.inodes,
            show_device: args.device,
            classify: args.classify,
            safe_print: if args.literal {
                false
            } else if args.safe_print {
                true
            } else {
                env::is_tty()
            },
            literal: args.literal,
            perm_mode: args.perm_mode,

            output_format,
            output_file: args.output_file,
            json_pretty: args.json_pretty,
            html_base: args.html_base,
            html_title: args.html_title,
            html_intro: args.html_intro,
            html_outro: args.html_outro,
            no_links: args.no_links,

            show_streams: args.show_streams,
            show_junctions: args.show_junctions,
            hide_system: args.hide_system,
            long_paths: args.long_paths,

            color_scheme,
            icon_set,

            parallel: args.parallel,
            threads: args.threads.map(|n| n as usize),
            queue_cap: args.queue_cap.map(|n| n as usize),
            max_entries: args.max_entries,
        })
    }

    /// Backward-compatible alias for `build()`.
    pub fn from_args(args: Args) -> Result<Self, TreeError> {
        Self::build(args)
    }
}
