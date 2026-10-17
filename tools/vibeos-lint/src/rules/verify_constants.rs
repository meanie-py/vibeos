use crate::{Corpus, Rule};
pub struct VerifyConstants;
impl Rule for VerifyConstants {
    fn name(&self) -> &'static str {
        "verify_constants"
    }
    fn source_sets(&self) -> &'static [crate::SourceSet] {
        super::verify_constants_shared::INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(super::verify_constants_shared::findings(corpus, false))
    }
}
