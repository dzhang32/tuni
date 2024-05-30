use crate::error::GtfGffError;
use crate::unify::TranscriptUnifier;
use log::{info, warn};

use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    rc::Rc,
};

/// Transcript ID in the format "transcript_id \"A.1\"".
pub type TranscriptId = Rc<str>;

/// Contains all details needed to identify a unique transcript.
///
/// If any fields are different between two `TranscriptSignature`s, they
/// must represent distinct transcripts. Both exons AND CDS regions must be
/// included to differentiate between transcripts that have:
/// 1. The same coding regions and different UTR.
/// 2. The same UTRs and different coding regions.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TranscriptSignature {
    /// Chromosome.
    chr: Rc<str>,

    /// Strand.
    strand: Rc<str>,

    /// The start and end coordinates of every exon in the transcript.
    ///
    /// Must be `BTreesSet`s as:
    /// 1. `TranscriptSignature` will be used a `HashMap`` key. `HashSet`s are not
    /// hashable as they do not have an order.
    /// 2. A `Vec<Rc<str>>` cannot be used as regions are not assumed to be
    /// sorted in the input GTF/GFF.
    exon_boundaries: BTreeSet<Rc<str>>,

    /// The start and end coordinates of every CDS region in the transcript.
    ///
    /// Must be a `BTreeSet` for the same reasons as above.
    cds_boundaries: BTreeSet<Rc<str>>,
}

impl TranscriptSignature {
    /// Create `TranscriptSignature`.
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

    /// Insert exon/CDS boundary into `TranscriptSignature`.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownFeatureError`](GtfGffError::UnknownFeatureError) if the
    /// feature is not "exon" or "CDS". This error likely indicates a bug in
    /// tuni when filtering GTF/GFF lines.
    fn insert_boundary(&mut self, feature: &str, value: Rc<str>) -> Result<(), GtfGffError> {
        match feature {
            "exon" => {
                self.exon_boundaries.insert(value);
            }
            "CDS" => {
                self.cds_boundaries.insert(value);
            }
            other => return Err(GtfGffError::UnknownFeatureError(other.to_string())),
        };
        Ok(())
    }
}

/// Parse lines within a GTF/GFF file.
///
/// `GtfGffRecord` requires a `transcript_id`. In `tuni`, this is satisfied as
/// `GtfGffRecord` are only created from "exon"/"CDS" lines, which should always
/// contain a `transcript_id`.
#[derive(Debug, PartialEq)]
struct GtfGffRecord {
    /// Feature e.g. "exon", "transcript", "CDS".
    feature: Rc<str>,

    /// Strand.
    strand: Rc<str>,

    /// Chromosome.
    chr: Rc<str>,

    /// Start coordinate.
    start: Rc<str>,

    /// End coordinate.
    end: Rc<str>,

    /// Transcript ID.
    transcript_id: Rc<str>,
}

impl GtfGffRecord {
    /// Create a `GtfGffRecord` from a line.
    ///
    /// # Errors
    ///
    /// Returns [`MissingTranscriptIdError`](GtfGffError::MissingTranscriptIdError)
    /// if the line does not contain a "transcript_id" attribute.
    fn from(line_split: &[&str]) -> Result<GtfGffRecord, GtfGffError> {
        let transcript_id = GtfGffRecord::get_transcript_id(line_split)
            .ok_or(GtfGffError::MissingTranscriptIdError(line_split.join("\t")))?;

        Ok(GtfGffRecord {
            chr: Rc::from(line_split[0]),
            feature: Rc::from(line_split[2]),
            strand: Rc::from(line_split[6]),
            start: Rc::from(line_split[3]),
            end: Rc::from(line_split[4]),
            transcript_id: Rc::from(transcript_id),
        })
    }

    /// Returns true if line represents a exon or CDS, otherwise false.
    fn is_exon_or_cds(line_split: &[&str]) -> bool {
        line_split[2] == "exon" || line_split[2] == "CDS"
    }

    /// Obtain the transcript ID.
    ///
    /// This relies on transcript ID attributes being named exactly
    /// "transcript_id".
    fn get_transcript_id<'a>(line_split: &[&'a str]) -> Option<&'a str> {
        line_split[8]
            .split(';')
            .find(|x| x.trim().starts_with("transcript_id"))
    }
}

/// Format outputted unified ID.
enum TuniIdFormatter {
    Gtf,
    Gff,
}

impl TuniIdFormatter {
    /// Create output formatter depending on input file type.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownExtensionError`](GtfGffError::UnknownExtensionError) if
    /// the provided extension is not "gtf"/"gff".
    fn from(gtf_gff_extension: &str) -> Result<TuniIdFormatter, GtfGffError> {
        match gtf_gff_extension {
            "gtf" => Ok(TuniIdFormatter::Gtf),
            "gff" => Ok(TuniIdFormatter::Gff),
            other => Err(GtfGffError::UnknownExtensionError(other.to_string())),
        }
    }

