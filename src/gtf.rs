use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
struct GtfRecord {
    chr: String,
    strand: String,
    start: String,
    end: String,
    transcript_id: String,
}

impl GtfRecord {
    fn from(line: &str, transcript_re: &Regex) -> GtfRecord {
        let line_split = line.split('\t').collect::<Vec<&str>>();
        // TODO: Handle errors.
        GtfRecord {
            chr: line_split[0].to_owned(),
            strand: line_split[6].to_owned(),
            start: line_split[3].to_owned(),
            end: line_split[4].to_owned(),
            transcript_id: transcript_re
                .captures(line_split[8])
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .to_owned(),
        }
    }
    fn is_exon(line: &str) -> bool {
        // TODO: Is there a chance different transcript can have the same
        // UTRs but different CDSs? I guess not, but if so, we would need to
        // compare CDS information.
        line.contains("\texon\t")
    }
}

fn load_gtf(gtf_path: PathBuf) -> HashMap<(String, String, String), HashSet<String>> {
    // We have already checked GTFs exist/are readable during cli parsing.
    let gtf = File::open(gtf_path).unwrap();

    // Avoid reading the entire file into memory at once.
    let reader = BufReader::new(gtf);
    let transcript_re = Regex::new(r#"transcript_id "([^"]+)"#).unwrap();
    // TODO: better name?
    let mut gtf_transcripts: HashMap<(String, String, String), HashSet<String>> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        if GtfRecord::is_exon(&line) {
            let record = GtfRecord::from(&line, &transcript_re);
            let transcript = gtf_transcripts
                .entry((record.chr, record.strand, record.transcript_id))
                .or_default();

            transcript.insert(record.start);
            transcript.insert(record.end);
        }
    }

    println!("{:?}", gtf_transcripts);
    gtf_transcripts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtfrecord_from() {
        let line = r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#;
        let transcript_re = Regex::new(r#"transcript_id "([^"]+)"#).unwrap();

        assert_eq!(
            GtfRecord::from(line, &transcript_re),
            GtfRecord {
                chr: String::from("chr1"),
                strand: String::from("+"),
                start: String::from("1"),
                end: String::from("2"),
                transcript_id: String::from("A"),
            }
        );
    }

    #[test]
    fn test_gtfrecord_is_exon() {
        // TODO: use parameterization.
        let exon = r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#;
        let cds = r#"chr1	RefSeq	CDS	1	2	.	+	.	transcript_id "A";"#;
        let transcript = r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "A";"#;

        assert!(GtfRecord::is_exon(exon));
        assert!(!GtfRecord::is_exon(cds));
        assert!(!GtfRecord::is_exon(transcript));
    }

    #[test]
    fn test_load_gtf() {
        let mut expected_transcripts: HashMap<(String, String, String), HashSet<String>> =
            HashMap::new();
        expected_transcripts.insert(
            (
                String::from("chr2"),
                String::from("+"),
                String::from("B_transcript.1"),
            ),
            HashSet::from([
                String::from("21"),
                String::from("22"),
                String::from("31"),
                String::from("32"),
            ]),
        );
        expected_transcripts.insert(
            (
                String::from("chr1"),
                String::from("-"),
                String::from("A_transcript.1"),
            ),
            HashSet::from([
                String::from("1"),
                String::from("12"),
                String::from("11"),
                String::from("2"),
            ]),
        );

        assert_eq!(
            load_gtf(PathBuf::from("tests/data/test.gtf")),
            expected_transcripts
        )
    }
}
