//! Compatibility tests: compare rtree output with system `tree` command.
//!
//! These tests create known directory structures and verify that rtree
//! produces identical output to the system `tree` for all compatible flags.
//!
//! Requirements:
//! - Unix system (entire file is gated with #![cfg(unix)])
//! - `tree` command installed (tests skip gracefully if not found)
//!
//! Usage:
//!   cargo test --test tree_compat                    # run all compat tests
//!   cargo test --test tree_compat -- --nocapture     # show output on failure
//!   cargo test --test tree_compat -- compat_json     # run specific test
#![cfg(all(unix, feature = "tree_compat"))]

use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Helpers
// ============================================================================

/// Check if system `tree` command is available.
fn has_tree() -> bool {
    Command::new("tree")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get rtree binary path (built by cargo test).
fn rtree_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop(); // remove test binary name
    p.pop(); // remove 'deps'
    p.push("rtree");
    assert!(p.exists(), "rtree binary not found at {:?}", p);
    p
}

/// Skip test if tree is not installed.
macro_rules! require_tree {
    () => {
        if !has_tree() {
            eprintln!("SKIPPED: system `tree` command not found");
            return;
        }
    };
}

/// Create standard test directory with rich structure:
///
/// ```text
/// testroot/
/// ├── UPPER.TXT
/// ├── alpha.txt
/// ├── beta.md
/// ├── docs/
/// │   ├── notes.md
/// │   └── readme.txt
/// ├── docs_link -> docs        (symlink)
/// ├── empty/
/// ├── gamma.rs
/// ├── many/                    (6 files, for --filelimit tests)
/// │   ├── f1 .. f6
/// ├── src/
/// │   ├── core/
/// │   │   └── lib.rs
/// │   └── main.rs
/// ├── versions/                (for -v version sort tests)
/// │   ├── file1.txt .. file20.txt
/// ├── .hidden                  (hidden file)
/// └── .config/                 (hidden directory)
///     └── settings.toml
/// ```
fn make_test_dir() -> TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("testroot");
    fs::create_dir(&root).unwrap();

    // Regular files at root level
    fs::write(root.join("alpha.txt"), "hello world").unwrap();
    fs::write(root.join("beta.md"), "# Beta\nContent").unwrap();
    fs::write(root.join("gamma.rs"), "fn main() {}").unwrap();
    fs::write(root.join("UPPER.TXT"), "UPPER CASE").unwrap();

    // Subdirectory with files
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("docs/readme.txt"), "readme content").unwrap();
    fs::write(root.join("docs/notes.md"), "notes here").unwrap();

    // Nested directories
    fs::create_dir_all(root.join("src/core")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/core/lib.rs"), "pub mod core;").unwrap();

    // Empty directory (for --prune tests)
    fs::create_dir(root.join("empty")).unwrap();

    // Directory with many files (for --filelimit tests)
    fs::create_dir(root.join("many")).unwrap();
    for i in 1..=6 {
        fs::write(root.join(format!("many/f{}", i)), format!("file {}", i)).unwrap();
    }

    // Version-numbered files (for -v sort tests)
    fs::create_dir(root.join("versions")).unwrap();
    for name in &[
        "file1.txt",
        "file2.txt",
        "file3.txt",
        "file10.txt",
        "file20.txt",
    ] {
        fs::write(root.join(format!("versions/{}", name)), "").unwrap();
    }

    // Hidden files (for -a tests)
    fs::write(root.join(".hidden"), "secret").unwrap();
    fs::create_dir(root.join(".config")).unwrap();
    fs::write(
        root.join(".config/settings.toml"),
        "[general]\nkey = \"val\"",
    )
    .unwrap();

    // Symlink (for -l tests)
    unix_fs::symlink("docs", root.join("docs_link")).unwrap();

    tmp
}

