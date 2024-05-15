use crate::gtf::{TranscriptId, TranscriptSignature};
use std::collections::{HashMap, HashSet};

pub type SampleTranscriptIds = HashSet<(String, String)>;
pub type UnifiedId = String;
const UNIFIED_ID_PREFIX: &str = "tuni_";

struct TranscriptUnifier {
    transcripts: HashMap<TranscriptSignature, SampleTranscriptIds>,
    unified_transcripts: HashMap<TranscriptId, UnifiedId>,
}

impl TranscriptUnifier {
    fn new() -> TranscriptUnifier {
        TranscriptUnifier {
            transcripts: HashMap::new(),
            unified_transcripts: HashMap::new(),
        }
    }

    fn add_transcripts(
        &mut self,
        gtf_file_name: String,
        gtf_transcripts: &mut HashMap<TranscriptId, TranscriptSignature>,
    ) {
        for (transcript_id, transcript_signature) in gtf_transcripts.drain() {
            let sample_transcript_id = self.transcripts.entry(transcript_signature).or_default();
            sample_transcript_id.insert((gtf_file_name.clone(), transcript_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtf::load_gtf;
    use std::{collections::BTreeSet, path::PathBuf};

    #[test]
    fn test_add_transcripts() {
        let mut transcript_unifier = TranscriptUnifier::new();
        let mut gtf_transcripts = load_gtf(PathBuf::from("tests/data/test_sample_1.gtf"));
        transcript_unifier.add_transcripts("test_sample_1.gtf".to_string(), &mut gtf_transcripts);

        let transcript_signature_a = BTreeSet::from([
            "chr1".to_string(),
            "-".to_string(),
            "1".to_string(),
            "11".to_string(),
            "12".to_string(),
            "2".to_string(),
        ]);
        let mut expected = HashMap::from([
            (
                transcript_signature_a.clone(),
                HashSet::from([("test_sample_1.gtf".to_string(), "A".to_string())]),
            ),
            (
                BTreeSet::from([
                    "chr2".to_string(),
                    "+".to_string(),
                    "21".to_string(),
                    "22".to_string(),
                    "31".to_string(),
                    "32".to_string(),
                ]),
                HashSet::from([("test_sample_1.gtf".to_string(), "B".to_string())]),
            ),
        ]);

        assert_eq!(transcript_unifier.transcripts, expected);

        // Test unification works correctly across multiple samples.
        let mut gtf_transcripts = load_gtf(PathBuf::from("tests/data/test_sample_2.gtf"));
        transcript_unifier.add_transcripts("test_sample_2.gtf".to_string(), &mut gtf_transcripts);

        expected
            .get_mut(&transcript_signature_a)
            .unwrap()
            .insert(("test_sample_2.gtf".to_string(), "A_2".to_string()));

        expected.insert(
            BTreeSet::from([
                "chr3".to_string(),
                "+".to_string(),
                "41".to_string(),
                "42".to_string(),
                "51".to_string(),
                "52".to_string(),
            ]),
            HashSet::from([("test_sample_2.gtf".to_string(), "C".to_string())]),
        );

        assert_eq!(transcript_unifier.transcripts, expected);
    }
}
