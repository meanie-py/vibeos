//! Compare C-commented bit masks with the live Rust mask.
use crate::{Corpus, Rule, SourceSet};
use regex::bytes::Regex as BytesRegex;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub struct MaskDrift;
const SOURCES: &[SourceSet] = &[
    SourceSet::tree("mask-rust", &["sdk/lib/rust"], &["rs"], &[])
        .with_pruning(&["target", ".responses"])
        .with_inputs(),
    crate::catalog::SOURCES[0],
    crate::catalog::SOURCES[1],
    crate::catalog::SOURCES[2],
];
impl Rule for MaskDrift {
    fn name(&self) -> &'static str {
        "scan_mask_drift"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut catalog = crate::catalog::catalog(c);
        let mut seen: BTreeMap<String, BTreeSet<i128>> = BTreeMap::new();
        for f in c.sources("mask-define-headers") {
            for m in define_re().captures_iter(&f.text) {
                if let Some(v) = crate::catalog::header_value(m.get(2).unwrap().as_str())
                    .or_else(|| eval_const(m.get(2).unwrap().as_str()))
                {
                    seen.entry(m[1].to_owned()).or_default().insert(v);
                }
            }
        }
        let fallback: BTreeMap<String, i128> = seen
            .into_iter()
            .filter_map(|(n, v)| (v.len() == 1).then(|| (n, *v.iter().next().unwrap())))
            .collect();
        let mut out = BTreeSet::new();
        for f in c.sources("mask-rust") {
            let p = f.path.to_string_lossy();
            let prefix: String = f.text.chars().take(800).collect();
            if p.contains("/tests/") || skip_file().is_match(&p) || generated().is_match(&prefix) {
                continue;
            }
            let bytes = f.text.as_bytes();
            if !prefilter().is_match(bytes) && !test_prefilter().is_match(bytes) {
                continue;
            }
            let (live, comments) = split_comments(f)?;
            // Keep the legacy one-based/zero-based mappings, but index the raw line
            // table once instead of restarting an iterator from the beginning for each
            // statement and each bit-test site.
            let raw_lines: Vec<_> = f.text.lines().collect();
            let local = in_file_consts(&f.text);
            let mut c_masks: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();
            for comment in &comments {
                for m in c_mask_re().captures_iter(comment) {
                    let target = m[1].to_ascii_lowercase();
                    let names = names(&trim_operand(&m[2]));
                    if !names.is_empty() {
                        let e = c_masks
                            .entry(target)
                            .or_insert_with(|| (m[1].to_owned(), BTreeSet::new()));
                        e.1.extend(names);
                    }
                }
            }
            let mut live_names: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            let mut live_vals: BTreeMap<String, BTreeSet<i128>> = BTreeMap::new();
            let mut suppressed_targets = BTreeSet::new();
            for (stmt, line0, line1) in statements(&live) {
                let hits: Vec<_> = rs_mask_re().captures_iter(stmt).collect();
                if hits.is_empty() {
                    continue;
                }
                let bound: BTreeSet<String> = let_re()
                    .captures_iter(stmt)
                    .map(|m| m[1].to_ascii_lowercase())
                    .collect();
                if raw_lines
                    .get(line0..=line1)
                    .unwrap_or(&[])
                    .iter()
                    .any(|l| suppressed(l))
                {
                    suppressed_targets.extend(bound);
                    for h in hits {
                        suppressed_targets.insert(h[1].to_ascii_lowercase());
                    }
                    continue;
                }
                for h in hits {
                    let target = h[1].to_ascii_lowercase();
                    let operand = trim_operand(&h[2]);
                    let ns = names(&operand);
                    let flat_target = target.replace('_', "");
                    let mut targets = bound.clone();
                    targets.insert(target.clone());
                    targets.insert(flat_target);
                    for t in targets {
                        if !ns.is_empty() {
                            live_names.entry(t).or_default().extend(ns.iter().cloned());
                        } else if let Some(v) = eval_const(&operand) {
                            live_vals.entry(t).or_default().insert(v);
                        }
                    }
                }
            }
            for (target, (written, ns)) in c_masks {
                let ambiguous_names = ns.len() > 1;
                if suppressed_targets.contains(&target) {
                    continue;
                }
                let flat = target.replace('_', "");
                let vals = live_vals.get(&target).or_else(|| live_vals.get(&flat));
                let live_ns = live_names.get(&target).or_else(|| live_names.get(&flat));
                for name in ns {
                    let kind = if vals.is_none_or(BTreeSet::is_empty) {
                        if live_ns.is_none_or(BTreeSet::is_empty) {
                            Some("DROPPED")
                        } else {
                            None
                        }
                    } else if ambiguous_names {
                        None
                    } else {
                        let want = local
                            .get(&name)
                            .copied()
                            .or_else(|| catalog.get(&name).copied())
                            .or_else(|| fallback.get(&name).copied());
                        match want {
                            None => Some("OPENCODED"),
                            Some(v)
                                if !vals
                                    .unwrap()
                                    .iter()
                                    .any(|x| (x & 0xffff_ffff) == (v & 0xffff_ffff)) =>
                            {
                                Some("MISMATCH")
                            }
                            _ => None,
                        }
                    };
                    if let Some(k) = kind {
                        out.insert(format!(
                            "{}: {} &= ~{} [{}]",
                            f.path.display(),
                            written,
                            name,
                            k
                        ));
                    }
                }
            }
            // TEST form pairing is intentionally separate: C expressions bind by (base,member),
            // while clear masks above use only the immediately-adjacent member identifier.
            for (target, name, kind) in scan_test_form(
                &f.text, &live, &comments, &local, &catalog, &fallback, &raw_lines,
            ) {
                out.insert(format!(
                    "{}: {} & {} [{}]",
                    f.path.display(),
                    target,
                    name,
                    kind
                ));
            }
        }
        Ok(out.into_iter().collect())
    }
}
fn skip_file() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"(bindgen_[a-z0-9_]+\.rs$|/target/|/\.responses/)").unwrap()
    })
}
fn generated() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"automatically generated by rust-bindgen|@generated").unwrap()
    })
}
fn prefilter() -> &'static BytesRegex {
    static R: OnceLock<BytesRegex> = OnceLock::new();
    R.get_or_init(|| BytesRegex::new(r"&=?\s*~").unwrap())
}
fn test_prefilter() -> &'static BytesRegex {
    static R: OnceLock<BytesRegex> = OnceLock::new();
    R.get_or_init(|| BytesRegex::new(r"&\s*\(?\s*[A-Z][A-Z0-9_]{2,}").unwrap())
}
fn define_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"(?m)^[ \t]*#[ \t]*define[ \t]+([A-Z][A-Z0-9_]{2,})[ \t]+(.+)$")
            .unwrap()
    })
}
fn c_mask_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"([A-Za-z_][A-Za-z0-9_]*)\s*&=?\s*~\s*(\(?[^;{}]*)").unwrap()
    })
}
fn rs_mask_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"([A-Za-z_][A-Za-z0-9_]*)\s*&=?\s*\(*\s*!\s*(\(?[^;{}]*)").unwrap()
    })
}
fn let_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        crate::regexes::cached(r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=;]+)?=")
            .unwrap()
    })
}
fn nolint() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"(?i)\bnolint\b\s*[:(\[]?\s*([a-z0-9_-]*)").unwrap())
}
fn suppressed(s: &str) -> bool {
    nolint().captures(s).is_some_and(|m| {
        matches!(
            m.get(1).unwrap().as_str().to_ascii_lowercase().as_str(),
            "" | "all" | "mask-drift" | "mask_drift" | "scan_mask_drift"
        )
    })
}
fn names(s: &str) -> BTreeSet<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| crate::regexes::cached(r"\b([A-Z][A-Z0-9_]{2,})\b").unwrap())
        .captures_iter(s)
        .map(|m| m[1].to_owned())
        .filter(|n| {
            ![
                "ULONG",
                "USHORT",
                "UCHAR",
                "ULONGLONG",
                "ULONG_PTR",
                "SIZE_T",
                "DWORD",
                "ACCESS_MASK",
                "UINT",
                "INT",
                "LONG",
                "WORD",
                "BYTE",
                "ULONG64",
                "ULONG32",
            ]
            .contains(&n.as_str())
        })
        .collect()
}
fn split_comments(f: &crate::SourceFile) -> Result<(String, Vec<String>), String> {
    let tree = f.rust_tree()?;
    let mut nodes: Vec<_> = crate::syntax::preorder(tree.root_node())
        .filter(|n| matches!(n.kind(), "line_comment" | "block_comment"))
        .map(|n| (n.start_byte(), n.end_byte()))
        .collect();
    nodes.sort();
    let b = f.text.as_bytes();
    let mut live = b.to_vec();
    let mut comments = Vec::new();
    for (s, e) in nodes {
        comments.push(String::from_utf8_lossy(&b[s..e]).into_owned());
        for x in &mut live[s..e] {
            if *x != b'\n' {
                *x = b' '
            }
        }
    }
    Ok((String::from_utf8_lossy(&live).into_owned(), comments))
}
fn statements(s: &str) -> Vec<(&str, usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut line = 0usize;
    for (i, c) in s.char_indices() {
        if matches!(c, ';' | '{' | '}') {
            let chunk = &s[pos..i];
            let end = line + chunk.bytes().filter(|b| *b == b'\n').count();
            if !chunk.trim().is_empty() {
                out.push((chunk, line, end));
            }
            line = end;
            pos = i + c.len_utf8();
        }
    }
    let tail = &s[pos..];
    if !tail.trim().is_empty() {
        out.push((
            tail,
            line,
            line + tail.bytes().filter(|b| *b == b'\n').count(),
        ));
    }
    out
}

