//! Detect locals/parameters one pointer level deeper than their C pointer typedef.
use crate::{Corpus, Rule, SourceFile};
use regex::{Captures, Regex};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

pub struct PtrTypedefIndirection;
impl Rule for PtrTypedefIndirection {
    fn name(&self) -> &'static str {
        "scan_ptr_typedef_indirection"
    }
    fn needs_module_inputs(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files: BTreeMap<_, _> = c.files.iter().map(|f| (f.path.clone(), f)).collect();
        let mut out = BTreeSet::new();
        for (module, inputs) in &c.module_inputs().modules {
            if inputs.c_files.is_empty() {
                continue;
            }
            let mut rust: BTreeMap<String, Vec<(&SourceFile, String, usize)>> = BTreeMap::new();
            for path in &inputs.rust_files {
                let Some(f) = files.get(path).copied() else {
                    continue;
                };
                let filename = f.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if filename.contains("bindgen") || filename.starts_with("types_") {
                    continue;
                }
                let tree = f.rust_tree()?;
                let mut stack = vec![tree.root_node()];
                while let Some(n) = stack.pop() {
                    if n.kind() == "function_item" {
                        if let (Some(name), Some(_body)) =
                            (n.child_by_field_name("name"), n.child_by_field_name("body"))
                        {
                            let fn_name = f.text[name.start_byte()..name.end_byte()].to_owned();
                            let body = f.text[n.start_byte()..n.end_byte()].to_owned();
                            // body_audit returns function_item nodes in a stack walk and includes attributes;
                            // find() of that full text preserves its legacy first-occurrence offset behavior.
                            let at = f.text.find(&body).unwrap_or(0);
                            rust.entry(fn_name).or_default().push((f, body, at));
                        }
                    }
                    if matches!(n.kind(), "macro_definition" | "macro_invocation") {
                        continue;
                    }
                    let mut cursor = n.walk();
                    stack.extend(n.named_children(&mut cursor));
                }
            }
            for cp in &inputs.c_files {
                let Some(cf) = files.get(cp).copied() else {
                    continue;
                };
                for (name, c_body) in c_definitions(&cf.text) {
                    for (rf, rust_body, at) in rust.get(&name).into_iter().flatten() {
                        for (line, msg) in findings_for(&c_body, &rf.text, rust_body, *at) {
                            out.insert(format!(
                                "{}:{line}: {name}: {msg} (C: {})",
                                rf.path.display(),
                                cf.path.display()
                            ));
                        }
                    }
                }
            }
            let _ = module;
        }
        Ok(out.into_iter().collect())
    }
}

