use crate::{Corpus, Rule};
use regex::Regex;
pub struct MisalignedCast;
const BYTE: &[&str] = &[
    "u8", "i8", "c_char", "c_uchar", "c_schar", "CHAR", "UCHAR", "BYTE", "BOOLEAN",
];
const WIDE: &[&str] = &[
    "u16",
    "i16",
    "WCHAR",
    "USHORT",
    "wchar_t",
    "c_ushort",
    "SHORT",
    "WORD",
    "u32",
    "i32",
    "ULONG",
    "LONG",
    "DWORD",
    "UINT",
    "INT",
    "c_ulong",
    "c_long",
    "c_int",
    "c_uint",
    "u64",
    "i64",
    "ULONGLONG",
    "LONGLONG",
];
const WIDE_PTR: &[&str] = &[
    "PWSTR",
    "LPWSTR",
    "PCWSTR",
    "LPCWSTR",
    "PWCHAR",
    "PWCH",
    "PCWCH",
    "LPWCH",
    "LPCWCH",
    "PUSHORT",
    "PWORD",
    "LPWORD",
    "PULONG",
    "PLONG",
    "PDWORD",
    "LPDWORD",
    "PUINT",
    "LPUINT",
    "PINT",
    "LPINT",
    "PULONGLONG",
    "PLONGLONG",
    "PWSZ",
    "PCWSZ",
    "PSHORT",
];
struct Patterns {
    blank: Regex,
    function: Regex,
    cast: Regex,
    prefilter: Regex,
    array: Regex,
    array_type: Regex,
    nolint: Regex,
    direct_before: Regex,
    direct_after: Regex,
    read_before: Regex,
    read_after: Regex,
    assign: Regex,
    type_pre: Regex,
    mul_pre: Regex,
    as_after: Regex,
}
impl Patterns {
    fn new() -> Self {
        let re = |s| Regex::new(s).expect("valid misaligned-cast regex");
        Self {
            blank: re(r#"(?s)//[^\n]*|/\*.*?\*/|b?"(?:\\.|[^"\\])*"|b?'(?:\\.|[^'\\\n])'"#),
            function: re(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{]*>)?\s*\("),
            cast: re(r"\bas\s+(?:\*\s*(mut|const)\s+([A-Za-z_][\w:]*)|([A-Z][A-Z0-9_]*)\b)"),
            prefilter: re(r"\bas\s+(?:\*\s*(?:mut|const)\s+[A-Za-z_]|P[A-Z]|LP[A-Z])"),
            array: re(
                r"(?:([A-Za-z_]\w*)\s*\.\s*as_(?:mut_)?ptr\s*\(\s*\))|(?:&\s*(?:mut\s+)?(?:\(\*\s*\w+\s*\)\s*\.\s*)?([A-Za-z_]\w*)\s*\[)|(?:addr_of(?:_mut)?!\s*\(\s*(?:\(\*\s*\w+\s*\)\s*\.\s*)?([A-Za-z_]\w*)\s*\[)",
            ),
            array_type: re(r"^\[\s*(?:[\w:]+::)?(\w+)\s*;"),
            nolint: re(r"//\s*nolint(?::\s*misaligned-cast\b|\s*$)"),
            direct_before: re(r"\*\s*\(\s*\(?\s*$"),
            direct_after: re(
                r"^\s*\)\s*(?:\.\s*(?:add|offset|sub|wrapping_add)\s*\([^()]*\)\s*\)?)?\s*(?:[^.\w(=]|=[^=]|$)",
            ),
            read_before: re(r"\(\s*$"),
            read_after: re(r"^\s*\)\s*\.\s*(?:write|read)\s*\("),
            assign: re(r"(?:\blet\s+(?:mut\s+)?)?([A-Za-z_]\w*)\s*(?::\s*[^=;]+)?=\s*$"),
            type_pre: re(r"(?:\bmut|\bconst)\s*$"),
            mul_pre: re(r"[\w)\]]\s*$"),
            as_after: re(r"^\s*as\b"),
        }
    }
}
struct Function<'a> {
    name: &'a str,
    params: &'a str,
    body: &'a str,
    start: usize,
}
fn matched(text: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let mut depth = 0isize;
    for (i, c) in text.bytes().enumerate().skip(open) {
        if c == left {
            depth += 1;
        } else if c == right {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn functions<'a>(text: &'a str, p: &Patterns) -> Vec<Function<'a>> {
    let mut out = Vec::new();
    for m in p.function.captures_iter(text) {
        let end = m.get(0).unwrap().end();
        let Some(close) = matched(text, end - 1, b'(', b')') else {
            continue;
        };
        let Some(brace) = text[close..].find('{').map(|n| close + n) else {
            continue;
        };
        if text[close..brace].contains(';') {
            continue;
        }
        let Some(body_end) = matched(text, brace, b'{', b'}') else {
            continue;
        };
        out.push(Function {
            name: m.get(1).unwrap().as_str(),
            params: &text[end..close],
            body: &text[brace..body_end + 1],
            start: brace,
        });
    }
    out
}
fn boundary_before(text: &str, at: usize) -> bool {
    text[..at]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
}
fn decl<'a>(scope: &'a str, name: &str) -> Option<&'a str> {
    let mut found = None;
    let mut search = 0;
    while let Some(relative) = scope[search..].find(name) {
        let at = search + relative;
        let next = at + name.len();
        search = at + name.chars().next().unwrap().len_utf8();
        if boundary_before(scope, at)
            || scope[next..]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let mut j = next;
        while scope
            .as_bytes()
            .get(j)
            .is_some_and(|c| b" \t\n".contains(c))
        {
            j += 1;
        }
        if scope.as_bytes().get(j) != Some(&b':') || scope.as_bytes().get(j + 1) == Some(&b':') {
            continue;
        }
        let mut depth = 0isize;
        let mut k = j + 1;
        while k < scope.len() {
            let c = scope.as_bytes()[k];
            if b"<[(".contains(&c) {
                depth += 1;
            } else if b">])".contains(&c) {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            } else if b",;={".contains(&c) && depth == 0 {
                break;
            }
            k += 1;
        }
        let ty = scope[j + 1..k].trim();
        if !ty.is_empty() {
            found = Some(ty);
        }
    }
    found
}
fn operand(text: &str, at: usize) -> (&str, usize) {
    let bytes = text.as_bytes();
    let mut j = at;
    while j > 0 && b" \t\n".contains(&bytes[j - 1]) {
        j -= 1;
    }
    let end = j;
    while j > 0 {
        let c = text[..j].chars().next_back().unwrap();
        if c == ')' || c == ']' {
            let mut depth = 0isize;
            let mut found = None;
            for (k, c) in text[..j].char_indices().rev() {
                if c == ')' || c == ']' {
                    depth += 1;
                } else if c == '(' || c == '[' {
                    depth -= 1;
                    if depth == 0 {
                        found = Some(k);
                        break;
                    }
                }
            }
            match found {
                Some(k) => j = k,
                None => break,
            };
            continue;
        }
        if c.is_alphanumeric() || "_.!".contains(c) {
            j -= c.len_utf8();
            continue;
        }
        break;
    }
    (&text[j..end], j)
}
fn slice_chars_before(text: &str, pos: usize, count: usize) -> &str {
    let start = text[..pos]
        .char_indices()
        .rev()
        .nth(count.saturating_sub(1))
        .map_or(0, |(i, _)| i);
    &text[start..pos]
}
fn slice_chars_after(text: &str, pos: usize, count: usize) -> &str {
    let end = text[pos..]
        .char_indices()
        .nth(count)
        .map_or(text.len(), |(i, _)| pos + i);
    &text[pos..end]
}
fn deref_direct(text: &str, start: usize, end: usize, p: &Patterns) -> bool {
    let before = slice_chars_before(text, start, 40);
    let after = slice_chars_after(text, end, 60);
    (p.direct_before.is_match(before) && p.direct_after.is_match(after))
        || (p.read_before.is_match(before) && p.read_after.is_match(after))
}
fn deref_of(var: &str, body: &str, p: &Patterns) -> Option<usize> {
    let v = regex::escape(var);
    // Negative lookahead after the first variable is checked explicitly below.
    let re=Regex::new(&format!(r"\*\s*\(?\s*({v})\b(?:\s*\.\s*(?:add|offset|sub|wrapping_add)\s*\([^()]*(?:\([^()]*\))*[^()]*\))?|\(\s*\*\s*{v}\s*\)|\b{v}\s*\.\s*(?:write|read)\s*\(")).expect("escaped dereference regex");
    for m in re.captures_iter(body) {
        let full = m.get(0).unwrap();
        if let Some(name) = m.get(1) {
            if p.as_after.is_match(&body[name.end()..]) {
                continue;
            }
        }
        let pre = slice_chars_before(body, full.start(), 12);
        if p.type_pre.is_match(pre) {
            continue;
        }
        if full.as_str().starts_with('*') && p.mul_pre.is_match(pre) {
            continue;
        }
        return Some(full.start());
    }
    None
}
fn scan(raw: &str, p: &Patterns) -> Vec<(usize, String, String, String)> {
    if [
        "automatically generated by rust-bindgen",
        "@generated",
        "Do not edit manually",
    ]
    .iter()
    .any(|s| raw.chars().take(2000).collect::<String>().contains(s))
        || !p.prefilter.is_match(raw)
    {
        return Vec::new();
    }
    let text = p.blank.replace_all(raw, |m: &regex::Captures<'_>| {
        m[0].as_bytes()
            .iter()
            .map(|c| if *c == b'\n' { '\n' } else { ' ' })
            .collect::<String>()
    });
    let lines: Vec<_> = raw.split('\n').collect();
    let funcs = functions(&text, p);
    let mut out = Vec::new();
    for m in p.cast.captures_iter(&text) {
        let elem = if let Some(elem) = m.get(2) {
            let e = elem.as_str().rsplit("::").next().unwrap();
            if WIDE.contains(&e) {
                e
            } else {
                continue;
            }
        } else {
            let e = m.get(3).unwrap().as_str();
            if WIDE_PTR.contains(&e) {
                e
            } else {
                continue;
            }
        };
        let full = m.get(0).unwrap();
        let (operand, op_start) = operand(&text, full.start());
        if operand.is_empty() {
            continue;
        }
        let owner = funcs
            .iter()
            .filter(|f| f.start <= full.start() && full.start() < f.start + f.body.len())
            .max_by_key(|f| f.start);
        let scope = owner.map(|f| format!("{}{}", f.params, f.body));
        let scope = scope.as_deref().unwrap_or(&text);
        // Default --lint excludes byte-pointer bases; only arrays reach this branch.
        let Some(base_match) = p.array.captures(operand) else {
            continue;
        };
        let base = base_match
            .get(1)
            .or_else(|| base_match.get(2))
            .or_else(|| base_match.get(3))
            .unwrap()
            .as_str();
        let Some(ty) = decl(scope, base).or_else(|| decl(&text, base)) else {
            continue;
        };
        let Some(array) = p.array_type.captures(ty.trim()) else {
            continue;
        };
        if !BYTE.contains(&array[1].rsplit("::").next().unwrap()) {
            continue;
        }
        let line = text[..full.start()].bytes().filter(|b| *b == b'\n').count() + 1;
        if p.nolint.is_match(lines[line - 1]) {
            continue;
        }
        let mut hit = deref_direct(&text, op_start, full.end(), p);
        if !hit {
            let before = slice_chars_before(&text, op_start, 120);
            if let (Some(lhs), Some(owner)) = (p.assign.captures(before), owner) {
                if text[full.end()..].trim_start().starts_with(';') {
                    let rel = full.end() - owner.start;
                    if let Some(at) = deref_of(&lhs[1], &owner.body[rel..], p) {
                        let tail = slice_chars_after(owner.body, rel + at, 80);
                        if !tail.split(';').next().unwrap_or(tail).contains("unaligned") {
                            hit = true;
                        }
                    }
                }
            }
        }
        if hit {
            out.push((
                line,
                owner.map_or("", |f| f.name).to_owned(),
                base.to_owned(),
                elem.to_owned(),
            ));
        }
    }
    out
}
impl Rule for MisalignedCast {
    fn name(&self) -> &'static str {
        "scan_misaligned_cast"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        let p = Patterns::new();
        let mut files: Vec<_> = corpus.rust_files().collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut out = Vec::new();
        for f in files {
            for (line, name, base, elem) in scan(&f.text, &p) {
                out.push(format!(
                    "{}:{line}: misaligned-cast {name}({base} as *{elem})",
                    f.path.display()
                ));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn hits(text: &str) -> Vec<(usize, String, String, String)> {
        scan(text, &Patterns::new())
    }
    #[test]
    fn arrays_direct_and_indirect() {
        assert_eq!(
            hits("fn foo() { let buf: [u8; 8] = [0;8]; *(buf.as_mut_ptr() as *mut u32) = 1; }")
                .len(),
            1
        );
        assert_eq!(hits("fn foo() { let buf: [BYTE; 8] = [0;8]; let p = buf.as_ptr() as *const ULONG; let x = *p; }").len(),1);
        assert_eq!(hits("struct S { Buffer: [u8; 8] }\nfn foo(ctx:*mut S) { *((*ctx).Buffer.as_mut_ptr() as *mut u32) = 1; }").len(),1);
    }
    #[test]
    fn safe_and_pointer_only() {
        for text in [
            "fn f(buf:*mut u8) { let p = buf as *mut u32; *p = 1; }",
            "fn f() { let buf:[u8;8]; let p = buf.as_ptr() as *const u32; memcpy(dst,p,4); }",
            "fn f() { let buf:[u8;8]; let p = buf.as_ptr() as *const u32; p.read_unaligned(); }",
            "fn f() { let buf:[u8;8]; let p = buf.as_ptr() as *const u8; let x = *p; }",
        ] {
            assert!(hits(text).is_empty(), "{text}");
        }
    }
    #[test]
    fn suppressions_and_comments() {
        assert!(hits("fn f(){let buf:[u8;8]; *(buf.as_ptr() as *const u32); } // nolint: misaligned-cast aligned").is_empty());
        assert!(
            hits("// @generated\nfn f(){let buf:[u8;8]; *(buf.as_ptr() as *const u32); }")
                .is_empty()
        );
        assert!(hits("fn f(){let buf:[u8;8]; /* *(buf.as_ptr() as *const u32); */ }").is_empty());
    }
    #[test]
    fn references_aliases_and_shadowing() {
        assert_eq!(
            hits("fn f(){let buf:[c_char;8]; *(buf.as_ptr() as PULONG) = 1; }").len(),
            1
        );
        assert_eq!(decl("x: u32; let x: [u8; 4];", "x"), Some("[u8; 4]"));
        assert_eq!(
            operand("x = (*ctx).Buffer.as_ptr().add(4) as *const u32", 34).0,
            "(*ctx).Buffer.as_ptr().add(4)"
        );
    }
    #[test]
    fn preserves_python_operand_and_assignment_limitations() {
        // Python's operand walk drops the leading &, so this base is not recognized.
        assert!(hits("fn f(){let buf:[c_char;8]; *(&buf[0] as PULONG) = 1; }").is_empty());
        // Python's assignment regex greedily captures `ctx` through the signature
        // annotation, then searches for a dereference of ctx rather than p.
        assert!(hits("struct S { Buffer: [u8; 8] }\nfn foo(ctx:*mut S) { let p = (*ctx).Buffer.as_mut_ptr() as *mut u32; p.write(1); }").is_empty());
    }
}
