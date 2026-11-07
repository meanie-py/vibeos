//! Shared authoritative layout and prototype model for WoW64 generation and audits.
use regex::{Captures, Regex};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub offset: usize,
    pub depth: usize,
    pub ctype: String,
    pub name: String,
    pub is_bitfield: bool,
    /// A bitfield's bit range within its storage unit as clang dumps it (`0-0`);
    /// empty for an ordinary field.
    #[serde(default)]
    pub bits: String,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: String,
    pub size: usize,
    pub align: usize,
    pub fields: Vec<Field>,
}
impl Record {
    pub fn has_union(&self) -> bool {
        self.name.starts_with("union") || self.fields.iter().any(|f| f.ctype.starts_with("union"))
    }
    pub fn has_bitfield(&self) -> bool {
        self.fields.iter().any(|f| f.is_bitfield)
    }
    pub fn top_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.depth == 1).collect()
    }
    /// `top_fields` with each run of bitfields that share a storage unit (consecutive,
    /// same byte offset) folded into ONE field of the declared type: the unit is what
    /// gets copied, and its bits never straddle a pointer. The folded field keeps the
    /// run's names (`a/b`) and bit ranges (`0-0,1-1`) so two layouts can be compared.
    pub fn marshal_fields(&self) -> Vec<Field> {
        let mut out: Vec<Field> = Vec::new();
        for f in self.top_fields() {
            if f.is_bitfield {
                if let Some(last) = out.last_mut() {
                    if !last.bits.is_empty() && last.offset == f.offset && last.ctype == f.ctype {
                        last.name = format!("{}/{}", last.name, f.name);
                        last.bits = format!("{},{}", last.bits, f.bits);
                        continue;
                    }
                }
                let mut unit = f.clone();
                unit.is_bitfield = false;
                if unit.bits.is_empty() {
                    unit.bits = "?".into();
                }
                out.push(unit);
            } else {
                out.push(f.clone());
            }
        }
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Param {
    pub ctype: String,
    pub name: String,
    pub sal: String,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Proto {
    pub name: String,
    pub ret: String,
    pub params: Vec<Param>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
        Regex::new(pat).expect("constant marshalgen regex"),
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
                bits: m.get(2).map_or(String::new(), |b| b.as_str()[1..].to_owned()),
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
    Regex::new(p).map_err(|e| format!("invalid marshalgen regex: {e}"))
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
pub fn norm_type(s: &str) -> String {
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
pub fn scalar_base(t: &str) -> &str {
    for p in ["LPC", "PC", "LP", "P"] {
        if let Some(s) = t.strip_prefix(p) {
            return s;
        }
    }
    t
}
pub fn is_safe_scalar(t: &str) -> bool {
    SAFE.contains(&t)
}
pub fn is_divergent_scalar(t: &str) -> bool {
    DIVERGENT.contains(&t)
}
pub fn is_string_pointee(t: &str) -> bool {
    ["STR", "CSTR", "WSTR", "CWSTR", "TSTR", "CTSTR"].contains(&t)
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
    if r32.has_union() || r64.has_union() {
        // Unchanged from before bitfield units: bitfields count as offset -1.
        let raw = |r: &Record| {
            (
                r.size,
                r.top_fields()
                    .iter()
                    .map(|f| (if f.is_bitfield { -1isize } else { f.offset as isize }, f.name.clone()))
                    .collect::<Vec<_>>(),
            )
        };
        let same = raw(&r32) == raw(&r64);
        return StructVerdict {
            cls: if same { "IDENTICAL" } else { "SEMANTIC" }.into(),
            // Wording kept from before bitfield units: the marshalgen_audit baseline
            // keys on it.
            detail: if same { "same layout (union/bitfield)" } else { "union/bitfield with divergent layout" }.into(),
            rec32: Some(r32),
            rec64: Some(r64),
        };
    }
    // Bitfields are compared as storage units (Record::marshal_fields). A unit whose
    // bits are laid out identically on both sides is an ordinary scalar that only
    // moved, which is what MENUBARINFO's fBarFocused/fFocused pair is.
    let m32 = r32.marshal_fields();
    let m64 = r64.marshal_fields();
    let bits = |fs: &Vec<Field>| fs.iter().map(|f| f.bits.clone()).collect::<Vec<_>>();
    if bits(&m32) != bits(&m64) || m32.iter().any(|f| f.bits.contains('?')) {
        return StructVerdict {
            cls: "SEMANTIC".into(),
            detail: "bitfield units differ across widths".into(),
            rec32: Some(r32),
            rec64: Some(r64),
        };
    }
    let f32: Vec<&Field> = m32.iter().collect();
    let f64: Vec<&Field> = m64.iter().collect();
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

pub mod probe;
pub use probe::{dump_expected, dump_layouts, extract_flags};

pub mod audit;
pub mod render;
pub use audit::{audit, crosscheck, parse_table_artifact};

#[cfg(test)]
mod tests;

pub use crate::wow64::metadata;
pub fn win32k_tus() -> Vec<(&'static str, &'static str)> {
    metadata()["WIN32K_TUS"]
        .as_array()
        .expect("probe recipes")
        .iter()
        .map(|p| (p[0].as_str().unwrap(), p[1].as_str().unwrap()))
        .collect()
}
