//! Faithful port of `tools/wow64gen/marshalgen.py audit` and its source-derived
//! wow64gen table helpers. The audit deliberately invokes clang: only its target
//! record-layout dump is authoritative for the active 32/64 header configuration.
use crate::{Corpus, Rule};
use regex::{Captures, Regex};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub struct MarshalgenAudit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub offset: usize,
    pub depth: usize,
    pub ctype: String,
    pub name: String,
    pub is_bitfield: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub size: usize,
    pub align: usize,
    pub fields: Vec<Field>,
}
impl Record {
    fn has_union(&self) -> bool {
        self.name.starts_with("union") || self.fields.iter().any(|f| f.ctype.starts_with("union"))
    }
    fn has_bitfield(&self) -> bool {
        self.fields.iter().any(|f| f.is_bitfield)
    }
    fn top_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.depth == 1).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub ctype: String,
    pub name: String,
    pub sal: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proto {
    pub name: String,
    pub ret: String,
    pub params: Vec<Param>,
}
#[derive(Clone, Debug)]
pub struct StructVerdict {
    pub cls: String,
    pub detail: String,
    pub rec32: Option<Record>,
    pub rec64: Option<Record>,
}

fn re(pat: &'static str) -> &'static Regex {
    static CACHE: OnceLock<std::sync::Mutex<HashMap<&'static str, &'static Regex>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    if let Some(r) = cache.lock().unwrap().get(pat).copied() {
        return r;
    }
    let leaked = Box::leak(Box::new(
        crate::regexes::cached(pat).expect("constant marshalgen regex"),
    ));
    cache.lock().unwrap().insert(pat, leaked);
    leaked
}

pub fn parse_layout_dump(text: &str) -> HashMap<String, Record> {
    let hdr = re(r"^\s*0 \| ((?:struct|union|class) .+)$");
    let foot = re(r"^\s*\| \[sizeof=(\d+), align=(\d+)");
    let field = re(r"^\s*(\d+)(:\d+-\d+)? \| (  +)(.+)$");
    let member = re(r"[A-Za-z_]\w*(?:\[\d+\])*$");
    let mut records = HashMap::new();
    let mut cur: Option<Record> = None;
    for line in text.lines() {
        if line.starts_with("*** Dumping AST Record Layout") {
            cur = None;
            continue;
        }
        if let Some(m) = hdr.captures(line) {
            if cur.is_none() {
                cur = Some(Record {
                    name: m[1].trim().to_owned(),
                    size: 0,
                    align: 0,
                    fields: vec![],
                });
                continue;
            }
        }
        let Some(r) = cur.as_mut() else { continue };
        if let Some(m) = foot.captures(line) {
            r.size = m[1].parse().unwrap_or(0);
            r.align = m[2].parse().unwrap_or(0);
            records.insert(r.name.clone(), r.clone());
            cur = None;
            continue;
        }
        if let Some(m) = field.captures(line) {
            let decl = &m[4];
            let pieces: Vec<_> = decl.rsplitn(2, ' ').collect();
            let (ctype, name) = if pieces.len() == 2 && member.is_match(pieces[0]) {
                (pieces[1].trim().to_owned(), pieces[0].to_owned())
            } else {
                (decl.to_owned(), String::new())
            };
            r.fields.push(Field {
                offset: m[1].parse().unwrap_or(0),
                depth: m[3].len() / 2,
                ctype,
                name,
                is_bitfield: m.get(2).is_some(),
            });
        }
    }
    records
}

fn ident_re() -> &'static Regex {
    re(r"[A-Za-z_][A-Za-z0-9_]*")
}
fn ann_re() -> &'static Regex {
    re(r"_[A-Z][A-Za-z0-9_]*_$|__[a-z][a-z0-9_]*$")
}
pub fn strip_annotations(raw: &str) -> (String, String) {
    let mut out = String::new();
    let mut i = 0;
    let b = raw.as_bytes();
    let mut sal = String::new();
    while i < b.len() {
        if let Some(m) = ident_re().find_at(raw, i).filter(|m| m.start() == i) {
            let tok = m.as_str();
            if ann_re().is_match(tok) {
                let low = tok.to_ascii_lowercase();
                if low.contains("inout") || low.contains("updates") {
                    sal = "inout".into()
                } else if low.contains("out") || low.contains("writes") {
                    if sal.is_empty() {
                        sal = "out".into()
                    }
                } else if low.contains("in") || low.contains("reads") {
                    if sal.is_empty() {
                        sal = "in".into()
                    }
                }
                i = m.end();
                while i < b.len() && b[i].is_ascii_whitespace() {
                    i += 1
                }
                if i < b.len() && b[i] == b'(' {
                    let mut d = 0isize;
                    while i < b.len() {
                        if b[i] == b'(' {
                            d += 1
                        } else if b[i] == b')' {
                            d -= 1;
                            if d == 0 {
                                i += 1;
                                break;
                            }
                        }
                        i += 1
                    }
                }
            } else {
                out.push_str(tok);
                i = m.end()
            }
        } else {
            let ch = raw[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8()
        }
    }
    (out, sal)
}
pub fn strip_comments(text: &str) -> String {
    let block = re(r"(?s)/\*.*?\*/")
        .replace_all(text, |c: &Captures<'_>| {
            "\n".repeat(c[0].bytes().filter(|b| *b == b'\n').count())
        })
        .into_owned();
    re(r"//[^\n]*").replace_all(&block, "").into_owned()
}
fn split_params(s: &str) -> Vec<String> {
    let (mut out, mut d, mut cur) = (vec![], 0isize, String::new());
    for ch in s.chars() {
        if ch == '(' {
            d += 1
        } else if ch == ')' {
            d -= 1
        }
        if ch == ',' && d == 0 {
            out.push(std::mem::take(&mut cur))
        } else {
            cur.push(ch)
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur)
    }
    out
}
pub fn extract_prototypes(text: &str, name_pat: &str) -> Result<HashMap<String, Proto>, String> {
    let text = strip_comments(text);
    let pat = re_dynamic(&format!(r"\b({name_pat})\s*\("))?;
    let mut protos = HashMap::new();
    let ann_skip: HashSet<&str> = [
        "NTSYSCALLAPI",
        "NTSYSAPI",
        "W32KAPI",
        "APIENTRY",
        "NTAPI",
        "WINAPI",
        "__stdcall",
        "__kernel_entry",
        "DECLSPEC_ADDRSAFE",
    ]
    .into_iter()
    .collect();
    let old = [
        ("INOUT", "inout"),
        ("IN", "in"),
        ("OUT", "out"),
        ("OPTIONAL", ""),
    ];
    let parse_param = re(r"(?s)(.+?)\s*(\**)?\s*([A-Za-z_]\w*)(\[\s*\d*\s*\])?$");
    for m in pat.find_iter(&text) {
        let name = m.as_str()[..m.as_str().len() - 1]
            .trim_end()
            .trim_end_matches('(')
            .trim()
            .to_owned();
        let name = name.split_whitespace().last().unwrap_or(&name).to_owned();
        let mut depth = 1;
        let mut i = m.end();
        let bytes = text.as_bytes();
        while i < bytes.len() && depth > 0 {
            if bytes[i] == b'(' {
                depth += 1
            } else if bytes[i] == b')' {
                depth -= 1
            }
            i += 1
        }
        if depth > 0 || i >= bytes.len() || bytes[i] != b';' {
            continue;
        }
        let arglist = text[m.end()..i - 1].to_owned();
        let pre = text[..m.start()].rsplit(';').next().unwrap_or("");
        let toks: Vec<_> = pre
            .split_whitespace()
            .filter(|t| !ann_skip.contains(*t))
            .collect();
        let ret = toks.last().copied().unwrap_or("?").to_owned();
        let mut params = vec![];
        if !matches!(arglist.trim(), "" | "VOID" | "void") {
            for raw in split_params(&arglist) {
                let (mut clean, mut sal) = strip_annotations(&raw);
                for (tok, d) in old {
                    let r = re(if tok == "INOUT" {
                        r"\bINOUT\b"
                    } else if tok == "IN" {
                        r"\bIN\b"
                    } else if tok == "OUT" {
                        r"\bOUT\b"
                    } else {
                        r"\bOPTIONAL\b"
                    });
                    if r.is_match(&clean) {
                        if sal.is_empty() {
                            sal = d.into()
                        }
                        clean = r.replace_all(&clean, " ").into_owned()
                    }
                }
                clean = re(r"\s+").replace_all(&clean, " ").trim().to_owned();
                if let Some(pm) = parse_param.captures(&clean) {
                    let stars = pm.get(2).map(|x| x.as_str()).unwrap_or("");
                    let mut ty = format!(
                        "{}{}{}",
                        pm[1].trim_end(),
                        if stars.is_empty() { "" } else { " " },
                        stars
                    )
                    .trim()
                    .to_owned();
                    if pm.get(4).is_some() {
                        ty.push_str(" *")
                    }
                    params.push(Param {
                        ctype: ty,
                        name: pm[3].into(),
                        sal,
                    })
                } else {
                    params.push(Param {
                        ctype: clean,
                        name: String::new(),
                        sal,
                    })
                }
            }
        }
        protos.insert(name.clone(), Proto { name, ret, params });
    }
    Ok(protos)
}
fn re_dynamic(p: &str) -> Result<Regex, String> {
    crate::regexes::cached(p).map_err(|e| format!("invalid marshalgen regex: {e}"))
}

