# 🪟 Windows Specifics

rt provides full NTFS support and handles Windows-specific filesystem features.

---

## Supported features

| Feature              | Description                        |
| -------------------- | ---------------------------------- |
| ✅ Junction points   | NTFS junction points               |
| ✅ Symbolic links    | Symbolic links                     |
| ✅ Hard links        | Hard links                         |
| ✅ ADS               | Alternate Data Streams             |
| ✅ `RHSA` attributes | Read-only, Hidden, System, Archive |
| ✅ Long paths        | Paths longer than 260 characters   |
| ✅ UNC paths         | `\\server\share\...`               |

---

## Alternate Data Streams (ADS)

Show NTFS Alternate Data Streams:

```powershell
rt --show-streams
```

---

## Junction points

Show junction point targets:

```powershell
rt --show-junctions
```

---

## Hiding system files

By default, the `-a` flag shows all files including system files. To hide system files even when using `-a`:

```powershell
rt -a --hide-system
```

---

## Long paths (> 260 characters)

```powershell
rt --long-paths "\\?\C:\Very\Long\Path\..."
```

> To make this work correctly, enable long path support in the Windows registry. See [troubleshooting.md](troubleshooting.md) for details.

---

## Permission format

```powershell
# Windows attributes (default)
rt -p --permissions windows

# POSIX format
rt -p --permissions posix
```

**Example output** (`--permissions windows`):

```
├── [RHSA--]  ntoskrnl.exe
├── [R-SA--]  hal.dll
└── [------]  readme.txt
```

Attributes: `R` — read-only, `H` — hidden, `S` — system, `A` — archive.

---

## UNC paths

```powershell
rt \\server\share\folder
```

---

## Reserved device names

Windows reserves certain filenames as device names: `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`. Names with extensions (e.g. `NUL.txt`) are also reserved.

**Behavior:** On Windows, retree detects reserved names during traversal and **skips** them with a warning to stderr. This prevents the Win32 API from opening a device handle instead of a file, which could return incorrect metadata.

**On Linux/macOS:** These are valid filenames and are listed normally.

**How can such files exist on NTFS?**
Only via WSL, `\\?\` prefix bypass, or third-party tools.

> The detection function `is_reserved_windows_name()` is available in `retree::platform` for external use.
