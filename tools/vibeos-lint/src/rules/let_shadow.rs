//! Suspicious nested underscore bindings that lose assignments to outer locals.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

pub struct LetShadow;
impl Rule for LetShadow {
    fn name(&self) -> &'static str {
        "scan_let_shadow"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for file in corpus.rust_files() {
            for (line, name) in scan_text(&file.text) {
                out.push(format!(
                    "{}:{line}: `let {name}` shadows `{}` in scope: a C assignment that is lost?",
                    file.resolved_path().display(),
                    &name[1..]
                ));
            }
        }
        Ok(out)
    }
}
struct Patterns {
    lexeme: Regex,
    function: Regex,
    binding: Regex,
    parameter: Regex,
    punctuation: Regex,
}
fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let re = |s| Regex::new(s).unwrap();
        Patterns {
            lexeme: re(r#"(?s)//[^\n]*|/\*.*?\*/|b?"(?:\\.|[^"\\])*"|'(?:\\.|[^\\'\n])'"#),
            function: re(r"\bfn\s+\w+\s*(?:<[^>{]*>)?\s*\("),
            binding: re(r"\blet\s+(?:mut\s+)?([A-Za-z_]\w*)"),
            parameter: re(r"(?:^|[(,])\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:"),
            punctuation: re(r"[{};]"),
        }
    })
}
fn blank(raw: &str) -> String {
    let mut bytes = raw.as_bytes().to_vec();
    for m in patterns().lexeme.find_iter(raw) {
        for b in &mut bytes[m.start()..m.end()] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
    String::from_utf8(bytes).unwrap()
}
fn params(code: &str, start: usize) -> (Vec<String>, usize) {
    let mut depth = 0usize;
    for (i, b) in code.bytes().enumerate().skip(start) {
        match b {
            b'(' | b'[' | b'<' => depth += 1,
            b')' | b']' | b'>' => {
                if depth == 0 {
                    let text = format!("({}", &code[start..i]);
                    let names = patterns()
                        .parameter
                        .captures_iter(&text)
                        .filter_map(|c| {
                            // Python's :(?!:) excludes paths in parameter types/patterns.
                            (!text[c.get(0).unwrap().end()..].starts_with(':'))
                                .then(|| c.get(1).unwrap().as_str().to_string())
                        })
                        .collect();
                    return (names, i + 1);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    (Vec::new(), code.len())
}
enum Event<'a> {
    Function,
    Binding(&'a str),
    Punctuation(u8),
}
fn scan_text(raw: &str) -> Vec<(usize, String)> {
    let code = blank(raw);
    let p = patterns();
    let mut events = Vec::new();
    for m in p.function.find_iter(&code) {
        events.push((m.end() - 1, Event::Function));
    }
    for c in p.binding.captures_iter(&code) {
        events.push((
            c.get(0).unwrap().start(),
            Event::Binding(c.get(1).unwrap().as_str()),
        ));
    }
    for m in p.punctuation.find_iter(&code) {
        events.push((m.start(), Event::Punctuation(m.as_str().as_bytes()[0])));
    }
    events.sort_by_key(|(pos, _)| *pos);
    let mut scopes: Vec<(HashSet<String>, bool)> = vec![(HashSet::new(), false)];
    let mut pending: Option<Vec<String>> = None;
    let mut skip_until = 0;
    let mut out = Vec::new();
    for (pos, event) in events {
        if pos < skip_until {
            continue;
        }
        match event {
            Event::Function => {
                let (names, end) = params(&code, pos + 1);
                pending = Some(names);
                skip_until = end;
            }
            Event::Punctuation(b'{') => {
                let is_fn = pending.is_some();
                scopes.push((
                    pending.take().unwrap_or_default().into_iter().collect(),
                    is_fn,
                ));
            }
            Event::Punctuation(b';') => pending = None,
            Event::Punctuation(b'}') => {
                if scopes.len() > 1 {
                    scopes.pop();
                }
            }
            Event::Binding(name) => {
                if name.starts_with('_')
                    && name.len() > 1
                    && !name.starts_with("__")
                    && !scopes.last().unwrap().1
                {
                    let outer = &name[1..];
                    for (names, is_fn) in scopes[..scopes.len() - 1].iter().rev() {
                        if names.contains(outer) {
                            let line = code[..pos].bytes().filter(|b| *b == b'\n').count() + 1;
                            if !raw
                                .lines()
                                .nth(line - 1)
                                .unwrap_or("")
                                .contains("nolint: let-shadow")
                            {
                                out.push((line, name.to_string()));
                            }
                            break;
                        }
                        if *is_fn {
                            break;
                        }
                    }
                }
                scopes.last_mut().unwrap().0.insert(name.to_string());
            }
            Event::Punctuation(_) => unreachable!(),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const SRC: &str = "\nunsafe fn enumerate(left: *mut V) -> u8 {\n    let DiskCount: u8 = 0;\n    let mut c: u32;\n    let mut _c: u32;                       // right: the C's own second variable\n    while more() {\n        let _DiskCount = DiskCount + 1;    // finding: C `DiskCount++`\n        let _left = &mut tmp;              // finding: a parameter re-pointed\n        let _hr = f(); // nolint: let-shadow (test)\n        let s = \"let _DiskCount = 1;\";     // right: inside a string\n        // let _DiskCount = 2;             right: a comment\n    }\n    DiskCount\n}\nunsafe fn other() {\n    if x {\n        let _DiskCount = 3;                // right: no DiskCount in THIS function\n    }\n}\n";

    #[test]
    fn only_nested_shadows_of_live_bindings() {
        assert_eq!(
            scan_text(SRC),
            vec![(7, "_DiskCount".into()), (8, "_left".into())]
        );
    }
    #[test]
    fn lint_output_and_rule_identity() {
        let corpus = Corpus {
            root: std::path::PathBuf::new(),
            files: vec![crate::SourceFile::new(
                "sdk/lib/rust/example.rs".into(),
                SRC.into(),
            )],
        };
        assert_eq!(LetShadow.name(), "scan_let_shadow");
        assert_eq!(LetShadow.run(&corpus).unwrap(), vec![
            "sdk/lib/rust/example.rs:7: `let _DiskCount` shadows `DiskCount` in scope: a C assignment that is lost?",
            "sdk/lib/rust/example.rs:8: `let _left` shadows `left` in scope: a C assignment that is lost?",
        ]);
    }
    #[test]
    fn prototypes_nested_functions_and_same_scope_are_clean() {
        let code = "fn declared(x: X); fn f() { let x = 1; let _x = 2; fn nested() { if yes { let _x = 3; } } }";
        assert!(scan_text(code).is_empty());
    }
    #[test]
    fn nested_parameter_types_and_exemptions() {
        let code = "fn f<T>(mut x: F<(A, B)>, y: [u8; 4]) { if yes { let __x = 1; let _y = 2; } }";
        assert_eq!(scan_text(code), vec![(1, "_y".into())]);
    }
}
