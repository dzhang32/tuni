# tuni

![crates](https://img.shields.io/crates/v/tuni)
[![ci](https://github.com/dzhang32/tuni/workflows/lint-test-package/badge.svg)](https://github.com/dzhang32/tuni/actions)

The goal of `tuni` is to unify transcripts across different samples.

## Overview

Transcript assembly tools can generate arbitary transcript IDs, which may lead to the same transcript being labelled with a different ID across samples.

For example, given two samples `sample_1.gtf` and `sample_2.gtf`:

**sample_1.gtf**

```text
chr1 test transcript 1 100 . + . transcript_id "A"; 
chr1 test exon 1 40 . + . transcript_id "A"; 
chr1 test exon 50 100 . + . transcript_id "A";
--snip-- 
```

**sample_2.gtf**

```text
chr1 test transcript 1 100 . + . transcript_id "B"; 
chr1 test exon 1 40 . + . transcript_id "B"; 
chr1 test exon 50 100 . + . transcript_id "B";
--snip-- 
```

The transcript displayed above is identical between the two samples, however the provided `transcript_id` is different for each sample, "A" vs "B".

`tuni` generates a `.tuni.gtf`/`.tuni.gff` for each input `.gtf`/`.gff`. These output files will contain an additional attribute field `tuni_id` which contains a unified ID that will be same for identical transcripts across different samples.

**sample_1.tuni.gtf**

```text
chr1 test transcript 1 100 . + . transcript_id "A"; tuni_id "tuni_0";
chr1 test exon 1 40 . + . transcript_id "A"; tuni_id "tuni_0";
chr1 test exon 50 100 . + . transcript_id "A"; tuni_id "tuni_0";
--snip-- 
```

**sample_2.tuni.gtf**

```text
chr1 test transcript 1 100 . + . transcript_id "B"; tuni_id "tuni_0";
chr1 test exon 1 40 . + . transcript_id "B"; tuni_id "tuni_0";
chr1 test exon 50 100 . + . transcript_id "B"; tuni_id "tuni_0";
--snip-- 
```

## Installation

Install [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) then run:

```bash
cargo install tuni
```

## Usage

```bash
Usage: tuni [OPTIONS] --gtf-gff-path <*.txt> --output-dir </output/dir/>

Options:
  -g, --gtf-gff-path <*.txt>       A text file containing GTF/GFF paths
  -o, --output-dir </output/dir/>  Directory where outputted GTF/GFFs will be stored
  -v, --verbose                    Print log messages
  -h, --help                       Print help
  -V, --version                    Print version
```

*Note: currently, only [version 2](https://www.ensembl.org/info/website/upload/gff.html) `.gff` files are accepted by `tuni`.*
