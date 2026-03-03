/// -v, -t, -c, -U, -r, --dirsfirst, --filesfirst, --sort=*
mod common;
use common::{rtree, CLEAN};

use std::fs;
use tempfile::tempdir;

// ============================================================================
// -v, --version-sort  (natural/version sort)
// ============================================================================

#[test]
fn test_version_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("file2.txt"), "").unwrap();
    fs::write(p.join("file10.txt"), "").unwrap();

    let output = rtree().args(["-v"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos1 = stdout.find("file1.txt").unwrap();
    let pos2 = stdout.find("file2.txt").unwrap();
    let pos10 = stdout.find("file10.txt").unwrap();

    assert!(
        pos1 < pos2 && pos2 < pos10,
        "Version sort should order: file1, file2, file10. Positions: {}, {}, {}",
        pos1,
        pos2,
        pos10
    );
}

// ============================================================================
// -t, --timesort  (sort by modification time, newest first)
// ============================================================================

#[test]
fn test_time_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("old.txt"), "old").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(p.join("new.txt"), "new").unwrap();

    let output = rtree().args(["-t"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_new = stdout.find("new.txt").expect("new.txt not found");
    let pos_old = stdout.find("old.txt").expect("old.txt not found");

    assert!(
        pos_new < pos_old,
        "With -t, newest file should appear first. new={}, old={}",
        pos_new,
        pos_old
    );
}

// ============================================================================
// -c, --ctime  (sort by change/creation time)
// ============================================================================

#[test]
fn test_ctime_sort_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-c").arg(dir.path()).assert().success();
}

// ============================================================================
// -U, --unsorted
// ============================================================================

#[test]
fn test_unsorted_accepted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree().arg("-U").arg(dir.path()).assert().success();
}

// ============================================================================
// -r, --reverse  (reverse sort order)
// ============================================================================

#[test]
fn test_reverse_sort_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("zzz.txt"), "").unwrap();

    let output = rtree().args(["-r"]).args(CLEAN).arg(p).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_z = stdout.find("zzz.txt").unwrap();
    let pos_a = stdout.find("aaa.txt").unwrap();

    assert!(
        pos_z < pos_a,
        "With -r, zzz.txt should appear before aaa.txt. z={}, a={}",
        pos_z,
        pos_a
    );
}

// ============================================================================
// --dirsfirst / --filesfirst
// ============================================================================

#[test]
fn test_dirs_first_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("aaa_file.txt"), "").unwrap();
    fs::create_dir(p.join("subdir")).unwrap();

    let output = rtree()
        .arg("--dirsfirst")
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_dir = stdout.find("subdir").unwrap();
    let pos_file = stdout.find("aaa_file.txt").unwrap();

    assert!(
        pos_dir < pos_file,
        "With --dirsfirst, directory should appear before file. dir={}, file={}",
        pos_dir,
        pos_file
    );
}

#[test]
fn test_files_first_order() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("aaa_dir")).unwrap();
    fs::write(p.join("zzz_file.txt"), "").unwrap();

    let output = rtree()
        .arg("--filesfirst")
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_file = stdout.find("zzz_file.txt").unwrap();
    let pos_dir = stdout.find("aaa_dir").unwrap();

    assert!(
        pos_file < pos_dir,
        "With --filesfirst, file should appear before directory. file={}, dir={}",
        pos_file,
        pos_dir
    );
}

// ============================================================================
// --sort=<TYPE>
// ============================================================================

#[test]
fn test_sort_name() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("bbb.txt"), "").unwrap();
    fs::write(p.join("aaa.txt"), "").unwrap();
    fs::write(p.join("ccc.txt"), "").unwrap();

    let output = rtree()
        .args(["--sort=name"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_a = stdout.find("aaa.txt").unwrap();
    let pos_b = stdout.find("bbb.txt").unwrap();
    let pos_c = stdout.find("ccc.txt").unwrap();

    assert!(
        pos_a < pos_b && pos_b < pos_c,
        "sort=name: expected aaa < bbb < ccc. a={}, b={}, c={}",
        pos_a,
        pos_b,
        pos_c
    );
}

#[test]
fn test_sort_size() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("small.txt"), "a").unwrap();
    fs::write(p.join("large.txt"), "a]".repeat(1000)).unwrap();

    let output = rtree()
        .args(["--sort=size"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let pos_small = stdout.find("small.txt").unwrap();
    let pos_large = stdout.find("large.txt").unwrap();

    assert!(
        pos_small < pos_large,
        "sort=size: small.txt should appear before large.txt. small={}, large={}",
        pos_small,
        pos_large
    );
}

#[test]
fn test_sort_mtime() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--sort=mtime"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_sort_ctime() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--sort=ctime"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_sort_version() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("v1.txt"), "").unwrap();
    fs::write(p.join("v2.txt"), "").unwrap();
    fs::write(p.join("v10.txt"), "").unwrap();

    let output = rtree()
        .args(["--sort=version"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let p1 = stdout.find("v1.txt").unwrap();
    let p2 = stdout.find("v2.txt").unwrap();
    let p10 = stdout.find("v10.txt").unwrap();

    assert!(
        p1 < p2 && p2 < p10,
        "sort=version: v1 < v2 < v10. got {}, {}, {}",
        p1,
        p2,
        p10
    );
}

#[test]
fn test_sort_none() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    rtree()
        .args(["--sort=none"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_sort_invalid_value() {
    rtree().args(["--sort=invalid", "."]).assert().failure();
}
