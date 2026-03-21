# ⚡ Performance

rtree is optimized for efficient traversal of large directory trees.

> **Test environment:** Windows, NTFS, end-to-end measurement (including process startup).  
> Times are median values from Criterion / hyperfine.

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

Parallel mode uses Rayon with a work-stealing algorithm for load balancing.

> **Note:** parallel mode is effective starting at ~1 000 files. On small trees, thread overhead may slightly increase total time.

---

## Benchmark results

### 📂 Directory traversal (plain output, `--noreport`)

| Files     | `seq`   | `par` (auto) | `streaming` |
|-----------|---------|--------------|-------------|
| 100       | 54 ms   | 14 ms        | 54 ms       |
| 10 000    | 5.3 s   | 861 ms       | 5.7 s       |
| 100 000   | 51.5 s  | 9.4 s        | 53.5 s      |
| 1 000 000 | ~576 s  | ~102 s       | 622 s           |

### 🧵 Thread scaling (10 000 files)

| Threads  | Time    | Speedup |
|----------|---------|---------|
| 1 (seq)  | 5.3 s   | 1.0×    |
| 2        | 2.95 s  | 1.8×    |
| 4        | 1.46 s  | 3.6×    |
| 8        | 940 ms  | 5.6×    |
| auto     | 861 ms  | 6.2×    |

### 🧵 Thread scaling (1 000 000 files, hyperfine)

| Threads |   Time   | Speedup |
| ------: | -------: | ------: |
|  1 (seq)| ~493 s   |    1.0× |
|       2 | ~412 s   |    1.2× |
|       4 | ~151 s   |    3.3× |
|       8 | ~102 s   |    4.8× |

### 📦 Output formats (10 000 files)

| Format | `seq`  | `par`   |
|--------|--------|---------|
| text   | 5.6 s  | 863 ms  |
| json   | 5.6 s  | 863 ms  |
| xml    | 5.6 s  | 866 ms  |

> Output format has virtually no impact on performance — the bottleneck is filesystem traversal.

### ⚙️ Options impact (10 000 files, seq)

| Option               | Time    |
|----------------------|---------|
| plain                | 5.6 s   |
| `--sort size`        | 5.6 s   |
| `--sort mtime`       | 5.6 s   |
| `-C` (color)         | 5.6 s   |
| `--icons`            | 5.6 s   |
| `-d` (dirs only)     | 1.25 s  |
| `-L 2` (depth 2)     | 75.5 ms |
| `-L 3` (depth 3)     | 255 ms  |

> Sorting, color, and icons add no measurable overhead. Depth limiting (`-L`) provides proportional speedup.

### 🌊 Streaming vs Standard (10 000 files)

| Mode                            | Time   |
|---------------------------------|--------|
| seq                             | 5.6 s  |
| streaming                       | 5.6 s  |
| par                             | 866 ms |
| streaming + `--max-entries 100` | 53 ms  |
| seq + `--max-entries 100`       | 5.6 s  |

> **Key difference:** `--max-entries` in streaming mode enables early termination — traversal stops after outputting N entries. In standard mode, the full tree is built first, then truncated.

### 💾 Memory usage

Measurement: `PeakWorkingSet64` (Windows Task Manager → "Peak Working Set").

| Files   | Sequential | Streaming | Parallel | Streaming savings |
|---------|------------|-----------|----------|-------------------|
| 10 000  | 15.6 MB    | 6.6 MB    | 20.8 MB    | **58%**             |
| 100 000 | 100.2 MB   | 10.4 MB   | 98.8 MB    | **90%**             |
| 1 000 000 |  938.9 MB  |  59.0 MB  | 828 MB   | **94%**             |

> Streaming mode does not build the tree in memory — it outputs entries as they are discovered. On large trees, memory savings reach 90%+.

---

## Running benchmarks

### Criterion (up to 100k files, ~10–15 min)

```bash
# Main suite (100 / 10k / 100k + formats + options + streaming)
cargo bench --bench rtree_perf

# Save baseline for regression tracking
cargo bench --bench rtree_perf -- --save-baseline main

# Compare against baseline
cargo bench --bench rtree_perf -- --baseline main
```

### Criterion (1M files, ~2 hours)

```bash
cargo bench --bench rtree_perf_xlarge
```

### Hyperfine (1M files, ~3–5 min, recommended)

```powershell
# Requires: winget install sharkdp.hyperfine
.\benches\bench_xlarge.ps1
```

### Memory measurement

```powershell
# All sizes
.\benches\bench_memory.ps1

# Specific sizes
.\benches\bench_memory.ps1 -Sizes 10k,100k
.\benches\bench_memory.ps1 -Sizes 1m -Runs 5
```

---

## Managing test trees

Trees are created once in `target/bench_trees/` and reused across runs.

```powershell
# Recreate all trees
Remove-Item -Recurse target\bench_trees

# Recreate specific tree
Remove-Item -Recurse target\bench_trees\medium_10k
```

---

## Criterion HTML reports

After running benchmarks, reports are available at:

```
target/criterion/report/index.html
```

Reports include: median, standard deviation, distribution plots, and regression analysis.
