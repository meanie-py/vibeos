//! Ratchet fully-Rust modules still linked by CMake around rust_cohab_stub.
//! The source-side filter mirrors scan_cmake_linked_rust.py; build-graph discovery
//! mirrors cargo_cell_gen.survey/link_edge and intentionally reads generated build data.
use crate::{Corpus, Rule};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub struct ScanCmakeLinkedRust;
static SURVEY_EDGE: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?m)^build \S+: C_\w+?_LINKER__(\w+?)_(?:Debug|Release)\S* ([^|\n]*)"));
static CMAKE_CMD: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?m)^[ \t]*([A-Za-z_][A-Za-z0-9_]*)[ \t]*\("));
static FACT_TARGET: LazyLock<Regex> = LazyLock::new(|| re(r"add_cargo_module\(\s*([\w.+-]+)"));
static STUB_TARGET: LazyLock<Regex> =
    LazyLock::new(|| re(r"add_(?:library|executable)\(\s*([\w.+-]+)"));
static BUILD_RS_LIB: LazyLock<Regex> = LazyLock::new(|| re(r#""([^"\n]+\.a)""#));
static LIB_ARCHIVE: LazyLock<Regex> = LazyLock::new(|| re(r"(?:.*/)?lib([\w.+-]+)\.a$"));
static INDEX_TARGET: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?m)^[ \t]*(add_library|rust_staticlib_variant|add_dependencies)[ \t]*\(")
});
static COMMENT: LazyLock<Regex> = LazyLock::new(|| re(r"(?m)//[^\n]*"));
fn re(p: &str) -> Regex {
    crate::regexes::cached(p).expect("valid cmake-linked-rust regex")
}

const CARGO_CELL_GEN_SOURCE: &str = include_str!("../../../cargo_cell_gen.py");

struct EngineTables {
    libs: BTreeMap<String, String>,
    deps: BTreeMap<String, String>,
}

static ENGINE_TABLES: LazyLock<Result<EngineTables, String>> =
    LazyLock::new(|| parse_engine_tables(CARGO_CELL_GEN_SOURCE));

/// Parsed directly from cargo_cell_gen.py so the checker cannot silently drift from
/// the converter's hand-maintained archive maps.
pub fn engine_table() -> Result<&'static BTreeMap<String, String>, String> {
    ENGINE_TABLES
        .as_ref()
        .map(|tables| &tables.libs)
        .map_err(Clone::clone)
}

pub fn engine_deps_table() -> Result<&'static BTreeMap<String, String>, String> {
    ENGINE_TABLES
        .as_ref()
        .map(|tables| &tables.deps)
        .map_err(Clone::clone)
}

fn parse_engine_tables(source: &str) -> Result<EngineTables, String> {
    fn parse(source: &str, name: &str) -> Result<BTreeMap<String, String>, String> {
        let start = source
            .find(&format!("{name} = {{"))
            .ok_or_else(|| format!("cargo_cell_gen.py has no {name} dictionary"))?;
        let tail = &source[start..];
        let end = tail
            .find("\n}")
            .ok_or_else(|| format!("cargo_cell_gen.py {name} dictionary is unterminated"))?;
        let body = &tail[tail.find('\n').unwrap_or(0) + 1..end];
        let entry = re(r#"^\s*"([^"]+)"\s*:\s*"([^"]+)"\s*,?\s*$"#);
        let mut rows = BTreeMap::new();
        for line in body.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let caps = entry
                .captures(line)
                .ok_or_else(|| format!("unrecognized {name} dictionary row: {line}"))?;
            let key = caps[1].to_owned();
            let value = caps[2].to_owned();
            if rows.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate {name} dictionary key {key}"));
            }
        }
        if rows.is_empty() {
            return Err(format!("cargo_cell_gen.py {name} dictionary is empty"));
        }
        Ok(rows)
    }
    Ok(EngineTables {
        libs: parse(source, "ENGINE_LIBS")?,
        deps: parse(source, "ENGINE_LIB_DEPS")?,
    })
}

fn read(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|b| {
            String::from_utf8_lossy(&b)
                .replace("\r\n", "\n")
                .replace('\r', "\n")
        })
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn walk(
    root: &Path,
    out: &mut Vec<PathBuf>,
    predicate: &dyn Fn(&Path) -> bool,
) -> Result<(), String> {
    walk_from(root, root, out, predicate)
}
fn walk_from(
    base: &Path,
    root: &Path,
    out: &mut Vec<PathBuf>,
    predicate: &dyn Fn(&Path) -> bool,
) -> Result<(), String> {
    let entries = fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel = path.strip_prefix(base).unwrap_or(&path);
        let top = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("");
        if top.starts_with("build-") || top.starts_with('.') {
            continue;
        }
        if path.is_dir()
            && !fs::symlink_metadata(&path)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
        {
            walk_from(base, &path, out, predicate)?;
        } else if predicate(&path) {
            out.push(path);
        }
    }
    Ok(())
}
fn path_is_cmake(p: &Path) -> bool {
    p.file_name().is_some_and(|n| n == "CMakeLists.txt")
        || p.extension().is_some_and(|x| x == "cmake")
}
fn source_facts(repo: &Path) -> Result<(BTreeSet<String>, BTreeSet<String>), String> {
    let mut files = Vec::new();
    walk(repo, &mut files, &path_is_cmake)?;
    let mut cells = BTreeSet::new();
    let mut stubbed = BTreeSet::new();
    for p in files {
        let text = read(&p)?;
        if text.contains("add_cargo_module") {
            cells.extend(FACT_TARGET.captures_iter(&text).map(|c| c[1].to_owned()));
        }
        if text.contains("rust_cohab_stub") {
            stubbed.extend(STUB_TARGET.captures_iter(&text).map(|c| c[1].to_owned()));
        }
    }
    Ok((cells, stubbed))
}
fn arch_link_exists(repo: &Path, arch: &str, target: &str) -> Result<bool, String> {
    let p = repo.join(format!("build-{arch}/build.ninja"));
    let ninja = read(&p)?;
    let target_re = re(&format!(
        r"(?m)^build \S+: C_\w+?_LINKER__{}_(?:Debug|Release|RelWithDebInfo|MinSizeRel)(?:\s|$)",
        regex::escape(target)
    ));
    Ok(target_re.is_match(&ninja))
}
fn library_index(
    repo: &Path,
) -> Result<BTreeMap<String, Option<(String, Vec<String>, Vec<String>)>>, String> {
    // cargo_cell_gen._library_index: duplicate target definitions are deliberately
    // ambiguous; they never resolve to an engine-built archive.
    let mut cmakes = Vec::new();
    walk(repo, &mut cmakes, &|p| {
        p.file_name().is_some_and(|n| n == "CMakeLists.txt")
    })?;
    let mut defs: BTreeMap<String, Vec<(String, Vec<String>)>> = BTreeMap::new();
    let mut deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in cmakes {
        let text = read(&p)?;
        for (cmd, args) in cmake_statements(&text) {
            if args.is_empty() {
                continue;
            }
            if cmd == "add_library" || cmd == "rust_staticlib_variant" {
                defs.entry(args[0].clone()).or_default().push((cmd, args));
            } else if cmd == "add_dependencies" {
                deps.entry(args[0].clone())
                    .or_default()
                    .extend(args.into_iter().skip(1));
            }
        }
    }
    Ok(defs
        .into_iter()
        .map(|(name, rows)| {
            let val = if rows.len() == 1 {
                let (kind, args) = rows.into_iter().next().unwrap();
                Some((kind, args, deps.remove(&name).unwrap_or_default()))
            } else {
                None
            };
            (name, val)
        })
        .collect())
}
fn cmake_statements_raw(text: &str) -> Vec<(String, Vec<String>, String)> {
    let mut out = Vec::new();
    let mut consumed = 0usize;
    for m in CMAKE_CMD.find_iter(text) {
        if m.start() < consumed {
            continue;
        }
        let mut depth = 0i32;
        let mut j = m.end() - 1;
        while j < text.len() {
            let b = text.as_bytes()[j];
            if b == b'(' {
                depth += 1
            } else if b == b')' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            } else if b == b'#'
                && depth == 1
                && (j == 0 || matches!(text.as_bytes()[j - 1], b' ' | b'\t' | b'\n'))
            {
                while j < text.len() && text.as_bytes()[j] != b'\n' {
                    j += 1;
                }
                if j > 0 {
                    j -= 1;
                }
            }
            j += 1;
        }
        if j > text.len() {
            break;
        }
        let raw = &text[m.end()..j];
        let clean = re(r"(?m)#[^\n]*").replace_all(raw, "");
        let args = re(r#""[^"
]*"|\S+"#)
        .find_iter(&clean)
        .map(|x| x.as_str().to_string())
        .collect();
        let command = m
            .as_str()
            .trim()
            .split('(')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        out.push((command, args, raw.to_owned()));
        consumed = j + 1;
    }
    out
}
fn cmake_statements(text: &str) -> Vec<(String, Vec<String>)> {
    cmake_statements_raw(text)
        .into_iter()
        .map(|(c, a, _)| (c, a))
        .collect()
}
fn engine_buildable(
    lib: &str,
    index: &BTreeMap<String, Option<(String, Vec<String>, Vec<String>)>>,
) -> bool {
    // Exact hand-maintained cargo_cell_gen.ENGINE_LIBS table (the only results needed
    // by this checker are the yes/no labels).
    if hand_engine_lib(lib) {
        return true;
    }
    let Some(c) = LIB_ARCHIVE.captures(lib) else {
        return false;
    };
    let Some(Some((kind, args, deps))) = index.get(&c[1]) else {
        return false;
    };
    if kind == "rust_staticlib_variant" {
        return args.len() >= 4 && !args[4..].iter().any(|x| x == "ENV");
    }
    !args.get(1).is_some_and(|x| {
        [
            "SHARED",
            "MODULE",
            "OBJECT",
            "INTERFACE",
            "IMPORTED",
            "ALIAS",
        ]
        .contains(&x.as_str())
    })
}

