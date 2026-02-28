# rtree 🌲

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Windows](https://img.shields.io/badge/platform-Windows-0078d4.svg)](https://www.microsoft.com/windows)

**GNU tree compatible directory listing for Windows, written in Rust.**

[🇷🇺 Русская версия](README.ru.md)

---

## 📖 Table of Contents

- [Overview](#-overview)
- [Features](#-features)
- [Screenshots](#-screenshots)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Usage](#-usage)
  - [Listing Options](#listing-options)
  - [Filtering Options](#filtering-options)
  - [Sorting Options](#sorting-options)
  - [Display Options](#display-options)
  - [File Information](#file-information)
  - [Export Options](#export-options)
  - [Icons](#icons)
  - [Windows-Specific Options](#windows-specific-options)
- [Configuration](#-configuration)
- [Color Customization](#-color-customization)
- [Icon Reference](#-icon-reference)
- [Examples](#-examples)
- [Building from Source](#-building-from-source)
- [Testing](#-testing)
- [Performance](#-performance)
- [Comparison with GNU tree](#-comparison-with-gnu-tree)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

---

## 🎯 Overview

**rtree** is a modern, fast, and feature-rich command-line utility for displaying directory structures in a tree-like format. It is designed as a Windows-native replacement for the GNU `tree` command with full compatibility and additional features tailored for Windows environments.

### Why rtree?

- **Windows-Native**: Built specifically for Windows with support for NTFS features like junctions, alternate data streams, and proper handling of Windows file attributes.
- **GNU tree Compatible**: Drop-in replacement with support for all major GNU tree flags.
- **Modern Features**: Nerd Font icons, LS_COLORS support, and multiple output formats.
- **Fast**: Written in Rust for maximum performance—handles millions of files efficiently.
- **Multilingual**: Full support for English and Russian interfaces.

---

## ✨ Features

### Core Features

- ✅ **Full GNU tree compatibility** — All major flags and options supported
- 🎨 **LS_COLORS support** — Colorize output based on file types and extensions
- 🔤 **Nerd Font icons** — Beautiful file and folder icons (requires Nerd Font)
- 📁 **Multiple output formats** — Text, JSON, XML, HTML
- 🔍 **Advanced filtering** — Glob patterns, exclusions, depth limits
- 📊 **Flexible sorting** — By name, size, date, with natural sorting support

### Windows-Specific Features

- 🪟 **NTFS support** — Junctions, symbolic links, hard links
- 📎 **Alternate Data Streams** — View NTFS ADS
- 🔒 **Windows attributes** — Hidden, System, ReadOnly, Archive flags
- 📏 **Long paths** — Support for paths longer than 260 characters
- 🌐 **UNC paths** — Network share support (`\\server\share`)

### Quality of Life

- 🌍 **Multilingual** — English and Russian interface
- ⚡ **High performance** — Handles 1M+ files without issues
- 🎯 **Zero configuration** — Works out of the box
- 📝 **Configurable** — TOML configuration file support

---

## 📸 Screenshots

### Basic Output

```
C:\Projects\myapp
├── src
│   ├── main.rs
│   └── lib.rs
├── tests
│   └── integration_test.rs
├── Cargo.toml
└── README.md

2 directories, 5 files
```

### With Icons and Colors

```
 C:\Projects\myapp
├──  src
│   ├──  main.rs
│   └──  lib.rs
├──  tests
│   └──  integration_test.rs
├──  Cargo.toml
└──  README.md

2 directories, 5 files
```

### JSON Output

```json
[
  {
    "type": "directory",
    "name": "src",
    "contents": [
      {"type": "file", "name": "main.rs", "size": 1234},
      {"type": "file", "name": "lib.rs", "size": 567}
    ]
  }
]
```

---

## 📦 Installation

### Using Cargo (Recommended)

```powershell
cargo install rtree
```

### Using Scoop

```powershell
scoop bucket add extras
scoop install rtree
```

### Using Winget

```powershell
winget install rtree
```

### Manual Installation

1. Download the latest release from [GitHub Releases](https://github.com/user/rtree/releases)
2. Extract `rtree.exe` to a directory in your PATH
3. Verify installation:

   ```powershell
   rtree --version
   ```

### Building from Source

```powershell
git clone https://github.com/user/rtree.git
cd rtree
cargo build --release
# Binary will be in target/release/rtree.exe
```

---

## 🚀 Quick Start

```powershell
# Display current directory
rtree

# Display specific directory
rtree C:\Users\Name\Projects

# Show hidden files
rtree -a

# Limit depth to 2 levels
rtree -L 2

# With colors and icons
rtree -C --icons

# Only directories
rtree -d

# Output as JSON
rtree -J > tree.json
```

---

## 📚 Usage

```
rtree [OPTIONS] [PATH...]
```

If no PATH is specified, the current directory is used. Multiple paths can be specified to display multiple trees.

### Listing Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-a` | `--all` | Show all files, including hidden files |
| `-d` | `--dirs-only` | List directories only |
| `-l` | `--follow` | Follow symbolic links |
| `-f` | `--full-path` | Print the full path prefix for each file |
| `-x` | `--one-fs` | Stay on the current filesystem |
| `-L N` | `--level N` | Descend only N levels deep |
| | `--filelimit N` | Do not descend directories with more than N entries |
| | `--noreport` | Omit the file/directory count at the end |

**Examples:**

```powershell
# Show all files including hidden
rtree -a

# Only show directories, max 3 levels deep
rtree -d -L 3

# Skip directories with more than 100 files
rtree --filelimit 100
```

### Filtering Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-P PATTERN` | `--pattern PATTERN` | Only list files matching the glob pattern |
| `-I PATTERN` | `--exclude PATTERN` | Exclude files matching the pattern (can be repeated) |
| | `--matchdirs` | Apply patterns to directory names as well |
| | `--ignore-case` | Case-insensitive pattern matching |
| | `--prune` | Do not display empty directories |

**Glob Pattern Syntax:**

- `*` — matches any sequence of characters
- `?` — matches any single character
- `[abc]` — matches any character in brackets
- `[a-z]` — matches any character in range
- `[!abc]` — matches any character NOT in brackets

**Examples:**

```powershell
# Only show Rust files
rtree -P "*.rs"

# Exclude node_modules and target directories
rtree -I "node_modules" -I "target"

# Show only source files, case-insensitive
rtree -P "*.[ch]" --ignore-case

# Apply pattern to directories too
rtree -P "src*" --matchdirs
```

### Sorting Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-v` | `--version-sort` | Natural sort (e.g., file2 before file10) |
| `-t` | `--timesort` | Sort by modification time |
| `-c` | `--ctime` | Sort by change time (metadata change) |
| `-U` | `--unsorted` | Leave files unsorted (as returned by OS) |
| `-r` | `--reverse` | Reverse the sort order |
| | `--dirsfirst` | List directories before files |
| | `--filesfirst` | List files before directories |
| | `--sort TYPE` | Sort by: `name`, `size`, `mtime`, `ctime`, `version`, `none` |

**Examples:**

```powershell
# Natural version sort
rtree -v

# Sort by modification time, newest first
rtree -t -r

# Directories first, then files by name
rtree --dirsfirst

# Sort by size
rtree --sort=size
```

### Display Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-i` | `--noindent` | Don't print indentation lines |
| `-A` | `--ansi` | Use ANSI line graphics (├── └──) |
| `-S` | `--cp437` | Use CP437 line graphics |
| `-n` | `--nocolor` | Turn off colorization |
| `-C` | `--color-always` | Always use colorization |
| | `--color WHEN` | When to colorize: `auto`, `always`, `never` |

**Examples:**

```powershell
# Force colors even when piping
rtree -C | less -R

# Plain ASCII output
rtree -S -n

# No tree lines, just indented list
rtree -i
```

### File Information

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-s` | `--size` | Print file size in bytes |
| `-h` | `--human` | Print human-readable sizes (1K, 2M, 3G) |
| | `--si` | Use SI units (1000-based instead of 1024) |
| `-D` | `--date` | Print modification date |
| | `--timefmt FMT` | Date format (strftime syntax) |
| `-p` | `--perm` | Print file permissions/attributes |
| `-u` | `--uid` | Print file owner (planned for future) |
| `-g` | `--gid` | Print file group (planned for future) |
| | `--inodes` | Print inode numbers |
| | `--device` | Print device numbers |
| `-F` | `--classify` | Append / for dirs, @ for links, * for executables |
| `-q` | `--safe` | Replace non-printable characters with ? |
| `-N` | `--literal` | Print non-printable characters as-is |

**Examples:**

```powershell
# Show sizes in human-readable format
rtree -h

# Show modification dates
rtree -D

# Custom date format
rtree -D --timefmt="%Y-%m-%d"

# Show permissions
rtree -p

# Full details
rtree -h -D -p -F
```

### Export Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-o FILE` | `--output FILE` | Output to file instead of stdout |
| `-H URL` | `--html URL` | Generate HTML with base URL for links |
| `-T TITLE` | `--title TITLE` | Set HTML page title |
| | `--nolinks` | Don't create hyperlinks in HTML |
| | `--hintro FILE` | Use custom HTML header from file |
| | `--houtro FILE` | Use custom HTML footer from file |
| `-X` | `--xml` | Output as XML |
| `-J` | `--json` | Output as JSON |

**Examples:**

```powershell
# Save to file
rtree -o tree.txt

# Generate HTML
rtree -H "https://example.com/files" -T "My Project" > tree.html

# Export as JSON for processing
rtree -J | jq '.[] | select(.type == "file")'

# Export as XML
rtree -X > tree.xml
```

### Icons

| Flag | Long Form | Description |
|------|-----------|-------------|
| | `--icons WHEN` | Show icons: `auto`, `always`, `never` |
| | `--no-icons` | Disable icons |
| | `--icon-style STYLE` | Icon style: `nerd`, `unicode`, `ascii` |

**Examples:**

```powershell
# Enable Nerd Font icons
rtree --icons

# Always show icons
rtree --icons=always

# Use Unicode emoji icons (no Nerd Font needed)
rtree --icons --icon-style=unicode

# ASCII-only icons
rtree --icons --icon-style=ascii
```

### Windows-Specific Options

| Flag | Long Form | Description |
|------|-----------|-------------|
| | `--show-streams` | Show NTFS Alternate Data Streams |
| | `--show-junctions` | Show junction points with their targets |
| | `--hide-system` | Hide system files even with `-a` |
| | `--permissions MODE` | Permission format: `posix` or `windows` |
| | `--long-paths` | Force use of \\?\ prefix for long paths |
| | `--lang LANG` | Interface language: `en` or `ru` |

**Examples:**

```powershell
# Show alternate data streams
rtree --show-streams

# Hide system files
rtree -a --hide-system

# Show POSIX-style permissions
rtree -p --permissions=posix

# Russian interface
rtree --lang=ru
```

---

## ⚙️ Configuration

> **Note**: TOML configuration file support is planned for future releases.
> Currently, rtree can be configured via command-line flags and environment variables.

### Environment Variables

### Environment Variables

| Variable | Description |
|----------|-------------|
| `TREE_COLORS` | Color configuration (overrides LS_COLORS) |
| `LS_COLORS` | GNU ls color configuration |
| `TREE_LANG` | Interface language (`en` or `ru`) |
| `NO_COLOR` | Disable all colors when set |

---

## 🎨 Color Customization

rtree supports the GNU LS_COLORS format for color customization.

### Setting Colors

```powershell
# Using environment variable
$env:LS_COLORS = "di=1;34:*.rs=0;33:*.md=0;36"

# Or use TREE_COLORS (takes precedence)
$env:TREE_COLORS = "di=1;34:ex=1;32:*.rs=1;33"
```

### Color Format

```
TYPE=STYLE;FOREGROUND;BACKGROUND
```

**Types:**

| Code | Type |
|------|------|
| `di` | Directory |
| `fi` | Regular file |
| `ln` | Symbolic link |
| `or` | Orphan (broken) link |
| `ex` | Executable |
| `*.ext` | Files with extension |

**Windows-specific types:**

| Code | Type |
|------|------|
| `hi` | Hidden file |
| `sy` | System file |
| `ro` | Read-only file |
| `jn` | Junction point |

**Styles:**

| Code | Style |
|------|-------|
| `0` | Reset/Normal |
| `1` | Bold |
| `2` | Dim |
| `3` | Italic |
| `4` | Underline |

**Foreground Colors:**

| Code | Color | Bright |
|------|-------|--------|
| `30` | Black | `90` |
| `31` | Red | `91` |
| `32` | Green | `92` |
| `33` | Yellow | `93` |
| `34` | Blue | `94` |
| `35` | Magenta | `95` |
| `36` | Cyan | `96` |
| `37` | White | `97` |

**256-Color Mode:**

```
38;5;N  # Foreground (N = 0-255)
48;5;N  # Background (N = 0-255)
```

**True Color (24-bit):**

```
38;2;R;G;B  # Foreground
48;2;R;G;B  # Background
```

### Default Color Scheme

```
di=1;34          # Directories: bold blue
ln=1;36          # Links: bold cyan
or=1;31;40       # Broken links: bold red on black
ex=1;32          # Executables: bold green
*.rs=0;33        # Rust files: yellow
*.py=0;33        # Python files: yellow
*.js=0;33        # JavaScript: yellow
*.md=0;36        # Markdown: cyan
*.txt=0           # Text: default
*.zip=1;33       # Archives: bold yellow
*.png=1;35       # Images: bold magenta
*.jpg=1;35
hi=2;37          # Hidden: dim white
sy=2;37          # System: dim white
```

---

## 🔤 Icon Reference

When using `--icons` with a Nerd Font, rtree displays icons based on file type, extension, or name.

### Directory Icons

| Directory | Icon | Description |
|-----------|------|-------------|
| (default) |  | Regular directory |
| `.git` |  | Git repository |
| `node_modules` |  | Node.js modules |
| `src` |  | Source code |
| `test`, `tests` |  | Test files |
| `docs` |  | Documentation |
| `.vscode` |  | VS Code settings |
| `.github` |  | GitHub files |
| `build`, `dist` |  | Build output |

### File Icons by Extension

| Extension | Icon | Category |
|-----------|------|----------|
| `.rs` |  | Rust |
| `.py` |  | Python |
| `.js` |  | JavaScript |
| `.ts` |  | TypeScript |
| `.go` |  | Go |
| `.java` |  | Java |
| `.c`, `.cpp` |  | C/C++ |
| `.html` |  | HTML |
| `.css` |  | CSS |
| `.json` |  | JSON |
| `.md` |  | Markdown |
| `.txt` |  | Text |
| `.pdf` |  | PDF |
| `.zip`, `.tar` |  | Archive |
| `.png`, `.jpg` |  | Image |
| `.mp3`, `.wav` |  | Audio |
| `.mp4`, `.mkv` |  | Video |
| `.exe` |  | Executable |
| `.dll` |  | Library |

### Special File Icons

| File | Icon | Description |
|------|------|-------------|
| `Cargo.toml` |  | Rust package |
| `package.json` |  | Node.js package |
| `Dockerfile` |  | Docker |
| `Makefile` |  | Make |
| `.gitignore` |  | Git config |
| `README` |  | Readme |
| `LICENSE` |  | License |

---

## 💡 Examples

### Basic Directory Listing

```powershell
PS> rtree C:\Projects\myapp

C:\Projects\myapp
├── src
│   ├── main.rs
│   ├── lib.rs
│   └── utils
│       └── helpers.rs
├── tests
│   └── integration.rs
├── Cargo.toml
└── README.md

3 directories, 5 files
```

### With File Sizes and Dates

```powershell
PS> rtree -h -D C:\Projects\myapp

C:\Projects\myapp
├── [ 4.2K]  [2025-01-15 10:30]  src
│   ├── [ 1.2K]  [2025-01-15 10:30]  main.rs
│   └── [  856]  [2025-01-14 15:20]  lib.rs
├── [  234]  [2025-01-10 09:00]  Cargo.toml
└── [ 2.1K]  [2025-01-15 10:00]  README.md

1 directory, 4 files
```

### Filtering by Pattern

```powershell
# Only Rust files
PS> rtree -P "*.rs" src/
src
├── main.rs
├── lib.rs
└── utils
    └── helpers.rs

# Exclude build directories
PS> rtree -I "target" -I "node_modules" .
```

### Export to Different Formats

```powershell
# JSON output
PS> rtree -J C:\Projects > project.json

# XML output  
PS> rtree -X C:\Projects > project.xml

# HTML with links
PS> rtree -H "file:///C:/Projects" -T "My Project" > project.html
```

### Windows-Specific Features

```powershell
# Show junction points
PS> rtree --show-junctions C:\Users
C:\Users
├── Default -> C:\Users\Default
├── Public
└── Username

# Show alternate data streams
PS> rtree --show-streams document.txt
document.txt
└── :Zone.Identifier:$DATA

# Show Windows attributes
PS> rtree -p --permissions=windows
├── [RHSA--]  system_file.dll
├── [--S---]  config.sys
└── [------]  readme.txt
```

### Russian Interface

```powershell
PS> rtree --lang=ru C:\Проекты

C:\Проекты
├── исходники
│   └── главный.rs
└── README.md

1 каталог, 2 файла
```

---

## 🔨 Building from Source

### Prerequisites

- Rust 1.75 or later
- Windows 10 1607+ or Windows 11
- (Optional) Nerd Font for icon support

### Build Steps

```powershell
# Clone the repository
git clone https://github.com/user/rtree.git
cd rtree

# Build in release mode
cargo build --release

# The binary will be at target/release/rtree.exe

# Install locally
cargo install --path .
```

### Build Options

```powershell
# Build with all features
cargo build --release --all-features

# Build with minimal features
cargo build --release --no-default-features

# Cross-compile for different architecture
cargo build --release --target x86_64-pc-windows-msvc
```

---

## 🧪 Testing

```powershell
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_natural_sort

# Run integration tests
cargo test --test cli_tests

# Run benchmarks
cargo bench
```

### Test Coverage

```powershell
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

---

## ⚡ Performance

rtree is optimized for handling large directory structures efficiently.

### Benchmarks

| Scenario | Files | Time |
|----------|-------|------|
| Small project | 100 | < 50ms |
| Medium project | 10,000 | < 1s |
| Large project | 100,000 | < 5s |
| Very large | 1,000,000 | < 30s |

### Optimization Techniques

- **Iterative traversal** — No stack overflow on deep directories
- **Streaming output** — Memory-efficient for large trees
- **Lazy metadata** — Only fetched when needed
- **Compiled patterns** — Fast glob matching
- **Icon caching** — O(1) icon lookups

---

## 📊 Comparison with GNU tree

| Feature | GNU tree | rtree |
|---------|----------|---------|
| Basic tree display | ✅ | ✅ |
| Color support | ✅ | ✅ |
| Pattern filtering | ✅ | ✅ |
| JSON output | ✅ | ✅ |
| XML output | ✅ | ✅ |
| HTML output | ✅ | ✅ |
| Nerd Font icons | ❌ | ✅ |
| NTFS junctions | ❌ | ✅ |
| Alternate Data Streams | ❌ | ✅ |
| Windows attributes | ❌ | ✅ |
| Long path support | ❌ | ✅ |
| UNC paths | Limited | ✅ |
| Multilingual | ❌ | ✅ |
| Native Windows | ❌ | ✅ |

---

## 🔧 Troubleshooting

### Icons not displaying correctly

1. Make sure you have a Nerd Font installed (e.g., "FiraCode Nerd Font")
2. Configure your terminal to use the Nerd Font
3. For Windows Terminal, add to settings.json:

   ```json
   "profiles": {
     "defaults": {
       "font": { "face": "FiraCode Nerd Font" }
     }
   }
   ```

### Colors not working

1. Make sure you're using Windows 10 1607 or later
2. Enable Virtual Terminal Processing:

   ```powershell
   # PowerShell
   $env:TERM = "xterm-256color"
   ```

3. Try forcing colors: `rtree -C`

### Permission denied errors

- Some directories (like `System Volume Information`) require administrator privileges
- Use `--hide-system` to skip system directories
- Run as Administrator for full access

### Long paths not working

1. Enable long path support in Windows:

   ```powershell
   # Run as Administrator
   Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1
   ```

2. Use `--long-paths` flag

### Output encoding issues

- rtree uses UTF-8 by default
- For legacy applications, use `--charset=cp1251` or `--charset=cp866`
- Set console to UTF-8: `chcp 65001`

---

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Report bugs** — Open an issue with detailed reproduction steps
2. **Request features** — Describe your use case
3. **Submit PRs** — Fork, branch, code, test, PR

### Development Setup

```powershell
# Fork and clone
git clone https://github.com/yourusername/rtree.git
cd rtree

# Create a branch
git checkout -b feature/my-feature

# Make changes and test
cargo test
cargo clippy
cargo fmt

# Commit and push
git commit -m "Add my feature"
git push origin feature/my-feature

# Open a Pull Request
```

### Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new features
- Update documentation

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2025 rtree contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 🙏 Acknowledgments

- [GNU tree](http://mama.indstate.edu/users/ice/tree/) — The original tree command
- [eza](https://github.com/eza-community/eza) — Inspiration for icons and colors
- [Nerd Fonts](https://www.nerdfonts.com/) — Beautiful icons
- Rust community — Amazing ecosystem

---

**Made with ❤️ and 🦀 Rust**
