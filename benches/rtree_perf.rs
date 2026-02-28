use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn generate_test_tree(base: &Path, num_files: usize) -> io::Result<()> {
    // Create subdirectories for more realistic tree structure
    let dirs = ["src", "lib", "tests", "docs", "examples"];
    for dir in &dirs {
        let dir_path = base.join(dir);
        fs::create_dir_all(&dir_path)?;
        
        // Distribute files across directories
        let files_per_dir = num_files / dirs.len();
        for i in 0..files_per_dir {
            let file = dir_path.join(format!("file_{:06}.txt", i));
            fs::write(file, b"x")?;
        }
    }
    
    // Add remaining files to root
    let remaining = num_files - (num_files / dirs.len() * dirs.len());
    for i in 0..remaining {
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

fn run_rtree_with_output(path: &Path, extra_args: &[&str]) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rtree"));
    cmd.arg(path)
        .arg("--noreport")
        .args(extra_args);
    
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

    let mut group = c.benchmark_group("medium_10k_files");
    group.sample_size(20);
    
    // Sequential
    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });
    
    // Parallel
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
    
    // JSON output
    group.bench_function("seq_json", |b| {
        b.iter(|| run_rtree(temp.path(), &["-J"]));
    });
    
    group.bench_function("par_json", |b| {
        b.iter(|| run_rtree(temp.path(), &["--parallel", "-J"]));
    });
    
    group.finish();
}

fn bench_large_100k(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    generate_test_tree(temp.path(), 100_000).unwrap();

    let mut group = c.benchmark_group("large_100k_files");
    group.sample_size(10);
    
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

    let mut group = c.benchmark_group("xlarge_1m_files");
    group.sample_size(3);
    
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

    let mut group = c.benchmark_group("output_formats_10k");
    group.sample_size(15);
    
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

    let mut group = c.benchmark_group("options_10k");
    group.sample_size(15);
    
    // Basic
    group.bench_function("plain", |b| {
        b.iter(|| run_rtree(temp.path(), &[]));
    });
    
    // With sorting
    group.bench_function("sort_size", |b| {
        b.iter(|| run_rtree(temp.path(), &["-S", "size"]));
    });
    
    group.bench_function("sort_time", |b| {
        b.iter(|| run_rtree(temp.path(), &["-S", "mtime"]));
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