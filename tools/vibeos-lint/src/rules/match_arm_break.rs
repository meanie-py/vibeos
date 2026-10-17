//! Detect a C-style break at the tail of a match arm inside a loop.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct MatchArmBreak;
impl Rule for MatchArmBreak {
    fn name(&self) -> &'static str {
        "scan_match_arm_break"
    }
    fn needs_sdk_rust(&self) -> bool {
        true
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.sdk_rust_files() {
            let path = file.path.to_string_lossy();
            for line in scan_text(&file.text) {
                out.push(format!("{path}:{line}: MATCH_ARM_BREAK"));
            }
        }
        Ok(out)
    }
}
fn noise() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"//[^\n]*|/\*.*?\*/|b?"(?:\\.|[^"\\])*"|b?'(?:\\.|[^'\\\n])'"#).unwrap()
    })
}
fn tokens() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{|\}|\bbreak\s*;").unwrap())
}
fn loop_head() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:\bwhile\b[^{;]*|\bfor\b[^{;]*)$").unwrap())
}
fn tail_chars(text: &str, count: usize) -> &str {
    let Some((start, _)) = text.char_indices().rev().nth(count.saturating_sub(1)) else {
        return text;
    };
    &text[start..]
}
fn scan_text(text: &str) -> Vec<usize> {
    if !text.contains("break") || !text.contains("=>") {
        return vec![];
    }
    let clean = noise()
        .replace_all(text, |c: &regex::Captures<'_>| {
            c[0].chars()
                .map(|x| if x == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned();
    let mut stack: Vec<(&str, usize)> = Vec::new();
    let mut found = Vec::new();
    for m in tokens().find_iter(&clean) {
        match m.as_str() {
            "{" => {
                let before = clean[..m.start()].trim_end();
                let kind = if before.ends_with("=>") {
                    "arm"
                } else if before.ends_with("loop") {
                    "loop-kw"
                } else if loop_head().is_match(tail_chars(before, 200)) {
                    "loop"
                } else {
                    "other"
                };
                stack.push((kind, m.start()));
            }
            "}" => {
                stack.pop();
            }
            _ if stack.last().is_some_and(|(k, _)| *k == "arm") => {
                let (_, arm_open) = *stack.last().unwrap();
                let after = clean[m.end()..].trim_start();
                if !after.starts_with('}') || clean[arm_open + 1..m.start()].trim().is_empty() {
                    continue;
                }
                if !stack[..stack.len() - 1]
                    .iter()
                    .any(|(k, _)| *k == "loop" || *k == "loop-kw")
                {
                    continue;
                }
                if stack.len() >= 3 && stack[stack.len() - 3].0 == "loop-kw" {
                    let body = clean[stack[stack.len() - 3].1 + 1..].trim_start();
                    if body.starts_with("match") {
                        continue;
                    }
                }
                found.push(clean[..m.start()].bytes().filter(|b| *b == b'\n').count() + 1);
            }
            _ => {}
        }
    }
    found
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mf_session_shape_fires() {
        let text = "fn f() {\n    while i < n {\n        match kind {\n            SOURCE => {\n                check(node);\n                break;\n            }\n            _ => {}\n        }\n        release(node);\n    }\n}\n";
        assert_eq!(scan_text(text), vec![6]);
    }

    #[test]
    fn meant_breaks_are_silent() {
        let text = "fn f() {\n    loop {\n        match x {\n            A => { a(); break; }\n            _ => break,\n        }\n    }\n    for c in s {\n        match c {\n            0 => break,\n            _ => { for d in t { g(d); break; } }\n        }\n    }\n    match y { B => { h(); \"break;\"; } _ => {} }\n}\n";
        assert!(scan_text(text).is_empty());
    }

    #[test]
    fn unicode_before_a_loop_header_keeps_character_window_boundaries() {
        let text = format!(
            "while {} x {{ match y {{ A => {{ f(); break; }} _ => {{}} }} }}",
            "→".repeat(100)
        );
        assert_eq!(scan_text(&text), vec![1]);
    }
}
