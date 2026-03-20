# rtree 🌲

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](https://github.com/M4dPac/rtree)
[![GitHub](https://img.shields.io/github/v/release/M4dPac/rtree?label=latest)](https://github.com/M4dPac/rtree/releases)
[![Build](https://img.shields.io/badge/status-pre--release-yellow.svg)](https://github.com/M4dPac/rtree)

**rtree** — современная GNU `tree`-совместимая утилита для отображения структуры каталогов.  
Написана на Rust. Оптимизирована для Windows. Работает на Windows, Linux и macOS.

[🇬🇧 English version](README.en.md)

---

## 🎯 Почему rtree?

- ✅ Совместимость с GNU `tree`
- ⚡ Параллельный обход (до 6× быстрее на больших деревьях)
- 🎨 Поддержка `LS_COLORS` и `TREE_COLORS`
- 🔤 Иконки (Nerd Font / Unicode / ASCII)
- 📦 Экспорт в JSON / XML / HTML
- 🪟 Полная поддержка NTFS (ADS, junctions, длинные пути)
- 🌍 Русский и английский интерфейс

---

## 📦 Установка

### Бинарные релизы

Скачайте готовый бинарник с [GitHub Releases](https://github.com/M4dPac/rtree/releases), распакуйте и добавьте в `PATH`.

### Cargo

```bash
cargo install rtree
```

### Сборка из исходников

```bash
git clone https://github.com/M4dPac/rtree.git
cd rtree
cargo build --release
```

Бинарный файл будет в `target/release/rtree`.

---

## 🚀 Быстрый старт

```bash
# Показать текущий каталог
rtree

# Показать скрытые файлы
rtree -a

# Ограничить глубину
rtree -L 2

# Только каталоги
rtree -d

# Цвета и иконки
rtree -C --icons always

# JSON-вывод
rtree -J > tree.json

# Форматированный JSON
rtree --json-pretty > tree.json

# Параллельный режим (авто-определение потоков)
rtree --parallel

# Параллельный режим с явным числом потоков
rtree --parallel --threads 4
```

---

## 📚 Использование

```
rtree [OPTIONS] [PATH...]
```

### Часто используемые опции

| Флаг             | Описание                   |
| ---------------- | -------------------------- |
| `-a`             | Показать скрытые файлы     |
| `-d`             | Только каталоги            |
| `-L N`           | Ограничить глубину         |
| `-P PATTERN`     | Фильтр по glob             |
| `-I PATTERN`     | Исключить по glob          |
| `-h`             | Человекочитаемые размеры   |
| `-D`             | Дата модификации           |
| `-J`             | JSON-вывод                 |
| `--json-pretty`  | Форматированный JSON-вывод |
| `-C`             | Цвет всегда                |
| `--icons always` | Включить иконки            |
| `--parallel`     | Параллельный обход         |
| `--threads N`    | Число рабочих потоков      |

### 📖 Полная документация

- 👉 [CLI Reference](docs/cli-reference.md)
- 🎨 [Настройка цветов](docs/colors.md)
- 🔤 [Иконки](docs/icons.md)
- ⚡ [Производительность](docs/performance.md)
- ⚙️ [Конфигурация](docs/configuration.md)
- 🪟 [Windows-специфика](docs/windows.md)
- 🛠️ [Troubleshooting](docs/troubleshooting.md)

---

## ⚡ Производительность

rtree использует Rayon (work-stealing), ленивую загрузку метаданных, оптимизированную сортировку и потоковый вывод.

Результаты реальных бенчмарков (median time, Criterion, режим `release`, Windows/NTFS, end-to-end):

| Файлов    | Обычный режим | Параллельный (авто) | Streaming   |
| --------- | ------------- | ------------------- | ----------- |
| 100       | ~54 мс        | ~14 мс              | ~54 мс      |
| 10 000    | ~5.3 с        | ~861 мс             | ~5.7 с      |
| 100 000   | ~51.5 с       | ~9.4 с              | ~53.5 с     |
| 1 000 000 | ~576 с        | ~102 с              | —           |

💾 **Потребление памяти** (PeakWorkingSet64):

| Файлов    | Sequential | Streaming | Экономия |
| --------- | ---------- | --------- | -------- |
| 10 000    | 15.6 МБ   | 6.6 МБ   | **58%**  |
| 100 000   | 100.2 МБ  | 10.4 МБ  | **90%**  |

> Параллельный режим эффективен начиная с ~1 000 файлов (ускорение до 6×).
> Streaming-режим не строит дерево в памяти — экономия 90%+ на больших деревьях.

Подробнее: 👉 [Бенчмарки](docs/performance.md)

---

## 📊 Сравнение с GNU tree

| Возможность        | GNU tree | rtree |
| ------------------ | :------: | :---: |
| Цвета              |    ✅    |  ✅   |
| JSON               |    ✅    |  ✅   |
| XML                |    ✅    |  ✅   |
| HTML               |    ✅    |  ✅   |
| Параллельный обход |    ❌    |  ✅   |
| Иконки             |    ❌    |  ✅   |
| NTFS ADS           |    ❌    |  ✅   |
| Junction points    |    ❌    |  ✅   |
| Длинные пути       |    ❌    |  ✅   |
| Потоковый вывод    |    ❌    |  ✅   |
| Многоязычность     |    ❌    |  ✅   |

---

## 🗺️ Roadmap

- [ ] Стабильный релиз на crates.io
- [ ] Конфигурационный файл (`~/.rtreerc.toml`)
- [ ] Поддержка `.gitignore` / `.treeignore`
- [ ] Суммарный размер каталогов (`--du`)
- [ ] Интерактивный режим
- [ ] Пакеты для Homebrew / Scoop / Winget

---

## 🤝 Вклад в проект

PR и issues приветствуются.

```bash
cargo test
cargo fmt
cargo clippy
```

Подробности: 👉 [Development Guide](docs/development.md)

---

## 📄 Лицензия

[MIT License](LICENSE)

---

Сделано с ❤️ и 🦀 Rust
