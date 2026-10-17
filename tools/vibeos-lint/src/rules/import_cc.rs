use crate::{
    rules::cc_mismatch::{self, convention_facts},
    Corpus, Rule, SourceFile, SourceSet,
};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::LazyLock,
};

pub struct ImportCc;
static SOURCES: &[SourceSet] = &[
    SourceSet::tree("abi_specs", &[""], &["spec"], &[]).with_pruning(&[
        ".git",
        "target",
        "build-i386",
        "build-amd64",
        "build-arm64",
    ]),
    SourceSet::tree("abi_cmake", &[""], &[], &["CMakeLists.txt"]).with_pruning(&[
        ".git",
        "target",
        "build-i386",
        "build-amd64",
        "build-arm64",
    ]),
    SourceSet::tree("abi_module_c", &[""], &["c", "h"], &[]).with_pruning(&[
        ".git",
        "target",
        "build-i386",
        "build-amd64",
        "build-arm64",
    ]),
];

static SPEC_LINE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^\s*(?:@|\d+)\s+(\w+)\s+((?:-\S+\s+)*)([\w?@$.]+)").unwrap()
});
static ARCH: LazyLock<Regex> = LazyLock::new(|| crate::regexes::cached(r"-arch=(\S+)").unwrap());
static ADD_IMPORTLIBS: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?s)add_importlibs\s*\(([^)]*)\)").unwrap());
static C_DEFINE: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^[A-Za-z_][\w\s*]*?\bNAME\s*\([^;{}]*\)\s*\{").unwrap()
});
static C_DEFINE_INDEX: LazyLock<Regex> = LazyLock::new(|| {
    crate::regexes::cached(r"(?m)^[A-Za-z_][\w\s*]*?\b([A-Za-z_]\w*)\s*\([^;{}]*\)\s*\{").unwrap()
});
static NOLINT: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"nolint:\s*import-cc").unwrap());
static ISLAND: LazyLock<Regex> =
    LazyLock::new(|| crate::regexes::cached(r"(?:^|/)(?:types|bindgen)\w*\.rs$").unwrap());

fn spec_conv(c: &str) -> Option<&'static str> {
    match c {
        "stdcall" => Some("stdcall"),
        "cdecl" | "varargs" => Some("C"),
        "fastcall" => Some("fastcall"),
        _ => None,
    }
}
fn includes_i386(list: &str) -> bool {
    let mut included = false;
    for item in list.split(',') {
        let neg = item.starts_with('!');
        if (matches!(item.trim_start_matches('!'), "i386" | "win32")) != neg {
            included = true;
        }
    }
    included
}

pub(crate) fn read_spec(text: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
    for line in text.lines() {
        if line.trim_start().starts_with(';') {
            continue;
        }
        let Some(m) = SPEC_LINE.captures(line) else {
            continue;
        };
        let Some(cc) = spec_conv(&m[1]) else { continue };
        let flags = &m[2];
        if flags.split_whitespace().any(|f| f == "-private") {
            continue;
        }
        if ARCH.captures(flags).is_some_and(|a| !includes_i386(&a[1])) {
            continue;
        }
        let raw = &m[3];
        let name = if raw.starts_with('?') {
            raw.to_owned()
        } else {
            raw.split('@').next().unwrap_or(raw).to_owned()
        };
        out.entry(name).or_default().insert(cc.to_owned());
    }
    out
}

fn index_specs<'a>(
    files: impl Iterator<Item = SourceFileRef<'a>>,
) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::<String, Vec<String>>::new();
    for f in files {
        if let Some(name) = f.path.file_stem().and_then(|s| s.to_str()) {
            out.entry(name.to_owned())
                .or_default()
                .push(f.path.to_string_lossy().replace('\\', "/"));
        }
    }
    out
}
struct SourceFileRef<'a> {
    path: &'a Path,
    text: &'a str,
}

