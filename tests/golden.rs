//! Golden tests: pin exact CLI output for behavior-preserving refactoring.
//!
//! Uses a fixed named directory structure ("golden") so the root name
//! is deterministic. All tests use `--no-icons --lang en -n` via helpers.

mod common;

use std::fs;
use tempfile::tempdir;

/// Standard golden directory:
///
/// ```text
/// golden/
/// ├── Cargo.toml
/// ├── docs/
/// │   └── readme.md
/// └── src/
///     ├── lib.rs
///     └── main.rs
/// ```
fn make_golden() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("golden");
    fs::create_dir(&root).unwrap();

    fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("docs/readme.md"), "# Readme").unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    (tmp, root)
}

// ════════════════════════════════════════════════════════
// TEXT — exact output
// ════════════════════════════════════════════════════════

#[test]
fn golden_text_noreport() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["--noreport"]);
    let expected = "\
golden
├── Cargo.toml
├── docs
│   └── readme.md
└── src
    ├── lib.rs
    └── main.rs
";
    assert_eq!(output, expected);
}

#[test]
fn golden_text_with_report() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &[]);
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines[0], "golden");
    assert_eq!(lines[1], "├── Cargo.toml");
    assert_eq!(lines[2], "├── docs");
    assert_eq!(lines[3], "│   └── readme.md");
    assert_eq!(lines[4], "└── src");
    assert_eq!(lines[5], "    ├── lib.rs");
    assert_eq!(lines[6], "    └── main.rs");

    let report = lines.last().unwrap().trim();
    assert_eq!(report, "2 directories, 4 files");
}

#[test]
fn golden_text_dirs_only() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-d", "--noreport"]);
    let expected = "\
golden
├── docs
└── src
";
    assert_eq!(output, expected);
}

#[test]
fn golden_text_depth_1() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-L", "1", "--noreport"]);
    let expected = "\
golden
├── Cargo.toml
├── docs
└── src
";
    assert_eq!(output, expected);
}

#[test]
fn golden_text_reverse() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-r", "--noreport"]);
    let expected = "\
golden
├── src
│   ├── main.rs
│   └── lib.rs
├── docs
│   └── readme.md
└── Cargo.toml
";
    assert_eq!(output, expected);
}

#[test]
fn golden_text_dirsfirst() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["--dirsfirst", "--noreport"]);
    let expected = "\
golden
├── docs
│   └── readme.md
├── src
│   ├── lib.rs
│   └── main.rs
└── Cargo.toml
";
    assert_eq!(output, expected);
}

#[test]
fn golden_text_full_path() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-f", "--noreport"]);
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[0].ends_with("golden"));
    let cargo_line = lines.iter().find(|l| l.contains("Cargo.toml")).unwrap();
    assert!(
        cargo_line.contains("golden/Cargo.toml") || cargo_line.contains("golden\\Cargo.toml"),
        "-f should show path prefix: {:?}",
        cargo_line
    );
}

#[test]
fn golden_text_noindent() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-i", "--noreport"]);
    assert!(!output.contains('├'));
    assert!(!output.contains('└'));
    assert!(!output.contains('│'));
    for name in [
        "golden",
        "Cargo.toml",
        "docs",
        "readme.md",
        "src",
        "lib.rs",
        "main.rs",
    ] {
        assert!(output.contains(name), "{name} missing with -i");
    }
}

#[test]
fn golden_text_streaming_matches_normal() {
    let (_tmp, root) = make_golden();
    let normal = common::run_rtree(&root, &["--noreport"]);
    let streaming = common::run_rtree(&root, &["--streaming", "--noreport"]);
    assert_eq!(streaming, normal);
}

#[test]
fn golden_text_parallel_same_names() {
    let (_tmp, root) = make_golden();
    let seq = common::run_rtree(&root, &["--noreport"]);
    let par = common::run_rtree(&root, &["--parallel", "--noreport"]);
    let mut seq_names = common::extract_names(&seq);
    let mut par_names = common::extract_names(&par);
    seq_names.sort();
    par_names.sort();
    assert_eq!(seq_names, par_names);
}

// ════════════════════════════════════════════════════════
// JSON — structural
// ════════════════════════════════════════════════════════

