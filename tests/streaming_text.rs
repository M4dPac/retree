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
#[ignore] // children traversal not yet implemented
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
// Root-only output
// ============================================================================

/// Streaming outputs root directory name and correct report.
#[test]
fn test_streaming_root_only_smoke() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    // With report
    let output = common::run_rtree(p, &["--streaming"]);

    let dir_name = p.file_name().unwrap().to_string_lossy();
    assert!(
        output.starts_with(&*dir_name),
        "streaming output should start with root dir name '{}', got:\n{}",
        dir_name,
        output
    );

    // Root-only: 0 directories (root excluded from count), 0 files
    assert!(
        output.contains("0 directories, 0 files"),
        "root-only streaming should report 0 dirs 0 files, got:\n{}",
        output
    );

    // With --noreport
    let output_nr = common::run_rtree(p, &["--streaming", "--noreport"]);
    let lines: Vec<&str> = output_nr.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "root-only streaming with --noreport should produce exactly 1 line, got: {:?}",
        lines
    );
}