const NOT_FN: &[&str] = &[
    "if", "else", "for", "while", "switch", "return", "do", "sizeof", "case", "default", "typedef",
    "struct", "union", "enum", "static", "extern", "const", "void", "break", "continue", "goto",
];
fn c_functions(text: &str) -> BTreeSet<String> {
    let mut code = clean_c(text);
    let mut depth = 0usize;
    let mut kept = String::new();
    for line in code.split_inclusive('\n') {
        let t = line.trim();
        if depth > 0 {
            if t.starts_with("#if") {
                depth += 1
            } else if t.starts_with("#endif") {
                depth -= 1
            } else if depth == 1 && t.starts_with("#else") {
                depth = 0
            }
            continue;
        }
        if if_zero().is_match(t) {
            depth = 1;
            continue;
        }
        kept.push_str(line);
    }
    code = cpp_directives().replace_all(&kept, "").into_owned();
    let mut out = BTreeSet::new();
    for m in c_candidate().captures_iter(&code) {
        let name = m.name("name").unwrap().as_str();
        if NOT_FN.contains(&name) {
            continue;
        }
        let open = m.get(0).unwrap().end() - 1;
        if !after_params(&code, open).starts_with('{') {
            continue;
        }
        out.insert(name.to_owned());
    }
    out
}
fn c_definitions(text: &str) -> BTreeMap<String, String> {
    let names = c_functions(text);
    let mut out = BTreeMap::new();
    // The legacy implementation compiled one regex and rescanned the entire raw
    // translation unit for every candidate name. This single lexical pass retains
    // the same per-name first raw occurrence that is followed by a function body.
    for m in raw_name_call().captures_iter(text) {
        let name = m.get(1).unwrap().as_str();
        if !names.contains(name) || out.contains_key(name) {
            continue;
        }
        let open = m.get(0).unwrap().end() - 1;
        let Some(close) = matching(text, open, '(', ')') else {
            continue;
        };
        let mut brace = close + 1;
        while brace < text.len() && text.as_bytes()[brace].is_ascii_whitespace() {
            brace += 1;
        }
        if brace >= text.len() || text.as_bytes()[brace] != b'{' {
            continue;
        }
        if let Some(end) = matching(text, brace, '{', '}') {
            out.insert(
                name.to_owned(),
                text[m.get(0).unwrap().start()..end + 1].to_owned(),
            );
        }
    }
    out
}
fn raw_name_call() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"\b([A-Za-z_]\w*)\s*\(").unwrap())
}
fn if_zero() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"#\s*if\s+0\b").unwrap())
}
fn after_params(s: &str, open: usize) -> &str {
    let mut d = 0i32;
    for (i, b) in s.as_bytes()[open..].iter().enumerate() {
        if *b == b'(' {
            d += 1
        } else if *b == b')' {
            d -= 1;
            if d == 0 {
                return s[open + i + 1..].trim_start();
            }
        }
    }
    ""
}
fn matching(s: &str, open: usize, a: char, b: char) -> Option<usize> {
    let mut d = 0i32;
    for (i, c) in s[open..].char_indices() {
        if c == a {
            d += 1
        } else if c == b {
            d -= 1;
            if d == 0 {
                return Some(open + i);
            }
        }
    }
    None
}
fn clean_c(s: &str) -> String {
    c_noise()
        .replace_all(s, |m: &Captures<'_>| {
            m[0].chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect::<String>()
        })
        .into_owned()
}
fn c_noise() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(
            r#"(?s)//[^\n]*|/\*.*?\*/|L?\"(?:\\.|[^\"\\])*\"|L?'(?:\\.|[^'\\])*'"#,
        )
        .unwrap()
    })
}
fn cpp_directives() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(?m)^[ \t]*#.*(?:\\\n.*)*$").unwrap())
}
fn c_candidate() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(
            r"(?m)^(?:[A-Za-z_][\w \t\*]*?[ \t\*])?(?P<name>[A-Za-z_]\w*)[ \t]*\(",
        )
        .unwrap()
    })
}
fn findings_for(
    c_body: &str,
    rust_text: &str,
    rust_body: &str,
    body_at: usize,
) -> Vec<(usize, String)> {
    findings_for_impl(c_body, rust_text, rust_body, body_at, None)
}
fn findings_for_impl(
    c_body: &str,
    rust_text: &str,
    rust_body: &str,
    body_at: usize,
    known_lines: Option<&[usize]>,
) -> Vec<(usize, String)> {
    let c_code = clean_c(c_body);
    let mut c_types: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for m in c_ptr_decl().captures_iter(&c_code) {
        let start = m.get(0).unwrap().start();
        if start > 0 {
            let prev = c_code.as_bytes()[start - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'*' {
                continue;
            }
        }
        let end = m.get(0).unwrap().end();
        let tail = &c_code[end..];
        let Some(ch) = tail.chars().next() else {
            continue;
        };
        if !matches!(ch, ';' | '=' | ',' | ')') {
            continue;
        }
        c_types
            .entry(m[2].to_owned())
            .or_default()
            .insert(m[1].to_owned());
    }
    let lines: Vec<_> = rust_text.lines().collect();
    let body_line = known_lines.map_or_else(
        || {
            rust_text[..body_at.min(rust_text.len())]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1
        },
        |starts| starts.partition_point(|&at| at <= body_at.min(rust_text.len())),
    );
    let mut out = Vec::new();
    let mut prev = 0usize;
    let mut line = body_line;
    for m in rust_ptr_decl().captures_iter(rust_body) {
        let (name, kind, ty) = (&m[1], &m[2], &m[3]);
        let start = m.get(0).unwrap().start();
        line += rust_body[prev..start]
            .bytes()
            .filter(|b| *b == b'\n')
            .count();
        prev = m.get(0).unwrap().end();
        if !c_types.get(name).is_some_and(|s| s.contains(ty)) {
            continue;
        }
        if nolint().is_match(lines.get(line - 1).copied().unwrap_or("")) {
            continue;
        }
        out.push((line,format!("`{name}: *{kind} {ty}` adds a level of indirection: the C declares `{ty} {name}`, and {ty} is already a pointer")));
    }
    out.sort();
    out.dedup();
    out
}
fn c_ptr_decl() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"((?:P|LP)[A-Z][A-Z0-9_]*)\s+([A-Za-z_]\w*)\s*").unwrap()
    })
}
fn rust_ptr_decl() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(||crate::regexes::cached(r"\b(?:let\s+(?:mut\s+)?)?([A-Za-z_]\w*)\s*:\s*\*(mut|const)\s+((?:P|LP)[A-Z][A-Z0-9_]*)\b").unwrap())
}
fn nolint() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"nolint:\s*ptr-typedef-indirection\b").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_filecopy_typedef_and_parameter_count_shape() {
        let c="UINT CALLBACK FileCopyCallback(PVOID Context, UINT Notification, UINT_PTR Param1, UINT_PTR Param2) { PFILEPATHS_W FilePathInfo; PCM_INT13_DRIVE_PARAMETER *Int13Drives; FilePathInfo = (PFILEPATHS_W)Param1; return 0; }";
        let body = c_definitions(c).remove("FileCopyCallback").unwrap();
        let r="pub unsafe extern \"system\" fn FileCopyCallback(Context:PVOID, Notification:UINT, Param1:UINT_PTR, Param2:UINT_PTR)->UINT {\nlet mut FilePathInfo: *mut PFILEPATHS_W = Param1 as *mut PFILEPATHS_W;\nlet mut Int13Drives: *mut PCM_INT13_DRIVE_PARAMETER = core::ptr::null_mut();\n0\n}";
        let at = r.find("pub unsafe").unwrap();
        let got = findings_for(&body, r, &r[at..], at);
        assert_eq!(got.len(), 1, "{got:?}");
        assert_eq!(got[0].0, 2);
        assert!(got[0].1.contains("FilePathInfo"));
    }
    #[test]
    fn genuine_pointer_to_pointer_and_nolint() {
        let c = "void f(){ PCM_INT13_DRIVE_PARAMETER *Int13Drives; }";
        let r =
            "fn f(){ let mut Int13Drives:*mut PCM_INT13_DRIVE_PARAMETER = core::ptr::null_mut(); }";
        let b = c_definitions(c).get("f").cloned().unwrap_or_default();
        assert!(findings_for(&b, r, r, 0).is_empty());
        let c = "void f(){ PFILEPATHS_W FilePathInfo; }";
        let r="fn f(){ let mut FilePathInfo: *mut PFILEPATHS_W = core::ptr::null_mut(); } // nolint: ptr-typedef-indirection";
        let b = c_definitions(c).get("f").cloned().unwrap_or_default();
        assert!(findings_for(&b, r, r, 0).is_empty());
    }
    #[test]
    fn skipped_rust_declarations_still_advance_line_cursor() {
        let c = "void f(){ PFOO second; }";
        let r = "fn f(){\nlet first: *mut POTHER;\nlet second: *mut PFOO;\n}";
        let body = c_definitions(c).remove("f").unwrap();
        let got = findings_for(&body, r, r, 0);
        assert_eq!(got.iter().map(|x| x.0).collect::<Vec<_>>(), vec![3]);
    }
}
