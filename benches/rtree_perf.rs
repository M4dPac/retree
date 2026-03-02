use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

fn generate_test_tree(base: &Path, num_files: usize) -> io::Result<()> {
    let mut count = 0usize;
    let dirs_per_level = 5;
    let files_per_dir = 20;

    fn fill(
        path: &Path,
        count: &mut usize,
        target: usize,
        dirs_per_level: usize,
        files_per_dir: usize,
        depth: usize,
    ) -> io::Result<()> {
        if *count >= target || depth > 8 {
            return Ok(());
        }
        for _i in 0..files_per_dir {
            if *count >= target {
                break;
            }
            let file = path.join(format!("file_{:06}.txt", *count));
            fs::write(&file, format!("content of file {}", *count))?;
            *count += 1;
        }
        for d in 0..dirs_per_level {
            if *count >= target {
                break;
            }
            let sub = path.join(format!("dir_{}", d));
            fs::create_dir_all(&sub)?;
            fill(
                &sub,
                count,
                target,
                dirs_per_level,
                files_per_dir,
                depth + 1,
            )?;
        }
        Ok(())
    }

    fill(
        base,
        &mut count,
        num_files,
        dirs_per_level,
        files_per_dir,
        0,
    )
}

fn run_rtree(path: &Path, extra_args: &[&str]) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rtree"));
    cmd.arg(path)
        .arg("--noreport")
        .args(extra_args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    let status = cmd.status().expect("failed to execute rtree");
    assert!(status.success());
}

#[allow(dead_code)]
fn run_rtree_with_output(path: &Path, extra_args: &[&str]) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rtree"));
    cmd.arg(path).arg("--noreport").args(extra_args);

    let output = cmd.output().expect("failed to execute rtree");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).to_string()
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

fn bench_small_100(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 100).unwrap();

    let mut group = c.benchmark_group("small_100_files");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(8));

    // Sequential
    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });

    // Parallel (auto threads)
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel"]));
    });

    // Parallel with specific threads
    group.bench_function("par_2", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "2"]));
    });

    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "4"]));
    });

    // With color
    group.bench_function("seq_color", |b| {
        b.iter(|| run_rtree(temp.path(), &["-C"]));
    });

    group.bench_function("par_color", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "-C"]));
    });

    group.finish();
}

fn bench_medium_10k(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 10_000).unwrap();
    run_rtree(temp.path(), &[]);

    let mut group = c.benchmark_group("medium_10k_files");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel"]));
    });
    group.bench_function("par_2", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "2"]));
    });
    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "4"]));
    });
    group.bench_function("par_8", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "8"]));
    });
    group.finish();
}

fn bench_large_100k(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 100_000).unwrap();
    run_rtree(temp.path(), &[]);

    let mut group = c.benchmark_group("large_100k_files");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    // Sequential
    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });

    // Parallel
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel"]));
    });

    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "4"]));
    });

    group.bench_function("par_8", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "8"]));
    });

    group.finish();
}

fn bench_xlarge_1m(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 1_000_000).unwrap();
    run_rtree(temp.path(), &[]);

    let mut group = c.benchmark_group("xlarge_1m_files");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    // Sequential - main target
    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });

    // Parallel - main target
    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel"]));
    });

    group.bench_function("par_4", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "4"]));
    });

    group.bench_function("par_8", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "--threads", "8"]));
    });

    group.finish();
}

// ============================================================================
// OUTPUT FORMAT BENCHMARKS
// ============================================================================

fn bench_output_formats(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 10_000).unwrap();
    run_rtree(temp.path(), &[]);

    let mut group = c.benchmark_group("output_formats_10k");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(10));

    // Text
    group.bench_function("text_seq", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });

    group.bench_function("text_par", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel"]));
    });

    // JSON
    group.bench_function("json_seq", |b| {
        b.iter(|| run_rtree(temp.path(), &["-J"]));
    });

    group.bench_function("json_par", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "-J"]));
    });

    // XML
    group.bench_function("xml_seq", |b| {
        b.iter(|| run_rtree(temp.path(), &["-X"]));
    });

    group.bench_function("xml_par", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "-X"]));
    });

    group.finish();
}

// ============================================================================
// OPTIONS BENCHMARKS
// ============================================================================

fn bench_options(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 10_000).unwrap();
    run_rtree(temp.path(), &[]);

    let mut group = c.benchmark_group("options_10k");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(10));

    // Basic
    group.bench_function("plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });

    // With sorting
    group.bench_function("sort_size", |b| {
        b.iter(|| run_rtree(temp.path(), &["--sort", "size"]));
    });

    group.bench_function("sort_time", |b| {
        b.iter(|| run_rtree(temp.path(), &["--sort", "mtime"]));
    });

    // With filtering
    group.bench_function("dirs_only", |b| {
        b.iter(|| run_rtree(temp.path(), &["-d"]));
    });

    // With depth limit
    group.bench_function("depth_2", |b| {
        b.iter(|| run_rtree(temp.path(), &["-L", "2"]));
    });

    group.bench_function("depth_3", |b| {
        b.iter(|| run_rtree(temp.path(), &["-L", "3"]));
    });

    // With colors/icons
    group.bench_function("color", |b| {
        b.iter(|| run_rtree(temp.path(), &["-C"]));
    });

    group.bench_function("icons", |b| {
        b.iter(|| run_rtree(temp.path(), &["--icons", "always"]));
    });

    group.finish();
}

// ============================================================================
// REGISTER ALL BENCHMARKS
// ============================================================================

criterion_group!(
    benches,
    bench_small_100,
    bench_medium_10k,
    bench_large_100k,
    bench_xlarge_1m,
    bench_output_formats,
    bench_options
);
criterion_main!(benches);