/// Run system `tree` command.
///
/// Always adds `-n` (no color) for consistent comparison.
/// Uses `current_dir` so both commands get the same relative path.
fn run_tree(dir: &Path, extra_args: &[&str]) -> String {
    let name = dir.file_name().unwrap();
    let parent = dir.parent().unwrap();

    let out = Command::new("tree")
        .current_dir(parent)
        .arg(name)
        .arg("-n") // no color — always, for deterministic output
        .args(extra_args)
        .env("LC_ALL", "en_US.UTF-8")
        .output()
        .expect("failed to execute tree");

    if !out.status.success() {
        panic!(
            "tree failed (status {:?}):\nstderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    String::from_utf8(out.stdout).expect("tree output not UTF-8")
}

/// Run system `tree` command, allowing non-zero exit codes.
///
/// Some tree flags (like --filelimit) return exit code 2 when directories
/// are skipped, which is not an error condition for our comparison.
fn run_tree_lenient(dir: &Path, extra_args: &[&str]) -> String {
    let name = dir.file_name().unwrap();
    let parent = dir.parent().unwrap();

    let out = Command::new("tree")
        .current_dir(parent)
        .arg(name)
        .arg("-n")
        .args(extra_args)
        .env("LC_ALL", "en_US.UTF-8")
        .output()
        .expect("failed to execute tree");

    if !out.status.success() && out.status.code() != Some(2) {
        panic!(
            "tree failed (status {:?}):\nstderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    String::from_utf8(out.stdout).expect("tree output not UTF-8")
}

/// Run rtree binary.
///
/// Always adds `--no-icons --lang en -n` for consistent comparison with tree.
/// - `--no-icons`: tree doesn't have icons
/// - `--lang en`: force English report format
/// - `-n`: no color (same as tree)
fn run_rtree(dir: &Path, extra_args: &[&str]) -> String {
    let name = dir.file_name().unwrap();
    let parent = dir.parent().unwrap();

    let out = Command::new(rtree_path())
        .current_dir(parent)
        .arg(name)
        .args(["--no-icons", "--lang", "en", "-n"])
        .args(extra_args)
        .env("LC_ALL", "en_US.UTF-8")
        .env("TREE_LANG", "en")
        .output()
        .expect("failed to execute rtree");

    if !out.status.success() {
        panic!(
            "rtree failed (status {:?}):\nstderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    String::from_utf8(out.stdout).expect("rtree output not UTF-8")
}

/// Normalize output for comparison:
/// - replace non-breaking spaces (U+00A0) with regular spaces
///   (GNU tree uses NBSP in tree-drawing, rtree uses regular spaces)
/// - trim trailing whitespace on each line
/// - remove trailing empty lines
fn normalize(s: &str) -> String {
    s.replace('\u{00A0}', " ")
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// Assert that tree and rtree produce identical normalized output.
/// On mismatch, prints detailed line-by-line diff.
fn assert_match(tree_out: &str, rtree_out: &str, description: &str) {
    let t = normalize(tree_out);
    let r = normalize(rtree_out);

    if t == r {
        return;
    }

    // Build detailed diff for debugging
    let t_lines: Vec<&str> = t.lines().collect();
    let r_lines: Vec<&str> = r.lines().collect();
    let max = t_lines.len().max(r_lines.len());

    eprintln!();
    eprintln!("========== MISMATCH: {} ==========", description);
    eprintln!("--- tree ({} lines) ---", t_lines.len());
    for (i, line) in t_lines.iter().enumerate() {
        eprintln!("{:3}| {}", i + 1, line);
    }
    eprintln!("--- rtree ({} lines) ---", r_lines.len());
    for (i, line) in r_lines.iter().enumerate() {
        eprintln!("{:3}| {}", i + 1, line);
    }
    eprintln!("--- differences ---");
    for i in 0..max {
        let tl = t_lines.get(i).copied().unwrap_or("<missing>");
        let rl = r_lines.get(i).copied().unwrap_or("<missing>");
        if tl != rl {
            eprintln!("  line {}:", i + 1);
            eprintln!("    tree:  {:?}", tl);
            eprintln!("    rtree: {:?}", rl);
        }
    }

    panic!("Output mismatch for: {}", description);
}

/// Helper: run both commands with same flags and assert exact match.
fn compare(dir: &Path, args: &[&str], description: &str) {
    let t = run_tree(dir, args);
    let r = run_rtree(dir, args);
    assert_match(&t, &r, description);
}

/// Helper: compare JSON output structurally (parsed, not string).
fn compare_json(dir: &Path, args: &[&str], description: &str) {
    let mut tree_args = vec!["-J"];
    tree_args.extend_from_slice(args);

    let mut rtree_args = vec!["-J"];
    rtree_args.extend_from_slice(args);

    let t = run_tree(dir, &tree_args);
    let r = run_rtree(dir, &rtree_args);

    let t_json: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| {
        panic!(
            "tree JSON parse error for {}:\n{}\nraw output:\n{}",
            description, e, t
        )
    });
    let r_json: serde_json::Value = serde_json::from_str(&r).unwrap_or_else(|e| {
        panic!(
            "rtree JSON parse error for {}:\n{}\nraw output:\n{}",
            description, e, r
        )
    });

    assert_eq!(
        t_json, r_json,
        "\nJSON mismatch for: {}\n--- tree ---\n{}\n--- rtree ---\n{}",
        description, t, r
    );
}

/// Helper: compare XML output (normalized whitespace).
fn compare_xml(dir: &Path, args: &[&str], description: &str) {
    let mut tree_args = vec!["-X"];
    tree_args.extend_from_slice(args);

    let mut rtree_args = vec!["-X"];
    rtree_args.extend_from_slice(args);

    let t = run_tree(dir, &tree_args);
    let r = run_rtree(dir, &rtree_args);

    let normalize_xml = |s: &str| -> String {
        s.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };

    let t_norm = normalize_xml(&t);
    let r_norm = normalize_xml(&r);

    assert_eq!(
        t_norm, r_norm,
        "\nXML mismatch for: {}\n--- tree ---\n{}\n--- rtree ---\n{}",
        description, t, r
    );
}

/// Helper: compare only file/dir names in output (ignore metadata formatting).
///
/// Useful for flags like -s, -p, -D where formatting may differ
/// but the listed entries must be the same.
fn compare_structure(dir: &Path, args: &[&str], description: &str) {
    let t = run_tree(dir, args);
    let r = run_rtree(dir, args);

    let extract_names = |s: &str| -> Vec<String> {
        s.lines()
            .map(|line| {
                // Strip tree drawing chars and metadata brackets
                let stripped = line
                    .trim_start_matches(|c: char| "│├└─ \t".contains(c))
                    .trim();
                // If line has [metadata] prefix, skip it
                if let Some(pos) = stripped.rfind(']') {
                    stripped[pos + 1..].trim().to_string()
                } else {
                    stripped.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect()
    };

    let t_names = extract_names(&t);
    let r_names = extract_names(&r);

    assert_eq!(
        t_names, r_names,
        "\nStructure mismatch for: {}\ntree names:  {:?}\nrtree names: {:?}\n--- tree ---\n{}\n--- rtree ---\n{}",
        description, t_names, r_names, t, r
    );
}

/// Create a simple directory (no symlinks, no hidden files).
fn make_simple_dir() -> TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("simple");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("file1.txt"), "hello").unwrap();
    fs::write(root.join("file2.txt"), "world").unwrap();
    fs::create_dir(root.join("subdir")).unwrap();
    fs::write(root.join("subdir/inner.txt"), "inner").unwrap();
    tmp
}

// ============================================================================
// GROUP 1: Basic structure (exact match expected)
// ============================================================================

#[test]
fn compat_default_with_report() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &[], "default (with report)");
}

#[test]
fn compat_default_noreport() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["--noreport"], "default --noreport");
}

#[test]
fn compat_dirs_only() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-d", "--noreport"], "-d --noreport");
}

#[test]
fn compat_max_depth_1() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-L", "1", "--noreport"], "-L 1 --noreport");
}

#[test]
fn compat_max_depth_2() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-L", "2", "--noreport"], "-L 2 --noreport");
}

