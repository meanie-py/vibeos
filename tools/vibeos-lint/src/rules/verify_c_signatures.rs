//! Static Rust/C declaration depth verifier, port of tools/verify_c_signatures.py.
use crate::{Corpus, Rule, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::LazyLock;

pub struct VerifyCSignatures;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("vcs-rust", &["sdk/lib/rust"], &["rs"], &["Cargo.toml"])
        .with_pruning(&["target", "tests", "examples", "benches"]),
    SourceSet::tree("vcs-c", &["dll", "drivers"], &["c"], &[])
        .with_pruning(&["target", "build-*", ".git"])
        .with_directories(),
    SourceSet::tree(
        "vcs-headers",
        &["sdk/include/psdk", "sdk/include/reactos", "sdk/include/xdk"],
        &["h", "idl"],
        &[],
    )
    .with_pruning(&["target", ".git"]),
];
static COMMENT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"/\*.*?\*/|//[^\n]*").unwrap());
static TAG: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\b(?:struct|union|enum)\s+([A-Za-z_]\w*)").unwrap());
static DECL_HANDLE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*DECLARE_HANDLE\s*\(\s*([A-Za-z_]\w*)\s*\)").unwrap()
});
static TYPEDEF: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^[ \t]*typedef[ \t]+([^;(){}\n]+?)[ \t]*;[ \t]*$").unwrap()
});
static ALIAS: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*#\s*define\s+([A-Za-z_]\w*)\s+([A-Za-z_]\w*)\s*$").unwrap()
});
static DEFINE_PTR: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*#\s*define\s+([A-Za-z_]\w*)\s+(?:const\s+|CONST\s+)*([A-Za-z_]\w*)\s*(\*+)\s*(?:const|__MIDL_CONST|CONST)?\s*$").unwrap()
});
static C_DEF: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?sm)^((?:[A-Za-z_][\w \t]*[ \t\*]+)?)(\w+)\s*\(([^;{]*?)\)\s*\n?\{")
        .unwrap()
});
static SAL_PARAM: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\b_[A-Za-z_]+_\s*\([^()]*(?:\([^()]*\)[^()]*)*\)").unwrap()
});
static SAL_BARE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"\b(?:_[A-Za-z_]+_|IN|OUT|OPTIONAL|FAR|NEAR|CONST)\b").unwrap()
});

