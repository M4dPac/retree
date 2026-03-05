# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-05

### Added

- GNU tree-compatible directory listing
- Tree, flat and full-path display modes
- Filtering by glob pattern (`-P`, `-I`) with case-insensitive option
- Multiple sort modes: name, size, mtime, ctime, version (natural), unsorted
- Directories-first / files-first ordering
- Color output with auto/always/never modes
- File info: size (bytes & human-readable), date, permissions, owner, group, inode, device
- File type indicators (`-F`)
- Symlink following with loop detection
- Export: HTML (with hyperlinks), XML, JSON (compact & pretty)
- Parallel directory traversal (`--parallel`, `--threads`)
- Nerd Font / Unicode / ASCII icon styles
- CP437 and ANSI line-drawing graphics
- Depth limiting (`-L`) and file count limiting (`--filelimit`)
- Cross-filesystem boundary control (`-x`)
- Hidden and system file handling
- Windows-specific: NTFS alternate data streams, junction points, long path support, Windows ACL permissions
- Internationalization: English and Russian (`--lang`, `TREE_LANG`)
- Custom date format (`--timefmt`)
- File output (`-o`)
- Summary report with file/directory counts
