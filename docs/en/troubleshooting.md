# 🛠️ Troubleshooting

---

## Icons are not displayed or icon columns appear empty

Install a [Nerd Font](https://www.nerdfonts.com/) and select it in your terminal settings. Most terminals (Windows Terminal, iTerm2, Alacritty, kitty) support changing fonts in their settings.

If Nerd Font is not needed, use the Unicode or ASCII style instead:

```bash
rtree --icon-style unicode
rtree --icon-style ascii
```

---

## Colors are not working

Force color output with the `-C` flag:

```bash
rtree -C
```

Also check that the `NO_COLOR` variable is not set:

```bash
# Linux / macOS
unset NO_COLOR

# Windows PowerShell
Remove-Item Env:NO_COLOR
```

---

## Access errors for files or directories

Run the terminal as Administrator (Windows) or use `sudo` (Linux / macOS).

---

## Long paths (> 260 characters) on Windows

Enable long path support in the Windows registry:

```powershell
# Via PowerShell (as Administrator)
Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name LongPathsEnabled -Value 1

# Or via registry editor
reg add "HKLM\SYSTEM\CurrentControlSet\Control\FileSystem" /v LongPathsEnabled /t REG_DWORD /d 1 /f
```

Then use the `--long-paths` flag:

```bash
rtree --long-paths "\\?\C:\Very\Long\Path\..."
```

---

## Character encoding issues

Specify encoding explicitly:

```bash
rtree --charset utf-8
```

On Windows, make sure the terminal uses UTF-8:

```powershell
# Set console code page to UTF-8
chcp 65001
```

---

## Wrong interface language

Specify the language explicitly via flag or environment variable:

```bash
rtree --lang en
rtree --lang ru
```

```bash
# Linux / macOS
export TREE_LANG=en

# Windows PowerShell
$env:TREE_LANG = "en"
```
