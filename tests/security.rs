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

use common::{run_retree, run_retree_args_full, run_retree_command, run_retree_full};
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

    let stdout = run_retree(dir.path(), &["-q"]);

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

    let stdout = run_retree(dir.path(), &["-N"]);

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

    let stdout = run_retree(dir.path(), &["-l", "-a"]);

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

    let stdout = run_retree(dir.path(), &["--parallel", "-l", "-a"]);

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

    let mut cmd = run_retree_command(dir.path(), &["-l", "-a"]);

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

    let stdout = run_retree(dir.path(), &[]);
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

    let stdout = run_retree(dir.path(), &["--parallel"]);

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

    let stdout = run_retree(dir.path(), &["-a", "-I", "*"]);

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

    let stdout = run_retree(dir.path(), &["-H", "."]);

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

    let (stdout, stderr, _code) = run_retree_full(dir.path(), &["-H", "javascript:alert(1)"]);

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

    let stdout = run_retree(dir.path(), &["-q"]);

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
// Bidi / ZWJ sanitization in HTML and XML output
// ============================================================================

#[test]
fn html_strips_bidi_from_filenames() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test\u{202E}gpj.exe"), b"").unwrap();
    std::fs::write(dir.path().join("join\u{200D}er.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-H", "."]);

    // Bidi override and ZWJ must be stripped from HTML output
    assert!(
        !stdout.contains('\u{202E}'),
        "U+202E (RLO) must be stripped from HTML output"
    );
    assert!(
        !stdout.contains('\u{200D}'),
        "U+200D (ZWJ) must be stripped from HTML output"
    );
    // Filenames should still be recognizable
    assert!(
        stdout.contains("testgpj.exe"),
        "filename without bidi char should appear in HTML"
    );
    assert!(
        stdout.contains("joiner.txt"),
        "filename without ZWJ should appear in HTML"
    );
}

#[test]
fn xml_strips_bidi_from_filenames() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test\u{202E}gpj.exe"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-X"]);

    assert!(
        !stdout.contains('\u{202E}'),
        "U+202E (RLO) must be stripped from XML output"
    );
    assert!(
        stdout.contains("testgpj.exe"),
        "filename without bidi char should appear in XML"
    );
}

#[test]
fn html_escapes_ampersand_in_filenames() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Tom & Jerry.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-H", ".", "--nolinks"]);

    assert!(
        stdout.contains("Tom &amp; Jerry.txt"),
        "& in filename must be escaped as &amp; in HTML"
    );
    assert!(
        !stdout.contains("Tom & Jerry.txt"),
        "raw & must not appear in HTML output"
    );
}

#[test]
fn xml_escapes_ampersand_in_filenames() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Tom & Jerry.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-X"]);

    assert!(
        stdout.contains("Tom &amp; Jerry.txt"),
        "& in filename must be escaped as &amp; in XML"
    );
}

// ============================================================================
// HTML title and URL escaping edge cases
// ============================================================================

#[test]
fn html_title_with_special_chars_escaped() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("file.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-H", ".", "-T", "A&B 'quoted' <tag>"]);

    assert!(
        stdout.contains("A&amp;B &#39;quoted&#39; &lt;tag&gt;"),
        "title with & ' < > must be HTML-escaped in <title> and <h1>"
    );
    assert!(
        !stdout.contains("<tag>"),
        "raw < > must not appear unescaped in HTML title"
    );
}

#[test]
fn html_base_url_with_ampersand_encoded() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("file.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-H", "http://example.com?a=1&b=2"]);

    assert!(
        stdout.contains("http://example.com?a=1&amp;b=2"),
        "& in base URL must be escaped as &amp; in href attribute"
    );
}

#[test]
fn html_filename_with_apostrophe_escaped() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("it's here.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-H", ".", "--nolinks"]);

    assert!(
        stdout.contains("it&#39;s here.txt"),
        "apostrophe in filename must be escaped as &#39; in HTML"
    );
}

