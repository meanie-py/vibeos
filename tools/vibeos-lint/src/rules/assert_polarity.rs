//! A translated ASSERT whose guard is the asserted condition rather than its negation.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::LazyLock;

pub struct AssertPolarity;
impl Rule for AssertPolarity {
    fn name(&self) -> &'static str {
        "scan_assert_polarity"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.sdk_rust_files() {
            for (line, guard, msg) in scan_text(&file.text) {
                out.push(format!("{}:{line}: [ASSERT_POLARITY] `if {guard}` guards ASSERT({msg}) — the guard must be its negation", file.path.display()));
            }
        }
        Ok(out)
    }
}

struct Patterns {
    site: Regex,
    cast: Regex,
    tokens: Regex,
}
static PATTERNS: LazyLock<Patterns> = LazyLock::new(|| Patterns {
    site: crate::regexes::cached(
        r#"(?s)\bif\s+([^{};]+?)\s*\{\s*(?:[\w:]+::)?RtlAssert\(\s*b"((?:[^"\\]|\\.)*?)\\0""#,
    )
    .unwrap(),
    cast: crate::regexes::cached(r"\bas\s+[\w:]+").unwrap(),
    tokens: crate::regexes::cached(r"!=|==|<=|>=|&&|\|\||!|<|>|\w+").unwrap(),
});

fn tokens(expr: &str) -> Vec<String> {
    let expr = PATTERNS.cast.replace_all(expr, "");
    let expr = expr
        .replace("->", ".")
        .replace("(*", "(")
        .replace(".is_null()", " == NULL");
    PATTERNS
        .tokens
        .find_iter(&expr)
        .map(|m| m.as_str().to_owned())
        .collect()
}
fn scan_text(text: &str) -> Vec<(usize, String, String)> {
    if !text.contains("RtlAssert") {
        return Vec::new();
    }
    let mut out = Vec::new();
    for m in PATTERNS.site.captures_iter(text) {
        let guard = m.get(1).unwrap().as_str();
        let msg = m.get(2).unwrap().as_str();
        if tokens(guard) == tokens(msg) {
            let at = m.get(0).unwrap().start();
            let line = text.as_bytes()[..at]
                .iter()
                .filter(|&&b| b == b'\n')
                .count()
                + 1;
            out.push((
                line,
                guard.split_whitespace().collect::<Vec<_>>().join(" "),
                msg.to_owned(),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    use std::path::PathBuf;
    fn site(guard: &str, msg: &str) -> String {
        format!(
            "    if {guard} {{\n        RtlAssert(\n            b\"{msg}\\0\".as_ptr() as PVOID,\n"
        )
    }
    fn corpus(src: &str) -> Corpus {
        Corpus::new(
            PathBuf::from("/fixture"),
            vec![SourceFile::new("src/init.rs".into(), src.into())],
        )
    }

    #[test]
    fn pcix_inversion_control_is_reported() {
        assert_eq!(
            scan_text(&site("!NT_SUCCESS(Status)", "!NT_SUCCESS(Status)")),
            vec![(
                1,
                "!NT_SUCCESS(Status)".into(),
                "!NT_SUCCESS(Status)".into()
            )]
        );
        assert_eq!(AssertPolarity.run(&corpus(&site("!NT_SUCCESS(Status)","!NT_SUCCESS(Status)"))).unwrap(),vec!["src/init.rs:1: [ASSERT_POLARITY] `if !NT_SUCCESS(Status)` guards ASSERT(!NT_SUCCESS(Status)) — the guard must be its negation"]);
    }
    #[test]
    fn correct_lowerings_pass_and_matching_rust_spellings_are_flagged() {
        assert!(scan_text(&site("NT_SUCCESS(Status)", "!NT_SUCCESS(Status)")).is_empty());
        assert!(scan_text(&site("!(ResultLength > 0)", "ResultLength > 0")).is_empty());
        assert!(scan_text(&site(
            "(*Entry).VendorID == PCI_INVALID_VENDORID as u16",
            "Entry->VendorID != PCI_INVALID_VENDORID"
        ))
        .is_empty());
        assert!(scan_text(&site("p.is_null()", "p")).is_empty());
        assert_eq!(
            scan_text(&site("(*Fdo).Fake == TRUE as u8", "Fdo->Fake == TRUE")).len(),
            1
        );
        assert_eq!(scan_text(&site("x.is_null()", "x == NULL")).len(), 1);
    }
    #[test]
    fn dedicated_scanner_control_bad_and_good() {
        let bad="\nunsafe fn f(Status: NTSTATUS) {\n    if !NT_SUCCESS(Status) {\n        RtlAssert(\n            b\"!NT_SUCCESS(Status)\\0\".as_ptr() as PVOID,\n            b\"init.c\\0\".as_ptr() as PVOID,\n            670,\n            null_mut(),\n        );\n    }\n}\n";
        let good = bad.replace("if !NT_SUCCESS", "if NT_SUCCESS");
        assert_eq!(AssertPolarity.run(&corpus(bad)).unwrap().len(), 1);
        assert!(AssertPolarity.run(&corpus(&good)).unwrap().is_empty());
    }
}
