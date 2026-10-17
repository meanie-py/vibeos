use crate::{Corpus, Rule};
use rayon::prelude::*;
use regex::Regex;
#[cfg(test)]
use std::collections::BTreeSet;
use std::collections::HashMap;

pub struct DoubleFetch;

struct Patterns {
    function: Regex,
    kernel_if: Regex,
    otherwise: Regex,
    return_tail: Regex,
    selection: Regex,
    exits: Regex,
    reader: Regex,
    nolint: Regex,
    cast: Regex,
    simple: Regex,
}

impl Patterns {
    fn new() -> Self {
        let re = |s| Regex::new(s).expect("valid double-fetch regex");
        Self {
            function: re(
                r#"^\s*(?:pub(?:\([\w:]+\))?\s+)?(?:const\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*(?:<[^(]*>)?\s*\("#,
            ),
            kernel_if: re(
                r"\bif\s+\(?\s*(\w*(?:mode|Mode)\w*)\s*(==|!=)\s*(?:0(?:i8|u8)?|KernelMode|KPROCESSOR_MODE::KernelMode)(?:\s+as\s+\w+)?\s*\)?\s*\{",
            ),
            otherwise: re(r"^\s*else\s*\{"),
            return_tail: re(r"\breturn\b[^;{}]*;\s*$"),
            selection: re(r"\belse(\s+if\b[^{}]*)?$"),
            exits: re(r"\b(return|break|continue)\b"),
            reader: re(
                r"\b(read_back|probe_read_user|probe_read_generic|safe_copy_from_user|copy_from_user|capture_from_user)\s*(?:::<[^>]*>)?\s*\(",
            ),
            nolint: re(r"nolint:\s*(double-fetch|double_fetch|scan_double_fetch|all)\b"),
            cast: re(r"\.\s*cast(?:_mut|_const)?\s*\(\s*\)$"),
            simple: re(r"^\w+$"),
        }
    }
}

