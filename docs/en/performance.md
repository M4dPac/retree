# ⚡ Performance

rtree is optimized for efficient traversal of large directory trees.

---

## Parallel mode

```bash
# Auto-detect threads (based on CPU core count)
rtree --parallel

# Explicit thread count
rtree --parallel --threads 4

# Configure internal queue capacity
rtree --parallel --queue-cap 8192
```

Parallel mode uses [Rayon](https://github.com/rayon-rs/rayon) with a work-stealing algorithm for load balancing.

> Parallel mode is effective starting at ~10 000 files. On small trees, thread overhead may slightly increase total time.

---

## Benchmarks

**Run all benchmarks:**

```bash
cargo bench
```

**Save a baseline:**

```bash
cargo bench -- --save-baseline main
```

**Compare against baseline:**

```bash
cargo bench -- --baseline main
```

> Measurement: median time (Criterion) · Mode: `release`

---

## Results

### 📂 Directory traversal (plain output)

|     Files |     seq | par (auto) | par (4 threads) |
| --------: | ------: | ---------: | --------------: |
|       100 | 4.41 ms |    4.91 ms |         5.47 ms |
|    10 000 |  139 ms |    44.7 ms |         44.7 ms |
|   100 000 |  1.65 s |     416 ms |          391 ms |
| 1 000 000 |  17.5 s |      8.9 s |           8.4 s |

### 📦 Output formats (10 000 files)

| Format |    seq |     par |
| ------ | -----: | ------: |
| `text` | 146 ms | 46.7 ms |
| `json` | 139 ms | 40.8 ms |
| `xml`  | 153 ms | 53.6 ms |

---

## Criterion HTML reports

After running `cargo bench`, open in a browser:

```
target/criterion/report/index.html
```

The report includes: median, standard deviation, distribution plots, and regression analysis.

