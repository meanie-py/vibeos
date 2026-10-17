use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct ExternDeclDrift;
static SOURCES: &[SourceSet] = &[
    SourceSet::tree("drift_rust", &["sdk/lib/rust"], &["rs"], &[]),
    SourceSet::tree("drift_manifests", &["sdk/lib/rust"], &[], &["Cargo.toml"]),
    SourceSet::tracked("drift_c", &["c"], &[]),
];
const PTR: &str = "ptr";
const ARCHES: &[(&str, usize)] = &[("i386", 4), ("amd64", 8), ("arm64", 8)];
static PATH_QUAL: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"\b(?:[A-Za-z_]\w*::)+([A-Za-z_]\w*)").unwrap());
static PTR_HEAD: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^(?:\*\s*(?:mut|const)|&\s*(?:mut\s+)?)\s*(.*)$").unwrap()
});
static OPTION: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"^(?:::)?(?:core::)?(?:option::)?Option\s*<(.*)>$").unwrap()
});
static FN_PTR: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"\bfn\s*\(").unwrap());
static IDENT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"^[A-Za-z_]\w*$").unwrap());
static NOLINT_HEAD: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"//\s*nolint\b").unwrap());
static C_FN: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^(?:(?:extern|inline|__inline|FORCEINLINE)\s+)*([A-Za-z_]\w*)((?:\s*\*)*)\s+(?:(?:NTAPI|WINAPI|__stdcall|__cdecl|__fastcall|FASTCALL|CALLBACK|APIENTRY|NTSYSAPI)\s+)*(\**)?\s*([A-Za-z_]\w*)\s*\(([^;{}()]*)\)\s*\{").unwrap()
});

#[derive(Clone, Debug)]
struct Sig {
    params: Vec<String>,
    variadic: bool,
}
#[derive(Default)]
struct Facts {
    aliases: BTreeMap<String, BTreeSet<String>>,
    aggregates: BTreeSet<String>,
    defs: BTreeMap<String, Vec<(String, String, Sig, String)>>,
    decls: BTreeMap<String, Vec<(String, usize, Sig, bool, String)>>,
    called: BTreeMap<String, BTreeSet<String>>,
    statics_def: BTreeMap<String, Vec<(String, String, String)>>,
    statics_decl: BTreeMap<String, Vec<(String, usize, String)>>,
    ret_def: BTreeMap<String, Vec<(String, String, String)>>,
    ret_decl: BTreeMap<String, Vec<(String, usize, String, String)>>,
}
#[derive(Default)]
struct Types {
    aliases: BTreeMap<String, BTreeSet<String>>,
    aggregates: BTreeSet<String>,
    cache: BTreeMap<String, Option<(Option<usize>, Option<String>)>>,
}

