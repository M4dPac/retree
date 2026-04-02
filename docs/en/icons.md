# 🔤 Icons

rt supports 3 icon styles:

| Style     | Flag                   |
| --------- | ---------------------- |
| Nerd Font | `--icon-style=nerd`    |
| Unicode   | `--icon-style=unicode` |
| ASCII     | `--icon-style=ascii`   |

Controlling icon display:

```bash
# Always show icons
rt --icons always

# Disable icons
rt --no-icons

# Choose a style
rt --icons always --icon-style unicode
```

---

## Nerd Font

> ⚠️ To display Nerd Font icons correctly, you must install a compatible font and select it in your terminal settings.  
> Download: https://www.nerdfonts.com/
>
> In text editors and Markdown viewers without Nerd Font, the icon columns in the tables below may appear empty — this is expected.

---

## Directory icons

| Directory       | Icon | Description         |
| --------------- | ---- | ------------------- |
| (default)       |      | Regular directory   |
| `.git`          |      | Git repository      |
| `node_modules`  |      | Node.js modules     |
| `src`           |      | Source code         |
| `test`, `tests` | 󰙨    | Tests               |
| `docs`          |      | Documentation       |
| `.config`       |      | Configuration       |
| `.vscode`       |      | VS Code settings    |
| `.idea`         |      | IntelliJ settings   |
| `.github`       |      | GitHub files        |
| `target`        |      | Build output (Rust) |
| `build`         |      | Build output        |
| `dist`          |      | Distribution        |
| `bin`           |      | Executables         |
| `lib`           |      | Libraries           |
| `vendor`        |      | Dependencies        |

### Windows directories

| Directory       | Icon | Description       |
| --------------- | ---- | ----------------- |
| `Windows`       |      | Windows directory |
| `Program Files` |      | Programs          |
| `Users`         |      | Users             |
| `Desktop`       |      | Desktop           |
| `Documents`     |      | Documents         |
| `Downloads`     |      | Downloads         |
| `Pictures`      |      | Pictures          |
| `Music`         |      | Music             |
| `Videos`        |      | Videos            |
| `AppData`       |      | App data          |

---

## File extension icons

| Extension                               | Icon | Category    |
| --------------------------------------- | ---- | ----------- |
| `.rs`                                   |      | Rust        |
| `.py`                                   |      | Python      |
| `.js`                                   |      | JavaScript  |
| `.ts`                                   |      | TypeScript  |
| `.jsx`, `.tsx`                          |      | React       |
| `.vue`                                  |      | Vue         |
| `.go`                                   |      | Go          |
| `.java`                                 |      | Java        |
| `.c`                                    |      | C           |
| `.cpp`, `.cc`                           |      | C++         |
| `.h`, `.hpp`                            |      | Header      |
| `.cs`                                   | 󰌛    | C#          |
| `.rb`                                   |      | Ruby        |
| `.php`                                  |      | PHP         |
| `.swift`                                |      | Swift       |
| `.kt`                                   |      | Kotlin      |
| `.lua`                                  |      | Lua         |
| `.zig`                                  |      | Zig         |
| `.sh`, `.bash`, `.zsh`                  |      | Shell       |
| `.ps1`                                  |      | PowerShell  |
| `.bat`, `.cmd`                          |      | Batch       |
| `.html`, `.htm`                         |      | HTML        |
| `.css`                                  |      | CSS         |
| `.scss`, `.sass`                        |      | Sass        |
| `.json`                                 |      | JSON        |
| `.yaml`, `.yml`                         |      | YAML        |
| `.xml`                                  |      | XML         |
| `.toml`                                 |      | TOML        |
| `.ini`, `.cfg`, `.conf`                 |      | Config      |
| `.md`                                   |      | Markdown    |
| `.txt`                                  |      | Text        |
| `.pdf`                                  |      | PDF         |
| `.doc`, `.docx`                         |      | Word        |
| `.xls`, `.xlsx`                         |      | Excel       |
| `.ppt`, `.pptx`                         |      | PowerPoint  |
| `.zip`, `.rar`, `.7z`, `.tar`, `.gz`    |      | Archive     |
| `.png`, `.jpg`, `.gif`, `.svg`, `.webp` |      | Image       |
| `.mp3`, `.wav`, `.flac`, `.ogg`         |      | Audio       |
| `.mp4`, `.mkv`, `.avi`, `.mov`, `.webm` |      | Video       |
| `.ttf`, `.otf`, `.woff`                 |      | Font        |
| `.sql`, `.db`, `.sqlite`                |      | Database    |
| `.exe`                                  |      | Executable  |
| `.msi`                                  |      | Installer   |
| `.dll`                                  |      | Library     |
| `.log`                                  |      | Log         |
| `.lock`                                 |      | Lock        |
| `.env`                                  |      | Environment |

---

## Unicode style

| Type          | Icon |
| ------------- | ---- |
| Directory     | 📁   |
| File          | 📄   |
| Symbolic link | 🔗   |
| Archive       | 📦   |
| Executable    | ⚙️   |

---

## ASCII style

| Type          | Marker |
| ------------- | ------ |
| Directory     | `[D]`  |
| File          | `[F]`  |
| Symbolic link | `[L]`  |
