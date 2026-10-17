//! C increments/decrements inside call arguments whose object is read later in that expression.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::sync::LazyLock;

pub struct ArgSideEffects;
impl Rule for ArgSideEffects {
    fn name(&self) -> &'static str {
        "scan_arg_side_effects"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        const SETS: &[SourceSet] = &[SourceSet::tree(
            "arg-side-effects",
            &[
                "dll/win32/msvcrt",
                "sdk/lib/rtl",
                "sdk/lib/crt",
                "ntoskrnl",
                "win32ss",
            ],
            &["c"],
            &[],
        )
        .with_pruning(&[])];
        SETS
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        const ROOTS: &[&str] = &[
            "dll/win32/msvcrt",
            "sdk/lib/rtl",
            "sdk/lib/crt",
            "ntoskrnl",
            "win32ss",
        ];
        let mut out = Vec::new();
        for root in ROOTS {
            let mut files: Vec<_> = corpus
                .sources("arg-side-effects")
                .filter(|f| {
                    f.path.starts_with(root) && f.path.extension().is_some_and(|e| e == "c")
                })
                .collect();
            files.sort_by(|a, b| a.path.cmp(&b.path));
            for file in files {
                for (line, obj, callee, _expr) in scan_text(&file.text) {
                    out.push(format!("{}:{line}: `{obj}` is incremented inside the argument list of `{callee}` and read again in the same expression — check the Rust performs the side effect BEFORE the dependent operand", file.path.display()));
                }
            }
        }
        Ok(out)
    }
}

