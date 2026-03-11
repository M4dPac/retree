//! Security and robustness regression tests.
//!
//! Covers: terminal injection, cycle detection, deep trees,
//! URL encoding, ANSI validation, control char filtering.
//!
//! Platform gates:
//! - ANSI in filenames: `#[cfg(unix)]` — Windows forbids control chars in names
//! - Symlink tests: `#[cfg(unix)]` — Windows requires elevated privileges
//! - Non-UTF-8 filenames: `#[cfg(target_os = "linux")]` — macOS enforces UTF-8
//! - Permission bits: `#[cfg(unix)]` — `set_mode` unavailable on Windows
//! - Windows-specific: `#[cfg(windows)]` — junctions, long paths, ADS

mod common;

use common::{run_rtree, run_rtree_args_full, run_rtree_command, run_rtree_full};
use tempfile::TempDir;

// ============================================================================
// Terminal injection (Unix only — Windows forbids control chars in filenames)
// ============================================================================

#[cfg(unix)]
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

#[cfg(unix)]
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
// Symlink cycle detection (Unix only — Windows symlinks require elevation)
// ============================================================================

#[cfg(unix)]
#[test]
fn sequential_symlink_cycle_shows_files() {
    use std::os::unix::fs as unix_fs;

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

#[cfg(unix)]
#[test]
fn parallel_symlink_cycle_shows_files() {
    use std::os::unix::fs as unix_fs;

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

#[cfg(unix)]
#[test]
fn direct_loop_no_hang() {
    use std::os::unix::fs as unix_fs;

    let dir = TempDir::new().unwrap();
    unix_fs::symlink(dir.path(), dir.path().join("self_loop")).unwrap();

    let mut cmd = run_rtree_command(dir.path(), &["-l", "-a"]);

    cmd.timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

// ============================================================================
// Deep tree (cross-platform)
// ============================================================================

/// 100 levels is safe for debug-mode stack on all platforms.
/// Production limit is MAX_INTERNAL_DEPTH=4096 (release builds with optimized frames).
const TEST_DEEP_LEVELS: usize = 100;

#[test]
fn deep_tree_sequential() {
    let dir = TempDir::new().unwrap();
    let mut p = dir.path().to_path_buf();
    for i in 0..TEST_DEEP_LEVELS {
        p = p.join(format!("d{}", i));
        std::fs::create_dir(&p).unwrap();
    }
    std::fs::write(p.join("bottom.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &[]);
    assert!(
        stdout.contains("bottom.txt"),
        "bottom.txt must appear in {}-level deep tree",
        TEST_DEEP_LEVELS
    );
}

#[test]
fn deep_tree_parallel_no_crash() {
    let dir = TempDir::new().unwrap();
    let mut p = dir.path().to_path_buf();
    for i in 0..TEST_DEEP_LEVELS {
        p = p.join(format!("d{}", i));
        std::fs::create_dir(&p).unwrap();
    }
    std::fs::write(p.join("bottom.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["--parallel"]);

    assert!(
        stdout.contains("bottom.txt"),
        "parallel {}-level tree must not crash",
        TEST_DEEP_LEVELS
    );
}

// ============================================================================
// Non-UTF-8 filter bypass (Linux only — macOS enforces UTF-8 in filenames)
// ============================================================================

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_excluded_by_wildcard() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("normal.txt"), b"").unwrap();

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
// HTML URL encoding (cross-platform)
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
// javascript: URL rejection (cross-platform)
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
// Bidi / ZWJ sanitization (cross-platform — Unicode filenames work everywhere)
// ============================================================================

#[test]
fn bidi_chars_sanitized_with_safe_print() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("\u{202E}reversed.txt"), b"").unwrap();
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
// CLI validation (cross-platform)
// ============================================================================

#[test]
fn threads_zero_rejected() {
    let (_stdout, stderr, code) = run_rtree_args_full(&["--parallel", "--threads", "0", "."]);

    assert_ne!(code, Some(0), "threads=0 must be rejected");
    assert!(
        stderr.contains("not in 1..=4096"),
        "stderr must explain the valid thread range"
    );
}

#[test]
fn queue_cap_zero_rejected() {
    let (_stdout, stderr, code) = run_rtree_args_full(&["--parallel", "--queue-cap", "0", "."]);

    assert_ne!(code, Some(0), "queue-cap=0 must be rejected");
    assert!(
        stderr.contains("not in 1..=65536"),
        "stderr must explain the valid queue capacity range"
    );
}

// ============================================================================
// Access denied (Unix only — requires chmod)
// ============================================================================

#[cfg(unix)]
#[test]
fn access_denied_continues_tree() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("denied");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("secret.txt"), b"").unwrap();

    let mut perms = std::fs::metadata(&sub).unwrap().permissions();
    perms.set_mode(0o000);
    std::fs::set_permissions(&sub, perms).unwrap();

    let (_stdout, stderr, code) = run_rtree_full(dir.path(), &["-a"]);

    let mut perms = std::fs::metadata(&sub).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&sub, perms).unwrap();

    assert!(
        stderr.contains("Permission denied") || stderr.contains("error 13"),
        "stderr must report access denied"
    );
    assert!(code.is_some(), "rtree must exit cleanly");
}

// ============================================================================
// Sequential vs parallel consistency (cross-platform)
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

    let seq_last = common::last_nonempty_line(&seq);
    let par_last = common::last_nonempty_line(&par);

    assert_eq!(
        seq_last, par_last,
        "sequential and parallel must produce the same summary line"
    );
}

// ============================================================================
// Executable detection (Unix — permission bits)
// ============================================================================

#[cfg(unix)]
#[test]
fn executable_bit_detected_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();

    let script = dir.path().join("script.sh");
    std::fs::write(&script, b"#!/bin/sh\necho hi").unwrap();
    let mut perms = std::fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).unwrap();

    std::fs::write(dir.path().join("readme.txt"), b"").unwrap();
    std::fs::write(dir.path().join("program.exe"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-F"]);

    assert!(
        stdout.contains("script.sh*"),
        "chmod +x file must get * marker with -F on Unix"
    );
    assert!(
        !stdout.contains("readme.txt*"),
        "non-executable file must not get * marker"
    );
    assert!(
        !stdout.contains("program.exe*"),
        ".exe without +x should not get * marker on Unix"
    );
}

// ============================================================================
// Windows-specific tests
// ============================================================================

#[cfg(windows)]
#[test]
fn windows_executable_by_extension() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("app.exe"), b"").unwrap();
    std::fs::write(dir.path().join("script.bat"), b"").unwrap();
    std::fs::write(dir.path().join("readme.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-F"]);
    assert!(stdout.contains("app.exe*"), ".exe must get * on Windows");
    assert!(stdout.contains("script.bat*"), ".bat must get * on Windows");
    assert!(!stdout.contains("readme.txt*"), ".txt must not get *");
}

