//! Rust source-rule port of `verify_ucrtbase_cohab.py`.
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub struct VerifyUcrtbaseCohab;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree(
        "ucrt-rust",
        &["sdk/lib/rust"],
        &["rs"],
        &["Cargo.toml", "build.rs"],
    )
    .with_inputs(),
    SourceSet::tree("ucrt-cpp", &["sdk/lib/ucrt"], &["cpp", "h", "c"], &[]).with_inputs(),
    SourceSet::tree("ucrt-inc", &["sdk/lib/ucrt/inc"], &["h"], &[]).with_inputs(),
    SourceSet::tree(
        "ucrt-cmake",
        &["dll/win32/ucrtbase", "dll/win32/msvcrt"],
        &[],
        &["CMakeLists.txt"],
    )
    .with_inputs(),
    SourceSet::tree("ucrt-spec", &["dll/win32/ucrtbase"], &["spec"], &[]).with_inputs(),
    SourceSet::tracked("ucrt-control", &["yaml"], &["ucrt-takeover.yaml"]),
];
const ROOT_CRATE: &str = "crt-shared-cargo";
const FORBIDDEN: &str = "msvcrt-crt-rust";
const CRATES: &[&str] = &["crt-shared-cargo", "msvcrt-crt-cargo"];
const ARCHES: &[(&str, &str)] = &[
    ("i386", "i686-pc-windows-gnu"),
    ("amd64", "x86_64-pc-windows-gnu"),
    ("arm64", "aarch64-pc-windows-gnullvm"),
];
const OURS: &[(&str, &str)] = &[
    (
        "msvcrt_shared",
        "dll/win32/msvcrt/CMakeFiles/msvcrt_shared.dir",
    ),
    ("crtmath", "sdk/lib/crt/CMakeFiles/crtmath.dir"),
    ("vcruntime", "sdk/lib/vcruntime/CMakeFiles/vcruntime.dir"),
    ("vcstartup", "sdk/lib/vcstartup/CMakeFiles/vcstartup.dir"),
    ("chkstk", "sdk/lib/crt/CMakeFiles/chkstk.dir"),
    ("wine", "sdk/lib/libwine/CMakeFiles/wine.dir"),
    ("setjmp", "sdk/lib/vcruntime/CMakeFiles/setjmp.dir"),
    ("pseh", "sdk/lib/pseh/CMakeFiles/pseh.dir"),
    ("ucrtbase-own", "dll/win32/ucrtbase/ucrtbase_stubs.rs.obj"),
];
const REBUILD: &[(&str, &str, &str)] = &[
    (
        "dll/win32/ucrtbase/CMakeLists.txt",
        "UCRT_SHARED_RUST_SOURCES",
        "crt-shared-cargo",
    ),
    (
        "dll/win32/msvcrt/CMakeLists.txt",
        "MSVCRT_CRT_RUST_SOURCES",
        "msvcrt-crt-cargo",
    ),
];
const ARCH_TAGS: &[(&str, &[&str])] = &[
    ("i386", &["i386", "win32"]),
    ("amd64", &["x86_64", "win64"]),
    ("arm64", &["arm64", "win64"]),
];
const UCRT_OBJ_DIR: &str = "sdk/lib/ucrt/CMakeFiles/ucrt.dir";

fn re(p: &'static str) -> &'static Regex {
    static R: OnceLock<BTreeMap<&'static str, Regex>> = OnceLock::new();
    R.get_or_init(|| {
        [
            r"(?m)^\s*([A-Za-z0-9_-]+)\s*=(.*)$",
            r"^\s*([A-Za-z0-9_-]+)\s*=(.*)$",
            r#"\bpath\s*=\s*"([^"]+)""#,
            r"\bpath\s*=",
            r"(?m)^\s*extern\s+crate\s+([A-Za-z0-9_]+)\s*;",
            r#"(?s)extern\s+"C"\s*\{(.*?)\n\s*\}"#,
            r"\bfn\s+([A-Z][A-Za-z0-9]*)\s*\(",
            r"\bfn\s+(\w+)\s*\(",
            r"\b(?:WINAPI|APIENTRY|NTAPI|__stdcall)\s+(\w+)\s*\(",
            r#"_CRTALLOC\("\.CRT\$X[^\"]*"\)\s*\w+\s+const\s+(\w+)\s*="#,
            r"(_CRT_LINKER_FORCE_INCLUDE|force_include!)\(\s*([\w]+)\s*\)",
            r"rust_crate_sources\(\s*([A-Za-z0-9_]+)\s+\S+\s+(\S+)\s*\)",
            r"set\(\s*\w+_RUST_CRATES\b",
            r"(?m)^\s*(?:@|\d+)\s+(\S+)((?:\s+-\S+)*)\s+(\S+?)\s*(\(.*\))?(?:\s+(\S+))?\s*$",
            r"-arch=(\S+)",
            r"^[A-Za-z_?@$][\w?@$]*$",
            r"\s*DLL Name: (\S+)",
            r"\s+\d+\s+(\S+)$",
            r"\s+\d+\s+0x[0-9a-fA-F]+\s+(\S+)$",
            r"(?m)^\s*(?:#|//).*?$",
        ]
        .into_iter()
        .map(|p| (p, crate::regexes::cached(p).unwrap()))
        .collect()
    });
    R.get().unwrap().get(p).unwrap()
}

fn source_map<'a>(c: &'a Corpus) -> BTreeMap<PathBuf, &'a SourceFile> {
    INPUTS
        .iter()
        .flat_map(|s| c.sources(s.name))
        .map(|f| (f.path.clone(), f))
        .collect()
}
fn content<'a>(files: &'a BTreeMap<PathBuf, &'a SourceFile>, path: &str) -> Option<&'a str> {
    files.get(Path::new(path)).map(|f| f.text.as_str())
}
fn crate_path(ws: &Path, name: &str) -> PathBuf {
    ws.join(name)
}

