/// базовая работа, -a, -d, -l, -f, -x, -L, --filelimit, --noreport
mod common;
use common::{rtree, CLEAN};

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// Basic Functionality
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
        .args(CLEAN)
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

    let stdout = common::output_stdout(&output);
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

    fs::create_dir_all(p.join("level1/level2/level3")).unwrap();

    rtree()
        .args(["-L", "2"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2"))
        .stdout(predicate::str::contains("level3").not());
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

    let stdout = common::output_stdout(&output);

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
