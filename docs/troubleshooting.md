# 🛠️ Troubleshooting

---

## Иконки отображаются некорректно

Установите [Nerd Font](https://www.nerdfonts.com/) и выберите его в настройках терминала.

---

## Цвета не работают

Принудительно включите цвета флагом `-C`:

```powershell
rtree -C
```

---

## Ошибки доступа

Запустите терминал от имени администратора.

---

## Длинные пути (> 260 символов)

Включите поддержку длинных путей в реестре Windows:

```powershell
# Через реестр
reg add "HKLM\SYSTEM\CurrentControlSet\Control\FileSystem" /v LongPathsEnabled /t REG_DWORD /d 1 /f

# Или через PowerShell
Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name LongPathsEnabled -Value 1
```

Затем используйте флаг `--long-paths`:

```powershell
rtree --long-paths "\\?\C:\Very\Long\Path\..."
```

---

## Проблемы с кодировкой

Явно укажите кодировку:

```powershell
rtree --charset=utf-8
```
