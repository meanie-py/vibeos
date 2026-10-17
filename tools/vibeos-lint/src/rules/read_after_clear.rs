//! Reads through a location just atomically cleared instead of using its old value.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct ReadAfterClear;
impl Rule for ReadAfterClear {
    fn name(&self) -> &'static str {
        "scan_read_after_clear"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut findings = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            let code = file.masked(&patterns().comments, Some("ORIGINAL C SOURCE"), true);
            for (line, what) in scan_code(&code) {
                findings.push(format!(
                    "{}:{line}: read-after-clear {what}",
                    file.path.display()
                ));
            }
        }
        findings.sort();
        Ok(findings)
    }
}
struct Patterns {
    skip: Regex,
    comments: Regex,
    clear: Regex,
    function: Regex,
    read: Regex,
}
fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let re = |s| Regex::new(s).unwrap();
        Patterns {
            skip: re(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"),
            comments: re(r"(?s)//[^\n]*|/\*.*?\*/"),
            clear: re(r"(?:\w+::)*InterlockedExchange(?:Pte|Pointer|64|SizeT)?\s*\(\s*(\w+)\s*(?:as\s+[\w:*\s]+?)?,\s*(?:0|(?:core::)?(?:ptr::)?null_mut\(\))\s*\)"),
            function: re(r"\bfn\s+\w+"),
            read: re(r"\(\*(\w+)\)\s*\.\s*\w+"),
        }
    })
}
#[cfg(test)]
fn scan_text(raw: &str) -> Vec<(usize, String)> {
    let raw = raw.split("ORIGINAL C SOURCE").next().unwrap();
    let p = patterns();
    let code = p
        .comments
        .replace_all(raw, |caps: &regex::Captures<'_>| {
            caps.get(0)
                .unwrap()
                .as_str()
                .chars()
                .map(|ch| if ch == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned();
    scan_code(&code)
}
fn scan_code(code: &str) -> Vec<(usize, String)> {
    let p = patterns();
    let mut findings = Vec::new();
    for clear in p.clear.captures_iter(&code) {
        let end = clear.get(0).unwrap().end();
        let var = clear.get(1).unwrap().as_str();
        let tail = &code[end..];
        let mut tail = &tail[..tail
            .char_indices()
            .nth(900)
            .map_or(tail.len(), |(at, _)| at)];
        if let Some(function) = p.function.find(tail) {
            tail = &tail[..function.start()];
        }
        if let Some(read) = p
            .read
            .captures_iter(tail)
            .find(|c| c.get(1).unwrap().as_str() == var)
        {
            let read = read.get(0).unwrap();
            let line = code[..end + read.start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            findings.push((
                line,
                format!(
                    "{var} cleared, then {}",
                    read.as_str()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            ));
        }
    }
    findings.sort();
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pte_read_after_clear() {
        let code = "fn f() {\nOldPte.Long = InterlockedExchangePte(PointerPte, 0);\n*WasDirty = (((*PointerPte).Long >> 6) & 1) as BOOLEAN;\n}";
        assert_eq!(
            scan_text(code),
            vec![(3, "PointerPte cleared, then (*PointerPte).Long".into())]
        );
    }
    #[test]
    fn namespaced_cast_and_null_pointer() {
        let code =
            "nt::InterlockedExchangePointer(P as *mut PVOID, core::ptr::null_mut()); (*P) . field";
        assert_eq!(
            scan_text(code),
            vec![(1, "P cleared, then (*P) . field".into())]
        );
    }
    #[test]
    fn does_not_cross_next_function_original_source_or_comments() {
        assert!(scan_text("InterlockedExchange(P, 0); fn next() { (*P).Long }").is_empty());
        assert!(scan_text("InterlockedExchange(P, 0); ORIGINAL C SOURCE (*P).Long").is_empty());
        assert!(scan_text("InterlockedExchange(P, 0); // (*P).Long").is_empty());
    }
    #[test]
    fn only_matching_pointer_within_character_window() {
        assert!(scan_text("InterlockedExchange(P, 0); (*Other).Long").is_empty());
        let code = format!("InterlockedExchange(P, 0); {} (*P).Long", "é".repeat(800));
        assert_eq!(scan_text(&code).len(), 1);
        let code = format!("InterlockedExchange(P, 0); {} (*P).Long", " ".repeat(901));
        assert!(scan_text(&code).is_empty());
    }
    #[test]
    fn old_value_and_nonzero_exchange_are_clean() {
        assert!(scan_text("old = InterlockedExchangePte(P, 0); old.Long").is_empty());
        assert!(scan_text("InterlockedExchange64(P, 1); (*P).Long").is_empty());
    }
}
