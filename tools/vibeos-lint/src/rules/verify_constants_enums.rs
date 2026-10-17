use crate::{Corpus, Rule};
pub struct VerifyConstantsEnums;
impl Rule for VerifyConstantsEnums {
    fn name(&self) -> &'static str {
        "verify_constants_enums"
    }
    fn source_sets(&self) -> &'static [crate::SourceSet] {
        super::verify_constants_shared::INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(super::verify_constants_shared::findings(corpus, true))
    }
}