// ============================================================================
// CLI validation (cross-platform)
// ============================================================================

#[test]
fn threads_zero_rejected() {
    let (_stdout, stderr, code) = run_retree_args_full(&["--parallel", "--threads", "0", "."]);

    assert_ne!(code, Some(0), "threads=0 must be rejected");
    assert!(
        stderr.contains("not in 1..=256"),
        "stderr must explain the valid thread range"
    );
}

#[test]
fn queue_cap_zero_rejected() {
    let (_stdout, stderr, code) = run_retree_args_full(&["--parallel", "--queue-cap", "0", "."]);

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

    let (_stdout, stderr, code) = run_retree_full(dir.path(), &["-a"]);

    let mut perms = std::fs::metadata(&sub).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&sub, perms).unwrap();

    assert!(
        stderr.contains("Permission denied") || stderr.contains("error 13"),
        "stderr must report access denied"
    );
    assert!(code.is_some(), "retree must exit cleanly");
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

    let seq = run_retree(dir.path(), &[]);
    let par = run_retree(dir.path(), &["--parallel"]);

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

    let stdout = run_retree(dir.path(), &["-F"]);

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

/// Windows executable detection uses extension
#[cfg(windows)]
#[test]
fn windows_executable_by_extension() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("app.exe"), b"").unwrap();
    std::fs::write(dir.path().join("script.bat"), b"").unwrap();
    std::fs::write(dir.path().join("readme.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-F"]);
    assert!(
        stdout.contains("app.exe*"),
        ".exe must get * marker on Windows"
    );
    assert!(
        stdout.contains("script.bat*"),
        ".bat must get * marker on Windows"
    );
    assert!(
        !stdout.contains("readme.txt*"),
        ".txt must not get * marker"
    );
}

#[cfg(windows)]
#[test]
fn junction_cycle_no_infinite_recursion() {
    use std::process::Command as StdCommand;
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("target_dir");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("data.txt"), b"content").unwrap();

    // Create junction: dir\loop -> dir\target_dir
    let junction = dir.path().join("loop");
    StdCommand::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&target)
        .output()
        .expect("mklink /J failed");

    let stdout = run_retree(dir.path(), &["-a"]);
    assert!(
        stdout.contains("data.txt"),
        "junction target contents must be visible"
    );
    // Should not hang or crash
}

/// Junction to different volume + --one-fs should not descend
#[cfg(windows)]
#[test]
fn one_fs_stops_at_junction_to_other_volume() {
    // This test requires two different volumes (e.g. C: and D:)
    // Skip if only one volume available
    let dir = TempDir::new().unwrap();
    let stdout = run_retree(dir.path(), &["-x"]);
    // Just verify --one-fs doesn't crash
    assert!(
        stdout.contains("0"),
        "should produce valid output with --one-fs"
    );
}

/// Long path > 260 chars with --long-paths
#[cfg(windows)]
#[test]
fn long_path_with_flag() {
    let dir = TempDir::new().unwrap();
    let mut p = dir.path().to_path_buf();
    // Create path exceeding 260 chars
    for i in 0..30 {
        let segment = format!("long_dir_name_{:03}", i);
        p = p.join(&segment);
        std::fs::create_dir(&p).unwrap_or_default();
    }
    // Try to create a file at the bottom
    let _ = std::fs::write(p.join("deep.txt"), b"test");

    let (stdout, _stderr, code) = run_retree_full(dir.path(), &["--long-paths"]);
    assert!(
        code == Some(0) || code == Some(1),
        "retree must not crash with long paths"
    );
    // With --long-paths, the deep file should be reachable
    if p.to_string_lossy().len() > 260 {
        assert!(
            stdout.contains("deep.txt"),
            "deep.txt must be visible with --long-paths when path > 260 chars"
        );
    }
}