const IDENT: &str = r"[A-Za-z_][A-Za-z0-9_]*";
struct Patterns {
    incdec: Regex,
    call_open: Regex,
    for_open: Regex,
}
static PATTERNS: LazyLock<Patterns> = LazyLock::new(|| {
    let member = format!(r"{IDENT}(?:\s*(?:->|\.)\s*{IDENT})*");
    Patterns {
        incdec: crate::regexes::cached(&format!(
            r"(?:(\+\+|--)\s*({member}))|(?:({member})\s*(\+\+|--))"
        ))
        .unwrap(),
        call_open: crate::regexes::cached(&format!(r"({IDENT})\s*\($")).unwrap(),
        for_open: crate::regexes::cached(r"\bfor\s*\(").unwrap(),
    }
});
const KEYWORDS: &[&str] = &[
    "if", "while", "for", "switch", "return", "sizeof", "defined",
];

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut block = false;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        if block {
            if bytes[i..].starts_with(b"*/") {
                out.push_str("  ");
                i += 2;
                block = false;
            } else {
                out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/*") {
            out.push_str("  ");
            i += 2;
            block = true;
        } else if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
        } else if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let begin = i;
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                i += if bytes[i] == b'\\' { 2 } else { 1 };
            }
            let end = (i + 1).min(bytes.len());
            out.extend(std::iter::repeat(' ').take(text[begin..end].chars().count()));
            i = end;
        } else {
            let ch = text[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn logical_lines(text: &str) -> Vec<(usize, String)> {
    let clean = strip_comments(text);
    let mut out = Vec::new();
    let (mut buf, mut start, mut depth) = (String::new(), 0usize, 0i32);
    for (i, raw) in clean.split('\n').enumerate() {
        let lineno = i + 1;
        let s = raw.trim();
        if buf.is_empty() && (s.is_empty() || matches!(s.chars().next(), Some('*' | '/' | '#'))) {
            continue;
        }
        if buf.is_empty() {
            start = lineno;
        }
        buf.push(' ');
        buf.push_str(s);
        depth += s.matches('(').count() as i32 - s.matches(')').count() as i32;
        if depth <= 0 || lineno - start > 20 {
            out.push((start, buf.trim().to_owned()));
            buf.clear();
            depth = 0;
        }
    }
    if !buf.is_empty() {
        out.push((start, buf.trim().to_owned()));
    }
    out
}

fn enclosing_call(expr: &str, at: usize) -> Option<String> {
    let bytes = expr.as_bytes();
    let mut depth = 0i32;
    for i in (0..at).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' if depth == 0 => {
                let cap = PATTERNS.call_open.captures(&expr[..i + 1])?;
                let name = cap.get(1)?.as_str();
                return (!KEYWORDS.contains(&name)).then(|| name.to_owned());
            }
            b'(' => depth -= 1,
            _ => (),
        }
    }
    None
}
fn in_for_clause(expr: &str, at: usize) -> bool {
    let head = &expr[..at];
    let Some(m) = PATTERNS.for_open.find_iter(head).last() else {
        return false;
    };
    let open = m.end() - 1;
    head[open..].matches('(').count() as i32 - head[open..].matches(')').count() as i32 >= 1
}

fn scan_text(text: &str) -> Vec<(usize, String, String, String)> {
    let mut out = Vec::new();
    for (line, expr) in logical_lines(text) {
        for m in PATTERNS.incdec.captures_iter(&expr) {
            let Some(raw_obj) = m.get(2).or_else(|| m.get(3)) else {
                continue;
            };
            // Python's `obj.replace(" ", "")` removes literal ASCII spaces only.
            let obj = raw_obj.as_str().replace(' ', "");
            let start = m.get(0).unwrap().start();
            if in_for_clause(&expr, start) {
                continue;
            }
            let Some(callee) = enclosing_call(&expr, start) else {
                continue;
            };
            let base = obj
                .split("->")
                .next()
                .unwrap_or(&obj)
                .split('.')
                .next()
                .unwrap_or("");
            let use_re = crate::regexes::cached(&format!(r"\b{}\b", regex::escape(base))).unwrap();
            if use_re.is_match(&expr[m.get(0).unwrap().end()..]) {
                out.push((line, obj, callee, expr.chars().take(200).collect()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const THE_BUG: &str = "static BOOL get_function_signature(struct parsed_symbol* sym, struct function_signature* fs)\n{\n    if (!get_calling_convention(*sym->current++,\n                                &fs->call_conv, &fs->exported,\n                                sym->flags & ~UNDNAME_NO_ALLOCATION_LANGUAGE) ||\n        !demangle_datatype(sym, &fs->return_ct, 0))\n        return FALSE;\n    return TRUE;\n}\n";
    #[test]
    fn shipped_bug_spanning_four_lines_is_found() {
        assert_eq!(
            scan_text(THE_BUG)
                .iter()
                .map(|h| (h.0, h.1.as_str(), h.2.as_str()))
                .collect::<Vec<_>>(),
            vec![(3, "sym->current", "get_calling_convention")]
        );
    }
    #[test]
    fn later_argument_and_same_line_forms_are_detected() {
        assert_eq!(scan_text("void f(void) { g(*p++, p, 0); }\n").len(), 1);
        let s = THE_BUG
            .replace("\n                                ", " ")
            .replace("\n        !demangle", " !demangle");
        assert_eq!(scan_text(&s).len(), 1);
    }
    #[test]
    fn for_clause_direct_condition_comments_strings_and_unobserved_are_clean() {
        assert!(
            scan_text("void f(void) { for (p = inpath; *p; p++) if (*p == '.') end = p; }\n")
                .is_empty()
        );
        assert!(
            scan_text("void f(void) { if (++a->num >= a->max) a->max = a->num; }\n").is_empty()
        );
        assert!(scan_text(
            "/* Round size++ up so space(size) is available. */\nvoid f(void) { return; }\n"
        )
        .is_empty());
        assert!(scan_text("void f(void) { log(\"g(*p++, p)\"); }\n").is_empty());
        assert!(scan_text("void f(void) { g(*p++); }\n").is_empty());
    }
    #[test]
    fn unbalanced_macro_resynchronizes_after_twenty_lines() {
        let s = format!(
            "#define ODD(x) ((x\nvoid f(void) {{ g(*p++, p, 0); }}\n{}",
            "void pad(void) { }\n".repeat(40)
        );
        assert_eq!(scan_text(&s).len(), 1);
        assert_eq!(scan_text(&s)[0].0, 2);
    }
    #[test]
    fn corpus_scope_is_the_five_original_c_trees() {
        let files = vec![
            crate::SourceFile::new(
                "dll/win32/msvcrt/undname.c".into(),
                "void f(void) { g(*p++, p, 0); }".into(),
            ),
            crate::SourceFile::new(
                "base/foo.c".into(),
                "void f(void) { g(*p++, p, 0); }".into(),
            ),
        ];
        let c = Corpus::new(std::path::PathBuf::from("fixture"), files);
        let out = ArgSideEffects.run(&c).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].starts_with("dll/win32/msvcrt/undname.c:1:"));
    }
}
