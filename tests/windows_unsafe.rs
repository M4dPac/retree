//! Integration tests for Windows-specific unsafe code paths.
//!
//! Exercises the public `platform::*` API that wraps Win32 FFI.
//! Only compiled on Windows (`#![cfg(windows)]`).
//!
//! Coverage map:
//!   platform::get_file_id           → attributes.rs  UNSAFE-A1
//!   platform::get_file_attributes_raw → attributes.rs  UNSAFE-A2
//!   platform::enable_ansi           → console.rs     UNSAFE-C1
//!   platform::is_tty                → console.rs     UNSAFE-C2
//!   platform::detect_system_language_id → locale.rs  UNSAFE-L1
//!   platform::get_file_owner        → permissions.rs UNSAFE-P1,P2,P3
//!   platform::get_file_group        → permissions.rs UNSAFE-P4
//!   platform::get_junction_target   → reparse.rs     UNSAFE-R1
//!   platform::get_alternate_streams → streams.rs     UNSAFE-S1
//!   platform::to_long_path          → mod.rs         (safe, regression)

#![cfg(windows)]

use std::fs;
use std::path::Path;
use std::process::Command;

// ═══════════════════════════════════════════════════════
// Attributes  (UNSAFE-A1, UNSAFE-A2)
// ═══════════════════════════════════════════════════════

#[test]
fn win_file_id_for_existing_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("test.txt");
    fs::write(&file, b"content").expect("write");

    let info = rtree::platform::get_file_id(&file);
    assert!(info.is_some());
    let info = info.expect("some");
    assert!(info.file_id > 0);
    assert!(info.volume_serial > 0);
    assert!(info.number_of_links >= 1);
}

#[test]
fn win_file_id_hardlinks() {
    let dir = tempfile::tempdir().expect("tempdir");
    let orig = dir.path().join("orig.txt");
    let link = dir.path().join("link.txt");
    fs::write(&orig, b"shared").expect("write");
    fs::hard_link(&orig, &link).expect("hard_link");

    let a = rtree::platform::get_file_id(&orig).expect("id orig");
    let b = rtree::platform::get_file_id(&link).expect("id link");
    assert_eq!(a.file_id, b.file_id);
    assert!(a.number_of_links >= 2);
}

#[test]
fn win_file_id_nonexistent_returns_none() {
    let info = rtree::platform::get_file_id(Path::new(r"C:\__no_such__\f.txt"));
    assert!(info.is_none());
}

#[test]
fn win_raw_attributes_directory_bit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let attrs = rtree::platform::get_file_attributes_raw(dir.path());
    assert!(attrs.is_some());
    // FILE_ATTRIBUTE_DIRECTORY = 0x10
    assert_ne!(attrs.expect("some") & 0x10, 0);
}

#[test]
fn win_raw_attributes_file_no_dir_bit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("f.txt");
    fs::write(&f, b"x").expect("write");
    let attrs = rtree::platform::get_file_attributes_raw(&f).expect("attrs");
    assert_eq!(attrs & 0x10, 0);
}

// ═══════════════════════════════════════════════════════
// Console  (UNSAFE-C1, UNSAFE-C2)
// ═══════════════════════════════════════════════════════

#[test]
fn win_enable_ansi_no_panic() {
    rtree::platform::enable_ansi();
}

#[test]
fn win_is_tty_no_panic() {
    // CI has piped stdout → typically false, but must not crash
    let _ = rtree::platform::is_tty();
}

// ═══════════════════════════════════════════════════════
// Locale  (UNSAFE-L1)
// ═══════════════════════════════════════════════════════

#[test]
fn win_language_id_nonzero() {
    let id = rtree::platform::detect_system_language_id();
    assert!(id.is_some());
    assert_ne!(id.expect("some"), 0);
}

// ═══════════════════════════════════════════════════════
// Permissions  (UNSAFE-P1, P2, P3, P4)
// ═══════════════════════════════════════════════════════

#[test]
fn win_file_owner_resolves() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("owned.txt");
    fs::write(&f, b"x").expect("write");

    let owner = rtree::platform::get_file_owner(&f);
    assert!(owner.is_some(), "owner must resolve");
    assert!(!owner.expect("some").is_empty());
}