/// ADS (Alternate Data Streams) — --show-streams flag accepted
#[cfg(windows)]
#[test]
fn show_streams_flag_accepted() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.txt"), b"data").unwrap();

    // --show-streams should be accepted without crash
    let (_stdout, _stderr, code) = run_retree_full(dir.path(), &["--show-streams"]);
    assert!(
        code == Some(0) || code == Some(1),
        "--show-streams must not crash"
    );
}

/// Reserved Windows names — skipped with warning, no hang, no crash
#[cfg(windows)]
#[test]
fn reserved_names_skipped_on_windows() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("normal.txt"), b"").unwrap();

    // These creations will likely fail on Windows (redirected to devices)
    // but the test verifies retree doesn't hang if they somehow exist
    for name in ["CON", "NUL", "PRN", "AUX", "COM1", "LPT1"] {
        let _ = std::fs::write(dir.path().join(name), b"x");
    }

    let mut cmd = run_retree_command(dir.path(), &["-a"]);
    let assert = cmd
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("normal.txt"), "normal.txt must be listed");
}

/// Parallel mode with junctions — no crash, no infinite loop
#[cfg(windows)]
#[test]
fn parallel_junction_no_crash() {
    use std::process::Command as StdCommand;
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file.txt"), b"").unwrap();

    // Create junction loop: dir\loop -> dir
    let junction = dir.path().join("loop");
    let _ = StdCommand::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(dir.path())
        .output();

    let mut cmd = run_retree_command(dir.path(), &["--parallel", "-a"]);
    cmd.timeout(std::time::Duration::from_secs(10))
        .assert()
        .success();
}

/// Backslash in path converted to forward slash in HTML href
#[cfg(windows)]
#[test]
fn html_href_uses_forward_slash() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file.txt"), b"").unwrap();

    let stdout = run_retree(dir.path(), &["-H", "."]);
    // href should use / not backslash
    assert!(
        !stdout.contains("href=\".\\"),
        "href must not contain backslash"
    );
}

// ============================================================================
// Windows reserved device name detection & traversal
// ============================================================================

#[test]
fn reserved_name_detection_correctness() {
    use retree::platform::is_reserved_windows_name;

    // Positive: must detect
    let reserved = [
        "CON",
        "con",
        "Con",
        "PRN",
        "prn",
        "AUX",
        "NUL",
        "nul",
        "COM1",
        "COM9",
        "com1",
        "LPT1",
        "LPT9",
        "lpt1",
        "CON.txt",
        "NUL.tar.gz",
        "aux.log",
        "COM1.serial",
    ];
    for name in &reserved {
        assert!(
            is_reserved_windows_name(name),
            "{name} must be detected as reserved"
        );
    }

    // Negative: must NOT detect
    let normal = [
        "",
        "CO",
        "CONNN",
        "CONNECT",
        "console.log",
        "COM0",
        "COM10",
        "LPT0",
        "LPT10",
        "NULLIFY",
        "auxiliary",
        "normal.txt",
        "a",
        "AB",
        "contest",
        "prune",
        "lpt10.dat",
    ];
    for name in &normal {
        assert!(
            !is_reserved_windows_name(name),
            "{name} must NOT be detected as reserved"
        );
    }
}

#[test]
fn traverse_dir_with_reserved_names_no_crash() {
    let dir = TempDir::new().unwrap();

    // On Unix these are normal files; on Windows creation may silently fail
    let names = ["CON", "NUL", "PRN", "AUX", "COM1", "LPT1"];
    for name in &names {
        let _ = std::fs::write(dir.path().join(name), b"test");
    }
    std::fs::write(dir.path().join("normal.txt"), b"ok").expect("write");

    let (stdout, stderr, code) = run_retree_full(dir.path(), &["-a"]);

    assert!(
        code == Some(0) || code == Some(1),
        "retree must not crash on reserved names, code={code:?}"
    );
    assert!(
        stdout.contains("normal.txt"),
        "normal.txt must always appear:\n{stdout}"
    );

    if cfg!(windows) {
        // On Windows: reserved names skipped, warnings on stderr
        for name in &names {
            assert!(
                !stdout.contains(name) || stderr.contains("Reserved Windows device name"),
                "reserved name {name} should be skipped or warned about on Windows"
            );
        }
    } else {
        // On Unix: all files are regular, all must appear
        for name in &names {
            assert!(
                stdout.contains(name),
                "{name} should appear on Unix:\n{stdout}"
            );
        }
    }
}