#[derive(Default, Clone)]
struct Model {
    stars: BTreeMap<String, usize>,
    alias: BTreeMap<String, String>,
    amb: BTreeSet<String>,
    tags: BTreeSet<String>,
}
fn norm(s: &str) -> String {
    let mods=crate::regexes::cached(r"\b(?:_W64|__w64|unsigned|signed|const|volatile|struct|union|enum|register|__declspec|__cdecl|__stdcall|CONST)\b").unwrap();
    let stripped = mods.replace_all(s, " ");
    let v: Vec<_> = stripped.split_whitespace().collect();
    v.last().copied().unwrap_or("int").into()
}
fn clean_noise(s: &str) -> String {
    let re=crate::regexes::cached(r"\[[^\]]*\]|\b(?:const|CONST|volatile|struct|union|enum|__MIDL_CONST|FAR|NEAR)\b|_[A-Za-z_]+_\([^()]*\)|\b_[A-Za-z_]+_\b").unwrap();
    re.replace_all(s, " ").into_owned()
}
fn braced(text: &str) -> Vec<String> {
    let mut out = vec![];
    for m in crate::regexes::cached(r"\btypedef\b")
        .unwrap()
        .find_iter(text)
    {
        let (mut depth, mut i, mut start) = (0isize, m.end(), None);
        while i < text.len() && i < m.end() + 200_000 {
            let b = text.as_bytes()[i];
            if b == b'{' {
                depth += 1
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    start = Some(i + 1)
                } else if depth < 0 {
                    break;
                }
            } else if depth == 0 && b == b'(' {
                break;
            } else if depth == 0 && b == b';' {
                if let Some(s) = start {
                    out.push(text[s..i].to_owned())
                }
                break;
            }
            i += 1
        }
    }
    out
}
fn model(files: impl Iterator<Item = String>) -> Model {
    let mut m = Model::default();
    let mut vs: BTreeMap<String, BTreeSet<(usize, String, bool)>> = BTreeMap::new();
    for (idx, raw) in files.enumerate() {
        let t = COMMENT.replace_all(&raw, " ").into_owned();
        let idl = raw.starts_with(".idl\n");
        m.tags.extend(TAG.captures_iter(&t).map(|x| x[1].into()));
        for tail in braced(&t) {
            let ds: Vec<_> = clean_noise(&tail)
                .split(',')
                .map(|x| x.trim().to_owned())
                .collect();
            let base = ds
                .iter()
                .find(|x| !x.is_empty() && !x.starts_with('*'))
                .cloned()
                .unwrap_or_default();
            if crate::regexes::cached(r"^[A-Za-z_]\w*$")
                .unwrap()
                .is_match(&base)
            {
                vs.entry(base.clone())
                    .or_default()
                    .insert((0, String::new(), idl));
                for d in ds {
                    if let Some(c) = crate::regexes::cached(r"^(\*+)\s*([A-Za-z_]\w*)$")
                        .unwrap()
                        .captures(&d)
                    {
                        vs.entry(c[2].into())
                            .or_default()
                            .insert((c[1].len(), base.clone(), idl));
                    }
                }
            }
        }
        for c in DECL_HANDLE.captures_iter(&t) {
            vs.entry(c[1].into())
                .or_default()
                .insert((1, String::new(), idl));
        }
        for c in TYPEDEF.captures_iter(&t) {
            let body = clean_noise(&c[1]);
            let parts: Vec<_> = body.split(',').map(str::trim).collect();
            if parts.is_empty() {
                continue;
            }
            let Some(first) = crate::regexes::cached(r"(?s)^(.*?)([A-Za-z_]\w*)\s*$")
                .unwrap()
                .captures(parts[0])
            else {
                continue;
            };
            let prefix = &first[1];
            if prefix.trim().is_empty() {
                continue;
            }
            let base = norm(&prefix.replace('*', " "));
            let decls = std::iter::once(format!(
                "{}{}",
                "*".repeat(prefix.matches('*').count()),
                &first[2]
            ))
            .chain(parts.iter().skip(1).map(|x| x.to_string()));
            for d in decls {
                if let Some(c) = crate::regexes::cached(r"^(\**)\s*([A-Za-z_]\w*)$")
                    .unwrap()
                    .captures(d.trim())
                {
                    let (name, stars) = (&c[2], c[1].len());
                    if name != base {
                        vs.entry(name.into())
                            .or_default()
                            .insert((stars, base.clone(), idl));
                    }
                }
            }
        }
        for c in ALIAS.captures_iter(&t) {
            vs.entry(c[1].into())
                .or_default()
                .insert((0, c[2].into(), idl));
        }
        for c in DEFINE_PTR.captures_iter(&t) {
            vs.entry(c[1].into())
                .or_default()
                .insert((c[3].len(), c[2].into(), idl));
        }
        let _ = idx;
    }
    for set in vs.values_mut() {
        if set.iter().any(|x| !x.2) {
            set.retain(|x| !x.2)
        }
    }
    for (n, set) in &vs {
        if let Some((s, b, _)) = set.iter().next() {
            m.stars.insert(n.clone(), *s);
            if !b.is_empty() {
                m.alias.insert(n.clone(), b.clone());
            }
        }
    }
    fn d(t: &str, m: &Model, seen: &mut BTreeSet<String>) -> usize {
        if t.is_empty() || m.tags.contains(t) || !seen.insert(t.into()) {
            return 0;
        }
        if let Some(s) = m.stars.get(t) {
            return *s + m.alias.get(t).map_or(0, |b| d(b, m, seen));
        }
        if let Some(b) = m.alias.get(t) {
            return d(b, m, seen);
        }
        0
    }
    for (n, set) in vs {
        let vals: BTreeSet<_> = set
            .into_iter()
            .map(|(s, b, _)| s + d(&b, &m, &mut BTreeSet::from([n.clone()])))
            .collect();
        if vals.len() > 1 {
            m.amb.insert(n.clone());
            m.stars.remove(&n);
            m.alias.remove(&n);
        }
    }
    m
}
const SCALARS:&str="void char short int long float double signed unsigned size_t wchar_t __int64 __int32 __int16 __int8 bool _Bool int8_t int16_t int32_t int64_t uint8_t uint16_t uint32_t uint64_t intptr_t uintptr_t ptrdiff_t intmax_t uintmax_t VOID CHAR SHORT INT LONG FLOAT DOUBLE BYTE WORD DWORD BOOL BOOLEAN UCHAR USHORT UINT ULONG LONGLONG ULONGLONG INT8 INT16 INT32 INT64 UINT8 UINT16 UINT32 UINT64 LONG32 LONG64 ULONG32 ULONG64 SIZE_T SSIZE_T DWORD_PTR ULONG_PTR LONG_PTR UINT_PTR INT_PTR NTSTATUS HRESULT WPARAM LPARAM LRESULT COLORREF ACCESS_MASK LCID LANGID ATOM WCHAR TCHAR DWORDLONG QWORD FARPROC PROC SOCKET GROUP i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64 c_void c_char c_schar c_uchar c_short c_ushort c_int c_uint c_long c_ulong c_longlong c_ulonglong c_float c_double bool_ char16_t";
fn depth(t: &str, m: &Model, seen: &mut BTreeSet<String>) -> Result<usize, String> {
    let t = t.trim();
    if t.is_empty() {
        return Ok(0);
    }
    if t.contains('*') {
        let n = t.matches('*').count();
        let p = crate::regexes::cached(r"[*]|\bmut\b|\bconst\b")
            .unwrap()
            .replace_all(t, " ");
        let base = p.split_whitespace().last().unwrap_or("");
        return Ok(n + if base.is_empty() {
            0
        } else {
            depth(base, m, seen)?
        });
    }
    let normalized = norm(t);
    let b = normalized.rsplit("::").next().unwrap_or("");
    if b == "PASCAL" || seen.contains(b) {
        return Ok(0);
    }
    if m.amb.contains(b) {
        return Err(format!("ambiguous:{b}"));
    }
    if m.tags.contains(b) {
        return Ok(0);
    }
    if let Some(s) = m.stars.get(b) {
        seen.insert(b.into());
        return Ok(*s + m.alias.get(b).map_or(Ok(0), |x| depth(x, m, seen))?);
    }
    if let Some(a) = m.alias.get(b) {
        seen.insert(b.into());
        return depth(a, m, seen);
    }
    if SCALARS.split_whitespace().any(|x| x == b) {
        return Ok(0);
    }
    let pref = b.strip_prefix("LP").or_else(|| b.strip_prefix('P'));
    if let Some(x) = pref {
        if x.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && (m.stars.contains_key(x) || m.alias.contains_key(x))
        {
            return Ok(1 + depth(x, m, seen)?);
        }
    }
    Err(format!("unknown:{b}"))
}
fn ptr_sized(t: &str, m: &Model) -> bool {
    let names="SIZE_T SSIZE_T DWORD_PTR ULONG_PTR LONG_PTR INT_PTR UINT_PTR HALF_PTR UHALF_PTR WPARAM LPARAM LRESULT usize isize size_t ssize_t ptrdiff_t intptr_t uintptr_t";
    if t.contains('*') {
        return false;
    }
    let mut b = norm(t).rsplit("::").next().unwrap_or("").to_owned();
    let mut seen = BTreeSet::new();
    while !b.is_empty() && seen.insert(b.clone()) {
        if names.split_whitespace().any(|x| x == b) {
            return true;
        }
        b = m.alias.get(&b).cloned().unwrap_or_default()
    }
    false
}
fn strip_sal(s: &str) -> String {
    let s = COMMENT.replace_all(s, " ");
    let decl=crate::regexes::cached(r"\b(?:DECLSPEC_\w+|WINAPI|WSAAPI|WSPAPI|APIENTRY|__RPC_USER|__RPC_STUB|RPC_ENTRY|WINAPIV)\b").unwrap();
    let s = decl.replace_all(&s, " ");
    SAL_BARE
        .replace_all(&SAL_PARAM.replace_all(&s, " "), "")
        .trim()
        .to_owned()
}
fn c_fns(s: &str) -> BTreeMap<String, (String, Vec<String>)> {
    let t = COMMENT.replace_all(s, " ");
    let conv="WSAAPI WSPAPI WINAPI NTAPI APIENTRY PASCAL FAR CALLBACK STDMETHODCALLTYPE inline static __cdecl __stdcall __RPC_USER __RPC_STUB RPC_ENTRY SEC_ENTRY CDECL WINAPIV DECLSPEC_NORETURN DESK_EXT_CALLBACK VFWAPI VFWAPIV";
    let mut out = BTreeMap::new();
    for c in C_DEF.captures_iter(&t) {
        let name = &c[2];
        if ["if", "while", "for", "switch", "return", "sizeof"].contains(&name) {
            continue;
        }
        let clean_ret = |value: &str| {
            let mut v = value
                .split_whitespace()
                .filter(|x| !conv.split_whitespace().any(|y| y == *x))
                .collect::<Vec<_>>()
                .join(" ");
            v = crate::regexes::cached(r"\bDECLSPEC_\w+\b")
                .unwrap()
                .replace_all(&v, "")
                .trim()
                .into();
            v
        };
        let mut ret = clean_ret(&c[1]);
        if ret.is_empty() {
            let before = &t[..c.get(0).unwrap().start()];
            for line in before.lines().rev().take(6) {
                let line = line.trim();
                if line.is_empty()
                    || conv.split_whitespace().any(|x| x == line)
                    || crate::regexes::cached(r"^(DECLSPEC_\w+|__declspec\s*\(.*\))$")
                        .unwrap()
                        .is_match(line)
                {
                    continue;
                }
                if crate::regexes::cached(r"^[A-Za-z_][\w *]*$")
                    .unwrap()
                    .is_match(line)
                {
                    ret = clean_ret(line);
                }
                break;
            }
        }
        let params = c[3]
            .split(',')
            .filter_map(|p| {
                let p = strip_sal(p);
                if p.is_empty() || p == "void" {
                    return None;
                }
                let arr = crate::regexes::cached(r"\[\s*\d*\s*\]\s*$")
                    .unwrap()
                    .is_match(&p);
                let ty = crate::regexes::cached(r"\b\w+\s*(\[\s*\d*\s*\])?$")
                    .unwrap()
                    .replace(&p, "")
                    .trim()
                    .to_owned();
                Some(if arr { format!("{ty} *") } else { ty })
            })
            .collect();
        out.insert(name.into(), (ret, params));
    }
    out
}
fn r_fns(s: &str) -> BTreeMap<String, (String, Vec<String>)> {
    let re=crate::regexes::cached(r#"(?s)#\[no_mangle\]\s*\npub unsafe extern \"[^\"]+\" fn (\w+)\(([^)]*)\)\s*(?:->\s*([^{]+?))?\s*\{"#).unwrap();
    let mut out = BTreeMap::new();
    for c in re.captures_iter(s) {
        let ps = c[2]
            .split(',')
            .filter_map(|p| p.split_once(':').map(|x| x.1.trim().to_owned()))
            .collect();
        out.insert(
            c[1].to_owned(),
            (c.get(3).map_or("", |x| x.as_str()).trim().to_owned(), ps),
        );
    }
    out
}
impl Rule for VerifyCSignatures {
    fn name(&self) -> &'static str {
        "verify_c_signatures"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let m = model(
            c.sources("vcs-headers")
                .map(|f| {
                    format!(
                        "{}\n{}",
                        if f.path.extension().is_some_and(|x| x == "idl") {
                            ".idl"
                        } else {
                            ".h"
                        },
                        f.text
                    )
                })
                .collect::<Vec<_>>()
                .into_iter(),
        );
        let rust: Vec<_> = c
            .sources("vcs-rust")
            .filter(|f| f.path.extension().is_some_and(|x| x == "rs"))
            .collect();
        let ordered_c = c.ordered_sources("vcs-c");
        let mut crates: BTreeMap<String, Vec<&crate::SourceFile>> = BTreeMap::new();
        for f in rust {
            let p = f.path.to_string_lossy();
            if let Some(i) = p.find("sdk/lib/rust/") {
                let tail = &p[i + 13..];
                if let Some((crate_dir, rest)) = tail.split_once("/") {
                    if crate_dir.ends_with("-rust") && rest.starts_with("src/") {
                        crates
                            .entry(crate_dir.trim_end_matches("-rust").into())
                            .or_default()
                            .push(f)
                    }
                }
            }
        }
        let mut out = vec![];
        for (name, files) in crates {
            // Python's Path.rglob searches dll before drivers and takes the first
            // directory match, even when that directory contains no C files.
            let mut selected_dir = None;
            for root in ["dll", "drivers"] {
                selected_dir = ordered_c
                    .iter()
                    .find(|x| {
                        x.is_directory()
                            && x.path.file_name().is_some_and(|n| n == name.as_str())
                            && x.path.starts_with(root)
                    })
                    .map(|x| x.path.as_path());
                if selected_dir.is_some() {
                    break;
                }
            }
            let Some(dir) = selected_dir else { continue };
            let mut by_stem = HashMap::new();
            for x in ordered_c.iter().filter(|x| {
                !x.is_directory()
                    && x.path.starts_with(dir)
                    && x.path.extension().is_some_and(|e| e == "c")
            }) {
                if let Some(stem) = x.path.file_stem() {
                    by_stem.insert(stem.to_string_lossy().to_string(), *x);
                }
            }
            for rf in files {
                let Some(stem) = rf.path.file_stem().map(|x| x.to_string_lossy().to_string())
                else {
                    continue;
                };
                let Some(cf) = by_stem.get(&stem) else {
                    continue;
                };
                let cs = c_fns(&cf.text);
                let rs = r_fns(&rf.text);
                for (n, (rr, rp)) in rs {
                    let Some((cr, cp)) = cs.get(&n) else { continue };
                    if let Ok(a) = depth(cr, &m, &mut BTreeSet::new()) {
                        if let Ok(b) = depth(&rr, &m, &mut BTreeSet::new()) {
                            if a != b || ptr_sized(cr, &m) != ptr_sized(&rr, &m) {
                                out.push(format!(
                                    "{name}/{}: {n} return: C `{cr}` vs Rust `{rr}`",
                                    rf.path.file_name().unwrap().to_string_lossy()
                                ));
                            }
                        }
                    }
                    if cp.len() != rp.len() {
                        continue;
                    }
                    for (i, (ct, rt)) in cp.iter().zip(&rp).enumerate() {
                        if let (Ok(a), Ok(b)) = (
                            depth(ct, &m, &mut BTreeSet::new()),
                            depth(rt, &m, &mut BTreeSet::new()),
                        ) {
                            if a != b || ptr_sized(ct, &m) != ptr_sized(rt, &m) {
                                out.push(format!(
                                    "{name}/{}: {n} param {i}: C `{ct}` vs Rust `{rt}`",
                                    rf.path.file_name().unwrap().to_string_lossy()
                                ));
                            }
                        }
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typedef_resolution_and_pointer_sized() {
        let m=model(vec![".h\ntypedef void *HANDLE, **PHANDLE;\ntypedef int INT;\ntypedef struct _FOO { int x; } FOO, *PFOO;\ntypedef unsigned long DWORD;\ntypedef unsigned long ULONG_PTR;\n".into()].into_iter());
        assert_eq!(depth("HANDLE", &m, &mut BTreeSet::new()).unwrap(), 1);
        assert_eq!(depth("PHANDLE", &m, &mut BTreeSet::new()).unwrap(), 2);
        assert_eq!(depth("PFOO", &m, &mut BTreeSet::new()).unwrap(), 1);
        assert_eq!(
            depth("POINT", &m, &mut BTreeSet::new()).unwrap_err(),
            "unknown:POINT"
        );
        assert!(ptr_sized("ULONG_PTR", &m));
        assert!(!ptr_sized("DWORD", &m));
    }
    #[test]
    fn original_resolution_and_bug_controls() {
        let m = model(
            vec![r#".h
 typedef void *HANDLE, **PHANDLE;
 typedef void *PVOID, *LPVOID;
 typedef char *LPWSTR, *LPCWSTR, *PCWSTR;
 typedef int INT;
 typedef unsigned int UINT;
 typedef unsigned int DWORD;
 typedef unsigned long ULONG_PTR, DWORD_PTR;
 typedef unsigned long SIZE_T;
 typedef struct _STARTUPINFOW { int x; } STARTUPINFOW, *LPSTARTUPINFOW;
 typedef struct _POINT { int x; } POINT;
 typedef struct _PALETTEENTRY { int x; } PALETTEENTRY;
 typedef struct _PIXELFORMATDESCRIPTOR { int x; } PIXELFORMATDESCRIPTOR;
 typedef struct _PERF_DATA_BLOCK { int x; } PERF_DATA_BLOCK;
 DECLARE_HANDLE(HWND);
 DECLARE_HANDLE(HMENU);
 DECLARE_HANDLE(HDC);
 DECLARE_HANDLE(HKEY);
 typedef int BOOL;
 typedef int NTSTATUS;
 typedef int LONG;
 typedef int SOCKET;
 typedef unsigned long WPARAM;
 typedef long LPARAM;
 typedef struct _HFONT { int x; } HFONT;
 typedef unsigned long ULONG;
"#
            .into()]
            .into_iter(),
        );
        for (t, want) in [
            ("HANDLE", 1),
            ("PHANDLE", 2),
            ("PVOID", 1),
            ("LPVOID", 1),
            ("LPWSTR", 1),
            ("LPCWSTR", 1),
            ("PCWSTR", 1),
            ("INT", 0),
            ("DWORD", 0),
            ("STARTUPINFOW *", 1),
            ("LPSTARTUPINFOW", 1),
            ("POINT", 0),
            ("PALETTEENTRY", 0),
            ("PIXELFORMATDESCRIPTOR", 0),
            ("PERF_DATA_BLOCK", 0),
            ("HWND", 1),
            ("HMENU", 1),
            ("HDC", 1),
            ("HKEY", 1),
            ("i32", 0),
            ("usize", 0),
            ("*mut c_void", 1),
        ] {
            assert_eq!(depth(t, &m, &mut BTreeSet::new()).unwrap(), want, "{t}");
        }
        for (c, r) in [
            ("INT", "LPINT"),
            ("HANDLE", "LPHANDLE"),
            ("HANDLE", "PHANDLE"),
            ("DWORD", "DWORD_PTR"),
            ("INT", "INT_PTR"),
            ("LONG", "LONG_PTR"),
            ("DWORD", "SIZE_T"),
        ] {
            let a = depth(c, &m, &mut BTreeSet::new()).unwrap();
            let b = depth(r, &m, &mut BTreeSet::new()).unwrap();
            assert!(a != b || ptr_sized(c, &m) != ptr_sized(r, &m), "{c} vs {r}");
        }
        for (c, r) in [
            ("INT", "i32"),
            ("DWORD", "u32"),
            ("SIZE_T", "usize"),
            ("ULONG_PTR", "usize"),
            ("DWORD_PTR", "usize"),
            ("HANDLE", "PVOID"),
            ("LPVOID", "*mut c_void"),
            ("BOOL", "i32"),
            ("NTSTATUS", "i32"),
            ("LPCWSTR", "LPCWSTR"),
            ("LONG", "c_long"),
        ] {
            let a = depth(c, &m, &mut BTreeSet::new()).unwrap();
            let b = depth(r, &m, &mut BTreeSet::new()).unwrap();
            assert!(a == b && ptr_sized(c, &m) == ptr_sized(r, &m), "{c} vs {r}");
        }
    }
    #[test]
    fn unknown_and_ambiguous_are_not_guessed() {
        let m = model(vec![".h\ntypedef char *BSTR;\ntypedef long BSTR;".into()].into_iter());
        assert!(
            depth("ThisTypeDoesNotExistAnywhere", &m, &mut BTreeSet::new())
                .unwrap_err()
                .starts_with("unknown:")
        );
        assert!(depth("BSTR", &m, &mut BTreeSet::new())
            .unwrap_err()
            .starts_with("ambiguous:"));
    }
    #[test]
    fn c_and_rust_declaration_parsers() {
        let c = c_fns("INT\nWSAAPI\nFoo(IN HANDLE h, _Out_ PVOID *out)\n{\n}\n");
        assert_eq!(c["Foo"].0, "INT");
        assert_eq!(c["Foo"].1, vec!["HANDLE", "PVOID *"]);
        let r=r_fns("#[no_mangle]\npub unsafe extern \"system\" fn Foo(h: HANDLE, out: *mut PVOID) -> INT { 0 }\n");
        assert_eq!(
            r["Foo"],
            ("INT".into(), vec!["HANDLE".into(), "*mut PVOID".into()])
        );
    }
    #[test]
    fn braced_and_ambiguous_typedefs() {
        let m = model(
            vec![
                ".h\ntypedef union _U { struct { int x; } HWND; } U, *PU;\n".into(),
                ".idl\ntypedef long BSTR;\ntypedef char *BSTR;\n".into(),
            ]
            .into_iter(),
        );
        assert_eq!(depth("PU", &m, &mut BTreeSet::new()).unwrap(), 1);
        assert!(depth("BSTR", &m, &mut BTreeSet::new()).is_err());
    }
}