    /// Format unified ID depending on input file type.
    fn format(&self, unified_id: &str) -> String {
        match self {
            TuniIdFormatter::Gtf => format!(r#" tuni_id "{}";"#, unified_id),
            TuniIdFormatter::Gff => format!(" tuni_id={};", unified_id),
        }
    }
}

/// Read unique transcripts from a GTF/GFF file.
///
/// Using the "transcript_id" as a differentiating key, build a
/// `TranscriptSignature` for every unique transcript.
///
/// # Errors
///
/// Returns [`LineReadError`](GtfGffError::LineReadError) if any line in the
/// GTF/GFF cannot be read.
pub fn read_gtf_gff(
    gtf_gff_path: &Path,
) -> Result<HashMap<TranscriptId, TranscriptSignature>, GtfGffError> {
    info!("{}", gtf_gff_path.display());

    let reader = open_gtf_gff_reader(gtf_gff_path);
    let mut gtf_gff_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|_| GtfGffError::LineReadError(gtf_gff_path.to_path_buf()))?;

        if !line.starts_with('#') {
            let line_split = line.split('\t').collect::<Vec<&str>>();

            if GtfGffRecord::is_exon_or_cds(&line_split) {
                let record = GtfGffRecord::from(&line_split)?;

                // Only insert chromosome and strand once, upon initialisation.
                let transcript_signature = gtf_gff_transcripts
                    .entry(record.transcript_id)
                    .or_insert(TranscriptSignature::from(
                        record.chr,
                        record.strand,
                        BTreeSet::new(),
                        BTreeSet::new(),
                    ));

                transcript_signature.insert_boundary(&record.feature, record.start)?;
                transcript_signature.insert_boundary(&record.feature, record.end)?;
            }
        }
    }

    Ok(gtf_gff_transcripts)
}

/// Write GTF/GFF file with unified transcript IDs.
///
/// # Errors
///
/// Returns [`UnknownExtensionError`](GtfGffError::UnknownExtensionError) if any
/// input file does not have a "gtf" or "gff" extension.
///
/// Returns [`FileCreateError`](GtfGffError::FileCreateError) if the output file
/// cannot be be created.
///
/// Returns [`LineReadError`](GtfGffError::LineReadError) if any line in the
/// GTF/GFF cannot be read.
///
/// Returns [`FileWriteError`](GtfGffError::FileWriteError) if any line in
/// the output GTF/GFF cannot be written.
pub fn write_unified_gtf_gff(
    gtf_gff_extension: &str,
    gtf_gff_path: &Path,
    output_dir: &Path,
    transcript_unifier: &TranscriptUnifier,
) -> Result<(), GtfGffError> {
    let gtf_gff_file_name = extract_file_name(gtf_gff_path);

    let mut output_path = output_dir.to_path_buf();
    output_path.push(gtf_gff_file_name.to_string());
    output_path.set_extension(format!("tuni.{}", gtf_gff_extension));

    info!("{}", output_path.display());

    let reader = open_gtf_gff_reader(gtf_gff_path);
    let mut writer = open_gtf_gff_writer(&output_path)?;

    let tuni_id_formatter = TuniIdFormatter::from(gtf_gff_extension)?;

    for line in reader.lines() {
        let mut line = line.map_err(|_| GtfGffError::LineReadError(gtf_gff_path.to_path_buf()))?;

        if !line.starts_with('#') {
            let line_split = line.split('\t').collect::<Vec<&str>>();
            let transcript_id = GtfGffRecord::get_transcript_id(&line_split);

            if let Some(transcript_id) = transcript_id {
                let unified_id = transcript_unifier
                    .get_unified_id(&[Rc::clone(&gtf_gff_file_name), Rc::from(transcript_id)]);

                match unified_id {
                    Some(unified_id) => line.push_str(&tuni_id_formatter.format(unified_id)),
                    None => warn!("Unrecognised transcript ID found {}", transcript_id),
                }
            }
        }

        writeln!(writer, "{}", line)
            .map_err(|_| GtfGffError::FileWriteError(output_path.clone()))?;
    }

    Ok(())
}

/// Isolate only the GTF/GFF file name from full path.
///
/// "/path/to/a.gtf" -> "a.gtf"
pub fn extract_file_name(gtf_gff_path: &Path) -> Rc<str> {
    // We have already checked GTF/GFF paths are valid files
    // with a ".gtf"/".gff" extension during cli argument parsing.
    Rc::from(gtf_gff_path.file_name().unwrap().to_str().unwrap())
}

/// Open reader that reads GTF/GFF line by line.
fn open_gtf_gff_reader(gtf_gff_path: &Path) -> BufReader<File> {
    // GTFs are checked to exist/be readable during cli argument parsing.
    let gtf_gff = File::open(gtf_gff_path).unwrap();

    // Avoid reading the entire file into memory at once.
    BufReader::new(gtf_gff)
}

