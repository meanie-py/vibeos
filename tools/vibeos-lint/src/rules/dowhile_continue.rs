//! Continue hazards in translated do/while, for-step, and loop macros.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

pub struct DowhileContinue;
impl Rule for DowhileContinue {
    fn name(&self) -> &'static str {
        "scan_dowhile_continue"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let mut steps = Vec::new();
        let mut macros = Vec::new();
        let mut loops = Vec::new();
        for file in corpus.rust_files() {
            let path = file.resolved_path().to_string_lossy();
            let code = code_only(&file.text);
            for (_, name, var, _) in scan_steps(&file.text, &code) {
                steps.push(format!(
                    "{path}: fn {name}: `continue` skips the while loop's trailing `{var}` update"
                ));
            }
            for (_, name) in scan_macros(&code) {
                macros.push(format!("{path}: macro {name}: loop advances after the spliced body; `continue` skips it"));
            }
            for (_, name, _) in scan_loops(&file.text, &code) {
                loops.push(format!(
                    "{path}: fn {name}: loop ends in a do-while condition that `continue` skips"
                ));
            }
        }
        steps.extend(macros);
        steps.extend(loops);
        Ok(steps)
    }
}

struct Patterns {
    lexeme: Regex,
    loops: Regex,
    nested: Regex,
    trailing_break: Regex,
    function: Regex,
    continues: Regex,
    whiles: Regex,
    step: Regex,
    macros: Regex,
    macro_loop: Regex,
    splice: Regex,
}
fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let re = |s| Regex::new(s).unwrap();
        Patterns {
            lexeme: re(r#"(?s)//[^\n]*|/\*.*?\*/|b?'(?:\\.|[^'\\])'|b?"(?:\\.|[^"\\])*""#),
            loops: re(r"(?:'(\w+)\s*:\s*)?\bloop\s*\{"),
            nested: re(r"\b(?:loop|while|for)\b"),
            trailing_break: re(r"\bif\b[^{};]*\{\s*break\s*;\s*\}\s*$"),
            function: re(r"\bfn\s+(\w+)"),
            continues: re(r"\bcontinue\b(?:\s*'(\w+))?\s*;"),
            whiles: re(r"(?:'(\w+)\s*:\s*)?\bwhile\b[^{;]*\{"),
            step: re(r"(?:^|[;{}])\s*([\w.*()\[\]]+?)\s*(?:=|\+=|-=)\s*[^;=][^;]*;\s*$"),
            macros: re(r"\bmacro_rules!\s*(\w+)\s*\{"),
            macro_loop: re(r"\b(?:while\b[^{;]*|loop\s*)\{"),
            splice: re(r"(?m)^\s*\$(\w+)\s*;?\s*$"),
        }
    })
}
fn code_only(raw: &str) -> String {
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
fn match_brace(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, b) in text.bytes().enumerate().skip(open) {
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn own_continues(body: &str, label: Option<&str>) -> Vec<usize> {
    let p = patterns();
    let mut masked = body.as_bytes().to_vec();
    for n in p.nested.find_iter(body) {
        if let Some(k) = body[n.end()..].find('{').map(|i| i + n.end()) {
            if let Some(e) = match_brace(body, k) {
                masked[k..=e].fill(b' ');
            }
        }
    }
    let flat = String::from_utf8(masked).unwrap();
    let mut out: Vec<_> = p
        .continues
        .captures_iter(&flat)
        .filter_map(|c| {
            let target = c.get(1).map_or("", |m| m.as_str());
            (target.is_empty() || Some(target) == label).then(|| c.get(0).unwrap().start())
        })
        .collect();
    if let Some(label) = label {
        // Python separately finds labelled continues inside the masked nested loops.
        for capture in p.continues.captures_iter(body) {
            let m = capture.get(0).unwrap();
            if capture
                .get(1)
                .is_some_and(|target| target.as_str() == label)
                && !flat[m.start()..].starts_with("continue")
            {
                out.push(m.start());
            }
        }
    }
    out.sort_unstable();
    out
}
fn location(raw: &str, code: &str, offset: usize) -> Option<(usize, String)> {
    let line = code[..offset].bytes().filter(|b| *b == b'\n').count() + 1;
    if raw
        .lines()
        .nth(line - 1)
        .unwrap_or("")
        .contains("nolint: dowhile-continue")
    {
        return None;
    }
    let function = patterns()
        .function
        .captures_iter(&code[..offset])
        .last()
        .map_or("?", |c| c.get(1).unwrap().as_str())
        .to_string();
    Some((line, function))
}
fn scan_loops(raw: &str, code: &str) -> Vec<(usize, String, usize)> {
    let mut out = Vec::new();
    for m in patterns().loops.captures_iter(code) {
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let Some(close) = match_brace(code, open) else {
            continue;
        };
        let body = &code[open + 1..close];
        if !patterns().trailing_break.is_match(body.trim_end()) {
            continue;
        }
        let count = own_continues(body, m.get(1).map(|v| v.as_str())).len();
        if count > 0 {
            if let Some((line, name)) = location(raw, code, whole.start()) {
                out.push((line, name, count));
            }
        }
    }
    out
}
fn block_end(body: &str, offset: usize) -> usize {
    let mut stack = Vec::new();
    for (i, b) in body.bytes().enumerate().take(offset) {
        if b == b'{' {
            stack.push(i);
        } else if b == b'}' {
            stack.pop();
        }
    }
    stack
        .last()
        .and_then(|open| match_brace(body, *open))
        .unwrap_or(body.len())
}
fn scan_steps(raw: &str, code: &str) -> Vec<(usize, String, String, usize)> {
    let mut out = Vec::new();
    for m in patterns().whiles.captures_iter(code) {
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let Some(close) = match_brace(code, open) else {
            continue;
        };
        let body = &code[open + 1..close];
        let Some(step) = patterns().step.captures(body.trim_end()) else {
            continue;
        };
        let variable = step.get(1).unwrap();
        if variable.as_str() == "_" {
            continue;
        }
        // Match the literal variable without recompiling a regex for every loop.
        // Check Python's lookbehind and assignment lookahead explicitly.
        let updates: Vec<_> = body
            .match_indices(variable.as_str())
            .filter_map(|(at, value)| {
                if at >= variable.start()
                    || body[..at]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c == '.' || c == '_' || c.is_alphanumeric())
                {
                    return None;
                }
                let rest = body[at + value.len()..].trim_start();
                let assignment = rest.starts_with("+=")
                    || rest.starts_with("-=")
                    || (rest.starts_with('=') && !rest.starts_with("=="));
                assignment.then(|| (at, block_end(body, at)))
            })
            .collect();
        let skipping = own_continues(body, m.get(1).map(|v| v.as_str()))
            .into_iter()
            .filter(|c| !updates.iter().any(|(u, end)| *u < *c && *c < *end))
            .count();
        if skipping > 0 {
            if let Some((line, name)) = location(raw, code, whole.start()) {
                out.push((line, name, variable.as_str().to_string(), skipping));
            }
        }
    }
    out
}
fn scan_macros(code: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for m in patterns().macros.captures_iter(code) {
        let whole = m.get(0).unwrap();
        let Some(end) = match_brace(code, whole.end() - 1) else {
            continue;
        };
        let macro_body = &code[whole.end()..end];
        for lm in patterns().macro_loop.find_iter(macro_body) {
            let open = lm.end() - 1;
            let Some(close) = match_brace(macro_body, open) else {
                continue;
            };
            let body = &macro_body[open + 1..close];
            let mut flat = body.as_bytes().to_vec();
            let mut i = 0;
            while i < body.len() {
                if body.as_bytes()[i] == b'{' {
                    let Some(end) = match_brace(body, i) else {
                        break;
                    };
                    flat[i + 1..end].fill(b' ');
                    i = end + 1;
                } else {
                    i += 1;
                }
            }
            let flat = String::from_utf8(flat).unwrap();
            if patterns()
                .splice
                .find_iter(&flat)
                .last()
                .is_some_and(|s| !flat[s.end()..].trim().is_empty())
            {
                out.push((
                    code[..whole.start()]
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count()
                        + 1,
                    m.get(1).unwrap().as_str().to_string(),
                ));
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const BUGGY: &str = "\nfn register(ctx: *mut Ctx) {\n    loop {\n        if read(ctx) == 0 {\n            continue; // skips SetupFindNextLine below\n        }\n        for i in 0..4 {\n            if i == 2 { continue; } // the for's own continue: not a finding\n        }\n        if next_line(ctx) == 0 {\n            break;\n        }\n    }\n}\n";
    const GUARDED_AND_UNRELATED: &str = "\nfn a() {\n    loop {\n        if x() { break; }\n        work();\n    }\n}\nfn b() {\n    loop { // nolint: dowhile-continue (C is for(;;))\n        if y() { continue; }\n        if done() { break; }\n    }\n}\nfn c() {\n    'outer: loop {\n        for i in 0..3 {\n            if i == 1 { continue 'outer; }\n        }\n        if done() { break; }\n    }\n}\n";
    const MACRO_BUGGY: &str = "\nmacro_rules! LIST_FOR_EACH_ENTRY {\n    ($elem:ident, $head:expr, $ty:ty, $field:ident, $body:block) => {{\n        $elem = first($head);\n        while not_head($elem) {\n            $body\n            $elem = next($elem);\n        }\n    }};\n}\n";
    const MACRO_FIXED: &str = "\nmacro_rules! LIST_FOR_EACH_ENTRY {\n    ($elem:ident, $head:expr, $body:block) => {{\n        let mut __advance = false;\n        loop {\n            if __advance {\n                $elem = next($elem);\n            }\n            __advance = true;\n            if is_head($elem) {\n                break;\n            }\n            $body\n        }\n    }};\n}\nmacro_rules! LIST_FOR_EACH_ENTRY_SAFE {\n    ($elem:ident, $head:expr, $body:block) => {{\n        while cur != head {\n            let next = (*cur).next;\n            cur = next;\n            $body\n        }\n    }};\n}\nmacro_rules! NOT_A_LOOP {\n    ($body:block) => {{\n        $body\n        done();\n    }};\n}\n";
    const STEP_BUGGY: &str = "\nfn validate(hive: *mut Hive, len: u32) {\n    let mut i = 0;\n    while i < len {\n        if bin(hive, i) == 0 {\n            continue;\n        }\n        if odd(i) {\n            i += 2;\n        } else if skip(i) {\n            continue; // the update above is in a sibling arm: does not cover this\n        }\n        i += 1;\n    }\n}\n";
    const STEP_COVERED: &str = "\nfn a(len: u32) {\n    let mut i = 0;\n    while i < len {\n        if bad(i) {\n            i += 1;\n            continue;\n        }\n        i += 1;\n    }\n}\nfn b(head: *mut Entry) {\n    let mut e = (*head).next;\n    while e != head {\n        let cur = e;\n        e = (*e).next; // advanced at the top: every continue below is covered\n        if skip(cur) {\n            continue;\n        }\n        work(cur);\n        e = (*cur).next;\n    }\n}\nfn c(len: u32) {\n    let mut i = 0;\n    while i < len {\n        for j in 0..4 {\n            if j == 1 { continue; } // the for's own\n        }\n        i += 1;\n    }\n}\n";
    const QUOTE_LITERAL: &str = "fn f() {\n    while !list.is_null() {\n        if v.is_null() {\n            list = (*list).next;\n            continue;\n        }\n        if q != 0 {\n            *p = '\"' as WCHAR;\n            p = p.add(1);\n        }\n        list = (*list).next;\n    }\n}\n";

    #[test]
    fn flags_the_skipped_condition() {
        assert_eq!(
            scan_loops(BUGGY, &code_only(BUGGY))
                .into_iter()
                .map(|(_, f, n)| (f, n))
                .collect::<Vec<_>>(),
            vec![("register".into(), 1)]
        );
    }
    #[test]
    fn trailing_break_without_continue_nolint_and_labelled_continue() {
        assert_eq!(
            scan_loops(GUARDED_AND_UNRELATED, &code_only(GUARDED_AND_UNRELATED))
                .into_iter()
                .map(|(_, f, n)| (f, n))
                .collect::<Vec<_>>(),
            vec![("c".into(), 1)]
        );
    }
    #[test]
    fn macro_advancing_after_its_body_is_flagged() {
        assert_eq!(
            scan_macros(&code_only(MACRO_BUGGY))
                .into_iter()
                .map(|(_, n)| n)
                .collect::<Vec<_>>(),
            vec!["LIST_FOR_EACH_ENTRY"]
        );
    }
    #[test]
    fn macro_advancing_first_or_outside_a_loop_is_clean() {
        assert!(scan_macros(&code_only(MACRO_FIXED)).is_empty());
    }
    #[test]
    fn a_continue_past_the_trailing_step_is_flagged() {
        assert_eq!(
            scan_steps(STEP_BUGGY, &code_only(STEP_BUGGY))
                .into_iter()
                .map(|(_, f, v, n)| (f, v, n))
                .collect::<Vec<_>>(),
            vec![("validate".into(), "i".into(), 2)]
        );
    }
    #[test]
    fn a_step_in_an_enclosing_block_or_a_nested_loop_continue_is_clean() {
        assert!(scan_steps(STEP_COVERED, &code_only(STEP_COVERED)).is_empty());
    }
    #[test]
    fn a_quote_char_literal_does_not_open_a_string() {
        assert!(scan_steps(QUOTE_LITERAL, &code_only(QUOTE_LITERAL)).is_empty());
    }
}
