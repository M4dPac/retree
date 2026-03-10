use std::collections::HashMap;
use std::sync::LazyLock;

use crate::cli::IconStyle;
use crate::core::walker::{EntryType, TreeEntry};

#[derive(Debug, Clone)]
pub struct IconSet {
    style: IconStyle,
    extension_icons: &'static HashMap<&'static str, &'static str>,
    name_icons: &'static HashMap<&'static str, &'static str>,
    dir_icons: &'static HashMap<&'static str, &'static str>,
}

// Nerd Font icons
static NERD_EXT_ICONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Programming languages
    m.insert("rs", "\u{e7a8}"); // Rust
    m.insert("py", "\u{e73c}"); // Python
    m.insert("js", "\u{e74e}"); // JavaScript
    m.insert("ts", "\u{e628}"); // TypeScript
    m.insert("jsx", "\u{e7ba}"); // React
    m.insert("tsx", "\u{e7ba}"); // React TypeScript
    m.insert("vue", "\u{e6a0}"); // Vue
    m.insert("go", "\u{e626}"); // Go
    m.insert("java", "\u{e738}"); // Java
    m.insert("c", "\u{e61e}"); // C
    m.insert("cpp", "\u{e61d}"); // C++
    m.insert("cc", "\u{e61d}"); // C++
    m.insert("h", "\u{e61f}"); // Header
    m.insert("hpp", "\u{e61f}"); // C++ Header
    m.insert("cs", "\u{f81a}"); // C#
    m.insert("rb", "\u{e739}"); // Ruby
    m.insert("php", "\u{e73d}"); // PHP
    m.insert("swift", "\u{e755}"); // Swift
    m.insert("kt", "\u{e634}"); // Kotlin
    m.insert("scala", "\u{e737}"); // Scala
    m.insert("lua", "\u{e620}"); // Lua
    m.insert("zig", "\u{e6a9}"); // Zig

    // Shell
    m.insert("sh", "\u{f489}"); // Shell
    m.insert("bash", "\u{f489}"); // Bash
    m.insert("zsh", "\u{f489}"); // Zsh
    m.insert("fish", "\u{f489}"); // Fish
    m.insert("ps1", "\u{ebc7}"); // PowerShell
    m.insert("psm1", "\u{ebc7}"); // PowerShell Module
    m.insert("bat", "\u{e629}"); // Batch
    m.insert("cmd", "\u{e629}"); // Cmd

    // Windows
    m.insert("exe", "\u{f013}"); // Executable
    m.insert("msi", "\u{f462}"); // Installer
    m.insert("dll", "\u{f0ad}"); // Library
    m.insert("sys", "\u{f013}"); // Driver
    m.insert("lnk", "\u{f0c1}"); // Shortcut
    m.insert("reg", "\u{f013}"); // Registry

    // Web
    m.insert("html", "\u{e736}"); // HTML
    m.insert("htm", "\u{e736}"); // HTML
    m.insert("css", "\u{e749}"); // CSS
    m.insert("scss", "\u{e74b}"); // Sass
    m.insert("sass", "\u{e74b}"); // Sass
    m.insert("less", "\u{e758}"); // Less

    // Data
    m.insert("json", "\u{e60b}"); // JSON
    m.insert("yaml", "\u{e6a8}"); // YAML
    m.insert("yml", "\u{e6a8}"); // YAML
    m.insert("xml", "\u{e619}"); // XML
    m.insert("toml", "\u{e6b2}"); // TOML
    m.insert("ini", "\u{e615}"); // INI
    m.insert("cfg", "\u{e615}"); // Config
    m.insert("conf", "\u{e615}"); // Config

    // Documents
    m.insert("md", "\u{e73e}"); // Markdown
    m.insert("markdown", "\u{e73e}");
    m.insert("txt", "\u{f0f6}"); // Text
    m.insert("pdf", "\u{f1c1}"); // PDF
    m.insert("doc", "\u{f1c2}"); // Word
    m.insert("docx", "\u{f1c2}"); // Word
    m.insert("xls", "\u{f1c3}"); // Excel
    m.insert("xlsx", "\u{f1c3}"); // Excel
    m.insert("ppt", "\u{f1c4}"); // PowerPoint
    m.insert("pptx", "\u{f1c4}"); // PowerPoint

    // Archives
    m.insert("zip", "\u{f1c6}"); // Archive
    m.insert("rar", "\u{f1c6}");
    m.insert("7z", "\u{f1c6}");
    m.insert("tar", "\u{f1c6}");
    m.insert("gz", "\u{f1c6}");
    m.insert("bz2", "\u{f1c6}");
    m.insert("xz", "\u{f1c6}");

    // Images
    m.insert("png", "\u{f1c5}"); // Image
    m.insert("jpg", "\u{f1c5}");
    m.insert("jpeg", "\u{f1c5}");
    m.insert("gif", "\u{f1c5}");
    m.insert("bmp", "\u{f1c5}");
    m.insert("svg", "\u{f1c5}");
    m.insert("webp", "\u{f1c5}");
    m.insert("ico", "\u{f1c5}");

    // Audio
    m.insert("mp3", "\u{f001}"); // Audio
    m.insert("wav", "\u{f001}");
    m.insert("flac", "\u{f001}");
    m.insert("ogg", "\u{f001}");
    m.insert("m4a", "\u{f001}");

    // Video
    m.insert("mp4", "\u{f008}"); // Video
    m.insert("mkv", "\u{f008}");
    m.insert("avi", "\u{f008}");
    m.insert("mov", "\u{f008}");
    m.insert("wmv", "\u{f008}");
    m.insert("webm", "\u{f008}");

    // Fonts
    m.insert("ttf", "\u{f031}"); // Font
    m.insert("otf", "\u{f031}");
    m.insert("woff", "\u{f031}");
    m.insert("woff2", "\u{f031}");

    // Database
    m.insert("sql", "\u{f1c0}"); // Database
    m.insert("db", "\u{f1c0}");
    m.insert("sqlite", "\u{f1c0}");

    // Misc
    m.insert("log", "\u{f0f6}"); // Log
    m.insert("lock", "\u{f023}"); // Lock
    m.insert("env", "\u{f462}"); // Env

    m
});

