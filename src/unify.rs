use crate::gtf::{TranscriptId, TranscriptSignature};
use std::collections::HashMap;

pub type UnifiedId = String;
const UNIFIED_ID_PREFIX: &str = "tuni_";

struct UnifiedTranscripts {
    count: u64,
    unified_transcripts: HashMap<TranscriptSignature, UnifiedId>,
}

impl UnifiedTranscripts {
    fn new() -> UnifiedTranscripts {
        UnifiedTranscripts {
            count: 1,
            unified_transcripts: HashMap::new(),
        }
    }

    fn update(&mut self, gtf_transcripts: &HashMap<TranscriptId, TranscriptSignature>) {
        for transcript_signature in gtf_transcripts.values() {
            if !self.unified_transcripts.contains_key(transcript_signature) {
                self.unified_transcripts.insert(
                    // TODO: optimise this by switching to String type
                    // whose ownership can be shared.
                    transcript_signature.clone(),
                    format!("{}{}", UNIFIED_ID_PREFIX, self.count),
                );
                self.count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtf::load_gtf;
    use std::{collections::BTreeSet, path::PathBuf};

    #[test]
    fn test_unify_transcripts() {
        let mut unified_transcripts = UnifiedTranscripts::new();
        let gtf_transcripts = load_gtf(PathBuf::from("tests/data/test_sample_1.gtf"));
        unified_transcripts.update(&gtf_transcripts);

        let mut expected = HashMap::from([
            (
                BTreeSet::from([
                    "chr1".to_string(),
                    "-".to_string(),
                    "1".to_string(),
                    "11".to_string(),
                    "12".to_string(),
                    "2".to_string(),
                ]),
                "tuni_1".to_string(),
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
                "tuni_2".to_string(),
            ),
        ]);

        assert_eq!(unified_transcripts.unified_transcripts, expected);

        // Test unification works correctly across multiple samples.
        let gtf_transcripts = load_gtf(PathBuf::from("tests/data/test_sample_2.gtf"));
        unified_transcripts.update(&gtf_transcripts);

        expected.insert(
            BTreeSet::from([
                "chr3".to_string(),
                "+".to_string(),
                "41".to_string(),
                "42".to_string(),
                "51".to_string(),
                "52".to_string(),
            ]),
            "tuni_3".to_string(),
        );

        assert_eq!(unified_transcripts.unified_transcripts, expected);
    }
}
