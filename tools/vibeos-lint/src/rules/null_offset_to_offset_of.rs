use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::sync::LazyLock;

pub struct NullOffsetToOffsetOf;
const SETS: &[SourceSet] =
    &[SourceSet::tree("rust-inputs", &["sdk/lib/rust"], &["rs"], &[]).with_inputs()];
static NULL_PLACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\(\s*\*\s*(?:::)?(?:core::ptr::|ptr::|std::ptr::)?null(?:_mut)?(?:::<[^>]*>)?\s*\(\s*\)",
    )
    .unwrap()
});
fn in_comment(text: &str, at: usize) -> bool {
    let start = text[..at].rfind('\n').map_or(0, |n| n + 1);
    text[start..at].contains("//")
}
fn hits(f: &SourceFile) -> usize {
    if !f.text.contains("null") {
        return 0;
    }
    NULL_PLACE
        .find_iter(&f.text)
        .filter(|m| !in_comment(&f.text, m.start()))
        .count()
}
impl Rule for NullOffsetToOffsetOf {
    fn name(&self) -> &'static str {
        "null_offset_to_offset_of"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SETS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for f in c
            .sources("rust-inputs")
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
        {
            for _ in 0..hits(f) {
                out.push(format!(
                    "{}: [NULL_PLACE]",
                    f.path.to_string_lossy().replace('\\', "/")
                ));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Corpus;
    use std::path::PathBuf;
    fn f(s: &str) -> SourceFile {
        SourceFile::new(PathBuf::from("rust/eventlog-rust/src/file.rs"), s.into())
    }
    const BAD:&str="unsafe fn f(e: PLIST_ENTRY) -> PLOGFILE {\n    let off = &raw mut (*::core::ptr::null_mut::<LOGFILE>()).ListEntry as ULONG_PTR;\n    (e as ULONG_PTR).wrapping_sub(off) as PLOGFILE\n}\n";
    const GOOD:&str="unsafe fn f(e: PLIST_ENTRY) -> PLOGFILE {\n    // was &raw mut (*core::ptr::null_mut::<LOGFILE>()).ListEntry\n    let off = core::mem::offset_of!(LOGFILE, ListEntry) as ULONG_PTR;\n    (e as ULONG_PTR).wrapping_sub(off) as PLOGFILE\n}\n";
    #[test]
    fn scanner_control_bad_and_good() {
        assert_eq!(hits(&f(BAD)), 1);
        assert_eq!(hits(&f(GOOD)), 0);
    }
    #[test]
    fn all_null_place_spellings_and_comment_quirk() {
        let s="let a = (*core::ptr::null::<T>()).x; let b = (*ptr::null_mut::<T>()).x; let c = (*std::ptr::null_mut::<T>()).x; // (*core::ptr::null_mut::<T>()).x\n";
        assert_eq!(hits(&f(s)), 3);
    }
    #[test]
    fn exact_paths_findings() {
        let c = Corpus::new(PathBuf::from("."), vec![f(BAD), f(GOOD)]);
        assert_eq!(
            NullOffsetToOffsetOf.run(&c).unwrap(),
            ["rust/eventlog-rust/src/file.rs: [NULL_PLACE]"]
        );
    }
}
