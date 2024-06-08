# tuni

The goal of `tuni` is to provide unified IDs for identical transcripts called across different samples using transcript assembly tools.

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