#[test]
fn win_file_owner_nonexistent() {
    let owner = rtree::platform::get_file_owner(Path::new(r"C:\__no_such__\x.txt"));
    assert!(owner.is_none());
}

#[test]
fn win_file_group_resolves_or_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("g.txt");
    fs::write(&f, b"x").expect("write");

    if let Some(g) = rtree::platform::get_file_group(&f) {
        assert!(!g.is_empty());
    }
    // None is acceptable on some Windows configs
}

// ═══════════════════════════════════════════════════════
// Junctions / Reparse  (UNSAFE-R1)
// ═══════════════════════════════════════════════════════

#[test]
fn win_junction_target_none_for_regular_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(rtree::platform::get_junction_target(dir.path()).is_none());
}

#[test]
fn win_junction_target_resolves() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("real");
    let junc = dir.path().join("junc");
    fs::create_dir(&target).expect("mkdir");

    let out = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junc)
        .arg(&target)
        .output()
        .expect("mklink");
    assert!(out.status.success(), "mklink /J failed");

    let resolved = rtree::platform::get_junction_target(&junc);
    assert!(resolved.is_some(), "must resolve junction target");
}

#[test]
fn win_junction_has_reparse_attribute() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("real");
    let junc = dir.path().join("junc");
    fs::create_dir(&target).expect("mkdir");

    let out = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junc)
        .arg(&target)
        .output()
        .expect("mklink");
    assert!(out.status.success());

    let attrs = rtree::platform::get_file_attributes_raw(&junc).expect("attrs");
    // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
    assert_ne!(attrs & 0x400, 0);
}

// ═══════════════════════════════════════════════════════
// Symlinks
// ═══════════════════════════════════════════════════════

#[test]
fn win_symlink_file_detected_as_reparse() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    fs::write(&target, b"data").expect("write");

    match std::os::windows::fs::symlink_file(&target, &link) {
        Ok(_) => {
            let attrs = rtree::platform::get_file_attributes_raw(&link).expect("attrs");
            assert_ne!(attrs & 0x400, 0, "symlink must be reparse point");
        }
        Err(e) => {
            eprintln!("symlink_file unavailable (no Developer Mode?): {e}");
        }
    }
}

#[test]
fn win_symlink_dir_detected_as_reparse() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("tdir");
    let link = dir.path().join("ldir");
    fs::create_dir(&target).expect("mkdir");

    match std::os::windows::fs::symlink_dir(&target, &link) {
        Ok(_) => {
            let attrs = rtree::platform::get_file_attributes_raw(&link).expect("attrs");
            assert_ne!(attrs & 0x400, 0);
        }
        Err(e) => {
            eprintln!("symlink_dir unavailable: {e}");
        }
    }
}

// ═══════════════════════════════════════════════════════
// Long Paths  (to_long_path — safe code, regression)
// ═══════════════════════════════════════════════════════

