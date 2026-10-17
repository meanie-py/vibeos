//! Port of rustconv.verify_ffi: conventions, extern arity, macro imports and flex arrays.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct VerifyFfi;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("ffi-rust", &["sdk/lib/rust"], &["rs"], &["Cargo.toml"])
        .with_pruning(&["target"]),
    SourceSet::tree("ffi-spec", &["ntoskrnl"], &["spec"], &[]),
];
static SPEC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^@\s+(stdcall|cdecl|fastcall)\s+(?:-arch=[^\s]+\s+)?(\w+)\s*\(([^)]*)\)").unwrap()
});
static EXTERN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*(?:pub\s+)?unsafe\s+extern\s+"(\w+)""#).unwrap());
static FN_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*pub\s+fn\s+(\w+)\s*\(").unwrap());
static LOCAL_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"#\[no_mangle\]\s*(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?(?:extern\s+"[A-Za-z-]+"\s+)?fn\s+(\w+)\s*[(<]"#).unwrap()
});
static FLEX: &[(&str, &str)] = &[
    ("_DEVICE_RELATIONS", "Objects"),
    ("DEVICE_RELATIONS", "Objects"),
    ("_CM_RESOURCE_LIST", "List"),
    ("CM_RESOURCE_LIST", "List"),
    ("_CM_FULL_RESOURCE_DESCRIPTOR", "PartialResourceList"),
    ("_IO_RESOURCE_REQUIREMENTS_LIST", "List"),
    ("_IO_RESOURCE_LIST", "Descriptors"),
    ("_KEY_VALUE_FULL_INFORMATION", "Data"),
    ("_KEY_VALUE_PARTIAL_INFORMATION", "Data"),
    ("_FILE_BOTH_DIR_INFORMATION", "FileName"),
    ("_FILE_DIRECTORY_INFORMATION", "FileName"),
    ("_FILE_FULL_DIR_INFORMATION", "FileName"),
    ("_FILE_NAMES_INFORMATION", "FileName"),
];
const MACROS: &[&str] = &[
    "CONTAINING_RECORD",
    "InitializeObjectAttributes",
    "IoGetCurrentIrpStackLocation",
    "IoGetNextIrpStackLocation",
    "IoSizeOfIrp",
    "ObReferenceObject",
    "ObDereferenceObject",
    "ExRaiseStatus",
    "IoSkipCurrentIrpStackLocation",
    "IoCopyCurrentIrpStackLocationToNext",
    "IoSetCompletionRoutine",
    "IoSetNextIrpStackLocation",
    "IoMarkIrpPending",
    "InsertTailList",
    "InsertHeadList",
    "RemoveHeadList",
    "RemoveTailList",
    "RemoveEntryList",
    "IsListEmpty",
    "InitializeListHead",
    "KeAcquireSpinLock",
    "KeReleaseSpinLock",
    "ExInitializeWorkItem",
    "RtlZeroMemory",
    "RtlCopyMemory",
    "RtlFillMemory",
    "RtlMoveMemory",
];
const MACRO_EXCEPTIONS: &[&str] = &[
    "ObfReferenceObject",
    "ObfDereferenceObject",
    "RtlZeroMemory",
    "RtlFillMemory",
    "RtlMoveMemory",
    "RtlCopyMemory",
];
const CDECL: &[&str] = &[
    "sprintf",
    "swprintf",
    "snprintf",
    "wcslen",
    "strlen",
    "strstr",
    "strcmp",
    "wcscmp",
    "memcpy",
    "memset",
    "memmove",
    "memcmp",
    "IsEqualGUID",
    "DbgPrint",
    "_snprintf",
    "_snwprintf",
];
const SHIM: &[&str] = &["_shim", "_c"];
#[derive(Clone, Debug)]
struct Ref {
    conv: String,
    params: usize,
}
fn spec(s: &str) -> BTreeMap<String, Ref> {
    let mut out = BTreeMap::new();
    for l in s.lines() {
        if let Some(c) = SPEC_RE.captures(l.trim()) {
            let args = c[3].trim();
            out.insert(
                c[2].into(),
                Ref {
                    conv: c[1].into(),
                    params: if args.is_empty() {
                        0
                    } else {
                        args.split_whitespace().count()
                    },
                },
            );
        }
    }
    out
}
fn count_params(d: &str) -> usize {
    let d = d.replace("->", "→");
    let Some(op) = d.find('(') else { return 0 };
    let mut depth = 0i32;
    let mut close = None;
    for (i, ch) in d[op..].char_indices() {
        let j = op + i;
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(j);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(end) = close else { return 0 };
    let mut n = 0;
    depth = 0;
    let mut cur = String::new();
    let inner = &d[op + 1..end];
    for ch in inner.chars() {
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth -= 1,
            ',' if depth == 0 => {
                if cur.contains(':') {
                    n += 1
                }
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    if cur.contains(':') {
        n += 1
    }
    n
}
fn bindgen(s: &str) -> BTreeMap<String, Ref> {
    let mut out = BTreeMap::new();
    let (mut conv, mut name, mut buf): (Option<String>, Option<String>, Option<String>) =
        (None, None, None);
    for line in s.lines() {
        if buf.is_none() {
            if let Some(c) = EXTERN_RE.captures(line) {
                conv = Some(c[1].into());
                continue;
            }
            if line.trim() == "}" {
                conv = None;
                continue;
            }
            if let Some(c) = FN_START.captures(line) {
                if let Some(cv) = conv.clone() {
                    name = Some(c[1].into());
                    buf = Some(line.trim().to_owned());
                    let _ = cv;
                }
            }
        } else if let Some(b) = buf.as_mut() {
            b.push(' ');
            b.push_str(line.trim());
        }
        if buf.as_ref().is_some_and(|x| x.trim_end().ends_with(';')) {
            let b = buf.take().unwrap();
            let n = name.take().unwrap();
            let cv = conv.clone().unwrap_or_default();
            out.insert(
                n,
                Ref {
                    conv: cv,
                    params: count_params(&b),
                },
            );
        }
    }
    out
}
fn params(decl: &str) -> usize {
    count_params(decl)
}
fn line_issues(
    file: &SourceFile,
    spec: &BTreeMap<String, Ref>,
    bg: &BTreeMap<String, Ref>,
    local: &[String],
) -> Vec<String> {
    let path = file.path.to_string_lossy().replace('\\', "/");
    if path.ends_with("kernel-rust/src/lib.rs") {
        return vec![];
    }
    let stem = file.path.file_stem().unwrap_or_default().to_string_lossy();
    let pathstr = path.as_str();
    let skip_file = stem.ends_with("_amd64")
        || stem.ends_with("_arm64")
        || pathstr
            .split('/')
            .any(|x| matches!(x, "amd64" | "arm64" | "arm"));
    let mut out = vec![];
    let mut in_ext = false;
    let (mut conv, mut depth, mut blockskip): (String, i32, bool) = (String::new(), 0, false);
    let mut pending_link: Option<String> = None;
    let mut pending_cfg = String::new();
    let mut fn_buf: Option<String> = None;
    let (mut fn_line, mut fn_name, mut rust_name) = (0usize, String::new(), String::new());
    let re_ext =
        Regex::new(r#"^(?:pub\s+)?(?:unsafe\s+)?extern\s+"(C|system|stdcall|cdecl)"\s*\{"#)
            .unwrap();
    let re_rejoin = Regex::new(
        r#"^\}\s*(?:pub\s+)?(?:unsafe\s+)?extern\s+"(C|system|stdcall|cdecl)"\s*\{\s*$"#,
    )
    .unwrap();
    let re_fn = Regex::new(r"^(?:pub\s+)?fn\s+(\w+)\s*\(").unwrap();
    let re_link = Regex::new(r#"#\[link_name\s*=\s*"(\w+)"\]"#).unwrap();
    let re_static = Regex::new(r"^(?:pub\s+)?(?:static\s+(?:mut\s+)?)\w+").unwrap();
    let re_cfg = Regex::new(r"#\[cfg\((.*)\)\]").unwrap();
    for (idx, line) in file.text.lines().enumerate() {
        let lineno = idx + 1;
        let stripped = line.split("//").next().unwrap_or("").trim();
        if !in_ext {
            if let Some(c) = re_cfg.captures(stripped) {
                pending_cfg = c[1].into();
                continue;
            }
            if let Some(c) = re_ext.captures(stripped) {
                conv = c[1].into();
                depth = stripped.matches('{').count() as i32 - stripped.matches('}').count() as i32;
                in_ext = depth > 0;
                let cfg = pending_cfg.replace(' ', "");
                blockskip = skip_file
                    || (cfg.contains("not(target_arch=\"x86\")") || {
                        let a: Vec<_> = Regex::new(r#"target_arch="(\w+)""#)
                            .unwrap()
                            .captures_iter(&cfg)
                            .map(|x| x[1].to_owned())
                            .collect();
                        !a.is_empty() && !a.iter().any(|x| x == "x86") && !cfg.contains("not(")
                    });
                pending_cfg.clear();
                if !in_ext {
                    let inner = stripped
                        .split_once('{')
                        .map(|x| x.1)
                        .unwrap_or("")
                        .rsplit_once('}')
                        .map(|x| x.0)
                        .unwrap_or("")
                        .trim();
                    if let Some(f) = re_fn.captures(inner) {
                        if inner.trim_end().ends_with(';') {
                            check(
                                &mut out,
                                &path,
                                lineno,
                                &f[1],
                                &f[1],
                                &conv,
                                params(inner),
                                spec,
                                bg,
                                blockskip,
                                local,
                            )
                        }
                    }
                }
                continue;
            }
            if !stripped.is_empty() && !stripped.starts_with("#[") {
                pending_cfg.clear()
            }
            continue;
        }
        if depth == 1 {
            if let Some(c) = re_rejoin.captures(stripped) {
                conv = c[1].into();
                pending_link = None;
                fn_buf = None;
                continue;
            }
        }
        depth += stripped.matches('{').count() as i32 - stripped.matches('}').count() as i32;
        if depth <= 0 {
            in_ext = false;
            conv.clear();
            pending_link = None;
            fn_buf = None;
            continue;
        }
        if let Some(c) = re_link.captures(stripped) {
            pending_link = Some(c[1].into());
            continue;
        }
        if fn_buf.is_none() {
            if re_static.is_match(stripped) {
                pending_link = None;
                continue;
            }
            let Some(f) = re_fn.captures(stripped) else {
                if !stripped.is_empty() && !stripped.starts_with("#[") {
                    pending_link = None
                }
                continue;
            };
            rust_name = f[1].into();
            fn_name = pending_link.take().unwrap_or_else(|| rust_name.clone());
            fn_line = lineno;
            fn_buf = Some(stripped.into());
            if MACROS.contains(&fn_name.as_str()) && !MACRO_EXCEPTIONS.contains(&fn_name.as_str()) {
                out.push(format!("[C_MACRO] {path}:{lineno}: {rust_name}: {fn_name} is a C macro, not a real function — should be inlined in Rust"));
            }
        } else if let Some(b) = fn_buf.as_mut() {
            b.push(' ');
            b.push_str(stripped)
        }
        if fn_buf.as_ref().is_some_and(|x| x.trim_end().ends_with(';')) {
            let b = fn_buf.take().unwrap();
            let slots = Regex::new(r"\(\s*_\s*:\s*usize").unwrap().is_match(&b);
            check(
                &mut out,
                &path,
                fn_line,
                &fn_name,
                &rust_name,
                &conv,
                if slots { 0 } else { params(&b) },
                spec,
                bg,
                blockskip,
                local,
            );
        }
    }
    out
}
fn check(
    out: &mut Vec<String>,
    path: &str,
    line: usize,
    name: &str,
    rust: &str,
    conv: &str,
    n: usize,
    spec: &BTreeMap<String, Ref>,
    bg: &BTreeMap<String, Ref>,
    skip: bool,
    local: &[String],
) {
    if CDECL.contains(&name) || SHIM.iter().any(|s| name.ends_with(s)) {
        return;
    }
    if conv == "C" && !skip && !local.iter().any(|x| x == name) {
        let r = spec
            .get(name)
            .filter(|x| matches!(x.conv.as_str(), "stdcall" | "fastcall"))
            .map(|x| (x, "ntoskrnl.spec"))
            .or_else(|| {
                bg.get(name)
                    .filter(|x| matches!(x.conv.as_str(), "stdcall" | "system"))
                    .map(|x| (x, "bindgen_i386.rs"))
            });
        if let Some((r, src)) = r {
            out.push(format!("[ABI] {path}:{line}: {rust}: declared extern \"C\" (cdecl), should be {} (per {src})",r.conv));
        }
    }
    if MACROS.contains(&name) || n == 0 {
        return;
    }
    let r = bg
        .get(name)
        .map(|x| (x, "bindgen_i386.rs"))
        .or_else(|| spec.get(name).map(|x| (x, "ntoskrnl.spec")));
    if let Some((r, src)) = r {
        if r.params > 0 {
            let mismatch = if src == "ntoskrnl.spec" {
                n > r.params
            } else {
                n != r.params
            };
            if mismatch {
                out.push(format!(
                    "[PARAM_COUNT] {path}:{line}: {rust}: Rust declares {n} params, {src} has {}",
                    r.params
                ));
            }
        }
    }
}
fn flex_issues(file: &SourceFile) -> Vec<String> {
    let p = file.path.to_string_lossy().replace('\\', "/");
    let mut out = vec![];
    for (li, line) in file.text.lines().enumerate() {
        for (structname, field) in FLEX {
            let pat = Regex::new(&format!(r"\.{field}\s*\[")).unwrap();
            let field_matches: Vec<_> = pat.find_iter(line).collect();
            if field_matches.is_empty() {
                continue;
            }
            let zero = Regex::new(&format!(
                r"\.{field}\s*\[\s*0\s*(?:as\s+usize\s*|usize\s*)?\]"
            ))
            .unwrap();
            if zero.is_match(line) {
                continue;
            }
            let array = Regex::new(&format!(r"\[([^\[\]]+)\]\s*\.{field}\s*\[")).unwrap();
            if array.captures_iter(line).any(|m| {
                !Regex::new(r"^\s*0\s*(?:as\s+usize\s*)?$")
                    .unwrap()
                    .is_match(&m[1])
            }) {
                continue;
            }
            if *field == "Data"
                && field_matches
                    .iter()
                    .all(|m| line[..m.start()].ends_with("DevicePrivate"))
            {
                continue;
            }
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with('*') {
                continue;
            }
            out.push(format!("[FLEX_ARRAY] {p}:{}: {field}: Direct indexing of flexible array {structname}.{field} — use pointer arithmetic instead (bindgen declares as [T; 1])",li+1));
        }
    }
    out
}
fn generated(s: &str) -> bool {
    s.lines()
        .next()
        .unwrap_or("")
        .contains("automatically generated by rust-bindgen")
}
fn crate_root(path: &Path, cargo_dirs: &BTreeSet<PathBuf>) -> PathBuf {
    path.ancestors()
        .find(|p| cargo_dirs.contains(*p))
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.parent().unwrap_or(Path::new("")).to_path_buf())
}
fn files(c: &Corpus) -> Vec<&SourceFile> {
    let mut f: Vec<_> = c
        .sources("ffi-rust")
        .filter(|f| {
            f.path.extension().is_some_and(|x| x == "rs")
                && f.path
                    .components()
                    .nth(4)
                    .is_some_and(|x| x.as_os_str() == "src")
                && !f.path.file_name().is_some_and(|n| {
                    let n = n.to_string_lossy();
                    n.starts_with("bindgen_")
                        || n.starts_with("size_check_")
                        || n.starts_with("offset_check_")
                })
                && !generated(&f.text)
        })
        .collect();
    f.sort_by(|a, b| a.path.cmp(&b.path));
    f
}
impl Rule for VerifyFfi {
    fn name(&self) -> &'static str {
        "verify_ffi"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let specf = c
            .sources("ffi-spec")
            .find(|f| f.path == Path::new("ntoskrnl/ntoskrnl.spec"));
        let bgf = c
            .sources("ffi-rust")
            .find(|f| f.path == Path::new("sdk/lib/rust/ntkrnl-sys/src/bindgen_i386.rs"));
        let sp = specf.map_or_else(BTreeMap::new, |f| spec(&f.text));
        let bg = bgf.map_or_else(BTreeMap::new, |f| bindgen(&f.text));
        let fs = files(c);
        let cargo_dirs: BTreeSet<_> = c
            .sources("ffi-rust")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .filter_map(|f| f.path.parent().map(Path::to_path_buf))
            .collect();
        let mut defs: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
        for f in c.sources("ffi-rust").filter(|f| {
            f.path.extension().is_some_and(|x| x == "rs")
                && f.path
                    .components()
                    .nth(4)
                    .is_some_and(|x| x.as_os_str() == "src")
        }) {
            let mut names = vec![];
            for m in LOCAL_DEF.captures_iter(&f.text) {
                names.push(m[1].into())
            }
            if !names.is_empty() {
                defs.entry(crate_root(&f.path, &cargo_dirs))
                    .or_default()
                    .extend(names);
            }
        }
        let mut out = vec![
            format!("Spec catalog: {} exports", sp.len()),
            format!("Bindgen catalog: {} functions", bg.len()),
        ];
        let mut counts = [
            ("ABI", 0usize),
            ("PARAM_COUNT", 0),
            ("C_MACRO", 0),
            ("FLEX_ARRAY", 0),
        ];
        for f in fs {
            let cr = crate_root(&f.path, &cargo_dirs);
            let local = defs.get(&cr).map(Vec::as_slice).unwrap_or(&[]);
            for issue in line_issues(f, &sp, &bg, local)
                .into_iter()
                .chain(flex_issues(f))
            {
                if let Some((tag, _)) = issue.strip_prefix('[').and_then(|x| x.split_once(']')) {
                    if let Some((_, n)) = counts.iter_mut().find(|(name, _)| *name == tag) {
                        *n += 1;
                    }
                }
                out.push(format!("  {issue}"));
            }
        }
        out.push(String::new());
        out.push("Summary:".into());
        for (tag, count) in &counts {
            if *count > 0 {
                out.push(format!("  {tag}: {count}"));
            }
        }
        let total: usize = counts.iter().map(|(_, n)| n).sum();
        out.push(format!(
            "  TOTAL: {total} issues across {} files",
            files(c).len()
        ));
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec_bindgen_and_nested_params() {
        let x = spec("@ stdcall Foo(long long)\n@ fastcall Bar()\n");
        assert_eq!(x["Foo"].params, 2);
        let b=bindgen("unsafe extern \"system\" {\n pub fn F(a: *mut (u8, u8), b: Option<unsafe extern fn(i32) -> i32>) -> i32;\n}\n");
        assert_eq!(b["F"].params, 2);
    }
    #[test]
    fn abi_and_arity_rules_and_local_suppression() {
        let mut o = vec![];
        let sp = spec("@ stdcall F(long)\n");
        check(
            &mut o,
            "x.rs",
            1,
            "F",
            "F",
            "C",
            2,
            &sp,
            &BTreeMap::new(),
            false,
            &[],
        );
        assert_eq!(o.len(), 2);
        let mut q = vec![];
        check(
            &mut q,
            "x.rs",
            1,
            "F",
            "F",
            "C",
            1,
            &sp,
            &BTreeMap::new(),
            false,
            &["F".into()],
        );
        assert!(q.is_empty());
    }
    #[test]
    fn original_local_shim_and_rejoined_block_controls() {
        let one = "#[no_mangle] pub unsafe extern \"C\" fn InterlockedIncrement(p: *mut i32) -> i32 { 1 }";
        let multi =
            "#[no_mangle]\npub unsafe extern \"system\" fn MultiLineForm(x: u32) -> u32 { x }";
        let names: Vec<_> = LOCAL_DEF
            .captures_iter(&format!("{one}\n{multi}"))
            .map(|m| m[1].to_owned())
            .collect();
        assert_eq!(names, ["InterlockedIncrement", "MultiLineForm"]);
        assert!(!LOCAL_DEF
            .is_match("unsafe extern \"C\" {\n fn InterlockedIncrement(p: *mut i32) -> i32;\n}"));
        let ok = SourceFile::new("fastfat-rust/src/verfysup.rs".into(), "extern \"C\" {\n fn helper(p: *mut u8);\n} extern \"system\" {\n fn KeSetEvent(e: *mut u8, i: i32, w: u8) -> i32;\n}\n".into());
        let sp = spec("@ stdcall KeSetEvent(a b c)\n");
        assert!(line_issues(&ok, &sp, &BTreeMap::new(), &[])
            .iter()
            .all(|x| !x.starts_with("[ABI]")));
        let wrong = SourceFile::new(
            "fastfat-rust/src/verfysup.rs".into(),
            ok.text.replace("} extern \"system\" {", "} extern \"C\" {"),
        );
        assert_eq!(
            line_issues(&wrong, &sp, &BTreeMap::new(), &[])
                .iter()
                .filter(|x| x.starts_with("[ABI]"))
                .count(),
            1
        );
    }
    #[test]
    fn macro_and_flexible_array_controls() {
        let f = SourceFile::new(
            "x.rs".into(),
            "extern \"system\" {\n fn IoSizeOfIrp(x: usize);\n}\n".into(),
        );
        let o = line_issues(&f, &BTreeMap::new(), &BTreeMap::new(), &[]);
        assert!(o[0].starts_with("[C_MACRO]"));
        let f=SourceFile::new("x.rs".into(),"p.Objects[i];\np.Objects[0];\nx.ViaUdmaTimings[k].Data[i];\nx.DevicePrivate.Data[i];\n// p.Objects[i]".into());
        let z = flex_issues(&f);
        assert_eq!(z.len(), 2);
        let f = SourceFile::new(
            "x.rs".into(),
            "DrainPartial = (*Partial).u.DevicePrivate.Data[1];\n".into(),
        );
        assert!(flex_issues(&f).is_empty());
        let f = SourceFile::new(
            "x.rs".into(),
            "x = (*p).u.DevicePrivate.Data[1] + (*info).Data[2];\n".into(),
        );
        assert_eq!(flex_issues(&f).len(), 2);
    }
}