fn dependencies(manifest: &str) -> BTreeSet<String> {
    let mut section = false;
    let mut out = BTreeSet::new();
    for line in manifest.lines() {
        let s = line.trim();
        if s.starts_with('[') {
            section = s.starts_with("[dependencies")
                || (s.starts_with("[target.") && s.contains("dependencies"));
            continue;
        }
        if !section || s.is_empty() || s.starts_with('#') {
            continue;
        }
        if let Some(m) = re(r"^\s*([A-Za-z0-9_-]+)\s*=(.*)$").captures(s) {
            if re(r"\bpath\s*=").is_match(&m[2]) {
                out.insert(m[1].to_owned());
            }
        }
    }
    out
}
fn dep_closure(start: &str, manifests: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let mut seen = vec![start.to_owned()];
    let mut i = 0;
    while i < seen.len() {
        let path = PathBuf::from("sdk/lib/rust")
            .join(&seen[i])
            .join("Cargo.toml");
        if let Some(m) = manifests.get(&path) {
            for d in dependencies(&m.text) {
                if !seen.contains(&d) {
                    seen.push(d);
                }
            }
        }
        i += 1;
    }
    seen
}
fn check_structure(files: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let mut out = Vec::new();
    let ws = Path::new("sdk/lib/rust");
    let start = ws.join(ROOT_CRATE);
    if !files.contains_key(&start.join("Cargo.toml")) {
        return vec![format!("sdk/lib/rust/{ROOT_CRATE}: crate is MISSING -- ucrtbase's Rust archive has no manifest. fix: restore the crate, or drop the cohab from dll/win32/ucrtbase/CMakeLists.txt")];
    }
    let manifests = files
        .iter()
        .filter(|(p, _)| p.file_name().is_some_and(|n| n == "Cargo.toml"))
        .map(|(p, f)| (p.clone(), *f))
        .collect::<BTreeMap<_, _>>();
    let mut parent = BTreeMap::<String, Option<String>>::from([(ROOT_CRATE.into(), None)]);
    let mut queue = VecDeque::from([ROOT_CRATE.to_owned()]);
    while let Some(cur) = queue.pop_front() {
        let path = Path::new("sdk/lib/rust").join(&cur).join("Cargo.toml");
        for dep in manifests
            .get(&path)
            .map(|m| dependencies(&m.text))
            .unwrap_or_default()
        {
            if !parent.contains_key(&dep) {
                parent.insert(dep.clone(), Some(cur.clone()));
                queue.push_back(dep);
            }
        }
    }
    if parent.contains_key(FORBIDDEN) {
        let mut chain = Vec::new();
        let mut node = FORBIDDEN.to_owned();
        loop {
            chain.push(node.clone());
            let Some(Some(prev)) = parent.get(&node) else {
                break;
            };
            node = prev.clone();
        }
        chain.reverse();
        out.push(format!("sdk/lib/rust/{ROOT_CRATE}/Cargo.toml: ucrtbase's Rust archive can reach {FORBIDDEN} via {} -- that pulls msvcrt's whole CRT floor into ucrtbase.dll, silently replacing genuine UCRT bodies under --allow-multiple-definition. fix: keep the shared surface in crt-shared-rust; msvcrt-crt-rust must depend on IT, never the reverse", chain.join(" -> ")));
    }
    out
}
fn check_extern(files: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let ws = Path::new("sdk/lib/rust");
    let manifest_path = ws.join(ROOT_CRATE).join("Cargo.toml");
    let lib_path = ws.join(ROOT_CRATE).join("src/lib.rs");
    let Some(lib) = content(files, lib_path.to_str().unwrap()) else {
        return vec![format!("sdk/lib/rust/{ROOT_CRATE}/src/lib.rs: MISSING")];
    };
    let Some(manifest) = content(files, manifest_path.to_str().unwrap()) else {
        return Vec::new();
    };
    let declared = re(r"(?m)^\s*extern\s+crate\s+([A-Za-z0-9_]+)\s*;")
        .captures_iter(lib)
        .map(|m| m[1].to_owned())
        .collect::<BTreeSet<_>>();
    dependencies(manifest).into_iter().filter(|d| d != "panic-common" && !declared.contains(&d.replace('-',"_"))).map(|dep|format!("sdk/lib/rust/{ROOT_CRATE}/src/lib.rs: depends on `{dep}` but never names it, so NONE of its objects reach libcrt_shared_cargo.a and whatever cohab that dependency was added for is silently not happening. fix: add `extern crate {};`",dep.replace('-',"_"))).collect()
}
fn check_rebuild(files: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let mut out = Vec::new();
    for (cmake, var, root) in REBUILD {
        let Some(text) = content(files, cmake) else {
            continue;
        };
        if let Some(m) = re(r"rust_crate_sources\(\s*([A-Za-z0-9_]+)\s+\S+\s+(\S+)\s*\)")
            .captures_iter(text)
            .find(|m| &m[1] == *var)
        {
            if &m[2] != *root {
                out.push(format!("{cmake}: {var} walks `{}`, but cargo builds `{root}`. fix: rust_crate_sources({var} <ws> {root})",&m[2]));
            }
        } else {
            out.push(format!("{cmake}: {var} is not derived by `rust_crate_sources({var} <ws> {root})` -- a hand-kept list drifts and ships a STALE archive. fix: derive it"));
        }
        if re(r"set\(\s*\w+_RUST_CRATES\b").is_match(text) {
            out.push(format!(
                "{cmake}: a hand-kept `*_RUST_CRATES` list is back. fix: remove it"
            ));
        }
    }
    out
}

