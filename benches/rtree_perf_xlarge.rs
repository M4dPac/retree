use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

#[path = "common/mod.rs"]
mod common;

use common::{run_rtree, tree_1m};

fn bench_xlarge_1m(c: &mut Criterion) {
    let t = tree_1m();

    let mut group = c.benchmark_group("xlarge_1m_files");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(700));
    group.warm_up_time(Duration::from_secs(10));

    group.bench_function("seq_plain", |b| {
        b.iter(|| run_rtree(&t.path, &[]));
    });

    group.bench_function("par_auto", |b| {
        b.iter(|| run_rtree(&t.path, &["--parallel"]));
    });

    group.finish();
}

criterion_group!(benches, bench_xlarge_1m);
criterion_main!(benches);
