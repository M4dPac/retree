//! Integration tests for rtree CLI
//! These tests define the expected behaviour (specification).
//! The implementation must conform to these tests, not the other way around.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[allow(deprecated)]
fn rtree() -> Command {
    Command::cargo_bin("rtree").unwrap()
}

// ============================================================================
// Helper functions
// ============================================================================

/// Extract file/dir names from text output (skip root line, strip tree chars)
fn extract_names(output: &str) -> Vec<String> {
    output
        .lines()
        .skip(1) // root directory line
        .map(|l| {
            l.replace("├── ", "")
                .replace("└── ", "")
                .replace("│   ", "")
                .replace("    ", "")
                .replace("|-- ", "")
                .replace("`-- ", "")
                .replace("|   ", "")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Recursively collect all entry names from JSON tree structure
fn collect_all_names(json: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            collect_entry_names(item, &mut names);
        }
    }
    names
}

fn collect_entry_names(entry: &serde_json::Value, names: &mut Vec<String>) {
    if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
        names.push(name.to_string());
    }
    if let Some(contents) = entry.get("contents").and_then(|c| c.as_array()) {
        for child in contents {
            collect_entry_names(child, names);
        }
    }
}

/// Count files and directories recursively in JSON structure
fn count_files_and_dirs(json: &serde_json::Value) -> (u64, u64) {
    let mut files = 0u64;
    let mut dirs = 0u64;
    if let Some(arr) = json.as_array() {
        for item in arr {
            count_entry_types(item, &mut files, &mut dirs);
        }
    }
    (files, dirs)
}

fn count_entry_types(entry: &serde_json::Value, files: &mut u64, dirs: &mut u64) {
    if let Some(t) = entry.get("type").and_then(|t| t.as_str()) {
        match t {
            "file" => *files += 1,
            "directory" => *dirs += 1,
            _ => {}
        }
    }
    if let Some(contents) = entry.get("contents").and_then(|c| c.as_array()) {
        for child in contents {
            count_entry_types(child, files, dirs);
        }
    }
}

/// Standard flags to get clean, predictable text output
const CLEAN: &[&str] = &["-n", "--no-icons", "--noreport", "--lang", "en"];

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_default_execution() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file1.txt"), "content").unwrap();
    fs::write(p.join("subdir/file2.txt"), "content").unwrap();

    rtree()
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_nonexistent_path() {
    rtree()
        .arg("/nonexistent/path/that/does/not/exist")
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
        .success()
        .stdout(predicate::str::is_match(r"(?i)rtree\s+\d+\.\d+").unwrap());
}

#[test]
fn test_empty_directory() {
    let dir = tempdir().unwrap();
    rtree().arg(dir.path()).assert().success();
}

#[test]
fn test_multiple_paths() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    fs::write(dir1.path().join("from_dir1.txt"), "").unwrap();
    fs::write(dir2.path().join("from_dir2.txt"), "").unwrap();

    rtree()
        .arg(dir1.path())
        .arg(dir2.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("from_dir1.txt"))
        .stdout(predicate::str::contains("from_dir2.txt"));
}

#[test]
fn test_special_characters_in_filename() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file with spaces.txt"), "").unwrap();
    fs::write(p.join("file-with-dashes.txt"), "").unwrap();

    rtree()
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("file with spaces.txt"))
        .stdout(predicate::str::contains("file-with-dashes.txt"));
}

// ============================================================================
// -a, --all  (show hidden files)
// ============================================================================

#[test]
fn test_all_flag_hides_dotfiles_by_default() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".hidden"), "").unwrap();
    fs::write(p.join("visible.txt"), "").unwrap();

    rtree()
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("visible.txt"))
        .stdout(predicate::str::contains(".hidden").not());
}

#[test]
fn test_all_flag_shows_dotfiles() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".hidden"), "").unwrap();
    fs::write(p.join("visible.txt"), "").unwrap();

    rtree()
        .arg("-a")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden"))
        .stdout(predicate::str::contains("visible.txt"));
}

// ============================================================================
// -d, --dirs-only
// ============================================================================

#[test]
fn test_dirs_only() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();
    fs::write(p.join("subdir/inner.txt"), "").unwrap();

    rtree()
        .arg("-d")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file.txt").not())
        .stdout(predicate::str::contains("inner.txt").not());
}

// ============================================================================
// -l, --follow  (follow symlinks)
// ============================================================================

#[cfg(unix)]
#[test]
fn test_follow_symlinks_enters_target() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("real_dir")).unwrap();
    fs::write(p.join("real_dir/inside.txt"), "content").unwrap();
    std::os::unix::fs::symlink(p.join("real_dir"), p.join("link_dir")).unwrap();

    // With -l the program must descend INTO the symlink and show contents
    rtree()
        .args(["-l"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("link_dir"))
        .stdout(predicate::str::contains("inside.txt"));
}

#[cfg(windows)]
#[test]
fn test_follow_symlinks_windows() {
    use std::os::windows::fs::symlink_dir;

    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("target")).unwrap();
    fs::write(p.join("target/inside.txt"), "content").unwrap();

    if symlink_dir(p.join("target"), p.join("link")).is_ok() {
        rtree()
            .args(["-l"])
            .args(CLEAN)
            .arg(p)
            .assert()
            .success()
            .stdout(predicate::str::contains("link"))
            .stdout(predicate::str::contains("inside.txt"));
    }
}

