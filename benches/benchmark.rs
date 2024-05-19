use assert_cmd::Command;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let temp_dir = tempdir().unwrap();

    cmd.arg(black_box("--gtf-paths"))
        .arg(black_box("tests/data/benches/gtf_paths.txt"))
        .arg(black_box("--output-dir"))
        .arg(temp_dir.path());

    c.bench_function("tuni benchmark", |b| b.iter(|| cmd.assert().success()));
}

criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.05).sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
