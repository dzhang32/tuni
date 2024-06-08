# tuni

The goal of `tuni` is to generate unified IDs for identical transcripts called across different samples.

## Background

Transcript assembly tools can generate arbitary transcript IDs that differ between identical transcripts across samples.

For instance, given two samples, `sample_1.gtf` and `sample_2.gtf`:

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

Given a list of `.gtf`/`.gff` files, `tuni` outputs a `tuni_id` that is unified for identical transcripts across different samples.

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

TODO: upload `tuni` to crates.io.

## Usage

`tuni` expects as input:

1. A `.txt` file that contains the paths to each input `.gtf` or `.gff` detailing transcripts to be unified. Currently, only [version 2](https://www.ensembl.org/info/website/upload/gff.html) `.gff` files are accepted.
2. A path to the output directory.

Executing `tuni`:

```bash
tuni -gtf-gff-path /path/to/gtf_paths.txt -output-dir /path/to/output/directory/
```

In the output directory, `tuni` will create a `.tuni.gtf`/`.tuni.gff` for each input `.gtf`/`.gff`. These `.tuni.*` output files will contain an additional attribute field `tuni_id` which contains unified ID that will be same for identical transcripts across different samples.
