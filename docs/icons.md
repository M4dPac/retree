# 🔤 Иконки

rt поддерживает 3 стиля иконок:

| Стиль     | Флаг                   |
| --------- | ---------------------- |
| Nerd Font | `--icon-style=nerd`    |
| Unicode   | `--icon-style=unicode` |
| ASCII     | `--icon-style=ascii`   |

Управление отображением:

```bash
# Включить иконки всегда
rt --icons always

# Отключить иконки
rt --no-icons

# Выбрать стиль
rt --icons always --icon-style unicode
```

---

## Nerd Font

> ⚠️ Для корректного отображения иконок Nerd Font необходимо установить совместимый шрифт и выбрать его в настройках терминала.  
> Скачать: https://www.nerdfonts.com/
>
> В текстовых редакторах и Markdown-просмотрщиках без Nerd Font ячейки таблиц ниже могут отображаться пустыми — это ожидаемое поведение.

---

## Иконки каталогов

| Каталог         | Иконка | Описание                |
| --------------- | ------ | ----------------------- |
| (по умолчанию)  |        | Обычный каталог         |
| `.git`          |        | Git-репозиторий         |
| `node_modules`  |        | Модули Node.js          |
| `src`           |        | Исходный код            |
| `test`, `tests` | 󰙨      | Тесты                   |
| `docs`          |        | Документация            |
| `.config`       |        | Конфигурация            |
| `.vscode`       |        | Настройки VS Code       |
| `.idea`         |        | Настройки IntelliJ      |
| `.github`       |        | Файлы GitHub            |
| `target`        |        | Результат сборки (Rust) |
| `build`         |        | Результат сборки        |
| `dist`          |        | Дистрибутив             |
| `bin`           |        | Исполняемые             |
| `lib`           |        | Библиотеки              |
| `vendor`        |        | Зависимости             |

### Windows-каталоги

| Каталог         | Иконка | Описание          |
| --------------- | ------ | ----------------- |
| `Windows`       |        | Каталог Windows   |
| `Program Files` |        | Программы         |
| `Users`         |        | Пользователи      |
| `Desktop`       |        | Рабочий стол      |
| `Documents`     |        | Документы         |
| `Downloads`     |        | Загрузки          |
| `Pictures`      |        | Изображения       |
| `Music`         |        | Музыка            |
| `Videos`        |        | Видео             |
| `AppData`       |        | Данные приложений |

---

## Иконки по расширению файлов

| Расширение                              | Иконка | Категория   |
| --------------------------------------- | ------ | ----------- |
| `.rs`                                   |        | Rust        |
| `.py`                                   |        | Python      |
| `.js`                                   |        | JavaScript  |
| `.ts`                                   |        | TypeScript  |
| `.jsx`, `.tsx`                          |        | React       |
| `.vue`                                  |        | Vue         |
| `.go`                                   |        | Go          |
| `.java`                                 |        | Java        |
| `.c`                                    |        | C           |
| `.cpp`, `.cc`                           |        | C++         |
| `.h`, `.hpp`                            |        | Header      |
| `.cs`                                   | 󰌛      | C#          |
| `.rb`                                   |        | Ruby        |
| `.php`                                  |        | PHP         |
| `.swift`                                |        | Swift       |
| `.kt`                                   |        | Kotlin      |
| `.lua`                                  |        | Lua         |
| `.zig`                                  |        | Zig         |
| `.sh`, `.bash`, `.zsh`                  |        | Shell       |
| `.ps1`                                  |        | PowerShell  |
| `.bat`, `.cmd`                          |        | Batch       |
| `.html`, `.htm`                         |        | HTML        |
| `.css`                                  |        | CSS         |
| `.scss`, `.sass`                        |        | Sass        |
| `.json`                                 |        | JSON        |
| `.yaml`, `.yml`                         |        | YAML        |
| `.xml`                                  |        | XML         |
| `.toml`                                 |        | TOML        |
| `.ini`, `.cfg`, `.conf`                 |        | Config      |
| `.md`                                   |        | Markdown    |
| `.txt`                                  |        | Text        |
| `.pdf`                                  |        | PDF         |
| `.doc`, `.docx`                         |        | Word        |
| `.xls`, `.xlsx`                         |        | Excel       |
| `.ppt`, `.pptx`                         |        | PowerPoint  |
| `.zip`, `.rar`, `.7z`, `.tar`, `.gz`    |        | Archive     |
| `.png`, `.jpg`, `.gif`, `.svg`, `.webp` |        | Image       |
| `.mp3`, `.wav`, `.flac`, `.ogg`         |        | Audio       |
| `.mp4`, `.mkv`, `.avi`, `.mov`, `.webm` |        | Video       |
| `.ttf`, `.otf`, `.woff`                 |        | Font        |
| `.sql`, `.db`, `.sqlite`                |        | Database    |
| `.exe`                                  |        | Executable  |
| `.msi`                                  |        | Installer   |
| `.dll`                                  |        | Library     |
| `.log`                                  |        | Log         |
| `.lock`                                 |        | Lock        |
| `.env`                                  |        | Environment |

---

## Unicode стиль

| Тип                  | Иконка |
| -------------------- | ------ |
| Каталог              | 📁     |
| Файл                 | 📄     |
| Символическая ссылка | 🔗     |
| Архив                | 📦     |
| Исполняемый          | ⚙️     |

---

## ASCII стиль

| Тип                  | Обозначение |
| -------------------- | ----------- |
| Каталог              | `[D]`       |
| Файл                 | `[F]`       |
| Символическая ссылка | `[L]`       |