fn trim_operand(s: &str) -> String {
    let mut d = 0i32;
    let mut i = 0;
    let b = s.as_bytes();
    while i < b.len() {
        if i + 1 < b.len() && matches!(&b[i..i + 2], b"==" | b"!=" | b"&&" | b"||" | b"<=" | b">=")
        {
            break;
        }
        match b[i] {
            b'(' => d += 1,
            b')' if d == 0 => break,
            b')' => d -= 1,
            b',' => break,
            b'<' | b'>'
                if !(i + 1 < b.len()
                    && ((b[i] == b'<' && b[i + 1] == b'<')
                        || (b[i] == b'>' && b[i + 1] == b'>'))) =>
            {
                break
            }
            _ => {}
        }
        i += 1;
    }
    s[..i].trim().to_owned()
}
fn in_file_consts(s: &str) -> BTreeMap<String, i128> {
    let re=crate::regexes::cached(r"(?m)^\s*(?:pub\s*(?:\([^)]*\)\s*)?)?(?:const|static)\s+([A-Z_][A-Z0-9_]*)\s*:\s*[^=]+?=\s*([^;]+);").unwrap();
    re.captures_iter(s)
        .filter_map(|m| eval_const(&m[2]).map(|v| (m[1].to_owned(), v)))
        .fold(BTreeMap::new(), |mut out, (name, value)| {
            out.entry(name).or_insert(value);
            out
        })
}
fn eval_const(s: &str) -> Option<i128> {
    let mut x = crate::regexes::cached(r"\bas\s+[\w:]+")
        .ok()?
        .replace_all(s, "")
        .into_owned();
    x = crate::regexes::cached(r"(0[xX][0-9a-fA-F_]+|\b\d[\d_]*)[uUlL]+\b")
        .ok()?
        .replace_all(&x, "$1")
        .into_owned();
    x = crate::regexes::cached(
        r"(0[xX][0-9a-fA-F_]+|\d[\d_]*)(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)?\b",
    )
    .ok()?
    .replace_all(&x, "$1")
    .into_owned();
    x = x.replace('!', "~");
    eval_expr(&x)
}
fn eval_expr(s: &str) -> Option<i128> {
    let x = s.trim().trim_matches(|c: char| c.is_whitespace());
    if let Some(rest) = x.strip_prefix('~') {
        return eval_expr(rest).map(|v| !v);
    }
    if let Some(rest) = x.strip_prefix('-') {
        return eval_expr(rest).map(|v| -v);
    }
    let clean = x.trim_matches(|c| c == '(' || c == ')').trim();
    if clean != x {
        return eval_expr(clean);
    }
    for op in ["|", "^", "&", "<<", ">>", "+", "-", "*"] {
        if let Some(i) = x.find(op) {
            let a = eval_expr(&x[..i])?;
            let b = eval_expr(&x[i + op.len()..])?;
            return Some(match op {
                "|" => a | b,
                "^" => a ^ b,
                "&" => a & b,
                "<<" => a.checked_shl(b as u32)?,
                ">>" => a.checked_shr(b as u32)?,
                "+" => a + b,
                _ => a - b,
            });
        }
    }
    let x = x.replace('_', "");
    if let Some(h) = x.strip_prefix("0x").or_else(|| x.strip_prefix("0X")) {
        i128::from_str_radix(h, 16).ok()
    } else {
        x.parse().ok()
    }
}

