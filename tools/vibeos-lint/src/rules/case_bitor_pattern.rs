//! Detect a C `case A | B:` translated as Rust's alternative `A | B` pattern.
use crate::{lexical, regexes::cached, Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::OnceLock;

pub struct CaseBitorPattern;
impl Rule for CaseBitorPattern {
    fn name(&self) -> &'static str {
        "scan_case_bitor_pattern"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        const SETS: &[SourceSet] = &[
            SourceSet::tracked("case-repo", &["c", "cpp", "h"], &[]),
            SourceSet::tree("case-inputs", &["sdk/lib/rust"], &["c", "cpp"], &[]).with_inputs(),
        ];
        SETS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut labels = BTreeSet::new();
        for file in corpus.sources("case-repo") {
            if has_c_case_prefix(&file.text) {
                labels.extend(c_bitor_labels(&file.text));
            }
        }
        for file in corpus.sources("case-inputs") {
            labels.extend(c_bitor_labels(&file.text));
        }
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (_line, alts) in scan_text(&file.text, &labels) {
                out.push(format!(
                    "{}: or-pattern `{alts}` is a C `case` that ORs them into one value",
                    if file.resolved_path().is_absolute() {
                        &file.path
                    } else {
                        file.resolved_path()
                    }
                    .display()
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    comments: Regex,
    c_case_prefix: Regex,
    c_case: Regex,
    arm: Regex,
    matches: Regex,
}
fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        let ident = r"[A-Za-z_]\w*";
        let path = format!(r"{ident}(?:::{ident})*");
        Patterns {
            comments: cached(r"(?s)//[^\n]*|/\*.*?\*/").unwrap(),
            c_case_prefix: cached(r"\bcase\s+\(?\s*\w+\s*\|").unwrap(),
            c_case: cached(&format!(r"\bcase\s+\(?\s*({ident}(?:\s*\|\s*{ident})+)\s*\)?\s*:")).unwrap(),
            arm: cached(&format!(r"(?m)(?:^|[{{,])[ \t]*({path}(?:\s*\|\s*{path})+)\s*(?:=>|\bif\b)")).unwrap(),
            matches: cached(&format!(r"\bmatches!\s*\([^,()]*(?:\([^()]*\)[^,()]*)*,\s*({path}(?:\s*\|\s*{path})+)\s*[,)]")).unwrap(),
        }
    })
}
fn blank(m: &regex::Match<'_>) -> String {
    m.as_str()
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect()
}
fn last_segment(p: &str) -> String {
    p.trim().rsplit("::").next().unwrap_or("").to_owned()
}
fn idents(expr: &str) -> BTreeSet<String> {
    expr.split('|').map(last_segment).collect()
}
fn has_c_case_prefix(text: &str) -> bool {
    text.lines()
        .any(|line| patterns().c_case_prefix.is_match(line))
}
fn c_bitor_labels(text: &str) -> BTreeSet<BTreeSet<String>> {
    let code = patterns()
        .comments
        .replace_all(text, |caps: &regex::Captures| blank(&caps.get(0).unwrap()));
    patterns()
        .c_case
        .captures_iter(&code)
        .filter_map(|cap| {
            let m = cap.get(0)?;
            if code[m.end()..].starts_with(':') {
                return None;
            }
            Some(idents(cap.get(1)?.as_str()))
        })
        .collect()
}
fn scan_text(text: &str, labels: &BTreeSet<BTreeSet<String>>) -> Vec<(usize, String)> {
    if !patterns()
        .arm
        .captures_iter(text)
        .any(|c| labels.contains(&idents(c.get(1).unwrap().as_str())))
        && !patterns()
            .matches
            .captures_iter(text)
            .any(|c| labels.contains(&idents(c.get(1).unwrap().as_str())))
    {
        return Vec::new();
    }
    let code = lexical::strip_noise(text);
    let lines: Vec<&str> = text.lines().collect();
    let mut hits = BTreeSet::new();
    for regex in [&patterns().arm, &patterns().matches] {
        for cap in regex.captures_iter(&code) {
            let Some(group) = cap.get(1) else {
                continue;
            };
            let alts = idents(group.as_str());
            if alts.len() < 2 || !labels.contains(&alts) {
                continue;
            }
            let line = code[..group.start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            if lines
                .get(line - 1)
                .is_some_and(|s| s.contains("nolint: case-bitor-pattern"))
            {
                continue;
            }
            hits.insert((line, alts.into_iter().collect::<Vec<_>>().join(" | ")));
        }
    }
    hits.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    const C: &str = r#"
switch (action) {
case TVGN_CARET|TVSI_NOSINGLEEXPAND:   /* one value: the OR */
case (A_FLAG | B_FLAG | C_FLAG):
    break;
case PLAIN:
case OTHER:
    break;
}
"#;
    #[test]
    fn c_labels_are_or_expressions_only() {
        let labels = c_bitor_labels(C);
        assert!(labels.contains(
            &["TVGN_CARET", "TVSI_NOSINGLEEXPAND"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        ));
        assert!(labels.contains(
            &["A_FLAG", "B_FLAG", "C_FLAG"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        ));
        assert!(!labels.contains(&["PLAIN", "OTHER"].into_iter().map(str::to_owned).collect()));
    }
    #[test]
    fn or_patterns_and_matches_are_flagged() {
        let rust = "\nmatch action {\n TVGN_CARET | TVSI_NOSINGLEEXPAND => {}\n crate::A_FLAG\n | B_FLAG\n | C_FLAG => {}\n _ => {}\n}\nlet b = matches!(action, TVGN_CARET | TVSI_NOSINGLEEXPAND);\n";
        assert_eq!(
            scan_text(rust, &c_bitor_labels(C)),
            vec![
                (3, "TVGN_CARET | TVSI_NOSINGLEEXPAND".into()),
                (4, "A_FLAG | B_FLAG | C_FLAG".into()),
                (9, "TVGN_CARET | TVSI_NOSINGLEEXPAND".into())
            ]
        );
    }
    #[test]
    fn guards_comments_nolint_and_unmatched_pairs_are_clean() {
        let rust = "\nmatch action {\n x if x == TVGN_CARET | TVSI_NOSINGLEEXPAND => {}\n PLAIN | OTHER => {}\n TVGN_CARET | TVSI_NOSINGLEEXPAND => {} // nolint: case-bitor-pattern (C lists both)\n // TVGN_CARET | TVSI_NOSINGLEEXPAND => {}\n _ => {}\n}\nlet m = TVGN_CARET | TVSI_NOSINGLEEXPAND;\n";
        assert!(scan_text(rust, &c_bitor_labels(C)).is_empty());
    }
    #[test]
    fn scanner_control_fixture_bad_and_good() {
        let c = "switch (action) {\ncase TVGN_CARET|TVSI_NOSINGLEEXPAND:\n    bSingleExpand = FALSE;\n    break;\n}\n";
        let bad = "unsafe fn select(action: u32) {\n    match action {\n        TVGN_CARET | TVSI_NOSINGLEEXPAND => { single = 0; }\n        _ => {}\n    }\n}\n";
        let good = "unsafe fn select(action: u32) {\n    match action {\n        x if x == TVGN_CARET | TVSI_NOSINGLEEXPAND => { single = 0; }\n        _ => {}\n    }\n}\n";
        assert_eq!(
            scan_text(bad, &c_bitor_labels(c)),
            vec![(3, "TVGN_CARET | TVSI_NOSINGLEEXPAND".into())]
        );
        assert!(scan_text(good, &c_bitor_labels(c)).is_empty());
    }
    #[test]
    fn comment_colon_and_string_noise_do_not_add_c_labels() {
        // Python c_bitor_labels masks comments only; it intentionally sees this string label.
        assert!(c_bitor_labels("// case X|Y:\nconst char *s=\"case P|Q:\";")
            .contains(&["P", "Q"].into_iter().map(str::to_owned).collect()));
        assert!(has_c_case_prefix("/* case A|B: */ case C;"));
        assert!(c_bitor_labels("case (A | B): break;")
            .contains(&["A", "B"].into_iter().map(str::to_owned).collect()));
        assert!(scan_text("let s = \"A | B =>\";\n", &c_bitor_labels(C)).is_empty());
    }
}
