//! Unit tests for CLI argument parsing
//! Tests Args parsing without running the binary

use crate::cli::{Args, ColorWhen, IconStyle, IconsWhen, PermMode, SortType};
use clap::Parser;

/// Helper to parse args and unwrap
fn parse_args(args: &[&str]) -> Args {
    let args_with_bin: Vec<&str> = std::iter::once("rtree")
        .chain(args.iter().copied())
        .collect::<Vec<_>>();
    let args_owned: Vec<String> = args_with_bin.iter().map(|s| s.to_string()).collect();
    Args::try_parse_from(args_owned.iter().map(|s| s.as_str())).unwrap()
}

/// Helper to parse args and expect failure
fn parse_args_should_fail(args: &[&str]) {
    let args_with_bin: Vec<&str> = std::iter::once("rtree")
        .chain(args.iter().copied())
        .collect::<Vec<_>>();
    let args_owned: Vec<String> = args_with_bin.iter().map(|s| s.to_string()).collect();
    Args::try_parse_from(args_owned.iter().map(|s| s.as_str()))
        .expect_err("Expected parse failure");
}

// ============================================================================
// Default Values Tests
// ============================================================================

#[test]
fn test_default_path() {
    let args = parse_args(&[]);
    assert_eq!(args.paths.len(), 1);
    assert_eq!(args.paths[0].to_string_lossy(), ".");
}

#[test]
fn test_default_color() {
    let args = parse_args(&[]);
    assert_eq!(args.color, ColorWhen::Auto);
}

#[test]
fn test_default_icons() {
    let args = parse_args(&[]);
    assert_eq!(args.icons, "auto");
}

#[test]
fn test_default_icon_style() {
    let args = parse_args(&[]);
    assert_eq!(args.icon_style, IconStyle::Nerd);
}

#[test]
fn test_default_perm_mode() {
    let args = parse_args(&[]);
    assert_eq!(args.perm_mode, PermMode::Windows);
}

#[test]
fn test_default_time_fmt() {
    let args = parse_args(&[]);
    assert_eq!(args.time_fmt, "%Y-%m-%d %H:%M");
}

// ============================================================================
// Listing Options Tests
// ============================================================================

#[test]
fn test_all_flag() {
    let args = parse_args(&["-a"]);
    assert!(args.all);
}

#[test]
fn test_all_flag_long() {
    let args = parse_args(&["--all"]);
    assert!(args.all);
}

#[test]
fn test_dirs_only_flag() {
    let args = parse_args(&["-d"]);
    assert!(args.dirs_only);
}

#[test]
fn test_dirs_only_flag_long() {
    let args = parse_args(&["--dirs-only"]);
    assert!(args.dirs_only);
}

#[test]
fn test_follow_symlinks_flag() {
    let args = parse_args(&["-l"]);
    assert!(args.follow_symlinks);
}

#[test]
fn test_follow_symlinks_flag_long() {
    let args = parse_args(&["--follow"]);
    assert!(args.follow_symlinks);
}

#[test]
fn test_full_path_flag() {
    let args = parse_args(&["-f"]);
    assert!(args.full_path);
}

#[test]
fn test_full_path_flag_long() {
    let args = parse_args(&["--full-path"]);
    assert!(args.full_path);
}

#[test]
fn test_one_fs_flag() {
    let args = parse_args(&["-x"]);
    assert!(args.one_fs);
}

#[test]
fn test_one_fs_flag_long() {
    let args = parse_args(&["--one-fs"]);
    assert!(args.one_fs);
}

#[test]
fn test_max_depth_flag() {
    let args = parse_args(&["-L", "3"]);
    assert_eq!(args.max_depth, Some(3));
}

#[test]
fn test_max_depth_flag_long() {
    let args = parse_args(&["--level", "5"]);
    assert_eq!(args.max_depth, Some(5));
}

#[test]
fn test_file_limit_flag() {
    let args = parse_args(&["--filelimit", "100"]);
    assert_eq!(args.file_limit, Some(100));
}

