use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub type TranscriptId = String;
pub type TranscriptSignature = BTreeSet<String>;

#[derive(Debug, PartialEq)]
struct GtfRecord {
    feature: String,
    strand: String,
    chr: String,
    start: String,
    end: String,
    transcript_id: String,
}

impl GtfRecord {
    fn from(line_split: &[&str], transcript_re: &Regex) -> GtfRecord {
        // TODO: Handle errors.
        // TODO: Use something other than String?
        GtfRecord {
            chr: line_split[0].to_owned(),
            feature: line_split[2].to_owned(),
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
    fn is_exon(line_split: &[&str]) -> bool {
        line_split[2] == "exon"
    }
}

pub fn load_gtf(gtf_path: PathBuf) -> HashMap<TranscriptId, TranscriptSignature> {
    // We have already checked GTFs exist/are readable during cli parsing.
    let gtf = File::open(gtf_path).unwrap();

    // Avoid reading the entire file into memory at once.
    let reader = BufReader::new(gtf);
    let transcript_re = Regex::new(r#"transcript_id "([^"]+)"#).unwrap();
    // TODO: better name?
    let mut gtf_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let line_split = line.split('\t').collect::<Vec<&str>>();

        if GtfRecord::is_exon(&line_split) {
            let record = GtfRecord::from(&line_split, &transcript_re);
            let transcript = gtf_transcripts
                .entry(record.transcript_id)
                .or_insert(BTreeSet::from([record.chr, record.strand]));

            transcript.insert(record.start);
            transcript.insert(record.end);
        }
    }

    // println!("{:?}", gtf_transcripts);
    gtf_transcripts
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_gtfrecord_from() {
        let line = r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#;
        let line_split = line.split('\t').collect::<Vec<&str>>();
        let transcript_re = Regex::new(r#"transcript_id "([^"]+)"#).unwrap();

        assert_eq!(
            GtfRecord::from(&line_split, &transcript_re),
            GtfRecord {
                feature: String::from("exon"),
                strand: String::from("+"),
                chr: String::from("chr1"),
                start: String::from("1"),
                end: String::from("2"),
                transcript_id: String::from("A"),
            }
        );
    }

    #[rstest]
    #[case(r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#, true)]
    #[case(r#"chr1	RefSeq	CDS	1	2	.	+	.	transcript_id "A";"#, false)]
    #[case(r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "A"#, false)]
    fn test_gtfrecord_is_exon(#[case] line: &str, #[case] expected: bool) {
        let line_split = line.split('\t').collect::<Vec<&str>>();

        assert_eq!(GtfRecord::is_exon(&line_split), expected);
    }

    #[test]
    fn test_load_gtf() {
        let mut expected_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

        expected_transcripts.insert(
            String::from("A"),
            BTreeSet::from([
                String::from("chr1"),
                String::from("-"),
                String::from("1"),
                String::from("12"),
                String::from("11"),
                String::from("2"),
            ]),
        );

        expected_transcripts.insert(
            String::from("B"),
            BTreeSet::from([
                String::from("chr2"),
                String::from("+"),
                String::from("21"),
                String::from("22"),
                String::from("31"),
                String::from("32"),
            ]),
        );

        assert_eq!(
            load_gtf(PathBuf::from("tests/data/test_sample_1.gtf")),
            expected_transcripts
        )
    }
}