fn text<'a>(n: tree_sitter::Node<'_>, src: &'a str) -> &'a str {
    &src[n.start_byte()..n.end_byte()]
}
fn walk<'a>(n: tree_sitter::Node<'a>, out: &mut Vec<tree_sitter::Node<'a>>) {
    out.push(n);
    for i in 0..n.child_count() {
        if let Some(c) = n.child(i) {
            walk(c, out);
        }
    }
}
fn in_foreign(n: tree_sitter::Node<'_>) -> bool {
    let mut p = n.parent();
    while let Some(x) = p {
        if x.kind() == "foreign_mod_item" {
            return true;
        }
        p = x.parent();
    }
    false
}
fn symbol(n: tree_sitter::Node<'_>, src: &str, ident: &str) -> String {
    let mut p = n.prev_sibling();
    while let Some(a) = p {
        if a.kind() != "attribute_item" {
            break;
        }
        if let Some(c) = LINKNAME.captures(text(a, src)) {
            return c[1].to_owned();
        }
        p = a.prev_sibling();
    }
    for i in 0..n.child_count() {
        if let Some(a) = n.child(i) {
            if a.kind() == "attribute_item" {
                if let Some(c) = LINKNAME.captures(text(a, src)) {
                    return c[1].to_owned();
                }
            }
        }
    }
    ident.to_owned()
}
static LINKNAME: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r#"(?:link_name|export_name)\s*=\s*"([^"]+)""#).unwrap()
});
fn sig(n: tree_sitter::Node<'_>, src: &str) -> Sig {
    let mut out = Sig {
        params: vec![],
        variadic: false,
    };
    if let Some(ps) = n.child_by_field_name("parameters") {
        for i in 0..ps.named_child_count() {
            if let Some(p) = ps.named_child(i) {
                match p.kind() {
                    "parameter" => {
                        if let Some(t) = p.child_by_field_name("type") {
                            out.params.push(norm(text(t, src)));
                        } else {
                            out.variadic = true;
                        }
                    }
                    "variadic_parameter" => out.variadic = true,
                    _ => {}
                }
            }
        }
    }
    out
}
fn ret(n: tree_sitter::Node<'_>, src: &str) -> String {
    n.child_by_field_name("return_type")
        .map_or(String::new(), |r| norm(text(r, src)))
}
fn nolint(s: &str) -> bool {
    NOLINT_HEAD.find_iter(s).any(|m| {
        let tail = s[m.end()..].trim_start();
        if let Some(category) = tail.strip_prefix(':') {
            let category = category.trim_start();
            category.starts_with("decl-drift")
                && category["decl-drift".len()..]
                    .chars()
                    .next()
                    .map_or(true, |c| !c.is_alphanumeric() && c != '_')
        } else {
            true
        }
    })
}
fn file_facts(f: &SourceFile, owner: Option<&str>, all: &mut Facts) -> Result<(), String> {
    let tree = f.rust_tree()?;
    let nodes = {
        let mut v = vec![];
        walk(tree.root_node(), &mut v);
        v
    };
    let src = &f.text;
    let lines = src.lines().collect::<Vec<_>>();
    let p = f.path.to_string_lossy().replace('\\', "/");
    for n in nodes {
        match n.kind() {
            "type_item" => {
                if let (Some(name), Some(ty)) =
                    (n.child_by_field_name("name"), n.child_by_field_name("type"))
                {
                    let a = text(name, src).to_owned();
                    let b = norm(text(ty, src));
                    if a != b {
                        all.aliases.entry(a).or_default().insert(b);
                    }
                }
            }
            "use_as_clause" => {
                if let (Some(path), Some(alias)) = (
                    n.child_by_field_name("path"),
                    n.child_by_field_name("alias"),
                ) {
                    all.aliases
                        .entry(text(alias, src).to_owned())
                        .or_default()
                        .insert(norm(text(path, src)));
                }
            }
            "struct_item" | "enum_item" | "union_item" => {
                if let Some(name) = n.child_by_field_name("name") {
                    all.aggregates.insert(text(name, src).to_owned());
                }
            }
            _ => {}
        }
        if owner.is_none() {
            continue;
        }
        if n.kind() == "call_expression" {
            if let Some(fun) = n.child_by_field_name("function") {
                if fun.kind() == "identifier" {
                    all.called
                        .entry(p.clone())
                        .or_default()
                        .insert(text(fun, src).to_owned());
                }
            }
        }
        if n.kind() == "function_signature_item" && in_foreign(n) {
            let ln = n.start_position().row;
            let line = lines.get(ln).copied().unwrap_or("");
            if nolint(line) {
                continue;
            }
            if let Some(nm) = n.child_by_field_name("name") {
                let ident = text(nm, src);
                let sym = symbol(n, src, ident);
                let s = sig(n, src);
                let r = ret(n, src);
                all.decls.entry(sym.clone()).or_default().push((
                    p.clone(),
                    ln + 1,
                    s.clone(),
                    s.variadic,
                    ident.to_owned(),
                ));
                all.ret_decl.entry(sym.clone()).or_default().push((
                    p.clone(),
                    ln + 1,
                    r,
                    ident.to_owned(),
                ));
            }
        }
        if n.kind() == "static_item" {
            if let (Some(nm), Some(ty)) =
                (n.child_by_field_name("name"), n.child_by_field_name("type"))
            {
                let ln = n.start_position().row;
                let sym = symbol(n, src, text(nm, src));
                let t = norm(text(ty, src));
                if in_foreign(n) {
                    if !nolint(lines.get(ln).copied().unwrap_or("")) {
                        all.statics_decl
                            .entry(sym)
                            .or_default()
                            .push((p.clone(), ln + 1, t));
                    }
                } else if has_export_attr(n, src) {
                    all.statics_def.entry(sym).or_default().push((
                        owner.unwrap().to_owned(),
                        p.clone(),
                        t,
                    ));
                }
            }
        }
        if n.kind() == "function_item" && has_export_attr(n, src) {
            if let Some(nm) = n.child_by_field_name("name") {
                let sym = symbol(n, src, text(nm, src));
                all.defs.entry(sym.clone()).or_default().push((
                    owner.unwrap().to_owned(),
                    p.clone(),
                    sig(n, src),
                    ret(n, src),
                ));
                all.ret_def.entry(sym).or_default().push((
                    owner.unwrap().to_owned(),
                    p.clone(),
                    ret(n, src),
                ));
            }
        }
    }
    Ok(())
}
fn has_export_attr(n: tree_sitter::Node<'_>, src: &str) -> bool {
    let mut p = n.prev_sibling();
    while let Some(a) = p {
        if a.kind() != "attribute_item" {
            break;
        }
        let t = text(a, src);
        if t.contains("no_mangle") || t.contains("export_name") {
            return true;
        }
        p = a.prev_sibling();
    }
    false
}
fn norm(s: &str) -> String {
    PATH_QUAL
        .replace_all(&s.split_whitespace().collect::<Vec<_>>().join(" "), "$1")
        .trim()
        .to_owned()
}
fn ptr_names(t: &str) -> bool {
    matches!(
        t,
        "usize"
            | "isize"
            | "ULONG_PTR"
            | "LONG_PTR"
            | "DWORD_PTR"
            | "SIZE_T"
            | "SSIZE_T"
            | "UINT_PTR"
            | "INT_PTR"
            | "KAFFINITY"
    )
}
fn base_width(t: &str) -> Option<usize> {
    Some(match t {
        "u8" | "i8" | "bool" | "c_char" | "c_uchar" | "c_schar" | "UCHAR" | "CHAR" | "BYTE"
        | "BOOLEAN" | "CCHAR" => 1,
        "u16" | "i16" | "c_short" | "c_ushort" | "USHORT" | "SHORT" | "WORD" | "WCHAR"
        | "LANGID" | "CSHORT" | "short" => 2,
        "u32" | "i32" | "f32" | "char" | "c_int" | "c_uint" | "c_long" | "c_ulong" | "ULONG"
        | "LONG" | "DWORD" | "ULONG32" | "LONG32" | "UINT" | "INT" | "NTSTATUS" | "HRESULT"
        | "BOOL" | "LOGICAL" | "int" => 4,
        "u64" | "i64" | "f64" | "c_longlong" | "c_ulonglong" | "ULONGLONG" | "LONGLONG"
        | "ULONG64" | "LONG64" | "DWORDLONG" | "DWORD64" | "LARGE_INTEGER" | "ULARGE_INTEGER" => 8,
        "u128" | "i128" => 16,
        _ => return None,
    })
}
fn resolve(t: &str, ty: &mut Types, stack: &mut Vec<String>) -> (Option<usize>, Option<String>) {
    let mut t = t.trim().to_owned();
    let mut d = 0usize;
    loop {
        if let Some(m) = PTR_HEAD.captures(&t) {
            d += 1;
            t = m[1].trim().to_owned();
            continue;
        }
        if let Some(m) = OPTION.captures(&t) {
            t = m[1].trim().to_owned();
            continue;
        }
        break;
    }
    if !IDENT.is_match(&t) {
        return if FN_PTR.is_match(&t) {
            (Some(d + 1), Some(PTR.into()))
        } else {
            (Some(d), None)
        };
    }
    if let Some(w) = base_width(&t) {
        return (Some(d), Some(w.to_string()));
    }
    if ptr_names(&t) {
        return (Some(d), Some(PTR.into()));
    }
    if stack.contains(&t) || stack.len() > 12 {
        return (None, None);
    }
    if let Some(c) = ty.cache.get(&t) {
        return c.as_ref().map_or((None, None), |(depth, width)| {
            (depth.map(|x| d + x), width.clone())
        });
    }
    if ty.aggregates.contains(&t) && ty.aliases.contains_key(&t) {
        return (None, None);
    }
    let Some(rhs) = ty.aliases.get(&t).cloned() else {
        return (if ty.aggregates.contains(&t) || matches!(t.as_str(), "c_void" | "void") {
            (Some(d), None)
        } else {
            (None, None)
        });
    };
    ty.cache.insert(t.clone(), None);
    stack.push(t.clone());
    let mut ds = BTreeSet::new();
    let mut ws = BTreeSet::new();
    for r in rhs {
        let (a, b) = resolve(&r, ty, stack);
        ds.insert(a);
        ws.insert(b);
    }
    stack.pop();
    let depth = if ds.len() == 1 {
        *ds.iter().next().unwrap()
    } else {
        None
    };
    let width = if ws.len() == 1 {
        ws.iter().next().unwrap().clone()
    } else {
        None
    };
    ty.cache.insert(t, Some((depth, width.clone())));
    (depth.map(|x| d + x), width)
}
fn bytes(w: &str) -> usize {
    if w == PTR {
        8
    } else {
        w.parse().unwrap_or(0)
    }
}
fn arch_ptr(path: &str) -> Option<usize> {
    let parts = Path::new(path)
        .components()
        .filter_map(|c| c.as_os_str().to_str());
    parts
        .into_iter()
        .find_map(|p| ARCHES.iter().find(|(a, _)| *a == p).map(|(_, w)| *w))
}
fn c_width(t: &str, ty: &mut Types) -> Option<String> {
    if t.ends_with('*') {
        return Some(PTR.into());
    }
    if let Some(w) = match t {
        "char" | "CHAR" | "UCHAR" | "BOOLEAN" | "CCHAR" | "BYTE" => Some(1),
        "short" | "SHORT" | "USHORT" | "WCHAR" | "CSHORT" | "WORD" => Some(2),
        "int" | "LONG" | "ULONG" | "DWORD" | "NTSTATUS" | "LOGICAL" | "BOOL" | "UINT" | "INT"
        | "ULONG32" | "LONG32" => Some(4),
        "LONGLONG" | "ULONGLONG" | "LONG64" | "ULONG64" | "DWORD64" | "LARGE_INTEGER"
        | "ULARGE_INTEGER" => Some(8),
        "ULONG_PTR" | "LONG_PTR" | "DWORD_PTR" | "SIZE_T" | "SSIZE_T" | "UINT_PTR" | "INT_PTR"
        | "KAFFINITY" | "PVOID" => return Some(PTR.into()),
        _ => None,
    } {
        return Some(w.to_string());
    }
    let (d, w) = resolve(t, ty, &mut vec![]);
    if d.is_none() {
        None
    } else if d.unwrap() > 0 {
        Some(PTR.into())
    } else {
        w
    }
}
fn read_undefined(decl: &str, def: &str) -> bool {
    bytes(def) > bytes(decl)
}
fn rust_owner(path: &str, dirs: &BTreeSet<PathBuf>) -> Option<String> {
    dirs.iter()
        .find(|d| Path::new(path).starts_with(d))
        .map(|d| d.to_string_lossy().to_string())
}
fn c_return_oracle(files: &[&SourceFile]) -> BTreeMap<String, (String, usize, String)> {
    let mut found = BTreeMap::new();
    let mut clash = BTreeSet::new();
    for f in files {
        let line_starts = line_starts(&f.text);
        let base = f.path.file_name().unwrap_or_default().to_string_lossy();
        if !base.to_ascii_lowercase().contains("cohab")
            && !base.to_ascii_lowercase().contains("rust")
        {
            continue;
        }
        for m in C_FN.captures_iter(&f.text) {
            let start = m.get(0).unwrap().start();
            let line_start = f.text[..start].rfind('\n').map_or(0, |at| at + 1);
            let prefix = f.text[line_start..start].trim_start();
            if ["static", "typedef", "return", "else"].iter().any(|word| {
                m.get(1).is_some_and(|ty| ty.as_str().trim() == *word)
                    || prefix.strip_prefix(word).is_some_and(|tail| {
                        tail.is_empty() || tail.chars().next().is_some_and(char::is_whitespace)
                    })
            }) {
                continue;
            }
            let ret_ty = format!(
                "{}{}",
                &m[1],
                if !m.get(2).unwrap().as_str().is_empty() || !m.get(3).unwrap().as_str().is_empty()
                {
                    "*"
                } else {
                    ""
                }
            );
            let name = m[4].to_owned();
            let line = line_number(&line_starts, start);
            if found
                .get(&name)
                .is_some_and(|(_, _, old): &(String, usize, String)| old != &ret_ty)
            {
                clash.insert(name.clone());
            }
            found.entry(name).or_insert((
                f.path.to_string_lossy().replace('\\', "/"),
                line,
                ret_ty,
            ));
        }
    }
    for n in clash {
        found.remove(&n);
    }
    found
}
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    starts
}
fn line_number(starts: &[usize], at: usize) -> usize {
    starts.partition_point(|&start| start <= at)
}
fn compare_statics(owner: &str, facts: &Facts, ty: &mut Types, out: &mut Vec<String>) {
    for (sym, defs) in &facts.statics_def {
        let local = defs
            .iter()
            .filter(|(o, _, _)| o == owner)
            .collect::<Vec<_>>();
        if local.is_empty() {
            continue;
        }
        let shapes = local
            .iter()
            .filter_map(|(_, _, t)| {
                let (d, w) = resolve(t, ty, &mut vec![]);
                d.map(|d| (d, if d > 0 { Some(PTR.to_owned()) } else { w }))
            })
            .collect::<BTreeSet<_>>();
        if shapes.len() != 1 {
            continue;
        }
        let (dd, dw) = shapes.iter().next().unwrap();
        let Some(ow_base) = dw.clone() else {
            continue;
        };
        let (dfile, dty) = (&local[0].1, &local[0].2);
        for (path, line, t) in facts.statics_decl.get(sym).into_iter().flatten() {
            if !Path::new(path).starts_with(owner) {
                continue;
            }
            let (Some(cd), Some(mut cw)) = ({
                let x = resolve(t, ty, &mut vec![]);
                (x.0, x.1)
            }) else {
                continue;
            };
            if cd > 0 {
                cw = PTR.to_owned();
            }
            let mut ow = ow_base.clone();
            let pointer = cw == PTR || ow == PTR;
            let ap = arch_ptr(path);
            if cw == PTR && ap.is_some() {
                cw = ap.unwrap().to_string();
            }
            if ow == PTR && ap.is_some() {
                ow = ap.unwrap().to_string();
            }
            if !read_undefined(&ow, &cw) {
                continue;
            }
            let pointer = if pointer {
                " [pointer-sized: i386 differs]"
            } else {
                ""
            };
            out.push(format!("{path}:{line}: {sym}: static WIDTH decl `{t}` ({cw}) wider than def `{dty}` ({ow}){pointer} ({})",Path::new(dfile).file_name().unwrap_or_default().to_string_lossy()));
        }
        let _ = dd;
    }
}
fn compare_returns(
    owner: &str,
    facts: &Facts,
    c_rets: &BTreeMap<String, (String, usize, String)>,
    ty: &mut Types,
    out: &mut Vec<String>,
) {
    const BOOLS: &[&str] = &["BOOL", "BOOLEAN", "bool", "WINBOOL", "u8", "UCHAR", "BYTE"];
    for (sym, sites) in &facts.ret_decl {
        for (path, line, rty, ident) in sites {
            if !Path::new(path).starts_with(owner)
                || rty.is_empty()
                || !facts.called.get(path).is_some_and(|s| s.contains(ident))
            {
                continue;
            }
            let local = facts
                .ret_def
                .get(sym)
                .into_iter()
                .flatten()
                .filter(|(o, _, _)| o == owner)
                .collect::<Vec<_>>();
            let (dw, authority, dty) = if !local.is_empty() {
                let shapes = local
                    .iter()
                    .map(|(_, _, t)| c_width(t, ty))
                    .collect::<BTreeSet<_>>();
                if shapes.len() != 1 {
                    continue;
                }
                let Some(w) = shapes.iter().next().unwrap().clone() else {
                    continue;
                };
                (
                    w,
                    format!(
                        "def `-> {}` ({})",
                        local[0].2,
                        Path::new(&local[0].1)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ),
                    local[0].2.clone(),
                )
            } else if let Some((cf, cl, ct)) = c_rets.get(sym) {
                if ct == "void" || ct == "VOID" {
                    continue;
                }
                let Some(w) = c_width(ct, ty) else {
                    continue;
                };
                (
                    w,
                    format!(
                        "C `{ct}` ({}:{cl})",
                        Path::new(cf)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ),
                    ct.clone(),
                )
            } else {
                continue;
            };
            let Some(mut cw) = c_width(rty, ty) else {
                continue;
            };
            let mut ow = dw.clone();
            let pointer = cw == PTR || ow == PTR;
            let ap = arch_ptr(path);
            if cw == PTR && ap.is_some() {
                cw = ap.unwrap().to_string();
            }
            if ow == PTR && ap.is_some() {
                ow = ap.unwrap().to_string();
            }
            if read_undefined(&ow, &cw) {
                out.push(format!("{path}:{line}: {sym}: RETURN WIDTH decl `-> {rty}` ({cw}) wider than {authority} ({ow}){}",if pointer{" [pointer-sized: i386 differs]"}else{""}));
            } else if bytes(&cw) < bytes(&ow)
                && (ow == PTR
                    || bytes(&ow) >= 8
                    || (bytes(&cw) == 1 && !BOOLS.contains(&norm(&dty).as_str())))
            {
                out.push(format!("{path}:{line}: {sym}: RETURN NARROWED decl `-> {rty}` ({cw}) drops bits of {authority} ({ow})"));
            }
        }
    }
}

