use std::collections::HashMap;
use std::env;

use crate::core::entry::{Entry as TreeEntry, EntryType};

#[derive(Debug, Clone)]
pub struct ColorScheme {
    type_colors: HashMap<String, String>,
    ext_colors: HashMap<String, String>,
}

impl ColorScheme {
    pub fn load() -> Self {
        let mut scheme = ColorScheme::default();

        // Check NO_COLOR first - if set, disable colors entirely
        if env::var("NO_COLOR").is_ok() {
            return scheme;
        }

        // Try TREE_COLORS first, then LS_COLORS
        if let Ok(colors) = env::var("TREE_COLORS") {
            scheme.parse_ls_colors(&colors);
        } else if let Ok(colors) = env::var("LS_COLORS") {
            scheme.parse_ls_colors(&colors);
        }

        scheme
    }

    /// Validate ANSI color code: only digits and semicolons allowed
    fn is_valid_ansi_code(code: &str) -> bool {
        !code.is_empty() && code.len() <= 20 && code.chars().all(|c| c.is_ascii_digit() || c == ';')
    }

    fn parse_ls_colors(&mut self, spec: &str) {
        for entry in spec.split(':') {
            if let Some((key, value)) = entry.split_once('=') {
                // Skip invalid ANSI codes
                if !Self::is_valid_ansi_code(value) {
                    continue;
                }
                if key.starts_with('*') {
                    // Extension: *.rs=0;33
                    let ext = key.trim_start_matches("*.");
                    self.ext_colors
                        .insert(ext.to_lowercase(), value.to_string());
                } else {
                    // Type: di=1;34
                    self.type_colors.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    pub fn get_color(&self, entry: &TreeEntry) -> String {
        // Check entry type first
        let type_key = match &entry.entry_type {
            EntryType::Directory => "di",
            EntryType::Symlink { broken: false, .. } => "ln",
            EntryType::Symlink { broken: true, .. } => "or",
            EntryType::Junction { .. } => "ln",
            EntryType::File => {
                // Check if executable
                if crate::platform::is_executable(&entry.path) {
                    "ex"
                } else {
                    "fi"
                }
            }
            _ => "fi",
        };

        if let Some(color) = self.type_colors.get(type_key) {
            return color.clone();
        }

        // Check extension
        if let Some(ext) = entry.path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if let Some(color) = self.ext_colors.get(&ext) {
                return color.clone();
            }
        }

        // Check Windows attributes
        if let Some(ref meta) = entry.metadata {
            if meta.attributes.hidden {
                if let Some(color) = self.type_colors.get("hi") {
                    return color.clone();
                }
            }
            if meta.attributes.system {
                if let Some(color) = self.type_colors.get("sy") {
                    return color.clone();
                }
            }
        }

        String::new()
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        let mut type_colors = HashMap::new();
        let mut ext_colors = HashMap::new();

        // Default colors (GNU tree compatible)
        type_colors.insert("di".to_string(), "1;34".to_string()); // Bold blue
        type_colors.insert("ln".to_string(), "1;36".to_string()); // Bold cyan
        type_colors.insert("or".to_string(), "1;31;40".to_string()); // Bold red on black
        type_colors.insert("ex".to_string(), "1;32".to_string()); // Bold green
        type_colors.insert("fi".to_string(), "0".to_string()); // Default

        // Windows extensions
        type_colors.insert("hi".to_string(), "2;37".to_string()); // Dim white
        type_colors.insert("sy".to_string(), "2;37".to_string()); // Dim white

        // Common extensions
        ext_colors.insert("exe".to_string(), "1;32".to_string());
        ext_colors.insert("bat".to_string(), "1;32".to_string());
        ext_colors.insert("cmd".to_string(), "1;32".to_string());
        ext_colors.insert("ps1".to_string(), "1;32".to_string());

        ext_colors.insert("zip".to_string(), "1;33".to_string());
        ext_colors.insert("rar".to_string(), "1;33".to_string());
        ext_colors.insert("7z".to_string(), "1;33".to_string());
        ext_colors.insert("tar".to_string(), "1;33".to_string());
        ext_colors.insert("gz".to_string(), "1;33".to_string());

        ext_colors.insert("png".to_string(), "1;35".to_string());
        ext_colors.insert("jpg".to_string(), "1;35".to_string());
        ext_colors.insert("gif".to_string(), "1;35".to_string());
        ext_colors.insert("svg".to_string(), "1;35".to_string());

        ext_colors.insert("rs".to_string(), "0;33".to_string());
        ext_colors.insert("py".to_string(), "0;33".to_string());
        ext_colors.insert("js".to_string(), "0;33".to_string());
        ext_colors.insert("ts".to_string(), "0;33".to_string());

        ColorScheme {
            type_colors,
            ext_colors,
        }
    }
}