#[test]
fn golden_json_full_structure() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-J"]);
    let json: serde_json::Value =
        serde_json::from_str(&output).unwrap_or_else(|e| panic!("JSON parse: {e}\nraw: {output}"));

    let arr = json.as_array().expect("root array");
    assert_eq!(arr.len(), 2, "directory + report");

    let r = &arr[0];
    assert_eq!(r["type"], "directory");
    assert_eq!(r["name"], "golden");
    let contents = r["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 3);

    assert_eq!(contents[0]["name"], "Cargo.toml");
    assert_eq!(contents[0]["type"], "file");

    assert_eq!(contents[1]["name"], "docs");
    assert_eq!(contents[1]["type"], "directory");
    let docs = contents[1]["contents"].as_array().unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0]["name"], "readme.md");

    assert_eq!(contents[2]["name"], "src");
    assert_eq!(contents[2]["type"], "directory");
    let src = contents[2]["contents"].as_array().unwrap();
    assert_eq!(src.len(), 2);
    assert_eq!(src[0]["name"], "lib.rs");
    assert_eq!(src[1]["name"], "main.rs");

    let report = &arr[1];
    assert_eq!(report["type"], "report");
    assert_eq!(report["directories"], 2);
    assert_eq!(report["files"], 4);
}

#[test]
fn golden_json_noreport() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-J", "--noreport"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let arr = json.as_array().unwrap();
    assert!(!arr.iter().any(|e| e["type"] == "report"));
}

#[test]
fn golden_json_dirs_only() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-J", "-d", "--noreport"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    fn has_file(v: &serde_json::Value) -> bool {
        if v["type"] == "file" {
            return true;
        }
        v["contents"]
            .as_array()
            .is_some_and(|c| c.iter().any(has_file))
    }
    assert!(!json.as_array().unwrap().iter().any(has_file));
}

#[test]
fn golden_json_depth_1() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-J", "-L", "1", "--noreport"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let contents = json[0]["contents"].as_array().unwrap();
    // At depth 1: Cargo.toml, docs, src (no nested contents)
    assert_eq!(contents.len(), 3);
    for c in contents {
        if c["type"] == "directory" {
            // Directories at depth limit may have empty or no contents
            let sub = c["contents"].as_array();
            assert!(
                sub.is_none() || sub.unwrap().is_empty(),
                "{} should have no children at -L 1",
                c["name"]
            );
        }
    }
}

#[test]
fn golden_json_parallel_same_names() {
    let (_tmp, root) = make_golden();
    let seq = common::run_rtree(&root, &["-J", "--noreport"]);
    let par = common::run_rtree(&root, &["--parallel", "-J", "--noreport"]);
    let seq_json: serde_json::Value = serde_json::from_str(&seq).unwrap();
    let par_json: serde_json::Value = serde_json::from_str(&par).unwrap();

    let mut seq_names = common::collect_all_names(&seq_json);
    let mut par_names = common::collect_all_names(&par_json);
    seq_names.sort();
    par_names.sort();
    assert_eq!(seq_names, par_names);
}

// ════════════════════════════════════════════════════════
// XML — structural
// ════════════════════════════════════════════════════════

#[test]
fn golden_xml_structure() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-X"]);

    assert!(output.starts_with("<?xml"));
    assert!(output.contains("<tree>"));
    assert!(output.contains("</tree>"));
    assert!(output.contains("name=\"golden\""));
    assert!(output.contains("name=\"Cargo.toml\""));
    assert!(output.contains("name=\"docs\""));
    assert!(output.contains("name=\"readme.md\""));
    assert!(output.contains("name=\"src\""));
    assert!(output.contains("name=\"lib.rs\""));
    assert!(output.contains("name=\"main.rs\""));
    assert!(output.contains("<report>"));
    assert!(output.contains("<directories>2</directories>"));
    assert!(output.contains("<files>4</files>"));
}

#[test]
fn golden_xml_noreport() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-X", "--noreport"]);
    assert!(!output.contains("<report>"));
}

#[test]
fn golden_xml_dirs_only() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-X", "-d", "--noreport"]);
    assert!(output.contains("name=\"docs\""));
    assert!(output.contains("name=\"src\""));
    assert!(!output.contains("name=\"Cargo.toml\""));
    assert!(!output.contains("name=\"readme.md\""));
}

// ════════════════════════════════════════════════════════
// HTML — structural
// ════════════════════════════════════════════════════════

#[test]
fn golden_html_structure() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-H", "."]);

    assert!(output.contains("<!DOCTYPE html>") || output.contains("<!doctype html>"));
    assert!(output.contains("<html"));
    assert!(output.contains("</html>"));
    assert!(output.contains("<body"));
    assert!(output.contains("Cargo.toml"));
    assert!(output.contains("readme.md"));
    assert!(output.contains("lib.rs"));
    assert!(output.contains("main.rs"));
}

#[test]
fn golden_html_has_links_by_default() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-H", "http://example.com"]);
    assert!(output.contains("<a "));
    assert!(output.contains("http://example.com"));
}

#[test]
fn golden_html_nolinks() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-H", ".", "--nolinks"]);
    assert!(!output.contains("<a "));
}

#[test]
fn golden_html_custom_title() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-H", ".", "-T", "My Project"]);
    assert!(output.contains("My Project"));
}