static NERD_NAME_ICONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    m.insert("Cargo.toml", "\u{e7a8}");
    m.insert("Cargo.lock", "\u{e7a8}");
    m.insert("package.json", "\u{e718}");
    m.insert("package-lock.json", "\u{e718}");
    m.insert("tsconfig.json", "\u{e628}");
    m.insert("Dockerfile", "\u{f308}");
    m.insert("docker-compose.yml", "\u{f308}");
    m.insert("docker-compose.yaml", "\u{f308}");
    m.insert("Makefile", "\u{e673}");
    m.insert("CMakeLists.txt", "\u{e673}");
    m.insert(".gitignore", "\u{e65d}");
    m.insert(".gitattributes", "\u{e65d}");
    m.insert(".gitmodules", "\u{e65d}");
    m.insert(".dockerignore", "\u{f308}");
    m.insert(".editorconfig", "\u{e615}");
    m.insert("README.md", "\u{f48a}");
    m.insert("README", "\u{f48a}");
    m.insert("LICENSE", "\u{f0f6}");
    m.insert("CHANGELOG.md", "\u{f4a2}");
    m.insert("CHANGELOG", "\u{f4a2}");

    m
});

static NERD_DIR_ICONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    m.insert(".git", "\u{e5fb}");
    m.insert("node_modules", "\u{e718}");
    m.insert("src", "\u{f121}");
    m.insert("test", "\u{f0668}");
    m.insert("tests", "\u{f0668}");
    m.insert("docs", "\u{f02d}");
    m.insert(".config", "\u{e5fc}");
    m.insert(".vscode", "\u{e70c}");
    m.insert(".idea", "\u{e7b5}");
    m.insert(".github", "\u{f408}");
    m.insert("target", "\u{f487}");
    m.insert("build", "\u{f0ad}");
    m.insert("dist", "\u{f466}");
    m.insert("bin", "\u{eae8}");
    m.insert("lib", "\u{f1c0}");
    m.insert("vendor", "\u{f187}");
    m.insert("Windows", "\u{e70f}");
    m.insert("Program Files", "\u{f0a0}");
    m.insert("Users", "\u{f0c0}");
    m.insert("Desktop", "\u{f108}");
    m.insert("Documents", "\u{f0f6}");
    m.insert("Downloads", "\u{f019}");
    m.insert("Pictures", "\u{f03e}");
    m.insert("Music", "\u{f001}");
    m.insert("Videos", "\u{f008}");
    m.insert("AppData", "\u{f013}");

    m
});