const DIVERGENT: &[&str] = &[
    "SIZE_T",
    "PSIZE_T",
    "ULONG_PTR",
    "LONG_PTR",
    "SSIZE_T",
    "INT_PTR",
    "UINT_PTR",
    "DWORD_PTR",
    "HANDLE",
    "PVOID",
    "LPVOID",
    "WPARAM",
    "LPARAM",
    "LRESULT",
    "KAFFINITY",
    "HWND",
    "HMENU",
    "HICON",
    "HCURSOR",
    "HBRUSH",
    "HDC",
    "HBITMAP",
    "HRGN",
    "HFONT",
    "HPALETTE",
    "HMODULE",
    "HINSTANCE",
    "HDESK",
    "HWINSTA",
    "HHOOK",
    "HKL",
    "HMONITOR",
    "HRAWINPUT",
    "HIMC",
];
const SAFE: &[&str] = &[
    "VOID",
    "CHAR",
    "UCHAR",
    "BOOLEAN",
    "BYTE",
    "WORD",
    "USHORT",
    "SHORT",
    "WCHAR",
    "INT",
    "UINT",
    "LONG",
    "ULONG",
    "DWORD",
    "BOOL",
    "FLOAT",
    "LONGLONG",
    "ULONGLONG",
    "LARGE_INTEGER",
    "ULARGE_INTEGER",
    "DWORD64",
    "UINT64",
    "INT64",
    "NTSTATUS",
    "ACCESS_MASK",
    "COLORREF",
    "LCID",
    "LANGID",
    "ATOM",
    "HRESULT",
    "int",
    "unsigned",
    "long",
    "short",
    "char",
    "void",
    "float",
    "double",
    "unsigned int",
    "unsigned long",
    "unsigned short",
    "unsigned char",
    "signed char",
    "long long",
    "unsigned long long",
    "wchar_t",
    "__int64",
];
const TAGS: &[(&str, &str)] = &[
    ("DEVMODEW", "struct _devicemodeW"),
    ("DEVMODEA", "struct _devicemodeA"),
    (
        "SECURITY_QUALITY_OF_SERVICE",
        "struct _SECURITY_QUALITY_OF_SERVICE",
    ),
];
fn norm_type(s: &str) -> String {
    let t = re(r"/\*.*?\*/").replace_all(s, " ");
    t.split_whitespace()
        .filter(|x| {
            !matches!(
                *x,
                "IN" | "OUT"
                    | "OPTIONAL"
                    | "CONST"
                    | "const"
                    | "FAR"
                    | "NEAR"
                    | "volatile"
                    | "__unaligned"
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn resolve_exact(ctype: &str, db: &HashMap<String, Record>) -> Option<Record> {
    let t = re(r"\s*(\*+|\[\d*\])\s*$")
        .replace(&norm_type(ctype), "")
        .trim()
        .to_owned();
    if t.starts_with("struct ") || t.starts_with("union ") || t.starts_with("class ") {
        return db.get(&t).cloned();
    }
    for (alias, tag) in TAGS {
        if t == *alias {
            return db.get(*tag).cloned();
        }
    }
    for kind in ["struct", "union"] {
        for tag in [
            format!("{kind} _{t}"),
            format!("{kind} tag{t}"),
            format!("{kind} _tag{t}"),
            format!("{kind} {t}"),
        ] {
            if let Some(r) = db.get(&tag) {
                return Some(r.clone());
            }
        }
    }
    None
}
pub fn resolve_pointee(ctype: &str, db: &HashMap<String, Record>) -> Option<Record> {
    let t = norm_type(ctype);
    if t.contains('*') {
        return resolve_exact(&t, db);
    }
    for p in ["LPC", "PC", "LP", "P"] {
        if let Some(s) = t.strip_prefix(p) {
            if let Some(r) = resolve_exact(s, db) {
                return Some(r);
            }
        }
    }
    None
}
fn scalar_base(t: &str) -> &str {
    for p in ["LPC", "PC", "LP", "P"] {
        if let Some(s) = t.strip_prefix(p) {
            return s;
        }
    }
    t
}
fn is_scalar(list: &[&str], t: &str) -> bool {
    list.contains(&t)
}
fn field_class(
    ctype: &str,
    d32: &HashMap<String, Record>,
    d64: &HashMap<String, Record>,
    seen: &mut HashSet<String>,
) -> (String, String) {
    let t = re(r"\[\d*\]$")
        .replace(&norm_type(ctype), "")
        .trim()
        .to_owned();
    if resolve_exact(&t, d32).is_some() {
        let sub = diff_struct_inner(&t, d32, d64, seen);
        if sub.cls == "IDENTICAL" {
            return ("SAME".into(), String::new());
        }
        if sub.cls == "WIDENING" {
            if ctype.trim_end().ends_with(']') {
                return ("SEMANTIC".into(), format!("array of divergent {t}"));
            }
            return ("WIDEN".into(), format!("nested {t}"));
        }
        return ("SEMANTIC".into(), format!("nested {t}: {}", sub.detail));
    }
    if t.contains('*') {
        if let Some(pr) = resolve_pointee(&t, d32) {
            let sub = diff_struct_inner(&pr.name, d32, d64, seen);
            if sub.cls == "SEMANTIC" {
                return (
                    "SEMANTIC".into(),
                    format!("ptr to divergent {}: {}", pr.name, sub.detail),
                );
            }
            if sub.cls == "WIDENING" {
                return (
                    "WIDEN".into(),
                    format!("ptr to divergent {} (deep)", pr.name),
                );
            }
        }
        return ("WIDEN".into(), String::new());
    }
    if is_scalar(SAFE, &t)
        || matches!(
            t.as_str(),
            "unsigned int" | "unsigned long" | "unsigned short" | "unsigned char"
        )
    {
        return ("SAME".into(), String::new());
    }
    if is_scalar(DIVERGENT, &t) {
        return ("WIDEN".into(), String::new());
    }
    let sb = scalar_base(&t);
    if is_scalar(SAFE, sb) || ["STR", "CSTR", "WSTR", "CWSTR", "TSTR", "CTSTR"].contains(&sb) {
        return ("WIDEN".into(), String::new());
    }
    if is_scalar(DIVERGENT, sb) {
        return ("WIDEN".into(), format!("ptr to {sb} (deep scalar)"));
    }
    if let Some(pr) = resolve_pointee(&t, d32) {
        let sub = diff_struct_inner(&pr.name, d32, d64, seen);
        if sub.cls == "SEMANTIC" {
            return (
                "SEMANTIC".into(),
                format!("ptr to divergent {}: {}", pr.name, sub.detail),
            );
        }
        if sub.cls == "WIDENING" {
            return (
                "WIDEN".into(),
                format!("ptr to divergent {} (deep)", pr.name),
            );
        }
        return ("WIDEN".into(), String::new());
    }
    ("SEMANTIC".into(), format!("unknown field type '{t}'"))
}
pub fn diff_struct(
    name: &str,
    d32: &HashMap<String, Record>,
    d64: &HashMap<String, Record>,
) -> StructVerdict {
    diff_struct_inner(name, d32, d64, &mut HashSet::new())
}
fn diff_struct_inner(
    name: &str,
    d32: &HashMap<String, Record>,
    d64: &HashMap<String, Record>,
    seen: &mut HashSet<String>,
) -> StructVerdict {
    let (Some(r32), Some(r64)) = (resolve_exact(name, d32), resolve_exact(name, d64)) else {
        return StructVerdict {
            cls: "SEMANTIC".into(),
            detail: format!("layout of '{name}' not in dump (add header to TU?)"),
            rec32: None,
            rec64: None,
        };
    };
    if seen.contains(&r32.name) {
        return StructVerdict {
            cls: "IDENTICAL".into(),
            detail: "recursion cut".into(),
            rec32: Some(r32),
            rec64: Some(r64),
        };
    }
    seen.insert(r32.name.clone());
    let f32 = r32.top_fields();
    let f64 = r64.top_fields();
    let layout = |fs: &Vec<&Field>, size| {
        (
            size,
            fs.iter()
                .map(|f| {
                    (
                        if f.is_bitfield {
                            -1isize
                        } else {
                            f.offset as isize
                        },
                        f.name.clone(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    };
    let same = layout(&f32, r32.size) == layout(&f64, r64.size);
    let names = |fs: &Vec<&Field>| fs.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
    if names(&f32) != names(&f64) {
        return StructVerdict {
            cls: "SEMANTIC".into(),
            detail: "field lists differ across widths".into(),
            rec32: Some(r32),
            rec64: Some(r64),
        };
    }
    if r32.has_bitfield() || r64.has_bitfield() || r32.has_union() || r64.has_union() {
        return StructVerdict {
            cls: if same { "IDENTICAL" } else { "SEMANTIC" }.into(),
            detail: if same {
                "same layout (union/bitfield)"
            } else {
                "union/bitfield with divergent layout"
            }
            .into(),
            rec32: Some(r32),
            rec64: Some(r64),
        };
    }
    let span = |fs: &Vec<&Field>, i: usize, total: usize| {
        let next = if i + 1 < fs.len() {
            fs[i + 1].offset
        } else {
            total
        };
        (fs[i].offset, next - fs[i].offset)
    };
    if same {
        return StructVerdict {
            cls: "IDENTICAL".into(),
            detail: String::new(),
            rec32: Some(r32),
            rec64: Some(r64),
        };
    }
    for (i, a) in f32.iter().enumerate() {
        let (_, s32) = span(&f32, i, r32.size);
        let (_, s64) = span(&f64, i, r64.size);
        if s32 == s64 {
            continue;
        }
        if (s32, s64) == (4, 8) {
            if let Some(pr) = resolve_pointee(&a.ctype, d32) {
                let sub = diff_struct_inner(&pr.name, d32, d64, seen);
                if sub.cls == "SEMANTIC" {
                    return StructVerdict {
                        cls: "SEMANTIC".into(),
                        detail: format!(
                            "field {}: ptr to divergent {} ({})",
                            a.name, pr.name, sub.detail
                        ),
                        rec32: Some(r32),
                        rec64: Some(r64),
                    };
                }
            }
            continue;
        }
        let (cls, det) = field_class(&a.ctype, d32, d64, seen);
        if cls == "SEMANTIC" {
            return StructVerdict {
                cls,
                detail: format!("field {} ({s32}->{s64}): {det}", a.name),
                rec32: Some(r32),
                rec64: Some(r64),
            };
        }
    }
    StructVerdict {
        cls: "WIDENING".into(),
        detail: String::new(),
        rec32: Some(r32),
        rec64: Some(r64),
    }
}

pub fn classify_param(
    p: &Param,
    d32: &HashMap<String, Record>,
    d64: &HashMap<String, Record>,
) -> (String, String) {
    let t = norm_type(&p.ctype);
    if resolve_exact(&t, d32).is_some() {
        if !t.contains('*') {
            let v = diff_struct(&t, d32, d64);
            if v.cls == "IDENTICAL" {
                return ("FLAT".into(), format!("by-value {t}, width-identical"));
            }
            return (v.cls, format!("by-value {t}: {}", v.detail));
        }
    }
    if !t.contains('*') && (is_scalar(SAFE, &t) || is_scalar(DIVERGENT, &t)) {
        return ("FLAT".into(), "scalar".into());
    }
    let mut pointee: Option<String> = None;
    if t.contains('*') {
        let base = re(r"\s*\*+$").replace(&t, "").trim().to_owned();
        if is_scalar(SAFE, &base)
            || [
                "void", "char", "wchar_t", "int", "unsigned", "long", "short",
            ]
            .contains(&base.as_str())
        {
            return ("FLAT".into(), format!("-> {base}: width-identical pointee"));
        }
        if is_scalar(DIVERGENT, &base) {
            return (
                "SCALAR_WIDEN".into(),
                format!("-> {base}: 4->8 scalar pointee"),
            );
        }
        pointee = Some(base)
    } else {
        let sb = scalar_base(&t);
        if sb != t {
            if is_scalar(SAFE, sb)
                || ["STR", "CSTR", "WSTR", "CWSTR", "TSTR", "CTSTR"].contains(&sb)
            {
                return ("FLAT".into(), format!("-> {sb}: width-identical pointee"));
            }
            if is_scalar(DIVERGENT, sb) {
                return (
                    "SCALAR_WIDEN".into(),
                    format!("-> {sb}: 4->8 scalar pointee"),
                );
            }
            pointee = Some(sb.into())
        }
    }
    if pointee.is_none() && !t.contains('*') {
        return ("SEMANTIC".into(), format!("unknown type '{t}'"));
    }
    if let Some(pr) = resolve_pointee(&t, d32) {
        let v = diff_struct(&pr.name, d32, d64);
        return (
            v.cls.clone(),
            if v.detail.is_empty() {
                format!("-> {}", pr.name)
            } else {
                format!("-> {}: {}", pr.name, v.detail)
            },
        );
    }
    (
        "SEMANTIC".into(),
        format!(
            "-> {}: layout not in dump (add header to TU?)",
            pointee.unwrap_or_default()
        ),
    )
}
pub fn audit_verdict(cls: &str, detail: &str) -> &'static str {
    if cls == "FLAT" || cls == "IDENTICAL" {
        "OK"
    } else if cls == "SEMANTIC"
        && ["not in dump", "unknown type", "unknown field type"]
            .iter()
            .any(|s| detail.contains(s))
    {
        "UNKNOWN"
    } else {
        "DIVERGENT"
    }
}

#[derive(Clone)]
struct Sc {
    manual: String,
    slots: usize,
}
fn parse_sysfuncs(text: &str) -> Vec<(String, usize)> {
    let (cpp, svc) = (
        re(r"^\s*#\s*(ifdef|ifndef|if|else|elif|endif)\b(.*)$"),
        re(r"^\s*SVC_\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*,\s*(\d+)\s*\)"),
    );
    let w64 = re(r"\b_WIN64\b");
    let mut out = vec![];
    let mut stack: Vec<(bool, bool)> = vec![];
    for line in text.lines() {
        if let Some(m) = cpp.captures(line) {
            let kind = &m[1];
            let rest = &m[2];
            if ["ifdef", "ifndef", "if"].contains(&kind) {
                let has = w64.is_match(rest);
                if !has {
                    stack.push((false, false))
                } else if kind == "ifdef" {
                    stack.push((true, true))
                } else if kind == "ifndef" {
                    stack.push((true, false))
                } else {
                    stack.push((true, !rest.contains('!')))
                }
            } else if kind == "else" {
                if let Some((true, suppress)) = stack.last_mut() {
                    *suppress = !*suppress
                }
            } else if kind == "endif" {
                stack.pop();
            }
            continue;
        }
        if stack.iter().any(|x| x.1) {
            continue;
        }
        if let Some(m) = svc.captures(line) {
            out.push((format!("Nt{}", &m[1]), m[2].parse().unwrap_or(0)))
        }
    }
    out
}
fn compute_thunk_table(
    root: &Path,
    table: &str,
    arch: &str,
) -> Result<HashMap<String, bool>, String> {
    let (spec, sys, lists) = if table == "win32k" {
        (
            "win32ss/win32u/win32u.spec",
            "win32ss/w32ksvc32.h",
            vec![
                "tools/wow64gen/manual-win32k.txt",
                "tools/wow64gen/automarshal-win32k.txt",
            ],
        )
    } else {
        (
            "dll/ntdll/def/ntdll.spec",
            "ntoskrnl/include/sysfuncs.h",
            vec!["tools/wow64gen/manual.txt"],
        )
    };
    let mut manual = HashSet::new();
    for f in lists {
        let t = read(root, f)?;
        for line in t.lines() {
            let name = line.split('#').next().unwrap_or("").trim();
            if !name.is_empty() {
                manual.insert(name.to_owned());
            }
        }
    }
    let spec_text = read(root, spec)?;
    let pat = re(r"^\s*@\s+stdcall\s+(?:-arch=(\S+)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)");
    let mut specs = HashMap::new();
    let spec_arch = if arch == "amd64" { "x86_64" } else { arch };
    for line in spec_text.lines() {
        let Some(m) = pat.captures(line) else {
            continue;
        };
        let name = &m[2];
        if !name.starts_with("Nt") {
            continue;
        }
        let entry = m.get(1).map(|x| x.as_str());
        if let Some(e) = entry {
            let arches: Vec<_> = e.split(',').collect();
            let neg = arches.first().is_some_and(|x| x.starts_with('!'));
            let found = arches
                .iter()
                .any(|x| x.trim_start_matches('!') == spec_arch);
            if (neg && found) || (!neg && !found) {
                continue;
            }
        }
        let args: Vec<_> = m[3].split_whitespace().map(str::to_owned).collect();
        let (mut slots, mut why) = (0, String::new());
        for tok in &args {
            let (n, manual) = match tok.as_str() {
                "long" | "s_long" | "word" | "ptr" | "str" | "wstr" => (1, false),
                "int64" | "double" => (2, false),
                "int128" => (4, true),
                _ => (0, false),
            };
            if n == 0 {
                why = format!("unknown arg token '{tok}'");
                break;
            }
            if manual {
                why = format!("arg '{tok}' needs deep marshalling")
            }
            slots += n
        }
        if manual.contains(name) && why.is_empty() {
            why = "listed in --manual-list (struct layout)".into()
        }
        specs.insert(name.to_owned(), Sc { manual: why, slots });
    }
    let sys = read(root, sys)?;
    let mut result = HashMap::new();
    for (name, argc) in parse_sysfuncs(&sys) {
        let mut sc = specs.get(&name).cloned().unwrap_or_else(|| Sc {
            manual: "in sysfuncs.h but not in .spec (no type info)".into(),
            slots: argc,
        });
        if sc.manual.is_empty() && sc.slots != argc {
            sc.manual = format!("spec arg dwords {} != sysfuncs.h argcount {argc}", sc.slots);
            sc.slots = argc
        }
        result.insert(name, sc.manual.is_empty());
    }
    Ok(result)
}
fn read(root: &Path, rel: &str) -> Result<String, String> {
    fs::read_to_string(root.join(rel)).map_err(|e| {
        format!(
            "marshalgen audit: cannot read {}: {e}",
            root.join(rel).display()
        )
    })
}
fn parse_table_artifact(text: &str) -> HashMap<String, bool> {
    let pat = re(r#"^\s*\{\s*"([^"]+)"\s*,\s*(\w+),\s*\d+\s*\},"#);
    let mut out = HashMap::new();
    for line in text.lines() {
        if let Some(m) = pat.captures(line) {
            out.insert(m[1].to_owned(), &m[2] != "0");
        }
    }
    out
}
fn crosscheck(
    computed: &HashMap<String, bool>,
    gen_text: &str,
    path: &Path,
    tname: &str,
) -> Result<(), String> {
    let built = parse_table_artifact(gen_text);
    if built.is_empty() {
        return Ok(());
    }
    let mut diffs: Vec<_> = computed
        .keys()
        .filter_map(|n| {
            built
                .get(n)
                .filter(|b| **b != computed[n])
                .map(|b| (n.clone(), computed[n], *b))
        })
        .collect();
    diffs.sort_by(|a, b| a.0.cmp(&b.0));
    if diffs.is_empty() {
        return Ok(());
    }
    let lines = diffs
        .iter()
        .take(12)
        .map(|(n, c, b)| {
            format!(
                "    {n}: sources say {}, built table says {}",
                if *c { "FLAT" } else { "marshalled" },
                if *b { "FLAT" } else { "marshalled" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let more = if diffs.len() > 12 {
        format!("\n    ... and {} more", diffs.len() - 12)
    } else {
        String::new()
    };
    Err(format!("audit {tname}: the BUILT thunk table disagrees with the checked-in sources on {} syscall(s) — {} is STALE.\n{lines}{more}\nRebuild before trusting this lint (`./vibeos-cli build --arch <a>`). Reporting stale findings here is how already-fixed sites get re-blessed into the baseline.",diffs.len(),path.display()))
}

const WIN32K_TUS: &[(&str, &str)] = &[
    ("user32_layout_probe.c", "#include <user32.h>\n"),
    ("gdi32/wine/rust_cohab_stub.c", "#include <precomp.h>\n"),
];
const NTDLL_TU:&str="#define WIN32_NO_STATUS\n#include <windef.h>\n#include <winbase.h>\n#include <winreg.h>\n#include <winuser.h>\n#define NTOS_MODE_USER\n#include <ndk/cmfuncs.h>\n#include <ndk/exfuncs.h>\n#include <ndk/iofuncs.h>\n#include <ndk/kefuncs.h>\n#include <ndk/ldrfuncs.h>\n#include <ndk/mmfuncs.h>\n#include <ndk/obfuncs.h>\n#include <ndk/psfuncs.h>\n#include <ndk/rtlfuncs.h>\n#include <ndk/sefuncs.h>\n#include <ndk/lpcfuncs.h>\n#include <ndk/pofuncs.h>\n#include <ndk/dbgkfuncs.h>\n#include <ndk/umfuncs.h>\n";
fn extract_flags(root: &Path, build: &Path, suffix: &str) -> Result<Vec<String>, String> {
    let v: Value = serde_json::from_str(&read(build, "compile_commands.json")?)
        .map_err(|e| format!("invalid compile_commands.json: {e}"))?;
    let entries = v
        .as_array()
        .ok_or("compile_commands.json is not an array")?;
    let e = entries
        .iter()
        .find(|e| e["file"].as_str().is_some_and(|s| s.ends_with(suffix)))
        .ok_or_else(|| format!("NO_TU:{suffix}"))?;
    let cmd = e["command"]
        .as_str()
        .ok_or("compile_commands entry has no command string")?;
    let tokens: Vec<_> = cmd.split_whitespace().collect();
    let (mut out, mut i) = (vec![], 0);
    while i < tokens.len() {
        let tok = tokens[i];
        if [
            "-I",
            "-isystem",
            "-include",
            "-D",
            "-U",
            "-iquote",
            "-idirafter",
        ]
        .contains(&tok)
            && i + 1 < tokens.len()
        {
            out.push(tok.to_owned());
            out.push(tokens[i + 1].replace("/tmp/vibeos", &root.to_string_lossy()));
            i += 2;
            continue;
        }
        if re(r"^(-I|-D|-isystem|--target=|-f|-W|-m)").is_match(tok)
            && !re(r"^(-fdiagnostics|-fcolor|-flto|-ffile-prefix|-fdebug)").is_match(tok)
        {
            out.push(tok.replace("/tmp/vibeos", &root.to_string_lossy()))
        }
        i += 1
    }
    Ok(out)
}
fn layout_dump(
    clang: &Path,
    flags: &[String],
    tu: &str,
) -> Result<HashMap<String, Record>, String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "vibeos-marshalgen-{}-{}.c",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::write(&path, tu).map_err(|e| format!("cannot create layout TU: {e}"))?;
    let out = Command::new(clang)
        .args(flags)
        .args(["-fsyntax-only", "-Xclang", "-fdump-record-layouts-complete"])
        .arg(&path)
        .output()
        .map_err(|e| format!("cannot run clang {}: {e}", clang.display()));
    let _ = fs::remove_file(&path);
    let out = out?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let errs = stderr
            .lines()
            .filter(|l| l.contains("error:"))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "layout TU failed to compile:\n{}",
            errs.chars().take(2000).collect::<String>()
        ));
    }
    Ok(parse_layout_dump(&String::from_utf8_lossy(&out.stdout)))
}
fn build_dir(root: &Path, requested: Option<&str>, candidates: &[&str]) -> Option<PathBuf> {
    let list: Vec<_> = requested
        .map(|x| vec![x])
        .unwrap_or_else(|| candidates.to_vec());
    list.into_iter()
        .map(|x| {
            let p = Path::new(x);
            if p.is_absolute() {
                p.to_owned()
            } else {
                root.join(p)
            }
        })
        .find(|p| p.join("compile_commands.json").is_file())
}

fn audit(
    root: &Path,
    build64: Option<&str>,
    build32: Option<&str>,
    table_sel: &str,
) -> Result<Vec<String>, String> {
    let b64 = build_dir(root, build64, &["build-amd64", "build-arm64"]);
    let b32 = build_dir(root, build32, &["build-i386"]);
    let (Some(b64), Some(b32)) = (b64, b32) else {
        return Ok(vec![]);
    };
    let clang = root.join(".build-tools/llvm-mingw/bin/clang");
    if !clang.is_file() {
        return Ok(vec![]);
    }
    let tables: Vec<&str> = if table_sel == "both" {
        vec!["win32k", "ntdll"]
    } else {
        vec![table_sel]
    };
    let arch = if b64
        .file_name()
        .is_some_and(|n| n.to_string_lossy().ends_with("arm64"))
    {
        "arm64"
    } else {
        "amd64"
    };
    let mut findings = vec![];
    for tname in tables {
        let (gen_rel, headers, tus): (&str, Vec<&str>, Vec<(&str, &str)>) = if tname == "win32k" {
            (
                "dll/win32/wow64/wow64_win32k_thunks.gen.c",
                vec![
                    "win32ss/include/ntuser.h",
                    "sdk/include/psdk/ntgdi.h",
                    "win32ss/include/ntgdibad.h",
                ],
                WIN32K_TUS.to_vec(),
            )
        } else if tname == "ntdll" {
            (
                "dll/win32/wow64/wow64_thunks.gen.c",
                vec![
                    "sdk/include/ndk/cmfuncs.h",
                    "sdk/include/ndk/dbgkfuncs.h",
                    "sdk/include/ndk/exfuncs.h",
                    "sdk/include/ndk/iofuncs.h",
                    "sdk/include/ndk/kdfuncs.h",
                    "sdk/include/ndk/kefuncs.h",
                    "sdk/include/ndk/lpcfuncs.h",
                    "sdk/include/ndk/mmfuncs.h",
                    "sdk/include/ndk/obfuncs.h",
                    "sdk/include/ndk/pofuncs.h",
                    "sdk/include/ndk/psfuncs.h",
                    "sdk/include/ndk/sefuncs.h",
                ],
                vec![("ntdll/rtl/rtl_um_rust_stub.c", NTDLL_TU)],
            )
        } else {
            return Err(format!("marshalgen audit: unknown table {tname}"));
        };
        let table = compute_thunk_table(root, tname, arch)?;
        let genpath = b64.join(gen_rel);
        if genpath.is_file() {
            let text = fs::read_to_string(&genpath)
                .map_err(|e| format!("cannot read {}: {e}", genpath.display()))?;
            crosscheck(&table, &text, &genpath, tname)?;
        }
        let (mut d32, mut d64) = (HashMap::new(), HashMap::new());
        for (suffix, tu) in tus {
            for (db, bdir) in [(&mut d32, &b32), (&mut d64, &b64)] {
                let flags=extract_flags(root,bdir,suffix).map_err(|e|if e==format!("NO_TU:{suffix}"){format!("audit {tname}: no TU matching '{suffix}' in {}/compile_commands.json — the layout probe cannot run, so this table would go UNCHECKED. Point cfg['tus'] at a TU that still exists.",bdir.display())}else{e})?;
                for (k, v) in layout_dump(&clang, &flags, tu)? {
                    db.entry(k).or_insert(v);
                }
            }
        }
        if d32.is_empty() || d64.is_empty() {
            return Err(format!("audit {tname}: layout dump came back empty (32:{} 64:{}) — refusing to report a clean table from no data.",d32.len(),d64.len()));
        }
        let mut header_text = String::new();
        for h in headers {
            header_text.push_str(&read(root, h)?);
            header_text.push('\n')
        }
        let protos = extract_prototypes(&header_text, r"Nt\w+")?;
        let mut names: Vec<_> = table.keys().collect();
        names.sort();
        for name in names {
            if !table[name] {
                continue;
            }
            let Some(p) = protos.get(name) else { continue };
            for prm in &p.params {
                let (cls, det) = classify_param(prm, &d32, &d64);
                let v = audit_verdict(&cls, &det);
                let line = format!("{} {}({}) {} {}", tname, name, prm.name, cls, det);
                if v == "DIVERGENT" {
                    findings.push(format!("FLAT-DIVERGENT: {line}"))
                }
            }
        }
    }
    Ok(findings)
}
impl Rule for MarshalgenAudit {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "marshalgen_audit"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        audit(&c.root, None, None, "both")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audit_verdict_matches_python_contract() {
        assert_eq!(audit_verdict("WIDENING", "-> UNICODE_STRING"), "DIVERGENT");
        assert_eq!(
            audit_verdict("SCALAR_WIDEN", "-> HWND: 4->8 scalar pointee"),
            "DIVERGENT"
        );
        assert_eq!(
            audit_verdict("SEMANTIC", "union/bitfield with divergent layout"),
            "DIVERGENT"
        );
        assert_eq!(
            audit_verdict("SEMANTIC", "-> FOO: layout not in dump (add header to TU?)"),
            "UNKNOWN"
        );
        assert_eq!(audit_verdict("SEMANTIC", "unknown type 'BAR'"), "UNKNOWN");
        assert_eq!(audit_verdict("FLAT", "scalar"), "OK");
        assert_eq!(audit_verdict("IDENTICAL", ""), "OK")
    }
    #[test]
    fn source_table_parser_tracks_win64_guest_conditionals() {
        let s="SVC_(A, 1)\n#ifdef _WIN64\nSVC_(Only64, 1)\n#else\nSVC_(B, 2)\n#endif\n#if defined(_WIN64)\nSVC_(Only64b, 3)\n#endif\n#if !defined(_WIN64)\nSVC_(C, 4)\n#endif\n";
        assert_eq!(
            parse_sysfuncs(s),
            vec![("NtA".into(), 1), ("NtB".into(), 2), ("NtC".into(), 4)]
        )
    }
    #[test]
    fn checked_in_win32k_table_is_source_derived_with_manual_entries_disabled() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate lives in tools/vibeos-lint");
        let table = compute_thunk_table(root, "win32k", "amd64").unwrap();
        assert!(table.len() > 500);
        assert_eq!(table.get("NtGdiCreateDIBSection"), Some(&false));
        assert_eq!(table.get("NtUserGetAtomName"), Some(&false));
        assert_eq!(table.get("NtGdiAbortDoc"), Some(&true));
    }
    #[test]
    fn stale_built_table_hard_fails_with_actionable_diff() {
        let c = HashMap::from([("NtFoo".to_owned(), false)]);
        let table = "{ \"NtFoo\", wow64_thunk_NtFoo, 1 },\n";
        let err = crosscheck(&c, table, Path::new("build/x.gen.c"), "ntdll").unwrap_err();
        assert!(err.contains("STALE") && err.contains("NtFoo"))
    }
    #[test]
    fn layout_parser_and_widening_classification() {
        let a=parse_layout_dump("*** Dumping AST Record Layout\n         0 | struct _X\n         0 |   ULONG A\n         4 |   PVOID P\n           | [sizeof=8, align=4]\n");
        let b=parse_layout_dump("*** Dumping AST Record Layout\n         0 | struct _X\n         0 |   ULONG A\n         8 |   PVOID P\n           | [sizeof=16, align=8]\n");
        assert_eq!(diff_struct("X", &a, &b).cls, "WIDENING")
    }
    #[test]
    fn prototype_extraction_strips_comments_and_sal() {
        let p=extract_prototypes("/* NtFake(VOID); */\nNTSTATUS NTAPI NtReal(_In_ HANDLE Handle, _Out_writes_(Count) PULONG Buffer, ULONG Count);\n",r"Nt\w+").unwrap();
        assert!(!p.contains_key("NtFake"));
        assert_eq!(p["NtReal"].params.len(), 3);
        assert_eq!(p["NtReal"].params[0].name, "Handle");
        assert_eq!(p["NtReal"].params[1].sal, "out")
    }
    #[test]
    fn no_build_means_empty_stdout() {
        let root = std::env::temp_dir().join(format!("marshalgen-no-build-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        assert!(audit(&root, None, None, "both").unwrap().is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
