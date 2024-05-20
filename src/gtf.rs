use crate::unify::TranscriptUnifier;

use regex::{Match, Regex};
use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    rc::Rc,
};

const TRANSCRIPT_ID_RE: &str = r#"transcript_id "([^"]+)"#;
pub type TranscriptId = Rc<str>;

// exon_boundaries and cds_boundaries must be BTreesSets as
// 1. we want to use TranscriptSignature as a key later on and HashSet is not hashable
// 2. we want values to be unique.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TranscriptSignature {
    chr: Rc<str>,
    strand: Rc<str>,
    exon_boundaries: BTreeSet<Rc<str>>,
    cds_boundaries: BTreeSet<Rc<str>>,
}

impl TranscriptSignature {
    pub fn from(
        chr: Rc<str>,
        strand: Rc<str>,
        exon_boundaries: BTreeSet<Rc<str>>,
        cds_boundaries: BTreeSet<Rc<str>>,
    ) -> TranscriptSignature {
        TranscriptSignature {
            chr,
            strand,
            exon_boundaries,
            cds_boundaries,
        }
    }

    fn insert_boundary(&mut self, feature: &str, value: Rc<str>) {
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

#[derive(Debug, PartialEq)]
struct GtfRecord {
    feature: Rc<str>,
    strand: Rc<str>,
    chr: Rc<str>,
    start: Rc<str>,
    end: Rc<str>,
    transcript_id: Rc<str>,
}

impl GtfRecord {
    fn from(line_split: &[&str], transcript_re: &Regex) -> GtfRecord {
        // TODO: Use something other than String?
        GtfRecord {
            chr: Rc::from(line_split[0]),
            feature: Rc::from(line_split[2]),
            strand: Rc::from(line_split[6]),
            start: Rc::from(line_split[3]),
            end: Rc::from(line_split[4]),
            transcript_id: Rc::from(
                GtfRecord::get_transcript_id(line_split, transcript_re)
                    .unwrap()
                    .as_str(),
            ),
        }
    }

    fn is_exon_or_cds(line_split: &[&str]) -> bool {
        line_split[2] == "exon" || line_split[2] == "CDS"
    }

    fn get_transcript_id<'a>(line_split: &[&'a str], transcript_re: &Regex) -> Option<Match<'a>> {
        // TODO: Handle errors.
        transcript_re.find(line_split[8])
    }
}

pub fn read_gtf(gtf_path: &Path) -> HashMap<TranscriptId, TranscriptSignature> {
    // We have already checked GTFs exist/are readable during cli parsing.
    let gtf = File::open(gtf_path).unwrap();

    // Avoid reading the entire file into memory at once.
    let reader = BufReader::new(gtf);
    // TODO: make this a static const.
    let transcript_re = Regex::new(TRANSCRIPT_ID_RE).unwrap();
    // TODO: better name?
    let mut gtf_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();

        if !line.starts_with('#') {
            let line_split = line.split('\t').collect::<Vec<&str>>();

            if GtfRecord::is_exon_or_cds(&line_split) {
                let record = GtfRecord::from(&line_split, &transcript_re);

                let transcript_signature = gtf_transcripts.entry(record.transcript_id).or_insert(
                    TranscriptSignature::from(
                        record.chr,
                        record.strand,
                        BTreeSet::new(),
                        BTreeSet::new(),
                    ),
                );

                transcript_signature.insert_boundary(&record.feature, record.start);
                transcript_signature.insert_boundary(&record.feature, record.end);
            }
        }
    }

    gtf_transcripts
}

pub fn write_unified_gtf(
    gtf_path: &Path,
    output_dir: &Path,
    transcript_unifier: &TranscriptUnifier,
) {
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
        let mut line = line.unwrap();

        if !line.starts_with('#') {
            let line_split = line.split('\t').collect::<Vec<&str>>();

            let transcript_id = GtfRecord::get_transcript_id(&line_split, &transcript_re);

            if let Some(captures) = transcript_id {
                // TODO: handle errors.
                let unified_id = transcript_unifier
                    .get_unified_id(&(Rc::from(gtf_file_name), Rc::from(captures.as_str())));
                line.push_str(&format!(r#" tuni_id "{}";"#, unified_id));
            }
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
                feature: Rc::from("exon"),
                strand: Rc::from("+"),
                chr: Rc::from("chr1"),
                start: Rc::from("1"),
                end: Rc::from("2"),
                transcript_id: Rc::from("transcript_id \"A"),
            }
        );
    }

    #[rstest]
    #[case(r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#, true)]
    #[case(r#"chr1	RefSeq	CDS	1	2	.	+	.	transcript_id "A";"#, true)]
    #[case(r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "A"#, false)]
    fn test_is_exon_or_cds(#[case] line: &str, #[case] expected: bool) {
        let line_split = line.split('\t').collect::<Vec<&str>>();

        assert_eq!(GtfRecord::is_exon_or_cds(&line_split), expected);
    }

    #[rstest]
    #[case(r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#, Some(""))]
    #[case(r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "A"#, Some(""))]
    #[case(r#"chr1	RefSeq	gene	1	2	.	+	.	gene_id "A"#, None)]
    fn test_get_transcript_id(#[case] line: &str, #[case] expected: Option<&str>) {
        let transcript_re = Regex::new(TRANSCRIPT_ID_RE).unwrap();
        let line_split = line.split('\t').collect::<Vec<&str>>();

        match expected {
            Some(_) => {
                let capture = GtfRecord::get_transcript_id(&line_split, &transcript_re);
                assert!(capture.is_some());
                assert_eq!(capture.unwrap().as_str(), "transcript_id \"A");
            }
            None => assert!(GtfRecord::get_transcript_id(&line_split, &transcript_re).is_none()),
        }
    }

    #[test]
    fn test_read_gtf() {
        let mut expected_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

        expected_transcripts.insert(
            Rc::from("transcript_id \"A"),
            TranscriptSignature::from(
                Rc::from("chr1"),
                Rc::from("-"),
                BTreeSet::from([Rc::from("1"), Rc::from("12"), Rc::from("11"), Rc::from("2")]),
                BTreeSet::new(),
            ),
        );

        expected_transcripts.insert(
            Rc::from("transcript_id \"B"),
            TranscriptSignature::from(
                Rc::from("chr2"),
                Rc::from("+"),
                BTreeSet::from([Rc::from("20"), Rc::from("30")]),
                BTreeSet::from([Rc::from("25"), Rc::from("29")]),
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
        transcript_unifier.add_transcripts(Rc::from("sample_1.gtf"), &mut gtf_transcripts);
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
            read_to_string(PathBuf::from("tests/data/unit/expected_sample_1.tuni.gtf"))
                .unwrap()
                .lines()
                .collect::<Vec<&str>>()
        );
    }
}
