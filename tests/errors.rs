/// stderr, exit codes
mod common;
use common::rtree;

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// stderr messages
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

// ============================================================================
// Exit codes
// ============================================================================

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