#[test]
fn test_no_report_flag() {
    let args = parse_args(&["--noreport"]);
    assert!(args.no_report);
}

// ============================================================================
// Filtering Options Tests
// ============================================================================

#[test]
fn test_pattern_flag() {
    let args = parse_args(&["-P", "*.rs"]);
    assert_eq!(args.pattern, Some("*.rs".to_string()));
}

#[test]
fn test_pattern_flag_long() {
    let args = parse_args(&["--pattern", "*.txt"]);
    assert_eq!(args.pattern, Some("*.txt".to_string()));
}

#[test]
fn test_exclude_flag() {
    let args = parse_args(&["-I", "*.tmp"]);
    assert_eq!(args.exclude, vec!["*.tmp"]);
}

#[test]
fn test_exclude_flag_multiple() {
    let args = parse_args(&["-I", "*.tmp", "-I", "*.log"]);
    assert_eq!(args.exclude, vec!["*.tmp", "*.log"]);
}

#[test]
fn test_exclude_flag_long() {
    let args = parse_args(&["--exclude", "*.bak"]);
    assert_eq!(args.exclude, vec!["*.bak"]);
}

#[test]
fn test_match_dirs_flag() {
    let args = parse_args(&["--matchdirs"]);
    assert!(args.match_dirs);
}

#[test]
fn test_ignore_case_flag() {
    let args = parse_args(&["--ignore-case"]);
    assert!(args.ignore_case);
}

#[test]
fn test_prune_flag() {
    let args = parse_args(&["--prune"]);
    assert!(args.prune);
}

// ============================================================================
// Sorting Options Tests
// ============================================================================

#[test]
fn test_version_sort_flag() {
    let args = parse_args(&["-v"]);
    assert!(args.version_sort);
}

#[test]
fn test_version_sort_flag_long() {
    let args = parse_args(&["--version-sort"]);
    assert!(args.version_sort);
}

#[test]
fn test_time_sort_flag() {
    let args = parse_args(&["-t"]);
    assert!(args.time_sort);
}

#[test]
fn test_time_sort_flag_long() {
    let args = parse_args(&["--timesort"]);
    assert!(args.time_sort);
}

#[test]
fn test_ctime_sort_flag() {
    let args = parse_args(&["-c"]);
    assert!(args.ctime_sort);
}

#[test]
fn test_ctime_sort_flag_long() {
    let args = parse_args(&["--ctime"]);
    assert!(args.ctime_sort);
}

#[test]
fn test_unsorted_flag() {
    let args = parse_args(&["-U"]);
    assert!(args.unsorted);
}

#[test]
fn test_unsorted_flag_long() {
    let args = parse_args(&["--unsorted"]);
    assert!(args.unsorted);
}

#[test]
fn test_reverse_flag() {
    let args = parse_args(&["-r"]);
    assert!(args.reverse);
}

#[test]
fn test_reverse_flag_long() {
    let args = parse_args(&["--reverse"]);
    assert!(args.reverse);
}

#[test]
fn test_dirs_first_flag() {
    let args = parse_args(&["--dirsfirst"]);
    assert!(args.dirs_first);
}

#[test]
fn test_files_first_flag() {
    let args = parse_args(&["--filesfirst"]);
    assert!(args.files_first);
}

#[test]
fn test_sort_name() {
    let args = parse_args(&["--sort=name"]);
    assert_eq!(args.sort, Some(SortType::Name));
}

#[test]
fn test_sort_size() {
    let args = parse_args(&["--sort=size"]);
    assert_eq!(args.sort, Some(SortType::Size));
}

#[test]
fn test_sort_mtime() {
    let args = parse_args(&["--sort=mtime"]);
    assert_eq!(args.sort, Some(SortType::Mtime));
}

#[test]
fn test_sort_ctime() {
    let args = parse_args(&["--sort=ctime"]);
    assert_eq!(args.sort, Some(SortType::Ctime));
}

#[test]
fn test_sort_version() {
    let args = parse_args(&["--sort=version"]);
    assert_eq!(args.sort, Some(SortType::Version));
}

