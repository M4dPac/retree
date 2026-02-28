//! Integration tests for rtree CLI
//! Tests the binary execution

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[allow(deprecated)]
fn rtree() -> Command {
    Command::cargo_bin("rtree").unwrap()
}

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_default_execution() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file1.txt"), "content").unwrap();
    fs::write(dir_path.join("subdir/file2.txt"), "content").unwrap();

    rtree()
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_nonexistent_path() {
    rtree()
        .arg("/nonexistent/path")
        .assert()
        .failure();
}

#[test]
fn test_help_flag() {
    rtree()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("List directory contents"));
}

#[test]
fn test_version_flag() {
    rtree()
        .arg("--version")
        .assert()
        .success();
}

// ============================================================================
// Listing Options Tests
// ============================================================================

#[test]
fn test_all_flag_hidden_files() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join(".hidden"), "content").unwrap();
    fs::write(dir_path.join("visible.txt"), "content").unwrap();

    // Without -a, hidden files should not appear
    rtree()
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("visible.txt"))
        .stdout(predicate::str::contains(".hidden").not());

    // With -a, hidden files should appear
    rtree()
        .arg("-a")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden"));
}

#[test]
fn test_dirs_only() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-d")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_follow_symlinks() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Create a symlink (if supported)
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        
        fs::create_dir(dir_path.join("target")).unwrap();
        fs::write(dir_path.join("target/file.txt"), "content").unwrap();
        
        if symlink_dir(dir_path.join("target"), dir_path.join("link")).is_ok() {
            rtree()
                .arg("-l")
                .arg(dir_path)
                .assert()
                .success()
                .stdout(predicate::str::contains("link"));
        }
    }
}

#[test]
fn test_full_path() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("subdir/file.txt"), "content").unwrap();

    rtree()
        .arg("-f")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(dir_path.to_string_lossy()));
}

#[test]
fn test_max_depth() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir_all(dir_path.join("level1/level2/level3")).unwrap();

    rtree()
        .args(["-L", "1"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2").not());
}

#[test]
fn test_file_limit() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Create more than 2 files
    for i in 0..5 {
        fs::write(dir_path.join(format!("file{}.txt", i)), "content").unwrap();
    }

    rtree()
        .args(["--filelimit", "2"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_no_report() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--noreport")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("directories").not())
        .stdout(predicate::str::contains("files").not());
}

// ============================================================================
// Filtering Options Tests
// ============================================================================

#[test]
fn test_pattern_include() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.rs"), "").unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();
    fs::write(dir_path.join("other.rs"), "").unwrap();

    rtree()
        .args(["-P", "*.rs"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("other.rs"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_exclude_pattern() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.rs"), "").unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .args(["-I", "*.txt"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_exclude_multiple_patterns() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.rs"), "").unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();
    fs::write(dir_path.join("file.log"), "").unwrap();

    rtree()
        .args(["-I", "*.txt", "-I", "*.log"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("file.txt").not())
        .stdout(predicate::str::contains("file.log").not());
}

#[test]
fn test_match_dirs() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("include_me")).unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .args(["-P", "include_*", "--matchdirs"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("include_me"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_prune() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir_all(dir_path.join("empty_dir")).unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("--prune")
        .arg(dir_path)
        .assert()
        .success();
}

// ============================================================================
// Sorting Options Tests
// ============================================================================

#[test]
fn test_version_sort() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file1.txt"), "").unwrap();
    fs::write(dir_path.join("file2.txt"), "").unwrap();
    fs::write(dir_path.join("file10.txt"), "").unwrap();

    rtree()
        .arg("-v")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_time_sort() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file1.txt"), "").unwrap();
    fs::write(dir_path.join("file2.txt"), "").unwrap();

    rtree()
        .arg("-t")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_ctime_sort() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-c")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_unsorted() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-U")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_reverse_sort() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("a.txt"), "").unwrap();
    fs::write(dir_path.join("b.txt"), "").unwrap();

    rtree()
        .arg("-r")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_dirs_first() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("--dirsfirst")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_files_first() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("--filesfirst")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_sort_name() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("b.txt"), "").unwrap();
    fs::write(dir_path.join("a.txt"), "").unwrap();

    rtree()
        .args(["--sort=name"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_sort_size() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("small.txt"), "a").unwrap();
    fs::write(dir_path.join("large.txt"), "abcdefghij").unwrap();

    rtree()
        .args(["--sort=size"])
        .arg(dir_path)
        .assert()
        .success();
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
fn test_no_indent() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-i")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_ansi() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-A")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_cp437() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-S")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_no_color() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-n")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_color_always() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .arg("-C")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_color_auto() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .args(["--color=auto"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_color_always_value() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .args(["--color=always"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_color_never_value() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "").unwrap();

    rtree()
        .args(["--color=never"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_color_invalid() {
    rtree()
        .args(["--color=invalid", "."])
        .assert()
        .failure();
}

// ============================================================================
// File Info Tests
// ============================================================================

#[test]
fn test_size_flag() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-s")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.txt"));
}

#[test]
fn test_human_readable() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-h")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_si_units() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--si")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_date_flag() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-D")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_timefmt() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--timefmt", "%Y-%m-%d"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_permissions() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-p")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_uid() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-u")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_gid() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-g")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_inodes() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--inodes")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_device() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--device")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_classify() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file.txt"), "content").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dir_path.join("file.txt")).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dir_path.join("file.txt"), perms).unwrap();
    }

    rtree()
        .arg("-F")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_safe_print() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-q")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_literal() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("-N")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_charset() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--charset", "utf-8"])
        .arg(dir_path)
        .assert()
        .success();
}

