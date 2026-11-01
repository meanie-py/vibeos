//! Native UCRT tier-2 case generation. No output is written by this module.
mod cache_identity;
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
include!("constants.rs");
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub export: String,
    pub name: String,
    pub kinds: Vec<String>,
    pub has_rust: bool,
}
#[derive(Clone, Debug)]
pub struct Selection {
    pub rows: Vec<Row>,
    pub quarantined: Vec<String>,
}
pub struct Generated {
    pub selection: Selection,
    pub cases: String,
    pub refs: String,
}
fn read(p: &Path) -> Result<String> {
    fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))
}
fn command(cmd: &mut Command) -> Result<String> {
    let o = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !o.status.success() {
        return Err(format!("{cmd:?}: {}", String::from_utf8_lossy(&o.stderr)));
    }
    String::from_utf8(o.stdout).map_err(|e| e.to_string())
}
fn visit_files(p: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !p.exists() {
        return Ok(());
    }
    if p.is_file() {
        out.push(p.to_owned());
        return Ok(());
    }
    for e in fs::read_dir(p).map_err(|e| format!("{}: {e}", p.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        if !e.file_type().map_err(|e| e.to_string())?.is_symlink() {
            visit_files(&e.path(), out)?;
        }
    }
    Ok(())
}
fn compiler(root: &Path, arch: &str) -> Result<PathBuf> {
    let cc = match arch {
        "i386" => "i686-w64-mingw32-clang",
        "amd64" => "x86_64-w64-mingw32-clang",
        "arm64" => "aarch64-w64-mingw32-clang",
        _ => return Err(format!("unknown arch {arch}")),
    };
    let p = root.join(".build-tools/llvm-mingw/bin").join(cc);
    if !p.is_file() {
        return Err(format!("{} not found", p.display()));
    }
    Ok(p)
}
// The identity includes the compiler and every header in its installation. A legacy raw
// Python cache lacks this identity and is deliberately regenerated, never trusted.
fn identity(root: &Path, arch: &str, cc: &Path) -> Result<String> {
    let base = cache_identity::identity(root, arch, cc, HEADERS)?;
    // Preserve the compiler's default configuration, but refuse cache reuse when
    // a config outside the hashed installation changes its include search.
    let trace = Command::new(cc)
        .args(["-###", "-fsyntax-only", "-x", "c", "-"])
        .output()
        .map_err(|e| e.to_string())?;
    if !trace.status.success() {
        return Err(format!(
            "compiler config inspection failed: {}",
            String::from_utf8_lossy(&trace.stderr)
        ));
    }
    let install = root
        .join(".build-tools/llvm-mingw")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let mut digest = Sha256::new();
    digest.update(base);
    digest.update(&trace.stdout);
    digest.update(&trace.stderr);
    for line in String::from_utf8_lossy(&trace.stderr).lines() {
        if let Some(path) = line.trim().strip_prefix("Configuration file:") {
            let path = Path::new(path.trim())
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.starts_with(&install) {
                return Err(format!(
                    "external clang configuration {} cannot be cached safely; omit --cache",
                    path.display()
                ));
            }
            digest.update(fs::read(&path).map_err(|e| e.to_string())?);
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}
#[derive(serde::Deserialize)]
struct AstType {
    #[serde(rename = "qualType", default)]
    qual_type: Option<String>,
}
struct AstNode(Vec<(String, String)>);
impl<'de> serde::Deserialize<'de> for AstNode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct NodeVisitor;
        impl<'de> serde::de::Visitor<'de> for NodeVisitor {
            type Value = AstNode;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a clang AST node")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<AstNode, M::Error> {
                let (mut kind, mut name, mut ty) =
                    (None::<String>, None::<String>, None::<AstType>);
                let mut children = Vec::<AstNode>::new();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "kind" => kind = map.next_value()?,
                        "name" => name = map.next_value()?,
                        "type" => ty = map.next_value()?,
                        "inner" => {
                            children = map
                                .next_value::<Option<Vec<AstNode>>>()?
                                .unwrap_or_default()
                        }
                        _ => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                let mut found = Vec::new();
                if kind.as_deref() == Some("FunctionDecl") {
                    if let Some(name) = name {
                        found.push((name, ty.and_then(|t| t.qual_type).unwrap_or_default()));
                    }
                }
                for AstNode(child) in children {
                    found.extend(child);
                }
                Ok(AstNode(found))
            }
        }
        d.deserialize_map(NodeVisitor)
    }
}