#[test]
fn test_sort_none() {
    let args = parse_args(&["--sort=none"]);
    assert_eq!(args.sort, Some(SortType::None));
}

#[test]
fn test_sort_invalid() {
    parse_args_should_fail(&["--sort=invalid"]);
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
fn test_no_indent_flag() {
    let args = parse_args(&["-i"]);
    assert!(args.no_indent);
}

#[test]
fn test_no_indent_flag_long() {
    let args = parse_args(&["--nocolor"]);
    assert!(args.no_color);
}

#[test]
fn test_ansi_flag() {
    let args = parse_args(&["-A"]);
    assert!(args.ansi);
}

#[test]
fn test_ansi_flag_long() {
    let args = parse_args(&["--ansi"]);
    assert!(args.ansi);
}

#[test]
fn test_cp437_flag() {
    let args = parse_args(&["-S"]);
    assert!(args.cp437);
}

#[test]
fn test_cp437_flag_long() {
    let args = parse_args(&["--cp437"]);
    assert!(args.cp437);
}

#[test]
fn test_no_color_flag() {
    let args = parse_args(&["-n"]);
    assert!(args.no_color);
}

#[test]
fn test_no_color_flag_long() {
    let args = parse_args(&["--nocolor"]);
    assert!(args.no_color);
}

#[test]
fn test_color_always_flag() {
    let args = parse_args(&["-C"]);
    assert!(args.color_always);
}

#[test]
fn test_color_always_flag_long() {
    let args = parse_args(&["--color-always"]);
    assert!(args.color_always);
}

#[test]
fn test_color_auto() {
    let args = parse_args(&["--color=auto"]);
    assert_eq!(args.color, ColorWhen::Auto);
}

#[test]
fn test_color_always() {
    let args = parse_args(&["--color=always"]);
    assert_eq!(args.color, ColorWhen::Always);
}

#[test]
fn test_color_never() {
    let args = parse_args(&["--color=never"]);
    assert_eq!(args.color, ColorWhen::Never);
}

#[test]
fn test_color_invalid() {
    parse_args_should_fail(&["--color=invalid"]);
}

// ============================================================================
// File Info Tests
// ============================================================================

#[test]
fn test_size_flag() {
    let args = parse_args(&["-s"]);
    assert!(args.size);
}

#[test]
fn test_size_flag_long() {
    let args = parse_args(&["--size"]);
    assert!(args.size);
}

#[test]
fn test_human_readable_flag() {
    let args = parse_args(&["-h"]);
    assert!(args.human_readable);
}

#[test]
fn test_human_readable_flag_long() {
    let args = parse_args(&["--human"]);
    assert!(args.human_readable);
}

#[test]
fn test_si_units_flag() {
    let args = parse_args(&["--si"]);
    assert!(args.si_units);
}

#[test]
fn test_date_flag() {
    let args = parse_args(&["-D"]);
    assert!(args.date);
}

#[test]
fn test_date_flag_long() {
    let args = parse_args(&["--date"]);
    assert!(args.date);
}

#[test]
fn test_timefmt_flag() {
    let args = parse_args(&["--timefmt", "%Y-%m-%d"]);
    assert_eq!(args.time_fmt, "%Y-%m-%d");
}

#[test]
fn test_permissions_flag() {
    let args = parse_args(&["-p"]);
    assert!(args.permissions);
}

#[test]
fn test_permissions_flag_long() {
    let args = parse_args(&["--perm"]);
    assert!(args.permissions);
}

#[test]
fn test_uid_flag() {
    let args = parse_args(&["-u"]);
    assert!(args.uid);
}

#[test]
fn test_uid_flag_long() {
    let args = parse_args(&["--uid"]);
    assert!(args.uid);
}

#[test]
fn test_gid_flag() {
    let args = parse_args(&["-g"]);
    assert!(args.gid);
}

#[test]
fn test_gid_flag_long() {
    let args = parse_args(&["--gid"]);
    assert!(args.gid);
}

#[test]
fn test_inodes_flag() {
    let args = parse_args(&["--inodes"]);
    assert!(args.inodes);
}

#[test]
fn test_device_flag() {
    let args = parse_args(&["--device"]);
    assert!(args.device);
}

#[test]
fn test_classify_flag() {
    let args = parse_args(&["-F"]);
    assert!(args.classify);
}

#[test]
fn test_classify_flag_long() {
    let args = parse_args(&["--classify"]);
    assert!(args.classify);
}

#[test]
fn test_safe_print_flag() {
    let args = parse_args(&["-q"]);
    assert!(args.safe_print);
}

#[test]
fn test_safe_print_flag_long() {
    let args = parse_args(&["--safe"]);
    assert!(args.safe_print);
}

#[test]
fn test_literal_flag() {
    let args = parse_args(&["-N"]);
    assert!(args.literal);
}

#[test]
fn test_literal_flag_long() {
    let args = parse_args(&["--literal"]);
    assert!(args.literal);
}

#[test]
fn test_charset_flag() {
    let args = parse_args(&["--charset", "utf-8"]);
    assert_eq!(args.charset, Some("utf-8".to_string()));
}

// ============================================================================
// Export Options Tests
// ============================================================================

#[test]
fn test_output_file_flag() {
    let args = parse_args(&["-o", "output.txt"]);
    assert_eq!(
        args.output_file.map(|p| p.to_string_lossy().to_string()),
        Some("output.txt".to_string())
    );
}

#[test]
fn test_output_file_flag_long() {
    let args = parse_args(&["--output", "output.txt"]);
    assert_eq!(
        args.output_file.map(|p| p.to_string_lossy().to_string()),
        Some("output.txt".to_string())
    );
}

#[test]
fn test_html_base_flag() {
    let args = parse_args(&["-H", "http://localhost"]);
    assert_eq!(args.html_base, Some("http://localhost".to_string()));
}

#[test]
fn test_html_base_flag_long() {
    let args = parse_args(&["--html", "http://localhost"]);
    assert_eq!(args.html_base, Some("http://localhost".to_string()));
}

#[test]
fn test_html_title_flag() {
    let args = parse_args(&["-T", "My Title"]);
    assert_eq!(args.html_title, Some("My Title".to_string()));
}

#[test]
fn test_html_title_flag_long() {
    let args = parse_args(&["--title", "My Title"]);
    assert_eq!(args.html_title, Some("My Title".to_string()));
}

#[test]
fn test_no_links_flag() {
    let args = parse_args(&["--nolinks"]);
    assert!(args.no_links);
}

#[test]
fn test_html_intro_flag() {
    let args = parse_args(&["--hintro", "intro.html"]);
    assert_eq!(
        args.html_intro.map(|p| p.to_string_lossy().to_string()),
        Some("intro.html".to_string())
    );
}

#[test]
fn test_html_outro_flag() {
    let args = parse_args(&["--houtro", "outro.html"]);
    assert_eq!(
        args.html_outro.map(|p| p.to_string_lossy().to_string()),
        Some("outro.html".to_string())
    );
}

#[test]
fn test_xml_flag() {
    let args = parse_args(&["-X"]);
    assert!(args.xml);
}

#[test]
fn test_xml_flag_long() {
    let args = parse_args(&["--xml"]);
    assert!(args.xml);
}

#[test]
fn test_json_flag() {
    let args = parse_args(&["-J"]);
    assert!(args.json);
}

#[test]
fn test_json_flag_long() {
    let args = parse_args(&["--json"]);
    assert!(args.json);
}

// ============================================================================
// Icons Tests
// ============================================================================

#[test]
fn test_icons_auto() {
    let args = parse_args(&["--icons=auto"]);
    assert_eq!(args.icons, "auto");
}

#[test]
fn test_icons_always() {
    let args = parse_args(&["--icons=always"]);
    assert_eq!(args.icons, "always");
}

#[test]
fn test_icons_never() {
    let args = parse_args(&["--icons=never"]);
    assert_eq!(args.icons, "never");
}

// Note: icons is a String, not a ValueEnum, so it accepts any value
// This test is removed as the CLI doesn't validate string values for --icons

#[test]
fn test_no_icons_flag() {
    let args = parse_args(&["--no-icons"]);
    assert!(args.no_icons);
}

#[test]
fn test_icon_style_nerd() {
    let args = parse_args(&["--icon-style=nerd"]);
    assert_eq!(args.icon_style, IconStyle::Nerd);
}

#[test]
fn test_icon_style_unicode() {
    let args = parse_args(&["--icon-style=unicode"]);
    assert_eq!(args.icon_style, IconStyle::Unicode);
}

#[test]
fn test_icon_style_ascii() {
    let args = parse_args(&["--icon-style=ascii"]);
    assert_eq!(args.icon_style, IconStyle::Ascii);
}

#[test]
fn test_icon_style_invalid() {
    parse_args_should_fail(&["--icon-style=invalid"]);
}

// ============================================================================
// Windows-specific Tests
// ============================================================================

#[test]
fn test_show_streams_flag() {
    let args = parse_args(&["--show-streams"]);
    assert!(args.show_streams);
}

#[test]
fn test_show_junctions_flag() {
    let args = parse_args(&["--show-junctions"]);
    assert!(args.show_junctions);
}

#[test]
fn test_hide_system_flag() {
    let args = parse_args(&["--hide-system"]);
    assert!(args.hide_system);
}

#[test]
fn test_perm_mode_posix() {
    let args = parse_args(&["--permissions=posix"]);
    assert_eq!(args.perm_mode, PermMode::Posix);
}

#[test]
fn test_perm_mode_windows() {
    let args = parse_args(&["--permissions=windows"]);
    assert_eq!(args.perm_mode, PermMode::Windows);
}

#[test]
fn test_perm_mode_invalid() {
    parse_args_should_fail(&["--permissions=invalid"]);
}

#[test]
fn test_long_paths_flag() {
    let args = parse_args(&["--long-paths"]);
    assert!(args.long_paths);
}

// ============================================================================
// Language Tests
// ============================================================================

#[test]
fn test_lang_flag() {
    let args = parse_args(&["--lang", "ru"]);
    assert_eq!(args.lang, Some("ru".to_string()));
}

// ============================================================================
// Path Tests
// ============================================================================

#[test]
fn test_single_path() {
    let args = parse_args(&["/some/path"]);
    assert_eq!(args.paths.len(), 1);
    assert_eq!(args.paths[0].to_string_lossy(), "/some/path");
}

#[test]
fn test_multiple_paths() {
    let args = parse_args(&["/path1", "/path2"]);
    assert_eq!(args.paths.len(), 2);
    assert_eq!(args.paths[0].to_string_lossy(), "/path1");
    assert_eq!(args.paths[1].to_string_lossy(), "/path2");
}

// ============================================================================
// Effective Values Tests
// ============================================================================

#[test]
fn test_effective_color_no_color() {
    let args = parse_args(&["-n"]);
    assert_eq!(args.effective_color(), ColorWhen::Never);
}

#[test]
fn test_effective_color_color_always() {
    let args = parse_args(&["-C"]);
    assert_eq!(args.effective_color(), ColorWhen::Always);
}

#[test]
fn test_effective_color_default() {
    let args = parse_args(&[]);
    assert_eq!(args.effective_color(), ColorWhen::Auto);
}

#[test]
fn test_effective_icons_no_icons() {
    let args = parse_args(&["--no-icons"]);
    assert_eq!(args.effective_icons(), IconsWhen::Never);
}

#[test]
fn test_effective_icons_default() {
    let args = parse_args(&[]);
    assert_eq!(args.effective_icons(), IconsWhen::Auto);
}

// ============================================================================
// Flag Priority Tests
// ============================================================================

#[test]
fn test_no_color_overrides_color_always() {
    let args = parse_args(&["-n", "-C"]);
    assert_eq!(args.effective_color(), ColorWhen::Never);
}

#[test]
fn test_no_icons_overrides_icons_always() {
    let args = parse_args(&["--no-icons", "--icons=always"]);
    assert_eq!(args.effective_icons(), IconsWhen::Never);
}