// ============================================================================
// -f, --full-path
// ============================================================================

#[test]
fn test_full_path_shows_path_prefix() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("subdir/file.txt"), "content").unwrap();

    let output = rtree().args(["-f"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // The file line must contain its path relative to root, not just the basename
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found in output");

    assert!(
        file_line.contains("subdir/file.txt") || file_line.contains("subdir\\file.txt"),
        "With -f, file.txt should show path prefix including parent dir, got: {:?}",
        file_line
    );
}

// ============================================================================
// -x, --one-fs  (stay on one filesystem)
// ============================================================================

#[test]
fn test_one_fs_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-x").arg(dir.path()).assert().success();
}

// ============================================================================
// -L, --level  (max depth)
// ============================================================================

#[test]
fn test_max_depth_limits_traversal() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("level1/level2/level3")).unwrap();
    fs::write(p.join("level1/level2/level3/deep.txt"), "").unwrap();

    rtree()
        .args(["-L", "1"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2").not())
        .stdout(predicate::str::contains("level3").not())
        .stdout(predicate::str::contains("deep.txt").not());
}

#[test]
fn test_max_depth_two() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b/c")).unwrap();

    rtree()
        .args(["-L", "2"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("b"))
        .stdout(predicate::str::contains("c").not());
}

// ============================================================================
// --filelimit  (skip dirs with more than N entries)
// ============================================================================

#[test]
fn test_filelimit_skips_large_dirs() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // big/ has 5 entries — exceeds limit of 2
    fs::create_dir(p.join("big")).unwrap();
    for i in 0..5 {
        fs::write(p.join(format!("big/file{}.txt", i)), "").unwrap();
    }

    // small/ has 1 entry — within limit
    fs::create_dir(p.join("small")).unwrap();
    fs::write(p.join("small/ok.txt"), "").unwrap();

    rtree()
        .args(["--filelimit", "2"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("ok.txt"))
        .stdout(predicate::str::contains("file0.txt").not());
}

// ============================================================================
// --noreport  (omit final statistics line)
// ============================================================================

#[test]
fn test_noreport_omits_statistics() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree()
        .args(["--noreport", "--lang", "en"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Report line matches pattern like "1 directory, 1 file"
    assert!(
        !predicate::str::is_match(r"\d+\s+director")
            .unwrap()
            .eval(&stdout),
        "With --noreport, output should not contain directory count, got:\n{}",
        stdout
    );
    assert!(
        !predicate::str::is_match(r"\d+\s+file")
            .unwrap()
            .eval(&stdout),
        "With --noreport, output should not contain file count, got:\n{}",
        stdout
    );
}

// ============================================================================
// -P, --pattern  (include filter)
// ============================================================================

#[test]
fn test_pattern_include_glob() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.rs"), "").unwrap();
    fs::write(p.join("file.txt"), "").unwrap();
    fs::write(p.join("other.rs"), "").unwrap();

    rtree()
        .args(["-P", "*.rs"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("other.rs"))
        .stdout(predicate::str::contains("file.txt").not());
}

// ============================================================================
// -I, --exclude  (exclude filter, multiple allowed)
// ============================================================================

#[test]
fn test_exclude_pattern() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("keep.rs"), "").unwrap();
    fs::write(p.join("skip.txt"), "").unwrap();

    rtree()
        .args(["-I", "*.txt"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("keep.rs"))
        .stdout(predicate::str::contains("skip.txt").not());
}

#[test]
fn test_exclude_multiple_patterns() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("keep.rs"), "").unwrap();
    fs::write(p.join("skip.txt"), "").unwrap();
    fs::write(p.join("skip.log"), "").unwrap();

    rtree()
        .args(["-I", "*.txt", "-I", "*.log"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("keep.rs"))
        .stdout(predicate::str::contains("skip.txt").not())
        .stdout(predicate::str::contains("skip.log").not());
}

// ============================================================================
// --matchdirs  (apply patterns to directory names too)
// ============================================================================

#[test]
fn test_matchdirs_filters_directories() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("include_me")).unwrap();
    fs::create_dir(p.join("exclude_me")).unwrap();
    fs::write(p.join("include_me/file.txt"), "").unwrap();
    fs::write(p.join("exclude_me/file.txt"), "").unwrap();

    rtree()
        .args(["-P", "include_*", "--matchdirs"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("include_me"))
        .stdout(predicate::str::contains("exclude_me").not());
}

// ============================================================================
// --ignore-case  (case insensitive pattern matching)
// ============================================================================

#[test]
fn test_ignore_case_pattern() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("FILE.RS"), "").unwrap();
    fs::write(p.join("other.txt"), "").unwrap();

    rtree()
        .args(["-P", "*.rs", "--ignore-case"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("FILE.RS"))
        .stdout(predicate::str::contains("other.txt").not());
}

// ============================================================================
// --prune  (omit empty directories)
// ============================================================================

