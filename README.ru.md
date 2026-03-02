# rtree 🌲

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](https://github.com/M4dPac/rtree)
[![GitHub](https://img.shields.io/github/v/release/M4dPac/rtree?label=latest)](https://github.com/M4dPac/rtree/releases)
[![Build](https://img.shields.io/badge/status-pre--release-yellow.svg)](https://github.com/M4dPac/rtree)

**rtree** — современная GNU `tree`‑совместимая утилита для отображения структуры каталогов.  
Написана на Rust. Оптимизирована для Windows. Работает на Windows, Linux и macOS.

[🇬🇧 English version](README.md)

---

## 🎯 Почему rtree?

- ✅ Совместимость с GNU `tree`
- ⚡ Параллельный обход (до 10–17× быстрее на больших деревьях)
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
rtree -C --icons

# JSON-вывод
rtree -J > tree.json

# Параллельный режим
rtree --parallel
```

---

## 📚 Использование

```
rtree [OPTIONS] [PATH...]
```

### Часто используемые опции

| Флаг         | Описание                 |
| ------------ | ------------------------ |
| `-a`         | Показать скрытые файлы   |
| `-d`         | Только каталоги          |
| `-L N`       | Ограничить глубину       |
| `-P PATTERN` | Фильтр по glob           |
| `-I PATTERN` | Исключить по glob        |
| `-h`         | Человекочитаемые размеры |
| `-D`         | Дата модификации         |
| `-J`         | JSON-вывод               |
| `-C`         | Цвет всегда              |
| `--icons`    | Включить иконки          |
| `--parallel` | Параллельный обход       |

### 📖 Полная документация

- 👉 [CLI Reference](docs/cli-reference.md)
- 🎨 [Настройка цветов](docs/colors.md)
- 🔤 [Иконки](docs/icons.md)
- ⚡ [Производительность](docs/performance.md)
- 🪟 [Windows-специфика](docs/windows.md)
- 🛠️ [Troubleshooting](docs/troubleshooting.md)

---

## ⚡ Производительность

rtree использует Rayon (work-stealing), ленивую загрузку метаданных, оптимизированную сортировку и потоковый вывод.

| Файлов    | Обычный режим | Параллельный |
| --------- | ------------- | ------------ |
| 10 000    | ~200 мс       | ~50 мс       |
| 100 000   | ~2 с          | ~500 мс      |
| 1 000 000 | ~15 с         | ~5 с         |

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
| Многоязычность     |    ❌    |  ✅   |

---

## 🗺️ Roadmap

- [ ] Стабильный релиз на crates.io
- [ ] Конфигурационный файл (`~/.rtreerc.toml`)
- [ ] Поддержка `.gitignore`
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
