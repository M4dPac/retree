# 🪟 Windows Specifics

rtree provides full NTFS support and handles Windows-specific filesystem features.

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
rtree --show-streams
```

---

## Junction points

Show junction point targets:

```powershell
rtree --show-junctions
```

---

## Hiding system files

By default, the `-a` flag shows all files including system files. To hide system files even when using `-a`:

```powershell
rtree -a --hide-system
```

---

## Long paths (> 260 characters)

```powershell
rtree --long-paths "\\?\C:\Very\Long\Path\..."
```

> To make this work correctly, enable long path support in the Windows registry. See [troubleshooting.md](troubleshooting.md) for details.

---

## Permission format

```powershell
# Windows attributes (default)
rtree -p --permissions windows

# POSIX format
rtree -p --permissions posix
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
rtree \\server\share\folder
```