#[test]
fn compat_show_all() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-a", "--noreport"], "-a --noreport");
}

#[test]
fn compat_full_path() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-f", "--noreport"], "-f --noreport");
}

#[test]
fn compat_no_indent() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-i", "--noreport"], "-i --noreport");
}

#[test]
fn compat_classify() {
    require_tree!();
    // Simple dir without executables to avoid platform detection differences
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("classtest");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("file.txt"), "text").unwrap();
    fs::create_dir(root.join("subdir")).unwrap();
    fs::write(root.join("subdir/inner.txt"), "inner").unwrap();
    unix_fs::symlink("file.txt", root.join("link")).unwrap();

    compare(&root, &["-F", "--noreport"], "-F --noreport");
}

// ============================================================================
// GROUP 2: Sorting (exact match expected)
// ============================================================================

#[test]
fn compat_dirs_first() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["--dirsfirst", "--noreport"],
        "--dirsfirst --noreport",
    );
}

#[test]
fn compat_files_first() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["--filesfirst", "--noreport"],
        "--filesfirst --noreport",
    );
}

#[test]
fn compat_reverse() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-r", "--noreport"], "-r --noreport");
}

#[test]
fn compat_reverse_dirs_first() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-r", "--dirsfirst", "--noreport"],
        "-r --dirsfirst --noreport",
    );
}

