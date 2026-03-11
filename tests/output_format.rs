/// -o, -H, -T, --nolinks, --hintro/--houtro, -X, -J
mod common;
use common::rtree;

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// -o, --output  (output to file)
// ============================================================================

#[test]
fn test_output_file_created_with_content() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    let output_path = p.join("output.txt");

    fs::write(p.join("file.txt"), "content").unwrap();

    rtree()
        .args(["-o", output_path.to_str().unwrap()])
        .arg(p)
        .assert()
        .success();

    assert!(output_path.exists(), "Output file should be created");
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(!content.is_empty(), "Output file should not be empty");
    assert!(
        content.contains("file.txt"),
        "Output file should contain tree listing. Got:\n{}",
        content
    );
}

// ============================================================================
// -H, --html  (HTML output)
// ============================================================================

#[test]
fn test_html_output_structure() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(stdout.contains("<!DOCTYPE html>") || stdout.contains("<!doctype html>"));
    assert!(stdout.contains("<html"));
    assert!(stdout.contains("<body"));
    assert!(stdout.contains("</html>"));
}

#[test]
fn test_html_contains_links_by_default() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(
        stdout.contains("<a "),
        "HTML output should contain hyperlinks by default. Got:\n{}",
        stdout
    );
}

// ============================================================================
// -T, --title  (HTML page title)
// ============================================================================

#[test]
fn test_html_title() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost", "-T", "My Custom Title"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(
        stdout.contains("My Custom Title"),
        "HTML should contain custom title"
    );
}

// ============================================================================
// --nolinks  (disable hyperlinks in HTML)
// ============================================================================

#[test]
fn test_nolinks_removes_hyperlinks() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree()
        .args(["-H", "http://localhost", "--nolinks"])
        .arg(p)
        .assert()
        .success();

    let stdout = common::output_stdout(&output);
    assert!(
        !stdout.contains("<a "),
        "With --nolinks, HTML should not contain <a> tags. Got:\n{}",
        stdout
    );
}

// ============================================================================
// --hintro / --houtro  (custom HTML intro/outro files)
// ============================================================================

#[test]
fn test_html_intro_file() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let intro_path = p.join("intro.html");
    fs::write(&intro_path, "<!-- CUSTOM INTRO MARKER -->").unwrap();
    fs::write(p.join("file.txt"), "content").unwrap();

    rtree()
        .args([
            "-H",
            "http://localhost",
            "--hintro",
            intro_path.to_str().unwrap(),
        ])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("CUSTOM INTRO MARKER"));
}

#[test]
fn test_html_outro_file() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let outro_path = p.join("outro.html");
    fs::write(&outro_path, "<!-- CUSTOM OUTRO MARKER -->").unwrap();
    fs::write(p.join("file.txt"), "content").unwrap();

    rtree()
        .args([
            "-H",
            "http://localhost",
            "--houtro",
            outro_path.to_str().unwrap(),
        ])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("CUSTOM OUTRO MARKER"));
}

// ============================================================================
// -X, --xml  (XML output)
// ============================================================================

#[test]
fn test_xml_output_structure() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "content").unwrap();

    let output = rtree().arg("-X").arg(p).assert().success();

    let stdout = common::output_stdout(&output);
    assert!(
        stdout.starts_with("<?xml"),
        "XML should start with <?xml declaration"
    );
    assert!(stdout.contains("<tree>"), "XML should contain <tree> tag");
    assert!(
        stdout.contains("</tree>"),
        "XML should contain closing </tree>"
    );
    assert!(
        stdout.contains("file.txt"),
        "XML should contain file entries"
    );
}

// ============================================================================
// -J, --json  (JSON output)
// ============================================================================

#[test]
fn test_json_output_structure() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "content").unwrap();
    fs::write(p.join("subdir/nested.txt"), "content").unwrap();

    let output = rtree()
        .args(["-J", "--lang", "en"])
        .arg(p)
        .assert()
        .success();

    let json: serde_json::Value = common::output_json(&output);

    let arr = json.as_array().expect("Root should be array");
    assert!(arr.len() >= 2, "Should have at least directory + report");

    let root = &arr[0];
    assert_eq!(root["type"].as_str(), Some("directory"));
    assert!(root["name"].as_str().is_some());

    let contents = root["contents"]
        .as_array()
        .expect("Root directory should have contents");

    let file_entry = contents
        .iter()
        .find(|e| e["name"].as_str() == Some("file.txt"))
        .expect("file.txt should be in root contents");
    assert_eq!(file_entry["type"].as_str(), Some("file"));

    let subdir_entry = contents
        .iter()
        .find(|e| e["name"].as_str() == Some("subdir"))
        .expect("subdir should be in root contents");
    assert_eq!(subdir_entry["type"].as_str(), Some("directory"));

    let subdir_contents = subdir_entry["contents"]
        .as_array()
        .expect("subdir should have contents");
    assert!(
        subdir_contents
            .iter()
            .any(|e| e["name"].as_str() == Some("nested.txt")),
        "nested.txt should be inside subdir"
    );

    let report = arr.last().unwrap();
    assert_eq!(report["type"].as_str(), Some("report"));
    assert!(report["directories"].is_number());
    assert!(report["files"].is_number());
}

#[test]
fn test_json_hierarchy_correct() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir_all(p.join("a/b")).unwrap();
    fs::write(p.join("a/b/deep.txt"), "").unwrap();
    fs::write(p.join("root.txt"), "").unwrap();

    let output = rtree().args(["-J"]).arg(p).assert().success();

    let json: serde_json::Value = common::output_json(&output);
    let root_contents = json[0]["contents"].as_array().unwrap();

    assert!(root_contents
        .iter()
        .any(|e| e["name"].as_str() == Some("root.txt")));

    assert!(!root_contents
        .iter()
        .any(|e| e["name"].as_str() == Some("deep.txt")));

    let a = root_contents
        .iter()
        .find(|e| e["name"].as_str() == Some("a"))
        .unwrap();
    let b = a["contents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"].as_str() == Some("b"))
        .unwrap();
    assert!(b["contents"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["name"].as_str() == Some("deep.txt")));
}

#[test]
fn test_json_noreport() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-J", "--noreport"]).arg(p).assert().success();

    let json: serde_json::Value = common::output_json(&output);
    let arr = json.as_array().unwrap();

    let has_report = arr.iter().any(|e| e["type"].as_str() == Some("report"));
    assert!(
        !has_report,
        "JSON with --noreport should not contain report entry"
    );
}

#[test]
fn test_json_dirs_only() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    let output = rtree().args(["-J", "-d"]).arg(p).assert().success();

    let json: serde_json::Value = common::output_json(&output);

    // inline count to avoid importing common helper for just one test
    fn count_files(v: &serde_json::Value) -> u64 {
        let mut n = 0u64;
        if let Some(arr) = v.as_array() {
            for item in arr {
                count_files_rec(item, &mut n);
            }
        }
        n
    }
    fn count_files_rec(e: &serde_json::Value, n: &mut u64) {
        if e.get("type").and_then(|t| t.as_str()) == Some("file") {
            *n += 1;
        }
        if let Some(c) = e.get("contents").and_then(|c| c.as_array()) {
            for child in c {
                count_files_rec(child, n);
            }
        }
    }

    let files = count_files(&json);
    assert_eq!(
        files, 0,
        "JSON with -d should contain no files, got {}",
        files
    );
}