fn importlibs<'a>(
    files: impl Iterator<Item = SourceFileRef<'a>>,
) -> (
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, BTreeSet<String>>,
) {
    let mut target = BTreeMap::<String, BTreeSet<String>>::new();
    let mut byfile = BTreeMap::<String, BTreeSet<String>>::new();
    for f in files {
        let path = f.path.to_string_lossy().replace('\\', "/");
        for m in ADD_IMPORTLIBS.captures_iter(f.text) {
            let mut tokens = m[1].split_whitespace();
            let Some(t) = tokens.next() else { continue };
            for lib in tokens {
                target
                    .entry(t.to_owned())
                    .or_default()
                    .insert(lib.to_owned());
                byfile
                    .entry(path.clone())
                    .or_default()
                    .insert(lib.to_owned());
            }
        }
    }
    (target, byfile)
}

fn crate_link_set(
    crate_name: &str,
    bycrate: &BTreeMap<String, BTreeSet<String>>,
    targets: &BTreeMap<String, BTreeSet<String>>,
    byfile: &BTreeMap<String, BTreeSet<String>>,
    specs: &BTreeMap<String, Vec<String>>,
) -> Option<(BTreeSet<String>, Vec<String>)> {
    if let Some(files) = bycrate.get(crate_name) {
        let mut libs = BTreeSet::new();
        for file in files {
            libs.extend(byfile.get(file).into_iter().flatten().cloned());
        }
        return Some((
            libs,
            files
                .iter()
                .map(|f| {
                    Path::new(f)
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_string_lossy()
                        .into_owned()
                })
                .collect(),
        ));
    }
    let mut stem = crate_name.to_owned();
    for suffix in ["-rust", "-sys", "-cargo"] {
        if let Some(s) = stem.strip_suffix(suffix) {
            stem = s.to_owned();
        }
    }
    if !targets.contains_key(&stem) || !specs.contains_key(&stem) {
        return None;
    }
    Some((
        targets.get(&stem).cloned().unwrap_or_default(),
        specs[&stem]
            .iter()
            .map(|p| {
                Path::new(p)
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_string_lossy()
                    .into_owned()
            })
            .collect(),
    ))
}

fn c_defines(module_dirs: &[String], name: &str, cfiles: &[&SourceFile]) -> Option<String> {
    let pat = C_DEFINE.as_str().replace("NAME", &regex::escape(name));
    let re = crate::regexes::cached(&pat).unwrap();
    cfiles.iter().find_map(|f| {
        let path = f.path.to_string_lossy().replace('\\', "/");
        let in_dir = module_dirs.iter().any(|d| {
            path == *d
                || path
                    .strip_prefix(d.as_str())
                    .is_some_and(|tail| tail.starts_with('/'))
        });
        (in_dir && re.is_match(&f.text)).then(|| path)
    })
}
type CDefIndex = Vec<(String, BTreeSet<String>)>;
fn c_def_index(cfiles: &[&SourceFile]) -> CDefIndex {
    cfiles
        .iter()
        .map(|f| {
            let path = f.path.to_string_lossy().replace('\\', "/");
            let names = C_DEFINE_INDEX
                .captures_iter(&f.text)
                .map(|m| m[1].to_owned())
                .collect();
            (path, names)
        })
        .collect()
}
fn indexed_c_defines(module_dirs: &[String], name: &str, index: &CDefIndex) -> Option<String> {
    index.iter().find_map(|(path, names)| {
        let in_dir = module_dirs.iter().any(|d| {
            path == d
                || path
                    .strip_prefix(d.as_str())
                    .is_some_and(|tail| tail.starts_with('/'))
        });
        (in_dir && names.contains(name)).then(|| path.clone())
    })
}

fn crates_named_by(cmake: &[&SourceFile], crates: &[String]) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    for name in crates {
        for f in cmake {
            let p = f.path.to_string_lossy().replace('\\', "/");
            if contains_bounded_name(&f.text, name) {
                out.entry(name.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(p);
            }
        }
    }
    out
}

