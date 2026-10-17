use crate::const_shared as shared;
use crate::{Corpus, Rule, SourceSet};
pub struct ConstContradictions;
const INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("const-contradictions-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target", ".responses"])
            .with_inputs(),
    ];
impl Rule for ConstContradictions {
    fn name(&self) -> &'static str {
        "scan_const_contradictions"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let d = shared::declarations(c, "const-contradictions-rust", false);
        let hits = shared::contradictions(&d, false);
        Ok(hits
            .into_iter()
            .map(|(n, p, v)| format!("{p}: {n} = {}", shared::render(&v)))
            .collect())
    }
}
