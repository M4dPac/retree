use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

#[path = "common/mod.rs"]
mod common;

use common::{run_rtree, tree_100, tree_100k, tree_10k};

// ============================================================================
// SMALL (100 files)
// ============================================================================

fn bench_small_100(c: &mut Criterion) {
    let t = tree_100();

    let mut group = c.benchmark_group("small_100_files");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(8));

    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel"]));
    });
    group.bench_function("par_2", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "2"]));
    });
    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "4"]));
    });
    group.bench_function("seq_color", |b| {
        b.iter(|| run_rtree(&t.path, &["-C"]));
    });
    group.bench_function("par_color", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "-C"]));
    });
    group.bench_function("streaming_plain", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming"]));
    });
    group.bench_function("streaming_color", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming", "-C"]));
    });

    group.finish();
}

// ============================================================================
// MEDIUM (10k files)
// ============================================================================

fn bench_medium_10k(c: &mut Criterion) {
    let t = tree_10k();

    let mut group = c.benchmark_group("medium_10k_files");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel"]));
    });
    group.bench_function("par_2", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "2"]));
    });
    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "4"]));
    });
    group.bench_function("par_8", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "8"]));
    });
    group.bench_function("streaming_plain", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming"]));
    });

    group.finish();
}

// ============================================================================
// LARGE (100k files)
// ============================================================================

fn bench_large_100k(c: &mut Criterion) {
    let t = tree_100k();

    let mut group = c.benchmark_group("large_100k_files");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(120));

    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel"]));
    });
    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "4"]));
    });
    group.bench_function("par_8", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "--threads", "8"]));
    });
    group.bench_function("streaming_plain", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming"]));
    });

    group.finish();
}

// ============================================================================
// OUTPUT FORMATS (reuses 10k tree)
// ============================================================================

fn bench_output_formats(c: &mut Criterion) {
    let t = tree_10k();

    let mut group = c.benchmark_group("output_formats_10k");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("text_seq", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });
    group.bench_function("text_par", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel"]));
    });
    group.bench_function("json_seq", |b| {
        b.iter(|| run_rtree(&t.path, &["-J"]));
    });
    group.bench_function("json_par", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "-J"]));
    });
    group.bench_function("xml_seq", |b| {
        b.iter(|| run_rtree(&t.path, &["-X"]));
    });
    group.bench_function("xml_par", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel", "-X"]));
    });

    group.finish();
}

// ============================================================================
// OPTIONS (reuses 10k tree)
// ============================================================================

fn bench_options(c: &mut Criterion) {
    let t = tree_10k();

    let mut group = c.benchmark_group("options_10k");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("plain", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });
    group.bench_function("sort_size", |b| {
        b.iter(|| run_rtree(&t.path, &["--sort", "size"]));
    });
    group.bench_function("sort_time", |b| {
        b.iter(|| run_rtree(&t.path, &["--sort", "mtime"]));
    });
    group.bench_function("dirs_only", |b| {
        b.iter(|| run_rtree(&t.path, &["-d"]));
    });
    group.bench_function("depth_2", |b| {
        b.iter(|| run_rtree(&t.path, &["-L", "2"]));
    });
    group.bench_function("depth_3", |b| {
        b.iter(|| run_rtree(&t.path, &["-L", "3"]));
    });
    group.bench_function("color", |b| {
        b.iter(|| run_rtree(&t.path, &["-C"]));
    });
    group.bench_function("icons", |b| {
        b.iter(|| run_rtree(&t.path, &["--icons", "always"]));
    });

    group.finish();
}

// ============================================================================
// STREAMING VS STANDARD (reuses 10k tree)
// ============================================================================

fn bench_streaming_comparison(c: &mut Criterion) {
    let t = tree_10k();

    let mut group = c.benchmark_group("streaming_vs_standard_10k");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });
    group.bench_function("streaming_plain", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming"]));
    });
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel"]));
    });
    group.bench_function("streaming_max100", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming", "--max-entries", "100"]));
    });
    group.bench_function("seq_max100", |b| {
        b.iter(|| run_rtree(&t.path, &["--max-entries", "100"]));
    });
    group.bench_function("seq_metadata", |b| {
        b.iter(|| run_rtree(&t.path, &["-s", "-D", "-p"]));
    });
    group.bench_function("streaming_metadata", |b| {
        b.iter(|| run_rtree(&t.path, &["--streaming", "-s", "-D", "-p"]));
    });

    group.finish();
}

// ============================================================================

criterion_group!(
    benches,
    bench_small_100,
    bench_medium_10k,
    bench_large_100k,
    bench_output_formats,
    bench_options,
    bench_streaming_comparison
);
criterion_main!(benches);