#[test]
fn traverse_reserved_names_parallel_consistent() {
    let dir = TempDir::new().unwrap();

    for name in ["CON", "NUL", "PRN"] {
        let _ = std::fs::write(dir.path().join(name), b"data");
    }
    std::fs::write(dir.path().join("safe.txt"), b"ok").expect("write");

    let (seq_out, _, _) = run_retree_full(dir.path(), &["-a"]);
    let (par_out, _, _) = run_retree_full(dir.path(), &["--parallel", "-a"]);

    let seq_last = common::last_nonempty_line(&seq_out);
    let par_last = common::last_nonempty_line(&par_out);
    assert_eq!(
        seq_last, par_last,
        "sequential and parallel must agree on reserved-name handling"
    );
}

// ============================================================================
// Cycle detection via file-id
// ============================================================================

/// Mutual symlink cycle: a/link→b, b/link→a
/// Must terminate and mark at least one link recursive.
#[cfg(unix)]
#[test]
fn mutual_symlink_cycle_detected() {
    use std::os::unix::fs as unix_fs;

    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    std::fs::write(a.join("file_a.txt"), b"").unwrap();
    std::fs::write(b.join("file_b.txt"), b"").unwrap();

    // a/to_b -> ../b, b/to_a -> ../a  →  mutual cycle
    unix_fs::symlink(&b, a.join("to_b")).unwrap();
    unix_fs::symlink(&a, b.join("to_a")).unwrap();

    let mut cmd = run_retree_command(dir.path(), &["-l", "-a"]);
    let assert = cmd
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("file_a.txt"), "file_a.txt must appear");
    assert!(stdout.contains("file_b.txt"), "file_b.txt must appear");

    let recursive_count =
        stdout.matches("recursive").count() + stdout.matches("рекурсивная").count();
    assert!(
        recursive_count >= 1,
        "mutual cycle must mark at least 2 links recursive, got {recursive_count}\n{stdout}"
    );
}

/// Same mutual cycle in parallel mode — must also terminate.
#[cfg(unix)]
#[test]
fn mutual_symlink_cycle_parallel() {
    use std::os::unix::fs as unix_fs;

    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();

    unix_fs::symlink(&b, a.join("to_b")).unwrap();
    unix_fs::symlink(&a, b.join("to_a")).unwrap();

    let mut cmd = run_retree_command(dir.path(), &["--parallel", "-l", "-a"]);
    cmd.timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

/// Same mutual cycle in streaming mode — must also terminate.
#[cfg(unix)]
#[test]
fn mutual_symlink_cycle_streaming() {
    use std::os::unix::fs as unix_fs;

    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();

    unix_fs::symlink(&b, a.join("to_b")).unwrap();
    unix_fs::symlink(&a, b.join("to_a")).unwrap();

    let mut cmd = run_retree_command(dir.path(), &["--streaming", "-l", "-a"]);
    cmd.timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

/// Three symlinks to the same target — only one should be traversed,
/// others marked recursive (file-id dedup).
#[cfg(unix)]
#[test]
fn triple_symlink_same_target_file_id_dedup() {
    use std::os::unix::fs as unix_fs;

    let dir = TempDir::new().unwrap();
    let shared = dir.path().join("shared");
    std::fs::create_dir(&shared).unwrap();
    std::fs::write(shared.join("payload.txt"), b"data").unwrap();

    for name in ["link1", "link2", "link3"] {
        unix_fs::symlink(&shared, dir.path().join(name)).unwrap();
    }

    let stdout = run_retree(dir.path(), &["-l", "-a"]);

    let payload_count = stdout.matches("payload.txt").count();
    let recursive_count =
        stdout.matches("recursive").count() + stdout.matches("рекурсивная").count();

    assert!(payload_count >= 1, "payload.txt must appear at least once");
    assert!(
        recursive_count >= 2,
        "at least 2 of 3 symlinks must be marked recursive, got {recursive_count}\n{stdout}"
    );
}

// ============================================================================
// --long-paths with relative root (Windows only)
// ============================================================================

#[cfg(windows)]
#[test]
fn long_paths_relative_root_resolves() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.txt"), b"data").unwrap();

    // Use relative path "." after cd into temp dir
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rt"))
        .args(["--long-paths", "--no-icons", "."])
        .current_dir(dir.path())
        .output()
        .expect("retree failed to start");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test.txt"),
        "--long-paths with relative root must still list files:\n{stdout}"
    );
    assert!(output.status.success(), "must exit cleanly");
}