fn scan(repo: &Path) -> Result<Vec<String>, String> {
    engine_table()?;
    engine_deps_table()?;
    let amd64 = read(&repo.join("build-amd64/build.ninja"))?;
    let (cells, stubbed) = source_facts(repo)?;
    let mut candidates = BTreeMap::<String, Vec<Vec<String>>>::new();
    for c in SURVEY_EDGE.captures_iter(&amd64) {
        let target = c[1].to_string();
        if c[2]
            .split_whitespace()
            .any(|x| x.rsplit('/').next() == Some("rust_cohab_stub.c.obj"))
        {
            candidates
                .entry(target)
                .or_default()
                .push(c[2].split_whitespace().map(str::to_owned).collect());
        }
    }
    let mut rows = Vec::new();
    let mut spec_cache = None;
    let mut library_cache = None;
    for target in candidates.keys() {
        if cells.contains(target) || !stubbed.contains(target) {
            continue;
        }
        for _inputs in &candidates[target] {
            let label = if ["i386", "amd64", "arm64"]
                .iter()
                .all(|a| arch_link_exists(repo, a, target).unwrap_or(false))
            {
                match model_convertible(repo, target, &mut spec_cache, &mut library_cache) {
                    Ok(()) => "convertible",
                    Err(e) if e.starts_with("PE_FATAL:") => {
                        return Err(e.trim_start_matches("PE_FATAL:").to_owned());
                    }
                    Err(_) => "refused",
                }
            } else {
                "refused"
            };
            rows.push(format!(
                "{target}: fully-Rust module still linked by cmake ({label})"
            ));
        }
    }
    let mut build_rs = Vec::new();
    let rust = repo.join("sdk/lib/rust");
    if rust.is_dir() {
        for ent in fs::read_dir(&rust).map_err(|e| e.to_string())? {
            let p = ent.map_err(|e| e.to_string())?.path();
            if p.is_dir()
                && p.file_name()
                    .and_then(|x| x.to_str())
                    .is_some_and(|n| n.ends_with("-cargo"))
            {
                let b = p.join("build.rs");
                if b.is_file() {
                    build_rs.push(b)
                }
            }
        }
    }
    build_rs.sort();
    let mut out = rows;
    let mut pending = Vec::new();
    for p in build_rs {
        let crate_name = p
            .parent()
            .and_then(|p| p.file_name())
            .unwrap_or_default()
            .to_string_lossy();
        let source = read(&p)?;
        let code = COMMENT.replace_all(&source, "");
        let libs: BTreeSet<String> = BUILD_RS_LIB
            .captures_iter(&code)
            .map(|c| c[1].to_owned())
            .collect();
        if !libs.is_empty() {
            pending.push((crate_name.into_owned(), libs));
        }
    }
    if !pending.is_empty() {
        if library_cache.is_none() {
            library_cache = Some(library_index(repo)?);
        }
        let index = library_cache.as_ref().unwrap();
        for (crate_name, libs) in pending {
            for lib in libs {
                let label = if engine_buildable(&lib, &index) {
                    "engine-buildable"
                } else {
                    "not engine-buildable"
                };
                out.push(format!("{crate_name}: links cmake-built {lib} ({label})"));
            }
        }
    }
    Ok(out)
}
impl Rule for ScanCmakeLinkedRust {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "scan_cmake_linked_rust"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        scan(&corpus.root)
    }
    fn source_sets(&self) -> &'static [crate::SourceSet] {
        &[]
    }
}

