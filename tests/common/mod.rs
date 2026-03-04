use assert_cmd::Command;

#[allow(deprecated)]
pub fn rtree() -> Command {
    Command::cargo_bin("rtree").unwrap()
}

/// Standard flags to get clean, predictable text output
#[allow(dead_code)]
pub const CLEAN: &[&str] = &["-n", "--no-icons", "--noreport", "--lang", "en"];

/// Extract file/dir names from text output (skip root line, strip tree chars)
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn collect_all_names(json: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            collect_entry_names(item, &mut names);
        }
    }
    names
}

#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[allow(dead_code)]
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
