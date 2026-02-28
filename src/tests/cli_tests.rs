use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tree_rs() -> Command {
    Command::cargo_bin("rtree").unwrap()
}

#[test]
fn test_basic_output() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Create test structure
    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file1.txt"), "content").unwrap();
    fs::write(dir_path.join("subdir/file2.txt"), "content").unwrap();

    tree_rs()
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_dirs_only() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("file.txt"), "content").unwrap();

    tree_rs()
        .arg("-d")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_depth_limit() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir_all(dir_path.join("level1/level2/level3")).unwrap();

    tree_rs()
        .args(["-L", "1"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2").not());
}

#[test]
fn test_hidden_files() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join(".hidden"), "content").unwrap();
    fs::write(dir_path.join("visible.txt"), "content").unwrap();

    // Without -a, hidden files should not appear
    tree_rs()
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("visible.txt"))
        .stdout(predicate::str::contains(".hidden").not());

    // With -a, hidden files should appear
    tree_rs()
        .arg("-a")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden"));
}

#[test]
fn test_pattern_filter() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.rs"), "").unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();
    fs::write(dir_path.join("other.rs"), "").unwrap();

    tree_rs()
        .args(["-P", "*.rs"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("other.rs"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_exclude_pattern() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.rs"), "").unwrap();
    fs::write(dir_path.join("file.txt"), "").unwrap();

    tree_rs()
        .args(["-I", "*.txt"])
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file.rs"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_json_output() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    let output = tree_rs().arg("-J").arg(dir_path).assert().success();

    // Verify it's valid JSON
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let _: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
}

#[test]
fn test_noreport() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::write(dir_path.join("file.txt"), "content").unwrap();

    tree_rs()
        .arg("--noreport")
        .arg(dir_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("directories").not())
        .stdout(predicate::str::contains("files").not());
}

#[test]
fn test_nonexistent_path() {
    tree_rs().arg("/nonexistent/path").assert().failure();
}