#[derive(Clone, Debug)]
struct Link {
    output: String,
    kind: String,
    entry: String,
    subsystem: Option<String>,
    image_base: Option<u64>,
    stack: Option<u64>,
    inputs: Vec<String>,
    libs: Vec<String>,
}
fn link_edge(repo: &Path, arch: &str, target: &str) -> Result<Link, String> {
    let ninja = read(&repo.join(format!("build-{arch}/build.ninja")))?;
    let pat = re(&format!(
        r"(?m)^build (\S+): (C_\w+?)_LINKER__{}_(?:Debug|Release|RelWithDebInfo|MinSizeRel)( .*)?\n((?:  .*\n)+)",
        regex::escape(target)
    ));
    let caps = pat.captures(&ninja).ok_or_else(|| {
        format!("{target}: no link edge in build-{arch}/build.ninja (a static lib?)")
    })?;
    let edge = caps.get(0).unwrap().as_str();
    let vars = &caps[4];
    let flags = re(r"(?m)^  LINK_FLAGS = (.*)$")
        .captures(vars)
        .map(|c| c[1].to_owned())
        .unwrap_or_default();
    let libs = re(r"(?m)^  LINK_LIBRARIES = (.*)$")
        .captures(vars)
        .map(|c| c[1].split_whitespace().map(str::to_owned).collect())
        .unwrap_or_default();
    let entry = re(r"-entry,(\S+)")
        .captures(&flags)
        .or_else(|| re(r"-Wl,-e,(\S+)").captures(&flags))
        .map(|c| c[1].to_owned())
        .unwrap_or_default();
    let subsystem = re(r"--subsystem,([\w:.]+)")
        .captures_iter(&flags)
        .last()
        .map(|c| c[1].to_owned());
    let image_base = re(r"--image-base,(0x[0-9a-fA-F]+)")
        .captures(&flags)
        .and_then(|c| u64::from_str_radix(c[1].trim_start_matches("0x"), 16).ok());
    let stack = re(r"--stack,(\d+)")
        .captures(&flags)
        .and_then(|c| c[1].parse().ok());
    if flags.contains("--delayload") {
        return Err(format!(
            "{target}: lld --delayload is not modelled (add_delay_importlibs is)"
        ));
    }
    let first = edge.lines().next().unwrap_or("");
    let defs = re(r"(\S+\.def)\b")
        .find_iter(first)
        .map(|m| m.as_str().to_owned())
        .collect::<Vec<_>>();
    let ins = caps.get(3).map(|m| m.as_str()).unwrap_or("");
    let mut inputs = ins
        .split('|')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for d in defs {
        if !inputs.contains(&d) {
            inputs.push(d)
        }
    }
    Ok(Link {
        output: caps[1].to_owned(),
        kind: caps[2].to_owned(),
        entry,
        subsystem,
        image_base,
        stack,
        inputs,
        libs,
    })
}
fn strip_comments(s: &str) -> String {
    re(r"(?s)/\*.*?\*/|//[^\n]*")
        .replace_all(s, "")
        .into_owned()
}
fn placeholder(path: &Path) -> Result<bool, String> {
    let src = read(path)?;
    let mut body = re(r"(?s)/\*.*?\*/|//[^\n]*")
        .replace_all(&src, "")
        .into_owned();
    body = re(r"typedef\s+int\s+\w+\s*;")
        .replace_all(&body, "")
        .into_owned();
    Ok(body.trim().is_empty())
}
fn source_list(text: &str) -> Vec<String> {
    let mut out = vec![];
    for (cmd, a) in cmake_statements(text) {
        if cmd == "list"
            && a.get(0).is_some_and(|x| x == "APPEND")
            && a.get(1).is_some_and(|x| x == "SOURCE")
        {
            out.extend(a.into_iter().skip(2));
        }
    }
    out
}
fn inlinable_source(text: &str, dir: &Path) -> Option<Vec<String>> {
    let src = source_list(text);
    if text.contains("PCH_SKIP_SOURCE") || src.is_empty() {
        return None;
    }
    for x in &src {
        if x.ends_with(".h") || x.ends_with(".hpp") {
            continue;
        }
        let p = dir.join(x);
        if !x.ends_with(".c") || !p.exists() || !placeholder(&p).ok()? {
            return None;
        }
    }
    Some(src)
}
#[derive(Default)]
struct Facts {
    unknown: Vec<String>,
    destinations: Vec<String>,
    body: Option<(String, Vec<String>, bool)>,
    body_var: String,
    module_type: Option<String>,
    unicode: bool,
    ssp: bool,
    name_on_cd: Option<String>,
    no_cab: bool,
    custom_archive: Option<String>,
}
fn cmake_facts(path: &Path, target: &str, libs: &[String]) -> Result<Facts, String> {
    let text = read(path)?;
    let mut f = Facts::default();
    let mut variants: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let linked: BTreeSet<String> = libs
        .iter()
        .map(|x| x.rsplit('/').next().unwrap_or(x).to_owned())
        .collect();
    let mut defined = false;
    const REPLACED: &[&str] = &[
        "add_executable",
        "add_library",
        "set_module_type",
        "add_importlibs",
        "add_delay_importlibs",
        "target_link_libraries",
        "add_pch",
        "add_cd_file",
        "add_dependencies",
    ];
    const KEPT: &[&str] = &["project", "add_driver_inf"];
    const TRANSFERRED: &[&str] = &[
        "target_compile_definitions",
        "target_include_directories",
        "target_compile_options",
        "set_wine_module_fixme",
        "set_wine_module",
    ];
    const NAME_PROPS: &[&str] = &[
        "SUFFIX",
        "OUTPUT_NAME",
        "PREFIX",
        "ENABLE_EXPORTS",
        "DEFINE_SYMBOL",
    ];
    for (cmd, a, raw) in cmake_statements_raw(&text) {
        if cmd == "rust_staticlib_variant" && a.len() >= 4 {
            variants.insert(a[0].clone(), a.clone());
        }
        if ["add_executable", "add_library"].contains(&cmd.as_str())
            && a.first().is_some_and(|x| x == target)
        {
            defined = true;
        }
        if a.is_empty()
            || (a[0] != target && !(cmd == "add_cd_file" && a.contains(&target.to_string())))
        {
            if raw.contains("${SOURCE}") || raw.contains("${PCH_SKIP_SOURCE}") {
                if inlinable_source(&text, path.parent().unwrap_or(Path::new("."))).is_none() {
                    f.unknown.push(format!(
                        "{cmd} {} shares ${{SOURCE}}",
                        a.first().map(String::as_str).unwrap_or("")
                    ));
                }
            }
            continue;
        }
        if cmd == "set_module_type" && a.iter().any(|x| x == "UNICODE") {
            f.unicode = true;
        }
        if cmd == "set_module_type" && a.len() > 1 {
            f.module_type = Some(a[1].clone());
        }
        if cmd == "add_cd_file" {
            let start = a
                .iter()
                .position(|x| x == "FOR")
                .map(|i| i + 1)
                .unwrap_or(a.len());
            let for_all = a[start..].iter().any(|x| x == "all" || x == "livecd");
            if for_all {
                f.no_cab |= a.iter().any(|x| x == "NO_CAB");
                if let Some(i) = a.iter().position(|x| x == "DESTINATION") {
                    if let Some(d) = a.get(i + 1) {
                        if !f.destinations.contains(d) {
                            f.destinations.push(d.clone());
                        }
                    }
                }
                if let Some(i) = a.iter().position(|x| x == "NAME_ON_CD") {
                    f.name_on_cd = a.get(i + 1).cloned();
                }
            } else if let Some(i) = a.iter().position(|x| x == "TARGET") {
                if a.get(i + 1).is_some_and(|x| x != target) {
                    f.unknown.push(format!("add_cd_file names another target"));
                }
            }
        } else if TRANSFERRED.contains(&cmd.as_str()) {
        } else if cmd == "set_target_properties"
            && a.len() >= 4
            && a[1] == "PROPERTIES"
            && a.len() % 2 == 0
            && a[2..]
                .iter()
                .step_by(2)
                .all(|k| NAME_PROPS.contains(&k.as_str()))
        {
        } else if cmd == "target_link_options" && subsystem_only(&a) {
        } else if cmd == "target_sources"
            && a.len() == 3
            && a[1] == "PRIVATE"
            && re(r"^\$<TARGET_OBJECTS:gcc_ssp_\w+>$").is_match(&a[2])
        {
            f.ssp = true;
        } else if !REPLACED.contains(&cmd.as_str()) && !KEPT.contains(&cmd.as_str()) {
            f.unknown.push(cmd);
        }
    }
    if !defined {
        f.unknown.push("target made in a loop or function".into());
    }
    let mut named: Vec<_> = variants
        .keys()
        .filter(|n| *n == &format!("{target}_rust") || n.strip_suffix("_rust") == Some(target))
        .cloned()
        .collect();
    if named.is_empty() {
        named = variants
            .keys()
            .filter(|n| linked.contains(&format!("lib{n}.a")))
            .cloned()
            .collect();
    }
    if named.len() == 1 {
        let name = &named[0];
        let a = &variants[name];
        let feats = a[2]
            .trim_matches('"')
            .replace(';', " ")
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        f.body = Some((a[1].clone(), feats, a[3].eq_ignore_ascii_case("TRUE")));
        f.body_var = name.clone();
        if a.len() > 4 && body_env(&text, &a[4..]).is_none() {
            f.unknown
                .push(format!("rust_staticlib_variant extras {:?}", &a[4..]));
        }
    } else if named.len() > 1 {
        f.unknown.push(format!("several bodies {named:?}"));
    } else if let Some((crate_name, archive)) = custom_body(&text) {
        f.body = Some((crate_name, Vec::new(), true));
        f.custom_archive = Some(archive);
    }
    Ok(f)
}

fn subsystem_only(a: &[String]) -> bool {
    let opts: Vec<_> = a
        .iter()
        .skip(1)
        .filter(|x| x.as_str() != "PRIVATE" && x.as_str() != "PUBLIC")
        .collect();
    !opts.is_empty()
        && opts
            .iter()
            .all(|x| re(r"^-Wl,--subsystem,[\w:.]+$").is_match(x.trim_matches('"')))
}