// Test-form scanner uses the C's (base,member) key; names in live Rust suppress
// literals for a target, and only one live literal can be decisive.
fn scan_test_form(
    text: &str,
    live: &str,
    comments: &[String],
    local: &BTreeMap<String, i128>,
    catalog: &BTreeMap<String, i128>,
    fallback: &BTreeMap<String, i128>,
    raw_lines: &[&str],
) -> Vec<(String, String, String)> {
    let mut c: BTreeMap<(String, String), (String, BTreeSet<String>, BTreeSet<i128>)> =
        BTreeMap::new();
    for s in comments {
        for (key, expr, operand, _line) in bit_tests(s) {
            let ns = names(&operand);
            let e = c
                .entry(key)
                .or_insert_with(|| (expr.clone(), BTreeSet::new(), BTreeSet::new()));
            if ns.is_empty() {
                if let Some(v) = eval_const(&operand) {
                    e.2.insert(v);
                }
            } else if ns.len() == 1
                && crate::regexes::cached(r"^\(?\s*[A-Z][A-Z0-9_]*\s*\)?$")
                    .unwrap()
                    .is_match(&operand)
            {
                e.1.extend(ns);
            }
        }
    }
    let mut vals: BTreeMap<(String, String), BTreeSet<i128>> = BTreeMap::new();
    let mut nms: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for (key, _, op, line) in bit_tests(live) {
        let ns = names(&op);
        if ns.is_empty() {
            if !raw_lines.get(line).is_some_and(|s| suppressed(s)) {
                if let Some(v) = eval_const(&op) {
                    vals.entry(key).or_default().insert(v);
                }
            }
        } else {
            nms.entry(key).or_default().extend(ns);
        }
    }
    let mut out = Vec::new();
    for (key, (expr, ns, cvals)) in c {
        let got = vals.get(&key);
        if got.is_none_or(|x| x.is_empty())
            || ns.len() != 1
            || nms.get(&key).is_some_and(|x| !x.is_empty())
            || got.unwrap().len() > 1
            || got.unwrap().iter().any(|x| cvals.contains(x))
        {
            continue;
        }
        let name = ns.iter().next().unwrap();
        let want = local
            .get(name)
            .or_else(|| catalog.get(name))
            .or_else(|| fallback.get(name));
        let kind = match want {
            None => Some("OPENCODED"),
            Some(v)
                if !got
                    .unwrap()
                    .iter()
                    .any(|x| (x & 0xffff_ffff) == (v & 0xffff_ffff)) =>
            {
                Some("MISMATCH")
            }
            _ => None,
        };
        if let Some(k) = kind {
            out.push((expr, name.clone(), k.into()));
        }
    }
    let _ = text;
    out
}
fn left_expr(s: &str, i: usize) -> String {
    let mut j = i;
    let mut depth = 0i32;
    while j > 0 {
        let c = s[..j].chars().next_back().unwrap();
        if c.is_whitespace() {
            let mut k = j - c.len_utf8();
            while k > 0 {
                let p = s[..k].chars().next_back().unwrap();
                if !p.is_whitespace() {
                    break;
                }
                k -= p.len_utf8();
            }
            let before = if k > 0 {
                s[..k].chars().next_back().unwrap()
            } else {
                '\0'
            };
            let after = s[j..].chars().next().unwrap_or('\0');
            if (before.is_alphanumeric() || before == '_')
                && (after.is_alphanumeric() || after == '_')
            {
                break;
            }
            j = k;
            continue;
        }
        if c == ')' || c == ']' {
            depth += 1;
            j -= c.len_utf8();
            continue;
        }
        if c == '(' || c == '[' {
            if depth == 0 {
                break;
            }
            depth -= 1;
            j -= c.len_utf8();
            continue;
        }
        if c == '>' && j >= 2 && &s.as_bytes()[j - 2..j] == b"->" {
            j -= 2;
            continue;
        }
        if !(c.is_alphanumeric() || matches!(c, '_' | '.' | '*')) {
            break;
        }
        j -= c.len_utf8();
    }
    s[j..i].to_owned()
}
fn bit_tests(s: &str) -> Vec<((String, String), String, String, usize)> {
    let mut out = Vec::new();
    let amp = crate::regexes::cached(r"&").unwrap();
    for m in amp.find_iter(s) {
        let i = m.start();
        if (i > 0 && s.as_bytes()[i - 1] == b'&')
            || s.as_bytes()
                .get(i + 1)
                .is_some_and(|b| *b == b'&' || *b == b'=')
        {
            continue;
        }
        let op = trim_operand(s[i + 1..].trim_start());
        if op.is_empty() || op.starts_with('~') || op.starts_with('!') {
            continue;
        }
        let expr = left_expr(s, i);
        if expr.trim().is_empty() {
            continue;
        }
        let ids: Vec<_> = crate::regexes::cached(r"[A-Za-z_][A-Za-z0-9_]*")
            .unwrap()
            .find_iter(&expr)
            .map(|x| x.as_str())
            .filter(|x| {
                !x.starts_with("__bindgen")
                    && !matches!(
                        *x,
                        "if" | "while"
                            | "return"
                            | "let"
                            | "mut"
                            | "as"
                            | "self"
                            | "Self"
                            | "unsafe"
                            | "const"
                            | "static"
                            | "match"
                            | "else"
                            | "in"
                            | "for"
                            | "do"
                            | "switch"
                            | "case"
                            | "break"
                            | "continue"
                            | "and"
                            | "or"
                            | "not"
                            | "sizeof"
                            | "true"
                            | "false"
                    )
            })
            .collect();
        if let (Some(a), Some(b)) = (ids.first(), ids.last()) {
            out.push((
                (a.to_ascii_lowercase(), b.to_ascii_lowercase()),
                expr.trim().to_owned(),
                op,
                s[..i].bytes().filter(|b| *b == b'\n').count(),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceFile;
    #[test]
    fn clear_mask_value_mismatch_and_recase() {
        let f=SourceFile::new("sdk/lib/rust/a.rs".into(),"const KEY_WOW64_RES: u32 = 0x300;\npub fn f(x:u32){let DesiredAccess=x & !0x100u32;}\n// DesiredAccess &= ~KEY_WOW64_RES;\n".into());
        let c = Corpus::new(".".into(), vec![f]);
        let got = MaskDrift.run(&c).unwrap();
        assert!(
            got.iter()
                .any(|x| x.contains("DesiredAccess &= ~KEY_WOW64_RES [MISMATCH]")),
            "{got:?}"
        );
    }
    #[test]
    fn dropped_mask_and_nolint() {
        let f = SourceFile::new(
            "sdk/lib/rust/a.rs".into(),
            "pub fn f(){ }\n// Flags &= ~SOME_FLAG;\n".into(),
        );
        let c = Corpus::new(".".into(), vec![f]);
        assert!(MaskDrift
            .run(&c)
            .unwrap()
            .iter()
            .any(|x| x.contains("DROPPED")));
    }
    #[test]
    fn original_scanner_control_bad_and_good() {
        let bad = r#"const KEY_WOW64_RES: ACCESS_MASK = 0x00000300;
const CR4_PGE: ULONG = 0x00000080;
pub unsafe fn NtOpenKey(DesiredAccess:u32)->u32 { let mut DesiredAccess=DesiredAccess & !0x0100u32; DesiredAccess }
pub unsafe fn KeFlushCurrentTb(){let cr3=readcr3();writecr3(cr3);}
// DesiredAccess &= ~KEY_WOW64_RES;
// Cr4 = __readcr4();
// __writecr4(Cr4 & ~CR4_PGE);
"#;
        let f = SourceFile::new("sdk/lib/rust/ntapi.rs".into(), bad.into());
        let got = MaskDrift.run(&Corpus::new(".".into(), vec![f])).unwrap();
        assert!(
            got.iter()
                .any(|x| x.contains("DesiredAccess &= ~KEY_WOW64_RES [MISMATCH]")),
            "{got:?}"
        );
        assert!(
            got.iter().any(|x| x.contains("Cr4 &= ~CR4_PGE [DROPPED]")),
            "{got:?}"
        );
        let good = r#"const KEY_WOW64_RES: ACCESS_MASK = 0x00000300;
const CR4_PGE: ULONG = 0x00000080;
const MEM_EXECUTE_OPTION_VALID_FLAGS: u32 = 0x3F;
pub unsafe fn NtOpenKey(DesiredAccess:u32)->u32 { let mut DesiredAccess=DesiredAccess & !KEY_WOW64_RES; DesiredAccess }
pub unsafe fn KeFlushCurrentTb(){let cr4:ULONG=readcr4();writecr4(cr4 & !CR4_PGE);}
pub unsafe fn MiValidateExecuteOptions(ExecuteOptions:u32)->bool {(ExecuteOptions & !MEM_EXECUTE_OPTION_VALID_FLAGS)!=0}
// DesiredAccess &= ~KEY_WOW64_RES;
// Cr4 = __readcr4();
// __writecr4(Cr4 & ~CR4_PGE);
// if (ExecuteOptions & ~MEM_EXECUTE_OPTION_VALID_FLAGS)
"#;
        let f = SourceFile::new("sdk/lib/rust/ntapi.rs".into(), good.into());
        assert!(MaskDrift
            .run(&Corpus::new(".".into(), vec![f]))
            .unwrap()
            .is_empty());
    }
}
