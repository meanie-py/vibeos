//! Address-sensitive initialized objects must not be moved by value.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct InitThenMove;
impl Rule for InitThenMove {
    fn name(&self) -> &'static str {
        "scan_init_then_move"
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
                    "{}:{line}: init-then-move {what}",
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
    init: Regex,
    init_ref: Regex,
    function: Regex,
    moved: Regex,
    tail: Regex,
    returned: Regex,
}
fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let re = |s| Regex::new(s).unwrap();
        Patterns {
            skip: re(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"),
            comments: re(r"(?s)//[^\n]*|/\*.*?\*/"),
            init: re(r"\b(\w*Initiali[sz]e\w*)\s*\(\s*(?:&\s*mut\s+)?(\w+)\.as_mut_ptr\(\)"),
            init_ref: re(r"\b(\w*Initiali[sz]e\w*)\s*\(\s*&\s*mut\s+(\w+)\b"),
            function: re(r"\bfn\s+\w+[^{;]*\{"),
            moved: re(r"(\w+)\.assume_init\(\)"),
            tail: re(r"(?:^|[;{}\n])\s*(\w+)\s*$"),
            returned: re(r"\breturn\s+(\w+)\s*;"),
        }
    })
}
#[cfg(test)]
fn code_only(raw: &str) -> String {
    let raw = raw.split("ORIGINAL C SOURCE").next().unwrap();
    patterns()
        .comments
        .replace_all(raw, |caps: &regex::Captures<'_>| {
            caps.get(0)
                .unwrap()
                .as_str()
                .chars()
                .map(|ch| if ch == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}
fn window(code: &str, start: usize, length: usize) -> &str {
    let tail = &code[start..];
    &tail[..tail
        .char_indices()
        .nth(length)
        .map_or(tail.len(), |(at, _)| at)]
}
fn returned_by_value(body: &str, var: &str) -> bool {
    patterns()
        .tail
        .captures(body.trim_end())
        .is_some_and(|c| c.get(1).unwrap().as_str() == var)
        || patterns()
            .returned
            .captures_iter(body)
            .any(|c| c.get(1).unwrap().as_str() == var)
}
#[cfg(test)]
fn scan_text(raw: &str) -> Vec<(usize, String)> {
    let code = code_only(raw);
    scan_code(&code)
}
fn scan_code(code: &str) -> Vec<(usize, String)> {
    let p = patterns();
    let mut findings = Vec::new();
    for init in p.init.captures_iter(&code) {
        let end = init.get(0).unwrap().end();
        let var = init.get(2).unwrap().as_str();
        let tail = window(&code, end, 6000);
        if let Some(moved) = p.moved.captures_iter(tail).find(|c| {
            c.get(1).unwrap().as_str() == var
                && tail[..c.get(0).unwrap().start()]
                    .chars()
                    .next_back()
                    .is_none_or(|ch| !(ch == '.' || ch == '_' || ch.is_alphanumeric()))
        }) {
            let line = code[..end + moved.get(0).unwrap().start()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            findings.push((
                line,
                format!(
                    "{}({var}.as_mut_ptr()) then {var}.assume_init()",
                    init.get(1).unwrap().as_str()
                ),
            ));
        }
    }
    for function in p.function.find_iter(&code) {
        let mut depth = 1usize;
        let mut end = code.len().saturating_sub(1);
        for (i, b) in code.bytes().enumerate().skip(function.end()) {
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
            }
            if depth == 0 {
                end = i;
                break;
            }
        }
        if end < function.end() {
            continue;
        }
        // The original scanner uses character offsets and drops the final character
        // of an unterminated body. Clamp a byte offset to the same UTF-8 boundary.
        while !code.is_char_boundary(end) {
            end -= 1;
        }
        let body = &code[function.end()..end];
        for init in p.init_ref.captures_iter(body) {
            let var = init.get(2).unwrap().as_str();
            if returned_by_value(body, var) {
                let line = code[..function.end() + init.get(0).unwrap().start()]
                    .bytes()
                    .filter(|b| *b == b'\n')
                    .count()
                    + 1;
                findings.push((
                    line,
                    format!(
                        "{}(&mut {var}..) then {var} returned by value",
                        init.get(1).unwrap().as_str()
                    ),
                ));
            }
        }
    }
    findings.sort();
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_maybeuninit_then_assume_init() {
        let code = "fn f() {\n    let mut E = MaybeUninit::uninit();\n    KeInitializeEvent(E.as_mut_ptr(), 0, 0);\n    let E = E.assume_init();\n}\n";
        assert_eq!(
            scan_text(code)
                .into_iter()
                .map(|(_, what)| what)
                .collect::<Vec<_>>(),
            vec!["KeInitializeEvent(E.as_mut_ptr()) then E.assume_init()"]
        );
    }
    #[test]
    fn test_constructor_returning_the_initialized_struct() {
        let code = "unsafe fn new_request() -> Req {\n    let mut req = Req { done: zeroed() };\n    KeInitializeEvent(&mut req.done, 0, 0);\n    req\n}\n";
        assert_eq!(
            scan_text(code)
                .into_iter()
                .map(|(_, what)| what)
                .collect::<Vec<_>>(),
            vec!["KeInitializeEvent(&mut req..) then req returned by value"]
        );
    }
    #[test]
    fn test_explicit_return_is_the_same_move() {
        let code = "fn g() -> L {\n    let mut head = L::default();\n    InitializeListHead(&mut head);\n    return head;\n}\n";
        assert_eq!(scan_text(code).len(), 1);
    }
    #[test]
    fn test_initialized_in_place_and_used_by_reference_is_fine() {
        let code = "fn h() -> NTSTATUS {\n    let mut req = new_request();\n    arm(&mut req);\n    KeInitializeEvent(&mut req.done, 0, 0);\n    wait(&req);\n    STATUS_SUCCESS\n}\n";
        assert!(scan_text(code).is_empty());
    }
    #[test]
    fn ignores_comments_original_source_and_unrelated_moves() {
        assert!(scan_text("fn f() { // KeInitializeEvent(E.as_mut_ptr()); E.assume_init();\nKeInitializeEvent(E.as_mut_ptr()); Other.assume_init(); obj.E.assume_init(); } ORIGINAL C SOURCE KeInitializeEvent(E.as_mut_ptr()); E.assume_init();").is_empty());
    }
    #[test]
    fn character_window_keeps_unicode_width() {
        let code = format!(
            "KeInitializeEvent(E.as_mut_ptr()); {} E.assume_init();",
            "é".repeat(4000)
        );
        assert_eq!(scan_text(&code).len(), 1);
    }
}