fn body_env(text: &str, extras: &[String]) -> Option<()> {
    if extras.len() != 1 {
        return None;
    }
    let m = re(r"^\$\{(\w+)\}$").captures(&extras[0])?;
    let var = regex::escape(&m[1]);
    let block = re(&format!(
        r##"(?s)set\({var}\)\n\s*if\(ARCH STREQUAL "(\w+)"\)\n\s*set\({var} ENV "RUSTFLAGS=([^"]*)"\)\nendif\(\)\n"##
    ));
    let c = block.captures(text)?;
    if !["i386", "amd64", "arm64"].contains(&&c[1]) {
        return None;
    }
    if re(&format!(r"\b{var}\b")).find_iter(text).count() != 3 {
        return None;
    }
    Some(())
}

fn custom_body(text: &str) -> Option<(String, String)> {
    let cmds: Vec<_> = cmake_statements_raw(text)
        .into_iter()
        .filter(|(cmd, _, raw)| cmd == "add_custom_command" && re(r"\bcargo build\b").is_match(raw))
        .collect();
    if cmds.len() != 1 {
        return None;
    }
    let raw = re(r"(?m)#[^\n]*").replace_all(&cmds[0].2, "");
    let pat = re(
        r##"(?s)^\s*OUTPUT \$\{(\w+)\}\s+COMMAND \$\{CMAKE_COMMAND\} -E env\s+CARGO_TARGET_DIR=\$\{(\w+)\}\s+RUSTFLAGS=\$\{(\w+)\}\s+cargo build --release --target \$\{(\w+)\}\s+-p ([\w-]+)\s+--manifest-path \$\{(\w+)\}/Cargo\.toml\s+COMMAND \$\{CMAKE_COMMAND\} -E copy\s+\$\{(\w+)\}/\$\{(\w+)\}/release/(lib\w+\.a)\s+\$\{(\w+)\}\s+DEPENDS \$\{(\w+)\}\s+\$\{(\w+)\}/([\w-]+)/Cargo\.toml\s+WORKING_DIRECTORY \$\{(\w+)\}\s+COMMENT "[^"]*"\s+VERBATIM\s*$"##,
    );
    let c = pat.captures(&raw)?;
    // The Python spells these as backreferences (\2 \4 \1 \6 \5 \6), which the
    // regex crate does not support; compare the repeated groups here instead.
    if [(7, 2), (8, 4), (10, 1), (12, 6), (13, 5), (14, 6)]
        .iter()
        .any(|&(again, first)| c[again] != c[first])
    {
        return None;
    }
    let archive = c[9].to_owned();
    let arch_values = |var: &str| -> Option<BTreeMap<String, String>> {
        let mut arch: Option<String> = None;
        let mut values = BTreeMap::new();
        for (cmd, args) in cmake_statements(text) {
            if cmd == "if" || cmd == "elseif" {
                arch = re(r##"^ARCH STREQUAL "?(\w+)"?$"##)
                    .captures(&args.join(" "))
                    .map(|m| m[1].to_owned())
                    .or_else(|| Some("?".into()));
            } else if cmd == "else" {
                arch = Some("?".into());
            } else if cmd == "endif" {
                arch = None;
            } else if cmd == "set" && args.first().is_some_and(|x| x == var) {
                if args.len() != 2 {
                    return None;
                }
                let a = arch.as_deref()?;
                if a == "?"
                    || values
                        .insert(a.to_owned(), args[1].trim_matches('"').to_owned())
                        .is_some()
                {
                    return None;
                }
            }
        }
        Some(values)
    };
    let expected = BTreeMap::from([
        ("i386".to_owned(), "i686-pc-windows-gnu".to_owned()),
        ("amd64".to_owned(), "x86_64-pc-windows-gnu".to_owned()),
        ("arm64".to_owned(), "aarch64-pc-windows-gnullvm".to_owned()),
    ]);
    if arch_values(&c[4])? != expected
        || arch_values(&c[3])?.keys().cloned().collect::<BTreeSet<_>>()
            != expected.keys().cloned().collect()
    {
        return None;
    }
    let lib = regex::escape(&c[1]);
    let archive_re = re(&format!(
        r##"set\({lib} "\$\{{CMAKE_CURRENT_BINARY_DIR\}}/{}"\)"##,
        regex::escape(&archive)
    ));
    let custom_target = re(&format!(r"add_custom_target\(\w+ DEPENDS \$\{{{lib}\}}\)"));
    if !archive_re.is_match(text) || !custom_target.is_match(text) {
        return None;
    }
    Some((c[5].to_owned(), archive))
}

fn engine_name(
    lib: &str,
    index: &BTreeMap<String, Option<(String, Vec<String>, Vec<String>)>>,
) -> Option<String> {
    if let Some(n) = engine_table()
        .expect("engine catalog parsed by model")
        .get(lib)
    {
        return Some(n.clone());
    }
    let c = LIB_ARCHIVE.captures(lib)?;
    let Some(Some((kind, args, _deps))) = index.get(&c[1]) else {
        return None;
    };
    if kind == "rust_staticlib_variant" {
        if args.len() < 4 || args[4..].iter().any(|x| x == "ENV") {
            return None;
        }
        let feats = args[2]
            .trim_matches('"')
            .split(|c: char| c == ',' || c == ' ' || c == ';')
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .join(",");
        return Some(format!(
            "engine:crate:{}:{}:{}",
            args[1],
            feats,
            args[3].to_ascii_uppercase()
        ));
    }
    if args.get(1).is_some_and(|x| {
        [
            "SHARED",
            "MODULE",
            "OBJECT",
            "INTERFACE",
            "IMPORTED",
            "ALIAS",
        ]
        .contains(&x.as_str())
    }) {
        return None;
    }
    Some(format!("engine:c:{}", args[0]))
}
fn hand_engine_lib(lib: &str) -> bool {
    engine_table()
        .expect("engine catalog parsed by model")
        .contains_key(lib)
}
fn placeholder_archive(repo: &Path, arch: &str, lib: &str) -> Result<bool, String> {
    let Some(c) = re(r"(?:.*/)?lib(\w+)\.a$").captures(lib) else {
        return Ok(false);
    };
    let db = repo.join(format!("build-{arch}/compile_commands.json"));
    let value: serde_json::Value =
        serde_json::from_str(&read(&db)?).map_err(|e| format!("{}: {e}", db.display()))?;
    let marker = format!("CMakeFiles/{}.dir/", &c[1]);
    let mut files = Vec::new();
    if let Some(rows) = value.as_array() {
        for row in rows {
            let cmd = row.get("command").and_then(|x| x.as_str()).unwrap_or("");
            if cmd.contains(&marker) {
                if let Some(file) = row.get("file").and_then(|x| x.as_str()) {
                    files.push(file.to_owned())
                }
            }
        }
    }
    if files.is_empty() {
        return Ok(false);
    }
    for f in files {
        if !f.ends_with(".c") {
            return Ok(false);
        }
        let p = PathBuf::from(f.replacen("/tmp/vibeos/", &format!("{}/", repo.display()), 1));
        if !placeholder(&p)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn cmake_spec_index(repo: &Path) -> Result<BTreeMap<String, Option<(String, String)>>, String> {
    let mut files = Vec::new();
    walk(repo, &mut files, &|p| {
        p.file_name().is_some_and(|x| x == "CMakeLists.txt")
    })?;
    let spec = re(r"(?is)spec2def\(\s*(\S+?)\s+(\S+?\.spec)\b([^)]*)\)");
    let mut cands: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for p in files {
        let rel = p
            .strip_prefix(repo)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        if rel.starts_with("build-")
            || rel.starts_with("vendor/")
            || rel.starts_with("modules/rostests")
            || rel.starts_with('.')
        {
            continue;
        }
        let text = read(&p)?;
        for c in spec.captures_iter(&text) {
            if !c[3].contains("ADD_IMPORTLIB") {
                continue;
            }
            let spec_rel = p
                .parent()
                .unwrap_or(repo)
                .join(&c[2])
                .strip_prefix(repo)
                .unwrap_or(Path::new(&c[2]))
                .to_string_lossy()
                .replace('\\', "/");
            let dll = &c[1];
            let base = dll
                .rsplit_once('.')
                .map(|x| x.0)
                .unwrap_or(dll)
                .to_lowercase();
            let row = (spec_rel, dll.to_owned());
            let v = cands.entry(base).or_default();
            if !v.contains(&row) {
                v.push(row)
            }
        }
    }
    let mut built: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    let pat = re(
        r"spec2def [^&|\n]*?-n=(\S+) [^&|\n]*?--implib -d=\S*/lib(\w+)_implib\.def /tmp/vibeos/(\S+\.spec)",
    );
    for a in ["i386", "amd64", "arm64"] {
        if let Ok(text) = read(&repo.join(format!("build-{a}/build.ninja"))) {
            for c in pat.captures_iter(&text) {
                let base = c[2].to_lowercase();
                let spec = posix_norm(&c[3]);
                built
                    .entry(base)
                    .or_default()
                    .insert((spec, c[1].to_owned()));
            }
        }
    }
    let keys: BTreeSet<_> = cands.keys().chain(built.keys()).cloned().collect();
    let mut out = BTreeMap::new();
    for k in keys {
        let x = if let Some(rows) = built.get(&k) {
            if rows.len() == 1 {
                rows.iter().next().cloned()
            } else {
                None
            }
        } else {
            let rows = &cands[&k];
            if rows.len() == 1 {
                Some(rows[0].clone())
            } else {
                None
            }
        };
        out.insert(k, x);
    }
    Ok(out)
}
fn posix_norm(s: &str) -> String {
    let absolute = s.starts_with('/');
    let mut stack: Vec<&str> = Vec::new();
    for part in s.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if stack.last().is_some_and(|x| *x != "..") {
                    stack.pop();
                } else if !absolute {
                    stack.push("..");
                }
            }
            x => stack.push(x),
        }
    }
    let joined = stack.join("/");
    if absolute {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".into()
    } else {
        joined
    }
}

