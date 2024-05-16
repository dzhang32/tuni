use crate::unify::TranscriptUnifier;

use regex::{Match, Regex};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub type TranscriptId = String;
pub type TranscriptSignature = BTreeSet<String>;

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
                .as_str()
                .to_owned(),
        }
    }

    fn is_exon(line_split: &[&str]) -> bool {
        line_split[2] == "exon"
    }

    fn get_transcript_id<'a>(line_split: &[&'a str], transcript_re: &Regex) -> Option<Match<'a>> {
        // TODO: Handle errors.
        transcript_re.captures(line_split[8]).unwrap().get(1)
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

fn write_unified_gtf(gtf_path: &Path, output_dir: &Path, transcript_unifier: TranscriptUnifier) {
    // TODO: switch to different String type or stick with OsString?
    let gtf_file_name = gtf_path.file_name().unwrap().to_str().unwrap();
    let mut output_path = output_dir.to_path_buf();
    output_path.push(gtf_file_name);
    output_path.set_extension("tuni.gtf");

    // TODO: Handle errors or check CLI when parsing.
    let mut output_unified_gtf = File::create(output_path).expect("Unable to create file");
    let mut writer = BufWriter::new(output_unified_gtf);

    let gtf = File::open(gtf_path).unwrap();
    let reader = BufReader::new(gtf);

    let transcript_re = Regex::new(TRANSCRIPT_ID_RE).unwrap();

    for line in reader.lines() {
        let mut line: String = line.unwrap();
        let line_split = line.split('\t').collect::<Vec<&str>>();

        let transcript_id = GtfRecord::get_transcript_id(&line_split, &transcript_re);

        if let Some(m) = transcript_id {
            let unified_id = transcript_unifier
                .get_unified_id(&(gtf_file_name.to_owned(), m.as_str().to_owned()));
            line.push_str(&format!(r#"tuni_id "{}";"#, unified_id));
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
            read_gtf(&PathBuf::from("tests/data/test_sample_1.gtf")),
            expected_transcripts
        )
    }

    #[test]
    fn test_write_unified_gtf() {
        let mut transcript_unifier = TranscriptUnifier::new();
        let gtf_path = PathBuf::from("tests/data/test_sample_1.gtf");
        let mut gtf_transcripts = read_gtf(&gtf_path);
        transcript_unifier.add_transcripts("test_sample_1.gtf".to_string(), &mut gtf_transcripts);
        transcript_unifier.unify_transcripts();

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_sample_1.tuni.gtf");

        write_unified_gtf(&gtf_path, &PathBuf::from("tests/data/"), transcript_unifier);
    }
}
