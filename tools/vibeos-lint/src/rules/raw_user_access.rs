use crate::{twins::Function, Corpus, Rule, SourceFile};
use regex::{Captures, Regex};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

pub struct RawUserAccess;
const RAW_PRIMS: &[&str] = &[
    "copy_nonoverlapping",
    "copy",
    "write_bytes",
    "write_volatile",
    "read_volatile",
    "write_unaligned",
    "read_unaligned",
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
    "from_raw_parts",
    "from_raw_parts_mut",
    "slice_from_raw_parts",
    "slice_from_raw_parts_mut",
];
const SKIP_PARTS: &[&str] = &["-sys", "/target/", "rtl-strings", "ntdll"];
static POINTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(\*\s*(mut|const)\b|P[A-Z0-9_]\w*$|LP[A-Z]\w*$)").unwrap());
static RAW_SYSCALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"pub\s+unsafe\s+extern\s+"system"\s+fn\s+(Nt\w+)\s*\("#).unwrap()
});
// Python's FN_RE is anchored at the raw signature line. This intentionally skips
// functions whose line starts with an attribute, even though tree-sitter indexes them.
static PY_FN_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^\s*(?:pub(?:\([\w:]+\))?\s+)?(?:const\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*(?:<[^(]*>)?\s*\("#,
    )
    .unwrap()
});
static NOLINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"nolint:\s*(raw-user-access|raw_user_access|scan_raw_user_access|all)\b").unwrap()
});
static ASSIGN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:\blet\s+(?:mut\s+)?)?(\w+)\s*(?::[^=;{}]*)?=").unwrap());
static KERNEL_IF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bif\s+\(?\s*(\w*(?:mode|Mode)\w*)\s*(==|!=)\s*(?:0(?:i8|u8)?|KernelMode|KPROCESSOR_MODE::KernelMode)(?:\s+as\s+\w+)?\s*\)?\s*\{").unwrap()
});
static ADDR_OF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\baddr_of(?:_mut)?!\s*\(").unwrap());
static CALL_TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"((?:\w+::)*)(\w+)\s*(?:::<[^<(]*>)?\s*$").unwrap());
static READBACK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bread_back\s*(?:::<[^>]*>)?\s*\(").unwrap());
const KEYWORDS: &[&str] = &[
    "if", "while", "match", "for", "return", "loop", "Some", "Ok", "Err",
];

#[derive(Clone)]
pub(crate) struct FnDef {
    pub(crate) function: Function,
    pub(crate) path: PathBuf,
    crate_key: String,
    pub(crate) raw: Arc<Vec<String>>,
    code: Arc<Vec<String>>,
    first: usize,
    last: usize,
}
#[derive(Clone, Debug)]
pub(crate) struct WalkItem {
    pub(crate) kind: &'static str,
    pub(crate) name: String,
    pub(crate) line: usize,
    pub(crate) text: String,
    pub(crate) callee: String,
    pub(crate) arg: usize,
}
fn strip_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut block = false;
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let b = raw.as_bytes();
        let mut v = Vec::new();
        let (mut i, mut string) = (0usize, false);
        while i < b.len() {
            if block {
                if let Some(p) = b[i..].windows(2).position(|x| x == b"*/") {
                    block = false;
                    i += p + 2
                } else {
                    i = b.len()
                }
                continue;
            }
            if string {
                let c = b[i];
                if c == b'\\' {
                    i = (i + 2).min(b.len());
                    continue;
                }
                if c == b'"' {
                    string = false;
                    v.push(b'"')
                }
                i += 1;
                continue;
            }
            if b.get(i..i + 2) == Some(b"//") {
                break;
            }
            if b.get(i..i + 2) == Some(b"/*") {
                block = true;
                i += 2;
                continue;
            }
            if b[i] == b'"' {
                string = true;
                v.push(b'"');
                i += 1;
                continue;
            }
            v.push(b[i]);
            i += 1;
        }
        out.push(String::from_utf8_lossy(&v).into_owned());
    }
    out
}
fn split_params(s: &str) -> Vec<(String, String)> {
    let (mut d, mut cur) = (0i32, String::new());
    let mut parts = Vec::new();
    for ch in s.chars() {
        if "(<[".contains(ch) {
            d += 1
        } else if ")>]".contains(ch) {
            d -= 1
        }
        if ch == ',' && d == 0 {
            parts.push(std::mem::take(&mut cur))
        } else {
            cur.push(ch)
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur)
    }
    parts
        .into_iter()
        .filter_map(|p| {
            let (n, t) = p.split_once(':')?;
            Some((
                n.replace("mut ", "").trim().to_string(),
                t.trim().to_string(),
            ))
        })
        .collect()
}
pub(crate) fn parse_file(file: &SourceFile, root: &Path) -> Vec<FnDef> {
    let raw = file.text.split('\n').map(str::to_owned).collect::<Vec<_>>();
    let code = strip_lines(&file.text);
    let raw = Arc::new(raw);
    let code = Arc::new(code);
    file.twin_functions(root)
        .iter()
        .filter_map(|f| {
            let sig_line = f.line.checked_sub(1).and_then(|i| raw.get(i))?;
            if !PY_FN_START.is_match(sig_line) {
                return None;
            }
            let first = file.line(f.body_start).saturating_sub(1);
            let end_byte = f.body_start + f.body.len().saturating_sub(1);
            let last = file.line(end_byte).saturating_sub(1);
            if first >= raw.len() {
                return None;
            }
            let path = file.resolved_path().to_path_buf();
            let ps = path.to_string_lossy();
            let crate_key = ps.split("/src/").next().unwrap_or(&ps).to_string();
            Some(FnDef {
                function: f.clone(),
                path,
                crate_key,
                raw: raw.clone(),
                code: code.clone(),
                first,
                last: last.min(raw.len() - 1),
            })
        })
        .collect()
}
fn ptr_params(f: &FnDef) -> Vec<String> {
    split_params(&f.function.params)
        .into_iter()
        .filter(|(_, t)| POINTER.is_match(t))
        .map(|(n, _)| n)
        .collect()
}
fn custom(p: &str) -> Regex {
    crate::regexes::cached(p).expect("valid generated scanner regex")
}
fn pattern_hit(re: &Regex, text: &str) -> bool {
    re.find_iter(text).any(|m| {
        let before = text[..m.start()].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.') {
            return false;
        }
        if re.as_str().contains("(?:core::ptr::") && before == Some(':') {
            return false;
        }
        let tail = &text[m.end()..];
        !(re.as_str().contains(")?=") && tail.starts_with('='))
    })
}
fn first_assignment<'a>(re: &Regex, text: &'a str) -> Option<Captures<'a>> {
    re.captures_iter(text).find(|m| {
        let end = m.get(0).unwrap().end();
        !text[end..].starts_with('=')
    })
}
fn access_patterns(name: &str, derived: bool) -> Vec<Regex> {
    let n = regex::escape(name);
    let prims = RAW_PRIMS
        .iter()
        .map(|s| regex::escape(s))
        .collect::<Vec<_>>()
        .join("|");
    if derived {
        return [
            format!(r"\*\s*{n}\b"),
            format!(r"\(\*{n}\)\s*\."),
            format!(r"\*\(\s*{n}\s+as\b"),
            format!(r"=\s*\*\s*{n}\b"),
            format!(
                r"(?:core::ptr::|ptr::|core::slice::|slice::)?(?:{prims})\s*\([^;]*?{n}\b[^;)]*,"
            ),
        ]
        .iter()
        .map(|p| custom(p))
        .collect();
    }
    let arith =
        r"(?:add|offset|sub|wrapping_add|wrapping_offset|wrapping_sub|byte_add|byte_offset)";
    [
        format!(r"\*\s*{n}\b\s*(\.{arith}\([^)]*\)\s*)?="),
        format!(r"\*\s*{n}\s*\.\s*{arith}\s*\("),
        format!(r"\(\*{n}\)\s*\."),
        format!(r"\*\(\s*{n}\s+as\b"),
        format!(r"=\s*\*\s*{n}\b"),
        format!(
            r"(?:core::ptr::|ptr::|core::slice::|slice::)?(?:{prims})\s*\((?:[^;]*?[(,\s])?{n}\b"
        ),
    ]
    .iter()
    .map(|p| custom(p))
    .collect()
}
fn forwarded(text: &str, name: &str) -> bool {
    let n = regex::escape(name);
    custom(&format!(r"^\s*(?:{n}(?:\s+as\s+[^,]+)?|{n}\s*\.\s*cast(?:_mut|_const)?(?:::<[^>]*>)?\(\)|(?:\w+::)*opaque_ptr(?:_const)?\(\s*{n}\s*\))\s*$")).is_match(text)
}
fn forwarded_field(text: &str, name: &str) -> bool {
    let n = regex::escape(name);
    custom(&format!(r"^\s*{n}\s*\.\s*\w+\s*(?:as\s+[^,]+)?\s*$")).is_match(text)
}
fn call_sites(body: &str, name: &str, field: bool) -> Vec<(String, usize, usize, usize)> {
    let mut out = vec![];
    let n = regex::escape(name);
    for m in custom(&format!(r"{n}\b")).find_iter(body).filter(|m| {
        !body[..m.start()]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '!')
    }) {
        let (mut d, mut k) = (0i32, m.start() as isize - 1);
        while k >= 0 {
            let c = body.as_bytes()[k as usize] as char;
            if ")] }".contains(c) && c != ' ' {
                d += 1
            } else if "([{ ".contains(c) && c != ' ' {
                if d == 0 {
                    break;
                }
                d -= 1
            }
            k -= 1
        }
        if k < 0 || body.as_bytes()[k as usize] != b'(' {
            continue;
        }
        let pre = &body[(k as usize).saturating_sub(160)..k as usize];
        let Some(c) = CALL_TAIL.captures(pre).filter(|m| {
            !pre[..m.get(0).unwrap().start()]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '!')
        }) else {
            continue;
        };
        let callee = c[2].to_string();
        if KEYWORDS.contains(&callee.as_str()) {
            continue;
        }
        let (mut args, mut cur, mut depth, mut q) =
            (Vec::new(), String::new(), 0i32, k as usize + 1);
        let mut idx = None;
        while q < body.len() {
            let ch = body.as_bytes()[q] as char;
            if q == m.start() {
                idx = Some(args.len())
            }
            if "([{ ".contains(ch) && ch != ' ' {
                depth += 1
            } else if ")]}".contains(ch) {
                if depth == 0 {
                    break;
                }
                depth -= 1
            }
            if ch == ',' && depth == 0 {
                args.push(std::mem::take(&mut cur))
            } else {
                cur.push(ch)
            }
            q += 1
        }
        args.push(cur);
        if let Some(i) = idx {
            if i < args.len()
                && if field {
                    forwarded_field(&args[i], name)
                } else {
                    forwarded(&args[i], name)
                }
            {
                out.push((callee, i, m.start(), args.len()))
            }
        }
    }
    out
}
fn blank_addr(body: &str) -> String {
    let mut chars = body.chars().collect::<Vec<_>>();
    for m in ADDR_OF.find_iter(body) {
        let mut depth = 0i32;
        for k in body[m.end() - 1..]
            .char_indices()
            .map(|(i, _)| m.end() - 1 + i)
        {
            if body.as_bytes()[k] == b'(' {
                depth += 1
            } else if body.as_bytes()[k] == b')' {
                depth -= 1;
                if depth == 0 {
                    for c in &mut chars[m.start()..k + 1] {
                        if *c != '\n' {
                            *c = ' '
                        }
                    }
                    break;
                }
            }
        }
    }
    chars.into_iter().collect()
}
fn stmt_end(text: &str, start: usize) -> usize {
    let (mut d, mut i) = (0i32, start);
    for c in text[start..].char_indices().map(|(o, c)| (start + o, c)) {
        let (k, ch) = c;
        if "([{ ".contains(ch) && ch != ' ' {
            d += 1
        } else if ")] }".contains(ch) && ch != ' ' {
            if d == 0 {
                return k;
            }
            d -= 1
        } else if ch == ';' && d == 0 {
            return k;
        }
    }
    i = text.len();
    i
}
fn first_arg(text: &str, lpar: usize) -> String {
    let (mut d, mut out) = (0i32, String::new());
    for (k, c) in text[lpar..].char_indices().map(|(i, c)| (lpar + i, c)) {
        if "([{ ".contains(c) && c != ' ' {
            d += 1;
            if d == 1 {
                continue;
            }
        } else if ")]}".contains(c) && c != ' ' {
            d -= 1;
            if d == 0 {
                break;
            }
        }
        if d == 1 && c == ',' {
            break;
        }
        out.push(c);
        let _ = k;
    }
    out
}
fn captures(text: &str, seed: &HashSet<String>) -> (HashMap<String, usize>, Vec<(String, usize)>) {
    let (mut captured, mut derived) = (HashMap::new(), Vec::new());
    if !text.contains("read_back") {
        return (captured, derived);
    }
    let mut tracked = seed.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for m in ASSIGN
            .captures_iter(text)
            .filter(|m| !text[m.get(0).unwrap().end()..].starts_with('='))
        {
            let whole = m.get(0).unwrap();
            let lhs = &m[1];
            let init = &text[whole.end()..stmt_end(text, whole.end())];
            if let Some(rb) = READBACK.find(init) {
                if !captured.contains_key(lhs) {
                    let arg = first_arg(init, rb.end() - 1);
                    if tracked.iter().any(|n| forwarded(&arg, n))
                        || captured.keys().any(|n| forwarded_field(&arg, n))
                    {
                        captured.insert(lhs.to_string(), whole.start());
                        changed = true
                    }
                }
                continue;
            }
            if tracked.contains(lhs) {
                continue;
            }
            let fld = custom(r"\s*(\w+)\s*\.\s*\w+\s*(?:as\b|$)");
            if let Some(f) = fld.captures(init) {
                if captured.contains_key(&f[1]) && !tracked.contains(lhs) {
                    derived.push((lhs.to_string(), whole.start()));
                    tracked.insert(lhs.to_string());
                    changed = true
                }
            }
        }
    }
    (captured, derived)
}
fn block_end(text: &str, open: usize) -> usize {
    let mut d = 0;
    for (i, c) in text.char_indices().filter(|(i, _)| *i >= open) {
        if c == '{' {
            d += 1
        } else if c == '}' {
            d -= 1;
            if d == 0 {
                return i;
            }
        }
    }
    text.len().saturating_sub(1)
}
fn kernel_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = vec![];
    for m in KERNEL_IF.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let open = whole.end() - 1;
        let close = block_end(text, open);
        if &m[2] == "==" {
            spans.push((open, close));
            continue;
        }
        let after = &text[close + 1..];
        if let Some(e) = custom(r"\s*else\s*\{").find(after) {
            let eo = close + 1 + e.end() - 1;
            spans.push((eo, block_end(text, eo)))
        } else if custom(r"\breturn\b[^;{}]*;\s*$").is_match(&text[open + 1..close]) {
            let mut d = 0;
            for (i, c) in text.char_indices().filter(|(i, _)| *i > close) {
                if c == '{' {
                    d += 1
                } else if c == '}' {
                    if d == 0 {
                        spans.push((close, i));
                        break;
                    }
                    d -= 1
                }
            }
        }
    }
    spans
}
fn field_patterns(name: &str) -> Vec<Regex> {
    let e = regex::escape(name);
    let p = RAW_PRIMS
        .iter()
        .map(|p| regex::escape(p))
        .collect::<Vec<_>>()
        .join("|");
    [
        format!(r"\*\s*{e}\s*\.\s*\w+"),
        format!(
            r"(?:core::ptr::|ptr::|core::slice::|slice::)?(?:{p})\s*\([^;]*?{e}\s*\.\s*\w+\b[^;)]*,"
        ),
    ]
    .iter()
    .map(|p| custom(p))
    .collect()
}
fn walk(fn_def: &FnDef, seed: &HashSet<String>) -> Vec<WalkItem> {
    let first = fn_def.first;
    let last = fn_def.last;
    let mut body = fn_def.code[first..=last].join("\n");
    body = blank_addr(&body);
    let names0 = seed.clone();
    let (captured, derived) = captures(&body, &names0);
    let derived_names = derived
        .iter()
        .map(|(n, _)| n.clone())
        .collect::<HashSet<_>>();
    let mut names = names0;
    names.extend(derived_names.iter().cloned());
    let mut dead: HashMap<String, Vec<(usize, usize)>> =
        names.iter().map(|n| (n.clone(), vec![])).collect();
    for (b, off) in &derived {
        dead.get_mut(b)
            .unwrap()
            .push((0, body[..*off].bytes().filter(|&x| x == b'\n').count()))
    }
    let patterns = names
        .iter()
        .map(|n| (n.clone(), access_patterns(n, derived_names.contains(n))))
        .collect::<HashMap<_, _>>();
    let fpatterns = captured
        .keys()
        .map(|s| (s.clone(), field_patterns(s)))
        .collect::<HashMap<_, _>>();
    let mut seen = HashSet::new();
    let mut out = vec![];
    let base_line = first;
    for (local, line) in body.split('\n').enumerate() {
        let Some(raw) = fn_def.raw.get(first + local) else {
            continue;
        };
        let rebind = first_assignment(
            &custom(r"^\s*(?:let\s+(?:mut\s+)?)?(\w+)\s*(?::[^=]*)?=\s*(.*)"),
            line,
        );
        if let Some(m) = rebind {
            let n = &m[1];
            if names.contains(n) {
                let rhs = &m[2];
                if rhs.trim_start().starts_with('=') {
                    continue;
                }
                let fld = custom(r"(\w+)\s*\.\s*\w+").captures(rhs);
                let is_derive = fld.is_some_and(|f| captured.contains_key(&f[1]));
                if !is_derive
                    && !custom(&format!(
                        r"\s*opaque_ptr(_const)?\(\s*{}\s*\)",
                        regex::escape(n)
                    ))
                    .is_match(rhs)
                {
                    let end = scope_end(&body, local, n);
                    dead.get_mut(n).unwrap().push((local, end));
                    continue;
                }
            }
        }
        if NOLINT.is_match(
            fn_def
                .raw
                .get(first + local)
                .map(String::as_str)
                .unwrap_or(raw),
        ) {
            continue;
        }
        for (name, pats) in &patterns {
            if dead[name].iter().any(|(a, b)| *a <= local && local <= *b) {
                continue;
            }
            if pats.iter().any(|p| pattern_hit(p, line)) && seen.insert((name.clone(), local + 1)) {
                out.push(WalkItem {
                    kind: "hit",
                    name: name.clone(),
                    line: first + local + 1,
                    text: fn_def
                        .raw
                        .get(first + local)
                        .map(|s| s.trim())
                        .unwrap_or(raw)
                        .into(),
                    callee: String::new(),
                    arg: 0,
                })
            }
        }
        for (s, pats) in &fpatterns {
            if pats.iter().any(|p| pattern_hit(p, line)) && seen.insert((s.clone(), local + 1)) {
                out.push(WalkItem {
                    kind: "hit",
                    name: s.clone(),
                    line: first + local + 1,
                    text: fn_def
                        .raw
                        .get(first + local)
                        .map(|s| s.trim())
                        .unwrap_or(raw)
                        .into(),
                    callee: String::new(),
                    arg: 0,
                })
            }
        }
        let _ = base_line;
    }
    for name in &names {
        for (callee, arg, off, nargs) in call_sites(&body, name, false) {
            let line = first + body[..off].bytes().filter(|&x| x == b'\n').count() + 1;
            let local = line - first - 1;
            if first + local >= fn_def.raw.len() {
                continue;
            }
            if NOLINT.is_match(&fn_def.raw[first + local])
                || dead[name].iter().any(|(a, b)| *a <= local && local <= *b)
            {
                continue;
            }
            if RAW_PRIMS.contains(&callee.as_str()) && arg < nargs.saturating_sub(1) {
                if seen.insert((name.clone(), line)) {
                    out.push(WalkItem {
                        kind: "hit",
                        name: name.clone(),
                        line,
                        text: fn_def.raw[first + local].trim().into(),
                        callee,
                        arg,
                    })
                }
            } else if !RAW_PRIMS.contains(&callee.as_str()) {
                out.push(WalkItem {
                    kind: "call",
                    name: name.clone(),
                    line,
                    text: fn_def.raw[first + local].trim().into(),
                    callee,
                    arg,
                })
            }
        }
    }
    for s in captured.keys() {
        for (callee, arg, off, nargs) in call_sites(&body, s, true) {
            let line = first + body[..off].bytes().filter(|&x| x == b'\n').count() + 1;
            let local = line - first - 1;
            if first + local >= fn_def.raw.len() || NOLINT.is_match(&fn_def.raw[first + local]) {
                continue;
            }
            if RAW_PRIMS.contains(&callee.as_str()) && arg < nargs.saturating_sub(1) {
                if seen.insert((s.clone(), line)) {
                    out.push(WalkItem {
                        kind: "hit",
                        name: s.clone(),
                        line,
                        text: fn_def.raw[first + local].trim().into(),
                        callee,
                        arg,
                    })
                }
            } else if !RAW_PRIMS.contains(&callee.as_str()) {
                out.push(WalkItem {
                    kind: "call",
                    name: s.clone(),
                    line,
                    text: fn_def.raw[first + local].trim().into(),
                    callee,
                    arg,
                })
            }
        }
    }
    let spans = kernel_spans(&body);
    out.retain(|w| {
        let offline = w.line - first - 1;
        let off = body
            .split('\n')
            .take(offline)
            .map(|s| s.len() + 1)
            .sum::<usize>();
        !spans.iter().any(|(a, b)| *a < off && off < *b)
    });
    out
}
fn scope_end(text: &str, line: usize, name: &str) -> usize {
    let starts = text
        .split('\n')
        .scan(0usize, |p, s| {
            let x = *p;
            *p += s.len() + 1;
            Some(x)
        })
        .collect::<Vec<_>>();
    let pos = starts.get(line).copied().unwrap_or(0);
    let mut depth = 0i32;
    let mut open = None;
    for i in (0..pos).rev() {
        let c = text.as_bytes()[i] as char;
        if c == '}' {
            depth += 1
        } else if c == '{' {
            if depth == 0 {
                open = Some(i);
                break;
            }
            depth -= 1
        }
    }
    let Some(op) = open else { return line };
    let n = regex::escape(name);
    let guarded = custom(&format!(
        r"\bif\s+(?:!\s*{n}\s*\.is_null\(\)|{n}\s*!=\s*(?:(?:core::)?ptr::)?null(?:_mut)?\(\))\s*$"
    ))
    .is_match(&text[..op]);
    let closing = |open: usize| -> Option<usize> {
        let mut d = 0i32;
        for (i, c) in text[open..].char_indices() {
            if c == '{' {
                d += 1;
            } else if c == '}' {
                d -= 1;
                if d == 0 {
                    return Some(open + i);
                }
            }
        }
        None
    };
    let from = if guarded {
        let mut outer_depth = 0i32;
        let mut outer = None;
        for i in (0..op).rev() {
            match text.as_bytes()[i] as char {
                '}' => outer_depth += 1,
                '{' if outer_depth == 0 => {
                    outer = Some(i);
                    break;
                }
                '{' => outer_depth -= 1,
                _ => {}
            }
        }
        let Some(outer) = outer else {
            return usize::MAX;
        };
        outer
    } else {
        op
    };
    if let Some(end) = closing(from) {
        return text[..end].bytes().filter(|&b| b == b'\n').count();
    }
    usize::MAX
}
fn syscall(f: &FnDef) -> bool {
    if f.function.name == "NtProcessStartup" {
        return false;
    }
    let lo = f.first.saturating_sub(40);
    let hi = f.first.min(f.raw.len().saturating_sub(1));
    f.raw[lo..=hi].iter().any(|l| {
        RAW_SYSCALL
            .captures(l)
            .is_some_and(|m| m[1] == f.function.name)
            && !l.trim_start().starts_with("//")
    })
}
fn skip(file: &SourceFile) -> bool {
    let p = file.resolved_path().to_string_lossy();
    SKIP_PARTS.iter().any(|s| p.contains(s))
        || file
            .resolved_path()
            .file_name()
            .is_some_and(|n| n.to_string_lossy().contains("bindgen"))
}
fn crate_of(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.split("/src/").next().unwrap_or(&s).to_string()
}
fn resolve<'a>(caller: &FnDef, name: &str, all: &'a [FnDef]) -> Option<&'a FnDef> {
    let local = all
        .iter()
        .filter(|f| f.function.name == name && f.crate_key == caller.crate_key)
        .collect::<Vec<_>>();
    if !local.is_empty() {
        return (local.len() == 1).then(|| local[0]);
    }
    let global = all
        .iter()
        .filter(|f| f.function.name == name)
        .collect::<Vec<_>>();
    (global.len() == 1).then(|| global[0])
}
pub(crate) fn walk_hits(f: &FnDef, names: &HashSet<String>) -> Vec<WalkItem> {
    walk(f, names)
}
pub(crate) fn body_code(f: &FnDef) -> String {
    f.code[f.first..=f.last].join("\n")
}
pub(crate) fn raw_line(f: &FnDef, line: usize) -> &str {
    line.checked_sub(1)
        .and_then(|i| f.raw.get(i))
        .map(String::as_str)
        .unwrap_or("")
}
pub(crate) fn is_skipped(file: &SourceFile) -> bool {
    skip(file)
}
fn scan_syscall(f: &FnDef, all: &[FnDef]) -> Vec<(String, String, usize, String, String)> {
    let params = ptr_params(f);
    let mut out = vec![];
    let mut seen = HashSet::new();
    let mut todo = params
        .iter()
        .map(|p| (f, p.clone(), p.clone(), String::new()))
        .collect::<Vec<_>>();
    while let Some((cur, name, origin, chain)) = todo.pop() {
        let key = (cur.path.clone(), cur.function.name.clone(), name.clone());
        if !seen.insert(key) {
            continue;
        }
        for w in walk(cur, &HashSet::from([name.clone()])) {
            if w.kind == "hit" {
                out.push((
                    origin.clone(),
                    chain.clone(),
                    w.line,
                    w.text,
                    cur.path.display().to_string(),
                ))
            } else if chain.matches("->").count() < 6 {
                if let Some(callee) = resolve(cur, &w.callee, all) {
                    if callee.function.name == f.function.name
                        && callee.path == f.path
                        && callee.function.body_start == f.function.body_start
                        || w.arg >= split_params(&callee.function.params).len()
                    {
                        continue;
                    }
                    let (p, t) = split_params(&callee.function.params)[w.arg].clone();
                    if POINTER.is_match(&t) {
                        let next_chain = format!("{chain} -> {}({p})", callee.function.name);
                        todo.push((callee, p, origin.clone(), next_chain))
                    }
                }
            }
        }
    }
    out
}
impl Rule for RawUserAccess {
    fn name(&self) -> &'static str {
        "scan_raw_user_access"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files = c.rust_files().filter(|f| !skip(f)).collect::<Vec<_>>();
        let all = files
            .iter()
            .flat_map(|f| parse_file(f, &c.root))
            .collect::<Vec<_>>();
        let mut rows = std::collections::BTreeMap::<String, Vec<(usize, String)>>::new();
        for f in &all {
            if !syscall(f) {
                continue;
            }
            for (param, chain, line, text, path) in scan_syscall(f, &all) {
                let tail = if chain.is_empty() {
                    String::new()
                } else {
                    format!(" {chain} @{path}")
                };
                let key = format!("{}: {} {param}{tail}", f.path.display(), f.function.name);
                rows.entry(key).or_default().push((line, text));
            }
        }
        Ok(rows.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(rel: &str, text: &str) -> SourceFile {
        SourceFile::new(rel.into(), text.into())
    }
    fn rows(files: Vec<SourceFile>) -> Vec<String> {
        let c = Corpus::new(PathBuf::from("/repo"), files);
        RawUserAccess.run(&c).unwrap()
    }
    const SYSCALL: &str = "pub unsafe extern \"system\" fn NtThing(P: *mut ULONG, Mode: KPROCESSOR_MODE) -> NTSTATUS {\n";
    const USTR: &str = "pub unsafe extern \"system\" fn NtStr(P: *mut _UNICODE_STRING, Mode: KPROCESSOR_MODE) -> NTSTATUS {\n";
    #[test]
    fn follows_pointer_through_callee_and_names_chain() {
        let src = format!(
            "{SYSCALL} helper(P as *mut u8,1); 0 }}\nunsafe fn helper(q:*mut u8,n:u32){{ inner(q); }}\nunsafe fn inner(r:*mut u8){{ *r=0; }}"
        );
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]),
            vec![
                "sdk/lib/rust/x-rust/src/a.rs: NtThing P  -> helper(q) -> inner(r) @sdk/lib/rust/x-rust/src/a.rs"
            ]
        )
    }
    #[test]
    fn does_not_resolve_attribute_prefixed_helper_python_fn_re_skips() {
        let src = format!(
            "{SYSCALL} helper(P); 0 }}\n#[inline(always)] unsafe fn helper(q:*mut ULONG){{ *q=1; }}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty());
    }
    #[test]
    fn safe_primitive_and_kernel_arm_are_clean() {
        let src = format!(
            "{SYSCALL} helper(P); 0 }}\nunsafe fn helper(q:*mut ULONG){{ let _=write_back(q,0); }}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty());
        let src = format!(
            "{SYSCALL}\n helper(P,Mode);\n 0\n}}\nunsafe fn helper(q:*mut ULONG, AccessMode:KPROCESSOR_MODE){{\n if AccessMode != 0 {{\n let _=write_back(q,0);\n }} else {{\n *q=0;\n }}\n}}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty())
    }
    #[test]
    fn cross_crate_unique_resolves_but_ambiguous_does_not() {
        let call = format!(
            "{SYSCALL} ObThing(P); 0 }}\nextern \"system\" {{ fn ObThing(q:*mut ULONG); }}"
        );
        let callee = "pub unsafe extern \"system\" fn ObThing(q:*mut ULONG) { *q=1; }";
        assert_eq!(
            rows(vec![
                source("sdk/lib/rust/x-rust/src/a.rs", &call),
                source("sdk/lib/rust/ob-rust/src/b.rs", callee)
            ]),
            vec![
                "sdk/lib/rust/x-rust/src/a.rs: NtThing P  -> ObThing(q) @sdk/lib/rust/ob-rust/src/b.rs"
            ]
        );
        assert!(rows(vec![
            source("sdk/lib/rust/x-rust/src/a.rs", &call),
            source("sdk/lib/rust/ob-rust/src/b.rs", callee),
            source("sdk/lib/rust/y-rust/src/c.rs", callee)
        ])
        .is_empty())
    }
    #[test]
    fn user_return_then_kernel_rest_and_arm_rebind() {
        let src = format!(
            "{SYSCALL}\n if Mode != 0 {{\n let v=read_back(P);\n return v;\n }}\n inner(P);\n 0\n}}\nunsafe fn inner(r:*mut ULONG){{\n *r=0;\n}}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty());
        let src = format!(
            "{SYSCALL}\n match Mode {{\n 1 => {{\n P=core::ptr::null_mut();\n }},\n _ => {{\n *P=3;\n }}\n }}\n 0\n}}"
        );
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).len(),
            1
        )
    }
    #[test]
    fn null_guard_capture_and_nolint_stop_chain() {
        let src = format!(
            "{SYSCALL}\n let mut local:ULONG=0;\n if !P.is_null() {{\n local=read_back(P);\n P=&mut local;\n }}\n inner(P);\n 0\n}}\nunsafe fn inner(r:*mut ULONG){{ *r=0; }}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty());
        let src = format!(
            "{SYSCALL} inner(P); // nolint: raw-user-access reviewed\n0 }}\nunsafe fn inner(r:*mut ULONG){{ *r=0; }}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty())
    }
    #[test]
    fn native_startup_and_embedded_capture_fields() {
        let src = "pub unsafe extern \"system\" fn NtProcessStartup(Peb:PPEB){ let x=(*Peb).ImageBaseAddress; }";
        assert!(rows(vec![source("sdk/lib/rust/u-rust/src/a.rs", src)]).is_empty());
        let src = format!(
            "{USTR} let hdr=match read_back(P) {{ Ok(v)=>v, Err(s)=>return s }}; core::ptr::copy_nonoverlapping(hdr.Buffer as *const u8,dst,hdr.Length as usize); 0 }}"
        );
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]),
            vec!["sdk/lib/rust/x-rust/src/a.rs: NtStr P"]
        );
        let src =
            format!("{USTR} let hdr=read_back(P).unwrap();\n let b=hdr.Buffer;\n *b=0;\n 0\n}}");
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]),
            vec!["sdk/lib/rust/x-rust/src/a.rs: NtStr P"]
        )
    }
    #[test]
    fn captured_copy_safe_scalar_length_and_inline_callee() {
        let src = format!(
            "{USTR} let hdr=read_back(P).unwrap(); let _=safe_copy_raw(dst,hdr.Buffer as *const _,hdr.Length as usize); 0 }}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty());
        let src = format!(
            "{USTR} let hdr=read_back(P).unwrap(); let cb=hdr.MaximumLength as usize; RtlZeroMemory(pool as *mut c_void,cb as _); 0 }}"
        );
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).is_empty());
        let src = format!(
            "{USTR} let oa=read_back(P).unwrap(); sink(oa.ObjectName); 0 }}\nunsafe fn sink(us:*mut _UNICODE_STRING){{ (*us).Length=0; }}"
        );
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]),
            vec![
                "sdk/lib/rust/x-rust/src/a.rs: NtStr P  -> sink(us) @sdk/lib/rust/x-rust/src/a.rs"
            ]
        );
        let safe = src.replace("(*us).Length=0;", "let _=read_back(us);");
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &safe)]).is_empty())
    }
    #[test]
    fn raw_slice_and_offset_accesses_are_found() {
        let src = format!("{SYSCALL} let s=core::slice::from_raw_parts(P,8); let _=s[0]; 0 }}");
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).len(),
            1
        );
        let src = format!("{SYSCALL} *P.offset(2)=7; let _=*P.offset(3); 0 }}");
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &src)]).len(),
            1
        )
    }
    #[test]
    fn original_control_bad_and_good() {
        let bad = "pub unsafe extern \"system\" fn NtOpenKeyedEvent(OutHandle: PHANDLE, Access: ULONG) -> NTSTATUS { let OutHandle=opaque_ptr(OutHandle); probe_write_generic(OutHandle,1,1)?; *OutHandle=core::ptr::null_mut(); STATUS_SUCCESS }";
        let good = "pub unsafe extern \"system\" fn NtOpenKeyedEvent(OutHandle: PHANDLE, Info: PIO_STATUS) -> NTSTATUS { let OutHandle=opaque_ptr(OutHandle); probe_write_generic(OutHandle,1,1)?; let _=write_back(OutHandle,core::ptr::null_mut()); let _=write_back(core::ptr::addr_of_mut!((*Info).Information),0); STATUS_SUCCESS }";
        assert_eq!(
            rows(vec![source("sdk/lib/rust/ex-rust/src/keyedevt.rs", bad)]),
            vec!["sdk/lib/rust/ex-rust/src/keyedevt.rs: NtOpenKeyedEvent OutHandle"]
        );
        assert!(rows(vec![source("sdk/lib/rust/ex-rust/src/keyedevt.rs", good)]).is_empty());
    }
    #[test]
    fn user_arm_raw_access_flags_and_clean_user_arm_does_not() {
        let src = "pub unsafe extern \"system\" fn NtThing(P:*mut ULONG,Mode:KPROCESSOR_MODE)->NTSTATUS {\n helper(P,Mode);\n 0\n}\nunsafe fn helper(q:*mut ULONG, AccessMode:KPROCESSOR_MODE) {\n if AccessMode != 0 {\n *q=1;\n } else {\n *q=0;\n }\n}";
        assert_eq!(
            rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", src)]).len(),
            1
        );
        let clean = src.replace("*q=1;", "let _=write_back(q,1);");
        assert!(rows(vec![source("sdk/lib/rust/x-rust/src/a.rs", &clean)]).is_empty());
    }
}