fn unique_common(values: &[Vec<String>]) -> BTreeSet<String> {
    let mut s: BTreeSet<String> = values
        .first()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();
    for v in values.iter().skip(1) {
        let x: BTreeSet<_> = v.iter().cloned().collect();
        s = s.intersection(&x).cloned().collect();
    }
    s
}
fn model_convertible(
    repo: &Path,
    target: &str,
    spec_cache: &mut Option<BTreeMap<String, Option<(String, String)>>>,
    library_cache: &mut Option<BTreeMap<String, Option<(String, Vec<String>, Vec<String>)>>>,
) -> Result<(), String> {
    engine_table()?;
    engine_deps_table()?;
    let mut links = BTreeMap::new();
    for a in ["i386", "amd64", "arm64"] {
        links.insert(a, link_edge(repo, a, target)?);
    }
    let first = &links["amd64"];
    let cmake_dir = repo.join(Path::new(&first.output).parent().unwrap_or(Path::new("")));
    let cmake = cmake_dir.join("CMakeLists.txt");
    if !cmake.exists() {
        return Err(format!(
            "{target}: no {}",
            cmake.strip_prefix(repo).unwrap_or(&cmake).display()
        ));
    }
    let facts = cmake_facts(&cmake, target, &first.libs)?;
    if !facts.unknown.is_empty() {
        return Err(format!("{target}: unmodelled statements on target"));
    }
    if facts.body.is_none() {
        return Err(format!(
            "{target}: no rust_staticlib_variant names its body crate"
        ));
    }
    if facts.ssp {
        for a in ["i386", "amd64", "arm64"] {
            let cache = read(&repo.join(format!("build-{a}/CMakeCache.txt")))?;
            if re(r"(?m)^STACK_PROTECTOR:BOOL=(ON|TRUE|1)$").is_match(&cache) {
                return Err(format!("{target}: STACK_PROTECTOR is on"));
            }
        }
    }
    let native = first
        .subsystem
        .as_deref()
        .unwrap_or("")
        .starts_with("native");
    let module_kind = if first.kind == "C_EXECUTABLE" {
        if !["windows", "console", "native"]
            .iter()
            .any(|x| first.subsystem.as_deref().unwrap_or("").starts_with(x))
        {
            return Err(format!("{target}: subsystem exe is not modelled"));
        }
        "Win32Exe"
    } else if ["C_MODULE_LIBRARY", "C_SHARED_LIBRARY"].contains(&first.kind.as_str()) {
        if native {
            if !["kernelmodedriver", "kerneldll"]
                .contains(&facts.module_type.as_deref().unwrap_or(""))
            {
                return Err(format!("{target}: kernel module type is not modelled"));
            }
            "KernelDll"
        } else {
            "Win32Dll"
        }
    } else {
        return Err(format!("{target}: link kind {}", first.kind));
    };
    let body_var = &facts.body_var;
    let body_archive = facts
        .custom_archive
        .clone()
        .unwrap_or_else(|| format!("lib{body_var}.a"));
    if facts.custom_archive.is_some() && module_kind != "KernelDll" {
        return Err(format!("{target}: custom body outside kernel driver"));
    }
    if spec_cache.is_none() {
        *spec_cache = Some(cmake_spec_index(repo)?);
    }
    let specs = spec_cache.as_ref().unwrap();
    let mut all_imports: BTreeMap<
        &str,
        (
            Vec<String>,
            Vec<String>,
            Option<String>,
            Vec<String>,
            Vec<(String, bool)>,
        ),
    > = BTreeMap::new();
    for a in ["i386", "amd64", "arm64"] {
        let l = &links[a];
        let mut imports = Vec::<String>::new();
        let mut delayed = Vec::<String>::new();
        let mut static_libs = Vec::<String>::new();
        let mut msvcrtex = None;
        let mut order = Vec::<(String, bool)>::new();
        let mut sequence = Vec::<String>::new();
        let mut after_body = false;
        for lib in &l.libs {
            let name = lib.rsplit('/').next().unwrap_or(lib);
            if name == body_archive {
                after_body = true;
                if !sequence.contains(&"body".to_owned()) {
                    sequence.push("body".into());
                }
                continue;
            }
            if lib.contains("libclang_rt") {
                continue;
            }
            if let Some(m) = re(r"^libmsvcrtex_(\w+)\.a$").captures(name) {
                msvcrtex = Some(match &m[1] {
                    "wexe" => "ExeWide".to_owned(),
                    "exe" => "ExeAnsi".to_owned(),
                    "dll" => "Dll".to_owned(),
                    _ => continue,
                });
                continue;
            }
            if let Some(m) = re(r"CMakeFiles/lib(\w+)_delayed\.dir/lib(\w+)_delayed\.a$")
                .captures(lib)
                .filter(|m| m[1] == m[2])
            {
                let base = m[1].to_lowercase();
                if !specs.get(&base).is_some_and(Option::is_some) {
                    return Err(format!(
                        "{target}: delay import {base}: no single spec2def --implib"
                    ));
                }
                if !delayed.contains(&base) {
                    delayed.push(base.clone());
                    order.push((base.clone(), true));
                }
                if sequence.contains(&format!("import:{base}")) {
                    return Err(format!("{target}: {base} both delay and regular import"));
                }
                sequence.push(format!("import:{base}"));
                continue;
            }
            if let Some(m) = re(r"CMakeFiles/lib(\w+)\.dir/lib(\w+)\.a$")
                .captures(lib)
                .filter(|m| m[1] == m[2])
            {
                let base = m[1].to_lowercase();
                if !specs.get(&base).is_some_and(Option::is_some) {
                    return Err(format!(
                        "{target}: import lib {base}: no single spec2def --implib"
                    ));
                }
                if !imports.contains(&base) {
                    imports.push(base.clone());
                    order.push((base.clone(), false));
                    sequence.push(format!("import:{base}"));
                }
                continue;
            }
            if !hand_engine_lib(lib) && placeholder_archive(repo, a, lib)? {
                continue;
            };
            let val = if hand_engine_lib(lib) {
                engine_name(lib, &BTreeMap::new()).unwrap_or_else(|| lib.clone())
            } else {
                if library_cache.is_none() {
                    *library_cache = Some(library_index(repo)?);
                }
                engine_name(lib, library_cache.as_ref().unwrap()).unwrap_or_else(|| lib.clone())
            };
            if !static_libs.contains(&val) {
                static_libs.push(val.clone());
                sequence.push(val)
            }
        }
        if !after_body {
            return Err(format!("{target}: {a} link has no {body_archive}"));
        }
        if !delayed.is_empty()
            && !static_libs
                .iter()
                .any(|x| re(r"/libdelayimp(_rust)?\.a$|^engine:delayimp(_rust)?$").is_match(x))
        {
            return Err(format!("{target}: delay imports without delayimp runtime"));
        }
        all_imports.insert(a, (imports, static_libs, msvcrtex, delayed, order));
    }
    let first_imports = &all_imports["amd64"];
    for a in ["i386", "arm64"] {
        let v = &all_imports[a];
        if v.0 != first_imports.0
            || v.2 != first_imports.2
            || v.3 != first_imports.3
            || v.4 != first_imports.4
        {
            return Err(format!("{target}: imports or CRT differ by arch"));
        }
    }
    let input_lists: Vec<Vec<String>> = ["i386", "amd64", "arm64"]
        .iter()
        .map(|a| {
            links[*a]
                .inputs
                .iter()
                .map(|x| x.rsplit('/').next().unwrap_or(x).to_owned())
                .collect()
        })
        .collect();
    let common = unique_common(&input_lists);
    for (idx, a) in ["i386", "amd64", "arm64"].iter().enumerate() {
        for obj in &links[*a].inputs {
            let nm = obj.rsplit('/').next().unwrap_or(obj);
            if common.contains(nm) {
                continue;
            }
            let src = cmake_dir.join(nm.strip_suffix(".obj").unwrap_or(nm));
            if !re(r"\.(c|s|S)\.obj$").is_match(nm) || !src.exists() {
                return Err(format!("{target}: link inputs differ by arch"));
            }
            let _ = idx;
        }
    }
    for obj in &first.inputs {
        let nm = obj.rsplit('/').next().unwrap_or(obj);
        if !common.contains(nm) {
            continue;
        }
        if nm.ends_with(".rc.res") {
            rc_command(repo, "amd64", obj)?;
        } else if nm.ends_with("_stubs.rs.obj") || nm.ends_with("_stubs.c.obj") {
        } else if nm.ends_with(".def") {
            if !re(r"spec2def\(\s*(\S+)\s+(\S+\.spec)").is_match(&read(&cmake)?) {
                return Err(format!("{target}: a .def without a spec2def()"));
            }
        } else if nm.ends_with(".c.obj") {
            let mut src = cmake_dir.join(nm.strip_suffix(".obj").unwrap());
            if let Some(m) = re(r"\.dir/((?:__/)+.*)\.obj$").captures(obj) {
                src = cmake_dir.join(m[1].replace("__/", "../"));
            }
            if !src.exists() {
                let rel = src
                    .strip_prefix(&cmake_dir)
                    .unwrap_or(Path::new(nm.strip_suffix(".obj").unwrap()));
                if !["i386", "amd64", "arm64"].iter().all(|a| {
                    repo.join(format!("build-{a}"))
                        .join(&cmake_dir.strip_prefix(repo).unwrap_or(Path::new("")))
                        .join(rel)
                        .exists()
                }) {
                    return Err(format!(
                        "{target}: compiled C {nm} found in no source or build dir"
                    ));
                }
            } else if nm == "rust_cohab_stub.c.obj" && placeholder(&src)? {
            }
        } else {
            return Err(format!("{target}: unmodelled link input {nm}"));
        }
    }
    if module_kind == "Win32Exe" && first_imports.2.is_none() && !native {
        return Err(format!("{target}: exe without msvcrtex CRT startup"));
    }
    let stacks: BTreeSet<_> = ["i386", "amd64", "arm64"]
        .iter()
        .map(|a| links[*a].stack)
        .collect();
    if stacks.len() > 1 {
        return Err(format!("{target}: stack reserve differs by arch"));
    }
    let mut shipped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for a in ["i386", "amd64", "arm64"] {
        let output = &links[a].output;
        let dirs = cd_dirs(repo, a, output)?;
        if facts.destinations.is_empty() {
            if !dirs.is_empty() {
                return Err(format!("{target}: on live CD with no FOR all/livecd"));
            }
            shipped.insert(a.into(), Vec::new());
        } else {
            if dirs.is_empty() {
                return Err(format!("{target}: CD destinations differ by arch"));
            }
            let patterns: Vec<_> = facts
                .destinations
                .iter()
                .map(|d| {
                    let escaped = regex::escape(d);
                    let inner = re(r"\\\$\\\{\w+\\\}")
                        .replace_all(&escaped, "[^/]+")
                        .into_owned();
                    re(&format!("^(?:{inner})$"))
                })
                .collect();
            let mut found = Vec::new();
            for dir in &dirs {
                let hits: Vec<_> = facts
                    .destinations
                    .iter()
                    .zip(&patterns)
                    .filter_map(|(d, p)| p.is_match(dir).then_some(d.clone()))
                    .collect();
                if hits.len() != 1 {
                    return Err(format!("{target}: CD destinations differ by arch"));
                }
                if !found.contains(&hits[0]) {
                    found.push(hits[0].clone())
                }
            }
            found.sort_by_key(|x| {
                facts
                    .destinations
                    .iter()
                    .position(|d| d == x)
                    .unwrap_or(usize::MAX)
            });
            shipped.insert(a.into(), found);
        }
    }
    if ["i386", "amd64", "arm64"]
        .iter()
        .any(|a| shipped[*a] != facts.destinations)
    {
        let used: BTreeSet<_> = shipped.values().flatten().cloned().collect();
        if used != facts.destinations.iter().cloned().collect() {
            return Err(format!(
                "{target}: CD destinations leave some declared destinations unused"
            ));
        }
    }
    if links
        .values()
        .map(|l| l.subsystem.as_deref().unwrap_or(""))
        .collect::<BTreeSet<_>>()
        .len()
        > 1
        && links.values().any(|l| l.subsystem.is_none())
    {
        return Err(format!("{target}: subsystem set on some arches only"));
    }
    let mut panics = BTreeMap::new();
    for a in ["i386", "amd64", "arm64"] {
        let l = &links[a];
        let path = repo.join(format!("build-{a}")).join(&l.output);
        if !path.is_file() {
            return Err(format!(
                "{target}: {}: build the module with cmake first",
                path.display()
            ));
        }
        let info = pe_imports_exports(&path).map_err(|e| {
            if e.starts_with("PE_IO:") {
                e.trim_start_matches("PE_IO:").to_owned()
            } else {
                format!("PE_FATAL:{e}")
            }
        })?;
        let panic = info.0.iter().any(|i| i.ends_with("!RtlWalkFrameChain"));
        panics.insert(a, panic);
        let bound: BTreeSet<_> = info
            .1
            .intersection(
                &info
                    .0
                    .iter()
                    .map(|x| {
                        x.split_once('!')
                            .map(|p| p.1.to_owned())
                            .unwrap_or_default()
                    })
                    .collect(),
            )
            .cloned()
            .collect();
        let allowed = target == "user32_vista"
            && bound
                .iter()
                .all(|x| ["GetDpiForSystem", "GetDpiForWindow"].contains(&x.as_str()));
        if !bound.is_empty() && !allowed {
            return Err(format!("{target}: exports resolve to import thunks"));
        }
    }
    if module_kind == "KernelDll"
        && (panics.values().collect::<BTreeSet<_>>().len() > 1
            || ["i386", "amd64", "arm64"]
                .iter()
                .any(|a| links[*a].image_base.is_none()))
    {
        return Err(format!("{target}: kernel link properties differ"));
    }
    Ok(())
}
fn rc_command(repo: &Path, arch: &str, obj: &str) -> Result<(String, Vec<String>), String> {
    let db = repo.join(format!("build-{arch}/compile_commands.json"));
    let v: serde_json::Value = serde_json::from_str(&read(&db)?).map_err(|e| e.to_string())?;
    if let Some(rows) = v.as_array() {
        for x in rows {
            let cmd = x.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let file = x.get("file").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.contains(obj) && file.ends_with(".rc") {
                let src = file.replacen("/tmp/vibeos/", "", 1);
                let defs = re(r"\s-D(\S+)")
                    .captures_iter(cmd)
                    .map(|c| c[1].to_owned())
                    .collect();
                return Ok((src, defs));
            }
        }
    }
    Err(format!(
        "no windres command for {obj} in build-{arch}/compile_commands.json"
    ))
}
fn cd_dirs(repo: &Path, arch: &str, output: &str) -> Result<BTreeSet<String>, String> {
    let boot = repo.join(format!("build-{arch}/boot"));
    let mut lists = Vec::new();
    for e in fs::read_dir(&boot).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        let n = p.file_name().unwrap_or_default().to_string_lossy();
        if n.starts_with("livecd.")
            && n.ends_with(".lst")
            && n != "livecd.cmake.lst"
            && n != "livecd.stripped.lst"
        {
            lists.push(p)
        }
    }
    if lists.len() != 1 {
        return Err(format!(
            "build-{arch}/boot: expected one livecd config list"
        ));
    }
    let want = format!("/tmp/vibeos/build-{arch}/{output}");
    let mut out = BTreeSet::new();
    for line in read(&lists[0])?.lines() {
        if line.split_once('=').is_some_and(|x| x.1 == want) {
            if let Some(dir) = line.split_once('=').unwrap().0.rsplit_once('/') {
                out.insert(dir.0.to_owned());
            }
        }
    }
    Ok(out)
}

