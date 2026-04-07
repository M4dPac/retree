# 👨‍💻 Разработка

---

## Сборка

```bash
cargo build --release
```

---

## Тесты

```bash
# Все тесты
cargo test

# С выводом в консоль
cargo test -- --nocapture

# Конкретный тест
cargo test test_name
```

---

## Форматирование и линтинг

```bash
# Форматирование кода
cargo fmt

# Линтер
cargo clippy
```

---

## Бенчмарки

retree использует [Criterion.rs](https://github.com/bheisler/criterion.rs) для воспроизводимых и статистически корректных бенчмарков.

```bash
# Запуск всех бенчмарков
cargo bench

# Сохранение baseline
cargo bench -- --save-baseline main

# Сравнение с baseline
cargo bench -- --baseline main
```

> ⚠️ Бенчмарки запускаются в `release`-режиме и могут занять значительное время (особенно `xlarge_1m`).

HTML-отчёты генерируются автоматически в `target/criterion/report/index.html`.

---

## Структура проекта

```
.
├── benches
│   └── retree_perf.rs
├── Cargo.lock
├── Cargo.toml
├── docs
│   ├── en/                   # Документация на английском
│   ├── cli-reference.md
│   ├── colors.md
│   ├── configuration.md
│   ├── development.md
│   ├── icons.md
│   ├── performance.md
│   ├── README.md
│   ├── troubleshooting.md
│   └── windows.md
├── README.ru.md
├── README.md
├── src
│   ├── app
│   │   ├── context.rs
│   │   ├── mod.rs
│   │   └── run.rs
│   ├── cli
│   │   ├── args.rs
│   │   ├── early_lang.rs
│   │   ├── localized.rs
│   │   └── mod.rs
│   ├── config
│   │   ├── env.rs
│   │   ├── mod.rs
│   │   └── options.rs
│   ├── core
│   │   ├── entry.rs
│   │   ├── filter
│   │   │   ├── mod.rs
│   │   │   └── pattern.rs
│   │   ├── mod.rs
│   │   ├── sorter
│   │   │   ├── mod.rs
│   │   │   └── natural.rs
│   │   ├── tree.rs
│   │   └── walker
│   │       ├── engine.rs
│   │       ├── entry.rs
│   │       ├── iterator.rs
│   │       └── mod.rs
│   ├── error.rs
│   ├── i18n
│   │   ├── i18n_tests.rs
│   │   ├── messages.rs
│   │   └── mod.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── platform
│   │   ├── mod.rs
│   │   ├── unix.rs
│   │   └── windows
│   │       ├── attributes.rs
│   │       ├── console.rs
│   │       ├── locale.rs
│   │       ├── mod.rs
│   │       ├── permissions.rs
│   │       ├── reparse.rs
│   │       └── streams.rs
│   ├── render
│   │   ├── context.rs
│   │   ├── helpers.rs
│   │   ├── html.rs
│   │   ├── json.rs
│   │   ├── mod.rs
│   │   ├── text.rs
│   │   ├── traits.rs
│   │   └── xml.rs
│   └── style
│       ├── colors.rs
│       ├── icons.rs
│       └── mod.rs
└── tests
    ├── basic.rs
    ├── common
    │   └── mod.rs
    ├── display.rs
    ├── errors.rs
    ├── filtering.rs
    ├── i18n.rs
    ├── output_format.rs
    ├── parallel.rs
    ├── sorting.rs
    └── tree_compat.rs
```
