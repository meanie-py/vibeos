//! Find an override that re-enters itself through its own generated base dispatcher.
use crate::{regexes::cached, Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::sync::OnceLock;

pub struct OverrideSelfDispatch;
const INPUTS: &[SourceSet] = &[SourceSet::tree(
    "override-self-dispatch-rust",
    &["sdk/lib/rust"],
    &["rs"],
    &[],
)
.with_pruning(&["target", ".responses"])
.with_inputs()];

impl Rule for OverrideSelfDispatch {
    fn name(&self) -> &'static str {
        "scan_override_self_dispatch"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.sources("override-self-dispatch-rust") {
            if skip_re().is_match(&file.path.to_string_lossy()) || !file.text.contains("__body") {
                continue;
            }
            for (line, _) in scan_file(file) {
                out.push(format!(
                    "{}:{line}: override-self-dispatch",
                    file.path.display()
                ));
            }
        }
        Ok(out)
    }
}

fn skip_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"(/target/|/\.responses/)").unwrap())
}
fn body_fn_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"\bfn (\w+)__body\s*(?:<[^>]*>)?\s*\(").unwrap())
}
fn nolint_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"//\s*nolint(?::\s*override-self-dispatch\b|\s*$)").unwrap())
}
fn word_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| cached(r"^\w$").unwrap())
}
fn body_span(text: &str, after: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let (mut depth, mut i) = (1i32, after);
    while i < bytes.len() && depth != 0 {
        depth += (bytes[i] == b'(') as i32 - (bytes[i] == b')') as i32;
        i += 1;
    }
    if depth != 0 {
        return None;
    }
    let brace = bytes[i..].iter().position(|b| *b == b'{')? + i;
    let (mut depth, mut j) = (0i32, brace);
    while j < bytes.len() {
        depth += (bytes[j] == b'{') as i32 - (bytes[j] == b'}') as i32;
        if depth == 0 {
            return Some((brace, j));
        }
        j += 1;
    }
    None
}
fn call_re(name: &str) -> Regex {
    let name = regex::escape(name);
    cached(&format!(
        r"(\(\s*\*\s*this_\s*\)|self|this_)\s*\.(?:__base\d+\.)+{name}\s*\("
    ))
    .unwrap()
}
fn preceding_char(text: &str, at: usize) -> Option<char> {
    text[..at].chars().next_back()
}
fn scan_file(file: &SourceFile) -> Vec<(usize, String)> {
    let text = &file.text;
    let mut findings = Vec::new();
    for f in body_fn_re().captures_iter(text) {
        let Some(name) = f.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let Some(span) = body_span(text, f.get(0).unwrap().end()) else {
            continue;
        };
        let call = call_re(name);
        for c in call.captures_iter(&text[span.0..span.1]) {
            let whole = c.get(0).unwrap();
            let owner = c.get(1).unwrap();
            let absolute_start = span.0 + whole.start();
            if owner.as_str() != "(*this_)"
                && preceding_char(text, absolute_start)
                    .is_some_and(|p| p == '.' || p == ')' || word_re().is_match(&p.to_string()))
            {
                continue;
            }
            let line_start = text[..absolute_start].rfind('\n').map_or(0, |n| n + 1);
            let line_end = text[absolute_start..]
                .find('\n')
                .map_or(text.len(), |n| absolute_start + n);
            let line_text = &text[line_start..line_end];
            if nolint_re().is_match(line_text) {
                continue;
            }
            let line = text[..absolute_start]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            findings.push((line, line_text.trim().to_owned()));
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    fn file(text: &str) -> SourceFile {
        SourceFile::new("sdk/lib/rust/mspaint-rust/src/tools.rs".into(), text.into())
    }
    #[test]
    fn original_bad_fixture_finds_derived_dispatcher() {
        let src = "\nimpl SmoothDrawTool {\n    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {\n        let this_: *mut Self = self;\n        (*this_).m_bDrawing = 0;\n        (*this_).__base0.OnEndDraw(bCancel);\n    }\n}\n";
        assert_eq!(
            scan_file(&file(src))
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>(),
            [6]
        );
    }
    #[test]
    fn original_good_fixture_allows_body_and_other_object_calls() {
        let src = "impl SmoothDrawTool {\n pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {\n (*this_).__base0.OnEndDraw__body(bCancel);\n (*(*this_).m_pChild).__base0.OnEndDraw(bCancel);\n }\n}";
        assert!(scan_file(&file(src)).is_empty());
    }
    #[test]
    fn supports_self_and_this_owners_and_multiple_base_layers() {
        let src = "fn M__body(&mut self) { self.__base0.__base1.M(); }\nfn N__body(&mut self) { this_.__base2.N(); }";
        assert_eq!(
            scan_file(&file(src))
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }
    #[test]
    fn nolint_is_on_call_line_and_nearby_objects_are_not_self() {
        let src = "fn M__body(&mut self) {\n self.__base0.M(); // nolint: override-self-dispatch\n (*(*this_).child).__base0.M();\n }";
        assert!(scan_file(&file(src)).is_empty());
        let src = "fn M__body(&mut self) {\n self.__base0.M(); // other reason\n }";
        assert_eq!(scan_file(&file(src)).len(), 1);
    }
    #[test]
    fn qualified_or_embedded_owners_are_rejected_like_python_lookbehind() {
        let src = "fn M__body(&mut self) {\n object.self.__base0.M();\n (*object).self.__base0.M();\n xthis_.__base0.M();\n self.__base0.M();\n }";
        let hits = scan_file(&file(src));
        assert_eq!(hits.len(), 1);
        assert!(hits[0].1.contains("self.__base0.M"));
    }
    #[test]
    fn nested_bodies_braces_and_utf8_preserve_line_numbers() {
        let src =
            "fn A__body(&mut self) { let msg = \"{ λ }\"; if true {\n self.__base0.A();\n } }";
        assert_eq!(scan_file(&file(src))[0].0, 2);
    }
}
