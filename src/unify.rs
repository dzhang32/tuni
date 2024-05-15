use crate::gtf::{TranscriptId, TranscriptSignature};
use std::collections::{HashMap, HashSet};

pub type SampleTranscriptId = (String, String);
pub type UnifiedId = String;
const UNIFIED_ID_PREFIX: &str = "tuni_";

struct TranscriptUnifier {
    transcripts: HashMap<TranscriptSignature, HashSet<SampleTranscriptId>>,
    unified_transcripts: HashMap<SampleTranscriptId, UnifiedId>,
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

    fn unify_transcripts(&mut self) {
        // TODO: Check if .drain should be used.
        for (i, sample_transcript_ids) in self.transcripts.values_mut().enumerate() {
            for sample_transcript_id in sample_transcript_ids.drain() {
                self.unified_transcripts
                    .insert(sample_transcript_id, format!("{}{}", UNIFIED_ID_PREFIX, i));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtf::load_gtf;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    fn test_add_transcripts() {
        let mut transcript_unifier = TranscriptUnifier::new();
        let gtf_paths = [
            PathBuf::from("tests/data/test_sample_1.gtf"),
            PathBuf::from("tests/data/test_sample_2.gtf"),
        ];
        for gtf_path in gtf_paths {
            let mut gtf_transcripts = load_gtf(&gtf_path);
            let gtf_file_name = gtf_path.file_name().unwrap().to_str().unwrap();
            transcript_unifier.add_transcripts(gtf_file_name.to_string(), &mut gtf_transcripts);
        }

        let expected_transcripts = HashMap::from([
            (
                BTreeSet::from([
                    "chr1".to_string(),
                    "-".to_string(),
                    "1".to_string(),
                    "11".to_string(),
                    "12".to_string(),
                    "2".to_string(),
                ]),
                HashSet::from([
                    ("test_sample_1.gtf".to_string(), "A".to_string()),
                    ("test_sample_2.gtf".to_string(), "A_2".to_string()),
                ]),
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
            (
                BTreeSet::from([
                    "chr3".to_string(),
                    "+".to_string(),
                    "41".to_string(),
                    "42".to_string(),
                    "51".to_string(),
                    "52".to_string(),
                ]),
                HashSet::from([("test_sample_2.gtf".to_string(), "C".to_string())]),
            ),
        ]);

        assert_eq!(transcript_unifier.transcripts, expected_transcripts);

        // unify_transcripts() loops through transcripts in an arbitrary order,
        // so we don't know (and cannot test) the exact unified ID that is
        // assigned to each transcript.
        transcript_unifier.unify_transcripts();

        let mut sample_transcript_ids = transcript_unifier
            .unified_transcripts
            .clone()
            .into_keys()
            .collect::<Vec<SampleTranscriptId>>();

        // into_keys() returns keys in an arbitary order.
        // sort() ensures keys match the order of this for comparison.
        sample_transcript_ids.sort();

        let expected_sample_transcript_ids = vec![
            ("test_sample_1.gtf".to_string(), "A".to_string()),
            ("test_sample_1.gtf".to_string(), "B".to_string()),
            ("test_sample_2.gtf".to_string(), "A_2".to_string()),
            ("test_sample_2.gtf".to_string(), "C".to_string()),
        ];

        assert_eq!(sample_transcript_ids, expected_sample_transcript_ids);

        // A and A_2 are the same transcript from different samples.
        // They should be assigned the same unified ID.
        assert_eq!(
            transcript_unifier
                .unified_transcripts
                .get(&("test_sample_1.gtf".to_string(), "A".to_string())),
            transcript_unifier
                .unified_transcripts
                .get(&("test_sample_2.gtf".to_string(), "A_2".to_string()))
        );
    }
}