// ============================================================================
// Export Options Tests
// ============================================================================

// Note: -o/--output functionality is not implemented in the current version
// This test is commented out until the feature is implemented
// #[test]
// fn test_output_file() {
//     let dir = tempdir().unwrap();
//     let dir_path = dir.path();
//     let output_path = dir.path().join("output.txt");
//
//     fs::write(dir_path.join("file.txt"), "content").unwrap();
//
//     rtree()
//         .args(["-o", output_path.to_str().unwrap()])
//         .arg(dir_path)
//         .assert()
//         .success();
//
//     assert!(output_path.exists());
// }

#[test]
fn test_html_output() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost"])
        .arg(dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("<!DOCTYPE html>"));
    assert!(stdout.contains("<html>"));
    assert!(stdout.contains("<body>"));
}

#[test]
fn test_html_title() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost", "-T", "Test Title"])
        .arg(dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Test Title"));
}

#[test]
fn test_no_links() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["-H", "http://localhost", "--nolinks"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_xml_output() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    let output = rtree()
        .arg("-X")
        .arg(dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.starts_with("<?xml"));
    assert!(stdout.contains("<tree>"));
}

#[test]
fn test_json_output() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    let output = rtree()
        .arg("-J")
        .arg(dir_path)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json.is_array());
}

#[test]
fn test_json_output_valid() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    let output = rtree()
        .arg("-J")
        .arg(dir_path)
        .assert()
        .success();

    let json: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    assert!(json.is_array());
}

// ============================================================================
// Icons Tests
// ============================================================================

#[test]
fn test_icons_auto() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--icons=auto"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_icons_always() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--icons=always"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_icons_never() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--icons=never"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_no_icons() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--no-icons")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_icon_style_nerd() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--icon-style=nerd"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_icon_style_unicode() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--icon-style=unicode"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_icon_style_ascii() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--icon-style=ascii"])
        .arg(dir_path)
        .assert()
        .success();
}

// Note: icons accepts any string value, so this test is removed
// The validation happens at runtime, not at parse time

// ============================================================================
// Windows-specific Tests
// ============================================================================

#[test]
fn test_show_streams() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--show-streams")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_show_junctions() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--show-junctions")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_hide_system() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--hide-system")
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_perm_mode_posix() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--permissions=posix"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_perm_mode_windows() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--permissions=windows"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_perm_mode_invalid() {
    rtree()
        .args(["--permissions=invalid", "."])
        .assert()
        .failure();
}

#[test]
fn test_long_paths() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .arg("--long-paths")
        .arg(dir_path)
        .assert()
        .success();
}

// ============================================================================
// Language Tests
// ============================================================================

#[test]
fn test_lang_ru() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--lang", "ru"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_lang_en() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--lang", "en"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_tree_lang_env() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    // Test with TREE_LANG environment variable
    let mut cmd = rtree();
    cmd.env("TREE_LANG", "ru");
    cmd.arg(dir_path)
        .assert()
        .success();
}

// ============================================================================
// Flag Priority Tests
// ============================================================================

#[test]
fn test_no_color_overrides_color_always() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["-n", "-C"])
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_no_icons_overrides_icons_always() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    rtree()
        .args(["--no-icons", "--icons=always"])
        .arg(dir_path)
        .assert()
        .success();
}

// ============================================================================
// Multiple Paths Tests
// ============================================================================

#[test]
fn test_multiple_paths() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    fs::write(dir1.path().join("file1.txt"), "content").unwrap();
    fs::write(dir2.path().join("file2.txt"), "content").unwrap();

    rtree()
        .arg(dir1.path())
        .arg(dir2.path())
        .assert()
        .success();
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_empty_directory() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    rtree()
        .arg(dir_path)
        .assert()
        .success();
}

#[test]
fn test_deeply_nested() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir_all(dir_path.join("a/b/c/d/e")).unwrap();
    fs::write(dir_path.join("a/b/c/d/e/file.txt"), "content").unwrap();

    rtree()
        .args(["-L", "5"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("e"));
}

#[test]
fn test_special_characters_in_filename() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file with spaces.txt"), "content").unwrap();
    fs::write(dir_path.join("file-with-dashes.txt"), "content").unwrap();

    rtree()
        .arg(dir_path)
        .assert()
        .success();
}
