//! Cargo cell link graph and CMake statement primitives.
use super::{local, read, Error, Link, Result, ARCHES};
use regex::Regex;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn re(s: &str) -> Regex {
    Regex::new(s).expect("constant regex")
}
fn edge_text(why: &str, arch: &str) -> String {
    match why {
        "cell" => format!("already a cargo cell on {arch} (add_cargo_module, no cmake link edge)"),
        "absent" => format!("not defined on {arch} in this configuration (an if(ARCH ...) gate or a build option such as --rostests)"),
        _ => format!("no link edge in build-{arch}/build.ninja"),
    }
}
pub fn no_edge_why(ninja: &str, target: &str) -> &'static str {
    let t = regex::escape(target);
    if re(&format!(
        r"(?m)^build [^:\n]*: CUSTOM_COMMAND \S*/{t}-cargo/Cargo\.toml"
    ))
    .is_match(ninja)
    {
        "cell"
    } else if !re(&format!(
        r"(?m)^build cmake_object_order_depends_target_{t}:"
    ))
    .is_match(ninja)
    {
        "absent"
    } else {
        "unknown"
    }
}
/// A model built from a build.ninja of the previous configure is a model of the previous
/// tree; refuse it (vibeos_build::ninja::require_configured).
fn require_configured(root: &Path, arch: &str) -> Result<()> {
    let build = local(root, &root.join(format!("build-{arch}")))?;
    vibeos_build::ninja::require_configured(&build, root).map_err(|e| Error::new(e.to_string()))
}
/// A link input named by its path inside the target's object dir (`client/i386/fiber.S.obj`),
/// else by file name: kernel32 compiles client/{i386,amd64,arm64}/fiber.S, the SAME object
/// name from a different source per arch, which by file name alone read as one common input.
fn obj_key(obj: &str) -> String {
    match obj.split_once(".dir/") {
        Some((_, rest)) => rest.to_owned(),
        None => obj.rsplit('/').next().unwrap_or(obj).to_owned(),
    }
}
/// The link's entry symbol: the LAST entry flag, as lld takes it. set_entrypoint appends
/// `-entry,<sym>` after the module type's default (ws2help: `-entry,_DllMainCRTStartup@12
/// ... -entry,_Ws2HelpDllMain@12`), and reading the first gave ws2help the wrong entry.
pub fn link_entry(flags: &str) -> String {
    let last = |pattern: &str| {
        re(pattern)
            .captures_iter(flags)
            .last()
            .map(|x| x[1].to_owned())
    };
    last(r"-entry,(\S+)")
        .or_else(|| last(r"-Wl,-e,(\S+)"))
        .unwrap_or_default()
}
/// A target as ninja's rule names spell it: cmake escapes `.` as `.2e`
/// (`C_MODULE_LIBRARY_LINKER__msadp32.2eacm_Debug`). Matching the raw name missed every
/// dotted target (codecs, typelibs, themes), which therefore never converted.
pub fn ninja_rule_name(target: &str) -> String {
    target.replace('.', ".2e")
}
pub fn link_edge(root: &Path, arch: &str, target: &str) -> Result<Link> {
    require_configured(root, arch)?;
    let path = local(root, &root.join(format!("build-{arch}/build.ninja")))?;
    let ninja = read(&path)?;
    let pattern = format!(
        r"(?m)^build (\S+): C(?:XX)?(_\w+?)_LINKER__{}_(?:Debug|Release|RelWithDebInfo|MinSizeRel)( .*\n|\n)((?:  .*\n)+)",
        regex::escape(&ninja_rule_name(target))
    );
    let Some(c) = re(&pattern).captures(&ninja) else {
        let why = no_edge_why(&ninja, target);
        return Err(Error::no_link(
            target,
            arch,
            why,
            format!("{target}: {}", edge_text(why, arch)),
        ));
    };
    let block = &c[4];
    let flags = re(r"(?m)^  LINK_FLAGS = (.*)$")
        .captures(block)
        .map(|x| x[1].to_owned())
        .unwrap_or_default();
    if flags.contains("--delayload") {
        return Err(Error::new(format!(
            "{target}: lld --delayload is not modelled (add_delay_importlibs is)"
        )));
    }
    // The LAST entry flag, as lld takes it: set_entrypoint appends `-entry,<sym>` after the
    // module type's default (ws2help: `-entry,_DllMainCRTStartup@12 ... -entry,
    // _Ws2HelpDllMain@12`), and reading the first gave ws2help the wrong entry.
    let capture = |pattern: &str| re(pattern).captures(&flags).map(|x| x[1].to_owned());
    let entry = link_entry(&flags);
    let subsystem = re(r"--subsystem,([\w:.]+)")
        .captures_iter(&flags)
        .last()
        .map(|x| x[1].to_owned());
    let image_base = capture(r"--image-base,(0x[0-9a-fA-F]+)")
        .and_then(|v| u64::from_str_radix(&v[2..], 16).ok());
    let stack = capture(r"--stack,(\d+)").and_then(|v| v.parse().ok());
    let mut inputs: Vec<String> = c[3]
        .split('|')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    for d in
        re(r"(\S+\.def)\b").captures_iter(c.get(0).unwrap().as_str().lines().next().unwrap_or(""))
    {
        if !inputs.iter().any(|x| x == &d[1]) {
            inputs.push(d[1].to_owned());
        }
    }
    let libs = re(r"(?m)^  LINK_LIBRARIES = (.*)$")
        .captures(block)
        .map(|x| x[1].split_whitespace().map(str::to_owned).collect())
        .unwrap_or_default();
    let require_defined = re(r"--require-defined=([^,\s]+)")
        .captures_iter(&flags)
        .map(|x| x[1].to_owned())
        .collect();
    Ok(Link {
        arch: arch.into(),
        output: c[1].into(),
        kind: format!("C{}", &c[2]),
        entry,
        subsystem,
        image_base,
        stack,
        inputs,
        libs,
        require_defined,
    })
}
type Missing = Vec<(&'static str, String)>;
fn link_edges_partial(root: &Path, target: &str) -> Result<(BTreeMap<String, Link>, Missing)> {
    let mut links = BTreeMap::new();
    let mut missing = Vec::new();
    for arch in ARCHES {
        match link_edge(root, arch, target) {
            Ok(link) => {
                links.insert(arch.to_owned(), link);
            }
            Err(e) if e.kind == "NoLinkEdge" => missing.push((arch, e.why.unwrap_or_default())),
            Err(e) => return Err(e),
        }
    }
    Ok((links, missing))
}
pub fn link_edges(root: &Path, target: &str) -> Result<BTreeMap<String, Link>> {
    let (links, missing) = link_edges_partial(root, target)?;
    if missing.is_empty() {
        return Ok(links);
    }
    Err(missing_edges_error(target, &links, &missing))
}
/// The links of a module, which may be absent on some arches only when the module's own
/// CMakeLists says so: every `if()` enclosing its add_library/add_executable must be a
/// pure `ARCH STREQUAL "<a>" [OR ...]` test, and together they must name exactly the
/// arches cmake linked it on. Absence for any other reason (a build option such as
/// --rostests, a gated add_subdirectory upstream) stays a refusal, since a cell built
/// from it would silently drop the module from an arch that should have it.
pub fn link_edges_gated(root: &Path, target: &str) -> Result<BTreeMap<String, Link>> {
    let (links, missing) = link_edges_partial(root, target)?;
    if missing.is_empty() {
        return Ok(links);
    }
    if links.is_empty() || missing.iter().any(|(_, w)| w != "absent") {
        return Err(missing_edges_error(target, &links, &missing));
    }
    let (arch, link) = links.iter().next().unwrap();
    let cmake = source_dir(root, arch, &link.output, target)?.join("CMakeLists.txt");
    let own = arch_gate(&super::fncall::cmake_text(&cmake, target)?, target);
    // Not gated in its own file: the parent's add_subdirectory(<dir>) may be.
    let gate = match own {
        Err(why) if why.ends_with("is not inside an if()") => {
            let dir = cmake.parent().unwrap_or(Path::new("."));
            let parent = dir.parent().map(|p| p.join("CMakeLists.txt"));
            let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
            match parent.filter(|p| p.is_file()) {
                Some(p) => subdirectory_gate(&std::fs::read_to_string(&p)?, name)
                    .map_err(|w| format!("{why}, and in {}: {w}", p.display())),
                None => Err(why),
            }
        }
        other => other,
    };
    let gate = gate.map_err(|why| {
        Error::new(format!(
            "{}\n  and the arch set is not proven by {}: {why}",
            missing_edges_error(target, &links, &missing),
            cmake.display()
        ))
    })?;
    let have: Vec<&str> = links.keys().map(String::as_str).collect();
    let mut want: Vec<&str> = gate.iter().map(String::as_str).collect();
    want.sort();
    let mut have_sorted = have.clone();
    have_sorted.sort();
    if want != have_sorted {
        return Err(Error::new(format!(
            "{target}: its if(ARCH ...) gate names {want:?} but cmake links it on {have_sorted:?}"
        )));
    }
    Ok(links)
}
/// The arch set an `if(ARCH STREQUAL ...)` nest grants `target`'s add_library/add_executable.
pub fn arch_gate(text: &str, target: &str) -> std::result::Result<Vec<String>, String> {
    gate_of(text, &["add_library", "add_executable"], target, false)
}
/// The arch set an `if(ARCH ...)` nest grants `add_subdirectory(<dir>)` in a parent
/// CMakeLists: legide (i386) and pl011 (arm64) are gated there, not in their own file.
pub fn subdirectory_gate(text: &str, dir: &str) -> std::result::Result<Vec<String>, String> {
    gate_of(text, &["add_subdirectory"], dir, true)
}
/// An `if()` condition's arch set: `ARCH STREQUAL "a" [OR ...]`, or (`and`, a parent's
/// add_subdirectory only) ARCH tests ANDed with build options (`ARCH STREQUAL "i386" AND NOT
/// ENABLE_UNIATA`). The whole rewritten CMakeLists stays under that parent condition, so the
/// options keep gating the cell. In a module's own file they would not: the cell block
/// re-emits only the arch part.
fn condition_arches(raw: &str, and: bool) -> Option<BTreeMap<String, ()>> {
    let term = re(r#"^ARCH\s+STREQUAL\s+"?(\w+)"?$"#);
    let raw = raw.trim();
    if and && raw.contains(" AND ") && !raw.contains(" OR ") && !raw.contains('(') {
        let mut out: Option<BTreeMap<String, ()>> = None;
        for t in raw.split(" AND ") {
            if let Some(c) = term.captures(t.trim()) {
                let one: BTreeMap<String, ()> = [(c[1].to_owned(), ())].into();
                out = Some(match out {
                    None => one,
                    Some(prev) => prev
                        .into_iter()
                        .filter(|(k, _)| one.contains_key(k))
                        .collect(),
                });
            }
        }
        return out;
    }
    raw.split(" OR ")
        .map(|t| term.captures(t.trim()).map(|c| (c[1].to_owned(), ())))
        .collect()
}
fn gate_of(
    text: &str,
    cmds: &[&str],
    target: &str,
    and: bool,
) -> std::result::Result<Vec<String>, String> {
    let mut stack: Vec<Option<BTreeMap<String, ()>>> = Vec::new();
    for (_, _, cmd, raw) in statements(text) {
        match cmd.as_str() {
            "if" => stack.push(condition_arches(&raw, and)),
            // A later branch is only refused when it encloses the target: pl011's
            // `if(ARCH ...) elseif(...)` RUST_TARGET chain closes before its add_library.
            "elseif" | "else" => {
                if let Some(top) = stack.last_mut() {
                    *top = None;
                }
            }
            "endif" => {
                stack.pop();
            }
            c if cmds.contains(&c) && args(&raw).first().map(String::as_str) == Some(target) => {
                if stack.is_empty() {
                    return Err(format!("its {cmd}({target} ...) is not inside an if()"));
                }
                let mut out: Option<Vec<String>> = None;
                for cond in &stack {
                    let set = cond.as_ref().ok_or_else(|| {
                        "an enclosing if() is not a pure ARCH STREQUAL test (or is an elseif/else branch)".to_owned()
                    })?;
                    let names: Vec<String> = set.keys().cloned().collect();
                    if let Some(n) = names.iter().find(|n| !ARCHES.contains(&n.as_str())) {
                        return Err(format!("the gate names an unknown arch {n}"));
                    }
                    out = Some(match out {
                        None => names,
                        Some(prev) => prev.into_iter().filter(|a| names.contains(a)).collect(),
                    });
                }
                return Ok(out.unwrap_or_default());
            }
            _ => {}
        }
    }
    Err(format!("no {}({target} ...)", cmds.join("/")))
}
fn missing_edges_error(target: &str, links: &BTreeMap<String, Link>, missing: &Missing) -> Error {
    let have: Vec<_> = links.keys().map(String::as_str).collect();
    let whys: Vec<_> = missing
        .iter()
        .map(|(a, w)| format!("{a}: {}", edge_text(w, a)))
        .collect();
    let pylist = |a: Vec<&str>| {
        format!(
            "[{}]",
            a.iter()
                .map(|x| format!("'{x}'"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let msg = if have.is_empty() && missing.iter().all(|x| x.1 == missing[0].1) {
        format!("{target}: {}", edge_text(&missing[0].1, "every arch"))
    } else if have.is_empty() {
        format!(
            "{target}: no cmake link edge on any arch ({})",
            whys.join(", ")
        )
    } else if missing.iter().all(|x| x.1 == "cell") {
        let mut m: Vec<_> = missing.iter().map(|x| x.0).collect();
        m.sort();
        format!("{target}: a cargo cell on {} but still cmake-linked on {} (a partly converted module is not modelled)",pylist(m),pylist(have))
    } else {
        format!("{target}: {}, but cmake links it on {}: the generator needs the same cmake-linked module on every arch, unless an if(ARCH ...) gate in its own CMakeLists names the arches",whys.join(", "),pylist(have))
    };
    Error::new(msg)
}
/// CMake calls with byte offsets suitable for rewriting UTF-8 input.
pub type Statement = (usize, usize, String, String);
pub fn statements(text: &str) -> Vec<Statement> {
    let pattern = re(r"(?m)^[ \t]*([A-Za-z_][A-Za-z0-9_]*)[ \t]*\(");
    let bytes = text.as_bytes();
    let mut end_prev = 0;
    let mut out = Vec::new();
    for m in pattern.captures_iter(text) {
        let whole = m.get(0).unwrap();
        if whole.start() < end_prev {
            continue;
        }
        let mut depth = 0usize;
        let mut j = whole.end() - 1;
        while j < bytes.len() {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                b'#' if depth == 1 && j > 0 && b" \t\n".contains(&bytes[j - 1]) => {
                    j = text[j..]
                        .find('\n')
                        .map(|p| j + p - 1)
                        .unwrap_or(bytes.len());
                }
                _ => {}
            }
            j += 1;
        }
        let raw_end = j.min(bytes.len());
        let mut end = (j + 1).min(bytes.len());
        if end < bytes.len() && bytes[end] == b'\n' {
            end += 1;
        }
        out.push((
            whole.start(),
            end,
            m[1].to_ascii_lowercase(),
            text[whole.end()..raw_end].to_owned(),
        ));
        end_prev = end;
    }
    out
}
pub fn args(raw: &str) -> Vec<String> {
    let clean = re(r"#[^\n]*").replace_all(raw, "");
    re(r#""[^"]*"|\S+"#)
        .find_iter(&clean)
        .map(|m| m.as_str().to_owned())
        .collect()
}
pub fn branches(stmts: &[Statement]) -> Vec<Vec<(usize, usize)>> {
    let mut stack = Vec::new();
    let mut out = Vec::new();
    let mut next = 0;
    for (_, _, cmd, _) in stmts {
        match cmd.as_str() {
            "if" => {
                stack.push((next, 0));
                next += 1;
            }
            "elseif" | "else" => {
                if let Some(x) = stack.last_mut() {
                    x.1 += 1;
                }
            }
            _ => {}
        }
        out.push(stack.clone());
        if cmd == "endif" {
            stack.pop();
        }
    }
    out
}
pub fn exclusive(p: &[(usize, usize)], q: &[(usize, usize)]) -> bool {
    for (a, b) in p.iter().zip(q) {
        if a != b {
            return a.0 == b.0;
        }
    }
    false
}
pub fn source_dir(root: &Path, arch: &str, output: &str, _target: &str) -> Result<PathBuf> {
    let parent = Path::new(output).parent().unwrap_or(Path::new(""));
    let inst = root
        .join(format!("build-{arch}"))
        .join(parent)
        .join("cmake_install.cmake");
    let cache = root.join(format!("build-{arch}/CMakeCache.txt"));
    if inst.exists() && cache.exists() {
        let ins = read(&local(root, &inst)?)?;
        let cac = read(&local(root, &cache)?)?;
        if let (Some(m), Some(h)) = (
            re(r"^# Install script for directory: (.+)").captures(&ins),
            re(r"(?m)^CMAKE_HOME_DIRECTORY:INTERNAL=(.+)$").captures(&cac),
        ) {
            if let Ok(rel) = Path::new(&m[1]).strip_prefix(&h[1]) {
                return source_candidate(root, rel);
            }
        }
    }
    source_candidate(root, parent)
}

fn source_candidate(root: &Path, rel: &Path) -> Result<PathBuf> {
    if rel.is_absolute()
        || rel
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(Error::new("source directory escapes selected checkout"));
    }
    let candidate = root.join(rel);
    let mut ancestor = candidate.as_path();
    while !ancestor.exists() {
        if ancestor.is_symlink() {
            return Err(Error::new("dangling source directory symlink"));
        }
        ancestor = ancestor.parent().ok_or("source has no checkout ancestor")?;
    }
    local(root, ancestor)?;
    Ok(candidate)
}

fn cmake_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for ent in std::fs::read_dir(dir).map_err(|e| Error::new(e.to_string()))? {
            let ent = ent.map_err(|e| Error::new(e.to_string()))?;
            let path = ent.path();
            let name = ent.file_name();
            let name = name.to_string_lossy();
            if ent
                .file_type()
                .map_err(|e| Error::new(e.to_string()))?
                .is_dir()
            {
                if dir == root
                    && (name.starts_with("build-")
                        || name.starts_with('.')
                        || name == "cmake-off-out")
                {
                    continue;
                }
                visit(root, &path, out)?;
            } else if name == "CMakeLists.txt" {
                out.push(local(root, &path)?);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(root, root, &mut out)?;
    out.sort();
    Ok(out)
}
pub type LibraryIndex = BTreeMap<String, (String, Vec<String>, Vec<String>)>;
pub fn library_index(root: &Path) -> Result<LibraryIndex> {
    let mut found: BTreeMap<String, Vec<(String, Vec<String>)>> = BTreeMap::new();
    let mut deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for cm in cmake_files(root)? {
        let text = read(&cm)?;
        if !text.contains("add_library") && !text.contains("rust_staticlib_variant") {
            continue;
        }
        for (_, _, cmd, raw) in statements(&text) {
            let a = args(&raw);
            if a.is_empty() {
                continue;
            }
            if cmd == "add_library" || cmd == "rust_staticlib_variant" {
                found.entry(a[0].clone()).or_default().push((cmd, a));
            } else if cmd == "add_dependencies" {
                deps.entry(a[0].clone())
                    .or_default()
                    .extend_from_slice(&a[1..]);
            }
        }
    }
    Ok(found
        .into_iter()
        .filter_map(|(name, defs)| {
            if defs.len() != 1 {
                return None;
            }
            let (cmd, a) = defs.into_iter().next().unwrap();
            let dep = deps.remove(&name).unwrap_or_default();
            Some((name, (cmd, a, dep)))
        })
        .collect())
}
pub fn engine_lib(lib: &str, index: &LibraryIndex) -> Option<(String, Vec<String>)> {
    let constants = super::constants();
    if let Some(name) = constants["ENGINE_LIBS"].get(lib).and_then(|v| v.as_str()) {
        let deps = constants["ENGINE_LIB_DEPS"]
            .get(name)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        return Some((name.into(), deps));
    }
    let cap = re(r"(?:^|/)lib([\w.+-]+)\.a$").captures(lib)?;
    let (kind, a, deps) = index.get(&cap[1])?;
    if kind == "rust_staticlib_variant" {
        if a.len() < 4 || a[4..].iter().any(|x| x == "ENV") {
            return None;
        }
        let feats = a[2]
            .trim_matches('"')
            .split([',', ' ', ';'])
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(",");
        return Some((
            format!("engine:crate:{}:{}:{}", a[1], feats, a[3].to_uppercase()),
            Vec::new(),
        ));
    }
    if a.get(1).is_some_and(|v| {
        [
            "SHARED",
            "MODULE",
            "OBJECT",
            "INTERFACE",
            "IMPORTED",
            "ALIAS",
        ]
        .contains(&v.as_str())
    }) {
        return None;
    }
    let mut d: Vec<String> = deps
        .iter()
        .filter(|d| !d.starts_with("rust_"))
        .cloned()
        .collect();
    if d.is_empty() {
        d.push("psdk".into());
    }
    Some((format!("engine:c:{}", a[0]), d))
}

fn one_list(root: &Path, arch: &str, stem: &str) -> Result<Option<PathBuf>> {
    let dir = root.join(format!("build-{arch}/boot"));
    let mut files = Vec::new();
    for item in std::fs::read_dir(&dir).map_err(|e| Error::new(e.to_string()))? {
        let p = item.map_err(|e| Error::new(e.to_string()))?.path();
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.starts_with(&format!("{stem}."))
            && name.ends_with(".lst")
            && name != format!("{stem}.cmake.lst")
            && name != format!("{stem}.stripped.lst")
        {
            files.push(p);
        }
    }
    files.sort();
    if files.len() > 1 {
        return Err(Error::new(format!(
            "build-{arch}/boot: several {stem}.<config>.lst: {files:?}"
        )));
    }
    files
        .into_iter()
        .next()
        .map(|p| local(root, &p))
        .transpose()
}
pub fn cd_dirs(
    root: &Path,
    arch: &str,
    output: &str,
) -> Result<std::collections::BTreeSet<String>> {
    let list = one_list(root, arch, "livecd")?.ok_or_else(|| {
        Error::new(format!(
            "build-{arch}/boot: expected one livecd.<config>.lst, found []"
        ))
    })?;
    let want = format!("/tmp/vibeos/build-{arch}/{output}");
    let mut out = std::collections::BTreeSet::new();
    for line in read(&list)?.lines() {
        if let Some((dest, source)) = line.split_once('=') {
            if source == want {
                out.insert(dest.rsplit_once('/').map(|x| x.0).unwrap_or(dest).into());
            }
        }
    }
    Ok(out)
}
pub fn cd_placement(root: &Path, arch: &str, output: &str) -> Result<Vec<String>> {
    let want = format!("/tmp/vibeos/build-{arch}/{output}");
    let mut out = std::collections::BTreeSet::new();
    for cd in ["livecd", "bootcd", "bootcdregtest"] {
        if let Some(path) = one_list(root, arch, cd)? {
            for line in read(&path)?.lines() {
                if let Some((dest, source)) = line.split_once('=') {
                    if source == want {
                        out.insert(format!("{cd}:{dest}"));
                    }
                }
            }
        }
    }
    let dir = root.join(format!("build-{arch}/boot/bootdata/packages"));
    let mut dyns = Vec::new();
    for item in std::fs::read_dir(&dir).map_err(|e| Error::new(e.to_string()))? {
        let p = item.map_err(|e| Error::new(e.to_string()))?.path();
        let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if n.starts_with("reactos.dff.") && n.ends_with(".dyn") {
            dyns.push(p);
        }
    }
    if dyns.len() != 1 {
        return Err(Error::new(format!(
            "build-{arch}: expected one reactos.dff.<config>.dyn, found {dyns:?}"
        )));
    }
    let pattern = re(&format!(r#"(?m)^"{}" (\d+)$"#, regex::escape(&want)));
    for cap in pattern.captures_iter(&read(&local(root, &dyns[0])?)?) {
        out.insert(format!("cab:{}", &cap[1]));
    }
    Ok(out.into_iter().collect())
}

pub fn spec_index(root: &Path) -> Result<BTreeMap<String, Option<(String, String)>>> {
    use std::collections::BTreeSet;
    let mut candidates: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    let pattern = re(r"spec2def\(\s*(\S+?)\s+(\S+?\.spec)\b([^)]*)\)");
    for cm in cmake_files(root)? {
        let rel = cm
            .strip_prefix(root)
            .map_err(|e| Error::new(e.to_string()))?
            .to_string_lossy();
        if rel.starts_with("vendor/") || rel.starts_with("modules/rostests") || rel.starts_with('.')
        {
            continue;
        }
        for c in pattern.captures_iter(&read(&cm)?) {
            if !c[3].contains("ADD_IMPORTLIB") {
                continue;
            }
            let path = cm.parent().unwrap().join(&c[2]);
            let spec = path
                .strip_prefix(root)
                .map_err(|e| Error::new(e.to_string()))?
                .to_string_lossy()
                .into_owned();
            let name = c[1]
                .rsplit_once('.')
                .map(|p| p.0)
                .unwrap_or(&c[1])
                .to_lowercase();
            candidates
                .entry(name)
                .or_default()
                .insert((spec, c[1].into()));
        }
    }
    let pattern = re(
        r"spec2def [^&|\n]*?-n=(\S+) [^&|\n]*?--implib -d=\S*/lib(\w+)_implib\.def /tmp/vibeos/(\S+\.spec)",
    );
    let mut built: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for arch in ARCHES {
        require_configured(root, arch)?;
        let path = local(root, &root.join(format!("build-{arch}/build.ninja")))?;
        for c in pattern.captures_iter(&read(&path)?) {
            let mut parts = Vec::new();
            for p in c[3].split('/') {
                match p {
                    "." | "" => {}
                    ".." => {
                        if parts.last().is_some_and(|p| *p != "..") {
                            parts.pop();
                        } else {
                            parts.push("..");
                        }
                    }
                    _ => parts.push(p),
                }
            }
            built
                .entry(c[2].to_lowercase())
                .or_default()
                .insert((parts.join("/"), c[1].into()));
        }
    }
    let keys: BTreeSet<_> = candidates.keys().chain(built.keys()).cloned().collect();
    Ok(keys
        .into_iter()
        .map(|k| {
            let vals = built.get(&k).or_else(|| candidates.get(&k)).unwrap();
            let v = if vals.len() == 1 {
                vals.iter().next().cloned()
            } else {
                None
            };
            (k, v)
        })
        .collect())
}

pub fn entry_symbol(arch: &str, sym: &str, stack: Option<&str>) -> String {
    if sym == "0" {
        return if arch == "i386" {
            "___ImageBase"
        } else {
            "__ImageBase"
        }
        .into();
    }
    if arch != "i386" {
        return sym.into();
    }
    match stack {
        Some(s) if !s.is_empty() => format!("_{sym}@{s}"),
        _ => format!("_{sym}"),
    }
}
pub fn subsystem_flag(sub: &str) -> String {
    format!(
        "{}:5.01",
        match sub {
            "10" => "efi_application",
            "11" => "efi_boot_service_driver",
            "12" => "efi_runtime_driver",
            "13" => "efi_rom",
            _ => sub,
        }
    )
}
pub fn check_statement_effects(
    target: &str,
    facts: &serde_json::Value,
    links: &BTreeMap<String, Link>,
) -> Result<()> {
    if let Some(want) = facts["set_subsystem"].as_str() {
        for (a, ln) in links {
            if ln.subsystem.as_deref() != Some(want) {
                return Err(Error::new(format!(
                    "{target}: set_subsystem says {want} but the {a} link has {}",
                    ln.subsystem.as_deref().unwrap_or("None")
                )));
            }
        }
    }
    if let Some(ep) = facts["set_entrypoint"].as_array() {
        if let Some(sym) = ep.first().and_then(|v| v.as_str()) {
            for (a, ln) in links {
                let want = entry_symbol(a, sym, ep.get(1).and_then(|v| v.as_str()));
                if ln.entry != want {
                    return Err(Error::new(format!(
                        "{target}: set_entrypoint says {want} but the {a} link has {}",
                        if ln.entry.is_empty() {
                            "no entry"
                        } else {
                            &ln.entry
                        }
                    )));
                }
            }
        }
    }
    if facts["require_defined"].as_bool() == Some(true)
        && links.values().any(|l| l.require_defined.is_empty())
    {
        return Err(Error::new(format!(
            "{target}: target_link_options --require-defined is on no link edge"
        )));
    }
    Ok(())
}

fn const_contains(key: &str, value: &str) -> bool {
    super::constants()[key]
        .as_array()
        .is_some_and(|v| v.iter().any(|x| x.as_str() == Some(value)))
}
pub fn placeholder(path: &Path) -> Result<bool> {
    let text = read(path)?;
    let cleaned = re(r"(?s)/\*.*?\*/|//[^\n]*").replace_all(&text, "");
    Ok(re(r"typedef\s+int\s+\w+\s*;")
        .replace_all(&cleaned, "")
        .trim()
        .is_empty())
}
pub fn inlinable_source(text: &str, dir: &Path) -> Result<Option<Vec<String>>> {
    let mut out = Vec::new();
    for (_, _, cmd, raw) in statements(text) {
        let a = args(&raw);
        if cmd == "list" && a.len() >= 2 && a[0] == "APPEND" && a[1] == "SOURCE" {
            out.extend_from_slice(&a[2..]);
        }
    }
    if text.contains("PCH_SKIP_SOURCE") || out.is_empty() {
        return Ok(None);
    }
    for p in &out {
        if p.ends_with(".h") || p.ends_with(".hpp") {
            continue;
        }
        let file = dir.join(p);
        if ![".c", ".cpp", ".cxx", ".cc"].iter().any(|x| p.ends_with(x))
            || !file.exists()
            || !placeholder(&file)?
        {
            return Ok(None);
        }
    }
    Ok(Some(out))
}
pub fn included_functions(cmake: &Path, text: &str) -> Result<std::collections::BTreeSet<String>> {
    let mut names = std::collections::BTreeSet::new();
    let dir = cmake.parent().unwrap_or(Path::new("."));
    for (_, _, cmd, raw) in statements(text) {
        let a = args(&raw);
        if cmd != "include" || a.is_empty() {
            continue;
        }
        let inc = a[0]
            .trim_matches('"')
            .replace("${CMAKE_CURRENT_SOURCE_DIR}", &dir.to_string_lossy());
        let path = dir.join(inc);
        if path.is_file() {
            for c in re(r"(?im)^[ \t]*(?:function|macro)[ \t]*\([ \t]*(\w+)")
                .captures_iter(&read(&path)?)
            {
                names.insert(c[1].to_lowercase());
            }
        }
    }
    Ok(names)
}
pub fn cmake_facts(cmake: &Path, target: &str, libs: &[String]) -> Result<serde_json::Value> {
    use serde_json::{json, Value};
    let text = super::fncall::cmake_text(cmake, target)?;
    let stmts = statements(&text);
    let paths = branches(&stmts);
    let home = stmts
        .iter()
        .position(|(_, _, c, r)| {
            ["add_library", "add_executable"].contains(&c.as_str())
                && args(r).first().is_some_and(|x| x == target)
        })
        .map(|i| paths[i].clone());
    let mut f = json!({"destinations":[],"name_on_cd":null,"body":null,"body_var":"","transfer":[],"reg_objects":[],"unknown":[],"unicode":false,"body_rustflags":{},"body_env_block":"","extra_cd":[],"no_cab":false});
    let mut variants: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut defined = false;
    for (i, (_, _, cmd, raw)) in stmts.iter().enumerate() {
        let a = args(raw);
        if cmd == "rust_staticlib_variant" && a.len() >= 4 {
            variants.insert(a[0].clone(), a.clone());
        }
        if ["add_library", "add_executable"].contains(&cmd.as_str())
            && a.first().is_some_and(|v| v == target)
        {
            defined = true;
        }
        // spec2def names the OUTPUT file (`msadp32.dll`), which for a dotted target
        // (`msadp32.acm`, `stdole2.tlb`) is the target's own name: it is the module's
        // .spec, read by its own pattern, not a statement on the target.
        if a.is_empty()
            || cmd == "spec2def"
            || (a[0] != target && !(cmd == "add_cd_file" && a.iter().any(|x| x == target)))
        {
            if re(r"\$\{(SOURCE|PCH_SKIP_SOURCE)\}").is_match(raw)
                && !home.as_ref().is_some_and(|p| exclusive(&paths[i], p))
                && inlinable_source(&text, cmake.parent().unwrap())?.is_none()
            {
                f["unknown"].as_array_mut().unwrap().push(json!(format!(
                    "{cmd} {} shares ${{SOURCE}}",
                    a.first().map(String::as_str).unwrap_or("")
                )));
            }
            continue;
        }
        if cmd == "set_module_type" {
            if a.iter().any(|x| x == "UNICODE") {
                f["unicode"] = json!(true);
            }
            if a.len() > 1 {
                f["module_type"] = json!(a[1]);
            }
        }
        if cmd == "add_cd_file" {
            let idx = a
                .iter()
                .position(|v| v == "FOR")
                .map(|x| x + 1)
                .unwrap_or(a.len());
            if !a[idx..].iter().any(|v| v == "all" || v == "livecd") {
                let j = a
                    .iter()
                    .position(|v| v == "TARGET")
                    .ok_or_else(|| Error::new(format!("{target}: add_cd_file has no TARGET")))?;
                if a.get(j + 1).map(String::as_str) != Some(target) {
                    return Err(Error::new(format!(
                        "{target}: add_cd_file names TARGET {}",
                        a.get(j + 1).map(String::as_str).unwrap_or("")
                    )));
                }
                let mut b = a.clone();
                b.drain(j..j + 2);
                f["extra_cd"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(b.join(" ")));
            } else {
                if a.iter().any(|x| x == "NO_CAB") {
                    f["no_cab"] = json!(true);
                }
                if let Some(v) = a
                    .iter()
                    .position(|x| x == "DESTINATION")
                    .and_then(|i| a.get(i + 1))
                {
                    let d = f["destinations"].as_array_mut().unwrap();
                    if !d.contains(&json!(v)) {
                        d.push(json!(v));
                    }
                }
                if let Some(v) = a
                    .iter()
                    .position(|x| x == "NAME_ON_CD")
                    .and_then(|i| a.get(i + 1))
                {
                    f["name_on_cd"] = json!(v);
                }
            }
        } else if const_contains("TRANSFERRED", cmd) {
            f["transfer"].as_array_mut().unwrap().push(json!(raw));
        } else if cmd == "set_target_properties"
            && a.len() >= 4
            && a[1] == "PROPERTIES"
            && a.len() % 2 == 0
            && a[2..]
                .iter()
                .step_by(2)
                .all(|v| const_contains("NAME_PROPERTIES", v))
        {
        } else if cmd == "set_target_cpp_properties" {
        } else if cmd == "target_link_options" {
            let opts: Vec<_> = a[1..]
                .iter()
                .filter(|v| *v != "PRIVATE" && *v != "PUBLIC")
                .map(|v| v.trim_matches('"'))
                .collect();
            if !opts.is_empty()&&opts.iter().all(|o|re(r"^-Wl,--subsystem,[\w:.]+$").is_match(o)) {} else if !opts.is_empty()&&opts.iter().all(|o|re(r"^(?:LINKER:|-Wl,)--require-defined=[^,\s]+(?:,--require-defined=[^,\s]+)*$").is_match(o)){f["require_defined"]=json!(true);}else {f["unknown"].as_array_mut().unwrap().push(json!(cmd));}
        } else if cmd == "set_subsystem" && a.len() == 2 {
            f["set_subsystem"] = json!(subsystem_flag(&a[1]));
        } else if cmd == "set_entrypoint" && (a.len() == 2 || a.len() == 3) {
            f["set_entrypoint"] = json!([Some(&a[1]), a.get(2)]);
        } else if cmd == "target_sources"
            && a.len() == 3
            && a[1] == "PRIVATE"
            && re(r"^\$<TARGET_OBJECTS:gcc_ssp_\w+>$").is_match(&a[2])
        {
            f["ssp"] = json!(true);
        } else if cmd == "add_idl_reg_scripts"
            && a.len() > 2
            && super::constants()["REG_SUFFIX"].get(&a[1]).is_some()
        {
            let suffix = super::constants()["REG_SUFFIX"][&a[1]].as_str().unwrap();
            for src in &a[2..] {
                let stem = Path::new(src)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                f["reg_objects"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(format!("{stem}{suffix}.o")));
            }
        } else if cmd == "link_strict" || cmd == "link_msvcrt_rust_cohab" {
            let why = if cmd == "link_strict" {
                "link_strict (the link drops --allow-multiple-definition, so lld rejects any duplicate symbol; reactos-link always passes the flag)"
            } else {
                "link_msvcrt_rust_cohab (--whole-archive of libmsvcrt_crt_cargo.a, a cmake-built archive of another crate; the cell has no whole-archive input)"
            };
            f["unknown"].as_array_mut().unwrap().push(json!(why));
        } else if !const_contains("REPLACED", cmd) && !const_contains("KEPT", cmd) {
            f["unknown"].as_array_mut().unwrap().push(json!(cmd));
        }
    }
    if !defined {
        f["unknown"]
            .as_array_mut()
            .unwrap()
            .push(json!("target made in a loop or function"));
        let funcs = included_functions(cmake, &text)?;
        let calls: Vec<_> = stmts
            .iter()
            .filter_map(|(_, _, c, r)| {
                let a = args(r);
                if funcs.contains(c)
                    || (a.first().is_some_and(|x| x == target)
                        && !const_contains("REPLACED", c)
                        && !const_contains("KEPT", c))
                {
                    Some(format!("{c}({})", a.join(" ")))
                } else {
                    None
                }
            })
            .collect();
        if !calls.is_empty() {
            f["unknown"].as_array_mut().unwrap().push(json!(format!(
                "made by a shared function, not an add_library of its own: {}",
                calls.join(", ")
            )));
        }
    }
    let linked: Vec<_> = libs
        .iter()
        .map(|x| x.rsplit('/').next().unwrap_or(x))
        .collect();
    let mut named: Vec<_> = variants
        .keys()
        .filter(|n| linked.contains(&format!("lib{n}.a").as_str()))
        .cloned()
        .collect();
    if named.len() != 1 {
        named = variants
            .keys()
            .filter(|n| *n == &format!("{target}_rust") || n.strip_suffix("_rust") == Some(target))
            .cloned()
            .collect();
        if named.is_empty() {
            named = variants
                .keys()
                .filter(|n| linked.contains(&format!("lib{n}.a").as_str()))
                .cloned()
                .collect();
        }
    }
    if named.len() == 1 {
        let a = &variants[&named[0]];
        let feats: Vec<_> = a[2]
            .trim_matches('"')
            .split([';', ' '])
            .filter(|v| !v.is_empty())
            .collect();
        f["body"] = json!([
            Value::String(a[1].clone()),
            json!(feats),
            json!(a[3].eq_ignore_ascii_case("TRUE"))
        ]);
        f["body_var"] = json!(named[0]);
        if a.len() > 4 {
            if let Some((flags, block)) = body_env(&text, &a[4..]) {
                f["body_rustflags"] = json!(flags);
                f["body_env_block"] = json!(block);
            } else if let Some(deps) = body_depends(&text, &a[4..]) {
                f["body_depends"] = json!(deps);
            } else {
                f["unknown"].as_array_mut().unwrap().push(json!(format!(
                    "rust_staticlib_variant extras [{}]",
                    a[4..]
                        .iter()
                        .map(|s| format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
        }
    } else if named.len() > 1 {
        f["unknown"]
            .as_array_mut()
            .unwrap()
            .push(json!(format!("several bodies {named:?}")));
    } else if let Some(custom) = custom_body_for(&text, target) {
        f["body"] = json!([custom["crate"], [], true]);
        f["body_rustflags"] = custom["rustflags"].clone();
        if let Some(d) = custom["depends"].as_array().filter(|d| !d.is_empty()) {
            f["body_depends"] = json!(d);
        }
        f["custom_body"] = custom;
    } else if re(r"\bcargo build\b").is_match(&text) {
        f["unknown"]
            .as_array_mut()
            .unwrap()
            .push(json!("custom cargo body is not yet modelled"));
    }
    Ok(f)
}

/// rust_staticlib_variant extras that are only file DEPENDS: each a `${VAR}` set to the
/// OUTPUT of an add_custom_command in the same CMakeLists (wow64's thunk-table drift
/// check). The cell keeps the command and hands the file to add_cargo_module's
/// EXTRA_DEPENDS, so the check still runs before the body builds and a failure still
/// stops the link.
pub fn body_depends(text: &str, extras: &[String]) -> Option<Vec<String>> {
    let var_re = re(r"^\$\{(\w+)\}$");
    let outputs: Vec<Vec<String>> = statements(text)
        .into_iter()
        .filter(|(_, _, cmd, _)| cmd == "add_custom_command")
        .map(|(_, _, _, raw)| {
            let a = args(&raw);
            let start = a
                .iter()
                .position(|x| x == "OUTPUT")
                .map_or(a.len(), |i| i + 1);
            a[start..]
                .iter()
                .take_while(|x| {
                    x.chars().any(|c| c.is_ascii_lowercase()) || x.contains('$') || x.contains('/')
                })
                .cloned()
                .collect()
        })
        .collect();
    let mut out = Vec::new();
    for extra in extras {
        let var = &var_re.captures(extra)?[1];
        let sets: Vec<_> = statements(text)
            .into_iter()
            .filter(|(_, _, cmd, raw)| {
                cmd == "set" && args(raw).first().map(String::as_str) == Some(var)
            })
            .collect();
        if sets.len() != 1 || args(&sets[0].3).len() != 2 {
            return None;
        }
        if !outputs.iter().any(|o| o.contains(extra)) {
            return None;
        }
        out.push(extra.clone());
    }
    (!out.is_empty()).then_some(out)
}
pub fn body_env(text: &str, extras: &[String]) -> Option<(BTreeMap<String, String>, String)> {
    if extras.len() != 1 {
        return None;
    }
    let var_re = re(r"^\$\{(\w+)\}$");
    let caps = var_re.captures(&extras[0])?;
    let var = &caps[1];
    let pat = format!(
        r#"set\({0}\)\nif\(ARCH STREQUAL "(\w+)"\)\n\s*set\({0} ENV "RUSTFLAGS=([^"]*)"\)\nendif\(\)\n"#,
        regex::escape(var)
    );
    let pattern = re(&pat);
    let c = pattern.captures(text)?;
    if !ARCHES.contains(&&c[1])
        || re(&format!(r"\b{}\b", regex::escape(var)))
            .find_iter(text)
            .count()
            != 3
    {
        return None;
    }
    let mut flags = BTreeMap::new();
    flags.insert(c[1].into(), c[2].into());
    Some((flags, c[0].into()))
}
pub fn enclosing_ifs(text: &str, target: &str) -> Result<Vec<String>> {
    let mut stack: Vec<Option<String>> = Vec::new();
    for (_, _, cmd, raw) in statements(text) {
        match cmd.as_str() {
            "if" => stack.push(Some(raw.split_whitespace().collect::<Vec<_>>().join(" "))),
            "elseif" | "else" => {
                if let Some(last) = stack.last_mut() {
                    *last = None
                }
            }
            "endif" => {
                stack.pop();
            }
            "add_library" | "add_executable" if args(&raw).first().is_some_and(|a| a == target) => {
                if stack.iter().any(Option::is_none) {
                    return Err(Error::new(format!(
                        "{target}: its link is in an elseif/else branch"
                    )));
                }
                return Ok(stack.into_iter().flatten().collect());
            }
            _ => {}
        }
    }
    Ok(Vec::new())
}
pub fn cmakelists(root: &Path) -> Result<Vec<PathBuf>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "*CMakeLists.txt"])
        .output()
        .map_err(|e| Error::new(e.to_string()))?;
    if !output.status.success() {
        return Err(Error::new(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|p| local(root, &root.join(p)))
        .collect()
}
pub fn call_span(text: &str, start: usize) -> Result<usize> {
    let begin = text
        .get(start..)
        .and_then(|s| s.find('('))
        .map(|i| i + start)
        .ok_or_else(|| Error::new("CMake call has no opening parenthesis"))?;
    let mut depth = 0usize;
    for (offset, byte) in text.as_bytes()[begin..].iter().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(begin + offset + 1);
                }
            }
            _ => {}
        }
    }
    Err(Error::new("CMake call has no closing parenthesis"))
}
pub fn arch_values(text: &str, var: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut arch = None;
    let pat = re(r#"^ARCH STREQUAL "?(\w+)"?$"#);
    for (_, _, cmd, raw) in statements(text) {
        let a = args(&raw);
        match cmd.as_str() {
            "if" | "elseif" => {
                arch = Some(
                    pat.captures(raw.trim())
                        .map(|c| c[1].to_owned())
                        .unwrap_or("?".into()),
                )
            }
            "else" => arch = Some("?".into()),
            "endif" => arch = None,
            "set" if a.first().is_some_and(|x| x == var) => {
                if a.len() != 2 || arch.as_deref().is_none_or(|a| a == "?") {
                    return BTreeMap::new();
                }
                out.insert(arch.clone().unwrap(), a[1].trim_matches('"').into());
            }
            _ => {}
        }
    }
    out
}
fn cargo_commands(text: &str) -> Vec<String> {
    statements(text)
        .into_iter()
        .filter(|(_, _, cmd, raw)| {
            cmd == "add_custom_command" && re(r"\bcargo build\b").is_match(raw)
        })
        .map(|(_, _, _, raw)| re(r"#[^\n]*").replace_all(&raw, "").into_owned())
        .collect()
}
/// The one custom `cargo build` body of a module that has exactly one.
pub fn custom_body(text: &str) -> Option<serde_json::Value> {
    let cmds = cargo_commands(text);
    if cmds.len() != 1 {
        return None;
    }
    custom_body_command(text, &cmds[0])
}
/// A module with SEVERAL custom `cargo build` commands (kernel32: kernel32-rust plus the
/// vista and shared archives kernel32_vista also links; lsasrv/samsrv: the body plus a
/// stub crate). The body is the command whose archive is `lib<target>_rust.a`; every
/// other one must be a plain `cargo build -p <crate>` (no RUSTFLAGS, no features), which
/// the engine repeats as `engine:crate:<crate>::TRUE`. The bare cmake targets in their
/// DEPENDS (native-widl, lsasrv_c) order the cell. Those commands stay in the CMakeLists
/// (another module may link their archive), and so do the variables they use.
pub fn custom_body_for(text: &str, target: &str) -> Option<serde_json::Value> {
    let cmds = cargo_commands(text);
    if cmds.len() < 2 {
        return custom_body(text);
    }
    let archive = re(r"/release/(lib\w+\.a)\s");
    let own = format!("lib{target}_rust.a");
    let (body, rest): (Vec<_>, Vec<_>) = cmds
        .iter()
        .partition(|raw| archive.captures(raw).is_some_and(|c| c[1] == own));
    if body.len() != 1 {
        return None;
    }
    let mut v = custom_body_command(text, body[0])?;
    let mut extra = Vec::new();
    let mut deps = Vec::new();
    for raw in &rest {
        if raw.contains("RUSTFLAGS=") || raw.contains("--features") || raw.contains("--no-default")
        {
            return None;
        }
        let krate = re(r"\s-p\s+([\w-]+)").captures(raw)?[1].to_owned();
        let ar = archive.captures(raw)?[1].to_owned();
        extra.push(serde_json::json!({"crate": krate, "archive": ar}));
        deps.extend(depends_targets(raw));
    }
    // Keep every variable a remaining command still reads.
    let used: Vec<String> = rest
        .iter()
        .flat_map(|r| {
            re(r"\$\{(\w+)\}")
                .captures_iter(r)
                .map(|c| c[1].to_owned())
                .collect::<Vec<_>>()
        })
        .collect();
    let vars: Vec<serde_json::Value> = v["vars"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|x| x.as_str().is_some_and(|n| !used.iter().any(|u| u == n)))
        .collect();
    v["vars"] = serde_json::json!(vars);
    v["extra"] = serde_json::json!(extra);
    let mut all: Vec<String> = v["depends"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    for d in deps {
        if !all.contains(&d) {
            all.push(d);
        }
    }
    v["depends"] = serde_json::json!(all);
    Some(v)
}
/// The bare cmake targets among a custom command's DEPENDS (not paths, not variables).
fn depends_targets(raw: &str) -> Vec<String> {
    let Some(rest) = raw.split_once("DEPENDS") else {
        return Vec::new();
    };
    rest.1
        .split_whitespace()
        .take_while(|t| !["WORKING_DIRECTORY", "COMMENT", "VERBATIM", "BYPRODUCTS"].contains(t))
        .filter(|t| !t.contains('/') && !t.contains("${") && !t.contains('.'))
        .map(str::to_owned)
        .collect()
}
/// oleaut32's form: `-E env ${X_ENV} cargo build ...`, with `set(X_ENV
/// CARGO_TARGET_DIR=${D})` and, under `if(ARCH STREQUAL "<a>")`, `list(APPEND X_ENV
/// "RUSTFLAGS=...")`. Returns the env variable, the command with the variable expanded to
/// its CARGO_TARGET_DIR, and the per-arch RUSTFLAGS; None for any other use of the variable.
fn env_list(text: &str, raw: &str) -> Option<(String, String, BTreeMap<String, String>)> {
    let var = re(r"-E\s+env\s+\$\{(\w+)\}\s+cargo\s+build").captures(raw)?[1].to_owned();
    let v = regex::escape(&var);
    let set = re(&format!(
        r"(?m)^[ \t]*set\(\s*{v}\s+CARGO_TARGET_DIR=\$\{{(\w+)\}}\s*\)[ \t]*$"
    ));
    let dir = set.captures(text)?[1].to_owned();
    let append = re(&format!(
        r#"(?m)^[ \t]*if\(ARCH STREQUAL "(\w+)"\)\n[ \t]*list\(APPEND {v} "RUSTFLAGS=([^"]*)"\)\n[ \t]*endif\(\)"#
    ));
    let mut flags = BTreeMap::new();
    for c in append.captures_iter(text) {
        flags.insert(c[1].to_owned(), c[2].to_owned());
    }
    // Every other mention of the variable is unmodelled: the set, the appends and the
    // command's own use are all there may be.
    let uses = re(&format!(r"\b{v}\b")).find_iter(text).count();
    if uses != 1 + flags.len() + 1 {
        return None;
    }
    let expanded = raw.replacen(
        &format!("${{{var}}}"),
        &format!("CARGO_TARGET_DIR=${{{dir}}}"),
        1,
    );
    Some((var, expanded, flags))
}
fn custom_body_command(text: &str, raw: &str) -> Option<serde_json::Value> {
    let env = env_list(text, raw);
    if env.is_none() && re(r"-E\s+env\s+\$\{").is_match(raw) {
        return None;
    }
    let raw = env
        .as_ref()
        .map(|e| e.1.clone())
        .unwrap_or_else(|| raw.to_owned());
    let pattern = re(
        r#"(?x)^\s*OUTPUT\s+\$\{(\w+)\}\s+COMMAND\s+\$\{CMAKE_COMMAND\}\s+-E\s+env\s+CARGO_TARGET_DIR=\$\{(\w+)\}\s+(?:RUSTFLAGS=\$\{(\w+)\}\s+)?cargo\s+build\s+--release\s+--target\s+\$\{(\w+)\}\s+-p\s+([\w-]+)\s+--manifest-path\s+\$\{(\w+)\}/Cargo\.toml\s+COMMAND\s+\$\{CMAKE_COMMAND\}\s+-E\s+copy(?:_if_different)?\s+\$\{(\w+)\}/\$\{(\w+)\}/release/(lib\w+\.a)\s+\$\{(\w+)\}\s+DEPENDS\s+\$\{(\w+)\}\s+\$\{(\w+)\}/([\w-]+)/Cargo\.toml(?P<more>(?:\s+[^\s]+?)*?)\s+WORKING_DIRECTORY\s+\$\{(\w+)\}\s+COMMENT\s+"[^"]*"(?:\s+VERBATIM)?\s*$"#,
    );
    let c = pattern.captures(&raw)?;
    if c[2] != c[7]
        || c[4] != c[8]
        || c[1] != c[10]
        || c[6] != c[12]
        || c[5] != c[13]
        || c[6] != c[15]
    {
        return None;
    }
    let targets = arch_values(text, &c[4]);
    // RUSTFLAGS is optional: most cohab DLLs pass none. When the command does pass a
    // variable, every arch must set it, as before.
    let flags_var = c.get(3).map(|m| m.as_str());
    let flags = flags_var.map(|v| arch_values(text, v)).unwrap_or_default();
    for arch in ARCHES {
        if targets.get(arch).map(String::as_str)
            != super::constants()["TARGET"]
                .get(arch)
                .and_then(|v| v.as_str())
            || (flags_var.is_some() && !flags.contains_key(arch))
        {
            return None;
        }
    }
    if targets.len() != ARCHES.len() || (flags_var.is_some() && flags.len() != ARCHES.len()) {
        return None;
    }
    let output = re(&format!(
        r#"set\({} "\$\{{CMAKE_CURRENT_BINARY_DIR\}}/{}"\)"#,
        regex::escape(&c[1]),
        regex::escape(&c[9])
    ));
    if !output.is_match(text) {
        return None;
    }
    let target_re = re(&format!(
        r"add_custom_target\((\w+) DEPENDS \$\{{{}\}}\)",
        regex::escape(&c[1])
    ));
    let custom = target_re.captures(text)?;
    let mut vars: Vec<_> = [c.get(1), c.get(2), c.get(3), c.get(4), c.get(6), c.get(11)]
        .into_iter()
        .flatten()
        .map(|m| m.as_str())
        .collect();
    let mut flags = flags;
    if let Some((var, _, env_flags)) = &env {
        vars.push(var.as_str());
        flags = env_flags.clone();
    }
    Some(
        serde_json::json!({"crate":&c[5],"archive":&c[9],"rustflags":flags.into_iter().filter(|(_,v)|!v.is_empty()).collect::<BTreeMap<_,_>>(),"vars":vars,"output_var":&c[1],"custom_target":&custom[1],"depends":depends_targets(&format!("DEPENDS {}", &c["more"]))}),
    )
}
pub fn placeholder_archive(root: &Path, arch: &str, lib: &str) -> Result<bool> {
    let pattern = re(r"(?:^|/)lib(\w+)\.a$");
    let Some(m) = pattern.captures(lib) else {
        return Ok(false);
    };
    let marker = format!("CMakeFiles/{}.dir/", &m[1]);
    let db = super::compile_db(root, arch)?;
    let mut files = Vec::new();
    for row in db {
        if row["command"].as_str().is_some_and(|c| c.contains(&marker)) {
            if let Some(file) = row["file"].as_str() {
                files.push(file.to_owned());
            }
        }
    }
    if files.is_empty() {
        return Ok(false);
    }
    for file in files {
        if !file.ends_with(".c") {
            return Ok(false);
        }
        let path = if let Some(rel) = file.strip_prefix("/tmp/vibeos/") {
            root.join(rel)
        } else {
            PathBuf::from(file)
        };
        if !placeholder(&local(root, &path)?)? {
            return Ok(false);
        }
    }
    Ok(true)
}
pub fn links_delayimp(static_libs: &[String]) -> bool {
    static_libs
        .iter()
        .any(|s| re(r"/libdelayimp(_rust)?\.a$|^engine:delayimp(_rust)?$").is_match(s))
}
fn strings(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
fn rel(root: &Path, p: &Path) -> Result<String> {
    Ok(local(root, p)?
        .strip_prefix(root.canonicalize().map_err(|e| Error::new(e.to_string()))?)
        .map_err(|e| Error::new(e.to_string()))?
        .to_string_lossy()
        .into_owned())
}
pub fn model(root: &Path, target: &str) -> Result<super::Cell> {
    use std::collections::BTreeSet;
    let links = link_edges_gated(root, target)?;
    let arches: Vec<&'static str> = ARCHES
        .into_iter()
        .filter(|a| links.contains_key(*a))
        .collect();
    // amd64 is the reference arch whenever the module is built there.
    let primary = if links.contains_key("amd64") {
        "amd64"
    } else {
        arches[0]
    };
    let first = &links[primary];
    let cmake_dir = source_dir(root, primary, &first.output, target)?;
    for a in arches.iter().copied() {
        if source_dir(root, a, &links[a].output, target)? != cmake_dir {
            return Err(Error::new(format!(
                "{target}: the source directory differs by arch"
            )));
        }
    }
    let cmake = cmake_dir.join("CMakeLists.txt");
    let refs = target_file_refs(root)?;
    let others: Vec<_> = refs
        .get(target)
        .into_iter()
        .flatten()
        .filter(|p| root.join(p) != cmake)
        .collect();
    if !others.is_empty() {
        return Err(Error::new(format!(
            "{target}: other CMakeLists read its file via $<TARGET_FILE...>: {others:?}"
        )));
    }
    let facts = cmake_facts(&cmake, target, &first.libs)?;
    let cmtext = super::fncall::cmake_text(&cmake, target)?;
    let custom = &facts["custom_body"];
    let bodyless = facts["body"].is_null() && custom.is_null();
    if bodyless {
        if re(r"\bcargo build\b").is_match(&cmtext) {
            return Err(Error::new(format!("{target}: no rust_staticlib_variant names its body crate, and its custom `cargo build` command is not the one shape _custom_body reads")));
        }
        let own = re(&format!(
            r"(?i)(?:^|/)lib{}\w*_rust\.a$",
            regex::escape(target)
        ));
        if links
            .values()
            .flat_map(|l| &l.libs)
            .any(|l| own.is_match(l))
        {
            return Err(Error::new(format!(
                "{target}: links its own Rust archive, but no body is declared"
            )));
        }
    }
    if facts["unknown"].as_array().is_some_and(|v| !v.is_empty()) {
        return Err(Error::new(format!(
            "{target}: unmodelled statements on the target: {}",
            facts["unknown"]
        )));
    }
    check_statement_effects(target, &facts, &links)?;
    if facts["ssp"] == true {
        for a in arches.iter().copied() {
            let p = local(root, &root.join(format!("build-{a}/CMakeCache.txt")))?;
            if re(r"(?m)^STACK_PROTECTOR:BOOL=(ON|TRUE|1)$").is_match(&read(&p)?) {
                return Err(Error::new(format!(
                    "{target}: STACK_PROTECTOR is on and links gcc_ssp objects"
                )));
            }
        }
    }
    let subsys = first.subsystem.as_deref().unwrap_or("");
    let kind = match first.kind.as_str() {
        "C_EXECUTABLE" => {
            if !["windows", "console", "native"]
                .iter()
                .any(|x| subsys.starts_with(x))
            {
                return Err(Error::new(format!(
                    "{target}: {subsys} exe is not modelled"
                )));
            }
            "Win32Exe"
        }
        "C_MODULE_LIBRARY" | "C_SHARED_LIBRARY" => {
            if subsys.starts_with("native") {
                let typ = facts["module_type"].as_str().unwrap_or("");
                if !const_contains("KERNEL_KINDS", typ) {
                    return Err(Error::new(format!(
                        "{target}: kernel-mode module type {typ} is not modelled"
                    )));
                }
                "KernelDll"
            } else {
                "Win32Dll"
            }
        }
        x => return Err(Error::new(format!("{target}: link kind {x}"))),
    };
    // A custom `cargo build` body is the same body whatever links it: the user-mode cohab
    // DLLs (advapi32, user32, ws2_32, ...) build theirs exactly as the drivers do.
    let body_ar = if !custom.is_null() {
        custom["archive"].as_str().map(str::to_owned)
    } else if bodyless {
        None
    } else {
        Some(format!("lib{}.a", facts["body_var"].as_str().unwrap_or("")))
    };
    let specs = spec_index(root)?;
    let index = library_index(root)?;
    type Per = (
        Vec<(String, String, String)>,
        Vec<String>,
        Option<String>,
        Vec<(String, String, String)>,
        Vec<(String, bool)>,
    );
    let mut per: BTreeMap<String, Per> = BTreeMap::new();
    let mut orders = BTreeMap::new();
    let mut late = BTreeMap::new();
    for a in arches.iter().copied() {
        let mut imports = Vec::new();
        let mut delayed = Vec::new();
        let mut static_libs = Vec::new();
        let mut crt = None;
        let mut order = Vec::new();
        let mut seq = Vec::new();
        let mut late_a = Vec::new();
        let mut after = false;
        for raw in &links[a].libs {
            let name = raw.rsplit('/').next().unwrap_or(raw);
            if Some(name) == body_ar.as_deref() {
                after = true;
                if !seq.contains(&"body".to_string()) {
                    seq.push("body".into());
                }
                continue;
            }
            if raw.contains("libclang_rt") {
                continue;
            }
            if let Some(c) = re(r"^libmsvcrtex_(\w+)\.a$").captures(name) {
                // A DLL links TWO: libmsvcrtex_dll.a (the startup) and libmsvcrtex_dllmain.a
                // (the default DllMain, msvcrtex.cmake). Only the startup names a kind, and
                // an unmapped companion must not clobber it: dinput's link listed dllmain
                // last, so the cell came out with no CRT startup and failed to link
                // (`undefined symbol: _DllMainCRTStartup@12`).
                if let Some(kind) = super::constants()["MSVCRTEX"]
                    .get(&c[1])
                    .and_then(|v| v.as_str())
                {
                    crt = Some(kind.to_owned());
                }
                if bodyless && !seq.contains(&"body".to_string()) {
                    seq.push("body".into());
                }
                continue;
            }
            let delay = re(r"CMakeFiles/lib(\w+)_delayed\.dir/lib(\w+)_delayed\.a$")
                .captures(raw)
                .filter(|c| c[1] == c[2]);
            let normal = re(r"CMakeFiles/lib(\w+)\.dir/lib(\w+)\.a$")
                .captures(raw)
                .filter(|c| c[1] == c[2]);
            if let Some(c) = delay.or(normal) {
                let is_delay = raw.ends_with("_delayed.a");
                let base = c[1].to_lowercase();
                let (spec, dll) = specs.get(&base).and_then(|v| v.as_ref()).ok_or_else(|| {
                    Error::new(format!(
                        "{target}: {} {base}: no single spec2def --implib",
                        if is_delay {
                            "delay import"
                        } else {
                            "import lib"
                        }
                    ))
                })?;
                let item = (spec.clone(), dll.clone(), base.clone());
                let list = if is_delay { &mut delayed } else { &mut imports };
                if !list.contains(&item) {
                    list.push(item);
                    order.push((base.clone(), is_delay));
                    let token = format!("import:{base}");
                    if is_delay && seq.contains(&token) {
                        return Err(Error::new(format!(
                            "{target}: {base} is both a delay and a regular import"
                        )));
                    }
                    seq.push(token);
                }
                continue;
            }
            if super::constants()["ENGINE_LIBS"].get(raw).is_none()
                && placeholder_archive(root, a, raw)?
            {
                continue;
            }
            // An archive of one of the module's other custom `cargo build` commands
            // (kernel32's vista/shared, lsasrv's lsa-stub): the engine builds that crate.
            let extra_crate = custom["extra"].as_array().and_then(|xs| {
                xs.iter().find_map(|x| {
                    let ar = x["archive"].as_str()?;
                    (raw.rsplit('/').next() == Some(ar)).then(|| {
                        format!("engine:crate:{}::TRUE", x["crate"].as_str().unwrap_or(""))
                    })
                })
            });
            let lib = extra_crate.unwrap_or_else(|| {
                engine_lib(raw, &index)
                    .map(|x| x.0)
                    .unwrap_or_else(|| raw.clone())
            });
            if !static_libs.contains(&lib) {
                static_libs.push(lib.clone());
                seq.push(lib.clone());
                if after {
                    late_a.push(lib);
                }
            }
        }
        if !after && !bodyless {
            return Err(Error::new(format!(
                "{target}: {a} link has no {}",
                body_ar.as_deref().unwrap_or("")
            )));
        }
        if !delayed.is_empty() && !links_delayimp(&static_libs) {
            return Err(Error::new(format!(
                "{target}: delay imports without the delayimp runtime"
            )));
        }
        orders.insert(a.to_owned(), seq);
        late.insert(a.to_owned(), late_a);
        per.insert(a.into(), (imports, static_libs, crt, delayed, order));
    }
    let main = &per[primary];
    if per
        .values()
        .any(|v| v.0 != main.0 || v.2 != main.2 || v.3 != main.3 || v.4 != main.4)
    {
        return Err(Error::new(format!(
            "{target}: imports or CRT differ by arch (not modelled)"
        )));
    }
    let mut cell = super::Cell::default();
    cell.target = target.into();
    cell.cmake_dir = cmake_dir.clone();
    cell.output_name = Path::new(&first.output)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    cell.module_kind = kind.into();
    if arches.len() != ARCHES.len() {
        cell.arches = arches.iter().map(|a| a.to_string()).collect();
    }
    cell.entries = links
        .iter()
        .map(|(a, l)| (a.clone(), l.entry.clone()))
        .collect();
    cell.subsystem = first.subsystem.clone();
    cell.image_base = links
        .iter()
        .map(|(a, l)| (a.clone(), l.image_base))
        .collect();
    cell.import_libs = main.0.clone();
    cell.msvcrtex = main.2.clone();
    cell.delay_import_libs = main.3.clone();
    if !main.3.is_empty() {
        cell.import_order = main.4.clone();
    }
    cell.static_libs = per.iter().map(|(a, v)| (a.clone(), v.1.clone())).collect();
    cell.late_libs = late;
    cell.link_order = orders;
    let stacks: BTreeSet<_> = links.values().map(|l| l.stack).collect();
    if stacks.len() > 1 {
        return Err(Error::new(format!(
            "{target}: stack reserve differs by arch {stacks:?}"
        )));
    }
    cell.stack = first.stack;
    if let Some(body) = facts["body"].as_array() {
        cell.body_crate = body[0].as_str().unwrap_or("").into();
        cell.body_features = strings(&body[1]);
        cell.body_defaults = body[2].as_bool().unwrap_or(true);
    }
    cell.body_var = facts["body_var"].as_str().unwrap_or("").into();
    cell.body_rustflags = serde_json::from_value(facts["body_rustflags"].clone())?;
    cell.body_env_block = facts["body_env_block"].as_str().unwrap_or("").into();
    cell.body_depends = strings(&facts["body_depends"]);
    cell.custom_body = if custom.is_null() {
        serde_json::json!({})
    } else {
        custom.clone()
    };
    let by_arch: BTreeMap<_, Vec<_>> = links
        .iter()
        .map(|(a, l)| (a.clone(), l.inputs.iter().map(|o| obj_key(o)).collect()))
        .collect();
    let common: BTreeSet<String> = by_arch[primary]
        .iter()
        .filter(|n| arches.iter().all(|a| by_arch[*a].contains(n)))
        .cloned()
        .collect();
    let mut arch_shims: BTreeMap<String, Vec<String>> =
        arches.iter().map(|a| (a.to_string(), Vec::new())).collect();
    for a in arches.iter().copied() {
        for obj in &links[a].inputs {
            let name = obj.rsplit('/').next().unwrap_or(obj);
            if common.contains(&obj_key(obj)) {
                continue;
            }
            let src = obj_source(&cmake_dir, obj);
            if !re(r"\.(c|s|S)\.obj$").is_match(name) || !src.exists() {
                return Err(Error::new(format!(
                    "{target}: link inputs differ by arch {by_arch:?}"
                )));
            }
            arch_shims.get_mut(a).unwrap().push(rel(root, &src)?);
        }
    }
    let mut rc_obj = String::new();
    let bin_dir = Path::new(&first.output).parent().unwrap_or(Path::new(""));
    for obj in &first.inputs {
        let name = obj.rsplit('/').next().unwrap_or(obj);
        if !common.contains(&obj_key(obj)) {
            continue;
        }
        if name.ends_with(".rc.res") {
            let rc = super::recipes::rc_command(primary, obj, root)?.0;
            if cell.rc.is_none() {
                cell.rc = Some(rc);
                rc_obj = obj.clone();
            } else {
                // A second .rc was silently dropped before (the last one won): wined3d lost
                // its HLSL RCDATA. Each must share the first one's flags, which the cell
                // renders once.
                if super::recipes::rc_flags_for(&rc, obj, root)?
                    != super::recipes::rc_flags_for(cell.rc.as_ref().unwrap(), &rc_obj, root)?
                {
                    return Err(Error::new(format!(
                        "{target}: {rc} is compiled with different flags from {}",
                        cell.rc.as_ref().unwrap()
                    )));
                }
                cell.extra_rc.push(rc);
            }
        } else if name.ends_with("_stubs.rs.obj") || name.ends_with("_stubs.c.obj") {
            cell.spec_stubs = true;
        } else if strings(&facts["reg_objects"]).contains(&name.to_owned()) {
            for a in arches.iter().copied() {
                cell.static_libs.get_mut(a).unwrap().insert(0, obj.clone());
                cell.link_order.get_mut(a).unwrap().insert(0, obj.clone());
            }
        } else if name.ends_with(".def") {
            let pat = re(r"spec2def\(\s*(\S+)\s+(\S+\.spec)");
            let caps: Vec<_> = pat.captures_iter(&cmtext).collect();
            // Several spec2def() calls are if()/else() branches (uxtheme: uxtheme_vista.spec
            // when DLL_EXPORT_VERSION >= 0x600, else uxtheme.spec). The first in the text was
            // taken, so the cell exported a spec this configuration does not build. Keep the
            // one this build's spec2def command reads.
            let cap = if caps.len() > 1 {
                let ninja = read(&local(root, &root.join(format!("build-{primary}/build.ninja")))?)?;
                let used: Vec<_> = caps
                    .iter()
                    .filter(|c| {
                        let spec = rel(root, &cmake_dir.join(&c[2])).unwrap_or_default();
                        ninja.contains(&format!("CUSTOM_COMMAND /tmp/vibeos/{spec} "))
                    })
                    .collect();
                if used.len() != 1 {
                    return Err(Error::new(format!(
                        "{target}: {} spec2def() calls and {} of their specs in build.ninja",
                        caps.len(),
                        used.len()
                    )));
                }
                used[0]
            } else {
                caps.first()
                    .ok_or_else(|| Error::new(format!("{target}: a .def without a spec2def()")))?
            };
            cell.spec = Some((rel(root, &cmake_dir.join(&cap[2]))?, cap[1].into()));
        } else if name.ends_with(".c.obj") || re(r"\.(cpp|cxx|cc)\.obj$").is_match(name) {
            // C++ compiles the same way: the cell replays cmake's compile command through a
            // clib recipe, whose `cxx` lines reactos-link runs with the cxx.<arch>.flags template.
            let mut src = obj_source(&cmake_dir, obj);
            if let Some(m) = re(r"\.dir/((?:__/)+.*)\.obj$").captures(obj) {
                src = cmake_dir.join(m[1].replace("__/", "../"));
                cell.foreign.push(rel(root, &src)?);
            }
            if !src.exists() {
                let generated = bin_dir.join(name.trim_end_matches(".obj"));
                for a in arches.iter().copied() {
                    local(root, &root.join(format!("build-{a}")).join(&generated)).map_err(
                        |_| {
                            Error::new(format!(
                                "{target}: compiled C {name} found in no source or build dir"
                            ))
                        },
                    )?;
                }
                let p = generated.to_string_lossy().into_owned();
                cell.c_shims.push(p.clone());
                cell.generated.push(p);
                continue;
            }
            if name == "rust_cohab_stub.c.obj" && placeholder(&local(root, &src)?)? {
                continue;
            }
            cell.c_shims.push(rel(root, &src)?);
        } else if re(r"\.(cpp|cxx|cc)\.obj$").is_match(name) {
            let src = obj_source(&cmake_dir, obj);
            if !src.exists() || !placeholder(&local(root, &src)?)? {
                return Err(Error::new(format!("{target}: compiled C++ {name} (reactos-link compiles C and asm shims only; cflags/ has no C++ template)")));
            }
        } else {
            return Err(Error::new(format!(
                "{target}: unmodelled link input {name}"
            )));
        }
    }
    if arch_shims.values().any(|v| !v.is_empty()) {
        cell.arch_shims = arch_shims;
    }
    if facts["unicode"] == true {
        cell.rc_defines = vec!["UNICODE".into(), "_UNICODE".into()];
    }
    if kind == "Win32Exe" && cell.msvcrtex.is_none() && !subsys.starts_with("native") {
        return Err(Error::new(format!(
            "{target}: exe without a msvcrtex CRT startup"
        )));
    }
    cell.destinations = strings(&facts["destinations"]);
    let mut shipped = BTreeMap::new();
    for a in arches.iter().copied() {
        let dirs = cd_dirs(root, a, &links[a].output)?;
        if cell.destinations.is_empty() {
            if !dirs.is_empty() {
                return Err(Error::new(format!(
                    "{target}: on the live CD at {dirs:?} with no FOR all/livecd"
                )));
            }
            shipped.insert(a.to_owned(), Vec::new());
            continue;
        }
        let mut matched = BTreeSet::new();
        for dir in &dirs {
            let mut hits = Vec::new();
            for dest in &cell.destinations {
                let mut pattern = String::from("^");
                let var = re(r"\$\{\w+\}");
                let mut last = 0;
                for m in var.find_iter(dest) {
                    pattern.push_str(&regex::escape(&dest[last..m.start()]));
                    pattern.push_str("[^/]+");
                    last = m.end();
                }
                pattern.push_str(&regex::escape(&dest[last..]));
                pattern.push('$');
                if re(&pattern).is_match(dir) {
                    hits.push(dest);
                }
            }
            if hits.len() != 1 {
                return Err(Error::new(format!("{target}: CD destinations differ by arch ({a} ships to {dirs:?}, the CMakeLists names {:?})",cell.destinations)));
            }
            matched.insert(hits[0].clone());
        }
        if dirs.is_empty() {
            return Err(Error::new(format!(
                "{target}: CD destinations differ by arch ({a} ships nowhere)"
            )));
        }
        shipped.insert(
            a.to_owned(),
            cell.destinations
                .iter()
                .filter(|v| matched.contains(*v))
                .cloned()
                .collect::<Vec<_>>(),
        );
    }
    if shipped.values().any(|v| v != &cell.destinations) {
        let all: BTreeSet<_> = shipped.values().flatten().cloned().collect();
        if all != cell.destinations.iter().cloned().collect() {
            return Err(Error::new(format!(
                "{target}: CD destinations {shipped:?} leave some unused"
            )));
        }
        cell.arch_destinations = shipped;
    }
    // A body-less module with no C of its own is still a module: its .def (forwarders,
    // data exports), its .res and the static libraries it links (comcat's
    // wine_dll_register) are the whole link. Such a cell is shell-less: rustc's bin
    // (its panic handler, which calls DbgPrint and NtTerminateProcess, and i386's
    // rsbegin/rsend) stays out of the link, so a module with no code gains no `.text`
    // and one importing no ntdll (comcat, msxml3r, wmi) still links. --convert's
    // pe_equiv check against cmake's link is the guard.
    if bodyless && cell.c_shims.is_empty() && cell.arch_shims.values().all(Vec::is_empty) {
        cell.shell_less = true;
    }
    // A Rust body cmake linked under -Wl,--whole-archive (cdrom) brings every member of
    // its staticlib, core and its panic handler included: rustc's bin would define all of
    // them a second time (537 duplicates, which link-guard refuses). cmake's link had no
    // shell, so neither does the cell.
    if cell.link_order.values().any(|order| {
        order
            .windows(2)
            .any(|w| w[0] == "-Wl,--whole-archive" && w[1] == "body")
    }) {
        cell.shell_less = true;
    }
    let subs: BTreeMap<_, _> = links
        .iter()
        .map(|(a, l)| (a.clone(), l.subsystem.clone().unwrap_or_default()))
        .collect();
    let unique: BTreeSet<_> = subs.values().collect();
    if unique.len() > 1 {
        if subs.values().any(String::is_empty) {
            return Err(Error::new(format!(
                "{target}: subsystem set on some arches only {subs:?}"
            )));
        }
        cell.subsystems = subs;
    }
    let mut panics = BTreeMap::new();
    for a in arches.iter().copied() {
        let pe = local(
            root,
            &root.join(format!("build-{a}")).join(&links[a].output),
        )?;
        panics.insert(a.to_owned(), super::pe::carries_panic_handler(&pe)?);
        let mut bound = super::pe::exports_bound_to_imports(&pe)?;
        if let Some((spec, _)) = &cell.spec {
            for sym in spec_imported_exports(&local(root, &root.join(spec))?)? {
                bound.remove(&sym);
            }
        }
        let permitted = super::constants()["REBOUND_EXPORTS"]
            .get(target)
            .map(strings)
            .unwrap_or_default();
        if bound.iter().any(|x| !permitted.contains(x)) {
            return Err(Error::new(format!(
                "{target}: {a} exports resolve to import thunks {bound:?}"
            )));
        }
    }
    if kind == "KernelDll" {
        if panics.values().collect::<BTreeSet<_>>().len() > 1 {
            return Err(Error::new(format!(
                "{target}: the body's panic handler on some arches only {panics:?}"
            )));
        }
        if links.values().any(|l| l.image_base.is_none()) {
            return Err(Error::new(format!(
                "{target}: a kernel-mode link without --image-base"
            )));
        }
        cell.kernel_kind = facts["module_type"].as_str().unwrap_or("").into();
    }
    cell.panics = panics;
    if super::constants()["REBOUND_EXPORTS"].get(target).is_some() {
        for order in cell.link_order.values_mut() {
            order.retain(|v| v != "body");
            order.insert(0, "body".into());
        }
    }
    if let Some(rc) = &cell.rc {
        cell.rc_flags = super::recipes::rc_flags_for(rc, &rc_obj, root)?;
    }
    cell.name_on_cd = facts["name_on_cd"].as_str().map(str::to_owned);
    cell.extra_cd = strings(&facts["extra_cd"]);
    cell.no_cab = facts["no_cab"] == true;
    if facts["require_defined"] == true {
        cell.require_defined = links
            .iter()
            .map(|(a, l)| (a.clone(), l.require_defined.clone()))
            .collect();
    }
    Ok(cell)
}
pub fn spec_imported_exports(path: &Path) -> Result<std::collections::BTreeSet<String>> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Default::default()),
        Err(e) => return Err(Error::new(format!("{}: {e}", path.display()))),
    };
    let mut out = std::collections::BTreeSet::new();
    for c in re(r"(?m)^\s*(?:\d+|@)\s+\w+\s+(?:-\S+\s+)*?-import\b(?:\s+-\S+)*\s+(\w+)")
        .captures_iter(&text)
    {
        out.insert(c[1].to_owned());
    }
    for c in
        re(r"(?m)^\s*(?:\d+|@)\s+\w+\s+(?:-\S+\s+)*(\w+)\([^)]*\)\s+(\w+)\s*$").captures_iter(&text)
    {
        if c[1] == c[2] {
            out.insert(c[1].to_owned());
        }
    }
    Ok(out)
}

/// Resolve the object path under its target directory, preserving source subdirectories.
pub fn obj_source(cmake_dir: &Path, obj: &str) -> PathBuf {
    let path = re(r"/CMakeFiles/[^/]+\.dir/(.+)\.obj$")
        .captures(obj)
        .map(|c| c[1].to_owned())
        .unwrap_or_else(|| {
            obj.rsplit('/')
                .next()
                .unwrap_or(obj)
                .trim_end_matches(".obj")
                .to_owned()
        });
    cmake_dir.join(path)
}
pub fn shim_name(root: &Path, source: &str, cell: &super::Cell) -> String {
    let dir = cell.cmake_dir.strip_prefix(root).ok();
    dir.and_then(|d| Path::new(source).strip_prefix(d).ok())
        .unwrap_or_else(|| {
            Path::new(source)
                .file_name()
                .map(Path::new)
                .unwrap_or(Path::new(source))
        })
        .to_string_lossy()
        .into_owned()
}
pub fn target_file_refs(root: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let pattern = re(r"\$<TARGET_(?:LINKER_)?FILE(?:_DIR|_NAME)?:([A-Za-z0-9_.+-]+)>");
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in cmake_files(root)? {
        let text = read(&path)?;
        // A target's own `add_custom_command(TARGET t POST_BUILD ... $<TARGET_FILE:t>)` is
        // not a consumer: a file holding one defines `t` itself. legide-rust's stale
        // cohab/CMakeLists.txt, which nothing includes, kept legide from converting.
        let own: Vec<(usize, usize, String)> = statements(&text)
            .into_iter()
            .filter(|(_, _, cmd, _)| cmd == "add_custom_command")
            .filter_map(|(s, e, _, raw)| {
                let a = args(&raw);
                (a.len() > 2 && a[0] == "TARGET" && a[2] == "POST_BUILD")
                    .then(|| (s, e, a[1].clone()))
            })
            .collect();
        for cap in pattern.captures_iter(&text) {
            let at = cap.get(0).unwrap().start();
            if own.iter().any(|(s, e, t)| (*s..*e).contains(&at) && *t == cap[1]) {
                continue;
            }
            let files = out.entry(cap[1].to_owned()).or_default();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if !files.contains(&relative) {
                files.push(relative);
            }
        }
    }
    Ok(out)
}
pub fn list_vars(text: &str) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for (_, _, cmd, raw) in statements(text) {
        let a = args(&raw);
        if cmd == "set" && !a.is_empty() {
            out.insert(a[0].clone(), a[1..].to_vec());
        } else if cmd == "list" && a.len() > 1 && a[0] == "APPEND" {
            out.entry(a[1].clone())
                .or_insert_with(Vec::new)
                .extend_from_slice(&a[2..]);
        }
    }
    out
}
pub fn expand(items: &[String], lists: &BTreeMap<String, Vec<String>>) -> Result<Vec<String>> {
    fn visit(
        items: &[String],
        lists: &BTreeMap<String, Vec<String>>,
        stack: &mut Vec<String>,
        out: &mut Vec<String>,
    ) -> Result<()> {
        for item in items {
            let name = item.strip_prefix("${").and_then(|s| s.strip_suffix('}'));
            if let Some((name, values)) = name.and_then(|n| lists.get(n).map(|v| (n, v))) {
                if stack.iter().any(|s| s == name) || stack.len() >= 128 {
                    return Err(Error::new(format!("recursive CMake list variable: {name}")));
                }
                stack.push(name.to_owned());
                visit(values, lists, stack, out)?;
                stack.pop();
            } else {
                out.push(item.clone());
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(items, lists, &mut Vec::new(), &mut out)?;
    Ok(out)
}

#[cfg(test)]
mod multi_body_tests {
    use super::{custom_body_for, obj_key};

    fn cmd(var: &str, krate: &str, lib: &str, more: &str) -> String {
        format!(
            "rust_storage_target({var}_DIR \"${{CMAKE_CURRENT_BINARY_DIR}}/t\" {krate})\n\
             set({var}_LIB \"${{CMAKE_CURRENT_BINARY_DIR}}/{lib}\")\n\
             add_custom_command(\n    OUTPUT ${{{var}_LIB}}\n    COMMAND ${{CMAKE_COMMAND}} -E env\n\
             \x20       CARGO_TARGET_DIR=${{{var}_DIR}}\n\
             \x20       cargo build --release --target ${{M_TARGET}}\n            -p {krate}\n\
             \x20           --manifest-path ${{M_WS}}/Cargo.toml\n\
             \x20   COMMAND ${{CMAKE_COMMAND}} -E copy\n\
             \x20       ${{{var}_DIR}}/${{M_TARGET}}/release/{lib}\n        ${{{var}_LIB}}\n\
             \x20   DEPENDS ${{{var}_SRCS}}\n            ${{M_WS}}/{krate}/Cargo.toml{more}\n\
             \x20   WORKING_DIRECTORY ${{M_WS}}\n    COMMENT \"Building {krate}\"\n    VERBATIM)\n\
             add_custom_target(rust_{var} DEPENDS ${{{var}_LIB}})\n"
        )
    }

    fn module() -> String {
        let mut t = String::from(
            "set(M_WS \"${CMAKE_SOURCE_DIR}/sdk/lib/rust\")\n\
             if(ARCH STREQUAL \"i386\")\n    set(M_TARGET \"i686-pc-windows-gnu\")\n\
             elseif(ARCH STREQUAL \"amd64\")\n    set(M_TARGET \"x86_64-pc-windows-gnu\")\n\
             elseif(ARCH STREQUAL \"arm64\")\n    set(M_TARGET \"aarch64-pc-windows-gnullvm\")\nendif()\n",
        );
        t += &cmd(
            "STUB",
            "m-stub",
            "libm_stub.a",
            "\n            native-widl\n            m_c",
        );
        t += &cmd(
            "MRUST",
            "m-rust",
            "libm_rust.a",
            "\n            ${M_WS}/m-sys/Cargo.toml",
        );
        t
    }

    #[test]
    fn the_body_is_the_command_named_for_the_target_and_the_rest_are_engine_crates() {
        let v = custom_body_for(&module(), "m").expect("models");
        assert_eq!(v["crate"], "m-rust");
        assert_eq!(v["archive"], "libm_rust.a");
        assert_eq!(v["extra"][0]["crate"], "m-stub");
        assert_eq!(v["extra"][0]["archive"], "libm_stub.a");
        let deps: Vec<_> = v["depends"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str().unwrap())
            .collect();
        assert_eq!(deps, ["native-widl", "m_c"]);
        // Variables the stub's command still reads stay in the CMakeLists.
        let vars: Vec<_> = v["vars"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str().unwrap())
            .collect();
        assert!(
            !vars.contains(&"M_WS") && !vars.contains(&"M_TARGET"),
            "{vars:?}"
        );
        assert!(
            vars.contains(&"MRUST_LIB") && vars.contains(&"MRUST_DIR"),
            "{vars:?}"
        );
    }

    #[test]
    fn an_extra_command_with_its_own_rustflags_or_features_is_refused() {
        let flagged = module().replace("-p m-stub", "-p m-stub --features x");
        assert!(custom_body_for(&flagged, "m").is_none());
        // No command is the target's own body.
        assert!(custom_body_for(&module(), "other").is_none());
    }

    #[test]
    fn an_env_list_carries_its_target_dir_and_per_arch_rustflags() {
        // oleaut32: `-E env ${X_ENV} cargo build`, the env a list with an i386 RUSTFLAGS.
        let body = cmd("MRUST", "m-rust", "libm_rust.a", "").replace(
            "-E env\n        CARGO_TARGET_DIR=${MRUST_DIR}\n",
            "-E env\n        ${MRUST_ENV}\n",
        );
        let text = format!(
            "set(M_WS \"${{CMAKE_SOURCE_DIR}}/sdk/lib/rust\")\n\
             if(ARCH STREQUAL \"i386\")\n    set(M_TARGET \"i686-pc-windows-gnu\")\n\
             elseif(ARCH STREQUAL \"amd64\")\n    set(M_TARGET \"x86_64-pc-windows-gnu\")\n\
             elseif(ARCH STREQUAL \"arm64\")\n    set(M_TARGET \"aarch64-pc-windows-gnullvm\")\nendif()\n\
             set(MRUST_ENV CARGO_TARGET_DIR=${{MRUST_DIR}})\n\
             if(ARCH STREQUAL \"i386\")\n    list(APPEND MRUST_ENV \"RUSTFLAGS=-C target-feature=+sse,+sse2\")\nendif()\n{body}"
        );
        let v = super::custom_body(&text).expect("models");
        assert_eq!(v["crate"], "m-rust");
        assert_eq!(v["rustflags"]["i386"], "-C target-feature=+sse,+sse2");
        assert!(v["rustflags"].get("amd64").is_none());
        assert!(v["vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "MRUST_ENV"));
        // Any other use of the env variable is unmodelled.
        let extra = text.replace(
            "endif()\nrust_storage",
            "endif()\nmessage(${MRUST_ENV})\nrust_storage",
        );
        assert!(super::custom_body(&extra).is_none());
    }

    #[test]
    fn the_entry_is_the_last_entry_flag_as_lld_takes_it() {
        let flags = "-nostdlib -Wl,-entry,_DllMainCRTStartup@12 -Wl,--image-base,0x7bdc0000 \
                     -Wl,-entry,_Ws2HelpDllMain@12 dll/win32/ws2help/ws2help.def";
        assert_eq!(super::link_entry(flags), "_Ws2HelpDllMain@12");
        assert_eq!(
            super::link_entry("-Wl,-entry,DllMainCRTStartup"),
            "DllMainCRTStartup"
        );
        assert_eq!(super::link_entry("-nostdlib"), "");
    }

    #[test]
    fn objects_are_keyed_by_their_path_in_the_target_dir() {
        let a = "dll/win32/kernel32/CMakeFiles/kernel32.dir/client/i386/fiber.S.obj";
        let b = "dll/win32/kernel32/CMakeFiles/kernel32.dir/client/amd64/fiber.S.obj";
        assert_ne!(obj_key(a), obj_key(b));
        assert_eq!(obj_key(a), "client/i386/fiber.S.obj");
        assert_eq!(obj_key("x/libfoo.a"), "libfoo.a");
    }
}