impl Rule for ExternDeclDrift {
    fn name(&self) -> &'static str {
        "scan_extern_decl_drift"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut facts = Facts::default();
        let manifests = c
            .sources("drift_manifests")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .collect::<Vec<_>>();
        let crate_dirs = manifests
            .iter()
            .map(|m| m.path.parent().unwrap().to_path_buf())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with("-rust"))
            })
            .collect::<BTreeSet<_>>();
        let mut sys_deps = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
        for m in &manifests {
            let dir = m.path.parent().unwrap();
            if !dir
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with("-rust"))
            {
                continue;
            }
            let deps = crate::regexes::cached(r"(?m)^\s*([A-Za-z0-9_-]+)\s*=")
                .unwrap()
                .captures_iter(&m.text)
                .map(|x| x[1].to_owned())
                .filter(|n| n.ends_with("-sys"))
                .filter_map(|n| {
                    let p = dir.parent()?.join(n).join("src");
                    c.files.iter().any(|f| f.path.starts_with(&p)).then_some(p)
                })
                .collect();
            sys_deps.insert(dir.join("src"), deps);
        }
        let mut islands = BTreeMap::<PathBuf, BTreeMap<String, Vec<(usize, Sig, String)>>>::new();
        // Resolve each bindgen file's first sys-crate owner once. Iteration order
        // matches the legacy BTreeMap flatten/find selection exactly.
        let files = c
            .sources("drift_rust")
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .collect::<Vec<_>>();
        for f in &files {
            let src = f.path.to_string_lossy().replace('\\', "/");
            let owner = f.path.ancestors().find(|d| crate_dirs.contains(*d));
            let owner_key = owner.map(|p| p.to_string_lossy().to_string());
            file_facts(f, owner_key.as_deref(), &mut facts)?;
            if owner.is_none()
                && f.path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().contains("bindgen"))
            {
                if let Some(sd) = sys_deps.values().flatten().find(|d| f.path.starts_with(d)) {
                    let mut local = Facts::default();
                    file_facts(f, Some("island"), &mut local)?;
                    let entry = islands.entry(sd.clone()).or_default();
                    for (n, rows) in local.decls {
                        entry
                            .entry(n)
                            .or_default()
                            .extend(rows.into_iter().map(|(_, ln, s, _, id)| (ln, s, id)));
                    }
                }
            }
            let _ = src;
        }
        let mut types = Types {
            aliases: facts.aliases.clone(),
            aggregates: facts.aggregates.clone(),
            cache: BTreeMap::new(),
        };
        let mut out = Vec::<String>::new();
        let cfiles = c
            .sources("drift_c")
            .filter(|f| f.path.extension().is_some_and(|e| e == "c"))
            .collect::<Vec<_>>();
        let c_rets = c_return_oracle(&cfiles);
        for (crate_src, deps) in &sys_deps {
            let crate_owner = crate_src.parent().unwrap_or(crate_src).to_string_lossy();
            compare_statics(&crate_owner, &facts, &mut types, &mut out);
            compare_returns(&crate_owner, &facts, &c_rets, &mut types, &mut out);
            let mut oracle =
                BTreeMap::<String, (Vec<(Option<usize>, Option<String>)>, Vec<String>)>::new();
            let mut all_sites = BTreeMap::<String, Vec<&(usize, Sig, String)>>::new();
            for dep in deps {
                for (sym, sites) in islands.get(dep).into_iter().flatten() {
                    all_sites.entry(sym.clone()).or_default().extend(sites);
                }
            }
            for (sym, sites) in all_sites {
                let mut shapes = BTreeSet::new();
                let mut pick = None;
                let mut valid = true;
                for (_line, s, _ident) in sites {
                    if s.variadic {
                        valid = false;
                        break;
                    }
                    let shape = s
                        .params
                        .iter()
                        .map(|t| resolve(t, &mut types, &mut vec![]))
                        .collect::<Vec<_>>();
                    shapes.insert(shape.clone());
                    pick = Some((shape, s.params.clone()));
                }
                if valid && shapes.len() == 1 {
                    if let Some(p) = pick {
                        oracle.insert(sym, p);
                    }
                }
            }
            for (sym, (shapes, params)) in oracle {
                if facts.defs.get(&sym).is_some_and(|defs| {
                    defs.iter()
                        .any(|(def_owner, _, _, _)| def_owner == crate_owner.as_ref())
                }) {
                    continue;
                }
                for (path, line, s, var, ident) in facts.decls.get(&sym).into_iter().flatten() {
                    if !Path::new(path).starts_with(crate_src)
                        || *var
                        || !facts.called.get(path).is_some_and(|x| x.contains(ident))
                    {
                        continue;
                    }
                    compare(
                        c, &mut types, &mut out, &sym, path, *line, s, &params, &shapes, "bindgen",
                    );
                }
            }
            for (sym, defs) in &facts.defs {
                // Python's per-crate `defs[name] = ...` keeps the last
                // definition encountered for a symbol. Match that authority
                // selection instead of treating every duplicate as independent.
                let Some((_, dpath, ds, dr)) = defs
                    .iter()
                    .rev()
                    .find(|(def_owner, _, _, _)| def_owner == crate_owner.as_ref())
                else {
                    continue;
                };
                for (path, line, s, var, ident) in facts.decls.get(sym).into_iter().flatten() {
                    if !Path::new(path).starts_with(crate_src)
                        || *var
                        || ds.variadic
                        || !facts.called.get(path).is_some_and(|x| x.contains(ident))
                    {
                        continue;
                    }
                    if ds.params.len() != s.params.len() {
                        out.push(format!(
                            "{path}:{line}: {sym}: ARITY decl {} vs def {} ({})",
                            s.params.len(),
                            ds.params.len(),
                            Path::new(dpath).file_name().unwrap().to_string_lossy()
                        ));
                        continue;
                    }
                    for (i, (ct, dt)) in s.params.iter().zip(&ds.params).enumerate() {
                        let (cd, mut cw) = resolve(ct, &mut types, &mut vec![]);
                        let (dd, mut dw) = resolve(dt, &mut types, &mut vec![]);
                        let ptr = arch_ptr(path);
                        if let Some(ptr) = ptr {
                            if cw.as_deref() == Some(PTR) {
                                cw = Some(ptr.to_string());
                            }
                            if dw.as_deref() == Some(PTR) {
                                dw = Some(ptr.to_string());
                            }
                        }
                        if let Some(m) = compare_param(
                            &sym,
                            i,
                            ct,
                            dt,
                            cd,
                            cw,
                            dd,
                            dw,
                            "def",
                            Some(
                                Path::new(dpath)
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .as_ref(),
                            ),
                        ) {
                            out.push(format!("{path}:{line}: {m}"));
                        }
                    }
                    let _ = dr;
                }
            }
        }
        out.sort();
        out.dedup();
        Ok(out)
    }
}
fn compare_param(
    sym: &str,
    i: usize,
    ct: &str,
    ot: &str,
    cd: Option<usize>,
    cw: Option<String>,
    od: Option<usize>,
    ow: Option<String>,
    oracle: &str,
    file: Option<&str>,
) -> Option<String> {
    let (Some(cd), Some(od)) = (cd, od) else {
        return None;
    };
    let shallow_w = if cd < od {
        cw.as_deref()
    } else {
        ow.as_deref()
    };
    if cd.abs_diff(od) == 1 && shallow_w == Some(PTR) {
        return None;
    }
    if cd != od {
        if cd.abs_diff(od) == 1
            && if cd < od {
                cw.as_deref()
            } else {
                ow.as_deref()
            } == Some(PTR)
        {
            return None;
        }
        return Some(format!(
            "{sym}: param {i} decl `{ct}` (*{cd}) vs {oracle} `{ot}` (*{od}){}",
            file.map_or(String::new(), |x| format!(" ({x})"))
        ));
    }
    let (Some(cw), Some(ow)) = (cw, ow) else {
        return None;
    };
    let bytes = |w: &str| if w == PTR { 8 } else { w.parse().unwrap_or(0) };
    if cw == ow || (cd == 0 && bytes(&ow) <= bytes(&cw)) {
        return None;
    }
    Some(format!(
        "{sym}: param {i} WIDTH decl `{ct}` ({cw}) vs {oracle} `{ot}` ({ow}){}",
        file.map_or(String::new(), |x| format!(" ({x})"))
    ))
}
fn compare(
    c: &Corpus,
    ty: &mut Types,
    out: &mut Vec<String>,
    sym: &str,
    path: &str,
    line: usize,
    s: &Sig,
    params: &[String],
    shapes: &[(Option<usize>, Option<String>)],
    oracle: &str,
) {
    if s.params.len() != shapes.len() {
        out.push(format!(
            "{path}:{line}: {sym}: ARITY decl {} vs {oracle} {}",
            s.params.len(),
            shapes.len()
        ));
        return;
    }
    let ptr = arch_ptr(path);
    for (i, (ct, (od, ow))) in s.params.iter().zip(shapes).enumerate() {
        let (cd, mut cw) = resolve(ct, ty, &mut vec![]);
        let mut ow = ow.clone();
        let pointer_sized = cw.as_deref() == Some(PTR) || ow.as_deref() == Some(PTR);
        if let Some(ptr) = ptr {
            if cw.as_deref() == Some(PTR) {
                cw = Some(ptr.to_string());
            }
            if ow.as_deref() == Some(PTR) {
                ow = Some(ptr.to_string());
            }
        }
        let msg = compare_param(sym, i, ct, &params[i], cd, cw, *od, ow, oracle, None);
        if let Some(m) = msg {
            let _ = pointer_sized;
            out.push(format!("{path}:{line}: {m}"));
        }
    }
    let _ = c;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_c_lines_match_prefix_counting_at_boundaries() {
        let src = "first\nsecond\n\nlast";
        let starts = line_starts(src);
        for at in [0, 5, 6, 12, 13, src.len()] {
            let old = src[..at].bytes().filter(|b| *b == b'\n').count() + 1;
            assert_eq!(line_number(&starts, at), old);
        }
    }

    fn run(items: &[(&str, &str)]) -> Vec<String> {
        let files = items
            .iter()
            .map(|(p, s)| SourceFile::new((*p).into(), (*s).into()))
            .collect();
        ExternDeclDrift
            .run(&Corpus::new(PathBuf::from("/repo"), files))
            .unwrap()
    }
    fn fixture(dep: bool, decl: &str, call: &str) -> Vec<(String, String)> {
        let dep_line = if dep {
            "bar-sys = { path = \"../bar-sys\" }\n"
        } else {
            ""
        };
        vec![
            ("bar-sys/Cargo.toml".into(), "[package]\nname = \"bar-sys\"\n".into()),
            ("bar-sys/src/bindgen_i386.rs".into(), "pub type SIZE_T = usize;\npub type ULONG = u32;\nextern \"C\" {\n pub fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: SIZE_T) -> *mut u8;\n}\n".into()),
            ("foo-rust/Cargo.toml".into(), format!("[package]\nname = \"foo-rust\"\n[dependencies]\n{dep_line}")),
            ("foo-rust/src/lib.rs".into(), format!("pub type SIZE_T = usize;\npub type ULONG = u32;\nextern \"C\" {{\n {decl}\n}}\npub unsafe fn go() {{ {call} }}\n")),
        ]
    }
    fn run_owned(items: Vec<(String, String)>) -> Vec<String> {
        let refs = items
            .iter()
            .map(|(p, s)| (p.as_str(), s.as_str()))
            .collect::<Vec<_>>();
        run(&refs)
    }

    #[test]
    fn substrate_width_drift_requires_dependency_and_call() {
        let bad = "fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: u32) -> *mut u8;";
        let call = "ExAllocatePoolWithTag(0, 8);";
        let got = run_owned(fixture(true, bad, call));
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].contains("ExAllocatePoolWithTag: param 1 WIDTH"));
        assert!(run_owned(fixture(
            true,
            "fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: SIZE_T) -> *mut u8;",
            call
        ))
        .is_empty());
        assert!(run_owned(fixture(false, bad, call)).is_empty());
        assert!(run_owned(fixture(true, bad, "")).is_empty());
    }

    #[test]
    fn arch_pointer_width_resolves_inside_arch_module() {
        let bad = "fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: u32) -> *mut u8;";
        let mut items = fixture(true, bad, "ExAllocatePoolWithTag(0, 8);");
        let src = items
            .iter_mut()
            .find(|(p, _)| p == "foo-rust/src/lib.rs")
            .unwrap();
        src.0 = "foo-rust/src/i386/lib.rs".into();
        assert!(run_owned(items).is_empty());
    }

    #[test]
    fn static_and_c_return_widths_are_reported_but_matching_width_is_clean() {
        let items = [
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            (
                "foo-rust/src/lib.rs",
                "pub type ULONG = u32;\npub type BOOLEAN = u8;\n#[no_mangle]\npub static mut Flag: BOOLEAN = 0;\nextern \"C\" { static Flag: ULONG; }\n",
            ),
            (
                "drv/rust_cohab.c",
                "UCHAR FatIndexBitSize(PBIOS_PARAMETER_BLOCK B){ return __c_FatIndexBitSize(B); }\n",
            ),
        ];
        let got = run(&items);
        assert!(
            got.iter().any(|x| x.contains("Flag: static WIDTH")),
            "{got:?}"
        );
    }

    #[test]
    fn per_arch_static_definitions_that_disagree_are_not_an_oracle() {
        let got = run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            ("foo-rust/src/lib.rs", "pub type ULONG = u32;\n#[no_mangle]\npub static mut Probe: ULONG = 0;\n#[no_mangle]\npub static mut Probe: usize = 0;\nextern \"C\" { static Probe: usize; }\n"),
        ]);
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn c_return_width_is_reported_and_matching_width_is_clean() {
        let c =
            "UCHAR FatIndexBitSize(PBIOS_PARAMETER_BLOCK B){ return __c_FatIndexBitSize(B); }\n";
        let decl = "extern \"C\" { fn FatIndexBitSize(b: *const u8) -> u32; }\npub unsafe fn go() { FatIndexBitSize(core::ptr::null()); }\n";
        let got = run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            ("foo-rust/src/lib.rs", decl),
            ("drv/rust_cohab.c", c),
        ]);
        assert!(
            got.iter().any(|x| x
                .contains("FatIndexBitSize: RETURN WIDTH decl `-> u32` (4) wider than C `UCHAR`")),
            "{got:?}"
        );
        let clean = decl.replace("-> u32", "-> u8");
        assert!(run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            ("foo-rust/src/lib.rs", &clean),
            ("drv/rust_cohab.c", c),
        ])
        .is_empty());
    }

    #[test]
    fn c_int_return_narrowed_to_byte_is_reported() {
        let got = run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n[dependencies]\nntdll-sys = { path = \"../ntdll-sys\" }\n"),
            ("foo-rust/src/lib.rs", "extern \"C\" { fn IsEqualGUID(a: *const u8, b: *const u8) -> u8; }\npub unsafe fn go() { IsEqualGUID(core::ptr::null(), core::ptr::null()); }\n"),
            ("ntdll-sys/src/bindgen.rs", "pub type ULONG = u32;\n"),
            ("drv/rust_cohab_stub.c", "int __stdcall IsEqualGUID(const void *a, const void *b) { return 1; }\n"),
        ]);
        assert!(
            got.iter()
                .any(|x| x.contains("IsEqualGUID: RETURN NARROWED")),
            "{got:?}"
        );
    }

    #[test]
    fn real_mlang_is_equal_guid_fixture() {
        let rust = include_str!("/home/curi/Projects/os-playground/vibeos-lint/sdk/lib/rust/mlang-rust/src/mlang/mlang.rs");
        let c = include_str!(
            "/home/curi/Projects/os-playground/vibeos-lint/dll/win32/mlang/rust_cohab_stub.c"
        );
        let manifest = include_str!(
            "/home/curi/Projects/os-playground/vibeos-lint/sdk/lib/rust/mlang-rust/Cargo.toml"
        );
        let got = run(&[
            ("sdk/lib/rust/mlang-rust/Cargo.toml", manifest),
            ("sdk/lib/rust/mlang-rust/src/mlang/mlang.rs", rust),
            (
                "sdk/lib/rust/ntdll-sys/src/bindgen.rs",
                "pub type ULONG = u32;\n",
            ),
            (
                "sdk/lib/rust/other-rust/Cargo.toml",
                "[package]\nname = \"other-rust\"\n[dependencies]\nntdll-sys = { path = \"../ntdll-sys\" }\n",
            ),
            (
                "sdk/lib/rust/other-rust/src/lib.rs",
                "#[no_mangle]\npub extern \"C\" fn IsEqualGUID(a: *const u8, b: *const u8) -> bool { true }\n",
            ),
            ("dll/win32/mlang/rust_cohab_stub.c", c),
        ]);
        assert!(
            got.iter()
                .any(|x| x.contains("mlang.rs:1816: IsEqualGUID: RETURN NARROWED")),
            "{got:?}"
        );
    }

    #[test]
    fn declaration_nolint_suppresses_only_that_declaration() {
        let decl = "extern \"C\" { fn Hit() -> ULONG; } // nolint: decl-drift (reviewed)\n";
        let source = "pub type ULONG = u32;\n#[no_mangle]\npub extern \"C\" fn Hit() -> u8 { 1 }\n"
            .to_owned()
            + decl
            + "pub unsafe fn go() { Hit(); }\n";
        let got = run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            ("foo-rust/src/lib.rs", &source),
        ]);
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn rust_definition_width_and_arity_are_checked_only_for_called_declarations() {
        let good = "#[no_mangle]\npub extern \"C\" fn Hit() -> u8 { 1 }\nextern \"C\" { fn Hit() -> ULONG; }\npub unsafe fn go() { Hit(); }\n";
        let got = run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            ("foo-rust/src/lib.rs", good),
        ]);
        assert!(got.iter().any(|x|x.contains("Hit: RETURN WIDTH decl `-> ULONG` (4) wider than def `-> u8`")),"{got:?}");
        let uncalled = good.replace("Hit();", "");
        assert!(run(&[
            ("foo-rust/Cargo.toml", "[package]\nname = \"foo-rust\"\n"),
            ("foo-rust/src/lib.rs", &uncalled)
        ])
        .is_empty());
    }
}