fn u16le(b: &[u8], o: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        b.get(o..o + 2).ok_or("truncated PE")?.try_into().unwrap(),
    ))
}
fn u32le(b: &[u8], o: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        b.get(o..o + 4).ok_or("truncated PE")?.try_into().unwrap(),
    ))
}
fn pe_imports_exports(path: &Path) -> Result<(BTreeSet<String>, BTreeSet<String>), String> {
    let b = fs::read(path).map_err(|e| format!("PE_IO:{}: {e}", path.display()))?;
    if b.get(..2) != Some(b"MZ") {
        return Err(format!("{}: no MZ header", path.display()));
    }
    let pe = u32le(&b, 0x3c)? as usize;
    if b.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Err(format!("{}: no PE signature", path.display()));
    }
    let nsect = u16le(&b, pe + 6)? as usize;
    let opt_size = u16le(&b, pe + 20)? as usize;
    let opt = pe + 24;
    let magic = u16le(&b, opt)?;
    let pe64 = magic == 0x20b;
    let ndir = opt + if pe64 { 108 } else { 92 };
    let n = u32le(&b, ndir)? as usize;
    let dir = ndir + 4;
    let sect = opt + opt_size;
    let mut sections = Vec::new();
    for i in 0..nsect {
        let s = sect + i * 40;
        let va = u32le(&b, s + 12)?;
        let vs = u32le(&b, s + 8)?;
        let rawsize = u32le(&b, s + 16)?;
        let raw = u32le(&b, s + 20)?;
        sections.push((va, vs, rawsize, raw));
    }
    let rva_off = |rva: u32| -> Result<usize, String> {
        for (va, vs, rs, raw) in &sections {
            if *va <= rva && rva < va + (*vs).max(*rs) {
                return Ok((*raw + (rva - *va)) as usize);
            }
        }
        Err(format!("RVA {rva:#x} is in no section"))
    };
    let cstr = |rva: u32| -> Result<String, String> {
        let o = rva_off(rva)?;
        let end = b[o..]
            .iter()
            .position(|x| *x == 0)
            .map(|x| o + x)
            .ok_or("unterminated PE string")?;
        Ok(b[o..end].iter().map(|x| char::from(*x)).collect())
    };
    let mut imports = BTreeSet::new();
    let mut exports = BTreeSet::new();
    if n > 0 {
        let rva = u32le(&b, dir)?;
        let size = u32le(&b, dir + 4)?;
        if rva != 0 {
            let o = rva_off(rva)?;
            let base = u32le(&b, o + 16)?;
            let nf = u32le(&b, o + 20)? as usize;
            let nn = u32le(&b, o + 24)? as usize;
            let funcs = u32le(&b, o + 28)?;
            let names = u32le(&b, o + 32)?;
            let ords = u32le(&b, o + 36)?;
            let mut by_ord = BTreeMap::new();
            for i in 0..nn {
                let nr = u32le(&b, rva_off(names)? + i * 4)?;
                let oi = u16le(&b, rva_off(ords)? + i * 2)?;
                by_ord.insert(oi, cstr(nr)?);
            }
            for i in 0..nf {
                let fr = u32le(&b, rva_off(funcs)? + i * 4)?;
                if fr == 0 {
                    continue;
                }
                let _fwd = if rva <= fr && fr < rva.saturating_add(size) {
                    Some(cstr(fr)?)
                } else {
                    None
                };
                exports.insert(
                    by_ord
                        .get(&(i as u16))
                        .cloned()
                        .unwrap_or_else(|| "(noname)".to_owned()),
                );
            }
        }
    }
    let width = if pe64 { 8 } else { 4 };
    let ordflag = if pe64 { 1u64 << 63 } else { 1u64 << 31 };
    let mut thunks = |rva: u32, dll: &str, prefix: &str| -> Result<(), String> {
        let mut off = rva_off(rva)?;
        loop {
            let v = if pe64 {
                u64::from_le_bytes(
                    b.get(off..off + 8)
                        .ok_or("truncated thunk")?
                        .try_into()
                        .unwrap(),
                )
            } else {
                u32le(&b, off)? as u64
            };
            if v == 0 {
                break;
            }
            let sym = if v & ordflag != 0 {
                format!("#{}", v & 0xffff)
            } else {
                cstr((v as u32 & 0x7fff_ffff) + 2)?
            };
            imports.insert(format!("{prefix}{dll}!{sym}"));
            off += width;
        }
        Ok(())
    };
    if n > 1 {
        let rva = u32le(&b, dir + 8)?;
        if rva != 0 {
            let mut off = rva_off(rva)?;
            loop {
                let ilt = u32le(&b, off)?;
                let name = u32le(&b, off + 12)?;
                let iat = u32le(&b, off + 16)?;
                if name == 0 {
                    break;
                }
                let dll = cstr(name)?.to_lowercase();
                thunks(if ilt == 0 { iat } else { ilt }, &dll, "")?;
                off += 20;
            }
        }
    }
    if n > 13 {
        let rva = u32le(&b, dir + 13 * 8)?;
        if rva != 0 {
            let mut off = rva_off(rva)?;
            loop {
                let name = u32le(&b, off + 4)?;
                let int = u32le(&b, off + 16)?;
                if name == 0 {
                    break;
                }
                let dll = cstr(name)?.to_lowercase();
                thunks(int, &dll, "delay ")?;
                off += 32;
            }
        }
    }
    Ok((imports, exports))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_body_matches_without_backreferences() {
        let body = |copy_dir: &str| {
            format!(
                r#"if(ARCH STREQUAL "i386")
    set(T i686-pc-windows-gnu)
    set(RF "-Ctarget-feature=+crt-static")
elseif(ARCH STREQUAL "amd64")
    set(T x86_64-pc-windows-gnu)
    set(RF "")
elseif(ARCH STREQUAL "arm64")
    set(T aarch64-pc-windows-gnullvm)
    set(RF "")
endif()
set(LIB "${{CMAKE_CURRENT_BINARY_DIR}}/libfoo_rust.a")
add_custom_command(
    OUTPUT ${{LIB}}
    COMMAND ${{CMAKE_COMMAND}} -E env CARGO_TARGET_DIR=${{TD}} RUSTFLAGS=${{RF}}
        cargo build --release --target ${{T}} -p foo-rust --manifest-path ${{WS}}/Cargo.toml
    COMMAND ${{CMAKE_COMMAND}} -E copy ${{{copy_dir}}}/${{T}}/release/libfoo_rust.a ${{LIB}}
    DEPENDS ${{SRCS}} ${{WS}}/foo-rust/Cargo.toml
    WORKING_DIRECTORY ${{WS}}
    COMMENT "Building foo-rust"
    VERBATIM)
add_custom_target(rust_foo DEPENDS ${{LIB}})
"#
            )
        };
        assert_eq!(
            custom_body(&body("TD")),
            Some(("foo-rust".to_owned(), "libfoo_rust.a".to_owned()))
        );
        // The copy must read the same CARGO_TARGET_DIR the build wrote.
        assert_eq!(custom_body(&body("OTHER")), None);
    }
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p = std::env::temp_dir()
                .join(format!("vibeos-cmake-model-{}-{id}", std::process::id()));
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn put(&self, rel: &str, text: &str) {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn make_pe() -> Vec<u8> {
        let mut b = vec![0u8; 0x200];
        b[0..2].copy_from_slice(b"MZ");
        b[0x3c..0x40].copy_from_slice(&(0x80u32).to_le_bytes());
        b[0x80..0x84].copy_from_slice(b"PE\0\0");
        b[0x84..0x86].copy_from_slice(&(0x8664u16).to_le_bytes());
        b[0x86..0x88].copy_from_slice(&(0u16).to_le_bytes());
        b[0x94..0x96].copy_from_slice(&(0xf0u16).to_le_bytes());
        let opt = 0x98;
        b[opt..opt + 2].copy_from_slice(&(0x20bu16).to_le_bytes());
        b[opt + 108..opt + 112].copy_from_slice(&(0u32).to_le_bytes());
        b
    }
    fn valid_fixture(extra: &str) -> Temp {
        let t = Temp::new();
        t.put("foo/CMakeLists.txt",&format!("add_library(foo MODULE rust_cohab_stub.c)\nrust_staticlib_variant(foo_rust foo-rust \"\" TRUE)\nadd_cd_file(TARGET foo DESTINATION /system32 FOR all)\n{extra}"));
        t.put("foo/rust_cohab_stub.c", "/* placeholder */\n");
        for arch in ["i386", "amd64", "arm64"] {
            t.put(&format!("build-{arch}/build.ninja"),"build foo/foo.dll: C_MODULE_LIBRARY_LINKER__foo_Debug foo/CMakeFiles/foo.dir/rust_cohab_stub.c.obj\n  LINK_FLAGS = --subsystem,windows\n  LINK_LIBRARIES = sdk/lib/rust/foo_rust/libfoo_rust.a\n");
            t.put(
                &format!("build-{arch}/boot/livecd.Release.lst"),
                &format!("/system32/foo.dll=/tmp/vibeos/build-{arch}/foo/foo.dll\n"),
            );
            let p = t.0.join(format!("build-{arch}/foo/foo.dll"));
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, make_pe()).unwrap();
        }
        t
    }
    fn model(t: &Temp) -> Result<(), String> {
        let mut specs = None;
        let mut libraries = None;
        model_convertible(&t.0, "foo", &mut specs, &mut libraries)
    }
    #[test]
    fn model_accepts_a_fully_modeled_three_arch_module() {
        let t = valid_fixture("");
        assert!(model(&t).is_ok(), "{:?}", model(&t));
    }
    #[test]
    fn model_refuses_an_unmodeled_cmake_statement() {
        let t = valid_fixture("unknown_target_property(foo X)\n");
        assert!(model(&t).is_err());
    }
    #[test]
    fn model_refuses_a_missing_arch_link_graph() {
        let t = valid_fixture("");
        fs::remove_file(t.0.join("build-arm64/build.ninja")).unwrap();
        assert!(model(&t).is_err());
    }
    #[test]
    fn scanner_emits_the_exact_convertible_finding_line() {
        let t = valid_fixture("");
        t.put("sdk/lib/rust/foo-cargo/build.rs", "fn main() {}\n");
        assert_eq!(
            scan(&t.0).unwrap(),
            vec!["foo: fully-Rust module still linked by cmake (convertible)"]
        );
    }
    #[test]
    fn scanner_emits_the_exact_refused_finding_line() {
        let t = valid_fixture("unknown_target_property(foo X)\n");
        t.put("sdk/lib/rust/foo-cargo/build.rs", "fn main() {}\n");
        assert_eq!(
            scan(&t.0).unwrap(),
            vec!["foo: fully-Rust module still linked by cmake (refused)"]
        );
    }
    #[test]
    fn scanner_preserves_multiple_survey_edges_for_one_target() {
        let t = valid_fixture("");
        let amd64 = t.0.join("build-amd64/build.ninja");
        let mut text = fs::read_to_string(&amd64).unwrap();
        text.push_str("build foo/foo.dll: C_MODULE_LIBRARY_LINKER__foo_Release foo/CMakeFiles/foo.dir/rust_cohab_stub.c.obj\n  LINK_FLAGS = --subsystem,windows\n  LINK_LIBRARIES = sdk/lib/rust/foo_rust/libfoo_rust.a\n");
        fs::write(amd64, text).unwrap();
        assert_eq!(
            scan(&t.0).unwrap(),
            vec![
                "foo: fully-Rust module still linked by cmake (convertible)",
                "foo: fully-Rust module still linked by cmake (convertible)",
            ]
        );
    }
    #[test]
    fn scanner_uses_current_cmake_source_to_filter_stale_link_edges() {
        let t = Temp::new();
        t.put(
            "build-amd64/build.ninja",
            "build m/m.dll: C_SHARED_LIBRARY_LINKER__m_Debug rust_cohab_stub.c.obj\nbuild n/n.dll: C_SHARED_LIBRARY_LINKER__n_Debug rust_cohab_stub.c.obj\nbuild o/o.dll: C_SHARED_LIBRARY_LINKER__o_Debug rust_cohab_stub.c.obj\n",
        );
        t.put(
            "dll/m/CMakeLists.txt",
            "add_cargo_module(m m-cargo m.dll)\n",
        );
        t.put(
            "dll/n/CMakeLists.txt",
            "add_library(n MODULE rust_cohab_stub.c)\n",
        );
        t.put("dll/n/rust_cohab_stub.c", "/* placeholder */\n");
        t.put("dll/o/CMakeLists.txt", "add_library(o MODULE o.c font.c)\n");
        assert_eq!(
            scan(&t.0).unwrap(),
            vec!["n: fully-Rust module still linked by cmake (refused)"]
        );
    }
    #[test]
    fn engine_library_fixture_matches_archive_classifier() {
        let t = Temp::new();
        t.put(
            "sdk/lib/x/CMakeLists.txt",
            "add_library(example src.c)\nadd_dependencies(example psdk)\n",
        );
        let idx = library_index(&t.0).unwrap();
        assert_eq!(
            engine_name("sdk/lib/x/libexample.a", &idx).as_deref(),
            Some("engine:c:example")
        );
        assert!(engine_name("sdk/lib/x/libmissing.a", &idx).is_none());
    }
    #[test]
    fn engine_tables_are_strictly_loaded_from_converter_source() {
        let libs = engine_table().unwrap();
        let deps = engine_deps_table().unwrap();
        assert_eq!(libs.len(), 38);
        assert_eq!(deps.len(), 21);
        assert_eq!(
            libs.get("sdk/lib/conutils/libconutils.a")
                .map(String::as_str),
            Some("engine:conutils")
        );
        assert_eq!(
            deps.get("engine:c:libcntpr").map(String::as_str),
            Some("psdk asm")
        );
    }
    #[test]
    fn engine_table_parser_fails_on_unknown_rows_and_duplicate_keys() {
        assert!(parse_engine_tables(
            "ENGINE_LIBS = {\n    UNKNOWN\n}\nENGINE_LIB_DEPS = {\n    \"x\": \"y\"\n}"
        )
        .is_err());
        assert!(parse_engine_tables("ENGINE_LIBS = {\n    \"x\": \"y\"\n    \"x\": \"z\"\n}\nENGINE_LIB_DEPS = {\n    \"x\": \"y\"\n}").is_err());
    }
}
