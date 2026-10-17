//! ABI-shape verifier over Rust exports and the centrally captured clang AST.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

pub struct VerifySignatures;
const INPUTS: &[SourceSet] =
    &[
        SourceSet::tree("verify-signatures-rust", &["sdk/lib/rust"], &["rs"], &[])
            .with_pruning(&["target"]),
    ];
const KERNEL_ROOTS: &[&str] = &[
    "sdk/lib/rust/ntoskrnl-rust/src",
    "sdk/lib/rust/lpc-rust/src",
    "sdk/lib/rust/fsrtl-rust/src",
    "sdk/lib/rust/dbgk-rust/src",
    "sdk/lib/rust/wmi-rust/src",
    "sdk/lib/rust/dbgprint-rust/src",
    "sdk/lib/rust/cc-rust/src",
    "sdk/lib/rust/iomgr-rust/src",
    "sdk/lib/rust/kdbg-rust/src",
    "sdk/lib/rust/config-rust/src",
    "sdk/lib/rust/ex-rust/src",
    "sdk/lib/rust/ob-rust/src",
    "sdk/lib/rust/ps-rust/src",
    "sdk/lib/rust/kd-rust/src",
    "sdk/lib/rust/kd64-rust/src",
    "sdk/lib/rust/vf-rust/src",
    "sdk/lib/rust/rtlk-rust/src",
    "sdk/lib/rust/po-rust/src",
    "sdk/lib/rust/inbv-rust/src",
    "sdk/lib/rust/vdm-rust/src",
    "sdk/lib/rust/fstub-rust/src",
    "sdk/lib/rust/se-rust/src",
    "sdk/lib/rust/pnpmgr-rust/src",
];
impl Rule for VerifySignatures {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "verify_signatures"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn needs_c_signature_asts(&self) -> &'static [&'static str] {
        &["amd64"]
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let ast = c.c_signature_ast("amd64")?;
        let (protos, ctypes) = extract(ast);
        let mut rust = Vec::new();
        for root in KERNEL_ROOTS {
            let dir = c.root.join(root);
            if !dir.exists() {
                continue;
            }
            let mut fs: Vec<_> = c
                .sources("verify-signatures-rust")
                .filter(|f| {
                    f.path.starts_with(root) && f.path.extension().is_some_and(|e| e == "rs")
                })
                .collect();
            fs.sort_by(|a, b| a.path.cmp(&b.path));
            let mut seen = BTreeSet::new();
            for f in fs {
                rust.extend(scan_rust_with_seen(&f.text, &f.path, &mut seen));
            }
        }
        let (findings, stats) = compare(&rust, &protos, &ctypes);
        let shown: Vec<_> = findings.iter().filter(|f| !f.suppressed).collect();
        let mut out = vec![
            "=== verify_signatures — arch=amd64 ===".into(),
            format!("headers: {}/build-amd64", c.root.display()),
            format!(
                "crates: {}",
                KERNEL_ROOTS
                    .iter()
                    .filter(|r| c.root.join(r).exists())
                    .map(|r| c.root.join(r).display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "Dumping C AST (clang)...".into(),
            format!(
                "  C prototypes: {} FunctionDecls, {} typedefs",
                protos.len(),
                ctypes.len()
            ),
            "Scanning exported Rust fns...".into(),
            format!("  exported Rust fns: {} (deduped twins)", rust.len()),
            "── coverage ──".into(),
            format!("  exported fns checked : {}", stats.checked),
            format!("  matched to a C proto : {}", stats.matched),
            format!("  no C proto (internal): {}", stats.no_c),
            format!("  signature OK         : {}", stats.ok),
            format!(
                "  mismatches           : {}  ({} shown, {} LOW-suppressed)",
                stats.mismatch,
                shown.len(),
                findings.len() - shown.len()
            ),
        ];
        let rank = |s: &str| match s {
            "HIGH" => 0,
            "MEDIUM" => 1,
            _ => 2,
        };
        let mut sorted = shown;
        sorted.sort_by_key(|f| rank(&f.conf));
        for f in sorted {
            out.push(format!(
                "[{}] {}  ({}:{})",
                f.conf,
                f.name,
                f.file.display(),
                f.line
            ));
            out.push(format!("    rust: {}", f.rust));
            out.push(format!("    C   : {}", f.c));
            out.extend(f.issues.iter().map(|x| format!("      - {x}")));
        }
        Ok(out)
    }
}
#[derive(Clone)]
struct Proto {
    ret: (String, String),
    params: Vec<(String, String)>,
    variadic: bool,
}
fn type_pair(v: &Value) -> (String, String) {
    let q = v
        .get("qualType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let d = v
        .get("desugaredQualType")
        .and_then(Value::as_str)
        .unwrap_or(&q)
        .to_owned();
    (q, d)
}
fn extract(ast: &Value) -> (BTreeMap<String, Proto>, BTreeMap<String, (String, String)>) {
    fn walk(
        v: &Value,
        ps: &mut BTreeMap<String, Proto>,
        ts: &mut BTreeMap<String, (String, String)>,
    ) {
        let Some(o) = v.as_object() else { return };
        match o.get("kind").and_then(Value::as_str).unwrap_or("") {
            "TypedefDecl" => {
                if let Some(n) = o.get("name").and_then(Value::as_str) {
                    ts.insert(n.into(), type_pair(o.get("type").unwrap_or(&Value::Null)));
                }
            }
            "FunctionDecl" => {
                if let Some(n) = o.get("name").and_then(Value::as_str) {
                    let qt = o
                        .get("type")
                        .and_then(|x| x.get("qualType"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ret = (
                        qt.split('(').next().unwrap_or(qt).trim().to_string(),
                        qt.split('(').next().unwrap_or(qt).trim().to_string(),
                    );
                    let params = o
                        .get("inner")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter(|x| x.get("kind").and_then(Value::as_str) == Some("ParmVarDecl"))
                        .map(|x| type_pair(x.get("type").unwrap_or(&Value::Null)))
                        .collect();
                    ps.insert(
                        n.into(),
                        Proto {
                            ret,
                            params,
                            variadic: qt.contains("..."),
                        },
                    );
                }
            }
            _ => {}
        }
        for c in o
            .get("inner")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            walk(c, ps, ts)
        }
    }
    let (mut p, mut t) = (BTreeMap::new(), BTreeMap::new());
    walk(ast, &mut p, &mut t);
    (p, t)
}
#[derive(Clone)]
struct RF {
    name: String,
    params: Vec<String>,
    ret: String,
    file: std::path::PathBuf,
    line: usize,
}
static EXPORT: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"#\[\s*(no_mangle|export_name\s*=\s*"([^"]+)")"#).unwrap()
});
static FN: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"(?:pub\s+)?(?:unsafe\s+)?extern\s+"([^"]+)"\s+fn\s+(\w+)\s*\("#)
        .unwrap()
});
fn params_split(s: &str) -> Vec<String> {
    let (mut d, mut out, mut cur) = (0isize, vec![], String::new());
    for ch in s.chars() {
        if "<([".contains(ch) {
            d += 1
        } else if ">)]".contains(ch) {
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
fn param_type(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s == "..." || s.starts_with("self") || s.starts_with("&self") {
        return None;
    }
    let (mut d,) = (0isize,);
    for (i, ch) in s.char_indices() {
        if "<([".contains(ch) {
            d += 1
        } else if ">)]".contains(ch) {
            d -= 1
        } else if ch == ':' && d == 0 {
            return Some(s[i + 1..].trim().into());
        }
    }
    Some(s.into())
}
fn balanced_params(s: &str, open: usize) -> (String, usize) {
    let (mut out, mut depth, mut j) = (String::new(), 0isize, open);
    while j < s.len() {
        if s[j..].starts_with("//") {
            j = s[j..].find('\n').map_or(s.len(), |x| j + x);
            continue;
        }
        if s[j..].starts_with("/*") {
            j = s[j..].find("*/").map_or(s.len(), |x| j + x + 2);
            continue;
        }
        let ch = s[j..].chars().next().unwrap();
        if ch == '(' {
            depth += 1
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        if j > open {
            out.push(ch)
        }
        j += ch.len_utf8()
    }
    (out, j)
}
/// Byte position immediately after at most `count` Unicode scalar values.
/// Python regex/string slices use character offsets; regex crate matches expose
/// byte offsets, so windows must cross this conversion boundary explicitly.
fn byte_after_chars(s: &str, start: usize, count: usize) -> usize {
    let tail = &s[start..];
    tail.char_indices()
        .nth(count)
        .map_or(s.len(), |(offset, _)| start + offset)
}

fn scan_rust(s: &str, path: &std::path::Path) -> Vec<RF> {
    scan_rust_with_seen(s, path, &mut BTreeSet::new())
}

fn scan_rust_with_seen(
    s: &str,
    path: &std::path::Path,
    seen: &mut BTreeSet<(String, usize)>,
) -> Vec<RF> {
    let mut out = vec![];
    for a in EXPORT.captures_iter(s) {
        let pos = a.get(0).unwrap().start();
        let end = byte_after_chars(s, pos, 600);
        let Some(f) = FN.captures(&s[pos..end]) else {
            continue;
        };
        let open = pos + f.get(0).unwrap().end() - 1;
        let name = f.get(2).unwrap().as_str();
        let link = a.get(2).map(|x| x.as_str());
        let real = link.unwrap_or(name).to_owned();
        let (p, j) = balanced_params(s, open);
        let mut ps: Vec<_> = params_split(&p)
            .iter()
            .filter_map(|x| param_type(x))
            .collect();
        let ret_start = (j + 1).min(s.len());
        let ret_end = byte_after_chars(s, ret_start, 400);
        let ret_text = &s[ret_start..ret_end];
        let rr = crate::regexes::cached(r"(?s)^\s*->\s*(.+?)\s*(\{|;|where\b)").unwrap();
        let ret = rr
            .captures(ret_text)
            .map_or("()".into(), |x| x[1].trim().into());
        let key = (real.clone(), ps.len());
        if !seen.insert(key) {
            continue;
        }
        let _ = &mut ps;
        out.push(RF {
            name: real,
            params: ps,
            ret,
            file: path.into(),
            line: s[..pos].bytes().filter(|b| *b == b'\n').count() + 1,
        });
    }
    out
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Shape {
    cat: &'static str,
    size: Option<usize>,
    sign: bool,
}
fn cshape(d: &str, arch: &str) -> Shape {
    let t = d.trim();
    let ptr = if arch == "i386" { 4 } else { 8 };
    if t.contains('*') || t.ends_with(']') || t.contains("(*") {
        return Shape {
            cat: "ptr",
            size: Some(ptr),
            sign: false,
        };
    }
    if t.starts_with("enum ") {
        return Shape {
            cat: "int",
            size: Some(4),
            sign: false,
        };
    }
    if t.starts_with("struct ") || t.starts_with("union ") {
        return Shape {
            cat: "aggregate",
            size: None,
            sign: false,
        };
    }
    let t = crate::regexes::cached(r"\b(const|volatile)\b")
        .unwrap()
        .replace_all(t, "")
        .trim()
        .to_owned();
    let (cat, size, sign) = match t.as_str() {
        "void" => ("void", 0, false),
        "_Bool" | "bool" => ("int", 1, false),
        "char" | "signed char" => ("int", 1, true),
        "unsigned char" => ("int", 1, false),
        "short" | "short int" => ("int", 2, true),
        "unsigned short" | "unsigned short int" => ("int", 2, false),
        "int" => ("int", 4, true),
        "unsigned int" | "unsigned" => ("int", 4, false),
        "long" | "long int" => ("int", 4, true),
        "unsigned long" | "unsigned long int" => ("int", 4, false),
        "long long" | "long long int" | "__int64" => ("int", 8, true),
        "unsigned long long" | "unsigned long long int" | "unsigned __int64" => ("int", 8, false),
        "float" => ("float", 4, true),
        "double" | "long double" => ("float", 8, true),
        "wchar_t" => ("int", 2, false),
        _ => ("unknown", 0, false),
    };
    Shape {
        cat,
        size: if cat == "unknown" { None } else { Some(size) },
        sign,
    }
}
fn cshape_resolved(q: &str, d: &str, arch: &str, ct: &BTreeMap<String, (String, String)>) -> Shape {
    let first = cshape(d, arch);
    if first.cat != "unknown" {
        return first;
    }
    let mods = crate::regexes::cached(r"\b(const|volatile)\b").unwrap();
    let normalized = mods.replace_all(d, "");
    let base = normalized.trim().split('<').next().unwrap_or("").trim();
    if let Some((_q, d2)) = ct.get(base) {
        return cshape(d2, arch);
    }
    let _ = q;
    first
}
fn rshape(t: &str, arch: &str, ct: &BTreeMap<String, (String, String)>) -> Shape {
    let t = t.trim();
    let ptr = if arch == "i386" { 4 } else { 8 };
    if t.starts_with("*const") || t.starts_with("*mut") || t.starts_with('&') {
        return Shape {
            cat: "ptr",
            size: Some(ptr),
            sign: false,
        };
    }
    if t.contains("fn(") || t.starts_with("extern ") || (t.contains("Option") && t.contains("fn")) {
        return Shape {
            cat: "ptr",
            size: Some(ptr),
            sign: false,
        };
    }
    let base = t
        .split('<')
        .next()
        .unwrap_or(t)
        .trim()
        .rsplit("::")
        .next()
        .unwrap_or("");
    // Python's classifier checks Rust primitives before the shared C typedef
    // table. Keep that order even if an unusual AST happens to define an alias
    // with a primitive spelling.
    let primitive = match base {
        "u8" => Some(("int", 1, false)),
        "i8" => Some(("int", 1, true)),
        "u16" => Some(("int", 2, false)),
        "i16" => Some(("int", 2, true)),
        "u32" => Some(("int", 4, false)),
        "i32" => Some(("int", 4, true)),
        "u64" => Some(("int", 8, false)),
        "i64" => Some(("int", 8, true)),
        "usize" => Some(("int", ptr, false)),
        "isize" => Some(("int", ptr, true)),
        "bool" => Some(("int", 1, false)),
        "f32" => Some(("float", 4, true)),
        "f64" => Some(("float", 8, true)),
        "c_void" | "()" => Some(("void", 0, false)),
        _ => None,
    };
    if let Some((cat, size, sign)) = primitive {
        return Shape {
            cat,
            size: Some(size),
            sign,
        };
    }
    if let Some((q, d)) = ct.get(base) {
        let _ = q;
        return cshape(d, arch);
    }
    let prim = match base {
        "u8" => ("int", 1, false),
        "i8" => ("int", 1, true),
        "u16" => ("int", 2, false),
        "i16" => ("int", 2, true),
        "u32" => ("int", 4, false),
        "i32" => ("int", 4, true),
        "u64" => ("int", 8, false),
        "i64" => ("int", 8, true),
        "usize" | "isize" => ("int", ptr, base == "isize"),
        "bool" => ("int", 1, false),
        "f32" => ("float", 4, true),
        "f64" => ("float", 8, true),
        "c_void" | "core::ffi::c_void" | "()" => ("void", 0, false),
        "ULONG" | "DWORD" | "UINT" | "ULONG32" | "ACCESS_MASK" | "LCID" => ("int", 4, false),
        "LONG" | "INT" | "LONG32" | "NTSTATUS" | "HRESULT" | "KPRIORITY" | "BOOL" => {
            ("int", 4, true)
        }
        "USHORT" | "WORD" | "WCHAR" | "LANGID" => ("int", 2, false),
        "SHORT" => ("int", 2, true),
        "UCHAR" | "BYTE" | "BOOLEAN" | "KIRQL" => ("int", 1, false),
        "CHAR" => ("int", 1, true),
        "ULONGLONG" | "ULONG64" | "DWORD64" | "QWORD" => ("int", 8, false),
        "LONGLONG" | "LONG64" => ("int", 8, true),
        "SIZE_T" | "ULONG_PTR" | "DWORD_PTR" | "UINT_PTR" | "WPARAM" => ("int", ptr, false),
        "SSIZE_T" | "LONG_PTR" | "INT_PTR" | "LPARAM" | "LRESULT" => ("int", ptr, true),
        "PVOID64" => ("ptr", 8, false),
        "HANDLE" | "HWND" | "HKEY" | "HDC" | "HMODULE" | "HINSTANCE" | "HICON" | "HMENU"
        | "HGDIOBJ" | "HBRUSH" | "HBITMAP" | "HPALETTE" | "HFONT" | "HRGN" | "SC_HANDLE"
        | "PVOID" | "LPVOID" | "PWSTR" | "PCWSTR" | "LPWSTR" | "LPCWSTR" | "PSTR" | "PCSTR"
        | "LPSTR" | "LPCSTR" | "PCHAR" | "PUCHAR" | "PWCHAR" | "PULONG" | "PLONG" | "PUSHORT"
        | "PSHORT" | "PBOOLEAN" | "PSIZE_T" | "PULONG_PTR" | "PLONG_PTR" | "PHANDLE"
        | "PLARGE_INTEGER" | "PLUID" | "PFILE_OBJECT" => ("ptr", ptr, false),
        _ => {
            if base.starts_with("NonNull") {
                return Shape {
                    cat: "ptr",
                    size: Some(ptr),
                    sign: false,
                };
            }
            if crate::regexes::cached(r"^(P|LP)[A-Z_]")
                .unwrap()
                .is_match(base)
            {
                return Shape {
                    cat: "ptr",
                    size: Some(ptr),
                    sign: false,
                };
            }
            return Shape {
                cat: "unknown",
                size: None,
                sign: false,
            };
        }
    };
    Shape {
        cat: prim.0,
        size: Some(prim.1),
        sign: prim.2,
    }
}
#[derive(Default)]
struct Stats {
    checked: usize,
    matched: usize,
    no_c: usize,
    ok: usize,
    mismatch: usize,
}
struct Finding {
    name: String,
    conf: &'static str,
    file: std::path::PathBuf,
    line: usize,
    rust: String,
    c: String,
    issues: Vec<String>,
    suppressed: bool,
}
fn compare(
    rf: &[RF],
    cp: &BTreeMap<String, Proto>,
    ct: &BTreeMap<String, (String, String)>,
) -> (Vec<Finding>, Stats) {
    let mut fs = vec![];
    let mut st = Stats::default();
    for r in rf {
        st.checked += 1;
        let Some(p) = cp.get(&r.name) else {
            st.no_c += 1;
            continue;
        };
        st.matched += 1;
        let mut issues = vec![];
        let mut conf = None;
        if p.variadic {
            if r.params.len() < p.params.len() {
                issues.push(format!(
                    "arity (variadic): rust {} < C fixed {}",
                    r.params.len(),
                    p.params.len()
                ));
                conf = Some("HIGH")
            }
        } else if r.params.len() != p.params.len() {
            issues.push(format!(
                "ARITY: rust {} params vs C {}",
                r.params.len(),
                p.params.len()
            ));
            conf = Some("HIGH")
        }
        if issues.is_empty() {
            for i in 0..r.params.len().min(p.params.len()) {
                let rs = rshape(&r.params[i], "amd64", ct);
                let cs = cshape_resolved(&p.params[i].0, &p.params[i].1, "amd64", ct);
                let why = why(rs, cs);
                if let Some(w) = why {
                    let sev = if w.starts_with("sign") {
                        "LOW"
                    } else {
                        "MEDIUM"
                    };
                    issues.push(format!(
                        "param#{} {} vs {}({}): {} [{}]",
                        i + 1,
                        r.params[i],
                        p.params[i].0,
                        p.params[i].1,
                        w,
                        sev
                    ));
                    conf = Some(max_conf(conf, sev));
                }
            }
            let rs = rshape(&r.ret, "amd64", ct);
            let cs = cshape_resolved(&p.ret.0, &p.ret.1, "amd64", ct);
            if let Some(w) = why(rs, cs) {
                let sev = if w.starts_with("sign") {
                    "LOW"
                } else {
                    "MEDIUM"
                };
                issues.push(format!("return {} vs {}: {} [{}]", r.ret, p.ret.0, w, sev));
                conf = Some(max_conf(conf, sev));
            }
        }
        if issues.is_empty() {
            st.ok += 1
        } else {
            st.mismatch += 1;
            let conf = conf.unwrap_or("LOW");
            let suppressed = conf == "LOW";
            fs.push(Finding {
                name: r.name.clone(),
                conf,
                file: r.file.clone(),
                line: r.line,
                rust: format!("fn {}({}) -> {}", r.name, r.params.join(", "), r.ret),
                c: format!(
                    "{} {}({}{})",
                    p.ret.0,
                    r.name,
                    p.params
                        .iter()
                        .map(|x| x.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    if p.variadic {
                        if p.params.is_empty() {
                            "..."
                        } else {
                            ", ..."
                        }
                    } else {
                        ""
                    }
                ),
                issues,
                suppressed,
            });
        }
    }
    (fs, st)
}
fn max_conf(a: Option<&'static str>, b: &'static str) -> &'static str {
    let rank = |x| match x {
        "HIGH" => 3,
        "MEDIUM" => 2,
        "LOW" => 1,
        _ => 0,
    };
    if rank(b) > rank(a.unwrap_or("")) {
        b
    } else {
        a.unwrap_or(b)
    }
}
fn why(a: Shape, b: Shape) -> Option<String> {
    if a.cat == "unknown" || b.cat == "unknown" || a.cat == "aggregate" || b.cat == "aggregate" {
        return None;
    }
    if a.cat != b.cat {
        if (a.cat == "ptr" && b.cat == "int" || b.cat == "ptr" && a.cat == "int")
            && a.size == b.size
        {
            return None;
        }
        return Some(if a.cat == "ptr" || b.cat == "ptr" {
            format!(
                "category {}({}) vs {}({})",
                a.cat,
                a.size.unwrap_or(0),
                b.cat,
                b.size.unwrap_or(0)
            )
        } else {
            format!("category {} vs {}", a.cat, b.cat)
        });
    }
    if a.size != b.size {
        return Some(format!(
            "width {} vs {}",
            a.size.unwrap_or(0),
            b.size.unwrap_or(0)
        ));
    }
    if a.sign != b.sign {
        return Some(format!(
            "sign {} vs {}",
            if a.sign { "s" } else { "u" },
            if b.sign { "s" } else { "u" }
        ));
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_rust_signature_controls() {
        let s = r#"#[no_mangle]
pub unsafe extern "system" fn IopCloseFile(
 Process: PEPROCESS,
 // ddk::ULONG_PTR, the alias (c_ulonglong on 64-bit), not usize: same size,
 HandleCount: ntkrnl_sys::ddk::ULONG_PTR,
) {}
"#;
        let f = scan_rust(s, std::path::Path::new("kernel.rs"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].params, vec!["PEPROCESS", "ntkrnl_sys::ddk::ULONG_PTR"]);
        let s = r#"#[no_mangle]
pub extern "system" fn Foo(a: ULONG, /* note: see ( below */ b: PVOID, // trailing ) paren
) -> NTSTATUS { 0 }
"#;
        let f = scan_rust(s, std::path::Path::new("x.rs"));
        assert_eq!(f[0].params, vec!["ULONG", "PVOID"]);
        assert_eq!(f[0].ret, "NTSTATUS");
        let s = r#"#[no_mangle]
pub extern "C" fn Bar(x: *mut [u8; 4], f: Option<fn(u32) -> u32>) {}"#;
        let f = scan_rust(s, std::path::Path::new("x.rs"));
        assert_eq!(f[0].params, vec!["*mut [u8; 4]", "Option<fn(u32) -> u32>"]);
    }
    #[test]
    fn python_character_windows_handle_multibyte_text_before_the_function() {
        let s = format!(
            "#[no_mangle]\n// {}\npub extern \"system\" fn UnicodeWindow() {{}}\n",
            "—".repeat(300)
        );
        let f = scan_rust(&s, std::path::Path::new("unicode.rs"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].name, "UnicodeWindow");
    }
    #[test]
    fn python_character_return_window_does_not_slice_inside_utf8() {
        let s = format!(
            "#[no_mangle]\npub extern \"system\" fn UnicodeReturn() -> u32 /*é{}*/ {{ 0 }}\n",
            "—".repeat(130)
        );
        let f = scan_rust(&s, std::path::Path::new("unicode.rs"));
        assert_eq!(f.len(), 1);
        assert!(f[0].ret.starts_with("u32 /*é"));
    }
    #[test]
    fn exported_twins_deduplicate_across_files_within_one_crate() {
        let mut seen = BTreeSet::new();
        let first = scan_rust_with_seen(
            "#[no_mangle]\npub extern \"system\" fn Twin(a: ULONG) {}",
            std::path::Path::new("a.rs"),
            &mut seen,
        );
        let second = scan_rust_with_seen(
            "#[no_mangle]\npub extern \"system\" fn Twin(a: ULONG) {}",
            std::path::Path::new("b.rs"),
            &mut seen,
        );
        assert_eq!(first.len(), 1);
        assert!(second.is_empty());
    }
    #[test]
    fn primitive_precedence_and_fixed_width_pointer_alias_match_python() {
        let mut aliases = BTreeMap::new();
        aliases.insert("u32".into(), ("u32".into(), "unsigned long long".into()));
        let primitive = rshape("u32", "i386", &aliases);
        assert_eq!(primitive.size, Some(4));
        let fixed = rshape("PVOID64", "i386", &BTreeMap::new());
        assert_eq!(fixed.cat, "ptr");
        assert_eq!(fixed.size, Some(8));
    }
    #[test]
    fn ast_facts_extract_prototypes_and_typedefs() {
        let ast = serde_json::json!({"kind":"TranslationUnitDecl","inner":[{"kind":"TypedefDecl","name":"SIZE_T","type":{"qualType":"SIZE_T","desugaredQualType":"unsigned long long"}},{"kind":"FunctionDecl","name":"Foo","type":{"qualType":"NTSTATUS (ULONG, PVOID)"},"inner":[{"kind":"ParmVarDecl","type":{"qualType":"ULONG","desugaredQualType":"unsigned int"}},{"kind":"ParmVarDecl","type":{"qualType":"PVOID","desugaredQualType":"void *"}}]}]});
        let (p, t) = extract(&ast);
        assert_eq!(p["Foo"].params.len(), 2);
        assert_eq!(p["Foo"].ret.0, "NTSTATUS");
        assert_eq!(t["SIZE_T"].1, "unsigned long long");
    }
    #[test]
    fn compare_detects_arity_width_and_filters_unknown_shapes() {
        let mut c = BTreeMap::new();
        c.insert(
            "Foo".into(),
            Proto {
                ret: ("void".into(), "void".into()),
                params: vec![("ULONG".into(), "unsigned int".into())],
                variadic: false,
            },
        );
        let rust = vec![RF {
            name: "Foo".into(),
            params: vec!["u64".into()],
            ret: "()".into(),
            file: "x.rs".into(),
            line: 1,
        }];
        let (f, s) = compare(&rust, &c, &BTreeMap::new());
        assert_eq!(s.mismatch, 1);
        assert_eq!(f[0].conf, "MEDIUM");
    }
}
