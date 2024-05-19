use crate::gtf::{TranscriptId, TranscriptSignature};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

pub type SampleTranscriptId = (Rc<str>, Rc<str>);
pub type UnifiedId = Rc<str>;

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
        gtf_file_name: Rc<str>,
        gtf_transcripts: &mut HashMap<TranscriptId, TranscriptSignature>,
    ) {
        // TODO: should I drain()?
        for (transcript_id, transcript_signature) in gtf_transcripts.drain() {
            let sample_transcript_id = self.transcripts.entry(transcript_signature).or_default();
            sample_transcript_id.insert((Rc::clone(&gtf_file_name), Rc::clone(&transcript_id)));
        }
    }

    pub fn unify_transcripts(&mut self) {
        // TODO: Check if .drain should be used.
        for (i, sample_transcript_ids) in self.transcripts.values_mut().enumerate() {
            for sample_transcript_id in sample_transcript_ids.drain() {
                self.unified_transcripts.insert(
                    sample_transcript_id,
                    Rc::from(format!("{}{}", UNIFIED_ID_PREFIX, i)),
                );
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
            transcript_unifier.add_transcripts(Rc::from(gtf_file_name), &mut gtf_transcripts);
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
                    (Rc::from("sample_1.gtf"), Rc::from("A")),
                    (Rc::from("sample_2.gtf"), Rc::from("A_2")),
                ]),
            ),
            (
                TranscriptSignature::from(
                    Rc::from("chr2"),
                    Rc::from("+"),
                    BTreeSet::from([Rc::from("20"), Rc::from("30")]),
                    BTreeSet::from([Rc::from("25"), Rc::from("29")]),
                ),
                HashSet::from([(Rc::from("sample_1.gtf"), Rc::from("B"))]),
            ),
            (
                TranscriptSignature::from(
                    Rc::from("chr2"),
                    Rc::from("+"),
                    BTreeSet::from([Rc::from("20"), Rc::from("30")]),
                    BTreeSet::from([Rc::from("26"), Rc::from("28")]),
                ),
                HashSet::from([(Rc::from("sample_2.gtf"), Rc::from("C"))]),
            ),
        ]);

        assert_eq!(transcript_unifier.transcripts, expected_transcripts);

        transcript_unifier.unify_transcripts();

        let expected_unified_transcripts = HashMap::from([
            (
                (Rc::from("sample_1.gtf"), Rc::from("A")),
                Rc::from("tuni_0"),
            ),
            (
                (Rc::from("sample_1.gtf"), Rc::from("B")),
                Rc::from("tuni_1"),
            ),
            (
                (Rc::from("sample_2.gtf"), Rc::from("A_2")),
                Rc::from("tuni_0"),
            ),
            (
                (Rc::from("sample_2.gtf"), Rc::from("C")),
                Rc::from("tuni_2"),
            ),
        ]);

        assert_eq!(
            transcript_unifier.unified_transcripts,
            expected_unified_transcripts
        );
    }
}