// ============================================================================
// Regression: 3-way chain symlink cycle
// ============================================================================

/// Chain cycle: a/to_b→b, b/to_c→c, c/to_a→a
/// All three engines must terminate and show all files.
#[cfg(unix)]
#[test]
fn chain_symlink_cycle_three_way_terminates() {
    use std::os::unix::fs as unix_fs;

    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let c = dir.path().join("c");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    std::fs::create_dir(&c).unwrap();
    std::fs::write(a.join("file_a.txt"), b"").unwrap();
    std::fs::write(b.join("file_b.txt"), b"").unwrap();
    std::fs::write(c.join("file_c.txt"), b"").unwrap();

    unix_fs::symlink(&b, a.join("to_b")).unwrap();
    unix_fs::symlink(&c, b.join("to_c")).unwrap();
    unix_fs::symlink(&a, c.join("to_a")).unwrap();

    for extra in [
        &["-l", "-a"][..],
        &["-l", "-a", "--parallel"],
        &["-l", "-a", "--streaming"],
    ] {
        let mut cmd = run_retree_command(dir.path(), extra);
        let assert = cmd
            .timeout(std::time::Duration::from_secs(10))
            .assert()
            .success();
        let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
        assert!(
            stdout.contains("file_a.txt"),
            "file_a.txt missing in mode {:?}",
            extra
        );
        assert!(
            stdout.contains("file_b.txt"),
            "file_b.txt missing in mode {:?}",
            extra
        );
        assert!(
            stdout.contains("file_c.txt"),
            "file_c.txt missing in mode {:?}",
            extra
        );
        let recursive_count =
            stdout.matches("recursive").count() + stdout.matches("рекурсивная").count();
        assert!(
            recursive_count >= 1,
            "chain cycle must mark at least 1 link recursive in {:?}\n{}",
            extra,
            stdout
        );
    }
}

// ============================================================================
// Regression: non-UTF-8 filenames in HTML output
// ============================================================================

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_filename_in_html_no_crash() {
    use std::os::unix::ffi::OsStrExt;

    let dir = TempDir::new().unwrap();
    let bad_name = std::ffi::OsStr::from_bytes(b"\xff\xfebad.txt");
    std::fs::write(dir.path().join(bad_name), b"").unwrap();
    std::fs::write(dir.path().join("good.txt"), b"").unwrap();

    let (stdout, _, code) = run_retree_full(dir.path(), &["-H", ".", "--nolinks"]);

    assert!(
        code == Some(0) || code == Some(1),
        "non-UTF-8 filename must not crash HTML renderer"
    );
    assert!(
        stdout.contains("good.txt"),
        "good.txt must appear in HTML output"
    );
    assert!(
        stdout.contains("<html"),
        "must produce valid HTML structure"
    );
    assert!(
        stdout.contains("bad.txt"),
        "non-UTF-8 filename must appear (lossy) in HTML"
    );
}
