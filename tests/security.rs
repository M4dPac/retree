//! Security and robustness regression tests.
//!
//! Covers: terminal injection, cycle detection, deep trees,
//! URL encoding, ANSI validation, control char filtering.

mod common;

use common::{run_rtree, run_rtree_args_full, run_rtree_command, run_rtree_full};
use std::os::unix::fs as unix_fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

// ============================================================================
// Terminal injection
// ============================================================================

#[test]
fn safe_print_replaces_ansi_escape() {
    let dir = TempDir::new().unwrap();
    let name = "\x1b[31mRED\x1b[0m";
    std::fs::write(dir.path().join(name), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-q"]);

    assert!(
        !stdout.contains('\x1b'),
        "ANSI escape must not appear in safe-print mode"
    );
    assert!(
        stdout.contains("?[31mRED?[0m"),
        "escape byte must be replaced with ?"
    );
}

#[test]
fn literal_mode_passes_ansi() {
    let dir = TempDir::new().unwrap();
    let name = "\x1b[31mRED\x1b[0m";
    std::fs::write(dir.path().join(name), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-N"]);

    assert!(
        stdout.contains("\x1b[31m"),
        "literal mode must pass ANSI escapes through unchanged"
    );
}

// ============================================================================
// Symlink cycle detection
// ============================================================================

#[test]
fn sequential_symlink_cycle_shows_files() {
    let dir = TempDir::new().unwrap();
    let shared = dir.path().join("shared");
    std::fs::create_dir(&shared).unwrap();
    std::fs::write(shared.join("file.txt"), b"").unwrap();

    unix_fs::symlink(&shared, dir.path().join("link1")).unwrap();
    unix_fs::symlink(&shared, dir.path().join("link2")).unwrap();

    let stdout = run_rtree(dir.path(), &["-l", "-a"]);

    assert!(stdout.contains("file.txt"), "files must be visible");
    let recursive_count =
        stdout.matches("recursive").count() + stdout.matches("рекурсивная").count();
    assert!(
        recursive_count >= 1,
        "at least one link must be marked recursive"
    );
}

#[test]
fn parallel_symlink_cycle_shows_files() {
    let dir = TempDir::new().unwrap();
    let shared = dir.path().join("shared");
    std::fs::create_dir(&shared).unwrap();
    std::fs::write(shared.join("file.txt"), b"").unwrap();

    unix_fs::symlink(&shared, dir.path().join("link1")).unwrap();
    unix_fs::symlink(&shared, dir.path().join("link2")).unwrap();

    let stdout = run_rtree(dir.path(), &["--parallel", "-l", "-a"]);

    assert!(
        stdout.contains("file.txt"),
        "files must be visible in parallel mode"
    );
    let recursive_count =
        stdout.matches("recursive").count() + stdout.matches("рекурсивная").count();
    assert!(
        recursive_count >= 1,
        "at least one link must be marked recursive in parallel mode"
    );
}

#[test]
fn direct_loop_no_hang() {
    let dir = TempDir::new().unwrap();
    unix_fs::symlink(dir.path(), dir.path().join("self_loop")).unwrap();

    let mut cmd = run_rtree_command(dir.path(), &["-l", "-a"]);

    cmd.timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

// ============================================================================
// Deep tree
// ============================================================================

#[test]
fn deep_tree_200_sequential() {
    let dir = TempDir::new().unwrap();
    let mut p = dir.path().to_path_buf();
    for i in 0..200 {
        p = p.join(format!("d{}", i));
        std::fs::create_dir(&p).unwrap();
    }
    std::fs::write(p.join("bottom.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &[]);
    assert!(
        stdout.contains("bottom.txt"),
        "bottom.txt must appear in 200-level deep tree"
    );
}

#[test]
fn deep_tree_200_parallel_no_crash() {
    let dir = TempDir::new().unwrap();
    let mut p = dir.path().to_path_buf();
    for i in 0..200 {
        p = p.join(format!("d{}", i));
        std::fs::create_dir(&p).unwrap();
    }
    std::fs::write(p.join("bottom.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["--parallel"]);

    assert!(
        stdout.contains("bottom.txt"),
        "parallel 200-level tree must not crash"
    );
}

// ============================================================================
// Non-UTF-8 filter bypass
// ============================================================================

#[test]
fn non_utf8_excluded_by_wildcard() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("normal.txt"), b"").unwrap();
    // Create non-UTF8 filename
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let bad_name = std::ffi::OsStr::from_bytes(b"\xff\xfebad");
        std::fs::write(dir.path().join(bad_name), b"").unwrap();
    }

    let stdout = run_rtree(dir.path(), &["-a", "-I", "*"]);

    assert!(
        !stdout.contains("normal"),
        "all files should be excluded by wildcard pattern"
    );
    assert!(
        stdout.contains('0'),
        "report should show 0 files when all are excluded"
    );
}

// ============================================================================
// HTML URL encoding
// ============================================================================

#[test]
fn html_href_url_encoded() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("report#2024.txt"), b"").unwrap();
    std::fs::write(dir.path().join("my file.txt"), b"").unwrap();
    std::fs::write(dir.path().join("50%off.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-H", "."]);

    assert!(
        stdout.contains("report%232024.txt"),
        "# must be encoded as %23"
    );
    assert!(
        stdout.contains("my%20file.txt"),
        "space must be encoded as %20"
    );
    assert!(stdout.contains("50%25off.txt"), "% must be encoded as %25");
}

// ============================================================================
// javascript: URL rejection
// ============================================================================

#[test]
fn javascript_url_rejected() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.txt"), b"").unwrap();

    let (stdout, stderr, _code) = run_rtree_full(dir.path(), &["-H", "javascript:alert(1)"]);

    assert!(
        stderr.contains("warning"),
        "should emit a warning for unsafe base URL"
    );
    assert!(
        !stdout.contains("javascript:"),
        "href must not contain javascript: scheme"
    );
}

// ============================================================================
// Bidi / ZWJ sanitization
// ============================================================================

#[test]
fn bidi_chars_sanitized_with_safe_print() {
    let dir = TempDir::new().unwrap();
    // U+202E Right-to-Left Override
    std::fs::write(dir.path().join("\u{202E}reversed.txt"), b"").unwrap();
    // U+200D Zero Width Joiner
    std::fs::write(dir.path().join("join\u{200D}er.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-q"]);

    assert!(
        stdout.contains("?reversed.txt"),
        "U+202E (RLO) must be replaced with ?"
    );
    assert!(
        stdout.contains("join?er.txt"),
        "U+200D (ZWJ) must be replaced with ?"
    );
}

// ============================================================================
// CLI validation
// ============================================================================

#[test]
fn threads_zero_rejected() {
    let (_stdout, stderr, code) = run_rtree_args_full(&["--parallel", "--threads", "0", "."]);

    assert_ne!(
        code,
        Some(0),
        "threads=0 must be rejected with non-zero exit"
    );
    assert!(
        stderr.contains("not in 1..=4096"),
        "stderr must explain the valid thread range"
    );
}

#[test]
fn queue_cap_zero_rejected() {
    let (_stdout, stderr, code) = run_rtree_args_full(&["--parallel", "--queue-cap", "0", "."]);

    assert_ne!(
        code,
        Some(0),
        "queue-cap=0 must be rejected with non-zero exit"
    );
    assert!(
        stderr.contains("not in 1..=65536"),
        "stderr must explain the valid queue capacity range"
    );
}

// ============================================================================
// Access denied
// ============================================================================

#[test]
fn access_denied_continues_tree() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("denied");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("secret.txt"), b"").unwrap();

    // Remove permissions
    let mut perms = std::fs::metadata(&sub).unwrap().permissions();
    perms.set_mode(0o000);
    std::fs::set_permissions(&sub, perms).unwrap();

    let (_stdout, stderr, code) = run_rtree_full(dir.path(), &["-a"]);

    // Restore permissions for cleanup
    let mut perms = std::fs::metadata(&sub).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&sub, perms).unwrap();

    assert!(
        stderr.contains("Permission denied") || stderr.contains("error 13"),
        "stderr must report access denied"
    );
    assert!(
        code.is_some(),
        "rtree must exit cleanly even when a directory is unreadable"
    );
}

// ============================================================================
// Sequential vs parallel consistency
// ============================================================================

#[test]
fn parallel_and_sequential_same_file_count() {
    let dir = TempDir::new().unwrap();
    for i in 0..50 {
        std::fs::write(dir.path().join(format!("f{}.txt", i)), b"").unwrap();
    }
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    for i in 0..30 {
        std::fs::write(sub.join(format!("g{}.txt", i)), b"").unwrap();
    }

    let seq = run_rtree(dir.path(), &[]);
    let par = run_rtree(dir.path(), &["--parallel"]);

    // Extract last line (report)
    let seq_last = common::last_nonempty_line(&seq);
    let par_last = common::last_nonempty_line(&par);

    assert_eq!(
        seq_last, par_last,
        "sequential and parallel must produce the same summary line"
    );
}