impl Rule for VerifyUcrtbaseCohab {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "verify_ucrtbase_cohab"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files = source_map(c);
        let mut out = check_structure(&files);
        out.extend(check_extern(&files));
        out.extend(check_rebuild(&files));
        // Build-dependent checks retain the original silent stdout skip when their
        // build tree or artifacts are absent; they never fabricate an archive verdict.
        out.extend(check_archive(c, &files)?);
        out.extend(check_force_includes(&files));
        out.extend(check_win32_abi(&files));
        out.extend(check_burndown(c, &files)?);
        out.extend(check_export_imports(c, &files)?);
        Ok(out)
    }
}

fn check_force_includes(files: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let init = Path::new("sdk/lib/ucrt/initializers");
    let hooks = files
        .iter()
        .filter(|(p, _)| p.starts_with(init) && p.extension().is_some_and(|e| e == "cpp"))
        .flat_map(|(_, f)| {
            re(r#"_CRTALLOC\("\.CRT\$X[^\"]*"\)\s*\w+\s+const\s+(\w+)\s*="#)
                .captures_iter(&f.text)
                .map(|m| m[1].to_owned())
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    if hooks.is_empty() {
        return Vec::new();
    }
    let blob = files
        .iter()
        .filter(|(p, _)| {
            (p.starts_with("sdk/lib/ucrt") || p.starts_with("sdk/lib/rust/ucrt-rust/src"))
                && !p.starts_with(init)
        })
        .map(|(_, f)| f.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    hooks.into_iter().filter(|sym|sym!="__acrt_tran_fma3_initializer"&&!re(r"(_CRT_LINKER_FORCE_INCLUDE|force_include!)\(\s*([\w]+)\s*\)").captures_iter(&blob).any(|m|m[2]==*sym))
        .map(|sym|format!("ucrt initializer `{sym}` is force-included by nothing: its archive member will not be linked and the hook will never run. A takeover deleted the C file that carried `_CRT_LINKER_FORCE_INCLUDE({sym})`. fix: add `crate::force_include!({sym});` to the Rust module that replaced it")).collect()
}
fn check_win32_abi(files: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let mut winapi = BTreeSet::new();
    for (p, f) in files.iter().filter(|(p, _)| {
        p.starts_with("sdk/lib/ucrt/inc") && p.extension().is_some_and(|e| e == "h")
    }) {
        let _ = p;
        winapi.extend(
            re(r"\b(?:WINAPI|APIENTRY|NTAPI|__stdcall)\s+(\w+)\s*\(")
                .captures_iter(&f.text)
                .map(|m| m[1].to_owned()),
        );
    }
    let mut out = Vec::new();
    for (p, f) in files.iter().filter(|(p, _)| {
        p.starts_with("sdk/lib/rust/ucrt-rust/src") && p.extension().is_some_and(|e| e == "rs")
    }) {
        for block in re(r#"(?s)extern\s+"C"\s*\{(.*?)\n\s*\}"#).captures_iter(&f.text) {
            let mut names = re(r"\bfn\s+([A-Z][A-Za-z0-9]*)\s*\(")
                .captures_iter(&block[1])
                .map(|m| m[1].to_owned())
                .collect::<BTreeSet<_>>();
            names.extend(
                re(r"\bfn\s+(\w+)\s*\(")
                    .captures_iter(&block[1])
                    .map(|m| m[1].to_owned())
                    .filter(|n| winapi.contains(n)),
            );
            for name in names {
                out.push(format!("{}: `{name}` is declared in an `extern \"C\"` block; a Win32 API is stdcall on i386. fix: move it to `extern \"system\"`",p.display()));
            }
        }
    }
    out
}

fn archive_path(build: &Path, triple: &str) -> Option<PathBuf> {
    [
        build
            .join("sdk/lib/rust/target")
            .join(triple)
            .join("release/libcrt_shared_cargo.a"),
        build.join("dll/win32/ucrtbase/libcrt_shared_cargo.a"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}
fn archive_crates(files: &BTreeMap<PathBuf, &SourceFile>) -> Vec<String> {
    let manifests = files
        .iter()
        .filter(|(p, _)| {
            p.starts_with("sdk/lib/rust") && p.file_name().is_some_and(|n| n == "Cargo.toml")
        })
        .map(|(p, f)| (p.clone(), *f))
        .collect();
    dep_closure(ROOT_CRATE, &manifests)
}
fn source_changed_at(root: &Path, files: &BTreeMap<PathBuf, &SourceFile>) -> Option<(f64, String)> {
    let crates = archive_crates(files);
    let mut args = vec!["log", "-1", "--format=%ct", "--"];
    let owned = crates
        .iter()
        .map(|c| format!("sdk/lib/rust/{c}"))
        .collect::<Vec<_>>();
    args.extend(owned.iter().map(String::as_str));
    let committed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    let mut status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .args(&owned)
        .output()
        .ok()?;
    if !committed.status.success() || !status.status.success() {
        return None;
    }
    let mut best = String::from_utf8_lossy(&committed.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0);
    let mut what = if best > 0.0 {
        "the last commit touching them".to_owned()
    } else {
        String::new()
    };
    for line in String::from_utf8_lossy(&status.stdout).lines() {
        let rel = line.get(3..).unwrap_or("").trim().trim_matches('"');
        if let Ok(meta) = std::fs::metadata(root.join(rel)) {
            if let Ok(modified) = meta.modified() {
                if let Ok(t) = modified.duration_since(std::time::UNIX_EPOCH) {
                    let t = t.as_secs_f64();
                    if t > best {
                        best = t;
                        what = rel.to_owned();
                    }
                }
            }
        }
    }
    (best > 0.0).then_some((best, what))
}
fn stale_against(
    root: &Path,
    files: &BTreeMap<PathBuf, &SourceFile>,
    archive: &Path,
) -> Option<String> {
    let archive_mtime = std::fs::metadata(archive)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs_f64();
    if let Some((when, what)) = source_changed_at(root, files) {
        return (when > archive_mtime).then_some(what);
    }
    for member in archive_crates(files) {
        let prefix = PathBuf::from("sdk/lib/rust").join(&member);
        let manifest = prefix.join("Cargo.toml");
        let build_script = prefix.join("build.rs");
        for (p, _) in files.iter().filter(|(p, _)| {
            p.starts_with(&prefix)
                && (p.starts_with(prefix.join("src"))
                    || p.as_path() == manifest.as_path()
                    || p.as_path() == build_script.as_path())
        }) {
            if let Ok(meta) = std::fs::metadata(root.join(p)) {
                if let Ok(t) = meta.modified().and_then(|x| {
                    x.duration_since(std::time::UNIX_EPOCH)
                        .map_err(std::io::Error::other)
                }) {
                    if t.as_secs_f64() > archive_mtime {
                        return Some(p.display().to_string());
                    }
                }
            }
        }
    }
    None
}
fn nm_defined(root: &Path, paths: &[PathBuf]) -> Option<BTreeSet<String>> {
    if paths.is_empty() {
        return Some(BTreeSet::new());
    }
    let mut commands = vec![
        root.join(".build-tools/llvm-mingw/bin/llvm-nm"),
        PathBuf::from("llvm-nm"),
        PathBuf::from("nm"),
    ];
    while let Some(nm) = commands.first().cloned() {
        commands.remove(0);
        let Ok(out) = Command::new(&nm).arg("--defined-only").args(paths).output() else {
            continue;
        };
        if !out.status.success() {
            continue;
        }
        let mut syms = BTreeSet::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let p = line.split_whitespace().collect::<Vec<_>>();
            if p.len() == 3 && p[1].len() == 1 && "TDBR".contains(p[1]) {
                syms.insert(p[2].to_owned());
            }
        }
        return Some(syms);
    }
    None
}
fn graph_objects(build: &Path) -> Option<BTreeSet<PathBuf>> {
    let text = std::fs::read_to_string(build.join("build.ninja")).ok()?;
    let mut out = BTreeSet::new();
    for line in text.lines() {
        if let Some(s) = line.strip_prefix("build ") {
            let target = s.split(':').next()?.split(" | ").next()?.trim();
            if target.ends_with(".obj") || target.ends_with(".o") {
                out.insert(build.join(target.replace("$ ", " ")));
            }
        }
    }
    Some(out)
}
fn objects_under(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_owned()];
    }
    let mut out = Vec::new();
    let Ok(iter) = std::fs::read_dir(path) else {
        return out;
    };
    let mut entries = iter
        .filter_map(Result::ok)
        .map(|x| x.path())
        .collect::<Vec<_>>();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            out.extend(objects_under(&p));
        } else if p.extension().is_some_and(|e| e == "obj" || e == "o") {
            out.push(p);
        }
    }
    out
}
fn parse_spec_exports(text: &str, arch: &str) -> Result<BTreeSet<String>, String> {
    let tag_re = crate::regexes::cached(r"-arch=(\S+)").unwrap();
    let spec_re = crate::regexes::cached(
        r"^\s*(?:@|\d+)\s+(\S+)((?:\s+-\S+)*)\s+(\S+?)\s*(\(.*\))?(?:\s+(\S+))?\s*$",
    )
    .unwrap();
    let wanted = ARCH_TAGS
        .iter()
        .find(|(a, _)| *a == arch)
        .map(|(_, x)| *x)
        .ok_or_else(|| format!("unknown arch {arch}"))?;
    let known = ["i386", "win32", "x86_64", "win64", "arm64", "arm"];
    let mut out = BTreeSet::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim_end();
        let Some(m) = spec_re.captures(line) else {
            continue;
        };
        let flags = &m[2];
        if let Some(a) = tag_re.captures(flags) {
            let tags = a[1].split(',').collect::<Vec<_>>();
            let yes = tags
                .iter()
                .copied()
                .filter(|t| !t.starts_with('!'))
                .collect::<BTreeSet<_>>();
            let no = tags
                .iter()
                .filter_map(|t| t.strip_prefix('!'))
                .collect::<BTreeSet<_>>();
            for t in yes.union(&no) {
                if !known.contains(t) {
                    return Err(format!(
                        "export_census: ucrtbase.spec: unknown -arch= tag(s) {t}"
                    ));
                }
            }
            if no.iter().any(|t| wanted.contains(t))
                || (!yes.is_empty() && !yes.iter().any(|t| wanted.contains(t)))
            {
                continue;
            }
        }
        out.insert(m[3].to_owned());
    }
    Ok(out)
}
fn takeovers(
    root: &Path,
    files: &BTreeMap<PathBuf, &SourceFile>,
    arch: Option<&str>,
) -> Result<BTreeMap<String, String>, String> {
    let Some(text) = content(files, "tools/ucrt-takeover.yaml") else {
        return Ok(BTreeMap::new());
    };
    let doc = crate::yaml::parse(text)?;
    let mut out = BTreeMap::new();
    if let Some(fams) = doc.get("families").and_then(Value::as_object) {
        for (fam, body) in fams {
            if let Some(a) = arch {
                if let Some(arches) = body.get("arches").and_then(Value::as_array) {
                    if !arches.iter().any(|x| x.as_str() == Some(a)) {
                        continue;
                    }
                }
            }
            if let Some(exports) = body.get("exports").and_then(Value::as_array) {
                for sym in exports.iter().filter_map(Value::as_str) {
                    out.insert(sym.to_owned(), fam.clone());
                }
            }
        }
    }
    let _ = root;
    Ok(out)
}
fn check_archive(
    c: &Corpus,
    files: &BTreeMap<PathBuf, &SourceFile>,
) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    let root = &c.root;
    for (arch, triple) in ARCHES {
        let build = root.join(format!("build-{arch}"));
        if !build.is_dir() {
            continue;
        }
        let Some(archive) = archive_path(&build, triple) else {
            continue;
        };
        if stale_against(root, files, &archive).is_some() {
            continue;
        }
        let live = graph_objects(&build);
        let mut objects = Vec::new();
        let mut missing = Vec::new();
        for (target, rel) in OURS {
            let d = build.join(rel);
            let found = objects_under(&d)
                .into_iter()
                .filter(|p| live.as_ref().is_none_or(|set| set.contains(p)))
                .collect::<Vec<_>>();
            if found.is_empty() {
                missing.push(format!("{target} ({})", d.display()));
            } else {
                objects.extend(found);
            }
        }
        if !missing.is_empty() {
            findings.push(format!("build-{arch}: libcrt_shared_cargo.a exists but the objects of {} do not -- cannot prove the Rust surface is a subset of the C's. fix: ./vibeos-cli build --arch {arch}",missing.join(", ")));
            continue;
        }
        let Some(rust) = nm_defined(root, &[archive.clone()]) else {
            findings.push(format!("build-{arch}: no usable nm for {} -- the subset assertion could not run. fix: install llvm-nm (this check must not silently pass)",archive.file_name().unwrap().to_string_lossy()));
            continue;
        };
        let Some(csyms) = nm_defined(root, &objects) else {
            findings.push(format!(
                "build-{arch}: nm could not read the complete C object surface"
            ));
            continue;
        };
        let spec_path = PathBuf::from("dll/win32/ucrtbase/ucrtbase.spec");
        let Some(spec) = content(files, spec_path.to_str().unwrap()) else {
            findings.push(format!("dll/win32/ucrtbase/ucrtbase.spec: MISSING -- cannot tell which symbols ucrtbase exports, so the shadowing assertion for build-{arch} could not run"));
            continue;
        };
        let exported = parse_spec_exports(spec, arch)?;
        let strip = |s: &str| {
            if *arch == "i386" {
                s.strip_prefix('_').unwrap_or(s).to_owned()
            } else {
                s.to_owned()
            }
        };
        let c_norm = csyms
            .iter()
            .map(|s| strip(s).to_owned())
            .collect::<BTreeSet<_>>();
        let declared = takeovers(root, files, Some(arch))?;
        let declared_norm = declared
            .keys()
            .map(|x| x.trim_start_matches('_').to_owned())
            .collect::<BTreeSet<_>>();
        let rust_norm = rust
            .iter()
            .map(|s| strip(s).trim_start_matches('_').to_owned())
            .collect::<BTreeSet<_>>();
        let gone = declared_norm
            .difference(&rust_norm)
            .cloned()
            .collect::<Vec<_>>();
        if !gone.is_empty() {
            findings.push(format!("build-{arch}: tools/ucrt-takeover.yaml declares {} export(s) that libcrt_shared_cargo.a does not define: {}. Either the body was removed and the declaration is stale, or the family's cargo feature is not enabled in crt-shared-cargo -- in which case the takeover silently did not happen and the Microsoft body is still live",gone.len(),gone.iter().take(12).cloned().collect::<Vec<_>>().join(", ")));
        }
        let extra = rust
            .iter()
            .filter(|s| {
                let norm = strip(s);
                let internal = s.starts_with(".refptr.")
                    || s.starts_with("_ZN")
                    || s.starts_with("__rust")
                    || s.starts_with("anon.")
                    || s.starts_with(".L")
                    || s.starts_with(".weak.")
                    || s.contains("_ZN")
                    || !norm
                        .chars()
                        .next()
                        .is_some_and(|x| x.is_ascii_alphabetic() || "_?@$".contains(x))
                    || !norm
                        .chars()
                        .all(|x| x.is_ascii_alphanumeric() || "_?@$".contains(x));
                !c_norm.contains(&norm)
                    && exported.contains(&norm)
                    && !internal
                    && !declared_norm.contains(norm.trim_start_matches('_'))
            })
            .cloned()
            .collect::<Vec<_>>();
        if !extra.is_empty() {
            let shown = format!(
                "{}{}",
                extra
                    .iter()
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                if extra.len() > 12 {
                    format!(", +{} more", extra.len() - 12)
                } else {
                    String::new()
                }
            );
            findings.push(format!("build-{arch}: libcrt_shared_cargo.a defines {} symbol(s) that ucrtbase EXPORTS but none of our own static libs on its link line ({}) defines, so each one silently replaces the genuine Microsoft UCRT body callers expect (--allow-multiple-definition raises nothing): {shown}. fix: move the body back to msvcrt-crt-rust; or, if this is a DELIBERATE takeover of the Microsoft body, implement it in ucrt-rust and declare it in tools/ucrt-takeover.yaml with the oracle that grades it; or -- if ucrtbase really does owe it to us -- add the C to SHARED_SOURCE so both sides agree on the surface",extra.len(),OURS.iter().take(8).map(|x|x.0).collect::<Vec<_>>().join(", ")));
        }
    }
    Ok(findings)
}
fn pe_exports_imports(
    objdump: &Path,
    dll: &Path,
) -> Option<(BTreeSet<String>, BTreeMap<String, String>)> {
    let output = Command::new(objdump).arg("-p").arg(dll).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let mut ex = BTreeSet::new();
    let mut imports = BTreeMap::new();
    let mut section = "";
    let mut dll_name = String::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(m) = re(r"\s*DLL Name: (\S+)").captures(line) {
            section = "imp";
            dll_name = m[1].to_owned();
            continue;
        }
        if line.starts_with("Export Table") {
            section = "exp";
            continue;
        }
        if section == "imp" && !dll_name.is_empty() {
            if let Some(m) = re(r"\s+\d+\s+(\S+)$").captures(line) {
                imports
                    .entry(m[1].to_owned())
                    .or_insert_with(|| dll_name.clone());
            }
        } else if section == "exp" {
            if let Some(m) = re(r"\s+\d+\s+0x[0-9a-fA-F]+\s+(\S+)$").captures(line) {
                ex.insert(m[1].to_owned());
            }
        }
    }
    Some((ex, imports))
}
fn check_export_imports(
    c: &Corpus,
    files: &BTreeMap<PathBuf, &SourceFile>,
) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let bundled = c.root.join(".build-tools/llvm-mingw/bin/llvm-objdump");
    for arch in ["i386", "amd64", "arm64"] {
        let dll = c
            .root
            .join(format!("build-{arch}/dll/win32/ucrtbase/ucrtbase.dll"));
        if !dll.is_file() {
            continue;
        }
        if burndown_stale(&c.root, files, arch).is_some() {
            continue;
        }
        let Some((exports, imports)) = pe_exports_imports(&bundled, &dll)
            .or_else(|| pe_exports_imports(Path::new("llvm-objdump"), &dll))
        else {
            out.push(format!(
                "ucrtbase {arch}: no usable llvm-objdump; cannot read {}",
                dll.display()
            ));
            continue;
        };
        for name in exports {
            if let Some(import) = imports.get(&name) {
                let allowed = [
                    "memchr", "memcmp", "memcpy", "memmove", "memset", "strchr", "strrchr",
                    "strstr", "wcschr", "wcsrchr", "wcsstr",
                ]
                .contains(&name.as_str())
                    && import.to_lowercase() == "ntdll.dll";
                if !allowed {
                    out.push(format!("ucrtbase {arch}: `{name}` is exported but answered by an import from {import} -- no object on the link line defines it, so the import library did. A takeover deleted its body without porting it. fix: port it into ucrt-rust (or, if the forward is deliberate, add it to EXPORT_IMPORT_ALLOWED with the reason)"));
                }
            }
        }
    }
    Ok(out)
}

