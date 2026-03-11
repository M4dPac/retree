/// -i, -A/-S, цвет, -s/-h/--si, -D/--timefmt, -p/-u/-g, --inodes, --device, -F, -q/-N, --charset, иконки, Windows-флаги, отступы
mod common;
use common::{rtree, CLEAN};

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// -i, --noindent  (no tree indentation)
// ============================================================================

#[test]
fn test_no_indent_removes_tree_chars() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("subdir/file.txt"), "").unwrap();

    let output = rtree().args(["-i"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);

    assert!(!stdout.contains('├'), "-i should remove ├");
    assert!(!stdout.contains('└'), "-i should remove └");
    assert!(!stdout.contains('│'), "-i should remove │");
    assert!(
        !stdout.contains("|--") && !stdout.contains("`--"),
        "-i should remove ASCII tree chars"
    );
}

// ============================================================================
// -A, --ansi / -S, --cp437  (line graphics style)
// ============================================================================

#[test]
fn test_ansi_line_graphics() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("file2.txt"), "").unwrap();

    let output = rtree()
        .args(["-A", "-n", "--no-icons", "--lang", "en"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    let has_unicode_tree = stdout.contains('├') || stdout.contains('└') || stdout.contains('│');
    assert!(
        has_unicode_tree,
        "With -A, output should use Unicode box-drawing chars. Got:\n{}",
        stdout
    );
}

#[test]
fn test_cp437_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-S").arg(dir.path()).assert().success();
}

// ============================================================================
// -n, --nocolor / -C, --color-always / --color=<WHEN>
// ============================================================================

#[test]
fn test_no_color_strips_ansi_escapes() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-n"]).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        !stdout.contains("\x1b["),
        "With -n, output must not contain ANSI escape codes. Got:\n{:?}",
        stdout
    );
}

#[test]
fn test_color_always_forces_ansi_escapes() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-C"]).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        stdout.contains("\x1b["),
        "With -C, output should contain ANSI escapes even in pipe. Got:\n{:?}",
        stdout
    );
}

#[test]
fn test_color_never() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    let output = rtree()
        .args(["--color=never"])
        .arg(dir.path())
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(
        !stdout.contains("\x1b["),
        "With --color=never, no ANSI escapes"
    );
}

#[test]
fn test_color_always_via_flag() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["--color=always"]).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        stdout.contains("\x1b["),
        "With --color=always, should have ANSI escapes"
    );
}

#[test]
fn test_color_auto_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--color=auto"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_color_invalid_value() {
    rtree().args(["--color=invalid", "."]).assert().failure();
}

#[test]
fn test_no_color_overrides_color_always() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-C", "-n"]).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        !stdout.contains("\x1b["),
        "With both -C and -n, -n should win (no ANSI). Got:\n{:?}",
        stdout
    );
}

#[test]
fn test_no_color_env_variable() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().env("NO_COLOR", "1").arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        !stdout.contains("\x1b["),
        "With NO_COLOR=1, output must not contain ANSI escapes"
    );
}

// ============================================================================
// -s, --size  (print size in bytes)
// ============================================================================

#[test]
fn test_size_shows_bytes() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap(); // 7 bytes

    let output = rtree().args(["-s"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        file_line.contains("7"),
        "With -s, file line should show size 7. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -h, --human  (human readable sizes)
// ============================================================================

#[test]
fn test_human_readable_shows_units() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("big.txt"), "x".repeat(2048)).unwrap();

    let output = rtree().args(["-h"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("big.txt"))
        .expect("big.txt not found");

    assert!(
        file_line.contains('K') || file_line.contains('M') || file_line.contains('G'),
        "With -h, large file should show human unit (K/M/G). Got: {:?}",
        file_line
    );
}

// ============================================================================
// --si  (SI units, powers of 1000)
// ============================================================================

#[test]
fn test_si_units_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "x".repeat(2048)).unwrap();

    rtree().arg("--si").arg(dir.path()).assert().success();
}

// ============================================================================
// -D, --date  (print modification date)
// ============================================================================

#[test]
fn test_date_shows_timestamp() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-D"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        predicate::str::is_match(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}")
            .unwrap()
            .eval(&stdout),
        "With -D, output should contain date in YYYY-MM-DD HH:MM format. Got:\n{}",
        stdout
    );
}

// ============================================================================
// --timefmt  (custom time format)
// ============================================================================