#[cfg(windows)]
#[test]
fn junction_cycle_no_infinite_recursion() {
    use std::process::Command as StdCommand;

    let dir = TempDir::new().unwrap();
    let target = dir.path().join("target_dir");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("data.txt"), b"content").unwrap();

    let junction = dir.path().join("loop");
    StdCommand::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&target)
        .output()
        .expect("mklink /J failed");

    let mut cmd = run_rtree_command(dir.path(), &["-a"]);
    cmd.timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    let stdout = run_rtree(dir.path(), &["-a"]);
    assert!(
        stdout.contains("data.txt"),
        "junction target contents must be visible"
    );
}

#[cfg(windows)]
#[test]
fn long_path_with_flag() {
    let dir = TempDir::new().unwrap();
    let mut p = dir.path().to_path_buf();
    for i in 0..30 {
        p = p.join(format!("long_dir_name_{:03}", i));
        std::fs::create_dir_all(&p).unwrap_or_default();
    }
    let _ = std::fs::write(p.join("deep.txt"), b"test");

    let (_stdout, _stderr, code) = run_rtree_full(dir.path(), &["--long-paths"]);
    assert!(
        code == Some(0) || code == Some(1),
        "rtree must not crash with long paths"
    );
}

#[cfg(windows)]
#[test]
fn show_streams_flag_accepted() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.txt"), b"data").unwrap();

    let (_stdout, _stderr, code) = run_rtree_full(dir.path(), &["--show-streams"]);
    assert!(
        code == Some(0) || code == Some(1),
        "--show-streams must not crash"
    );
}

#[cfg(windows)]
#[test]
fn parallel_junction_no_crash() {
    use std::process::Command as StdCommand;

    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file.txt"), b"").unwrap();

    let junction = dir.path().join("loop");
    let _ = StdCommand::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(dir.path())
        .output();

    let mut cmd = run_rtree_command(dir.path(), &["--parallel", "-a"]);
    cmd.timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}

#[cfg(windows)]
#[test]
fn html_href_uses_forward_slash() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file.txt"), b"").unwrap();

    let stdout = run_rtree(dir.path(), &["-H", "."]);
    assert!(
        !stdout.contains("href=\".\\"),
        "href must not contain backslash on Windows"
    );
}