// Match the Python parser: remove comment and string CONTENTS, retaining string
// quotes and line boundaries. This intentionally is not a full Rust lexer.
fn strip(text: &str) -> Vec<String> {
    let mut block = false;
    text.lines()
        .map(|raw| {
            let chars: Vec<_> = raw.chars().collect();
            let mut out = String::new();
            let mut string = false;
            let mut i = 0;
            while i < chars.len() {
                let c = chars[i];
                let next = chars.get(i + 1).copied();
                if block {
                    if c == '*' && next == Some('/') {
                        block = false;
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else if string {
                    if c == '\\' {
                        i += 2;
                    } else {
                        if c == '"' {
                            string = false;
                            out.push(c);
                        }
                        i += 1;
                    }
                } else if c == '/' && next == Some('/') {
                    break;
                } else if c == '/' && next == Some('*') {
                    block = true;
                    i += 2;
                } else {
                    if c == '"' {
                        string = true;
                    }
                    out.push(c);
                    i += 1;
                }
            }
            out
        })
        .collect()
}

fn body_end(lines: &[String], start: usize) -> usize {
    let mut depth = 0isize;
    let mut seen = false;
    for (i, line) in lines.iter().enumerate().skip(start) {
        for c in line.bytes() {
            if c == b'{' {
                depth += 1;
                seen = true;
            } else if c == b'}' {
                depth -= 1;
                if seen && depth == 0 {
                    return i;
                }
            }
        }
    }
    lines.len().saturating_sub(1)
}

fn functions(raw: &[&str], lines: &[String], p: &Patterns) -> Vec<(String, usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let m = match p.function.captures(raw[i]) {
            Some(m) if !raw[i].trim_start().starts_with("//") => m,
            _ => {
                i += 1;
                continue;
            }
        };
        let mut depth = 0isize;
        let mut j = i;
        let mut done = false;
        while j < lines.len() && !done {
            let segment = if j == i {
                &raw[j][m.get(0).unwrap().end() - 1..]
            } else {
                &lines[j]
            };
            for c in segment.bytes() {
                if c == b'(' {
                    depth += 1;
                } else if c == b')' {
                    depth -= 1;
                    if depth == 0 {
                        done = true;
                        break;
                    }
                }
            }
            j += 1;
        }
        let mut k = j - 1;
        let mut tail = if k == i {
            &lines[k][lines[k].find(')').map_or(0, |n| n + 1)..]
        } else {
            &lines[k]
        };
        while k < lines.len() && !tail.contains('{') && !tail.contains(';') {
            k += 1;
            tail = lines.get(k).map_or("{", |s| s.as_str());
        }
        if tail.split('{').next().unwrap_or(tail).contains(';') {
            i = k + 1;
            continue;
        }
        out.push((m[1].to_owned(), j - 1, body_end(lines, j - 1)));
        i = j;
    }
    out
}

fn close_block(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0isize;
    for (i, c) in text.bytes().enumerate().skip(open) {
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn kernel_spans(text: &str, p: &Patterns) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for m in p.kernel_if.captures_iter(text) {
        let open = m.get(0).unwrap().end() - 1;
        let close = close_block(text, open).unwrap_or(text.len().saturating_sub(1));
        if &m[2] == "==" {
            out.push((open, close));
            continue;
        }
        if let Some(otherwise) = p.otherwise.find(&text[close + 1..]) {
            let open = close + otherwise.end();
            out.push((
                open,
                close_block(text, open).unwrap_or(text.len().saturating_sub(1)),
            ));
        } else if p.return_tail.is_match(&text[open + 1..close]) {
            let mut depth = 0isize;
            for (k, c) in text.bytes().enumerate().skip(close + 1) {
                if c == b'{' {
                    depth += 1;
                } else if c == b'}' {
                    if depth == 0 {
                        out.push((close, k));
                        break;
                    }
                    depth -= 1;
                }
            }
        }
    }
    out
}

fn arg_at(text: &str, open: usize, index: usize) -> Option<&str> {
    let mut depth = 0isize;
    let mut start = open + 1;
    let mut current = 0;
    for (k, c) in text.bytes().enumerate().skip(open) {
        if b"([{".contains(&c) {
            depth += 1;
        } else if b")]}".contains(&c) {
            depth -= 1;
            if depth == 0 {
                return (current == index).then_some(&text[start..k]);
            }
        }
        if c == b',' && depth == 1 {
            if current == index {
                return Some(&text[start..k]);
            }
            current += 1;
            start = k + 1;
        }
    }
    (current == index).then_some(&text[start..])
}

fn norm(expr: &str, p: &Patterns) -> String {
    let mut e = expr.trim();
    loop {
        let prev = e;
        let mut depth = 0isize;
        let mut last = None;
        for (i, c) in e.bytes().enumerate() {
            if b"([{".contains(&c) {
                depth += 1;
            } else if b")]}".contains(&c) {
                depth -= 1;
            } else if depth == 0
                && e.as_bytes()[i..].starts_with(b" as ")
                && (i == 0 || e.as_bytes()[i - 1] != b'_')
            {
                last = Some(i);
            }
        }
        if let Some(last) = last {
            e = e[..last].trim();
        }
        if let Some(cast) = p.cast.find(e) {
            e = e[..cast.start()].trim();
        }
        if e == prev {
            break;
        }
    }
    if e.starts_with('(') && e.ends_with(')') {
        let mut depth = 0isize;
        let mut wrap = true;
        for (i, c) in e.bytes().enumerate() {
            depth += isize::from(c == b'(') - isize::from(c == b')');
            if depth == 0 && i != e.len() - 1 {
                wrap = false;
                break;
            }
        }
        if wrap {
            e = e[1..e.len() - 1].trim();
        }
    }
    e.chars().filter(|c| !c.is_whitespace()).collect()
}

fn enclosing(text: &str, pos: usize) -> Vec<usize> {
    let mut stack = Vec::new();
    for (i, c) in text.bytes().enumerate().take(pos) {
        if c == b'{' {
            stack.push(i);
        } else if c == b'}' {
            stack.pop();
        }
    }
    stack
}

fn previous_word_or_dot(text: &str, start: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_some_and(|c| c == '.' || c == '_' || c.is_alphanumeric())
}

fn exclusive(text: &str, a: usize, b: usize, p: &Patterns) -> bool {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    let pa = enclosing(text, a);
    let pb = enclosing(text, b);
    let mut k = 0;
    while k < pa.len() && k < pb.len() && pa[k] == pb[k] {
        k += 1;
    }
    let arm = |brace: usize| {
        let mut start = brace.saturating_sub(200);
        while !text.is_char_boundary(start) {
            start += 1;
        }
        let pre = text[start..brace].trim_end();
        pre.ends_with("=>") || p.selection.is_match(pre)
    };
    if k < pa.len() && k < pb.len() && (arm(pa[k]) || arm(pb[k])) {
        return true;
    }
    if k < pa.len() {
        let close = close_block(text, pa[k]).unwrap_or(text.len());
        if b > close {
            for m in p.exits.find_iter(&text[a..close]) {
                if previous_word_or_dot(&text[a..close], m.start()) {
                    continue;
                }
                let seg = &text[a..a + m.start()];
                let depth = seg.bytes().filter(|c| *c == b'{').count() as isize
                    - seg.bytes().filter(|c| *c == b'}').count() as isize;
                if depth <= 0 {
                    return true;
                }
            }
        }
    }
    false
}

fn reassigned(text: &str, a: usize, b: usize, src: &str, p: &Patterns) -> bool {
    if !p.simple.is_match(src) {
        return false;
    }
    let re = Regex::new(&format!(r"\b{}\s*(?:=|\+=|-=)", regex::escape(src)))
        .expect("escaped assignment regex");
    let text = &text[a..b];
    let found = re.find_iter(text).any(|m| {
        !previous_word_or_dot(text, m.start()) && text.as_bytes().get(m.end()) != Some(&b'=')
    });
    found
}

fn scan(text: &str, p: &Patterns) -> Vec<(String, String)> {
    let raw: Vec<_> = text.lines().collect();
    let lines = strip(text);
    let mut out = Vec::new();
    for (name, body0, end) in functions(&raw, &lines, p) {
        let body = lines[body0..=end].join("\n");
        let kernel = kernel_spans(&body, p);
        let mut sites: HashMap<String, Vec<usize>> = HashMap::new();
        for m in p.reader.captures_iter(&body) {
            let full = m.get(0).unwrap();
            let off = full.start();
            if previous_word_or_dot(&body, off)
                || kernel.iter().any(|(a, b)| *a <= off && off <= *b)
            {
                continue;
            }
            let line = body0 + body[..off].bytes().filter(|b| *b == b'\n').count();
            if p.nolint.is_match(raw[line]) {
                continue;
            }
            let arg_index = if matches!(
                &m[1],
                "read_back" | "probe_read_user" | "probe_read_generic"
            ) {
                0
            } else {
                1
            };
            if let Some(arg) = arg_at(&body, full.end() - 1, arg_index) {
                let src = norm(arg, p);
                if !src.is_empty() {
                    sites.entry(src).or_default().push(off);
                }
            }
        }
        for (src, mut offs) in sites {
            offs.sort_unstable();
            offs.dedup();
            if (0..offs.len()).any(|i| {
                (i + 1..offs.len()).any(|j| {
                    !exclusive(&body, offs[i], offs[j], p)
                        && !reassigned(&body, offs[i], offs[j], &src, p)
                })
            }) {
                out.push((name.clone(), src));
            }
        }
    }
    out
}

impl Rule for DoubleFetch {
    fn name(&self) -> &'static str {
        "scan_double_fetch"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let out: Vec<Vec<String>> = corpus
            .rust_files()
            .par_bridge()
            .map(|file| {
                let path = file.resolved_path().to_string_lossy();
                if ["-sys", "/target/", "rtl-strings", "ntdll"]
                    .iter()
                    .any(|part| path.contains(part))
                    || file
                        .path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().contains("bindgen"))
                {
                    return Vec::new();
                }
                let mut out = Vec::new();
                for (name, src) in scan(&file.text, &p) {
                    out.push(format!("{path}: {name} {src}"));
                }
                out
            })
            .collect();
        Ok(out.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sources(body: &str) -> BTreeSet<String> {
        scan(&format!("unsafe fn foo() {{\n{body}\n}}"), &Patterns::new())
            .into_iter()
            .map(|(_, src)| src)
            .collect()
    }
    #[test]
    fn repeated_normalized_reads() {
        assert_eq!(
            sources("let a = read_back(p as *const T); let b = probe_read_user(p.cast());"),
            BTreeSet::from(["p".to_owned()])
        );
        assert_eq!(
            sources("safe_copy_from_user(dst,p,4); capture_from_user(dst, p, 4);"),
            BTreeSet::from(["p".to_owned()])
        );
        assert!(sources("read_back(p); read_back(q);").is_empty());
        assert!(sources("for i in 0..n { read_back(p.add(i)); }").is_empty());
    }
    #[test]
    fn exclusive_and_exiting_arms() {
        for body in [
            "if a { read_back(p); } else { read_back(p); }",
            "match x { 0 => { read_back(p); }, _ => { read_back(p); } }",
            "if a { read_back(p); return; } read_back(p);",
            "loop { if a { read_back(p); break; } read_back(p); }",
        ] {
            assert!(sources(body).is_empty(), "{body}");
        }
        assert!(!sources("if a { read_back(p); } read_back(p);").is_empty());
    }
    #[test]
    fn reassignment_and_equality() {
        assert!(sources("read_back(p); p = p.add(1); read_back(p);").is_empty());
        assert!(!sources("read_back(p); if p == q {} read_back(p);").is_empty());
        assert!(sources("read_back(p); p += 1; read_back(p);").is_empty());
    }
    #[test]
    fn suppressions_kernel_arms_comments_and_strings() {
        assert!(sources("read_back(p); // nolint: double-fetch reason\nread_back(p);").is_empty());
        assert!(
            sources("if mode != KernelMode { read_back(p); } else { read_back(p); }").is_empty()
        );
        assert!(sources("if mode == KernelMode { read_back(p); read_back(p); }").is_empty());
        assert!(
            sources("if mode != KernelMode { read_back(p); return 0; } read_back(p);").is_empty()
        );
        assert!(sources("read_back(p); /* read_back(p); */ let s = \"read_back(p)\";").is_empty());
    }
    #[test]
    fn inner_casts_and_outer_parentheses() {
        let p = Patterns::new();
        assert_eq!(
            norm(" (*(p as *const T)).field as *const u8 ", &p),
            "(*(pas*constT)).field"
        );
        assert_eq!(norm(" (p) ", &p), "p");
        assert_eq!(arg_at("f(a, (b,c), d)", 1, 1), Some(" (b,c)"));
    }
    #[test]
    fn declarations_and_multiline_signatures() {
        let text = "extern \"C\" {\n fn no_body(p: T);\n}\npub unsafe extern \"system\" fn Example(\n p: T,\n) -> T\n{\n read_back(p);\n read_back(p);\n}\n";
        assert_eq!(
            scan(text, &Patterns::new()),
            vec![("Example".to_owned(), "p".to_owned())]
        );
    }
}
