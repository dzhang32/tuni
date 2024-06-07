# tuni

The goal of `tuni` is to provide a unified ID for transcripts called by transcript assembly tools across different samples.

## Installation

TODO: upload `tuni` to crates.io.

## Usage

`tuni` expects as input a `.txt` file that contains the paths to each input `.gtf` or `.gff` and a path to the output directory. Currently, only `.gff` version 2 are accepted.

TODO: link out to documentation.

```
tuni -g tests/data/benches/gtf_paths.txt -o /tmp -v
```
