//! Detect dereferencing an extra pointer level after casting to a pointer typedef.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct PtrTypedefCastRead;
impl Rule for PtrTypedefCastRead {
    fn name(&self) -> &'static str {
        "scan_ptr_typedef_cast_read"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            let code = code_only(&file.text);
            for m in patterns().cast_read.captures_iter(&code) {
                let whole = m.get(0).unwrap();
                let line = code[..whole.start()]
                    .bytes()
                    .filter(|b| *b == b'\n')
                    .count()
                    + 1;
                if patterns().nolint.is_match(file.line_text(line)) {
                    continue;
                }
                out.push(format!(
                    "{}:{line}: ptr-typedef-cast-read {}",
                    file.path.display(),
                    &m[1]
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    comments: Regex,
    cast_read: Regex,
    nolint: Regex,
    skip: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        comments: crate::regexes::cached(r"(?s)//[^\n]*|/\*.*?\*/").unwrap(),
        cast_read: crate::regexes::cached(r"\(\*\([^;\n]{1,200}?\s+as\s+\*(?:mut|const)\s+(L?P[A-Z][A-Za-z0-9_]*)\s*\)\s*\.read\(\)\s*\)").unwrap(),
        nolint: crate::regexes::cached(r"nolint:\s*ptr-typedef-cast-read\b").unwrap(),
        skip: crate::regexes::cached(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)").unwrap(),
    })
}
fn code_only(text: &str) -> String {
    patterns()
        .comments
        .replace_all(text, |m: &regex::Captures<'_>| {
            m[0].chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}
#[cfg(test)]
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let code = code_only(text);
    let lines: Vec<_> = text.lines().collect();
    patterns()
        .cast_read
        .captures_iter(&code)
        .filter_map(|m| {
            let start = m.get(0)?.start();
            let line = code[..start].bytes().filter(|b| *b == b'\n').count() + 1;
            if patterns()
                .nolint
                .is_match(lines.get(line - 1).copied().unwrap_or(""))
            {
                None
            } else {
                Some((line, m[1].to_owned()))
            }
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    const BAD: &str = "pub unsafe fn locale(e: PVOID) -> PCWSTR {\n    // C: ((PGENENTRY)e)->Id\n    (*(e as *mut PGENENTRY).read()).Id\n}\n";
    const GOOD: &str = "pub unsafe fn locale(e: PVOID) -> PCWSTR {\n    (*(e as PGENENTRY)).Id\n}\npub unsafe fn slot(h: *mut PVOID) -> PVOID {\n    (h as *mut PVOID).read()\n}\n";
    #[test]
    fn dedicated_and_control_fixtures() {
        assert_eq!(
            scan_text(
                "\nLocaleId = (*(GetListEntryData(ListEntry) as *mut PGENENTRY).read()).Id;\n"
            ),
            vec![(2, "PGENENTRY".into())]
        );
        assert_eq!(scan_text(BAD), vec![(3, "PGENENTRY".into())]);
        assert!(scan_text(GOOD).is_empty());
        let reviewed = "let x = (*(slot as *mut PFOO).read()).Next; // nolint: ptr-typedef-cast-read (slot holds a PFOO)\n";
        assert!(scan_text(reviewed).is_empty());
    }
}
