# 🪟 Windows-специфика

rtree обеспечивает полную поддержку NTFS и Windows-специфичных возможностей файловой системы.

---

## Поддерживаемые возможности

| Возможность        | Описание                           |
| ------------------ | ---------------------------------- |
| ✅ Junction points | Точки соединения NTFS              |
| ✅ Symbolic links  | Символические ссылки               |
| ✅ Hard links      | Жёсткие ссылки                     |
| ✅ ADS             | Альтернативные потоки данных       |
| ✅ Атрибуты `RHSA` | Read-only, Hidden, System, Archive |
| ✅ Длинные пути    | Пути длиннее 260 символов          |
| ✅ UNC-пути        | `\\server\share\...`               |

---

## Альтернативные потоки данных (ADS)

```powershell
rtree --show-streams
```

---

## Длинные пути (> 260 символов)

```powershell
rtree --long-paths "\\?\C:\Very\Long\Path\..."
```

---

## Формат прав доступа

```powershell
# Windows-атрибуты (по умолчанию)
rtree -p --permissions=windows

# POSIX-формат
rtree -p --permissions=posix
```

**Пример вывода** (`--permissions=windows`):

```
├── [RHSA--]  ntoskrnl.exe
├── [R-SA--]  hal.dll
└── [------]  readme.txt
```