#[test]
fn compat_version_sort() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot/versions");
    compare(&dir, &["-v", "--noreport"], "-v --noreport (version sort)");
}

#[test]
fn compat_sort_name() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["--sort=name", "--noreport"],
        "--sort=name --noreport",
    );
}

// ============================================================================
// GROUP 3: Filtering (exact match expected)
// ============================================================================

#[test]
fn compat_pattern_txt() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-P", "*.txt", "--noreport"],
        "-P '*.txt' --noreport",
    );
}

#[test]
fn compat_exclude() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-I", "docs", "--noreport"], "-I 'docs' --noreport");
}

#[test]
fn compat_pattern_matchdirs() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-P", "*.txt", "--matchdirs", "--noreport"],
        "-P '*.txt' --matchdirs --noreport",
    );
}

#[test]
fn compat_ignore_case() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-P", "*.txt", "--ignore-case", "--noreport"],
        "-P '*.txt' --ignore-case --noreport",
    );
}

#[test]
fn compat_prune() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["--prune", "--noreport"], "--prune --noreport");
}

#[test]
fn compat_prune_with_pattern() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-P", "*.txt", "--prune", "--noreport"],
        "-P '*.txt' --prune --noreport",
    );
}

#[test]
fn compat_filelimit() {
    require_tree!();
    // Create a custom structure for filelimit test:
    // root/           (2 entries — will open)
    // ├── small/      (2 entries — will open)
    // │   ├── a.txt
    // │   └── b.txt
    // └── large/      (5 entries — exceeds filelimit 4, won't open)
    //     ├── f1..f5.txt
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("filelimit_test");
    fs::create_dir(&root).unwrap();

    fs::create_dir(root.join("small")).unwrap();
    fs::write(root.join("small/a.txt"), "a").unwrap();
    fs::write(root.join("small/b.txt"), "b").unwrap();

    fs::create_dir(root.join("large")).unwrap();
    for i in 1..=5 {
        fs::write(root.join(format!("large/f{}.txt", i)), "").unwrap();
    }

    let args = &["--filelimit", "4", "--noreport"];
    let t = run_tree_lenient(&root, args);
    let r = run_rtree(&root, args);
    assert_match(&t, &r, "--filelimit 4 --noreport");
}

// ============================================================================
// GROUP 4: Flag combinations (exact match expected)
// ============================================================================

#[test]
fn compat_dirs_only_depth() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-d", "-L", "1", "--noreport"], "-d -L 1 --noreport");
}

#[test]
fn compat_all_dirs_first() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-a", "--dirsfirst", "--noreport"],
        "-a --dirsfirst --noreport",
    );
}

