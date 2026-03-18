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

// ============================================================================
// Nested / recursive DFS
// ============================================================================

/// Nested structure: streaming matches normal output.
#[test]
fn test_streaming_nested_execution() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b/c")).unwrap();
    fs::write(p.join("a/b/c/deep.txt"), "").unwrap();
    fs::write(p.join("a/b/mid.txt"), "").unwrap();
    fs::write(p.join("a/top.txt"), "").unwrap();
    fs::write(p.join("root.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--noreport"]);
    let normal = common::run_rtree(p, &["--noreport"]);

    assert_eq!(
        streaming, normal,
        "nested streaming should match normal output"
    );
}

/// DFS order: directory contents appear immediately after the directory.
#[test]
fn test_streaming_depth_first_order_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("alpha")).unwrap();
    fs::write(p.join("alpha/inside.txt"), "").unwrap();
    fs::write(p.join("beta.txt"), "").unwrap();

    let output = common::run_rtree(p, &["--streaming", "--noreport"]);
    let names = common::extract_names(&output);

    assert_eq!(
        names,
        vec!["alpha", "inside.txt", "beta.txt"],
        "DFS: alpha's child should appear before beta.txt, got: {:?}",
        names
    );
}

// ============================================================================
// Indentation and tree connectors
// ============================================================================

/// ├── for non-last, └── for last sibling.
#[test]
fn test_streaming_last_branch_rendering_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("zzz.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--noreport"]);
    let normal = common::run_rtree(p, &["--noreport"]);

    assert_eq!(streaming, normal, "branch chars should match normal mode");
    assert!(
        streaming.contains("├── aaa.txt"),
        "first child uses ├──:\n{}",
        streaming
    );
    assert!(
        streaming.contains("└── zzz.txt"),
        "last child uses └──:\n{}",
        streaming
    );
}

/// │ prefix for non-last parent, space prefix for last parent.
#[test]
fn test_streaming_indentation_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("dir_a")).unwrap();
    fs::write(p.join("dir_a/child.txt"), "").unwrap();
    fs::create_dir(p.join("dir_b")).unwrap();
    fs::write(p.join("dir_b/child.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--noreport"]);
    let normal = common::run_rtree(p, &["--noreport"]);

    assert_eq!(streaming, normal, "indentation should match normal mode");

    // dir_a is not last → its child gets │ prefix
    assert!(
        streaming.contains("│   └── child.txt"),
        "non-last parent should produce │ prefix:\n{}",
        streaming
    );
}

/// Deeper nesting: │ and spaces propagate correctly through 3+ levels.
#[test]
fn test_streaming_nested_indentation_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // dir_a not last → children get │; inner is last-child of dir_a → its children get "│       "
    fs::create_dir_all(p.join("dir_a/inner")).unwrap();
    fs::write(p.join("dir_a/inner/deep.txt"), "").unwrap();
    // dir_b is last → children get space
    fs::create_dir(p.join("dir_b")).unwrap();
    fs::write(p.join("dir_b/leaf.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--noreport"]);
    let normal = common::run_rtree(p, &["--noreport"]);

    assert_eq!(
        streaming, normal,
        "nested indentation should match normal mode:\nstreaming:\n{}normal:\n{}",
        streaming, normal
    );
}

// ============================================================================
// Feature parity: filtering, depth, sorting
// ============================================================================

/// -a: streaming shows hidden files.
#[test]
fn test_streaming_show_all() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join(".hidden"), "").unwrap();
    fs::write(p.join("visible.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "-a", "--noreport"]);
    let normal = common::run_rtree(p, &["-a", "--noreport"]);
    assert_eq!(streaming, normal, "-a streaming should match normal");
}

/// -d: streaming shows only directories.
#[test]
fn test_streaming_dirs_only() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();
    fs::write(p.join("subdir/inner.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "-d", "--noreport"]);
    let normal = common::run_rtree(p, &["-d", "--noreport"]);
    assert_eq!(streaming, normal, "-d streaming should match normal");
}

/// -L: streaming respects max depth.
#[test]
fn test_streaming_depth_limit() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("l1/l2/l3")).unwrap();
    fs::write(p.join("l1/l2/l3/deep.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "-L", "2", "--noreport"]);
    let normal = common::run_rtree(p, &["-L", "2", "--noreport"]);
    assert_eq!(streaming, normal, "-L 2 streaming should match normal");
}

/// -P: streaming include pattern filters files.
#[test]
fn test_streaming_pattern_include() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.rs"), "").unwrap();
    fs::write(p.join("file.txt"), "").unwrap();
    fs::write(p.join("other.rs"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "-P", "*.rs", "--noreport"]);
    let normal = common::run_rtree(p, &["-P", "*.rs", "--noreport"]);
    assert_eq!(streaming, normal, "-P streaming should match normal");
}

/// -I: streaming exclude pattern hides entries.
#[test]
fn test_streaming_exclude() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("keep.rs"), "").unwrap();
    fs::write(p.join("skip.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "-I", "*.txt", "--noreport"]);
    let normal = common::run_rtree(p, &["-I", "*.txt", "--noreport"]);
    assert_eq!(streaming, normal, "-I streaming should match normal");
}

/// --filelimit: streaming skips large directories' children.
#[test]
fn test_streaming_filelimit() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("big")).unwrap();
    for i in 0..5 {
        fs::write(p.join(format!("big/file{}.txt", i)), "").unwrap();
    }
    fs::create_dir(p.join("small")).unwrap();
    fs::write(p.join("small/ok.txt"), "").unwrap();

    let output = common::run_rtree(p, &["--streaming", "--filelimit", "2", "--noreport"]);

    // Core behavior: children of big dir are skipped
    assert!(output.contains("big"), "big dir shown:\n{}", output);
    assert!(output.contains("ok.txt"), "small/ok.txt shown:\n{}", output);
    assert!(
        !output.contains("file0.txt"),
        "big's children hidden:\n{}",
        output
    );
}

