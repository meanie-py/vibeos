//! Detect executable Rust text swallowed by a trailing line comment.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct CommentSwallowedCode;
impl Rule for CommentSwallowedCode {
    fn name(&self) -> &'static str {
        "scan_comment_swallowed_code"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            if patterns().skip.is_match(&file.path.to_string_lossy()) {
                continue;
            }
            for (line, stmt) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: comment-swallowed {stmt}",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    strings: Regex,
    swallowed: Regex,
    skip: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        strings: crate::regexes::cached(r#"b?"(?:\\.|[^"\\])*""#).unwrap(),
        swallowed: crate::regexes::cached(r"\b(?:removed|stripped|elided|dropped)\b\s+(?P<stmt>(?:return\b[^;]*|[A-Za-z_][\w.]*\s*=\s*[^=;][^;]*|[A-Za-z_][\w:]*\([^;]*\))\s*;)\s*$").unwrap(),
        skip: crate::regexes::cached(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)").unwrap(),
    })
}
fn scan_text(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let code = patterns().strings.replace_all(line, "\"\"");
        let Some(at) = code.find("//") else {
            continue;
        };
        if at == 0 || code[..at].trim().is_empty() {
            continue;
        }
        if let Some(m) = patterns().swallowed.captures(&code[at + 2..]) {
            let stmt = m
                .name("stmt")
                .unwrap()
                .as_str()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            out.push((i + 1, stmt));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const BAD: &str = "pub unsafe fn load(r: u32) -> i32 {\n    match r { 1 => { /* Fail */ // DPRINT removed return STATUS_NOT_REGISTRY_FILE;\n    } _ => {} }\n    0\n}\n";
    const GOOD: &str = "pub unsafe fn load(r: u32) -> i32 {\n    match r {\n        1 => {\n            return STATUS_NOT_REGISTRY_FILE; // DPRINT removed: the C printed here\n        }\n        _ => {}\n    }\n    // DPRINT removed return X; (a whole-line comment is prose, never code)\n    0\n}\n";
    #[test]
    fn scanner_control_bad_and_good_fixtures() {
        assert_eq!(
            scan_text(BAD),
            vec![(2, "return STATUS_NOT_REGISTRY_FILE;".into())]
        );
        assert!(scan_text(GOOD).is_empty());
    }
    #[test]
    fn assignment_and_call_shapes_fire_but_need_code_before_comment() {
        assert_eq!(scan_text("x(); // DPRINT stripped foo.bar = value;\nx(); // elided do_thing(1);\n// removed return X;\n"), vec![(1,"foo.bar = value;".into()),(2,"do_thing(1);".into())]);
    }
}
