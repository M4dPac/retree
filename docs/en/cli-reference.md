# 📖 CLI Reference

```
rtree [OPTIONS] [PATH...]
```

If no path is given, the current directory is used. Multiple paths can be specified to display multiple trees.

---

## 🖥️ Listing Options

| Flag   | Long form         | Description                                     |
| ------ | ----------------- | ----------------------------------------------- |
| `-a`   | `--all`           | Show all files including hidden                 |
| `-d`   | `--dirs-only`     | List directories only                           |
| `-l`   | `--follow`        | Follow symbolic links                           |
| `-f`   | `--full-path`     | Print full path prefix                          |
| `-x`   | `--one-fs`        | Stay on current filesystem                      |
| `-L N` | `--level N`       | Descend only N levels deep                      |
|        | `--filelimit N`   | Skip dirs with more than N entries              |
|        | `--noreport`      | Omit final report                               |
|        | `--parallel`      | Enable parallel traversal                       |
|        | `--threads N`     | Number of worker threads                        |
|        | `--queue-cap N`   | Queue capacity per thread (default: 4096)       |
|        | `--streaming`     | Streaming text output (render during traversal) |
|        | `--max-entries N` | Limit total displayed entries                   |

> **Note:** `--streaming` applies to text output only. With `--prune` or structured formats (`-J`, `-X`, `-H`), standard traversal is used automatically.

---

## 🔍 Filtering

| Flag         | Long form           | Description                       |
| ------------ | ------------------- | --------------------------------- |
| `-P PATTERN` | `--pattern PATTERN` | List only files matching glob     |
| `-I PATTERN` | `--exclude PATTERN` | Exclude files matching glob       |
|              | `--matchdirs`       | Apply patterns to directories too |
|              | `--ignore-case`     | Case-insensitive pattern matching |
|              | `--prune`           | Do not print empty directories    |

### Glob syntax

| Symbol   | Description                       |
| -------- | --------------------------------- |
| `*`      | Any sequence of characters        |
| `?`      | Any single character              |
| `[abc]`  | Any character in the brackets     |
| `[a-z]`  | Any character in the range        |
| `[!abc]` | Any character NOT in the brackets |

---

## 🔢 Sorting

| Flag          | Long form        | Description                                         |
| ------------- | ---------------- | --------------------------------------------------- |
| `-v`          | `--version-sort` | Natural sort (`file2` before `file10`)              |
| `-t`          | `--timesort`     | Sort by modification time                           |
| `-c`          | `--ctime`        | Sort by metadata change time                        |
| `-U`          | `--unsorted`     | Leave files unsorted                                |
| `-r`          | `--reverse`      | Reverse sort order                                  |
|               | `--dirsfirst`    | List directories before files                       |
|               | `--filesfirst`   | List files before directories                       |
| `--sort TYPE` |                  | `name`, `size`, `mtime`, `ctime`, `version`, `none` |

---

## 🎨 Display

| Flag           | Long form        | Description               |
| -------------- | ---------------- | ------------------------- |
| `-i`           | `--noindent`     | No indentation lines      |
| `-A`           | `--ansi`         | Use ANSI line graphics    |
| `-S`           | `--cp437`        | Use CP437 line graphics   |
| `-n`           | `--nocolor`      | Turn colorization off     |
| `-C`           | `--color-always` | Always use color          |
| `--color WHEN` |                  | `auto`, `always`, `never` |

---

## 📄 File Information

| Flag            | Long form    | Description                                                       |
| --------------- | ------------ | ----------------------------------------------------------------- |
| `-s`            | `--size`     | Print size in bytes                                               |
| `-h`            | `--human`    | Print human-readable sizes (`1K`, `2M`, `3G`)                     |
|                 | `--si`       | Use SI units (powers of 1000)                                     |
| `-D`            | `--date`     | Print modification date                                           |
| `--timefmt FMT` |              | Time format string (`strftime` syntax), default: `%Y-%m-%d %H:%M` |
| `-p`            | `--perm`     | Print permissions / attributes                                    |
| `-u`            | `--uid`      | Print file owner                                                  |
| `-g`            | `--gid`      | Print file group                                                  |
|                 | `--inodes`   | Print inode number                                                |
|                 | `--device`   | Print device number                                               |
| `-F`            | `--classify` | Append `/` for dirs, `@` for links, `*` for executables           |
| `-q`            | `--safe`     | Replace non-printable chars with `?`                              |
| `-N`            | `--literal`  | Print non-printable chars as-is                                   |
| `--charset ENC` |              | Output character encoding (e.g. `utf-8`)                          |

---

## 📤 Export

| Flag       | Long form       | Description                |
| ---------- | --------------- | -------------------------- |
| `-o FILE`  | `--output FILE` | Write output to file       |
| `-J`       | `--json`        | JSON output                |
|            | `--json-pretty` | Pretty-printed JSON output |
| `-X`       | `--xml`         | XML output                 |
| `-H URL`   | `--html URL`    | HTML output with base URL  |
| `-T TITLE` | `--title TITLE` | HTML page title            |
|            | `--nolinks`     | Disable hyperlinks in HTML |
|            | `--hintro FILE` | Use custom HTML intro file |
|            | `--houtro FILE` | Use custom HTML outro file |

---

## 🔤 Icons

| Flag                 | Description                |
| -------------------- | -------------------------- |
| `--icons WHEN`       | `auto`, `always`, `never`  |
| `--no-icons`         | Disable icons              |
| `--icon-style STYLE` | `nerd`, `unicode`, `ascii` |

More details: [icons.md](icons.md)

---

## 🪟 Windows

| Flag                 | Description                      |
| -------------------- | -------------------------------- |
| `--show-streams`     | Show NTFS Alternate Data Streams |
| `--show-junctions`   | Show junction point targets      |
| `--hide-system`      | Hide system files                |
| `--permissions MODE` | `posix` / `windows`              |
| `--long-paths`       | Force `\\?\` long path prefix    |

---

## 🌍 Localization

| Flag          | Description                                      |
| ------------- | ------------------------------------------------ |
| `--lang LANG` | Interface language: `en` / `ru` (or `TREE_LANG`) |
