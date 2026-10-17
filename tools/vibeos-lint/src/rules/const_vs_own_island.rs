use crate::const_shared as shared;
use crate::{Corpus, Rule, SourceSet};
pub struct ConstVsOwnIsland;
const INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("const-vs-own-island-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target", ".responses"])
            .with_inputs(),
    ];
impl Rule for ConstVsOwnIsland {
    fn name(&self) -> &'static str {
        "scan_const_vs_own_island"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let d = shared::declarations(c, "const-vs-own-island-rust", true);
        let i = shared::own_islands(c.sources("const-vs-own-island-rust"));
        Ok(shared::island_hits(&d, &i, true)
            .into_iter()
            .map(|(n, p, v)| format!("{p}: {n} = {} [VS_OWN_ISLAND]", shared::render(&v)))
            .collect())
    }
}
