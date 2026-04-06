/// --parallel, --threads, --queue-cap, эквивалентность с sequential
mod common;
use common::{collect_all_names, count_files_and_dirs, extract_names, retree, CLEAN};

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// Basic parallel acceptance
// ============================================================================

#[test]
fn test_parallel_basic() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("subdir/file2.txt"), "").unwrap();

    retree()
        .arg("--parallel")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("subdir"));
}

#[test]
fn test_parallel_with_threads() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    retree()
        .args(["--parallel", "--threads", "4"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_parallel_with_queue_cap() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "").unwrap();

    retree()
        .args(["--parallel", "--queue-cap", "64"])
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_parallel_empty_dir() {
    let dir = tempdir().unwrap();

    retree()
        .arg("--parallel")
        .arg(dir.path())
        .assert()
        .success();
}

// ============================================================================
// Parallel + filtering/depth
// ============================================================================

#[test]
fn test_parallel_with_depth_limit() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("level1/level2/level3")).unwrap();

    retree()
        .args(["--parallel", "-L", "1"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2").not());
}

#[test]
fn test_parallel_dirs_only() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    retree()
        .args(["--parallel", "-d"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_parallel_exclude_filter() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("keep.rs"), "").unwrap();
    fs::write(p.join("skip.txt"), "").unwrap();

    retree()
        .args(["--parallel", "-I", "*.txt"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("keep.rs"))
        .stdout(predicate::str::contains("skip.txt").not());
}

// ============================================================================
// Parallel output formats
// ============================================================================

#[test]
fn test_parallel_json_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = retree()
        .args(["--parallel", "-J"])
        .arg(p)
        .assert()
        .success();

    let json: serde_json::Value = common::output_json(&output);
    assert!(json.is_array());
}

#[test]
fn test_parallel_xml_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = retree()
        .args(["--parallel", "-X"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(stdout.starts_with("<?xml"));
    assert!(stdout.contains("<tree>"));
}

#[test]
fn test_parallel_html_output() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = retree()
        .args(["--parallel", "-H", "http://localhost"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(stdout.contains("<html"));
}

// ============================================================================
// Parallel indentation
// ============================================================================

#[test]
fn test_parallel_indentation() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("subdir/child.txt"), "").unwrap();

    let output = retree()
        .args(["--parallel"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    let child_line = stdout
        .lines()
        .find(|l| l.contains("child.txt"))
        .expect("child.txt not found");

    assert!(
        child_line.contains("│") || child_line.starts_with("    "),
        "child.txt should be indented in parallel mode. Got: {:?}",
        child_line
    );
}

// ============================================================================
// Parallel vs Sequential equivalence
// ============================================================================

#[test]
fn test_parallel_sequential_file_count_match() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("subdir1/nested")).unwrap();
    fs::create_dir_all(p.join("subdir2")).unwrap();

    for i in 0..10 {
        fs::write(p.join(format!("file{}.txt", i)), "").unwrap();
    }
    for i in 0..5 {
        fs::write(p.join(format!("subdir1/file{}.txt", i)), "").unwrap();
    }
    for i in 0..3 {
        fs::write(p.join(format!("subdir2/file{}.txt", i)), "").unwrap();
    }
    fs::write(p.join("subdir1/nested/deep.txt"), "").unwrap();

    let seq = retree().args(["-J"]).arg(p).assert().success();
    let seq_json: serde_json::Value = serde_json::from_slice(&seq.get_output().stdout).unwrap();
    let (seq_files, seq_dirs) = count_files_and_dirs(&seq_json);

    let par = retree()
        .args(["--parallel", "-J"])
        .arg(p)
        .assert()
        .success();
    let par_json: serde_json::Value = serde_json::from_slice(&par.get_output().stdout).unwrap();
    let (par_files, par_dirs) = count_files_and_dirs(&par_json);

    assert_eq!(
        seq_files, par_files,
        "File count mismatch: seq={}, par={}",
        seq_files, par_files
    );
    assert_eq!(
        seq_dirs, par_dirs,
        "Dir count mismatch: seq={}, par={}",
        seq_dirs, par_dirs
    );
}

#[test]
fn test_parallel_sequential_names_match() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for l1 in 0..3 {
        for l2 in 0..3 {
            let sub = p.join(format!("dir{}/sub{}", l1, l2));
            fs::create_dir_all(&sub).unwrap();
            for f in 0..5 {
                fs::write(sub.join(format!("file{}.txt", f)), "").unwrap();
            }
        }
    }

    let seq = retree().args(["-J"]).arg(p).assert().success();
    let seq_json: serde_json::Value = serde_json::from_slice(&seq.get_output().stdout).unwrap();

    let par = retree()
        .args(["--parallel", "-J"])
        .arg(p)
        .assert()
        .success();
    let par_json: serde_json::Value = serde_json::from_slice(&par.get_output().stdout).unwrap();

    let mut seq_names = collect_all_names(&seq_json);
    let mut par_names = collect_all_names(&par_json);
    seq_names.sort();
    par_names.sort();

    assert_eq!(
        seq_names, par_names,
        "Parallel and sequential should list same files"
    );
}

#[test]
fn test_parallel_no_duplicates() {
    use std::collections::HashSet;

    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b/c")).unwrap();
    for i in 0..5 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
        fs::write(p.join(format!("a/f{}.txt", i)), "").unwrap();
        fs::write(p.join(format!("a/b/f{}.txt", i)), "").unwrap();
        fs::write(p.join(format!("a/b/c/f{}.txt", i)), "").unwrap();
    }

    let output = retree()
        .args(["--parallel", "-J"])
        .arg(p)
        .assert()
        .success();
    let json: serde_json::Value = common::output_json(&output);

    fn collect_paths(entry: &serde_json::Value, prefix: &str, paths: &mut Vec<String>) {
        if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
            let full = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };
            paths.push(full.clone());
            if let Some(contents) = entry.get("contents").and_then(|c| c.as_array()) {
                for child in contents {
                    collect_paths(child, &full, paths);
                }
            }
        }
    }

    let mut all_paths = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            collect_paths(item, "", &mut all_paths);
        }
    }

    let unique: HashSet<_> = all_paths.iter().collect();
    assert_eq!(
        all_paths.len(),
        unique.len(),
        "Duplicate entries found in parallel mode"
    );
}

#[test]
fn test_parallel_text_same_names_as_sequential() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("subdir1")).unwrap();
    fs::create_dir_all(p.join("subdir2")).unwrap();
    fs::write(p.join("file1.txt"), "").unwrap();
    fs::write(p.join("subdir1/file2.txt"), "").unwrap();
    fs::write(p.join("subdir2/file3.txt"), "").unwrap();

    let seq = retree().args(CLEAN).arg(p).assert().success();
    let seq_stdout = String::from_utf8(seq.get_output().stdout.clone()).unwrap();

    let par = retree()
        .args(["--parallel"])
        .args(CLEAN)
        .arg(p)
        .assert()
        .success();
    let par_stdout = String::from_utf8(par.get_output().stdout.clone()).unwrap();

    let mut seq_names = extract_names(&seq_stdout);
    let mut par_names = extract_names(&par_stdout);
    seq_names.sort();
    par_names.sort();

    assert_eq!(
        seq_names, par_names,
        "Sequential and parallel should list same entries"
    );
}
