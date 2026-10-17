//! Detect C assertion diagnostics translated into undefined behavior or kernel hangs.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct AssertToUb;
impl Rule for AssertToUb {
    fn name(&self) -> &'static str {
        "scan_assert_to_ub"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            let path = file.path.to_string_lossy();
            if skip_file(&path) {
                continue;
            }
            for (line, kind, _) in assertion_facts(&file.text) {
                out.push(format!("{path}:{line}: assert-to-{kind}"));
            }
        }
        Ok(out)
    }
}
fn re(slot: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    slot.get_or_init(|| Regex::new(pat).unwrap())
}
fn skip_file(path: &str) -> bool {
    static SKIP: OnceLock<Regex> = OnceLock::new();
    re(
        &SKIP,
        r"(bindgen_[a-z0-9_]+\.rs$|/target/|/\.responses/|/tests?/)",
    )
    .is_match(path)
}
fn assertion_facts(raw: &str) -> Vec<(usize, &'static str, String)> {
    static STRINGS: OnceLock<Regex> = OnceLock::new();
    static NOLINT: OnceLock<Regex> = OnceLock::new();
    static HIT: OnceLock<Regex> = OnceLock::new();
    static PANIC: OnceLock<Regex> = OnceLock::new();
    static ORIGIN: OnceLock<Regex> = OnceLock::new();
    static NORETURN: OnceLock<Regex> = OnceLock::new();
    let strings = re(&STRINGS, r#"(?s)b?"(?:\\.|[^"\\])*""#);
    let nolint = re(&NOLINT, r"//\s*nolint(?::\s*assert-to-ub\b|\s*$|:?\s*\()");
    let hit = re(&HIT, r"\bunreachable_unchecked\s*\(");
    let panic = re(
        &PANIC,
        r"\b(?:core::)?(?:panic|unreachable|todo|unimplemented)!\s*\(",
    );
    let origin = re(
        &ORIGIN,
        r"(NT_ASSERT|PAGED_CODE|Pageable code|IRQL too high for paged|ASSERT|assert:|Assertion)",
    );
    let noreturn = re(
        &NORETURN,
        r"(?i)(BugCheck|RaiseStatus|RaiseException|KiSystemStartup|DEFAULT_UNREACHABLE)",
    );
    let code = strings
        .replace_all(raw, |c: &regex::Captures<'_>| {
            c[0].chars()
                .map(|x| if x == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned();
    let raw_lines: Vec<_> = raw.split('\n').collect();
    let code_lines: Vec<_> = code.split('\n').collect();
    let mut out = Vec::new();
    for (i, line) in raw_lines.iter().enumerate() {
        let stripped = line.trim();
        if stripped.starts_with("//") || stripped.starts_with('*') || nolint.is_match(line) {
            continue;
        }
        let lo = i.saturating_sub(3);
        let ctx = raw_lines[lo..i].join("\n");
        let src = stripped.chars().take(110).collect::<String>();
        if hit.is_match(code_lines.get(i).copied().unwrap_or("")) {
            if !noreturn.is_match(&ctx) {
                out.push((i + 1, "ub", src));
            }
        } else if panic.is_match(code_lines.get(i).copied().unwrap_or(""))
            && origin.is_match(&format!("{line}\n{ctx}"))
        {
            out.push((i + 1, "hang", src));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaped_newline_inside_string_is_not_code() {
        assert!(assertion_facts("let s = \"escaped\\\nunreachable_unchecked()\";").is_empty());
    }
    #[test]
    fn positive_and_negative_fixtures() {
        let bad = "// PAGED_CODE diagnostic\nfn a() { unreachable_unchecked(); }\nfn b() { panic!(\"NT_ASSERT failed\"); }\n";
        assert_eq!(
            assertion_facts(bad)
                .iter()
                .map(|x| (x.0, x.1))
                .collect::<Vec<_>>(),
            vec![(2, "ub"), (3, "hang")]
        );
        let good = "fn a() { unreachable_unchecked(); } // nolint: assert-to-ub (noreturn)\n// NT_ASSERT\nfn b() { unreachable_unchecked(); }\n// BugCheck below\nfn c() { unreachable_unchecked(); }\nfn d() { panic!(\"unimplemented!()\"); }\n";
        // The original scanner still flags b: its NT_ASSERT origin signal is
        // relevant only to panic lowering, while UB suppression checks only
        // the noreturn context. The adjacent BugCheck text does suppress c.
        assert_eq!(
            assertion_facts(good)
                .iter()
                .map(|x| (x.0, x.1))
                .collect::<Vec<_>>(),
            vec![(3, "ub")]
        );
        assert!(skip_file("sdk/lib/rust/nt/target/generated.rs"));
        assert!(!skip_file("sdk/lib/rust/nt/live.rs"));
    }
}
