use crate::unify::TranscriptUnifier;

use regex::{Captures, Regex};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub type TranscriptId = String;

// exon_boundaries and cds_boundaries must be BTreesSets as
// 1. we want to use TranscriptSignature as a key later on and HashSet is not hashable
// 2. we want values to be unique.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TranscriptSignature {
    chr: String,
    strand: String,
    exon_boundaries: BTreeSet<String>,
    cds_boundaries: BTreeSet<String>,
}

impl TranscriptSignature {
    pub fn from(
        chr: String,
        strand: String,
        exon_boundaries: BTreeSet<String>,
        cds_boundaries: BTreeSet<String>,
    ) -> TranscriptSignature {
        TranscriptSignature {
            chr,
            strand,
            exon_boundaries,
            cds_boundaries,
        }
    }

    fn insert_boundary(&mut self, feature: &str, value: String) {
        match feature {
            "exon" => self.exon_boundaries.insert(value),
            "CDS" => self.cds_boundaries.insert(value),
            // TODO: Handle errors.
            _ => panic!(
                "Feature must be one of 'exon' or 'CDS', instead found {}",
                feature
            ),
        };
    }
}

const TRANSCRIPT_ID_RE: &str = r#"transcript_id "([^"]+)"#;

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
        // TODO: Use something other than String?
        GtfRecord {
            chr: line_split[0].to_owned(),
            feature: line_split[2].to_owned(),
            strand: line_split[6].to_owned(),
            start: line_split[3].to_owned(),
            end: line_split[4].to_owned(),
            transcript_id: GtfRecord::get_transcript_id(line_split, transcript_re)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .to_owned(),
        }
    }

    fn is_exon_or_cds(line_split: &[&str]) -> bool {
        line_split[2] == "exon" || line_split[2] == "CDS"
    }

    fn get_transcript_id<'a>(
        line_split: &[&'a str],
        transcript_re: &Regex,
    ) -> Option<Captures<'a>> {
        // TODO: Handle errors.
        transcript_re.captures(line_split[8])
    }
}

pub fn read_gtf(gtf_path: &Path) -> HashMap<TranscriptId, TranscriptSignature> {
    // We have already checked GTFs exist/are readable during cli parsing.
    let gtf = File::open(gtf_path).unwrap();

    // Avoid reading the entire file into memory at once.
    let reader = BufReader::new(gtf);
    let transcript_re = Regex::new(TRANSCRIPT_ID_RE).unwrap();
    // TODO: better name?
    let mut gtf_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let line_split = line.split('\t').collect::<Vec<&str>>();

        if GtfRecord::is_exon_or_cds(&line_split) {
            let record = GtfRecord::from(&line_split, &transcript_re);

            // TODO: Make TranscriptSignature a struct.
            let transcript_signature =
                gtf_transcripts
                    .entry(record.transcript_id)
                    .or_insert(TranscriptSignature::from(
                        record.chr,
                        record.strand,
                        BTreeSet::new(),
                        BTreeSet::new(),
                    ));

            transcript_signature.insert_boundary(&record.feature, record.start);
            transcript_signature.insert_boundary(&record.feature, record.end);
        }
    }

    // println!("{:?}", gtf_transcripts);
    gtf_transcripts
}

pub fn write_unified_gtf(
    gtf_path: &Path,
    output_dir: &Path,
    transcript_unifier: &TranscriptUnifier,
) {
    // TODO: switch to different String type or stick with OsString?
    let gtf_file_name = gtf_path.file_name().unwrap().to_str().unwrap();
    let mut output_path = output_dir.to_path_buf();
    output_path.push(gtf_file_name);
    output_path.set_extension("tuni.gtf");

    // TODO: Handle errors or check CLI when parsing.
    let output_unified_gtf = File::create(output_path).expect("Unable to create file");
    let mut writer = BufWriter::new(output_unified_gtf);

    let gtf = File::open(gtf_path).unwrap();
    let reader = BufReader::new(gtf);

    let transcript_re = Regex::new(TRANSCRIPT_ID_RE).unwrap();

    for line in reader.lines() {
        let mut line: String = line.unwrap();
        let line_split = line.split('\t').collect::<Vec<&str>>();

        let transcript_id = GtfRecord::get_transcript_id(&line_split, &transcript_re);

        if let Some(captures) = transcript_id {
            // TODO: handle errors.
            let unified_id = transcript_unifier.get_unified_id(&(
                gtf_file_name.to_owned(),
                captures.get(1).unwrap().as_str().to_owned(),
            ));
            line.push_str(&format!(r#" tuni_id "{}";"#, unified_id));
        }

        // TODO: Handle errors or check CLI when parsing.
        writeln!(writer, "{}", line).expect("Unable to write to file.");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use rstest::rstest;
    use std::fs::read_to_string;
    use tempfile::tempdir;

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
    #[case(r#"chr1	RefSeq	CDS	1	2	.	+	.	transcript_id "A";"#, true)]
    #[case(r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "A"#, false)]
    fn test_gtfrecord_is_exon(#[case] line: &str, #[case] expected: bool) {
        let line_split = line.split('\t').collect::<Vec<&str>>();

        assert_eq!(GtfRecord::is_exon_or_cds(&line_split), expected);
    }

    #[test]
    fn test_read_gtf() {
        let mut expected_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

        expected_transcripts.insert(
            String::from("A"),
            TranscriptSignature::from(
                "chr1".to_string(),
                "-".to_string(),
                BTreeSet::from([
                    String::from("1"),
                    String::from("12"),
                    String::from("11"),
                    String::from("2"),
                ]),
                BTreeSet::new(),
            ),
        );

        expected_transcripts.insert(
            String::from("B"),
            TranscriptSignature::from(
                "chr2".to_string(),
                "+".to_string(),
                BTreeSet::from([String::from("20"), String::from("30")]),
                BTreeSet::from([String::from("25"), String::from("29")]),
            ),
        );

        assert_eq!(
            read_gtf(&PathBuf::from("tests/data/unit/sample_1.gtf")),
            expected_transcripts
        )
    }

    #[test]
    fn test_write_unified_gtf() {
        let mut transcript_unifier = TranscriptUnifier::new();
        let gtf_path = PathBuf::from("tests/data/unit/sample_1.gtf");
        let mut gtf_transcripts = read_gtf(&gtf_path);
        transcript_unifier.add_transcripts("sample_1.gtf".to_string(), &mut gtf_transcripts);
        transcript_unifier.unify_transcripts();

        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("sample_1.tuni.gtf");

        write_unified_gtf(&gtf_path, temp_dir.path(), &transcript_unifier);
        // .collect() as <Vec<&str>> for easier debugging.
        assert_eq!(
            read_to_string(output_path)
                .unwrap()
                .lines()
                .collect::<Vec<&str>>(),
            read_to_string(PathBuf::from("tests/data/unit/expected_unified_gtf.gtf"))
                .unwrap()
                .lines()
                .collect::<Vec<&str>>()
        );
    }
}