impl IconSet {
    pub fn new(style: IconStyle) -> Self {
        IconSet {
            style,
            extension_icons: &NERD_EXT_ICONS,
            name_icons: &NERD_NAME_ICONS,
            dir_icons: &NERD_DIR_ICONS,
        }
    }

    pub fn get_icon(&self, entry: &TreeEntry) -> String {
        match self.style {
            IconStyle::Nerd => self.get_nerd_icon(entry),
            IconStyle::Unicode => self.get_unicode_icon(entry),
            IconStyle::Ascii => self.get_ascii_icon(entry),
        }
    }

    fn get_nerd_icon(&self, entry: &TreeEntry) -> String {
        let name = entry.name_str();

        // Check special types first
        match &entry.entry_type {
            EntryType::Symlink { broken: true, .. } => {
                return "\u{f127}".to_string(); // Broken link
            }
            EntryType::Symlink { .. } | EntryType::Junction { .. } => {
                return "\u{f0c1}".to_string(); // Link
            }
            _ => {}
        }

        // Check by exact name
        if let Some(icon) = self.name_icons.get(name) {
            return icon.to_string();
        }

        // Check directory names
        if entry.entry_type.is_directory() {
            if let Some(icon) = self.dir_icons.get(name) {
                return icon.to_string();
            }
            return "\u{f07b}".to_string(); // Default folder
        }

        // Check by extension
        if let Some(ext) = entry.path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if let Some(icon) = self.extension_icons.get(ext.as_str()) {
                return icon.to_string();
            }
        }

        // Check Windows attributes
        if let Some(ref meta) = entry.metadata {
            if meta.attributes.hidden {
                return "\u{f070}".to_string(); // Hidden
            }
            if meta.attributes.system {
                return "\u{f013}".to_string(); // System
            }
        }

        // Default file icon
        "\u{f016}".to_string()
    }

    fn get_unicode_icon(&self, entry: &TreeEntry) -> String {
        match &entry.entry_type {
            EntryType::Directory => "📁".to_string(),
            EntryType::Symlink { broken: true, .. } => "🔗".to_string(),
            EntryType::Symlink { .. } | EntryType::Junction { .. } => "🔗".to_string(),
            EntryType::File => {
                // Basic file type detection
                if let Some(ext) = entry.path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    match ext.as_str() {
                        "txt" | "md" | "doc" | "docx" => "📄".to_string(),
                        "png" | "jpg" | "gif" | "svg" => "🖼️".to_string(),
                        "mp3" | "wav" | "flac" => "🎵".to_string(),
                        "mp4" | "mkv" | "avi" => "🎬".to_string(),
                        "zip" | "rar" | "7z" | "tar" => "📦".to_string(),
                        "exe" | "msi" => "⚙️".to_string(),
                        _ => "📄".to_string(),
                    }
                } else {
                    "📄".to_string()
                }
            }
            _ => "📄".to_string(),
        }
    }

    fn get_ascii_icon(&self, entry: &TreeEntry) -> String {
        match &entry.entry_type {
            EntryType::Directory => "[D]".to_string(),
            EntryType::Symlink { .. } | EntryType::Junction { .. } => "[L]".to_string(),
            EntryType::File => "[F]".to_string(),
            _ => "[?]".to_string(),
        }
    }
}
