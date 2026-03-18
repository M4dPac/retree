/// Integration tests for --streaming text output mode.
///
/// Streaming engine renders text during DFS traversal instead of
/// building a full tree in memory. Output must be identical to
/// the normal (tree-based) text renderer.
mod common;

use std::fs;
use tempfile::tempdir;

// ============================================================================
// Smoke tests
// ============================================================================

/// --streaming flag is accepted and process exits successfully.
#[test]
fn test_streaming_flag_smoke() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("hello.txt"), "").unwrap();

    common::rtree()
        .args(common::CLEAN)
        .arg("--streaming")
        .arg(dir.path())
        .assert()
        .success();
}

/// --streaming produces identical text output to normal mode.
#[test]
#[ignore] // children traversal not yet recursive
fn test_streaming_basic_execution() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file1.txt"), "content").unwrap();
    fs::write(p.join("subdir/file2.txt"), "content").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--noreport"]);
    let normal = common::run_rtree(p, &["--noreport"]);

    assert_eq!(
        streaming, normal,
        "streaming should produce identical output to normal mode"
    );
}

// ============================================================================
// Empty / flat directory
// ============================================================================

/// Empty directory: root line only, 0 directories 0 files.
#[test]
fn test_streaming_empty_directory() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let output = common::run_rtree(p, &["--streaming"]);
    let dir_name = p.file_name().unwrap().to_string_lossy();
    assert!(
        output.starts_with(&*dir_name),
        "output should start with root dir name '{}', got:\n{}",
        dir_name,
        output
    );
    assert!(
        output.contains("0 directories, 0 files"),
        "empty dir should report 0 dirs 0 files, got:\n{}",
        output
    );
}

/// Flat directory: children listed with correct tree chars.
#[test]
fn test_streaming_flat_directory_lists_children() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("file2.txt"), "").unwrap();

    let output = common::run_rtree(p, &["--streaming", "--noreport"]);

    assert!(
        output.contains("file1.txt"),
        "should list file1.txt:\n{}",
        output
    );
    assert!(
        output.contains("file2.txt"),
        "should list file2.txt:\n{}",
        output
    );
    assert!(output.contains("subdir"), "should list subdir:\n{}", output);

    // Last entry uses └──, others use ├──
    assert!(
        output.contains("├── "),
        "should contain branch char:\n{}",
        output
    );
    assert!(
        output.contains("└── "),
        "should contain last-branch char:\n{}",
        output
    );
}

/// Flat directory: children sorted by name.
#[test]
fn test_streaming_flat_directory_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("cherry.txt"), "").unwrap();
    fs::write(p.join("apple.txt"), "").unwrap();
    fs::write(p.join("banana.txt"), "").unwrap();

    let output = common::run_rtree(p, &["--streaming", "--noreport"]);
    let names = common::extract_names(&output);

    assert_eq!(
        names,
        vec!["apple.txt", "banana.txt", "cherry.txt"],
        "children should be sorted by name"
    );
}
