# 🎨 Color Configuration

retree supports the following environment variables:

- `LS_COLORS` — standard GNU ls format
- `TREE_COLORS` — takes priority over `LS_COLORS`
- `NO_COLOR` — disables all colors (any value)

---

## Examples

**Linux / macOS:**

```bash
export LS_COLORS="di=1;34:*.rs=0;33"
export TREE_COLORS="di=1;34:ex=1;32:*.rs=1;33"
```

**Windows PowerShell:**

```powershell
$env:LS_COLORS = "di=1;34:*.rs=0;33"
$env:TREE_COLORS = "di=1;34:ex=1;32:*.rs=1;33"
```

---

## Format

```
TYPE=STYLE;FG;BG
```

Multiple types are separated by colons: `di=1;34:*.rs=0;33`.

### Types

| Code    | Description        |
| ------- | ------------------ |
| `di`    | Directory          |
| `fi`    | Regular file       |
| `ln`    | Symbolic link      |
| `or`    | Broken link        |
| `ex`    | Executable         |
| `hi`    | Hidden (Windows)   |
| `sy`    | System (Windows)   |
| `*.ext` | Files by extension |

### Styles

| Code | Style     |
| ---- | --------- |
| `0`  | Normal    |
| `1`  | Bold      |
| `2`  | Dim       |
| `3`  | Italic    |
| `4`  | Underline |

---

## Colors

**Standard** (`30–37`) and **bright** (`90–97`):

| Code | Color   | Bright |
| ---- | ------- | ------ |
| `30` | Black   | `90`   |
| `31` | Red     | `91`   |
| `32` | Green   | `92`   |
| `33` | Yellow  | `93`   |
| `34` | Blue    | `94`   |
| `35` | Magenta | `95`   |
| `36` | Cyan    | `96`   |
| `37` | White   | `97`   |

**256 colors:**

```
38;5;N        # foreground
48;5;N        # background
```

**True Color (24-bit):**

```
38;2;R;G;B    # foreground
48;2;R;G;B    # background
```

---

## Default colors

```
di=1;34       # Directories: bold blue
ln=1;36       # Links: bold cyan
or=1;31;40    # Broken links: bold red on black
ex=1;32       # Executables: bold green
fi=0          # Regular files: terminal default
hi=2;37       # Hidden (Windows): dim white
sy=2;37       # System (Windows): dim white

*.rs=0;33     *.py=0;33     *.js=0;33
*.zip=1;33    *.tar=1;33    *.gz=1;33
*.png=1;35    *.jpg=1;35    *.gif=1;35
*.exe=1;32    *.bat=1;32    *.ps1=1;32
```
