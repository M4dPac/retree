# rtree 🌲

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](https://github.com/M4dPac/rtree)
[![GitHub](https://img.shields.io/github/v/release/M4dPac/rtree?label=latest)](https://github.com/M4dPac/rtree/releases)
[![Build](https://img.shields.io/badge/status-pre--release-yellow.svg)](https://github.com/M4dPac/rtree)

**rtree** — a modern GNU `tree`-compatible utility for displaying directory structures.  
Written in Rust. Optimized for Windows. Runs on Windows, Linux, and macOS.

[🇷🇺 Русская версия](README.md)

---

## 🎯 Why rtree?

- ✅ GNU `tree` compatibility
- ⚡ Parallel traversal (up to 6× faster on large trees)
- 🎨 `LS_COLORS` and `TREE_COLORS` support
- 🔤 Icons (Nerd Font / Unicode / ASCII)
- 📦 Export to JSON / XML / HTML
- 🪟 Full NTFS support (ADS, junctions, long paths)
- 🌍 English and Russian interface

---

## 📦 Installation

### Binary releases

Download the prebuilt binary from [GitHub Releases](https://github.com/M4dPac/rtree/releases), extract it, and add to `PATH`.

### Cargo

```bash
cargo install rtree
```

### Build from source

```bash
git clone https://github.com/M4dPac/rtree.git
cd rtree
cargo build --release
```

The binary will be at `target/release/rtree`.

---

## 🚀 Quick start

```bash
# Show current directory
rtree

# Show hidden files
rtree -a

# Limit depth
rtree -L 2

# Directories only
rtree -d

# Colors and icons
rtree -C --icons always

# JSON output
rtree -J > tree.json

# Pretty-printed JSON
rtree --json-pretty > tree.json

# Parallel mode (auto-detect threads)
rtree --parallel

# Parallel mode with explicit thread count
rtree --parallel --threads 4
```

---

## 📚 Usage

```
rtree [OPTIONS] [PATH...]
```

### Common options

| Flag             | Description                |
| ---------------- | -------------------------- |
| `-a`             | Show hidden files          |
| `-d`             | Directories only           |
| `-L N`           | Limit depth                |
| `-P PATTERN`     | Filter by glob             |
| `-I PATTERN`     | Exclude by glob            |
| `-h`             | Human-readable sizes       |
| `-D`             | Show modification date     |
| `-J`             | JSON output                |
| `--json-pretty`  | Pretty-printed JSON output |
| `-C`             | Always use color           |
| `--icons always` | Enable icons               |
| `--parallel`     | Parallel traversal         |
| `--threads N`    | Number of worker threads   |

### 📖 Full documentation

- 👉 [CLI Reference](docs/en/cli-reference.md)
- 🎨 [Color configuration](docs/en/colors.md)
- 🔤 [Icons](docs/en/icons.md)
- ⚡ [Performance](docs/en/performance.md)
- ⚙️ [Configuration](docs/en/configuration.md)
- 🪟 [Windows specifics](docs/en/windows.md)
- 🛠️ [Troubleshooting](docs/en/troubleshooting.md)

---

## ⚡ Performance

rtree uses Rayon (work-stealing), lazy metadata loading, optimized sorting, and streaming output.

Real benchmark results (median time, Criterion, `release` mode, Windows/NTFS, end-to-end):

| Files     | Sequential | Parallel (auto) | Streaming   |
| --------- | ---------- | --------------- | ----------- |
| 100       | ~54 ms     | ~14 ms          | ~54 ms      |
| 10 000    | ~5.3 s     | ~861 ms         | ~5.7 s      |
| 100 000   | ~51.5 s    | ~9.4 s          | ~53.5 s     |
| 1 000 000 | ~576 s     | ~102 s          | ~622 s           |

💾 **Memory usage** (PeakWorkingSet64):

| Files     | Sequential | Streaming | Savings  |
| --------- | ---------- | --------- | -------- |
| 10 000    | 15.6 MB   | 6.6 MB    | **58%**  |
| 100 000   | 100.2 MB  | 10.4 MB   | **90%**  |

> Parallel mode is effective starting at ~1 000 files (up to 6× speedup).
> Streaming mode does not build the tree in memory — 90%+ savings on large trees.

More details: 👉 [Benchmarks](docs/en/performance.md)

---

## 📊 Comparison with GNU tree

| Feature            | GNU tree | rtree |
| ------------------ | :------: | :---: |
| Colors             |    ✅    |  ✅   |
| JSON               |    ✅    |  ✅   |
| XML                |    ✅    |  ✅   |
| HTML               |    ✅    |  ✅   |
| Parallel traversal |    ❌    |  ✅   |
| Icons              |    ❌    |  ✅   |
| NTFS ADS           |    ❌    |  ✅   |
| Junction points    |    ❌    |  ✅   |
| Long paths         |    ❌    |  ✅   |
| Streaming output   |    ❌    |  ✅   |
| Multilingual UI    |    ❌    |  ✅   |

---

## 🗺️ Roadmap

- [ ] Stable release on crates.io
- [ ] Config file (`~/.rtreerc.toml`)
- [ ] `.gitignore` / `.treeignore` support
- [ ] Directory size aggregation (`--du`)
- [ ] Interactive mode
- [ ] Homebrew / Scoop / Winget packages

---

## 🤝 Contributing

PRs and issues are welcome.

```bash
cargo test
cargo fmt
cargo clippy
```

Details: 👉 [Development Guide](docs/en/development.md)

---

## 📄 License

[MIT License](LICENSE)

---

Made with ❤️ and 🦀 Rust
