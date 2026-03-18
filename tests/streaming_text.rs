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