#[test]
fn win_long_path_prefix_regular() {
    let p = Path::new(r"C:\Users\test\file.txt");
    let lp = rtree::platform::to_long_path(p, true);
    assert!(lp.to_string_lossy().starts_with(r"\\?\"));
}

#[test]
fn win_long_path_disabled() {
    let p = Path::new(r"C:\Users\test\file.txt");
    let lp = rtree::platform::to_long_path(p, false);
    assert_eq!(lp, p);
}

#[test]
fn win_long_path_no_double_prefix() {
    let p = Path::new(r"\\?\C:\Data");
    let lp = rtree::platform::to_long_path(p, true);
    assert_eq!(lp, p, "must not double-prefix");
}

#[test]
fn win_long_path_device_path_unchanged() {
    let p = Path::new(r"\\.\PhysicalDrive0");
    let lp = rtree::platform::to_long_path(p, true);
    assert_eq!(lp, p, "device paths must pass through");
}

#[test]
fn win_long_path_unc() {
    let p = Path::new(r"\\server\share\dir");
    let lp = rtree::platform::to_long_path(p, true);
    let s = lp.to_string_lossy();
    assert!(
        s.starts_with(r"\\?\UNC\"),
        "UNC must become \\\\?\\UNC\\, got: {s}"
    );
    assert!(s.contains("server"), "must preserve server name");
}

#[test]
fn win_long_path_relative_unchanged() {
    let p = Path::new(r"relative\path");
    let lp = rtree::platform::to_long_path(p, true);
    assert_eq!(lp, p);
}

#[test]
fn win_long_path_deep_nesting_file_ops() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut deep = dir.path().to_path_buf();

    // Build path > 260 chars
    let segment = "a".repeat(50);
    for _ in 0..6 {
        deep = deep.join(&segment);
    }
    assert!(
        deep.to_string_lossy().len() > 260,
        "path must exceed MAX_PATH"
    );

    let long = rtree::platform::to_long_path(&deep, true);

    match fs::create_dir_all(&long) {
        Ok(_) => {
            let file = long.join("test.txt");
            fs::write(&file, b"long path content").expect("write to long path");
            let content = fs::read(&file).expect("read from long path");
            assert_eq!(content, b"long path content");
        }
        Err(e) => {
            eprintln!("Long path ops failed (LongPathsEnabled?): {e}");
        }
    }
}

// ═══════════════════════════════════════════════════════
// ADS  (UNSAFE-S1) — additional integration coverage
// ═══════════════════════════════════════════════════════

#[test]
fn win_ads_empty_for_plain_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("plain.txt");
    fs::write(&f, b"hello").expect("write");

    let streams = rtree::platform::get_alternate_streams(&f);
    assert!(streams.is_empty());
}

#[test]
fn win_ads_creation_and_enum() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("ads.txt");
    fs::write(&f, b"body").expect("write");

    let ads_path = format!("{}:secret", f.display());
    match fs::write(&ads_path, b"hidden_data") {
        Ok(_) => {
            let streams = rtree::platform::get_alternate_streams(&f);
            assert_eq!(streams.len(), 1);
            assert_eq!(streams[0].name, "secret");
            assert_eq!(streams[0].size, 11);
        }
        Err(e) => {
            eprintln!("ADS write failed (non-NTFS?): {e}");
        }
    }
}

#[test]
fn win_ads_nonexistent_returns_empty() {
    let streams = rtree::platform::get_alternate_streams(Path::new(r"C:\__no_such_42__"));
    assert!(streams.is_empty());
}

// ═══════════════════════════════════════════════════════
// Executable detection  (safe, regression)
// ═══════════════════════════════════════════════════════

#[test]
fn win_is_executable_by_extension() {
    for ext in &["exe", "bat", "cmd", "com", "ps1", "vbs", "js", "msi"] {
        let p = format!("test.{ext}");
        assert!(
            rtree::platform::is_executable(Path::new(&p)),
            "{ext} should be executable"
        );
    }
}

#[test]
fn win_is_not_executable_by_extension() {
    for ext in &["txt", "rs", "md", "toml", "lock", "yml"] {
        let p = format!("test.{ext}");
        assert!(
            !rtree::platform::is_executable(Path::new(&p)),
            "{ext} should NOT be executable"
        );
    }
}

#[test]
fn win_no_extension_not_executable() {
    assert!(!rtree::platform::is_executable(Path::new("no_ext")));
}

// ═══════════════════════════════════════════════════════
// Handle leak stress test
// ═══════════════════════════════════════════════════════

#[test]
fn win_handle_leak_stress_attributes() {
    // Call get_file_id + get_file_attributes_raw many times.
    // If handles leak, we hit the per-process limit (~16k).
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("stress.txt");
    fs::write(&f, b"x").expect("write");

    for _ in 0..2000 {
        let _ = rtree::platform::get_file_id(&f);
        let _ = rtree::platform::get_file_attributes_raw(&f);
    }
}

#[test]
fn win_handle_leak_stress_permissions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("stress_perm.txt");
    fs::write(&f, b"x").expect("write");

    for _ in 0..1000 {
        let _ = rtree::platform::get_file_owner(&f);
        let _ = rtree::platform::get_file_group(&f);
    }
}

#[test]
fn win_handle_leak_stress_junction() {
    let dir = tempfile::tempdir().expect("tempdir");
    for _ in 0..1000 {
        let _ = rtree::platform::get_junction_target(dir.path());
    }
}

#[test]
fn win_handle_leak_stress_streams() {
    let dir = tempfile::tempdir().expect("tempdir");
    let f = dir.path().join("stress_ads.txt");
    fs::write(&f, b"x").expect("write");

    for _ in 0..1000 {
        let _ = rtree::platform::get_alternate_streams(&f);
    }
}