pub fn ast_functions(
    root: &Path,
    arch: &str,
    cache: Option<&Path>,
) -> Result<HashMap<String, String>> {
    let cc = compiler(root, arch)?;
    let key = if cache.is_some() {
        Some(identity(root, arch, &cc)?)
    } else {
        None
    };
    if let (Some(p), Some(k)) = (cache, key.as_ref()) {
        if let Ok(s) = fs::read_to_string(p) {
            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                if v.get("identity").and_then(Value::as_str) == Some(k) {
                    if let Some(m) = v.get("functions").and_then(Value::as_object) {
                        return m
                            .iter()
                            .map(|(k, v)| {
                                v.as_str()
                                    .map(|v| (k.clone(), v.to_owned()))
                                    .ok_or_else(|| "invalid AST cache".to_owned())
                            })
                            .collect();
                    }
                }
            }
        }
    }
    let src = HEADERS
        .iter()
        .map(|h| format!("#include <{h}>\n"))
        .collect::<String>();
    let mut c = Command::new(cc);
    c.args(["-fsyntax-only", "-Xclang", "-ast-dump=json", "-x", "c", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = c.spawn().map_err(|e| e.to_string())?;
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(src.as_bytes())
        .map_err(|e| e.to_string())?;
    // Drain stderr concurrently so an erroring compiler cannot fill its pipe and
    // deadlock the JSON reader. Keep stdout streaming instead of buffering it.
    let stderr = child.stderr.take().ok_or("missing compiler stderr")?;
    let stderr_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        use std::io::Read;
        let result = std::io::BufReader::new(stderr).read_to_end(&mut bytes);
        result.map(|_| bytes)
    });
    let stdout = child.stdout.take().ok_or("missing compiler stdout")?;
    let parsed = serde_json::from_reader::<_, AstNode>(std::io::BufReader::new(stdout));
    let status = child.wait().map_err(|e| e.to_string())?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| "compiler stderr reader panicked")?
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!(
            "clang could not parse the UCRT headers: {}",
            String::from_utf8_lossy(&stderr)
        ));
    }
    let AstNode(nodes) = parsed.map_err(|e| format!("invalid clang AST: {e}"))?;
    let mut out = HashMap::new();
    for (name, ty) in nodes {
        out.entry(name).or_insert(ty);
    }
    if let (Some(p), Some(k)) = (cache, key) {
        let data = serde_json::to_vec(&json!({"identity":k,"functions":out}))
            .map_err(|e| e.to_string())?;
        if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(p, data).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    Ok(out)
}
fn renamed(root: &Path, arch: &str, lib: &str, prefixes: &[&str]) -> Result<HashSet<String>> {
    let path = root.join(format!("build-{arch}/{lib}"));
    if !path.is_file() {
        return Err(format!(
            "{} not found -- run tools/ucrt_tier2_refset.py --arch {arch} first",
            path.display()
        ));
    }
    let out = command(
        Command::new(root.join(".build-tools/llvm-mingw/bin/llvm-nm"))
            .arg("--defined-only")
            .arg(crate::probe::local(root, &path)?),
    )?;
    Ok(out
        .lines()
        .filter_map(|l| l.split_whitespace().nth(2))
        .filter_map(|s| prefixes.iter().find_map(|p| s.strip_prefix(p)))
        .map(str::to_owned)
        .collect())
}
fn nm(root: &Path, paths: &[PathBuf]) -> Result<HashSet<String>> {
    if paths.is_empty() {
        return Ok(HashSet::new());
    }
    let mut c = Command::new(root.join(".build-tools/llvm-mingw/bin/llvm-nm"));
    c.arg("--defined-only");
    for path in paths {
        c.arg(crate::probe::local(root, path)?);
    }
    let text = command(&mut c)?;
    Ok(text
        .lines()
        .filter_map(|l| {
            let v: Vec<_> = l.split_whitespace().collect();
            if v.len() == 3 && ["T", "D", "B", "R"].contains(&v[1]) {
                Some(v[2].to_owned())
            } else {
                None
            }
        })
        .collect())
}
fn undecorate(s: &str) -> String {
    s.trim_start_matches('_')
        .split('@')
        .next()
        .unwrap_or("")
        .to_owned()
}
fn metadata(root: &Path, arch: &str) -> Result<(HashSet<String>, HashMap<String, String>)> {
    let build = root.join(format!("build-{arch}"));
    crate::probe::local(root, &build)?;
    let ninja = read(&crate::probe::local(root, &build.join("build.ninja"))?)?;
    let re = Regex::new(r"sdk/lib/ucrt/CMakeFiles/ucrt\.dir/\S+?\.obj").unwrap();
    let objs: BTreeSet<_> = re
        .find_iter(&ninja)
        .map(|m| build.join(m.as_str()))
        .filter(|p| p.is_file())
        .collect();
    let mut ours = Vec::new();
    for dir in [
        "dll/win32/msvcrt/CMakeFiles/msvcrt_shared.dir",
        "sdk/lib/crt/CMakeFiles/crtmath.dir",
        "sdk/lib/vcruntime/CMakeFiles/vcruntime.dir",
        "sdk/lib/vcstartup/CMakeFiles/vcstartup.dir",
        "sdk/lib/crt/CMakeFiles/chkstk.dir",
        "sdk/lib/libwine/CMakeFiles/wine.dir",
        "sdk/lib/vcruntime/CMakeFiles/setjmp.dir",
        "sdk/lib/pseh/CMakeFiles/pseh.dir",
        "dll/win32/ucrtbase/CMakeFiles/ucrtbase.dir",
    ] {
        let mut v = Vec::new();
        visit_files(&build.join(dir), &mut v)?;
        ours.extend(
            v.into_iter()
                .filter(|p| p.extension().is_some_and(|e| e == "obj")),
        );
    }
    let a = build.join("dll/win32/ucrtbase/libcrt_shared_cargo.a");
    if a.is_file() {
        ours.push(a);
    }
    let ours: HashSet<_> = nm(root, &ours)?.iter().map(|x| undecorate(x)).collect();
    let mut ms = HashSet::new();
    let mut byfile = HashMap::new();
    for p in objs {
        let name = p
            .strip_prefix(build.join("sdk/lib/ucrt/CMakeFiles/ucrt.dir"))
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .trim_end_matches(".obj")
            .to_owned();
        for s in nm(root, &[p])? {
            let s = undecorate(&s);
            ms.insert(s.clone());
            byfile.insert(s, name.clone());
        }
    }
    let spec = read(&root.join("dll/win32/ucrtbase/ucrtbase.spec"))?;
    let re =
        Regex::new(r"^\s*(?:@|\d+)\s+(\S+)((?:\s+-\S+)*)\s+(\S+?)\s*(\(.*\))?(?:\s+(\S+))?\s*$")
            .unwrap();
    let want: &[&str] = match arch {
        "i386" => &["i386", "win32"],
        "amd64" => &["x86_64", "win64"],
        "arm64" => &["arm64", "win64"],
        _ => return Err(format!("unknown arch {arch}")),
    };
    let mut exports = HashSet::new();
    for l in spec.lines() {
        let l = l.split('#').next().unwrap_or("").trim_end();
        if let Some(c) = re.captures(l) {
            let flags = c.get(2).unwrap().as_str();
            let mut accept = true;
            for flag in flags
                .split_whitespace()
                .filter_map(|f| f.strip_prefix("-arch="))
            {
                let mut yes = false;
                let mut matched = false;
                for tag in flag.split(',') {
                    let neg = tag.starts_with('!');
                    let tag = tag.trim_start_matches('!');
                    if !["i386", "win32", "x86_64", "win64", "arm64", "arm"].contains(&tag) {
                        return Err(format!("unknown -arch tag {tag}"));
                    }
                    if neg && want.contains(&tag) {
                        accept = false
                    }
                    if !neg {
                        yes = true;
                        if want.contains(&tag) {
                            matched = true;
                        }
                    }
                }
                if yes && !matched {
                    accept = false;
                }
            }
            if accept {
                exports.insert(undecorate(&c[3]));
            }
        }
    }
    let owed = exports
        .intersection(&ms)
        .filter(|s| !ours.contains(*s))
        .cloned()
        .collect();
    Ok((owed, byfile))
}
fn int_type(s: &str) -> bool {
    matches!(
        s,
        "int"
            | "unsigned int"
            | "long"
            | "unsigned long"
            | "short"
            | "unsigned short"
            | "char"
            | "signed char"
            | "unsigned char"
            | "wint_t"
            | "wchar_t"
            | "size_t"
            | "__int64"
            | "long long"
            | "unsigned long long"
            | "unsigned __int64"
            | "errno_t"
            | "intptr_t"
            | "uintptr_t"
    )
}
pub fn shape(s: &str) -> Option<Vec<String>> {
    let re = Regex::new(r"^(.*?)\s*\((.*?)\)(?:\s*__attribute__.*)?$").unwrap();
    let c = re.captures(s)?;
    if !int_type(c[1].trim()) {
        return None;
    }
    let p = c[2].trim();
    if p.is_empty() || p == "void" {
        return Some(vec![]);
    }
    p.split(',')
        .map(|p| {
            let p = p.trim();
            Some(
                match p {
                    "_locale_t" => "L",
                    "const char *" => "SN",
                    "const wchar_t *" => "SW",
                    "const unsigned char *" => "SU",
                    "wctype_t" => "I",
                    _ if int_type(p) => "I",
                    _ => return None,
                }
                .to_owned(),
            )
        })
        .collect()
}
pub fn select(root: &Path, arch: &str, cache: Option<&Path>) -> Result<Selection> {
    let funcs = ast_functions(root, arch, cache)?;
    let (owed, byfile) = metadata(root, arch)?;
    let rust = renamed(root, arch, "libucrt_rust.a", &["_rust_", "rust_"])?;
    let names: BTreeSet<_> = renamed(root, arch, "libucrt_ref.a", &["_ref_", "ref_"])?
        .into_iter()
        .collect();
    let mut rows = Vec::new();
    let mut quarantined = Vec::new();
    for name in names {
        let Some(kinds) = funcs.get(&name).and_then(|s| shape(s)) else {
            continue;
        };
        let export = undecorate(&name);
        if !owed.contains(&export)
            || !byfile.get(&export).is_some_and(|s| {
                ["convert/", "string/", "mbstring/", "math/", "stdlib/"]
                    .iter()
                    .any(|p| s.starts_with(p))
            })
        {
            continue;
        }
        if [name.as_str(), export.as_str()]
            .iter()
            .any(|s| s.contains("coll") || QUARANTINE_EXACT.contains(s))
        {
            quarantined.push(export);
            continue;
        }
        rows.push(Row {
            export,
            name: name.clone(),
            kinds,
            has_rust: rust.contains(&name),
        });
    }
    quarantined.sort();
    Ok(Selection { rows, quarantined })
}
pub fn render_rows(rows: &[Row]) -> String {
    let mut o: Vec<String> = vec![
        "/* GENERATED by tools/gen_ucrt_tier2_cases.py -- do not edit.".into(),
        " *".into(),
        " * Declares each graded export's two renamed copies with __typeof__ of the REAL".into(),
        " * declaration, so no signature is ever re-spelled and none can drift from the".into(),
        " * header the bare side is declared by.".into(),
        " */".into(),
        "".into(),
        "/* The SAME header set the generator parsed. Emitting it here rather than trusting".into(),
        " * the harness to include the right ones is what keeps __typeof__ resolvable: a".into(),
        " * header the harness omitted took out every mbstring declaration first time. */".into(),
    ];
    o.extend(HEADERS.iter().map(|h| format!("#include <{h}>")));
    o.push(String::new());
    for r in rows {
        o.push(format!("extern __typeof__({0}) ref_{0};", r.name));
        if r.has_rust {
            o.push(format!("extern __typeof__({0}) rust_{0};", r.name));
        }
    }
    o.push(String::new());
    let shapes: BTreeSet<_> = rows.iter().map(|r| r.kinds.clone()).collect();
    for kinds in shapes {
        let macro_name = format!(
            "GRADE_{}",
            if kinds.is_empty() {
                "VOID".into()
            } else {
                kinds.join("_")
            }
        );
        let loops: Vec<_> = kinds
            .iter()
            .enumerate()
            .filter(|(_, k)| *k != "L")
            .map(|(i, _)| i)
            .collect();
        let big = loops.len() == 1 && kinds[loops[0]] == "I";
        let mut lines = Vec::new();
        let mut args = Vec::new();
        let mut depth = 0;
        for (i, k) in kinds.iter().enumerate() {
            if k == "L" {
                args.push("(_locale_t)0".to_owned());
                continue;
            }
            let (arr, cnt) = match (k.as_str(), big) {
                ("I", true) => ("g_int_full", "G_INT_FULL_N"),
                ("I", false) => ("g_int_small", "G_INT_SMALL_N"),
                ("SN", _) => ("g_narrow", "G_NARROW_N"),
                ("SW", _) => ("g_wide", "G_WIDE_N"),
                ("SU", _) => ("g_mbstr", "G_MBSTR_N"),
                _ => panic!("unsupported shape"),
            };
            lines.push(format!(
                "{}for (size_t i{i} = 0; i{i} < {cnt} && !g_fault_flag; i{i}++) \\",
                "    ".repeat(depth + 1)
            ));
            args.push(format!("{arr}[i{i}]"));
            depth += 1;
        }
        let label = loops
            .first()
            .map_or("\"()\"".to_owned(), |i| format!("label_idx(i{i})"));
        let argl = args.join(", ");
        let ref_c = format!("(long long)NORM(ref_##fn({argl}))); ")
            .trim_end()
            .to_owned();
        o.push(format!("#define {macro_name}(fn) \\"));
        o.extend(lines.clone());
        o.push(format!(
            "{}grade(#fn, {label}, (long long)NORM(fn({argl})), \\",
            "    ".repeat(depth + 1)
        ));
        o.push(format!(
            "{}(long long)NORM(rust_##fn({argl})), \\",
            "    ".repeat(depth + 2)
        ));
        o.push(format!("{}{ref_c}", "    ".repeat(depth + 2)));
        o.push(String::new());
        o.push(format!("#define {macro_name}_NORUST(fn) \\"));
        o.extend(lines);
        o.push(format!(
            "{}grade2(#fn, {label}, (long long)NORM(fn({argl})), \\",
            "    ".repeat(depth + 1)
        ));
        o.push(format!("{}{ref_c}", "    ".repeat(depth + 2)));
        o.push(String::new());
    }
    let pred = Regex::new(r"^_?(is[a-z]|isw|_?ismb|_?iswcsym)").unwrap();
    let cmp = Regex::new(r"(cmp|coll)(_l)?$").unwrap();
    for r in rows {
        let m = if r.kinds.is_empty() {
            "VOID".into()
        } else {
            r.kinds.join("_")
        };
        let norm = if pred.is_match(&r.name) {
            "(!!(x))"
        } else if cmp.is_match(&r.name) {
            "(sgn(x))"
        } else {
            "(x)"
        };
        o.extend([
            "#undef NORM".into(),
            format!("#define NORM(x) {norm}"),
            format!("static void ucrtdiff_run_{}(void)", r.name),
            "{".into(),
            format!(
                "    GRADE_{m}{}({});",
                if r.has_rust { "" } else { "_NORUST" },
                r.name
            ),
            "}".into(),
            String::new(),
        ]);
    }
    o.extend([
        "#undef NORM".into(),
        String::new(),
        "static void run_generated(void)".into(),
        "{".into(),
    ]);
    if !rows.is_empty() {
        o.extend(["    int outcome;".into(), String::new()]);
    }
    for r in rows {
        o.extend([
            format!("    if (export_begin(\"{}\")) {{", r.name),
            format!("        outcome = run_guarded(ucrtdiff_run_{});", r.name),
            format!("        export_end(outcome, {});", u8::from(r.has_rust)),
            "    }".into(),
            String::new(),
        ]);
    }
    o.extend(["}".into(), String::new()]);
    o.join("\n") + "\n"
}
pub fn render_refs(root: &Path, arch: &str) -> Result<String> {
    let have = renamed(root, arch, "libucrt_ref.a", &["_ref_", "ref_"])?;
    let plain = renamed(
        root,
        arch,
        "libucrt_ref.a",
        if arch == "i386" { &["_"] } else { &[""] },
    )?;
    let mut lines = vec![
        "/* GENERATED by tools/gen_ucrt_tier2_cases.py -- do not edit. 1 when the reference"
            .to_owned(),
        " * set defines ref_<name> for a hand-graded export, 0 once a takeover removed it. */"
            .to_owned(),
    ];
    for n in HAND_GRADED {
        lines.push(format!(
            "#define UCRTDIFF_REF_HAS_{n} {}",
            u8::from(have.contains(*n))
        ));
    }
    for n in REF_INTERNALS {
        lines.push(format!(
            "#define UCRTDIFF_REF_HAS_{n} {}",
            u8::from(plain.contains(*n))
        ));
    }
    let alias = root.join(format!("build-{arch}/libucrt_ours.refalias"));
    let aliases: HashSet<String> = if alias.is_file() {
        read(&crate::probe::local(root, &alias)?)?
            .lines()
            .map(|l| {
                let x = l.split('=').next().unwrap_or("");
                x.split_once("ref_").map_or(x, |(_, n)| n).to_owned()
            })
            .collect()
    } else {
        HashSet::new()
    };
    for n in REF_DRIVERS {
        lines.push(format!(
            "#define UCRTDIFF_REF_CALLABLE_{n} {}",
            u8::from(have.contains(*n) || aliases.contains(*n))
        ));
    }
    Ok(lines.join("\n") + "\n")
}
pub fn generate(root: &Path, arch: &str, cache: Option<&Path>) -> Result<Generated> {
    let selection = select(root, arch, cache)?;
    let cases = render_rows(&selection.rows);
    let refs = render_refs(root, arch)?;
    Ok(Generated {
        selection,
        cases,
        refs,
    })
}
pub fn stats(arch: &str, rows: &[Row]) -> String {
    let mut hist = BTreeMap::<String, (usize, usize)>::new();
    for (i, r) in rows.iter().enumerate() {
        let key = if r.kinds.is_empty() {
            "VOID".into()
        } else {
            r.kinds.join("_")
        };
        let e = hist.entry(key).or_insert((0, i));
        e.0 += 1;
    }
    let mut counts: Vec<_> = hist.into_iter().collect();
    counts.sort_by_key(|(_, (n, i))| (std::cmp::Reverse(*n), *i));
    let n = rows.iter().filter(|r| r.has_rust).count();
    let mut s = format!(
        "{arch}: {} gradable exports ({n} with a Rust body to grade, {} DLL-vs-ref only)\n",
        rows.len(),
        rows.len() - n
    );
    for (sh, (n, _)) in counts {
        s.push_str(&format!("  {n:4}  ({sh})\n"));
    }
    s
}
pub fn quarantine_report(s: &Selection) -> Option<String> {
    if s.quarantined.is_empty() {
        None
    } else {
        Some(format!(
            "gen-ucrt-tier2-cases: QUARANTINED {} export(s) -- {QUARANTINE_REASON}\n  {}",
            s.quarantined.len(),
            s.quarantined.join(" ")
        ))
    }
}

#[cfg(test)]
mod ast_tests {
    use super::AstNode;
    #[test]
    fn streamed_nodes_keep_preorder_and_ignore_unknown_fields() {
        let text = r#"{"inner":[{"inner":[{"kind":"FunctionDecl","name":"f","type":{"qualType":"long (void)"}}],"unknown":{"huge":[1,2,3]},"kind":"FunctionDecl","name":"f","type":{"qualType":"int (int)"}}]}"#;
        let AstNode(nodes) = serde_json::from_str(text).unwrap();
        assert_eq!(
            nodes,
            [
                ("f".into(), "int (int)".into()),
                ("f".into(), "long (void)".into())
            ]
        );
    }
    #[test]
    fn shapes_match_python_for_real_header_signatures() {
        let cases: Vec<(String, Option<Vec<String>>)> =
            serde_json::from_str(include_str!("../../tests/compiler/ucrt-shapes.json")).unwrap();
        for (signature, expected) in cases {
            assert_eq!(super::shape(&signature), expected, "{signature}");
        }
    }
    #[test]
    fn malformed_ast_is_an_error() {
        assert!(serde_json::from_str::<AstNode>(r#"{"kind":"FunctionDecl", "inner":[}"#).is_err());
    }
}