#[test]
fn test_prune_hides_empty_dirs() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("empty_dir")).unwrap();
    fs::create_dir(p.join("nonempty_dir")).unwrap();
    fs::write(p.join("nonempty_dir/file.txt"), "").unwrap();

    rtree()
        .arg("--prune")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("empty_dir").not())
        .stdout(predicate::str::contains("nonempty_dir"))
        .stdout(predicate::str::contains("file.txt"));
}

// ============================================================================
// -v, --version-sort  (natural/version sort)
// ============================================================================

#[test]
fn test_version_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("file2.txt"), "").unwrap();
    fs::write(p.join("file10.txt"), "").unwrap();

    let output = rtree().args(["-v"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos1 = stdout.find("file1.txt").unwrap();
    let pos2 = stdout.find("file2.txt").unwrap();
    let pos10 = stdout.find("file10.txt").unwrap();

    // Natural sort: file1 < file2 < file10  (NOT file1 < file10 < file2)
    assert!(
        pos1 < pos2 && pos2 < pos10,
        "Version sort should order: file1, file2, file10. Positions: {}, {}, {}",
        pos1,
        pos2,
        pos10
    );
}

// ============================================================================
// -t, --timesort  (sort by modification time, newest first)
// ============================================================================

#[test]
fn test_time_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("old.txt"), "old").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(p.join("new.txt"), "new").unwrap();

    let output = rtree().args(["-t"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_new = stdout.find("new.txt").expect("new.txt not found");
    let pos_old = stdout.find("old.txt").expect("old.txt not found");

    assert!(
        pos_new < pos_old,
        "With -t, newest file should appear first. new={}, old={}",
        pos_new,
        pos_old
    );
}

// ============================================================================
// -c, --ctime  (sort by change/creation time)
// ============================================================================

#[test]
fn test_ctime_sort_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-c").arg(dir.path()).assert().success();
}

// ============================================================================
// -U, --unsorted
// ============================================================================

#[test]
fn test_unsorted_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-U").arg(dir.path()).assert().success();
}

// ============================================================================
// -r, --reverse  (reverse sort order)
// ============================================================================

#[test]
fn test_reverse_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("zzz.txt"), "").unwrap();

    let output = rtree().args(["-r"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_z = stdout.find("zzz.txt").unwrap();
    let pos_a = stdout.find("aaa.txt").unwrap();

    assert!(
        pos_z < pos_a,
        "With -r, zzz.txt should appear before aaa.txt. z={}, a={}",
        pos_z,
        pos_a
    );
}

// ============================================================================
// --dirsfirst / --filesfirst
// ============================================================================

#[test]
fn test_dirs_first_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // Name aaa_file alphabetically before subdir
    fs::write(p.join("aaa_file.txt"), "").unwrap();
    fs::create_dir(p.join("subdir")).unwrap();

    let output = rtree()
        .arg("--dirsfirst")
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_dir = stdout.find("subdir").unwrap();
    let pos_file = stdout.find("aaa_file.txt").unwrap();

    assert!(
        pos_dir < pos_file,
        "With --dirsfirst, directory should appear before file. dir={}, file={}",
        pos_dir,
        pos_file
    );
}

#[test]
fn test_files_first_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("aaa_dir")).unwrap();
    fs::write(p.join("zzz_file.txt"), "").unwrap();

    let output = rtree()
        .arg("--filesfirst")
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_file = stdout.find("zzz_file.txt").unwrap();
    let pos_dir = stdout.find("aaa_dir").unwrap();

    assert!(
        pos_file < pos_dir,
        "With --filesfirst, file should appear before directory. file={}, dir={}",
        pos_file,
        pos_dir
    );
}

// ============================================================================
// --sort=<TYPE>
// ============================================================================

#[test]
fn test_sort_name() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("bbb.txt"), "").unwrap();
    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("ccc.txt"), "").unwrap();

    let output = rtree()
        .args(["--sort=name"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_a = stdout.find("aaa.txt").unwrap();
    let pos_b = stdout.find("bbb.txt").unwrap();
    let pos_c = stdout.find("ccc.txt").unwrap();

    assert!(
        pos_a < pos_b && pos_b < pos_c,
        "sort=name: expected aaa < bbb < ccc. a={}, b={}, c={}",
        pos_a,
        pos_b,
        pos_c
    );
}

#[test]
fn test_sort_size() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("small.txt"), "a").unwrap(); // 1 byte
    fs::write(p.join("large.txt"), "a]".repeat(1000)).unwrap(); // 2000 bytes

    let output = rtree()
        .args(["--sort=size"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_small = stdout.find("small.txt").unwrap();
    let pos_large = stdout.find("large.txt").unwrap();

    // Sorted by size: small before large (ascending)
    assert!(
        pos_small < pos_large,
        "sort=size: small.txt should appear before large.txt. small={}, large={}",
        pos_small,
        pos_large
    );
}

#[test]
fn test_sort_mtime() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--sort=mtime"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_sort_ctime() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--sort=ctime"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_sort_version() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("v1.txt"), "").unwrap();
    fs::write(p.join("v2.txt"), "").unwrap();
    fs::write(p.join("v10.txt"), "").unwrap();

    let output = rtree()
        .args(["--sort=version"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let p1 = stdout.find("v1.txt").unwrap();
    let p2 = stdout.find("v2.txt").unwrap();
    let p10 = stdout.find("v10.txt").unwrap();

    assert!(
        p1 < p2 && p2 < p10,
        "sort=version: v1 < v2 < v10. got {}, {}, {}",
        p1,
        p2,
        p10
    );
}

#[test]
fn test_sort_none() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--sort=none"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_sort_invalid_value() {
    rtree().args(["--sort=invalid", "."]).assert().failure();
}

