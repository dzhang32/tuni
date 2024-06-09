# tuni benchmark tests

## Download input data

Benchmark tests are run on the CHESS data (~292Mb) and the Ensembl GTF (~1.4Gb). These data sets can be downloaded using the below:

```bash
curl -L -o tests/data/benches/chess3.0.1.primary.gtf.gz https://github.com/chess-genome/chess/releases/download/v.3.0.1/chess3.0.1.primary.gtf.gz
curl -L -o tests/data/benches/Homo_sapiens.GRCh38.112.chr.gtf.gz https://ftp.ensembl.org/pub/release-112/gtf/homo_sapiens/Homo_sapiens.GRCh38.112.chr.gtf.gz

gunzip tests/data/benches/chess3.0.1.primary.gtf.gz
gunzip tests/data/benches/Homo_sapiens.GRCh38.112.chr.gtf.gz
```

## Executing a benchmark

This will run the two samples downloaded above through `tuni` 10 times using `criterion`.

```bash
cargo bench
```

## Manual execution

It can be useful to execute a single run of `tuni` using the benchmark samples (e.g. whilst using `-v` to get a gauge of which step(s) take the longest to execute).

```bash
cargo run --release -- -g tests/data/benches/gtf_paths.txt -o /tmp -v
```
