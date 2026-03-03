/// -P, -I, --matchdirs, --ignore-case, --prune
mod common;
use common::rtree;

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

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

    fs::create_dir(p.join("hollow")).unwrap();
    fs::create_dir(p.join("filled")).unwrap();
    fs::write(p.join("filled/file.txt"), "").unwrap();

    rtree()
        .arg("--prune")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("hollow").not())
        .stdout(predicate::str::contains("filled"))
        .stdout(predicate::str::contains("file.txt"));
}