/// Sorting: streaming sort order matches normal mode.
#[test]
fn test_streaming_sort_order_matches_regular_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("cherry")).unwrap();
    fs::create_dir(p.join("apple")).unwrap();
    fs::write(p.join("cherry/c.txt"), "").unwrap();
    fs::write(p.join("apple/a.txt"), "").unwrap();
    fs::write(p.join("banana.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--noreport"]);
    let normal = common::run_rtree(p, &["--noreport"]);
    assert_eq!(
        streaming, normal,
        "sort order streaming should match normal"
    );
}

// ============================================================================
// max_entries
// ============================================================================

/// --max-entries truncates streaming output.
#[test]
fn test_streaming_max_entries_truncates() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..15 {
        fs::write(p.join(format!("file_{:02}.txt", i)), "").unwrap();
    }

    let (stdout, stderr, code) =
        common::run_rtree_full(p, &["--streaming", "--max-entries", "5", "--noreport"]);
    assert_eq!(code, Some(0), "exit 0 on truncation");
    assert!(
        stderr.contains("output truncated at 5 entries (--max-entries)"),
        "truncation message expected, stderr: {stderr}"
    );
    let names = common::extract_names(&stdout);
    assert!(
        names.len() <= 5,
        "at most 5 entries, got {}: {names:?}",
        names.len()
    );
}

/// --max-entries with enough room: no truncation.
#[test]
fn test_streaming_max_entries_no_truncation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("a.txt"), "").unwrap();
    fs::write(p.join("b.txt"), "").unwrap();

    let (stdout, stderr, code) = common::run_rtree_full(p, &["--streaming", "--max-entries", "10"]);
    assert_eq!(code, Some(0));
    assert!(
        !stderr.contains("truncated"),
        "should NOT truncate, stderr: {stderr}"
    );
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains("b.txt"));
}

/// --max-entries with nested dirs: DFS count includes subdirs.
#[test]
fn test_streaming_max_entries_nested() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // 6 entries: dir_a, f1, f2, dir_b, f3, root.txt
    fs::create_dir(p.join("dir_a")).unwrap();
    fs::create_dir(p.join("dir_b")).unwrap();
    fs::write(p.join("dir_a/f1.txt"), "").unwrap();
    fs::write(p.join("dir_a/f2.txt"), "").unwrap();
    fs::write(p.join("dir_b/f3.txt"), "").unwrap();
    fs::write(p.join("root.txt"), "").unwrap();

    let (stdout, stderr, _) =
        common::run_rtree_full(p, &["--streaming", "--max-entries", "3", "--noreport"]);
    assert!(
        stderr.contains("output truncated at 3 entries (--max-entries)"),
        "truncation expected, stderr: {stderr}"
    );
    let names = common::extract_names(&stdout);
    assert!(
        names.len() <= 3,
        "at most 3 entries, got {}: {names:?}",
        names.len()
    );
}

// ============================================================================
// prune: fallback to normal mode
// ============================================================================

/// --prune + --streaming: falls back to normal mode, works correctly.
#[test]
fn test_streaming_prune_falls_back_to_normal() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("hollow")).unwrap();
    fs::create_dir(p.join("filled")).unwrap();
    fs::write(p.join("filled/file.txt"), "").unwrap();

    let streaming = common::run_rtree(p, &["--streaming", "--prune", "--noreport"]);
    let normal = common::run_rtree(p, &["--prune", "--noreport"]);

    assert_eq!(
        streaming, normal,
        "--prune + --streaming should produce same output as --prune alone"
    );
    assert!(
        !streaming.contains("hollow"),
        "hollow dir should be pruned:\n{}",
        streaming
    );
    assert!(
        streaming.contains("filled"),
        "filled dir should remain:\n{}",
        streaming
    );
}