#[test]
fn compat_full_path_dirs_only() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-f", "-d", "--noreport"], "-f -d --noreport");
}

#[test]
fn compat_no_indent_dirs_only() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-i", "-d", "--noreport"], "-i -d --noreport");
}

#[test]
fn compat_all_reverse_depth() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(
        &dir,
        &["-a", "-r", "-L", "2", "--noreport"],
        "-a -r -L 2 --noreport",
    );
}

#[test]
fn compat_classify_dirs_first() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("cftest");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("aaa.txt"), "text").unwrap();
    fs::create_dir(root.join("bbb")).unwrap();
    fs::write(root.join("bbb/inner.txt"), "x").unwrap();

    compare(
        &root,
        &["-F", "--dirsfirst", "--noreport"],
        "-F --dirsfirst --noreport",
    );
}

// ============================================================================
// GROUP 5: Follow symlinks
// ============================================================================

#[test]
fn compat_follow_symlinks() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare(&dir, &["-l", "-L", "2", "--noreport"], "-l -L 2 --noreport");
}

// ============================================================================
// GROUP 6: JSON output (parsed structural comparison)
// ============================================================================

#[test]
fn compat_json_simple() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_json(&dir, &[], "JSON simple");
}

#[test]
fn compat_json_dirs_only() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare_json(&dir, &["-d"], "JSON -d");
}

#[test]
fn compat_json_noreport() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_json(&dir, &["--noreport"], "JSON --noreport");
}

#[test]
fn compat_json_max_depth() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare_json(&dir, &["-L", "1"], "JSON -L 1");
}

#[test]
fn compat_json_all() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare_json(&dir, &["-a"], "JSON -a");
}

// ============================================================================
// GROUP 7: XML output (normalized string comparison)
// ============================================================================

#[test]
fn compat_xml_simple() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_xml(&dir, &[], "XML simple");
}

#[test]
fn compat_xml_noreport() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_xml(&dir, &["--noreport"], "XML --noreport");
}

#[test]
fn compat_xml_dirs_only() {
    require_tree!();
    let tmp = make_test_dir();
    let dir = tmp.path().join("testroot");
    compare_xml(&dir, &["-d"], "XML -d");
}

// ============================================================================
// GROUP 8: Metadata flags (structural comparison — formatting may differ)
//
// These tests verify that the SAME files appear in the SAME order.
// Exact formatting of metadata fields may differ from tree.
// ============================================================================

#[test]
fn compat_size_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(&dir, &["-s", "--noreport"], "-s structure");
}

#[test]
fn compat_human_size_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(&dir, &["-h", "--noreport"], "-h structure");
}

#[test]
fn compat_permissions_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(&dir, &["-p", "--noreport"], "-p structure");
}

#[test]
fn compat_date_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(
        &dir,
        &["-D", "--timefmt", "%Y-%m-%d", "--noreport"],
        "-D structure",
    );
}

#[test]
fn compat_owner_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(&dir, &["-u", "--noreport"], "-u structure");
}

#[test]
fn compat_group_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(&dir, &["-g", "--noreport"], "-g structure");
}

#[test]
fn compat_inodes_structure() {
    require_tree!();
    let tmp = make_simple_dir();
    let dir = tmp.path().join("simple");
    compare_structure(&dir, &["--inodes", "--noreport"], "--inodes structure");
}

// ============================================================================
// GROUP 9: Report format (must match exactly)
// ============================================================================

#[test]
fn compat_report_plural() {
    require_tree!();
    // Creates: 1 subdir, 3 files -> "1 directory, 3 files"
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("reporttest");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), "").unwrap();
    fs::write(root.join("b.txt"), "").unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/c.txt"), "").unwrap();

    let t = run_tree(&root, &[]);
    let r = run_rtree(&root, &[]);

    let get_report = |s: &str| -> String {
        s.lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    assert_eq!(
        get_report(&t),
        get_report(&r),
        "Report format mismatch (plural):\n  tree:  {:?}\n  rtree: {:?}\n\ntree full:\n{}\nrtree full:\n{}",
        get_report(&t),
        get_report(&r),
        t,
        r
    );
}

