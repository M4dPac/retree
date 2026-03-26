//! Configuration structures and enums.
//!
//! Contains the unified `Config` — the single source of truth for all settings.

use std::path::PathBuf;

use crate::cli::PermMode;
use crate::cli::{Args, ColorWhen, IconsWhen};
use crate::core::filter::Filter;
use crate::core::sorter::{SortConfig, SortType};
use crate::error::TreeError;
use crate::style::colors::ColorScheme;
use crate::style::icons::{IconSet, IconStyle};

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
    pub json_pretty: bool,
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

    /// Maximum total entries to display (streaming early termination)
    pub max_entries: Option<usize>,

    /// Streaming mode: traverse and render simultaneously
    pub streaming: bool,
}

impl Config {
    /// Build unified configuration from CLI arguments and environment.
    ///
    /// Priority: CLI args > ENV > (future: TOML) > defaults
    pub fn build(args: Args) -> Result<Self, TreeError> {
        let color_when = if args.no_color {
            ColorWhen::Never
        } else if args.color_always {
            ColorWhen::Always
        } else {
            args.color
        };
        let color_enabled = match color_when {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto => crate::platform::is_tty(),
        };

        let icons_when = if args.no_icons {
            IconsWhen::Never
        } else {
            match args.icons.to_lowercase().as_str() {
                "always" => IconsWhen::Always,
                "never" => IconsWhen::Never,
                _ => IconsWhen::Auto,
            }
        };
        let icons_enabled = match icons_when {
            IconsWhen::Always => true,
            IconsWhen::Never => false,
            IconsWhen::Auto => crate::platform::is_tty() && color_enabled,
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

        // Warn if multiple output formats are specified
        {
            let format_count = [args.json, args.xml, args.html_base.is_some()]
                .iter()
                .filter(|&&x| x)
                .count();
            if format_count > 1 {
                let chosen = if args.json {
                    "JSON"
                } else if args.xml {
                    "XML"
                } else {
                    "HTML"
                };
                eprintln!(
                    "rtree: warning: multiple output formats specified, using {}",
                    chosen
                );
            }
        }

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

        if args.dirs_first && args.files_first {
            eprintln!(
                "rtree: warning: --dirsfirst and --filesfirst are mutually exclusive, using --dirsfirst"
            );
        }

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
            files_first: args.files_first && !args.dirs_first,
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
                crate::platform::is_tty()
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
            max_entries: args.max_entries.filter(|&n| n > 0),
            streaming: args.streaming,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Args, ColorWhen, PermMode};
    use crate::style::icons::IconStyle;
    use std::path::PathBuf;

    /// Construct Args with clap-matching defaults (no Parser::parse required).
    fn default_args() -> Args {
        Args {
            paths: vec![PathBuf::from(".")],
            all: false,
            dirs_only: false,
            follow_symlinks: false,
            full_path: false,
            one_fs: false,
            max_depth: None,
            file_limit: None,
            no_report: false,
            pattern: None,
            exclude: vec![],
            match_dirs: false,
            ignore_case: false,
            prune: false,
            version_sort: false,
            time_sort: false,
            ctime_sort: false,
            unsorted: false,
            reverse: false,
            dirs_first: false,
            files_first: false,
            sort: None,
            no_indent: false,
            ansi: false,
            cp437: false,
            no_color: false,
            color_always: false,
            color: ColorWhen::Auto,
            size: false,
            human_readable: false,
            help: None,
            si_units: false,
            date: false,
            time_fmt: "%Y-%m-%d %H:%M".into(),
            permissions: false,
            uid: false,
            gid: false,
            inodes: false,
            device: false,
            classify: false,
            safe_print: false,
            literal: false,
            charset: None,
            output_file: None,
            html_base: None,
            html_title: None,
            no_links: false,
            html_intro: None,
            html_outro: None,
            xml: false,
            json: false,
            json_pretty: false,
            icons: "auto".into(),
            no_icons: false,
            icon_style: IconStyle::Nerd,
            show_streams: false,
            show_junctions: false,
            hide_system: false,
            perm_mode: PermMode::Windows,
            long_paths: false,
            lang: None,
            parallel: false,
            streaming: false,
            threads: None,
            queue_cap: Some(64),
            max_entries: None,
        }
    }

    // ── Output format ──────────────────────────────────

    #[test]
    fn default_format_is_text() {
        let config = Config::build(default_args()).unwrap();
        assert_eq!(config.output_format, OutputFormat::Text);
    }

    #[test]
    fn json_flag_sets_json_format() {
        let mut args = default_args();
        args.json = true;
        let config = Config::build(args).unwrap();
        assert_eq!(config.output_format, OutputFormat::Json);
    }

    #[test]
    fn xml_flag_sets_xml_format() {
        let mut args = default_args();
        args.xml = true;
        let config = Config::build(args).unwrap();
        assert_eq!(config.output_format, OutputFormat::Xml);
    }

    #[test]
    fn html_base_sets_html_format() {
        let mut args = default_args();
        args.html_base = Some("https://example.com".into());
        let config = Config::build(args).unwrap();
        assert_eq!(config.output_format, OutputFormat::Html);
    }

    #[test]
    fn json_takes_priority_over_xml() {
        let mut args = default_args();
        args.json = true;
        args.xml = true;
        let config = Config::build(args).unwrap();
        assert_eq!(config.output_format, OutputFormat::Json);
    }

    // ── Color ──────────────────────────────────────────

    #[test]
    fn no_color_disables_color() {
        let mut args = default_args();
        args.no_color = true;
        let config = Config::build(args).unwrap();
        assert!(!config.color_enabled);
    }

    #[test]
    fn color_always_enables_color() {
        let mut args = default_args();
        args.color_always = true;
        let config = Config::build(args).unwrap();
        assert!(config.color_enabled);
    }

    #[test]
    fn no_color_overrides_color_always() {
        let mut args = default_args();
        args.no_color = true;
        args.color_always = true;
        let config = Config::build(args).unwrap();
        assert!(!config.color_enabled);
    }

    // ── Icons ──────────────────────────────────────────

    #[test]
    fn no_icons_disables_icons() {
        let mut args = default_args();
        args.no_icons = true;
        args.color_always = true; // ensure color wouldn't block
        let config = Config::build(args).unwrap();
        assert!(!config.icons_enabled);
    }

    #[test]
    fn icons_always_enables_icons() {
        let mut args = default_args();
        args.icons = "always".into();
        let config = Config::build(args).unwrap();
        assert!(config.icons_enabled);
    }

    #[test]
    fn icons_never_disables_icons() {
        let mut args = default_args();
        args.icons = "never".into();
        let config = Config::build(args).unwrap();
        assert!(!config.icons_enabled);
    }

    // ── Size ───────────────────────────────────────────

    #[test]
    fn human_readable_implies_show_size() {
        let mut args = default_args();
        args.human_readable = true;
        let config = Config::build(args).unwrap();
        assert!(config.show_size);
        assert!(config.human_readable);
    }

    #[test]
    fn size_flag_enables_show_size() {
        let mut args = default_args();
        args.size = true;
        let config = Config::build(args).unwrap();
        assert!(config.show_size);
        assert!(!config.human_readable);
    }

    // ── Safe print ─────────────────────────────────────

    #[test]
    fn literal_disables_safe_print() {
        let mut args = default_args();
        args.literal = true;
        args.safe_print = true; // explicit --safe + --literal → literal wins
        let config = Config::build(args).unwrap();
        assert!(!config.safe_print);
        assert!(config.literal);
    }

    // ── Sorting ────────────────────────────────────────

    #[test]
    fn default_sort_is_name() {
        let config = Config::build(default_args()).unwrap();
        assert_eq!(config.sort_config.sort_type, SortType::Name);
    }

    #[test]
    fn unsorted_sets_sort_none() {
        let mut args = default_args();
        args.unsorted = true;
        let config = Config::build(args).unwrap();
        assert_eq!(config.sort_config.sort_type, SortType::None);
    }

    #[test]
    fn version_sort_flag() {
        let mut args = default_args();
        args.version_sort = true;
        let config = Config::build(args).unwrap();
        assert_eq!(config.sort_config.sort_type, SortType::Version);
    }

    #[test]
    fn sort_flag_overrides_individual_flags() {
        let mut args = default_args();
        args.version_sort = true;
        args.sort = Some(SortType::Size);
        let config = Config::build(args).unwrap();
        assert_eq!(config.sort_config.sort_type, SortType::Size);
    }

    #[test]
    fn dirs_first_and_reverse() {
        let mut args = default_args();
        args.dirs_first = true;
        args.reverse = true;
        let config = Config::build(args).unwrap();
        assert!(config.sort_config.dirs_first);
        assert!(config.sort_config.reverse);
    }

    // ── Line style ─────────────────────────────────────

    #[test]
    fn cp437_sets_line_style() {
        let mut args = default_args();
        args.cp437 = true;
        let config = Config::build(args).unwrap();
        assert_eq!(config.line_style, LineStyle::Cp437);
    }

    #[test]
    fn charset_ibm437_sets_cp437() {
        let mut args = default_args();
        args.charset = Some("IBM437".into());
        let config = Config::build(args).unwrap();
        assert_eq!(config.line_style, LineStyle::Cp437);
    }

    #[test]
    fn charset_ascii_sets_ascii() {
        let mut args = default_args();
        args.charset = Some("ASCII".into());
        let config = Config::build(args).unwrap();
        assert_eq!(config.line_style, LineStyle::Ascii);
    }

    #[test]
    fn charset_utf8_sets_ansi() {
        let mut args = default_args();
        args.charset = Some("UTF-8".into());
        let config = Config::build(args).unwrap();
        assert_eq!(config.line_style, LineStyle::Ansi);
    }

    // ── Max entries ────────────────────────────────────

    #[test]
    fn max_entries_zero_becomes_none() {
        let mut args = default_args();
        args.max_entries = Some(0);
        let config = Config::build(args).unwrap();
        assert!(config.max_entries.is_none());
    }

    #[test]
    fn max_entries_positive_preserved() {
        let mut args = default_args();
        args.max_entries = Some(100);
        let config = Config::build(args).unwrap();
        assert_eq!(config.max_entries, Some(100));
    }

    // ── Threads ────────────────────────────────────────

    #[test]
    fn threads_u64_converted_to_usize() {
        let mut args = default_args();
        args.threads = Some(4);
        let config = Config::build(args).unwrap();
        assert_eq!(config.threads, Some(4usize));
    }

    // ── Filter ─────────────────────────────────────────

    #[test]
    fn invalid_pattern_returns_error() {
        let mut args = default_args();
        args.pattern = Some("[invalid".into());
        assert!(Config::build(args).is_err());
    }

    #[test]
    fn valid_pattern_accepted() {
        let mut args = default_args();
        args.pattern = Some("*.rs".into());
        assert!(Config::build(args).is_ok());
    }

    // ── Passthrough fields ─────────────────────────────

    #[test]
    fn bool_flags_passed_through() {
        let mut args = default_args();
        args.dirs_only = true;
        args.follow_symlinks = true;
        args.full_path = true;
        args.prune = true;
        args.classify = true;
        args.parallel = true;
        args.streaming = true;
        let config = Config::build(args).unwrap();
        assert!(config.dirs_only);
        assert!(config.follow_symlinks);
        assert!(config.full_path);
        assert!(config.prune);
        assert!(config.classify);
        assert!(config.parallel);
        assert!(config.streaming);
    }

    #[test]
    fn xml_takes_priority_over_html() {
        let mut args = default_args();
        args.xml = true;
        args.html_base = Some("https://example.com".into());
        let config = Config::build(args).unwrap();
        assert_eq!(config.output_format, OutputFormat::Xml);
    }

    #[test]
    fn json_takes_priority_over_html() {
        let mut args = default_args();
        args.json = true;
        args.html_base = Some("https://example.com".into());
        let config = Config::build(args).unwrap();
        assert_eq!(config.output_format, OutputFormat::Json);
    }

    #[test]
    fn dirs_first_wins_over_files_first() {
        let mut args = default_args();
        args.dirs_first = true;
        args.files_first = true;
        let config = Config::build(args).unwrap();
        assert!(config.sort_config.dirs_first);
        assert!(!config.sort_config.files_first);
    }

    #[test]
    fn files_first_alone_works() {
        let mut args = default_args();
        args.files_first = true;
        let config = Config::build(args).unwrap();
        assert!(!config.sort_config.dirs_first);
        assert!(config.sort_config.files_first);
    }
}