// ============================================================================
// -i, --noindent  (no tree indentation)
// ============================================================================

#[test]
fn test_no_indent_removes_tree_chars() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("subdir/file.txt"), "").unwrap();

    let output = rtree().args(["-i"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    assert!(!stdout.contains('├'), "-i should remove ├");
    assert!(!stdout.contains('└'), "-i should remove └");
    assert!(!stdout.contains('│'), "-i should remove │");
    assert!(
        !stdout.contains("|--") && !stdout.contains("`--"),
        "-i should remove ASCII tree chars"
    );
}

// ============================================================================
// -A, --ansi / -S, --cp437  (line graphics style)
// ============================================================================

#[test]
fn test_ansi_line_graphics() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("file2.txt"), "").unwrap();

    let output = rtree()
        .args(["-A", "-n", "--no-icons", "--lang", "en"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // ANSI line graphics use Unicode box-drawing characters
    let has_unicode_tree = stdout.contains('├') || stdout.contains('└') || stdout.contains('│');
    assert!(
        has_unicode_tree,
        "With -A, output should use Unicode box-drawing chars. Got:\n{}",
        stdout
    );
}

#[test]
fn test_cp437_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-S").arg(dir.path()).assert().success();
}

// ============================================================================
// -n, --nocolor / -C, --color-always / --color=<WHEN>
// ============================================================================

#[test]
fn test_no_color_strips_ansi_escapes() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-n"]).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\x1b["),
        "With -n, output must not contain ANSI escape codes. Got:\n{:?}",
        stdout
    );
}

#[test]
fn test_color_always_forces_ansi_escapes() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-C"]).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("\x1b["),
        "With -C, output should contain ANSI escapes even in pipe. Got:\n{:?}",
        stdout
    );
}

#[test]
fn test_color_never() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    let output = rtree()
        .args(["--color=never"])
        .arg(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\x1b["),
        "With --color=never, no ANSI escapes"
    );
}

#[test]
fn test_color_always_via_flag() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["--color=always"]).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("\x1b["),
        "With --color=always, should have ANSI escapes"
    );
}

#[test]
fn test_color_auto_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--color=auto"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_color_invalid_value() {
    rtree().args(["--color=invalid", "."]).assert().failure();
}

// ============================================================================
// Color priority: -n wins over -C  (effective_color logic in args.rs)
// ============================================================================

#[test]
fn test_no_color_overrides_color_always() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    // -n should win over -C regardless of order
    let output = rtree().args(["-C", "-n"]).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\x1b["),
        "With both -C and -n, -n should win (no ANSI). Got:\n{:?}",
        stdout
    );
}

// ============================================================================
// NO_COLOR env variable  (de-facto standard: https://no-color.org)
// ============================================================================

#[test]
fn test_no_color_env_variable() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().env("NO_COLOR", "1").arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\x1b["),
        "With NO_COLOR=1, output must not contain ANSI escapes"
    );
}

// ============================================================================
// -s, --size  (print size in bytes)
// ============================================================================

#[test]
fn test_size_shows_bytes() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap(); // 7 bytes

    let output = rtree().args(["-s"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        file_line.contains("7"),
        "With -s, file line should show size 7. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -h, --human  (human readable sizes)
// ============================================================================

#[test]
fn test_human_readable_shows_units() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // Create a file > 1KB
    fs::write(p.join("big.txt"), "x".repeat(2048)).unwrap();

    let output = rtree().args(["-h"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("big.txt"))
        .expect("big.txt not found");

    // Should show something like "2.0K" or "2K"
    assert!(
        file_line.contains('K') || file_line.contains('M') || file_line.contains('G'),
        "With -h, large file should show human unit (K/M/G). Got: {:?}",
        file_line
    );
}

// ============================================================================
// --si  (SI units, powers of 1000)
// ============================================================================

#[test]
fn test_si_units_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "x".repeat(2048)).unwrap();

    rtree().arg("--si").arg(dir.path()).assert().success();
}

// ============================================================================
// -D, --date  (print modification date)
// ============================================================================

#[test]
fn test_date_shows_timestamp() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-D"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Default format: %Y-%m-%d %H:%M
    assert!(
        predicate::str::is_match(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}")
            .unwrap()
            .eval(&stdout),
        "With -D, output should contain date in YYYY-MM-DD HH:MM format. Got:\n{}",
        stdout
    );
}

// ============================================================================
// --timefmt  (custom time format)
// ============================================================================

