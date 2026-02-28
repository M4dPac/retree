use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn generate_test_tree(base: &Path, num_files: usize) -> io::Result<()> {
    for i in 0..num_files {
        let file = base.join(format!("file_{:06}.txt", i));
        fs::write(file, b"x")?;
    }
    Ok(())
}

fn run_rtree(path: &Path, extra_args: &[&str]) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rtree"));
    cmd.arg(path)
        .arg("--noreport")
        .args(extra_args)
        .stdout(std::process::Stdio::null());

    let status = cmd.status().expect("failed to execute rtree");
    assert!(status.success());
}

fn bench_small(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 100).unwrap();

    let mut group = c.benchmark_group("small_100_files");
    group.bench_function("plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });
    group.bench_function("color", |b| {
        b.iter(|| run_rtree(temp.path(), &["-C"]));
    });
    group.finish();
}

fn bench_medium(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 10_000).unwrap();

    let mut group = c.benchmark_group("medium_10k_files");
    group.sample_size(20);
    group.bench_function("plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });
    group.bench_function("json", |b| {
        b.iter(|| run_rtree(temp.path(), &["-J"]));
    });
    group.finish();
}

fn bench_large(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 100_000).unwrap();

    let mut group = c.benchmark_group("large_100k_files");
    group.sample_size(10);
    group.bench_function("plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });
    group.finish();
}

criterion_group!(benches, bench_small, bench_medium, bench_large);
criterion_main!(benches);

