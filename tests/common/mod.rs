#![allow(dead_code)]

use assert_cmd::assert::Assert;
use assert_cmd::Command;
use std::path::{Path, PathBuf};

pub fn retree() -> Command {
    Command::new(retree_path())
}

pub fn retree_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    p.pop();
    let name = format!("rt{}", std::env::consts::EXE_SUFFIX);
    p.push(name);
    assert!(p.exists(), "rt binary not found at {:?}", p);
    p
}

/// Standard flags to get clean, predictable text output
pub const CLEAN: &[&str] = &["-n", "--no-icons", "--noreport", "--lang", "en"];

#[allow(dead_code)]
pub fn retree_command(args: &[&str]) -> Command {
    let mut cmd = Command::new(retree_path());
    cmd.args(args)
        .env("LC_ALL", "en_US.UTF-8")
        .env("TREE_LANG", "en");
    cmd
}

/// Build argument list for directory-based tests.
///
/// Adds `--no-icons --lang en -n`, then extra arguments, then the target path.
pub fn retree_dir_args(dir: &Path, extra_args: &[&str]) -> Vec<String> {
    let mut args = vec![
        "--no-icons".to_string(),
        "--lang".to_string(),
        "en".to_string(),
        "-n".to_string(),
    ];
    args.extend(extra_args.iter().map(|s| s.to_string()));
    args.push(dir.to_string_lossy().into_owned());
    args
}

/// Build an `retree` command for a directory-based test.
pub fn run_retree_command(dir: &Path, extra_args: &[&str]) -> Command {
    let args = retree_dir_args(dir, extra_args);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    retree_command(&refs)
}

/// Run `retree` and return `(stdout, stderr, exit_code)`.
pub fn run_retree_args_full(args: &[&str]) -> (String, String, Option<i32>) {
    let out = retree_command(args)
        .output()
        .expect("failed to execute retree");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (stdout, stderr, out.status.code())
}

/// Run `retree` on a directory and return `(stdout, stderr, exit_code)`.
pub fn run_retree_full(dir: &Path, extra_args: &[&str]) -> (String, String, Option<i32>) {
    let args = retree_dir_args(dir, extra_args);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_retree_args_full(&refs)
}

/// Run `retree` on a directory and return stdout only.
///
/// Panics on non-zero exit.
pub fn run_retree(dir: &Path, extra_args: &[&str]) -> String {
    let (stdout, stderr, code) = run_retree_full(dir, extra_args);
    if code != Some(0) {
        panic!("retree failed (status {:?}):\nstderr: {}", code, stderr);
    }
    stdout
}

pub fn output_stdout(output: &Assert) -> String {
    String::from_utf8(output.get_output().stdout.clone()).unwrap()
}

pub fn output_stderr(output: &Assert) -> String {
    String::from_utf8(output.get_output().stderr.clone()).unwrap()
}

pub fn output_json(output: &Assert) -> serde_json::Value {
    serde_json::from_slice(&output.get_output().stdout).unwrap()
}

pub fn last_nonempty_line(s: &str) -> &str {
    s.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("")
}

/// Extract file/dir names from text output (skip root line, strip tree chars)
pub fn extract_names(output: &str) -> Vec<String> {
    output
        .lines()
        .skip(1) // root directory line
        .map(|l| {
            l.replace("├── ", "")
                .replace("└── ", "")
                .replace("│   ", "")
                .replace("    ", "")
                .replace("|-- ", "")
                .replace("`-- ", "")
                .replace("|   ", "")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Recursively collect all entry names from JSON tree structure
pub fn collect_all_names(json: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            collect_entry_names(item, &mut names);
        }
    }
    names
}

pub fn collect_entry_names(entry: &serde_json::Value, names: &mut Vec<String>) {
    if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
        names.push(name.to_string());
    }
    if let Some(contents) = entry.get("contents").and_then(|c| c.as_array()) {
        for child in contents {
            collect_entry_names(child, names);
        }
    }
}

/// Count files and directories recursively in JSON structure
pub fn count_files_and_dirs(json: &serde_json::Value) -> (u64, u64) {
    let mut files = 0u64;
    let mut dirs = 0u64;
    if let Some(arr) = json.as_array() {
        for item in arr {
            count_entry_types(item, &mut files, &mut dirs);
        }
    }
    (files, dirs)
}

pub fn count_entry_types(entry: &serde_json::Value, files: &mut u64, dirs: &mut u64) {
    if let Some(t) = entry.get("type").and_then(|t| t.as_str()) {
        match t {
            "file" => *files += 1,
            "directory" => *dirs += 1,
            _ => {}
        }
    }
    if let Some(contents) = entry.get("contents").and_then(|c| c.as_array()) {
        for child in contents {
            count_entry_types(child, files, dirs);
        }
    }
}
