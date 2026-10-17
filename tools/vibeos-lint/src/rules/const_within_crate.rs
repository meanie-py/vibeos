use crate::const_shared as shared;
use crate::{Corpus, Rule, SourceSet};
pub struct ConstWithinCrate;
const INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("const-within-crate-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target", ".responses"])
            .with_inputs(),
    ];
impl Rule for ConstWithinCrate {
    fn name(&self) -> &'static str {
        "scan_const_within_crate"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let d = shared::declarations(c, "const-within-crate-rust", false);
        Ok(shared::contradictions(&d, true)
            .into_iter()
            .map(|(n, p, v)| format!("{p}: {n} = {} [WITHIN_CRATE]", shared::render(&v)))
            .collect())
    }
}