fn burndown_stale(
    root: &Path,
    files: &BTreeMap<PathBuf, &SourceFile>,
    arch: &str,
) -> Option<String> {
    let triple = ARCHES.iter().find(|(a, _)| *a == arch)?.1;
    let Some(archive) = archive_path(&root.join(format!("build-{arch}")), triple) else {
        return Some("no libcrt_shared_cargo.a in the build".into());
    };
    if let Some(why) = stale_against(root, files, &archive) {
        return Some(why);
    }
    let Some(mut defs) = nm_defined(root, &[archive]) else {
        return Some("nm cannot read the archive".into());
    };
    if arch == "i386" {
        defs = defs
            .into_iter()
            .map(|x| x.strip_prefix('_').unwrap_or(&x).to_owned())
            .collect();
    }
    let declared = takeovers(root, files, Some(arch)).ok()?;
    let missing = declared
        .keys()
        .filter(|s| !defs.contains(*s))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Some(format!(
            "the archive lacks {} export(s) this tree declares taken over (e.g. {}) -- it was built from a different tree",
            missing.len(),
            missing.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    None
}

fn undecorate(s: &str) -> &str {
    s.trim_start_matches('_').split('@').next().unwrap_or(s)
}

fn burndown_count(
    root: &Path,
    files: &BTreeMap<PathBuf, &SourceFile>,
    arch: &str,
    triple: &str,
) -> Option<usize> {
    let build = root.join(format!("build-{arch}"));
    let ninja = std::fs::read_to_string(build.join("build.ninja")).ok()?;
    let linked_re = Regex::new(&format!(r"{}/\S+?\.obj", regex::escape(UCRT_OBJ_DIR))).ok()?;
    let ucrt_objs = linked_re
        .find_iter(&ninja)
        .map(|m| build.join(m.as_str()))
        .filter(|p| p.is_file())
        .collect::<BTreeSet<_>>();
    let mut ours_paths = Vec::new();
    for (_, rel) in OURS.iter().take(8) {
        ours_paths.extend(objects_under(&build.join(rel)));
    }
    ours_paths.extend(objects_under(
        &build.join("dll/win32/ucrtbase/CMakeFiles/ucrtbase.dir"),
    ));
    if let Some(a) = archive_path(&build, triple) {
        ours_paths.push(a);
    }
    let ours = nm_defined(root, &ours_paths)?
        .into_iter()
        .map(|s| undecorate(&s).to_owned())
        .collect::<BTreeSet<_>>();
    let mut ms = BTreeSet::new();
    for obj in ucrt_objs {
        if let Some(syms) = nm_defined(root, &[obj]) {
            ms.extend(syms.into_iter().map(|s| undecorate(&s).to_owned()));
        }
    }
    let spec = content(files, "dll/win32/ucrtbase/ucrtbase.spec")?;
    let exported = parse_spec_exports(spec, arch)
        .ok()?
        .into_iter()
        .map(|s| undecorate(&s).to_owned())
        .collect::<BTreeSet<_>>();
    Some(
        exported
            .intersection(&ms)
            .filter(|x| !ours.contains(*x))
            .count(),
    )
}

fn check_burndown(
    c: &Corpus,
    files: &BTreeMap<PathBuf, &SourceFile>,
) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let Some(yaml) = content(files, "tools/ucrt-takeover.yaml") else {
        return Ok(out);
    };
    let doc = crate::yaml::parse(yaml)?;
    let Some(ceilings) = doc.get("burndown").and_then(Value::as_object) else {
        return Ok(out);
    };
    for (arch, value) in ceilings {
        let Some(ceiling) = value.as_u64().map(|x| x as usize) else {
            continue;
        };
        let build = c.root.join(format!("build-{arch}"));
        if !build.is_dir() || burndown_stale(&c.root, files, arch).is_some() {
            continue;
        }
        let Some(triple) = ARCHES.iter().find(|(a, _)| *a == arch).map(|x| x.1) else {
            continue;
        };
        let Some(have) = burndown_count(&c.root, files, arch, triple) else {
            continue;
        };
        if have > ceiling {
            out.push(format!(
                "ucrt burn-down REGRESSED on {arch}: {have} exports are answered by vendored Microsoft C, up from the recorded {ceiling}. Something re-enabled a sdk/lib/ucrt body or dropped one of ours. Investigate before re-recording: the ceiling in tools/ucrt-takeover.yaml only ever goes DOWN. (python3 tools/ucrt_burndown.py --arch {arch})"
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(items: &[(&str, &str)]) -> BTreeMap<PathBuf, &'static SourceFile> {
        let owned = items
            .iter()
            .map(|(p, s)| {
                (
                    PathBuf::from(p),
                    SourceFile::new(PathBuf::from(p), (*s).into()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        // Keep the helper straightforward while returning the same borrowed-file shape
        // used by the scanner.
        Box::leak(Box::new(owned))
            .iter()
            .map(|(p, f)| (p.clone(), f))
            .collect()
    }

    #[test]
    fn structure_walks_transitive_edges_but_ignores_features_and_registry_deps() {
        let clean = files(&[
            ("sdk/lib/rust/crt-shared-cargo/Cargo.toml", "[dependencies]\nhelper = { path = \"../helper\" }\nlibm = { version = \"0.2\" }\n[features]\nwrong = [\"msvcrt-crt-rust/cpp\"]\n"),
            ("sdk/lib/rust/helper/Cargo.toml", "[dependencies]\ncrt-shared-rust = { path = \"../crt-shared-rust\" }\n"),
            ("sdk/lib/rust/crt-shared-rust/Cargo.toml", "[package]\nname=\"crt-shared-rust\"\n"),
        ]);
        assert!(check_structure(&clean).is_empty());

        let bad = files(&[
            (
                "sdk/lib/rust/crt-shared-cargo/Cargo.toml",
                "[dependencies]\nhelper = { path = \"../helper\" }\n",
            ),
            (
                "sdk/lib/rust/helper/Cargo.toml",
                "[dependencies]\nmsvcrt-crt-rust = { path = \"../msvcrt-crt-rust\" }\n",
            ),
        ]);
        let f = check_structure(&bad);
        assert_eq!(f.len(), 1);
        assert!(f[0].contains("crt-shared-cargo -> helper -> msvcrt-crt-rust"));
    }

    #[test]
    fn missing_crate_and_extern_crate_reachability_are_reported() {
        assert!(check_structure(&files(&[]))[0].contains("MISSING"));
        let f = files(&[
            ("sdk/lib/rust/crt-shared-cargo/Cargo.toml", "[dependencies]\ncnt-rust = { path = \"../cnt-rust\" }\npanic-common = { path = \"../panic-common\" }\n"),
            ("sdk/lib/rust/crt-shared-cargo/src/lib.rs", "extern crate panic_common;\n"),
        ]);
        let got = check_extern(&f);
        assert_eq!(got.len(), 1);
        assert!(got[0].contains("extern crate cnt_rust;"));
    }

    #[test]
    fn rebuild_lists_reject_hand_lists_and_wrong_roots() {
        let f = files(&[
            ("dll/win32/ucrtbase/CMakeLists.txt", "set(UCRT_SHARED_RUST_CRATES a b)\nrust_crate_sources(UCRT_SHARED_RUST_SOURCES sdk/lib/rust msvcrt-crt-cargo)\n"),
            ("dll/win32/msvcrt/CMakeLists.txt", "set(MSVCRT_CRT_RUST_CRATES a b)\n"),
        ]);
        let got = check_rebuild(&f);
        assert_eq!(got.len(), 4);
        assert!(got.iter().any(|x| x.contains("walks `msvcrt-crt-cargo`")));
        assert!(got.iter().any(|x| x.contains("hand-kept")));
    }

    #[test]
    fn hook_retention_and_win32_abi_checks_have_positive_and_negative_controls() {
        let hook = files(&[
            (
                "sdk/lib/ucrt/initializers/hook.cpp",
                "_CRTALLOC(\".CRT$XIC\") _PIFV const hook = f;",
            ),
            ("sdk/lib/rust/ucrt-rust/src/hook.rs", "// nothing"),
        ]);
        assert_eq!(check_force_includes(&hook).len(), 1);
        let pass = files(&[
            (
                "sdk/lib/ucrt/initializers/hook.cpp",
                "_CRTALLOC(\".CRT$XIC\") _PIFV const hook = f;",
            ),
            (
                "sdk/lib/rust/ucrt-rust/src/hook.rs",
                "crate::force_include!(hook);",
            ),
        ]);
        assert!(check_force_includes(&pass).is_empty());

        let abi = files(&[
            ("sdk/lib/ucrt/inc/api.h", "BOOL WINAPI ReadFile(void);"),
            ("sdk/lib/rust/ucrt-rust/src/api.rs", "extern \"C\" { fn ReadFile(); fn malloc();\n}\nextern \"system\" { fn WriteFile(); }"),
        ]);
        let got = check_win32_abi(&abi);
        assert_eq!(got.len(), 1);
        assert!(got[0].contains("ReadFile"));
    }

    #[test]
    fn spec_arch_filters_and_unknown_tags_are_exact() {
        let spec = "@ stdcall -arch=!i386 sqrtf()\n@ stdcall -arch=i386 sqrt()\n";
        assert!(!parse_spec_exports(spec, "i386").unwrap().contains("sqrtf"));
        assert!(parse_spec_exports(spec, "amd64").unwrap().contains("sqrtf"));
        assert!(parse_spec_exports("@ stdcall -arch=madeup bad()\n", "amd64").is_err());
    }

    #[test]
    fn build_ninja_authority_excludes_objects_not_built_by_current_graph() {
        let t = std::env::temp_dir().join(format!("ucrt-cohab-{}", std::process::id()));
        let d = t.join("sdk/lib/vcruntime/CMakeFiles/vcruntime.dir/i386");
        std::fs::create_dir_all(&d).unwrap();
        let live = t.join("sdk/lib/vcruntime/CMakeFiles/vcruntime.dir/purecall.cpp.obj");
        let stale = d.join("_chkesp.s.obj");
        std::fs::write(&live, []).unwrap();
        std::fs::write(&stale, []).unwrap();
        std::fs::write(t.join("build.ninja"), "build sdk/lib/vcruntime/CMakeFiles/vcruntime.dir/purecall.cpp.obj: CXX_COMPILER x.cpp\nbuild sdk/lib/vcruntime/libvcruntime.a: C_STATIC_LIBRARY_LINKER x\n").unwrap();
        let graph = graph_objects(&t).unwrap();
        assert!(graph.contains(&live));
        assert!(!graph.contains(&stale));
        let _ = std::fs::remove_dir_all(t);
    }
}
