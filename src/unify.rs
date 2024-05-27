use crate::gtf_gff::{TranscriptId, TranscriptSignature};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

/// Sample grouped with transcript ID.
///
/// Used to uniquely identify transcripts across samples, as separate samples
/// could use the same ID for different transcripts.
pub type SampleTranscriptId = [Rc<str>; 2];

/// `UnifiedId` will be same for the same transcript across different samples.
pub type UnifiedId = Rc<str>;

/// Forms `UnifiedId` along with an integer e.g. "tuni_1".
const UNIFIED_ID_PREFIX: &str = "tuni_";

/// Unify transcript IDs across different samples.
///
/// Groups together same transcripts (that share the same `TranscriptSignature`)
/// across different samples, then creates a `UnifiedId` that identifies each
/// transcript.
pub struct TranscriptUnifier {
    /// Using the `TranscriptSignature` as a key, group transcripts across
    /// different samples.
    ///
    /// BTreeMap here trades some performance (as insertion/lookup is log(N))
    /// to retain order of keys, so each transcript ID is given the same
    /// unified ID every time. This is helpful for testing but could be
    /// swapped for a HashMap if performance is key.
    grouped_transcripts: BTreeMap<TranscriptSignature, HashSet<SampleTranscriptId>>,

    /// Link each sample transcript ID to a unified ID.
    unified_transcripts: HashMap<SampleTranscriptId, UnifiedId>,
}

impl TranscriptUnifier {
    /// Initialise `TranscriptUnifier`.
    pub fn new() -> TranscriptUnifier {
        TranscriptUnifier {
            grouped_transcripts: BTreeMap::new(),
            unified_transcripts: HashMap::new(),
        }
    }

    /// Group transcripts across different samples under the same
    /// `TranscriptSignature`.
    pub fn group_transcripts(
        &mut self,
        gtf_gff_file_name: Rc<str>,
        gtf_gff_transcripts: &mut HashMap<TranscriptId, TranscriptSignature>,
    ) {
        for (transcript_id, transcript_signature) in gtf_gff_transcripts.drain() {
            let sample_transcript_id = self
                .grouped_transcripts
                .entry(transcript_signature)
                .or_default();
            sample_transcript_id.insert([Rc::clone(&gtf_gff_file_name), Rc::clone(&transcript_id)]);
        }
    }

    /// Create a unified ID for each unique `TranscriptSignature`.
    pub fn unify_transcripts(&mut self) {
        for (i, sample_transcript_ids) in self.grouped_transcripts.values_mut().enumerate() {
            for sample_transcript_id in sample_transcript_ids.drain() {
                self.unified_transcripts.insert(
                    sample_transcript_id,
                    Rc::from(format!("{}{}", UNIFIED_ID_PREFIX, i)),
                );
            }
        }
    }

    /// Obtain unified ID based on (sample, transcript ID).
    ///
    /// Returns unified ID if present, otherwise `None`.
    pub fn get_unified_id(&self, sample_transcript_id: &SampleTranscriptId) -> Option<&Rc<str>> {
        self.unified_transcripts.get(sample_transcript_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtf_gff;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    fn test_transcript_unifier() {
        let mut transcript_unifier = TranscriptUnifier::new();

        // Sample 2 is an unsorted GTF, ensuring unification works
        // regardless if input is sorted.
        let gtf_gff_paths = [
            PathBuf::from("tests/data/unit/sample_1.gtf"),
            PathBuf::from("tests/data/unit/sample_2.gtf"),
        ];
        for gtf_gff_path in gtf_gff_paths {
            let mut gtf_gff_transcripts = gtf_gff::read_gtf_gff(&gtf_gff_path).unwrap();
            let gtf_file_name = gtf_gff::extract_file_name(&gtf_gff_path);
            transcript_unifier.group_transcripts(gtf_file_name, &mut gtf_gff_transcripts);
        }

        let expected_transcripts = BTreeMap::from([
            (
                TranscriptSignature::from(
                    Rc::from("chr1"),
                    Rc::from("-"),
                    BTreeSet::from([Rc::from("1"), Rc::from("11"), Rc::from("12"), Rc::from("2")]),
                    BTreeSet::new(),
                ),
                HashSet::from([
                    [Rc::from("sample_1.gtf"), Rc::from("transcript_id \"A\"")],
                    [Rc::from("sample_2.gtf"), Rc::from("transcript_id \"A_2\"")],
                ]),
            ),
            (
                TranscriptSignature::from(
                    Rc::from("chr2"),
                    Rc::from("+"),
                    BTreeSet::from([Rc::from("20"), Rc::from("30")]),
                    BTreeSet::from([Rc::from("25"), Rc::from("29")]),
                ),
                HashSet::from([[Rc::from("sample_1.gtf"), Rc::from("transcript_id \"B\"")]]),
            ),
            (
                TranscriptSignature::from(
                    Rc::from("chr2"),
                    Rc::from("+"),
                    BTreeSet::from([Rc::from("20"), Rc::from("30")]),
                    BTreeSet::from([Rc::from("26"), Rc::from("28")]),
                ),
                HashSet::from([[Rc::from("sample_2.gtf"), Rc::from("transcript_id \"C\"")]]),
            ),
        ]);

        assert_eq!(transcript_unifier.grouped_transcripts, expected_transcripts);

        transcript_unifier.unify_transcripts();

        let expected_unified_transcripts = HashMap::from([
            (
                [Rc::from("sample_1.gtf"), Rc::from("transcript_id \"A\"")],
                Rc::from("tuni_0"),
            ),
            (
                [Rc::from("sample_1.gtf"), Rc::from("transcript_id \"B\"")],
                Rc::from("tuni_1"),
            ),
            (
                [Rc::from("sample_2.gtf"), Rc::from("transcript_id \"A_2\"")],
                Rc::from("tuni_0"),
            ),
            (
                [Rc::from("sample_2.gtf"), Rc::from("transcript_id \"C\"")],
                Rc::from("tuni_2"),
            ),
        ]);

        assert_eq!(
            transcript_unifier.unified_transcripts,
            expected_unified_transcripts
        );
    }
}
