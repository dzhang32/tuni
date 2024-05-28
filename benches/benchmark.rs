use assert_cmd::Command;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

/// Benchmark tuni performance across large samples.
pub fn benchmark_tuni(c: &mut Criterion) {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let temp_dir = tempdir().unwrap();

    cmd.arg(black_box("--gtf-gff-path"))
        .arg(black_box("tests/data/benches/gtf_paths.txt"))
        .arg(black_box("--output-dir"))
        .arg(temp_dir.path());

    c.bench_function("tuni benchmark", |b| b.iter(|| cmd.assert().success()));
}

// criterion's statistics is not intended for longer benchmarks:
// https://github.com/bheisler/criterion.rs/issues/322
// Here, we time how long it takes to run tuni on large samples (10 iterations),
// ignoring the statistical comparison in the output.
criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.05).sample_size(10);
    targets = benchmark_tuni
}
criterion_main!(benches);