#[test]
fn test_timefmt_custom_format() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-D", "--timefmt", "%Y/%m/%d"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);

    assert!(
        predicate::str::is_match(r"\d{4}/\d{2}/\d{2}")
            .unwrap()
            .eval(&stdout),
        "Custom timefmt should produce YYYY/MM/DD format. Got:\n{}",
        stdout
    );

    let file_line = stdout.lines().find(|l| l.contains("file.txt")).unwrap();
    assert!(
        !predicate::str::is_match(r"\d{2}:\d{2}")
            .unwrap()
            .eval(file_line),
        "Custom timefmt %Y/%m/%d should not include time. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -p, --perm  (print permissions)
// ============================================================================

#[test]
fn test_permissions_shows_perm_string() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-p"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    // TODO: After implementing POSIX-style permissions on Windows (via ACL),
    // consider unifying the check or adding a separate Windows-specific test.
    // Unix: rwxr-xr-x format, Windows: [RHSACE] attribute format
    let has_perms = file_line.contains("rw")
        || file_line.contains("r-")
        || file_line.contains("RW")
        || file_line.contains("R-")
        || file_line.contains("RA")
        || file_line.contains('[');
    assert!(
        has_perms,
        "With -p, file line should contain permissions. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -u, --uid  (print file owner)
// ============================================================================

#[test]
fn test_uid_shows_owner() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-u"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        file_line.len() > "└── file.txt".len(),
        "With -u, file line should include owner info. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -g, --gid  (print file group)
// ============================================================================

#[test]
fn test_gid_shows_group() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().args(["-g"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        file_line.len() > "└── file.txt".len(),
        "With -g, file line should include group info. Got: {:?}",
        file_line
    );
}

// ============================================================================
// --inodes  (print inode number)
// ============================================================================

#[test]
fn test_inodes_shows_number() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["--inodes"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        predicate::str::is_match(r"\d{2,}").unwrap().eval(file_line),
        "With --inodes, line should contain inode number. Got: {:?}",
        file_line
    );
}

// ============================================================================
// --device  (print device number)
// ============================================================================

#[test]
fn test_device_shows_number() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["--device"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    let file_line = stdout
        .lines()
        .find(|l| l.contains("file.txt"))
        .expect("file.txt not found");

    assert!(
        predicate::str::is_match(r"\d+").unwrap().eval(file_line),
        "With --device, line should contain device number. Got: {:?}",
        file_line
    );
}

// ============================================================================
// -F, --classify  (append type indicator)
// ============================================================================

#[test]
fn test_classify_appends_slash_to_dirs() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-F"]).args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        stdout.contains("subdir/"),
        "With -F, directories should have / suffix. Got:\n{}",
        stdout
    );
}

// ============================================================================
// -q, --safe / -N, --literal / --charset
// ============================================================================

#[test]
fn test_safe_print_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-q").arg(dir.path()).assert().success();
}

#[test]
fn test_literal_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-N").arg(dir.path()).assert().success();
}

#[test]
fn test_charset_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--charset", "utf-8"])
        .arg(dir.path())
        .assert()
        .success();
}

// ============================================================================
// Icons
// ============================================================================

#[test]
fn test_icons_auto_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icons=auto"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icons_always_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icons=always"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icons_never_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icons=never"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_no_icons_flag() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("--no-icons").arg(dir.path()).assert().success();
}

#[test]
fn test_no_icons_overrides_icons_always() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--no-icons", "--icons=always"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icon_style_nerd() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icon-style=nerd"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icon_style_unicode() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icon-style=unicode"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_icon_style_ascii() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--icon-style=ascii"])
        .arg(dir.path())
        .assert()
        .success();
}

// ============================================================================
// Windows-specific flags
// ============================================================================

#[test]
fn test_show_streams_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--show-streams")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_show_junctions_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--show-junctions")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_hide_system_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--hide-system")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_long_paths_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .arg("--long-paths")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_permissions_mode_posix() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--permissions=posix"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_permissions_mode_windows() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--permissions=windows"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_permissions_mode_invalid() {
    rtree()
        .args(["--permissions=invalid", "."])
        .assert()
        .failure();
}

// ============================================================================
// Tree indentation / hierarchy (regression tests)
// ============================================================================

#[test]
fn test_tree_indentation_child_under_dir() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("root_file.txt"), "").unwrap();
    fs::write(p.join("subdir/child_file.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let child_line = stdout
        .lines()
        .find(|l| l.contains("child_file.txt"))
        .expect("child_file.txt not found");

    assert!(
        child_line.contains("│") || child_line.starts_with("    "),
        "child_file.txt should be indented under subdir. Got: {:?}",
        child_line
    );
}

#[test]
fn test_tree_deep_indentation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b/c")).unwrap();
    fs::write(p.join("a/b/c/deep.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let deep_line = stdout
        .lines()
        .find(|l| l.contains("deep.txt"))
        .expect("deep.txt not found");

    let name_pos = deep_line.find("deep.txt").unwrap();
    assert!(
        name_pos >= 12,
        "deep.txt should have >= 3 levels of indentation (offset >= 12). Got offset {}: {:?}",
        name_pos,
        deep_line
    );
}

#[test]
fn test_last_branch_marker_single_child() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("only_child.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let child_line = stdout
        .lines()
        .find(|l| l.contains("only_child.txt"))
        .expect("only_child.txt not found");

    assert!(
        child_line.contains("└── ") || child_line.contains("`-- "),
        "Single child should use last-branch marker (└──). Got: {:?}",
        child_line
    );
}

#[test]
fn test_branch_markers_multiple_children() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("zzz.txt"), "").unwrap();

    let output = rtree().args(CLEAN).arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    let first_line = stdout
        .lines()
        .find(|l| l.contains("aaa.txt"))
        .expect("aaa.txt not found");
    let last_line = stdout
        .lines()
        .find(|l| l.contains("zzz.txt"))
        .expect("zzz.txt not found");

    assert!(
        first_line.contains("├── ") || first_line.contains("|-- "),
        "Non-last child should use branch marker (├──). Got: {:?}",
        first_line
    );
    assert!(
        last_line.contains("└── ") || last_line.contains("`-- "),
        "Last child should use last-branch marker (└──). Got: {:?}",
        last_line
    );
}
