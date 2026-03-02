# 👨‍💻 Разработка

---

## Сборка

```powershell
cargo build --release
```

---

## Тесты

```powershell
# Все тесты
cargo test

# С выводом
cargo test -- --nocapture

# Конкретный тест
cargo test test_name
```

---

## Форматирование и линтинг

```powershell
# Форматирование кода
cargo fmt

# Линтер
cargo clippy
```

---

## Бенчмарки

rtree использует [Criterion.rs](https://github.com/bheisler/criterion.rs) для воспроизводимых и статистически корректных бенчмарков.

#### ▶️ Запуск всех бенчмарков

```powershell
cargo bench
```

> ⚠️ Бенчмарки запускаются в `release`-режиме и могут занять значительное время (особенно `xlarge_1m`).

---

## Структура проекта

```

 src
├──  cli.rs
├──  config.rs
├──  error.rs
├──  filter
│   ├──  mod.rs
│   └──  pattern.rs
├──  format
│   ├──  html.rs
│   ├──  json.rs
│   ├──  mod.rs
│   ├──  text.rs
│   └──  xml.rs
├──  i18n
│   ├──  i18n_tests.rs
│   ├──  messages.rs
│   └──  mod.rs
├──  main.rs
├──  sorter
│   ├──  mod.rs
│   └──  natural.rs
├──  style
│   ├──  colors.rs
│   ├──  icons.rs
│   └──  mod.rs
├──  walker
│   ├──  engine.rs
│   ├──  entry.rs
│   ├──  iterator.rs
│   └──  mod.rs
└──  windows
    ├──  attributes.rs
    ├──  console.rs
    ├──  mod.rs
    ├──  permissions.rs
    ├──  reparse.rs
    └──  streams.rs
       └──  streams.rs
```