#[test]
fn test_timefmt_custom_format() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-D", "--timefmt", "%Y/%m/%d"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    assert!(
        predicate::str::is_match(r"\d{4}/\d{2}/\d{2}")
            .unwrap()
            .eval(&stdout),
        "Custom timefmt should produce YYYY/MM/DD format. Got:\n{}",
        stdout
    );

    // Should NOT contain HH:MM since we only asked for date
    let file_line = stdout.lines().find(|l| l.contains("file.txt")).unwrap();
    assert!(
        !predicate::str::is_match(r"\d{2}:\d{2}")
            .unwrap()
            .eval(file_line),
        "Custom timefmt %Y/%m/%d should not include time. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -p, --perm  (print permissions)
// ============================================================================

#[test]
fn test_permissions_shows_perm_string() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-p"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    // Should contain permission chars: either posix (rwx) or windows (RW/RA etc)
    let has_perms = file_line.contains("rw")
        || file_line.contains("r-")
        || file_line.contains("RW")
        || file_line.contains("R-")
        || file_line.contains("RA");
    assert!(
        has_perms,
        "With -p, file line should contain permissions. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -u, --uid  (print file owner)
// ============================================================================

#[test]
fn test_uid_shows_owner() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-u"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    // Owner info adds extra content before the filename
    // The line should be longer than just tree_chars + filename
    assert!(
        file_line.len() > "└── file.txt".len(),
        "With -u, file line should include owner info. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -g, --gid  (print file group)
// ============================================================================

#[test]
fn test_gid_shows_group() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-g"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        file_line.len() > "└── file.txt".len(),
        "With -g, file line should include group info. Got: {:?}",
        file_line
    );
}

// ============================================================================
// --inodes  (print inode number)
// ============================================================================

#[test]
fn test_inodes_shows_number() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["--inodes"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    // Inode is a number, typically > 100
    assert!(
        predicate::str::is_match(r"\d{2,}").unwrap().eval(file_line),
        "With --inodes, line should contain inode number. Got: {:?}",
        file_line
    );
}

// ============================================================================
// --device  (print device number)
// ============================================================================

#[test]
fn test_device_shows_number() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["--device"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        predicate::str::is_match(r"\d+").unwrap().eval(file_line),
        "With --device, line should contain device number. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -F, --classify  (append type indicator: / for dirs, * for exec, @ for symlink)
// ============================================================================

#[test]
fn test_classify_appends_slash_to_dirs() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-F"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("subdir/"),
        "With -F, directories should have / suffix. Got:\n{}",
        stdout
    );
}

// ============================================================================
// -q, --safe  (replace non-printable with ?)
// ============================================================================

#[test]
fn test_safe_print_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-q").arg(dir.path()).assert().success();
}

// ============================================================================
// -N, --literal  (print non-printable as-is)
// ============================================================================

#[test]
fn test_literal_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-N").arg(dir.path()).assert().success();
}

// ============================================================================
// --charset
// ============================================================================

#[test]
fn test_charset_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--charset", "utf-8"])
        .arg(dir.path())
        .assert()
        .success();
}

// ============================================================================
// -o, --output  (output to file)
// ============================================================================

#[test]
fn test_output_file_created_with_content() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    let output_path = p.join("output.txt");

    fs::write(p.join("file.txt"), "content").unwrap();

    rtree()
        .args(["-o", output_path.to_str().unwrap()])
        .arg(p)
        .assert()
        .success();

    assert!(output_path.exists(), "Output file should be created");
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(!content.is_empty(), "Output file should not be empty");
    assert!(
        content.contains("file.txt"),
        "Output file should contain tree listing. Got:\n{}",
        content
    );
}

// ============================================================================
// -H, --html  (HTML output)
// ============================================================================

#[test]
fn test_html_output_structure() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("<!DOCTYPE html>") || stdout.contains("<!doctype html>"));
    assert!(stdout.contains("<html"));
    assert!(stdout.contains("<body"));
    assert!(stdout.contains("</html>"));
}

#[test]
fn test_html_contains_links_by_default() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("<a "),
        "HTML output should contain hyperlinks by default. Got:\n{}",
        stdout
    );
}

// ============================================================================
// -T, --title  (HTML page title)
// ============================================================================

#[test]
fn test_html_title() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost", "-T", "My Custom Title"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("My Custom Title"),
        "HTML should contain custom title"
    );
}

// ============================================================================
// --nolinks  (disable hyperlinks in HTML)
// ============================================================================

#[test]
fn test_nolinks_removes_hyperlinks() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost", "--nolinks"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("<a "),
        "With --nolinks, HTML should not contain <a> tags. Got:\n{}",
        stdout
    );
}

// ============================================================================
// --hintro / --houtro  (custom HTML intro/outro files)
// ============================================================================

#[test]
fn test_html_intro_file() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let intro_path = p.join("intro.html");
    fs::write(&intro_path, "<!-- CUSTOM INTRO MARKER -->").unwrap();
    fs::write(p.join("file.txt"), "content").unwrap();

    rtree()
        .args([
            "-H",
            "http://localhost",
            "--hintro",
            intro_path.to_str().unwrap(),
        ])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("CUSTOM INTRO MARKER"));
}

#[test]
fn test_html_outro_file() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let outro_path = p.join("outro.html");
    fs::write(&outro_path, "<!-- CUSTOM OUTRO MARKER -->").unwrap();
    fs::write(p.join("file.txt"), "content").unwrap();

    rtree()
        .args([
            "-H",
            "http://localhost",
            "--houtro",
            outro_path.to_str().unwrap(),
        ])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("CUSTOM OUTRO MARKER"));
}

// ============================================================================
// -X, --xml  (XML output)
// ============================================================================

