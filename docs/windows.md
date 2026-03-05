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

Показать альтернативные потоки данных NTFS:

```powershell
rtree --show-streams
```

---

## Junction points

Показать цели точек соединения:

```powershell
rtree --show-junctions
```

---

## Скрытие системных файлов

По умолчанию флаг `-a` показывает все файлы, включая системные. Чтобы скрыть системные файлы даже при `-a`:

```powershell
rtree -a --hide-system
```

---

## Длинные пути (> 260 символов)

```powershell
rtree --long-paths "\\?\C:\Very\Long\Path\..."
```

> Для корректной работы включите поддержку длинных путей в реестре Windows. Подробнее: [troubleshooting.md](troubleshooting.md).

---

## Формат прав доступа

```powershell
# Windows-атрибуты (по умолчанию)
rtree -p --permissions windows

# POSIX-формат
rtree -p --permissions posix
```

**Пример вывода** (`--permissions windows`):

```
├── [RHSA--]  ntoskrnl.exe
├── [R-SA--]  hal.dll
└── [------]  readme.txt
```

Атрибуты: `R` — только чтение, `H` — скрытый, `S` — системный, `A` — архивный.

---

## UNC-пути

```powershell
rtree \\server\share\folder
```
