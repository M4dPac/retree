/// Integration tests for --max-entries flag
mod common;
use common::{rtree, CLEAN};

use std::fs;
use tempfile::tempdir;

// ============================================================================
// Truncation occurs when entries exceed the limit
// ============================================================================

#[test]
fn test_max_entries_truncates_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // 15 files — well above limit of 5
    for i in 0..15 {
        fs::write(p.join(format!("file_{:02}.txt", i)), "").unwrap();
    }

    let output = rtree()
        .args(CLEAN)
        .args(["--max-entries", "5"])
        .arg(p)
        .assert()
        .success();

    let stderr = common::output_stderr(&output);
    assert!(
        stderr.contains("output truncated at 5 entries (--max-entries)"),
        "Expected truncation message in stderr, got: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    let names = common::extract_names(&stdout);
    assert!(
        names.len() <= 5,
        "Expected at most 5 entries, got {}: {names:?}",
        names.len()
    );
}

// ============================================================================
// No truncation when entries fit within the limit
// ============================================================================

#[test]
fn test_max_entries_no_truncation_within_limit() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("a.txt"), "").unwrap();
    fs::write(p.join("b.txt"), "").unwrap();
    fs::write(p.join("c.txt"), "").unwrap();

    let output = rtree()
        .args(CLEAN)
        .args(["--max-entries", "10"])
        .arg(p)
        .assert()
        .success();

    let stderr = common::output_stderr(&output);
    assert!(
        !stderr.contains("truncated"),
        "Should NOT truncate when within limit, stderr: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains("b.txt"));
    assert!(stdout.contains("c.txt"));
}

// ============================================================================
// Edge case: --max-entries 1
// ============================================================================

#[test]
fn test_max_entries_one() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..10 {
        fs::write(p.join(format!("file_{}.txt", i)), "").unwrap();
    }

    let output = rtree()
        .args(CLEAN)
        .args(["--max-entries", "1"])
        .arg(p)
        .assert()
        .success();

    let stderr = common::output_stderr(&output);
    assert!(
        stderr.contains("output truncated at 1 entries (--max-entries)"),
        "Expected truncation message, got: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    let names = common::extract_names(&stdout);
    assert!(
        names.len() <= 1,
        "Expected at most 1 entry, got {}: {names:?}",
        names.len()
    );
}

// ============================================================================
// Truncation with nested subdirectories (dirs + files count toward limit)
// ============================================================================

#[test]
fn test_max_entries_with_subdirectories() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // 6 entries total: dir_a, dir_a/f1, dir_a/f2, dir_b, dir_b/f3, root.txt
    fs::create_dir(p.join("dir_a")).unwrap();
    fs::create_dir(p.join("dir_b")).unwrap();
    fs::write(p.join("dir_a/f1.txt"), "").unwrap();
    fs::write(p.join("dir_a/f2.txt"), "").unwrap();
    fs::write(p.join("dir_b/f3.txt"), "").unwrap();
    fs::write(p.join("root.txt"), "").unwrap();

    let output = rtree()
        .args(CLEAN)
        .args(["--max-entries", "3"])
        .arg(p)
        .assert()
        .success();

    let stderr = common::output_stderr(&output);
    assert!(
        stderr.contains("output truncated at 3 entries (--max-entries)"),
        "Expected truncation message, got: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    let names = common::extract_names(&stdout);
    assert!(
        names.len() <= 3,
        "Expected at most 3 entries, got {}: {names:?}",
        names.len()
    );
}

// ============================================================================
// Truncation does NOT cause non-zero exit code
// ============================================================================

#[test]
fn test_max_entries_exit_code_zero() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..5 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
    }

    rtree()
        .args(CLEAN)
        .args(["--max-entries", "2"])
        .arg(p)
        .assert()
        .success();
}

// ============================================================================
// Without --max-entries all entries are shown, no truncation message
// ============================================================================

#[test]
fn test_without_max_entries_shows_all() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..5 {
        fs::write(p.join(format!("item_{}.txt", i)), "").unwrap();
    }

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stderr = common::output_stderr(&output);
    assert!(
        !stderr.contains("truncated"),
        "Without --max-entries there should be no truncation, stderr: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    for i in 0..5 {
        let name = format!("item_{}.txt", i);
        assert!(stdout.contains(&name), "{name} missing from output");
    }
}

// ============================================================================
// Exact boundary: entries == limit → no truncation
// ============================================================================

#[test]
fn test_max_entries_exact_boundary_no_truncation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // Exactly 3 entries in a flat directory
    fs::write(p.join("one.txt"), "").unwrap();
    fs::write(p.join("two.txt"), "").unwrap();
    fs::write(p.join("three.txt"), "").unwrap();

    // Limit set above entry count — no truncation
    let output = rtree()
        .args(CLEAN)
        .args(["--max-entries", "4"])
        .arg(p)
        .assert()
        .success();

    let stderr = common::output_stderr(&output);
    assert!(
        !stderr.contains("truncated"),
        "When entries < limit, no truncation expected, stderr: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    assert!(stdout.contains("one.txt"));
    assert!(stdout.contains("two.txt"));
    assert!(stdout.contains("three.txt"));
}

#[test]
fn test_max_entries_at_exact_count_no_truncation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("one.txt"), "").unwrap();
    fs::write(p.join("two.txt"), "").unwrap();
    fs::write(p.join("three.txt"), "").unwrap();

    // Limit == entry count → all entries shown, nothing truncated
    let output = rtree()
        .args(CLEAN)
        .args(["--max-entries", "3"])
        .arg(p)
        .assert()
        .success();

    let stderr = common::output_stderr(&output);
    assert!(
        !stderr.contains("truncated"),
        "When entries == limit, no truncation expected, stderr: {stderr}"
    );

    let stdout = common::output_stdout(&output);
    assert!(stdout.contains("one.txt"));
    assert!(stdout.contains("two.txt"));
    assert!(stdout.contains("three.txt"));
}