#[test]
fn test_xml_output_structure() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().arg("-X").arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.starts_with("<?xml"),
        "XML should start with <?xml declaration"
    );
    assert!(stdout.contains("<tree>"), "XML should contain <tree> tag");
    assert!(
        stdout.contains("</tree>"),
        "XML should contain closing </tree>"
    );
    assert!(
        stdout.contains("file.txt"),
        "XML should contain file entries"
    );
}

// ============================================================================
// -J, --json  (JSON output)
// ============================================================================

#[test]
fn test_json_output_structure() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "content").unwrap();
    fs::write(p.join("subdir/nested.txt"), "content").unwrap();

    let output = rtree()
        .args(["-J", "--lang", "en"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Output must be valid JSON");

    // Root is an array
    let arr = json.as_array().expect("Root should be array");
    assert!(arr.len() >= 2, "Should have at least directory + report");

    // First element is root directory
    let root = &arr[0];
    assert_eq!(root["type"].as_str(), Some("directory"));
    assert!(root["name"].as_str().is_some());

    // Root has contents
    let contents = root["contents"]
        .as_array()
        .expect("Root directory should have contents");

    // file.txt at top level
    let file_entry = contents
        .iter()
        .find(|e| e["name"].as_str() == Some("file.txt"))
        .expect("file.txt should be in root contents");
    assert_eq!(file_entry["type"].as_str(), Some("file"));

    // subdir at top level with nested.txt inside
    let subdir_entry = contents
        .iter()
        .find(|e| e["name"].as_str() == Some("subdir"))
        .expect("subdir should be in root contents");
    assert_eq!(subdir_entry["type"].as_str(), Some("directory"));

    let subdir_contents = subdir_entry["contents"]
        .as_array()
        .expect("subdir should have contents");
    assert!(
        subdir_contents
            .iter()
            .any(|e| e["name"].as_str() == Some("nested.txt")),
        "nested.txt should be inside subdir"
    );

    // Last element is report
    let report = arr.last().unwrap();
    assert_eq!(report["type"].as_str(), Some("report"));
    assert!(report["directories"].is_number());
    assert!(report["files"].is_number());
}

#[test]
fn test_json_hierarchy_correct() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b")).unwrap();
    fs::write(p.join("a/b/deep.txt"), "").unwrap();
    fs::write(p.join("root.txt"), "").unwrap();

    let output = rtree().args(["-J"]).arg(p).assert().success();

    let json: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let root_contents = json[0]["contents"].as_array().unwrap();

    // root.txt at top level
    assert!(root_contents
        .iter()
        .any(|e| e["name"].as_str() == Some("root.txt")));

    // deep.txt NOT at top level
    assert!(!root_contents
        .iter()
        .any(|e| e["name"].as_str() == Some("deep.txt")));

    // deep.txt inside a/b/
    let a = root_contents
        .iter()
        .find(|e| e["name"].as_str() == Some("a"))
        .unwrap();
    let b = a["contents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"].as_str() == Some("b"))
        .unwrap();
    assert!(b["contents"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["name"].as_str() == Some("deep.txt")));
}

#[test]
fn test_json_noreport() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-J", "--noreport"]).arg(p).assert().success();

    let json: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let arr = json.as_array().unwrap();

    let has_report = arr.iter().any(|e| e["type"].as_str() == Some("report"));
    assert!(
        !has_report,
        "JSON with --noreport should not contain report entry"
    );
}

#[test]
fn test_json_dirs_only() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-J", "-d"]).arg(p).assert().success();

    let json: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let (files, _dirs) = count_files_and_dirs(&json);

    assert_eq!(
        files, 0,
        "JSON with -d should contain no files, got {}",
        files
    );
}

// ============================================================================
// Icons
// ============================================================================

#[test]
fn test_icons_auto_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icons=auto"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icons_always_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icons=always"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icons_never_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icons=never"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_no_icons_flag() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("--no-icons").arg(dir.path()).assert().success();
}

#[test]
fn test_no_icons_overrides_icons_always() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    // --no-icons should win (effective_icons logic)
    rtree()
        .args(["--no-icons", "--icons=always"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icon_style_nerd() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icon-style=nerd"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icon_style_unicode() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icon-style=unicode"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icon_style_ascii() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icon-style=ascii"])
        .arg(dir.path())
        .assert()
        .success();
}

// ============================================================================
// Windows-specific flags (acceptance tests — verify they don't crash)
// ============================================================================

#[test]
fn test_show_streams_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--show-streams")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_show_junctions_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--show-junctions")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_hide_system_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--hide-system")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_long_paths_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--long-paths")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_permissions_mode_posix() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--permissions=posix"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_permissions_mode_windows() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--permissions=windows"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_permissions_mode_invalid() {
    rtree()
        .args(["--permissions=invalid", "."])
        .assert()
        .failure();
}

// ============================================================================
// Language / i18n
// ============================================================================

#[test]
fn test_report_in_english() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    rtree()
        .args(["--lang", "en"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("director"))
        .stdout(predicate::str::contains("file"));
}

#[test]
fn test_report_in_russian() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    rtree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("каталог"))
        .stdout(predicate::str::contains("файл"));
}

#[test]
fn test_russian_plural_one_file() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("single.txt"), "").unwrap();

    rtree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 файл"));
}