#[test]
fn golden_html_report() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-H", "."]);
    assert!(
        output.contains("2 directories") || output.contains("2 director"),
        "HTML should contain dir count"
    );
    assert!(
        output.contains("4 files") || output.contains("4 file"),
        "HTML should contain file count"
    );
}

// ════════════════════════════════════════════════════════
// TEXT — additional edge cases
// ════════════════════════════════════════════════════════

#[test]
fn golden_text_empty_dir() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("empty");
    fs::create_dir(&root).unwrap();

    let output = common::run_rtree(&root, &["--noreport"]);
    assert_eq!(output, "empty\n");
}

#[test]
fn golden_text_single_file() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("single");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("only.txt"), "").unwrap();

    let output = common::run_rtree(&root, &["--noreport"]);
    let expected = "\
single
└── only.txt
";
    assert_eq!(output, expected);
}

#[test]
fn golden_text_pattern_include() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-P", "*.rs", "--noreport"]);
    // -P shows matching files, all directories are still shown
    assert!(output.contains("lib.rs"));
    assert!(output.contains("main.rs"));
    assert!(!output.contains("Cargo.toml"));
    assert!(!output.contains("readme.md"));
    assert!(output.contains("src"), "parent dir of matching files shown");
}

#[test]
fn golden_text_exclude() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-I", "docs", "--noreport"]);
    assert!(!output.contains("docs"));
    assert!(!output.contains("readme.md"));
    assert!(output.contains("src"));
    assert!(output.contains("Cargo.toml"));
}

#[test]
fn golden_text_prune_empty_dirs() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("prunetest");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("hollow")).unwrap();
    fs::create_dir(root.join("filled")).unwrap();
    fs::write(root.join("filled/data.txt"), "x").unwrap();

    let output = common::run_rtree(&root, &["--prune", "--noreport"]);
    assert!(!output.contains("hollow"));
    assert!(output.contains("filled"));
    assert!(output.contains("data.txt"));
}

#[test]
fn golden_text_classify() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-F", "--noreport"]);
    assert!(output.contains("docs/"), "dirs get / suffix with -F");
    assert!(output.contains("src/"), "dirs get / suffix with -F");
    // Files without executable bit don't get suffix
    assert!(
        output.contains("Cargo.toml") && !output.contains("Cargo.toml*"),
        "non-exec files have no suffix"
    );
}

#[test]
fn golden_text_report_empty() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("reportempty");
    fs::create_dir(&root).unwrap();

    let output = common::run_rtree(&root, &[]);
    assert!(output.contains("0 directories, 0 files"));
}

#[test]
fn golden_text_report_singular() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("singular");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("one_dir")).unwrap();
    fs::write(root.join("one_dir/one_file.txt"), "").unwrap();

    let output = common::run_rtree(&root, &[]);
    assert!(output.contains("1 directory, 1 file"));
}

#[test]
fn golden_text_with_report_exact() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &[]);
    let expected = "\
golden
├── Cargo.toml
├── docs
│   └── readme.md
└── src
    ├── lib.rs
    └── main.rs

2 directories, 4 files
";
    assert_eq!(output, expected);
}

// ════════════════════════════════════════════════════════
// JSON — additional edge cases
// ════════════════════════════════════════════════════════

#[test]
fn golden_json_empty_dir() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("jempty");
    fs::create_dir(&root).unwrap();

    let output = common::run_rtree(&root, &["-J"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr[0]["type"], "directory");
    assert_eq!(arr[0]["name"], "jempty");
    let report = arr.last().unwrap();
    assert_eq!(report["directories"], 0);
    assert_eq!(report["files"], 0);
}

#[test]
fn golden_json_exclude() {
    let (_tmp, root) = make_golden();
    let output = common::run_rtree(&root, &["-J", "-I", "docs", "--noreport"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let names = common::collect_all_names(&json);
    assert!(!names.contains(&"docs".to_string()));
    assert!(!names.contains(&"readme.md".to_string()));
    assert!(names.contains(&"src".to_string()));
}

// ════════════════════════════════════════════════════════
// XML — additional edge cases
// ════════════════════════════════════════════════════════

#[test]
fn golden_xml_empty_dir() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("xempty");
    fs::create_dir(&root).unwrap();

    let output = common::run_rtree(&root, &["-X"]);
    assert!(output.contains("<?xml"));
    assert!(output.contains("name=\"xempty\""));
    assert!(output.contains("<directories>0</directories>"));
    assert!(output.contains("<files>0</files>"));
}

#[test]
fn golden_xml_escapes_special_chars() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("xmlesc");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("Tom & Jerry.txt"), "").unwrap();

    let output = common::run_rtree(&root, &["-X", "--noreport"]);
    assert!(output.contains("Tom &amp; Jerry.txt"));
    assert!(!output.contains("Tom & Jerry.txt"));
}
