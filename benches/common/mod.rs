#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

pub struct SharedTree {
    pub path: PathBuf,
}

fn bench_tree_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("bench_trees")
}

pub fn persistent_tree(name: &str, num_files: usize) -> SharedTree {
    let base = bench_tree_root().join(name);
    let marker = base.join(".tree_ready");

    if !marker.exists() {
        eprintln!(
            "[bench] Creating {} files in {} ...",
            num_files,
            base.display()
        );
        if base.exists() {
            fs::remove_dir_all(&base).ok();
        }
        fs::create_dir_all(&base).unwrap();
        generate_test_tree(&base, num_files).unwrap();
        fs::write(&marker, format!("{}", num_files)).unwrap();
        // Warm FS cache
        run_rtree(&base, &[]);
        eprintln!("[bench] Tree ready.");
    }

    SharedTree { path: base }
}

// ── Parallel tree generation ───────────────────────────────────────

pub fn generate_test_tree(base: &Path, num_files: usize) -> io::Result<()> {
    let counter = AtomicUsize::new(0);
    fill_recursive(base, num_files, &counter, 0);
    Ok(())
}

fn fill_recursive(path: &Path, target: usize, counter: &AtomicUsize, depth: usize) {
    if counter.load(Ordering::Relaxed) >= target || depth > 8 {
        return;
    }

    // Files in current directory
    for _ in 0..20 {
        let n = counter.fetch_add(1, Ordering::Relaxed);
        if n >= target {
            return;
        }
        let _ = fs::write(path.join(format!("file_{:06}.txt", n)), b"x");
    }

    // Subdirectories
    let subdirs: Vec<PathBuf> = (0..5)
        .filter(|_| counter.load(Ordering::Relaxed) < target)
        .map(|d| {
            let sub = path.join(format!("dir_{}", d));
            let _ = fs::create_dir_all(&sub);
            sub
        })
        .collect();

    // Parallel at top levels (≤125 tasks), sequential deeper
    if depth < 3 {
        use rayon::prelude::*;
        subdirs.par_iter().for_each(|dir| {
            fill_recursive(dir, target, counter, depth + 1);
        });
    } else {
        for dir in &subdirs {
            fill_recursive(dir, target, counter, depth + 1);
        }
    }
}

// ── Runner ─────────────────────────────────────────────────────────

pub fn run_rtree(path: &Path, extra_args: &[&str]) {
    let status = Command::new(env!("CARGO_BIN_EXE_rtree"))
        .arg(path)
        .arg("--noreport")
        .args(extra_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to execute rtree");
    assert!(status.success());
}

// ── OnceLock shared trees ──────────────────────────────────────────
// Each tree is created at most once per process, persists on disk
// between runs. To recreate: delete target/bench_trees/

pub fn tree_100() -> &'static SharedTree {
    static T: OnceLock<SharedTree> = OnceLock::new();
    T.get_or_init(|| persistent_tree("small_100", 100))
}

pub fn tree_10k() -> &'static SharedTree {
    static T: OnceLock<SharedTree> = OnceLock::new();
    T.get_or_init(|| persistent_tree("medium_10k", 10_000))
}

pub fn tree_100k() -> &'static SharedTree {
    static T: OnceLock<SharedTree> = OnceLock::new();
    T.get_or_init(|| persistent_tree("large_100k", 100_000))
}

pub fn tree_1m() -> &'static SharedTree {
    static T: OnceLock<SharedTree> = OnceLock::new();
    T.get_or_init(|| persistent_tree("xlarge_1m", 1_000_000))
}
