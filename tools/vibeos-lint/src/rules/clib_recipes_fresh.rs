//! The cargo cells' committed C-library recipes (sdk/lib/rust/reactos-link/clibs/<arch>/)
//! are DERIVED from cmake's compile commands. Nothing re-derived them when a CMake flag moved:
//! pseh's include went from -isystem to -I on 2026-11-06 and ~380 recipes drifted unseen,
//! because a drifted recipe only breaks a cmake-off build. Each finding is a recipe that
//! differs from what the configured build-<arch> compile databases derive, is missing, or
//! is read by no cell. Fix: `tools/codegen cargo-cell --clib-recipes`.
use crate::{Corpus, Rule};

pub struct ClibRecipesFresh;

impl Rule for ClibRecipesFresh {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "clib_recipes_fresh"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        vibeos_codegen::cells::driver::recipe_drift(&corpus.root).map_err(|e| e.to_string())
    }
    fn source_sets(&self) -> &'static [crate::SourceSet] {
        &[]
    }
}