fn contains_bounded_name(haystack: &str, needle: &str) -> bool {
    fn is_word_or_dash(c: char) -> bool {
        c == '-' || c == '_' || c.is_alphanumeric()
    }
    haystack.match_indices(needle).any(|(at, _)| {
        let before_ok = haystack[..at]
            .chars()
            .next_back()
            .map_or(true, |c| !is_word_or_dash(c));
        let after = at + needle.len();
        let after_ok = haystack[after..]
            .chars()
            .next()
            .map_or(true, |c| !is_word_or_dash(c));
        before_ok && after_ok
    })
}

fn scan_crate(
    crate_files: &[&SourceFile],
    crate_name: &str,
    oracle: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    module_dirs: &[String],
    cfiles: &[&SourceFile],
) -> Vec<String> {
    let facts = crate_files
        .iter()
        .map(|f| (f.path.clone(), convention_facts(&f.text)))
        .collect::<BTreeMap<_, _>>();
    let c_index = c_def_index(cfiles);
    scan_crate_indexed(
        crate_files,
        crate_name,
        oracle,
        module_dirs,
        &facts,
        &c_index,
    )
}
fn scan_crate_indexed(
    crate_files: &[&SourceFile],
    crate_name: &str,
    oracle: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    module_dirs: &[String],
    facts: &BTreeMap<
        std::path::PathBuf,
        (
            Vec<(String, String, bool)>,
            Vec<(String, String, usize, String)>,
        ),
    >,
    c_index: &CDefIndex,
) -> Vec<String> {
    let mut defs = BTreeMap::<String, BTreeSet<String>>::new();
    let mut out = Vec::new();
    for f in crate_files {
        let path = f.path.to_string_lossy().replace('\\', "/");
        if !cc_mismatch::included_i386(&f.path, &f.text) {
            continue;
        }
        for (n, cc, excluded) in &facts[&f.path].0 {
            if !excluded {
                defs.entry(n.clone())
                    .or_default()
                    .insert(normalize(cc).to_owned());
            }
        }
    }
    let mut local = BTreeMap::<String, Option<String>>::new();
    for f in crate_files {
        let path = f.path.to_string_lossy().replace('\\', "/");
        if !cc_mismatch::included_i386(&f.path, &f.text) {
            continue;
        }
        for (name, cc, _line, text) in &facts[&f.path].1 {
            if defs.contains_key(name) || NOLINT.is_match(text) {
                continue;
            }
            let owners = oracle
                .iter()
                .filter(|(_, map)| map.contains_key(name))
                .map(|(lib, map)| (lib, map[name].clone()))
                .collect::<Vec<_>>();
            let convs = owners
                .iter()
                .flat_map(|(_, s)| s.iter().cloned())
                .collect::<BTreeSet<_>>();
            if convs.len() != 1 {
                continue;
            }
            let want = convs.iter().next().unwrap();
            let got = normalize(cc);
            if got == want {
                continue;
            }
            if !local.contains_key(name) {
                local.insert(name.clone(), indexed_c_defines(module_dirs, name, c_index));
            }
            if local[name].is_some() {
                continue;
            }
            let tag = if ISLAND.is_match(&path) {
                "IMPORT_CC_ISLAND"
            } else {
                "IMPORT_CC"
            };
            let owner = owners
                .iter()
                .map(|(s, _)| s.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("/");
            out.push(format!("{path}: [{tag}] {name} declared extern \"{cc}\" but {owner}.spec exports it {want} (i386 link wants {}, we emit {})",decoration(want),decoration(got)));
        }
    }
    let _ = crate_name;
    out
}
fn normalize(cc: &str) -> &str {
    cc_mismatch::NORM_I386
        .iter()
        .find(|(a, _)| *a == cc)
        .map_or(cc, |(_, b)| b)
}
fn decoration(cc: &str) -> &str {
    match cc {
        "stdcall" => "_Name@N",
        "fastcall" => "@Name@N",
        "C" => "_Name",
        _ => cc,
    }
}

impl Rule for ImportCc {
    fn name(&self) -> &'static str {
        "scan_import_cc"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        SOURCES
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let specs_files = c
            .sources("abi_specs")
            .filter(|f| f.path.extension().is_some_and(|e| e == "spec"))
            .map(|f| SourceFileRef {
                path: &f.path,
                text: &f.text,
            })
            .collect::<Vec<_>>();
        let specs = index_specs(specs_files.iter().map(|f| SourceFileRef {
            path: f.path,
            text: f.text,
        }));
        let mut exports = BTreeMap::<String, BTreeMap<String, BTreeSet<String>>>::new();
        for (module, paths) in &specs {
            let mut merged = BTreeMap::<String, BTreeSet<String>>::new();
            for path in paths {
                if let Some(f) = c
                    .sources("abi_specs")
                    .find(|f| f.path.to_string_lossy().replace('\\', "/") == *path)
                {
                    for (name, cc) in read_spec(&f.text) {
                        merged.entry(name).or_default().extend(cc);
                    }
                }
            }
            exports.insert(module.clone(), merged);
        }
        let cmake = c
            .sources("abi_cmake")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "CMakeLists.txt"))
            .collect::<Vec<_>>();
        let cmake_refs = cmake.iter().copied().collect::<Vec<_>>();
        let (targets, byfile) = importlibs(cmake.iter().map(|f| SourceFileRef {
            path: &f.path,
            text: &f.text,
        }));
        let crate_names = c
            .rust_files()
            .filter_map(|f| crate_root(&f.path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|p| {
                Path::new(&p)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        let bycrate = crates_named_by(&cmake_refs, &crate_names);
        let rust = c.rust_files().collect::<Vec<_>>();
        let mut groups = BTreeMap::<String, Vec<&SourceFile>>::new();
        for f in &rust {
            if let Some(k) = crate_root(&f.path) {
                groups.entry(k).or_default().push(f);
            }
        }
        let cfiles = c
            .sources("abi_module_c")
            .filter(|f| f.path.extension().is_some_and(|e| e == "c" || e == "h"))
            .collect::<Vec<_>>();
        let cfiles = cfiles.iter().copied().collect::<Vec<_>>();
        let fact_index = rust
            .iter()
            .map(|f| (f.path.clone(), convention_facts(&f.text)))
            .collect::<BTreeMap<_, _>>();
        let c_index = c_def_index(&cfiles);
        let mut findings = Vec::new();
        for (path, files) in groups {
            let name = Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let Some((libs, dirs)) = crate_link_set(&name, &bycrate, &targets, &byfile, &specs)
            else {
                continue;
            };
            if libs.is_empty() {
                continue;
            }
            let oracle = libs
                .into_iter()
                .filter_map(|lib| exports.get(&lib).map(|e| (lib, e.clone())))
                .collect::<BTreeMap<_, _>>();
            if oracle.is_empty() {
                continue;
            }
            findings.extend(scan_crate_indexed(
                &files,
                &name,
                &oracle,
                &dirs,
                &fact_index,
                &c_index,
            ));
        }
        findings.sort();
        findings.dedup();
        Ok(findings)
    }
}
fn crate_root(path: &Path) -> Option<String> {
    let s = path.to_string_lossy().replace('\\', "/");
    s.split("/src/").next().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_c_definition_lookup_matches_legacy_pattern() {
        let files = vec![
            SourceFile::new(
                "dll/demo/a.c".into(),
                "DWORD SetErrorMode(UINT mode) { return 0; }\n".into(),
            ),
            SourceFile::new(
                "dll/demo/b.c".into(),
                "static void *AcquireFastMutex(void *p) { return p; }\n".into(),
            ),
            SourceFile::new(
                "dll/other/a.c".into(),
                "DWORD Outside(UINT mode) { return 0; }\n".into(),
            ),
            SourceFile::new(
                "dll/demo/proto.h".into(),
                "DWORD Prototype(UINT mode);\n".into(),
            ),
        ];
        let refs = files.iter().collect::<Vec<_>>();
        let index = c_def_index(&refs);
        let dirs = vec!["dll/demo".to_owned()];
        for name in [
            "SetErrorMode",
            "AcquireFastMutex",
            "Outside",
            "Prototype",
            "Missing",
        ] {
            assert_eq!(
                c_defines(&dirs, name, &refs),
                indexed_c_defines(&dirs, name, &index),
                "{name}"
            );
        }
    }
    #[test]
    fn spec_parser_preserves_i386_scope_and_flags() {
        let spec = "@ stdcall SetErrorMode(long)\n@ stdcall -norelay GetProcessHeap()\n@ cdecl -arch=i386 memcpy(ptr ptr long)\n@ stdcall -arch=arm64 Acquire(ptr)\n@ cdecl -arch=!i386 _chgsignf(float)\n@ stdcall -arch=win32,arm64 Both(long)\n@ cdecl -arch=win64 SixtyFourOnly(long)\n@ stdcall -private Private(long)\n@ stub Stub\n@ extern Data\n;@ stdcall Commented(long)\n";
        let o = read_spec(spec);
        assert_eq!(o["SetErrorMode"], BTreeSet::from(["stdcall".into()]));
        assert_eq!(o["GetProcessHeap"], BTreeSet::from(["stdcall".into()]));
        assert_eq!(o["memcpy"], BTreeSet::from(["C".into()]));
        assert_eq!(o["Both"], BTreeSet::from(["stdcall".into()]));
        for n in [
            "Acquire",
            "_chgsignf",
            "SixtyFourOnly",
            "Private",
            "Stub",
            "Data",
            "Commented",
        ] {
            assert!(!o.contains_key(n), "{n}");
        }
    }

    #[test]
    fn import_link_and_island_findings() {
        let spec = read_spec("@ stdcall SetErrorMode(long)\n@ cdecl memcpy(ptr ptr long)\n");
        let oracle = BTreeMap::from([("kernel32".into(), spec)]);
        let f = SourceFile::new(
            "sdk/lib/rust/demo-rust/src/bindgen_i386.rs".into(),
            "extern \"C\" { fn SetErrorMode(m:u32)->u32; }".into(),
        );
        let got = scan_crate(&[&f], "demo-rust", &oracle, &[], &[]);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].contains("[IMPORT_CC_ISLAND]") && got[0].contains("SetErrorMode"));
        let f = SourceFile::new(
            "sdk/lib/rust/demo-rust/src/lib.rs".into(),
            "extern \"system\" { fn SetErrorMode(m:u32)->u32; }".into(),
        );
        assert!(scan_crate(&[&f], "demo-rust", &oracle, &[], &[]).is_empty());
    }

    #[test]
    fn local_definition_nolint_unknown_and_symbol_resolution() {
        let oracle = BTreeMap::from([(
            "kernel32".into(),
            read_spec("@ stdcall SetErrorMode(long)\n@ fastcall ExAcquireFastMutexUnsafe(ptr)\n"),
        )]);
        let clean = SourceFile::new(
            "sdk/lib/rust/demo-rust/src/lib.rs".into(),
            "extern \"system\" { fn SetErrorMode(m:u32)->u32; }\n#[no_mangle]\nunsafe extern \"C\" fn SetErrorMode(m:u32)->u32 {0}\n".into(),
        );
        // Same-crate definition means this is not an import oracle question.
        assert!(scan_crate(&[&clean], "demo-rust", &oracle, &[], &[]).is_empty());
        let waived = SourceFile::new(
            "sdk/lib/rust/demo-rust/src/lib.rs".into(),
            "extern \"C\" { fn SetErrorMode(m:u32)->u32; } // nolint: import-cc reviewed\n".into(),
        );
        assert!(scan_crate(&[&waived], "demo-rust", &oracle, &[], &[]).is_empty());
        let unknown = SourceFile::new(
            "sdk/lib/rust/demo-rust/src/lib.rs".into(),
            "extern \"C\" { fn NotInSpec(m:u32)->u32; }\n".into(),
        );
        assert!(scan_crate(&[&unknown], "demo-rust", &oracle, &[], &[]).is_empty());
        let alias = SourceFile::new(
            "sdk/lib/rust/demo-rust/src/lib.rs".into(),
            "extern \"C\" { #[link_name = \"SetErrorMode\"] fn Alias(m:u32)->u32; }\n".into(),
        );
        assert_eq!(
            scan_crate(&[&alias], "demo-rust", &oracle, &[], &[]).len(),
            1
        );
    }

    #[test]
    fn module_c_definition_must_be_a_body_and_crate_link_selection_is_scoped() {
        let c = SourceFile::new(
            "dll/demo/shim.c".into(),
            "DWORD SetErrorMode(UINT mode)\n{\n return 0;\n}\n".into(),
        );
        assert!(c_defines(&["dll/demo".into()], "SetErrorMode", &[&c]).is_some());
        let prototype = SourceFile::new(
            "dll/demo/use.c".into(),
            "DWORD SetErrorMode(UINT mode);\nvoid f(void) { SetErrorMode(1); }\n".into(),
        );
        assert!(c_defines(&["dll/demo".into()], "SetErrorMode", &[&prototype]).is_none());
        let targets = BTreeMap::from([
            ("ps".into(), BTreeSet::from(["kernel32".into()])),
            ("wldap32".into(), BTreeSet::from(["kernel32".into()])),
        ]);
        let specs = BTreeMap::from([("wldap32".into(), vec!["dll/wldap32/wldap32.spec".into()])]);
        assert!(crate_link_set(
            "ps-rust",
            &BTreeMap::new(),
            &targets,
            &BTreeMap::new(),
            &specs
        )
        .is_none());
        assert_eq!(
            crate_link_set(
                "wldap32-rust",
                &BTreeMap::new(),
                &targets,
                &BTreeMap::new(),
                &specs
            )
            .unwrap()
            .0,
            BTreeSet::from(["kernel32".into()])
        );
        let named = BTreeMap::from([(
            "kernel-rust".into(),
            BTreeSet::from(["ntoskrnl/CMakeLists.txt".into()]),
        )]);
        assert!(crate_link_set(
            "kernel-rust",
            &named,
            &targets,
            &BTreeMap::new(),
            &BTreeMap::new()
        )
        .unwrap()
        .0
        .is_empty());
    }
    #[test]
    fn arch_and_parked_files_are_outside_python_i386_walk() {
        let spec = BTreeMap::from([(
            "kernel32".into(),
            read_spec("@ stdcall SetErrorMode(long)\n"),
        )]);
        for path in [
            "sdk/lib/rust/demo-rust/src/types_amd64.rs",
            "sdk/lib/rust/demo-rust/src/arm64/decl.rs",
            "sdk/lib/rust/demo-rust/src/parked.rs",
        ] {
            let body = if path.ends_with("parked.rs") {
                "//! ORPHAN-BY-DESIGN: intentionally parked\nextern \"C\" { fn SetErrorMode(x:u32); }\n"
            } else {
                "extern \"C\" { fn SetErrorMode(x:u32); }\n"
            };
            let f = SourceFile::new(path.into(), body.into());
            assert!(
                scan_crate(&[&f], "demo-rust", &spec, &[], &[]).is_empty(),
                "{path}"
            );
        }
    }
}
