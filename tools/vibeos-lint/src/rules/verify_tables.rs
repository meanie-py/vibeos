//! Port of tools/verify_tables.py. The scanner is deliberately conservative:
//! unresolved elements never become mismatches.
use crate::big_math;
use crate::const_values::Big;
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::OnceLock;

pub struct VerifyTables;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree(
        "tables-rust",
        &["sdk/lib/rust/ntoskrnl-rust/src"],
        &["rs"],
        &[],
    )
    .with_pruning(&[]),
    SourceSet::tree(
        "tables-c-ntoskrnl",
        &["ntoskrnl"],
        &["c", "h", "cpp", "hpp"],
        &[],
    )
    .with_pruning(&[]),
    SourceSet::tree(
        "tables-c-sdk-include",
        &["sdk/include"],
        &["c", "h", "cpp", "hpp"],
        &[],
    )
    .with_pruning(&[]),
    SourceSet::tree(
        "tables-c-sdk-lib",
        &["sdk/lib"],
        &["c", "h", "cpp", "hpp"],
        &[],
    )
    .with_pruning(&[]),
];
impl Rule for VerifyTables {
    fn name(&self) -> &'static str {
        "verify_tables"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut rust: Vec<_> = c
            .ordered_sources("tables-rust")
            .into_iter()
            .filter(|f| {
                f.path.extension().is_some_and(|x| x == "rs")
                    && !f.path.to_string_lossy().contains("old_translations")
                    && !f.path.to_string_lossy().ends_with(".bak")
            })
            .collect();
        rust.sort_by(|a, b| a.path.cmp(&b.path));
        let cfiles: Vec<_> = [
            "tables-c-ntoskrnl",
            "tables-c-sdk-include",
            "tables-c-sdk-lib",
        ]
        .into_iter()
        .flat_map(|n| c.ordered_sources(n))
        .collect();
        let defines = harvest_defines(&cfiles);
        let mut c_index: HashMap<String, Vec<CArray>> = HashMap::new();
        let mut c_order = Vec::new();
        for f in &cfiles {
            for a in extract_c_arrays(f) {
                if !c_index.contains_key(&a.name) {
                    c_order.push(a.name.clone());
                }
                c_index.entry(a.name.clone()).or_default().push(a);
            }
        }
        let mut norm: HashMap<String, Vec<CArray>> = HashMap::new();
        for name in &c_order {
            let es = &c_index[name];
            norm.entry(normalize(name))
                .or_default()
                .extend(es.iter().cloned());
        }
        let mut tables = Vec::new();
        for f in &rust {
            tables.extend(extract_rust_tables(f));
        }
        let mut matched = 0usize;
        let mut unmatched = 0usize;
        let mut clean = 0usize;
        let mut mismatch = Vec::new();
        let mut unverifiable = Vec::new();
        for rt in &tables {
            let (entries, exact) = if let Some(x) = c_index.get(&rt.name) {
                (x.clone(), true)
            } else if let Some(x) = norm.get(&normalize(&rt.name)) {
                (x.clone(), false)
            } else {
                unmatched += 1;
                continue;
            };
            let rv = resolve_elems(&rt.elems, &defines);
            let mut ranked: Vec<_> = entries
                .iter()
                .map(|e| (rank(&rt.file.path, &e.file.path), e))
                .collect();
            ranked.sort_by_key(|x| x.0);
            let best_rank = ranked[0].0;
            let mut best = None;
            let mut cv = None;
            for (_r, e) in ranked.into_iter().filter(|x| x.0 == best_rank) {
                let parsed = resolve_elems(&split_initializer(&e.body), &defines);
                if parsed.is_some() {
                    best = Some(e.clone());
                    cv = parsed;
                    break;
                }
                if best.is_none() {
                    best = Some(e.clone());
                }
            }
            let ce = best.unwrap();
            matched += 1;
            let (Some(rv), Some(cv)) = (rv, cv) else {
                let mut reason = Vec::new();
                if resolve_elems(&rt.elems, &defines).is_none() {
                    reason.push(format!(
                        "rust unresolved: {}",
                        py_list(&unresolved(&rt.elems, &defines))
                    ));
                }
                if resolve_elems(&split_initializer(&ce.body), &defines).is_none() {
                    reason.push(format!(
                        "c unresolved: {}",
                        py_list(&unresolved(&split_initializer(&ce.body), &defines))
                    ));
                }
                unverifiable.push((rt, ce, reason.join("; ")));
                continue;
            };
            let mut diffs = Vec::new();
            for i in 0..rv.len().max(cv.len()) {
                let a = rv.get(i);
                let b = cv.get(i);
                if a != b {
                    diffs.push((i, a.cloned(), b.cloned()));
                }
            }
            if rv.len() != cv.len() || !diffs.is_empty() {
                if !exact
                    && rv.len() == cv.len()
                    && !diffs.is_empty()
                    && diffs
                        .iter()
                        .all(|(_, a, b)| ascii_case_flip(a.as_ref(), b.as_ref()))
                {
                    let reason=format!("fuzzy name-match + diffs are pure case-flips (likely UNRELATED C array '{}')",ce.name);
                    unverifiable.push((rt, ce, reason));
                    continue;
                }
                mismatch.push((rt, ce, rv, cv, diffs, exact));
            } else {
                clean += 1;
            }
        }
        let mut out = vec![
            "".into(),
            "=".repeat(78),
            "DATA-TABLE DIFFER REPORT".into(),
            "=".repeat(78),
            format!("Rust integer tables found : {}", tables.len()),
            format!("  matched to a C array    : {matched}"),
            format!("  no C counterpart        : {unmatched}"),
            format!("  -> clean (identical)    : {clean}"),
            format!("  -> MISMATCH             : {}", mismatch.len()),
            format!("  -> unverifiable         : {}", unverifiable.len()),
            "".into(),
        ];
        if !mismatch.is_empty() {
            out.push("#".repeat(78));
            out.push(format!(
                "# {} MISMATCHED TABLE(S) — possible fabrication",
                mismatch.len()
            ));
            out.push("#".repeat(78));
            for (rt, ce, rv, cv, diffs, exact) in mismatch {
                out.push(String::new());
                out.push(format!(
                    "  TABLE: {}   (elem {})   [{} match]",
                    rt.name,
                    rt.elem_type,
                    if exact { "exact-name" } else { "fuzzy-name" }
                ));
                out.push(format!(
                    "    Rust: {}:{}   len={}",
                    rt.file.path.display(),
                    rt.line,
                    rv.len()
                ));
                out.push(format!(
                    "    C   : {}:{}   len={}  (C name: {})",
                    ce.file.path.display(),
                    ce.line,
                    cv.len(),
                    ce.name
                ));
                if rv.len() != cv.len() {
                    out.push(format!(
                        "    !! LENGTH MISMATCH: rust={} c={}",
                        rv.len(),
                        cv.len()
                    ));
                }
                out.push(format!("    differing elements: {}", diffs.len()));
                for (i, a, b) in diffs.iter().take(8) {
                    out.push(format!(
                        "      [{i}] rust={}  c={}",
                        a.as_ref().map(py_hex).unwrap_or_else(|| "MISSING".into()),
                        b.as_ref().map(py_hex).unwrap_or_else(|| "MISSING".into())
                    ));
                }
                if diffs.len() > 8 {
                    out.push(format!("      ... +{} more", diffs.len() - 8));
                }
            }
            out.push(String::new());
        }
        if !unverifiable.is_empty() {
            out.push(format!(
                "({} unverifiable tables not shown; use --verbose)",
                unverifiable.len()
            ));
        }
        Ok(out)
    }
}
#[derive(Clone)]
struct RustTable<'a> {
    name: String,
    elem_type: String,
    file: &'a SourceFile,
    line: usize,
    elems: Vec<String>,
}
#[derive(Clone)]
struct CArray<'a> {
    name: String,
    body: String,
    file: &'a SourceFile,
    line: usize,
}
fn re(p: &'static str) -> &'static Regex {
    static M: OnceLock<BTreeMap<&'static str, Regex>> = OnceLock::new();
    M.get_or_init(||[r"\b(?:pub\s+)?(?:static|const)\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:\s*\[\s*([A-Za-z_]\w*)\s*;\s*([0-9A-Za-z_]+)\s*\]\s*=\s*\[",r"(?:^|[\n;}])\s*(?:(?:const|static|extern|volatile|unsigned|signed)\s+)*([A-Za-z_]\w*)\s+([A-Za-z_]\w*)\s*\[[^\];=]*\]\s*=\s*\{",r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+?)\s*(?:/\*.*)?$",r"^b?'(.*)'$",r"^0[xX][0-9a-fA-F]+$",r"^0[bB][01]+$",r"^[+-]?[0-9]+$",r"^[A-Za-z_]\w*$"].into_iter().map(|p|(p,crate::regexes::cached(p).unwrap())).collect()).get(p).unwrap()
}
const TYPES: &[&str] = &[
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "UCHAR",
    "USHORT",
    "ULONG",
    "ULONG64",
    "ULONG_PTR",
    "ULONGLONG",
    "CHAR",
    "SHORT",
    "LONG",
    "LONG64",
    "LONGLONG",
    "BYTE",
    "WORD",
    "DWORD",
    "WCHAR",
    "wchar_t",
    "UINT8",
    "UINT16",
    "UINT32",
    "UINT64",
    "INT8",
    "INT16",
    "INT32",
    "INT64",
    "SIZE_T",
];
fn extract_rust_tables<'a>(f: &'a SourceFile) -> Vec<RustTable<'a>> {
    let mut o = Vec::new();
    for m in re(r"\b(?:pub\s+)?(?:static|const)\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:\s*\[\s*([A-Za-z_]\w*)\s*;\s*([0-9A-Za-z_]+)\s*\]\s*=\s*\[").captures_iter(&f.text){if !TYPES.contains(&m[2].as_ref()){continue;}let start=m.get(0).unwrap().end()-1;if let Some((body,_))=extract_balanced(&f.text,start,'[',']'){let line=f.line(m.get(0).unwrap().start());o.push(RustTable{name:m[1].into(),elem_type:m[2].into(),file:f,line,elems:split_initializer(body)});}}
    o
}
fn extract_c_arrays<'a>(f: &'a SourceFile) -> Vec<CArray<'a>> {
    let mut out = Vec::new();
    for m in re(r"(?:^|[\n;}])\s*(?:(?:const|static|extern|volatile|unsigned|signed)\s+)*([A-Za-z_]\w*)\s+([A-Za-z_]\w*)\s*\[[^\];=]*\]\s*=\s*\{").captures_iter(&f.text){if !TYPES.contains(&m[1].as_ref())&&!matches!(m[1].to_string().as_str(),"unsigned"|"int"|"char"|"short"|"long"){continue;}let st=m.get(0).unwrap().end()-1;if let Some((body,_))=extract_balanced(&f.text,st,'{','}'){out.push(CArray{name:m[2].into(),body:body.into(),file:f,line:f.line(m.get(0).unwrap().start())});}}
    out
}
fn extract_balanced(s: &str, start: usize, open: char, close: char) -> Option<(&str, usize)> {
    let mut d = 0;
    for (off, c) in s[start..].char_indices() {
        if c == open {
            d += 1;
        } else if c == close {
            d -= 1;
            if d == 0 {
                return Some((&s[start + 1..start + off], start + off + 1));
            }
        }
    }
    None
}
fn split_initializer(s: &str) -> Vec<String> {
    let no = crate::regexes::cached(r"/\*.*?\*/|//[^\n]*")
        .unwrap()
        .replace_all(s, " ")
        .into_owned();
    let mut d = 0i32;
    let mut start = 0;
    let mut out = Vec::new();
    for (i, c) in no.char_indices() {
        match c {
            '(' | '[' | '{' => d += 1,
            ')' | ']' | '}' => d -= 1,
            ',' if d == 0 => {
                let e = no[start..i].trim();
                if !e.is_empty() {
                    out.push(e.to_owned());
                }
                start = i + 1
            }
            _ => {}
        }
    }
    let e = no[start..].trim();
    if !e.is_empty() {
        out.push(e.into());
    }
    out
}
fn harvest_defines<'a>(fs: &[&'a SourceFile]) -> BTreeMap<String, String> {
    let mut d = BTreeMap::new();
    for f in fs {
        if !f
            .path
            .extension()
            .is_some_and(|e| matches!(e.to_str(), Some("c" | "h" | "hpp")))
        {
            continue;
        }
        for l in f.text.lines() {
            if let Some(m) =
                re(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+?)\s*(?:/\*.*)?$").captures(l)
            {
                let name = &m[1];
                let after = l.split_once(name).map(|x| x.1).unwrap_or("");
                if after.starts_with('(') {
                    continue;
                }
                d.entry(name.into()).or_insert(m[2].trim().into());
            }
        }
    }
    d
}
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}
fn arch(p: &Path) -> Option<&str> {
    p.components()
        .filter_map(|c| c.as_os_str().to_str())
        .find(|x| ["arm64", "amd64", "i386", "arm", "x86", "x64", "aarch64"].contains(x))
}
fn rank(a: &Path, b: &Path) -> u8 {
    let (x, y) = (arch(a), arch(b));
    let mut n = 0;
    if let (Some(x), Some(y)) = (x, y) {
        if x != y {
            n += 100;
        }
    } else if x.is_some() || y.is_some() {
        n += 10;
    }
    if a.parent().and_then(Path::file_name) != b.parent().and_then(Path::file_name) {
        n += 1;
    }
    n
}
fn ascii_case_flip(a: Option<&Big>, b: Option<&Big>) -> bool {
    let (Some(a), Some(b)) = (a, b) else {
        return false;
    };
    if a == b {
        return true;
    }
    let (Ok(x), Ok(y)) = (a.decimal().parse::<u8>(), b.decimal().parse::<u8>()) else {
        return false;
    };
    x.is_ascii_alphabetic()
        && y.is_ascii_alphabetic()
        && x.to_ascii_lowercase() == y.to_ascii_lowercase()
}
fn resolve_elems(es: &[String], defs: &BTreeMap<String, String>) -> Option<Vec<Big>> {
    let mut out = Vec::new();
    for e in es {
        out.push(eval(e, defs, 0)?);
    }
    Some(out)
}
fn unresolved(es: &[String], defs: &BTreeMap<String, String>) -> Vec<String> {
    es.iter()
        .filter(|e| eval(e, defs, 0).is_none())
        .take(3)
        .cloned()
        .collect()
}
fn py_list(v: &[String]) -> String {
    format!(
        "[{}]",
        v.iter().map(|s| py_repr(s)).collect::<Vec<_>>().join(", ")
    )
}
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(quote);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}
fn eval(raw: &str, defs: &BTreeMap<String, String>, depth: u8) -> Option<Big> {
    if depth > 40 {
        return None;
    }
    let mut s = raw.trim().to_owned();
    s = crate::regexes::cached(r"/\*.*?\*/")
        .ok()?
        .replace_all(&s, " ")
        .into_owned();
    s = crate::regexes::cached(r"//.*$")
        .ok()?
        .replace(&s, "")
        .trim()
        .to_owned();
    if s.is_empty() {
        return None;
    }
    if let Some(i) = s.rfind(" as ") {
        if s[i + 4..]
            .trim()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return eval(&s[..i], defs, depth + 1);
        }
    }
    if s.starts_with('(') {
        if let Some((inside, end)) = extract_balanced(&s, 0, '(', ')') {
            if end == s.len() {
                return eval(inside, defs, depth + 1);
            }
            if let Some(v) = eval(&s[end..], defs, depth + 1) {
                return Some(v);
            }
        }
    }
    for op in ["|", "+", "<<", "*"] {
        let pieces = split_top(&s, op);
        if pieces.len() > 1 {
            let vals = pieces
                .iter()
                .map(|x| eval(x, defs, depth + 1))
                .collect::<Option<Vec<_>>>()?;
            let mut acc = vals[0].clone();
            for v in vals.iter().skip(1) {
                acc = match op {
                    "|" => big_math::bitor(&acc, v),
                    "+" => big_math::add(&acc, v),
                    "<<" => big_math::shl(&acc, v).ok()?,
                    "*" => big_math::mul(&acc, v),
                    _ => return None,
                };
            }
            return Some(acc);
        }
    }
    if s.starts_with('(') {
        if let Some(i) = s.find(')') {
            let typ = s[1..i].trim();
            if typ.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return eval(&s[i + 1..], defs, depth + 1);
            }
        }
    }
    if s.starts_with('b') || s.starts_with('\'') {
        if let Some(c) = re(r"^b?'(.*)'$").captures(&s) {
            let t = &c[1];
            let n = if t.chars().count() == 1 {
                t.chars().next()? as u32
            } else if let Some(h) = t.strip_prefix("\\x") {
                u32::from_str_radix(h, 16).ok()?
            } else {
                match t {
                    "\\0" => 0,
                    "\\n" => 10,
                    "\\r" => 13,
                    "\\t" => 9,
                    "\\\\" => 92,
                    "\\'" => 39,
                    "\\\"" => 34,
                    _ => return None,
                }
            };
            return Some(Big::parse(&n.to_string())?);
        }
    }
    let t = s.replace('_', "");
    let cleaned = crate::regexes::cached(r"(?i)(u|l)+$")
        .ok()?
        .replace(&t, "")
        .to_string();
    if re(r"^0[bB][01]+$").is_match(&cleaned) {
        let mut value = Big::zero();
        for bit in cleaned[2..].bytes() {
            value = big_math::mul(&value, &Big::parse("2")?);
            if bit == b'1' {
                value = big_math::add(&value, &Big::parse("1")?);
            }
        }
        return Some(value);
    }
    if re(r"^0[xX][0-9a-fA-F]+$").is_match(&cleaned) {
        return Big::parse(&cleaned);
    }
    if re(r"^[+-]?[0-9]+$").is_match(&cleaned) {
        // Python's int(token) branch interprets every decimal digit sequence as base 10,
        // including C spellings with a leading zero such as `01`. Big::parse uses the
        // repository's base-0 parser, so canonicalize decimal digits before passing it on.
        let (sign, digits) = if let Some(x) = cleaned.strip_prefix('-') {
            ("-", x)
        } else if let Some(x) = cleaned.strip_prefix('+') {
            ("", x)
        } else {
            ("", cleaned.as_str())
        };
        let digits = digits.trim_start_matches('0');
        let digits = if digits.is_empty() { "0" } else { digits };
        return Big::parse(&format!("{sign}{digits}"));
    }
    if re(r"^[A-Za-z_]\w*$").is_match(&s) {
        return eval(defs.get(&s)?, defs, depth + 1);
    }
    None
}
fn split_top(s: &str, op: &str) -> Vec<String> {
    let mut d = 0i32;
    let mut out = Vec::new();
    let mut st = 0;
    let mut iter = s.char_indices().peekable();
    while let Some((i, ch)) = iter.next() {
        match ch {
            '(' | '[' => d += 1,
            ')' | ']' => d -= 1,
            _ => {}
        }
        if d == 0 && s[i..].starts_with(op) && (op != "|" || !s[i..].starts_with("||")) {
            out.push(s[st..i].into());
            st = i + op.len();
            while iter.peek().is_some_and(|(at, _)| *at < st) {
                iter.next();
            }
        } else if d == 0 && op == "|" && s[i..].starts_with("||") {
            iter.next();
        }
    }
    if st == 0 {
        vec![s.into()]
    } else {
        out.push(s[st..].into());
        out
    }
}
fn py_hex(v: &Big) -> String {
    let s = v.decimal();
    let (sign, mag) = s.strip_prefix('-').map_or(("", s.as_str()), |x| ("-", x));
    let mut d = mag.bytes().map(|x| x - b'0').collect::<Vec<_>>();
    let mut h = Vec::new();
    while d.iter().any(|x| *x != 0) {
        let mut carry = 0u16;
        for x in &mut d {
            let n = carry * 10 + *x as u16;
            *x = (n / 16) as u8;
            carry = n % 16;
        }
        h.push(char::from_digit(carry as u32, 16).unwrap());
        while d.len() > 1 && d[0] == 0 {
            d.remove(0);
        }
    }
    if h.is_empty() {
        h.push('0');
    }
    format!("{s}(0x{sign}{})", h.iter().rev().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evaluator_supports_arbitrary_python_integer_ops() {
        let d = BTreeMap::new();
        assert_eq!(
            eval("0x100000000000000000000 | 3", &d, 0)
                .unwrap()
                .decimal(),
            "1208925819614629174706179"
        );
        assert_eq!(
            eval("(1 + 2) << 70", &d, 0).unwrap().decimal(),
            "3541774862152233910272"
        );
        assert_eq!(eval("b'\\x41'", &d, 0).unwrap().decimal(), "65");
        assert_eq!(eval("0b10101", &d, 0).unwrap().decimal(), "21");
        assert_eq!(eval("01", &d, 0).unwrap().decimal(), "1");
        assert_eq!(eval("-0012", &d, 0).unwrap().decimal(), "-12");
        assert!(eval("+000", &d, 0).is_none());
        assert!(eval("A || B", &d, 0).is_none());
        assert_eq!(eval("'λ'", &d, 0).unwrap().decimal(), "955");
    }
    #[test]
    fn splits_nested_elements_and_resolves_defines() {
        let d = BTreeMap::from([("A".into(), "0x10".into())]);
        assert_eq!(split_initializer("1, (2 | 4), A,").len(), 3);
        assert_eq!(eval("A | 3", &d, 0).unwrap().decimal(), "19");
    }
    #[test]
    fn detects_rust_and_c_declarations() {
        let f = SourceFile::new("x.rs".into(), "pub static T: [u8; 2] = [1, 2];".into());
        assert_eq!(extract_rust_tables(&f).len(), 1);
    }
    #[test]
    fn python_hex_rendering_and_case_flip_guard_match_oracle() {
        assert_eq!(py_hex(&Big::parse("-26").unwrap()), "-26(0x-1a)");
        assert!(ascii_case_flip(
            Some(&Big::parse("65").unwrap()),
            Some(&Big::parse("97").unwrap())
        ));
        assert!(!ascii_case_flip(
            Some(&Big::parse("65").unwrap()),
            Some(&Big::parse("98").unwrap())
        ));
    }
    #[test]
    fn reports_only_resolved_table_differences() {
        let files = vec![
            SourceFile::new(
                "sdk/lib/rust/ntoskrnl-rust/src/tables.rs".into(),
                "pub static GOOD: [u8; 2] = [1, 2];\npub static BAD: [u8; 2] = [1, 9];".into(),
            ),
            SourceFile::new(
                "ntoskrnl/tables.c".into(),
                "static UCHAR GOOD[] = {1, 2};\nstatic UCHAR BAD[] = {1, 2};".into(),
            ),
        ];
        let corpus = Corpus::new("/repo".into(), files);
        let lines = VerifyTables.run(&corpus).unwrap();
        assert!(lines
            .iter()
            .any(|l| l == "# 1 MISMATCHED TABLE(S) — possible fabrication"));
        assert!(lines.iter().any(|l| l.contains("TABLE: BAD")));
        assert!(!lines.iter().any(|l| l.contains("TABLE: GOOD")));
    }
}
