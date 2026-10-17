use crate::const_shared as shared;
use crate::{Corpus, Rule, SourceSet};
pub struct ConstVsIsland;
const INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("const-vs-island-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target", ".responses"])
            .with_inputs(),
    ];
impl Rule for ConstVsIsland {
    fn name(&self) -> &'static str {
        "scan_const_vs_island"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let d = shared::declarations(c, "const-vs-island-rust", true);
        let i = shared::islands(c.sources("const-vs-island-rust"), false);
        Ok(shared::island_hits(&d, &i, false)
            .into_iter()
            .map(|(n, p, v)| format!("{p}: {n} = {} [VS_ISLAND]", shared::render(&v)))
            .collect())
    }
}
