# 👨‍💻 Development

---

## Build

```bash
cargo build --release
```

---

## Tests

```bash
# Run all tests
cargo test

# With console output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

---

## Formatting and linting

```bash
# Format code
cargo fmt

# Linter
cargo clippy
```

---

## Benchmarks

retree uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for reproducible, statistically sound benchmarks.

```bash
# Run all benchmarks
cargo bench

# Save a baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

> ⚠️ Benchmarks run in `release` mode and may take a long time (especially `xlarge_1m`).

HTML reports are generated automatically at `target/criterion/report/index.html`.

---

## Project structure

```
.
├── benches
│   └── retree_perf.rs
├── Cargo.lock
├── Cargo.toml
├── docs
│   ├── en/                   # English documentation
│   ├── cli-reference.md
│   ├── colors.md
│   ├── configuration.md
│   ├── development.md
│   ├── icons.md
│   ├── performance.md
│   ├── README.md
│   ├── troubleshooting.md
│   └── windows.md
├── README.en.md
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