#[test]
fn compat_report_singular() {
    require_tree!();
    // Creates: 1 subdir, 1 file -> "1 directory, 1 file"
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("singular");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("dir")).unwrap();
    fs::write(root.join("dir/file.txt"), "").unwrap();

    let t = run_tree(&root, &[]);
    let r = run_rtree(&root, &[]);

    let get_report = |s: &str| -> String {
        s.lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    assert_eq!(
        get_report(&t),
        get_report(&r),
        "Report format mismatch (singular):\n  tree:  {:?}\n  rtree: {:?}",
        get_report(&t),
        get_report(&r)
    );
}

#[test]
fn compat_report_zero_files() {
    require_tree!();
    // Creates: 2 subdirs, 0 files -> "2 directories, 0 files"
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("zerofiles");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("a")).unwrap();
    fs::create_dir(root.join("b")).unwrap();

    let t = run_tree(&root, &[]);
    let r = run_rtree(&root, &[]);

    let get_report = |s: &str| -> String {
        s.lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    assert_eq!(
        get_report(&t),
        get_report(&r),
        "Report mismatch (zero files)"
    );
}

// ============================================================================
// GROUP 10: Edge cases
// ============================================================================

#[test]
fn compat_empty_directory() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("emptytest");
    fs::create_dir(&root).unwrap();

    compare(&root, &["--noreport"], "empty directory --noreport");
}

#[test]
fn compat_single_file() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("singletest");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("only.txt"), "alone").unwrap();

    compare(&root, &["--noreport"], "single file --noreport");
}

#[test]
fn compat_deep_nesting() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("deeptest");
    fs::create_dir_all(root.join("a/b/c/d/e")).unwrap();
    fs::write(root.join("a/b/c/d/e/deep.txt"), "").unwrap();

    compare(&root, &["--noreport"], "deep nesting --noreport");
}

#[test]
fn compat_deep_with_limit() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("deeplimit");
    fs::create_dir_all(root.join("a/b/c/d/e")).unwrap();
    fs::write(root.join("a/b/c/d/e/deep.txt"), "").unwrap();

    compare(
        &root,
        &["-L", "3", "--noreport"],
        "deep nesting -L 3 --noreport",
    );
}

#[test]
fn compat_many_siblings() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("siblings");
    fs::create_dir(&root).unwrap();
    for i in 1..=20 {
        fs::write(root.join(format!("file{:02}.txt", i)), "").unwrap();
    }

    compare(&root, &["--noreport"], "many siblings --noreport");
}

#[test]
fn compat_special_chars_in_name() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("special");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("hello world.txt"), "spaces").unwrap();
    fs::write(root.join("file (1).txt"), "parens").unwrap();
    fs::write(root.join("file-name_v2.txt"), "dashes").unwrap();

    compare(&root, &["--noreport"], "special chars in filenames");
}

#[test]
fn compat_empty_with_report() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("emptyreport");
    fs::create_dir(&root).unwrap();

    compare(&root, &[], "empty directory with report");
}

// ============================================================================
// GROUP 11: Time sort (verify order, not exact times)
// ============================================================================

#[test]
fn compat_time_sort() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("tsort");
    fs::create_dir(&root).unwrap();

    // Create files with different modification times
    fs::write(root.join("old.txt"), "old").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(root.join("mid.txt"), "mid").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(root.join("new.txt"), "new").unwrap();

    compare(&root, &["-t", "--noreport"], "-t --noreport (time sort)");
}

#[test]
fn compat_time_sort_reverse() {
    require_tree!();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("tsortrev");
    fs::create_dir(&root).unwrap();

    fs::write(root.join("old.txt"), "old").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(root.join("mid.txt"), "mid").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(root.join("new.txt"), "new").unwrap();

    compare(
        &root,
        &["-t", "-r", "--noreport"],
        "-t -r --noreport (time sort reverse)",
    );
}
