# 🛠️ Troubleshooting

---

## Иконки отображаются некорректно или видны пустые ячейки

Установите [Nerd Font](https://www.nerdfonts.com/) и выберите его в настройках терминала. Большинство терминалов (Windows Terminal, iTerm2, Alacritty, kitty) поддерживают смену шрифта в настройках.

Если Nerd Font не нужен, используйте Unicode или ASCII стиль:

```bash
rt --icon-style unicode
rt --icon-style ascii
```

---

## Цвета не работают

Принудительно включите цвета флагом `-C`:

```bash
rt -C
```

Также проверьте, что переменная `NO_COLOR` не установлена:

```bash
# Linux / macOS
unset NO_COLOR

# Windows PowerShell
Remove-Item Env:NO_COLOR
```

---

## Ошибки доступа к файлам или каталогам

Запустите терминал от имени администратора (Windows) или используйте `sudo` (Linux / macOS).

---

## Длинные пути (> 260 символов) на Windows

Включите поддержку длинных путей в реестре Windows:

```powershell
# Через PowerShell (от администратора)
Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name LongPathsEnabled -Value 1

# Или через реестр
reg add "HKLM\SYSTEM\CurrentControlSet\Control\FileSystem" /v LongPathsEnabled /t REG_DWORD /d 1 /f
```

Затем используйте флаг `--long-paths`:

```bash
rt --long-paths "\\?\C:\Very\Long\Path\..."
```

---

## Проблемы с кодировкой символов

Явно укажите кодировку:

```bash
rt --charset utf-8
```

На Windows убедитесь, что терминал использует UTF-8:

```powershell
# Установить кодировку консоли
chcp 65001
```

---

## Неверный язык интерфейса

Укажите язык явно через флаг или переменную окружения:

```bash
rt --lang en
rt --lang ru
```

```bash
# Linux / macOS
export TREE_LANG=en

# Windows PowerShell
$env:TREE_LANG = "en"
```