#[test]
fn test_russian_plural_few_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..3 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
    }

    rtree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 файла"));
}

#[test]
fn test_russian_plural_many_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..5 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
    }

    rtree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("5 файлов"));
}

#[test]
fn test_tree_lang_env_switches_language() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    rtree()
        .env("TREE_LANG", "ru")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("каталог"));
}

// ============================================================================
// Tree Indentation / Hierarchy (regression tests)
// ============================================================================

#[test]
fn test_tree_indentation_child_under_dir() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("root_file.txt"), "").unwrap();
    fs::write(p.join("subdir/child_file.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    let child_line = stdout
        .lines()
        .find(|l| l.contains("child_file.txt"))
        .expect("child_file.txt not found");

    // child_file.txt must be indented (deeper than root-level entries)
    assert!(
        child_line.contains("│") || child_line.starts_with("    "),
        "child_file.txt should be indented under subdir. Got: {:?}",
        child_line
    );
}

#[test]
fn test_tree_deep_indentation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b/c")).unwrap();
    fs::write(p.join("a/b/c/deep.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    let deep_line = stdout
        .lines()
        .find(|l| l.contains("deep.txt"))
        .expect("deep.txt not found");

    // deep.txt at depth 4 (a/b/c/deep.txt) needs >= 3 prefix segments
    let name_pos = deep_line.find("deep.txt").unwrap();
    assert!(
        name_pos >= 12,
        "deep.txt should have >= 3 levels of indentation (offset >= 12). Got offset {}: {:?}",
        name_pos,
        deep_line
    );
}

#[test]
fn test_last_branch_marker_single_child() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("only_child.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let child_line = stdout
        .lines()
        .find(|l| l.contains("only_child.txt"))
        .expect("only_child.txt not found");

    assert!(
        child_line.contains("└── ") || child_line.contains("`-- "),
        "Single child should use last-branch marker (└──). Got: {:?}",
        child_line
    );
}

#[test]
fn test_branch_markers_multiple_children() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("zzz.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    let first_line = stdout
        .lines()
        .find(|l| l.contains("aaa.txt"))
        .expect("aaa.txt not found");
    let last_line = stdout
        .lines()
        .find(|l| l.contains("zzz.txt"))
        .expect("zzz.txt not found");

    assert!(
        first_line.contains("├── ") || first_line.contains("|-- "),
        "Non-last child should use branch marker (├──). Got: {:?}",
        first_line
    );
    assert!(
        last_line.contains("└── ") || last_line.contains("`-- "),
        "Last child should use last-branch marker (└──). Got: {:?}",
        last_line
    );
}

// ============================================================================
// Error Output Tests (stderr / exit codes)
// ============================================================================

#[test]
fn test_error_not_found_stderr() {
    rtree()
        .args(["--lang", "en"])
        .arg("/nonexistent/path/xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("rtree:"));
}

#[test]
fn test_error_not_directory_stderr() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("regular_file.txt");
    fs::write(&file_path, "content").unwrap();

    rtree()
        .args(["--lang", "en"])
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("rtree:"));
}

#[test]
fn test_error_not_found_russian() {
    rtree()
        .args(["--lang", "ru"])
        .arg("/nonexistent/path/xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("rtree:"));
}

#[test]
fn test_error_not_directory_russian() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("regular_file.txt");
    fs::write(&file_path, "content").unwrap();

    rtree()
        .args(["--lang", "ru"])
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("rtree:"));
}

#[test]
fn test_stdout_clean_on_success() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_exit_code_success() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg(dir.path()).assert().code(0);
}

#[test]
fn test_exit_code_not_found() {
    rtree()
        .arg("/nonexistent/path/xyz")
        .assert()
        .code(predicate::ne(0));
}

#[test]
fn test_exit_code_not_directory() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    rtree().arg(&file_path).assert().code(predicate::ne(0));
}

// ============================================================================
// Parallel Execution Tests
// ============================================================================

#[test]
fn test_parallel_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("subdir/file2.txt"), "").unwrap();

    rtree()
        .arg("--parallel")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("subdir"));
}

#[test]
fn test_parallel_with_threads() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--parallel", "--threads", "4"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_parallel_with_queue_cap() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--parallel", "--queue-cap", "64"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_parallel_empty_dir() {
    let dir = tempdir().unwrap();

    rtree().arg("--parallel").arg(dir.path()).assert().success();
}

