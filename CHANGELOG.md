# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- `--long-paths` now correctly applies the `\\?\` prefix from the root of traversal, preventing MAX_PATH errors on deeply nested trees
- `--one-fs` now reports an error when the volume serial of a directory cannot be determined, instead of silently descending into it
- Junction targets with non-UTF-8 (WTF-16) path components are now resolved correctly instead of being silently corrupted
- Cycle detection now uses OS file identity (volume serial + file ID) instead of path canonicalization, preventing missed cycles on Windows with junctions, UNC paths, and `\\?\` aliases

## [0.4.0] - 2026-03-19

### Added

- Support --show-streams to display NTFS Alternate Data Streams (Windows only)
- Display ADS entries with optional size and full-path rendering
- `--streaming` mode for text output: single-pass DFS rendering with lower memory usage
  - Supports `-a`, `-d`, `-L`, `-P`, `-I`, `-f`, `--filelimit`, `--max-entries`, `--one-fs`
  - Per-directory sorting, filtering, symlink/junction cycle detection, ADS support
  - `--prune` and structured formats (`-J`, `-X`, `-H`) fall back to standard traversal
  - Output truncated at `--max-entries` limit with stderr notification

### Windows

- Enumerate NTFS streams using Win32 FindFirstStreamW API
- Filter default ::$DATA stream automatically

### Changed

- --max-entries truncation now applies during tree flattening
- When entries equal the limit, output is no longer marked as truncated

### Fixed

- Skip Windows reserved device names (CON, NUL, PRN, COM1–9, LPT1–9) during traversal with a warning to stderr
- Reserved Windows device names no longer affect exit code (treated as warnings)
- Added internal recursion depth limit (4096) to prevent stack overflow on deeply nested directory trees

## [0.3.0] - 2026-03-12

### Added

- Stack-based streaming tree iterator with support for --max-entries
- Early termination of traversal when entry limit is reached
- New --max-entries option to limit total displayed entries
- BuildResult::truncated flag indicating output truncation
- Print stderr notification when output is truncated by --max-entries
- Localized help text for --max-entries (EN, RU)
- Support --charset option to control tree line style (ASCII, CP437, UTF-8)

### Changed

- Sequential traversal backend rewritten to use heap-based stack instead of recursion
- Sequential traversal now uses streaming iterator by default
- --prune now cascades through nested empty directories to match GNU tree behavior

### Performance

- Replace spin-wait directory limiter with Condvar-based backpressure to reduce CPU usage in parallel mode

### Fixed

- Prevent device identifier truncation by using 64-bit device and volume IDs (improves correctness of --one-fs handling)
- Eliminate OS stack overflow risk in sequential traversal on deep directory trees

## [0.2.0] - 2026-03-11

### Added

- add RFC 3986 compliant URI encoder for safe HTML href generation
- enable `--safe-print` automatically when output is a TTY (unless `--literal` is used)
- validate `--threads` and `--queue-cap` ranges and set safer default queue-cap (64)
- add optional `tree_compat` feature to enable GNU tree compatibility tests
- Centralized executable detection logic in platform module

### Performance

- reduce filesystem stat calls in dirs-first sorting
- add backpressure to parallel traversal to limit concurrent directory reads
- use custom rayon thread pool respecting `--threads`

### Fixed

- strip illegal control characters from XML output to ensure valid XML 1.0
- reject unsafe URL schemes (`javascript:`, `data:`, `vbscript:`) in `-H` HTML base option
- percent-encode file paths in HTML links to prevent URL injection
- sanitize Unicode bidi overrides and zero-width characters in `--safe-print` mode
- apply `--safe-print` sanitization to metadata fields in text output
- prevent stack overflow via internal depth limit in traversal
- recover from poisoned mutexes in parallel mode
- increase parallel worker stack size to match main thread and avoid premature stack overflow
- fix parallel symlink traversal where premature visited insert prevented descent
- Correct executable detection on Unix: now based on permission bits instead of file extension
- Prevent stack overflow on deep directory trees by running main logic on an 8MB stack (notably fixes Windows 1MB default stack limit)

## [0.1.4] - 2026-03-09

### Added

- sanitize control characters in text output when `--safe-print` is enabled

### Fixed

- make symlink recursion check atomic in parallel walker to prevent race conditions
- preserve correct Windows error when GetFileInformationByHandle fails
- validate Windows reparse point buffer length to prevent out-of-bounds reads
- avoid unnecessary DeviceIoControl calls when detecting junctions on Windows
- prevent descending into junctions unless `--show-junctions` is enabled
- escape single quotes in HTML output to prevent attribute injection
- validate ANSI color codes in LS_COLORS to prevent escape injection
- correctly handle non-UTF-8 file names in filtering and prune logic
- apply `--safe-print` sanitization to the entire formatted name
- prevent metadata loss in XML output when non-UTF-8 bytes are present
- flush output before exit to prevent data loss when writing to files

## [0.1.3] - 2026-03-09

### Fixed

- detect directory cycles (junctions, symlinks, mount points) and mark entries as recursive instead of descending infinitely
- ensure directory cycle detection works even when canonicalize fails by falling back to original path
- ensure file ID is collected when using `--one-fs` or `--show-device`
- correctly enforce `--one-fs` by skipping directories on different volumes
- use long path prefix for directory traversal when `--long-paths` is enabled
- make Windows long path conversion UTF-16 safe
- correctly handle UNC and device paths when using `--long-paths`

## [0.1.2] - 2026-03-08

### Fixed

- prevent exponential backtracking in glob pattern matcher
- avoid potential infinite recursion when symlink tracking lock fails
- fix panic in natural sort number parsing
- escape HTML and XML output to prevent injection issues
- harden JSON renderer against stack underflow panics

## [0.1.1] - 2026-03-05

### Added

- implement file owner and group resolution on Windows using Win32 Security API
- enable `-u` and `-g` flags support on Windows

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