/// Open writer that writes GTF/GFF line by line.
fn open_gtf_gff_writer(output_path: &Path) -> Result<BufWriter<File>, GtfGffError> {
    let unified_gtf_gff = File::create(output_path)
        .map_err(|_| GtfGffError::FileCreateError(output_path.to_path_buf()))?;
    Ok(BufWriter::new(unified_gtf_gff))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use rstest::rstest;
    use std::fs::read_to_string;
    use tempfile::tempdir;

    #[test]
    fn test_gtf_gff_record_from() {
        let line = r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#;
        let line_split = line.split('\t').collect::<Vec<&str>>();

        assert_eq!(
            GtfGffRecord::from(&line_split).unwrap(),
            GtfGffRecord {
                feature: Rc::from("exon"),
                strand: Rc::from("+"),
                chr: Rc::from("chr1"),
                start: Rc::from("1"),
                end: Rc::from("2"),
                transcript_id: Rc::from("transcript_id \"A\""),
            }
        );

        // No transcript_id field.
        let line = r#"chr1	RefSeq	gene	1	2	.	+	.	gene_id "A";"#;
        let line_split = line.split('\t').collect::<Vec<&str>>();
        assert!(GtfGffRecord::from(&line_split)
            .is_err_and(|e| e.to_string().contains("No transcript_id found in line")))
    }

    #[rstest]
    #[case(r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#, true)]
    #[case(r#"chr1	RefSeq	CDS	1	2	.	+	.	transcript_id "A";"#, true)]
    #[case(r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "A"#, false)]
    fn test_is_exon_or_cds(#[case] line: &str, #[case] expected: bool) {
        let line_split = line.split('\t').collect::<Vec<&str>>();

        assert_eq!(GtfGffRecord::is_exon_or_cds(&line_split), expected);
    }

    #[rstest]
    #[case(
        r#"chr1	RefSeq	exon	1	2	.	+	.	transcript_id "A";"#,
        Some("transcript_id \"A\"")
    )]
    #[case(
        r#"chr1	RefSeq	transcript	1	2	.	+	.	transcript_id "B";"#,
        Some("transcript_id \"B\"")
    )]
    #[case(r#"chr1	RefSeq	gene	1	2	.	+	.	gene_id "A";"#, None)]
    fn test_get_transcript_id(#[case] line: &str, #[case] expected: Option<&str>) {
        let line_split = line.split('\t').collect::<Vec<&str>>();

        match expected {
            Some(transcript_id) => {
                assert_eq!(
                    GtfGffRecord::get_transcript_id(&line_split).unwrap(),
                    transcript_id
                );
            }
            None => assert!(GtfGffRecord::get_transcript_id(&line_split).is_none()),
        }
    }

    #[test]
    fn test_transcript_signature() {
        let mut transcript_signature = TranscriptSignature::from(
            Rc::from("chr1"),
            Rc::from("+"),
            BTreeSet::new(),
            BTreeSet::new(),
        );

        transcript_signature
            .insert_boundary("exon", Rc::from("1"))
            .unwrap();
        transcript_signature
            .insert_boundary("CDS", Rc::from("2"))
            .unwrap();

        assert_eq!(
            transcript_signature.exon_boundaries,
            BTreeSet::from([Rc::from("1")])
        );
        assert_eq!(
            transcript_signature.cds_boundaries,
            BTreeSet::from([Rc::from("2")])
        );
        assert!(transcript_signature
            .insert_boundary("not_a_feature", Rc::from("1"))
            .is_err_and(|e| e.to_string().contains("Feature must be 'exon' or 'CDS'")))
    }

    #[test]
    fn test_read_gtf_gff() {
        let mut expected_transcripts: HashMap<TranscriptId, TranscriptSignature> = HashMap::new();

        expected_transcripts.insert(
            Rc::from("transcript_id \"A\""),
            TranscriptSignature::from(
                Rc::from("chr1"),
                Rc::from("-"),
                BTreeSet::from([Rc::from("1"), Rc::from("12"), Rc::from("11"), Rc::from("2")]),
                BTreeSet::new(),
            ),
        );

        expected_transcripts.insert(
            Rc::from("transcript_id \"B\""),
            TranscriptSignature::from(
                Rc::from("chr2"),
                Rc::from("+"),
                BTreeSet::from([Rc::from("20"), Rc::from("30")]),
                BTreeSet::from([Rc::from("25"), Rc::from("29")]),
            ),
        );

        assert_eq!(
            read_gtf_gff(&PathBuf::from("tests/data/unit/sample_1.gtf")).unwrap(),
            expected_transcripts
        )
    }

    #[test]
    fn test_write_unified_gtf() {
        let gtf_gff_path = PathBuf::from("tests/data/unit/sample_1.gtf");
        let mut gtf_gff_transcripts = read_gtf_gff(&gtf_gff_path).unwrap();

        let mut transcript_unifier = TranscriptUnifier::new();
        transcript_unifier.group_transcripts(Rc::from("sample_1.gtf"), &mut gtf_gff_transcripts);
        transcript_unifier.unify_transcripts();

        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("sample_1.tuni.gtf");
        write_unified_gtf_gff("gtf", &gtf_gff_path, temp_dir.path(), &transcript_unifier).unwrap();

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