#[test]
fn test_parallel_with_depth_limit() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("level1/level2/level3")).unwrap();

    rtree()
        .args(["--parallel", "-L", "1"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2").not());
}

#[test]
fn test_parallel_dirs_only() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    rtree()
        .args(["--parallel", "-d"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_parallel_exclude_filter() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("keep.rs"), "").unwrap();
    fs::write(p.join("skip.txt"), "").unwrap();

    rtree()
        .args(["--parallel", "-I", "*.txt"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("keep.rs"))
        .stdout(predicate::str::contains("skip.txt").not());
}

// ============================================================================
// Parallel vs Sequential Equivalence (using JSON for reliable comparison)
// ============================================================================

#[test]
fn test_parallel_sequential_file_count_match() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("subdir1/nested")).unwrap();
    fs::create_dir_all(p.join("subdir2")).unwrap();

    for i in 0..10 {
        fs::write(p.join(format!("file{}.txt", i)), "").unwrap();
    }
    for i in 0..5 {
        fs::write(p.join(format!("subdir1/file{}.txt", i)), "").unwrap();
    }
    for i in 0..3 {
        fs::write(p.join(format!("subdir2/file{}.txt", i)), "").unwrap();
    }
    fs::write(p.join("subdir1/nested/deep.txt"), "").unwrap();

    // Sequential
    let seq = rtree().args(["-J"]).arg(p).assert().success();
    let seq_json: serde_json::Value = serde_json::from_slice(&seq.get_output().stdout).unwrap();
    let (seq_files, seq_dirs) = count_files_and_dirs(&seq_json);

    // Parallel
    let par = rtree().args(["--parallel", "-J"]).arg(p).assert().success();
    let par_json: serde_json::Value = serde_json::from_slice(&par.get_output().stdout).unwrap();
    let (par_files, par_dirs) = count_files_and_dirs(&par_json);

    assert_eq!(
        seq_files, par_files,
        "File count mismatch: seq={}, par={}",
        seq_files, par_files
    );
    assert_eq!(
        seq_dirs, par_dirs,
        "Dir count mismatch: seq={}, par={}",
        seq_dirs, par_dirs
    );
}

#[test]
fn test_parallel_sequential_names_match() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for l1 in 0..3 {
        for l2 in 0..3 {
            let sub = p.join(format!("dir{}/sub{}", l1, l2));
            fs::create_dir_all(&sub).unwrap();
            for f in 0..5 {
                fs::write(sub.join(format!("file{}.txt", f)), "").unwrap();
            }
        }
    }

    let seq = rtree().args(["-J"]).arg(p).assert().success();
    let seq_json: serde_json::Value = serde_json::from_slice(&seq.get_output().stdout).unwrap();

    let par = rtree().args(["--parallel", "-J"]).arg(p).assert().success();
    let par_json: serde_json::Value = serde_json::from_slice(&par.get_output().stdout).unwrap();

    let mut seq_names = collect_all_names(&seq_json);
    let mut par_names = collect_all_names(&par_json);
    seq_names.sort();
    par_names.sort();

    assert_eq!(
        seq_names, par_names,
        "Parallel and sequential should list same files"
    );
}

#[test]
fn test_parallel_no_duplicates() {
    use std::collections::HashSet;

    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b/c")).unwrap();
    for i in 0..5 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
        fs::write(p.join(format!("a/f{}.txt", i)), "").unwrap();
        fs::write(p.join(format!("a/b/f{}.txt", i)), "").unwrap();
        fs::write(p.join(format!("a/b/c/f{}.txt", i)), "").unwrap();
    }

    let output = rtree().args(["--parallel", "-J"]).arg(p).assert().success();

    let json: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();

    // Use full paths to avoid false positives from same names in different dirs
    fn collect_paths(entry: &serde_json::Value, prefix: &str, paths: &mut Vec<String>) {
        if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
            let full = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };
            paths.push(full.clone());
            if let Some(contents) = entry.get("contents").and_then(|c| c.as_array()) {
                for child in contents {
                    collect_paths(child, &full, paths);
                }
            }
        }
    }

    let mut all_paths = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            collect_paths(item, "", &mut all_paths);
        }
    }

    let unique: HashSet<_> = all_paths.iter().collect();
    assert_eq!(
        all_paths.len(),
        unique.len(),
        "Duplicate entries found in parallel mode"
    );
}

#[test]
fn test_parallel_json_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["--parallel", "-J"]).arg(p).assert().success();

    let json: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    assert!(json.is_array());
}

#[test]
fn test_parallel_xml_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["--parallel", "-X"]).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.starts_with("<?xml"));
    assert!(stdout.contains("<tree>"));
}

#[test]
fn test_parallel_html_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree()
        .args(["--parallel", "-H", "http://localhost"])
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("<html"));
}

#[test]
fn test_parallel_indentation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("subdir/child.txt"), "").unwrap();

    let output = rtree()
        .args(["--parallel"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    let child_line = stdout
        .lines()
        .find(|l| l.contains("child.txt"))
        .expect("child.txt not found");

    assert!(
        child_line.contains("│") || child_line.starts_with("    "),
        "child.txt should be indented in parallel mode. Got: {:?}",
        child_line
    );
}

#[test]
fn test_parallel_text_same_names_as_sequential() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("subdir1")).unwrap();
    fs::create_dir_all(p.join("subdir2")).unwrap();
    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("subdir1/file2.txt"), "").unwrap();
    fs::write(p.join("subdir2/file3.txt"), "").unwrap();

    let seq = rtree().args(CLEAN).arg(p).assert().success();
    let seq_stdout = String::from_utf8(seq.get_output().stdout.clone()).unwrap();

    let par = rtree()
        .args(["--parallel"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();
    let par_stdout = String::from_utf8(par.get_output().stdout.clone()).unwrap();

    let mut seq_names = extract_names(&seq_stdout);
    let mut par_names = extract_names(&par_stdout);
    seq_names.sort();
    par_names.sort();

    assert_eq!(
        seq_names, par_names,
        "Sequential and parallel should list same entries"
    );
}
