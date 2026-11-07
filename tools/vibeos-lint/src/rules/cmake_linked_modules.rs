//! Cargo supremacy's ratchet: every PE module cmake still links is a finding, so the
//! baseline is the remaining list and a NEW finding is a module that was added (or moved
//! back) outside a cargo cell. New ports aim for cargo from the start. Fix:
//! `tools/codegen cargo-cell --convert <target>`; when a conversion lands, drop its line
//! from tools/lint-baseline/cmake_linked_modules.txt. `tools/codegen cargo-cell
//! --cmake-linked` prints the same list.
use crate::{Corpus, Rule};

pub struct CmakeLinkedModules;

impl Rule for CmakeLinkedModules {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "cmake_linked_modules"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        vibeos_codegen::cells::driver::cmake_linked(&corpus.root).map_err(|e| e.to_string())
    }
    fn source_sets(&self) -> &'static [crate::SourceSet] {
        &[]
    }
}
