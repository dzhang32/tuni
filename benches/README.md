# Tuni benchmarks

## Setup

```bash
curl -L -o tests/data/benches/chess3.0.1.primary.gtf.gz https://github.com/chess-genome/chess/releases/download/v.3.0.1/chess3.0.1.primary.gtf.gz
curl -L -o tests/data/benches/Homo_sapiens.GRCh38.111.chr.gtf.gz https://ftp.ensembl.org/pub/release-112/gtf/homo_sapiens/Homo_sapiens.GRCh38.111.chr.gtf.gz

gunzip tests/data/benches/chess3.0.1.primary.gtf.gz
gunzip tests/data/benches/Homo_sapiens.GRCh38.111.chr.gtf.gz
```