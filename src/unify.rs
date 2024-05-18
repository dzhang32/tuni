use crate::gtf::{TranscriptId, TranscriptSignature};
use std::collections::{BTreeMap, HashMap, HashSet};

pub type SampleTranscriptId = (String, String);
pub type UnifiedId = String;

const UNIFIED_ID_PREFIX: &str = "tuni_";

pub struct TranscriptUnifier {
    transcripts: BTreeMap<TranscriptSignature, HashSet<SampleTranscriptId>>,
    unified_transcripts: HashMap<SampleTranscriptId, UnifiedId>,
}

impl TranscriptUnifier {
    pub fn new() -> TranscriptUnifier {
        TranscriptUnifier {
            transcripts: BTreeMap::new(),
            unified_transcripts: HashMap::new(),
        }
    }

    pub fn add_transcripts(
        &mut self,
        gtf_file_name: String,
        gtf_transcripts: &mut HashMap<TranscriptId, TranscriptSignature>,
    ) {
        // TODO: optimise clones.
        for (transcript_id, transcript_signature) in gtf_transcripts.drain() {
            let sample_transcript_id = self.transcripts.entry(transcript_signature).or_default();
            sample_transcript_id.insert((gtf_file_name.clone(), transcript_id.clone()));
        }
    }

    pub fn unify_transcripts(&mut self) {
        // TODO: Check if .drain should be used.
        for (i, sample_transcript_ids) in self.transcripts.values_mut().enumerate() {
            for sample_transcript_id in sample_transcript_ids.drain() {
                self.unified_transcripts
                    .insert(sample_transcript_id, format!("{}{}", UNIFIED_ID_PREFIX, i));
            }
        }
    }

    pub fn get_unified_id(&self, sample_transcript_id: &SampleTranscriptId) -> &str {
        // TODO: Handle errors better.
        self.unified_transcripts.get(sample_transcript_id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtf::read_gtf;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    fn test_transcript_unifier() {
        let mut transcript_unifier = TranscriptUnifier::new();

        // Sample 2 is an unsorted GTF, ensuring unification works
        // regardless if input is sorted.
        let gtf_paths = [
            PathBuf::from("tests/data/unit/sample_1.gtf"),
            PathBuf::from("tests/data/unit/sample_2.gtf"),
        ];
        for gtf_path in gtf_paths {
            let mut gtf_transcripts = read_gtf(&gtf_path);
            let gtf_file_name = gtf_path.file_name().unwrap().to_str().unwrap();
            transcript_unifier.add_transcripts(gtf_file_name.to_string(), &mut gtf_transcripts);
        }

        let expected_transcripts = BTreeMap::from([
            (
                TranscriptSignature::from(
                    "chr1".to_string(),
                    "-".to_string(),
                    BTreeSet::from([
                        "1".to_string(),
                        "11".to_string(),
                        "12".to_string(),
                        "2".to_string(),
                    ]),
                    BTreeSet::new(),
                ),
                HashSet::from([
                    ("sample_1.gtf".to_string(), "A".to_string()),
                    ("sample_2.gtf".to_string(), "A_2".to_string()),
                ]),
            ),
            (
                TranscriptSignature::from(
                    "chr2".to_string(),
                    "+".to_string(),
                    BTreeSet::from(["20".to_string(), "30".to_string()]),
                    BTreeSet::from(["25".to_string(), "29".to_string()]),
                ),
                HashSet::from([("sample_1.gtf".to_string(), "B".to_string())]),
            ),
            (
                TranscriptSignature::from(
                    "chr2".to_string(),
                    "+".to_string(),
                    BTreeSet::from(["20".to_string(), "30".to_string()]),
                    BTreeSet::from(["26".to_string(), "28".to_string()]),
                ),
                HashSet::from([("sample_2.gtf".to_string(), "C".to_string())]),
            ),
        ]);

        assert_eq!(transcript_unifier.transcripts, expected_transcripts);

        transcript_unifier.unify_transcripts();

        let expected_unified_transcripts = HashMap::from([
            (
                ("sample_1.gtf".to_string(), "A".to_string()),
                "tuni_0".to_string(),
            ),
            (
                ("sample_1.gtf".to_string(), "B".to_string()),
                "tuni_1".to_string(),
            ),
            (
                ("sample_2.gtf".to_string(), "A_2".to_string()),
                "tuni_0".to_string(),
            ),
            (
                ("sample_2.gtf".to_string(), "C".to_string()),
                "tuni_2".to_string(),
            ),
        ]);

        assert_eq!(
            transcript_unifier.unified_transcripts,
            expected_unified_transcripts
        );
    }
}
