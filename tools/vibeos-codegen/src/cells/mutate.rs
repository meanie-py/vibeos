//! Planning-only mutations. No function writes to the checkout.
use super::graph::{args, engine_lib, library_index, statements};
use super::recipes::{c_lib_for, rc_commands, rc_flags_for};
use super::render::{c_binding, rc_binding};
use super::{Error, Result, ARCHES};
use regex::{Captures, Regex};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
pub struct Plan {
    pub edits: BTreeMap<PathBuf, String>,
    pub changed: Vec<String>,
    pub skipped: Vec<String>,
    pub provenance: Vec<String>,
}
fn rx(s: &str) -> Regex {
    Regex::new(s).expect("static mutation regex")
}
fn read(p: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(p)?)
}
fn current(plan: &Plan, p: &Path) -> Result<String> {
    match plan.edits.get(p) {
        Some(s) => Ok(s.clone()),
        None => read(p),
    }
}
fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut v = crate::source_walk::files(root).map_err(Error::new)?;
    v.retain(|p| p.file_name().is_some_and(|x| x == "CMakeLists.txt"));
    v.sort();
    Ok(v)
}
pub fn cmakelists(root: &Path) -> Result<Vec<PathBuf>> {
    files(root)
}
fn crate_dirs(root: &Path, crates: Option<&[String]>) -> Result<Vec<PathBuf>> {
    let rust = root.join("sdk/lib/rust");
    let mut v = Vec::new();
    if let Some(names) = crates.filter(|x| !x.is_empty()) {
        for n in names {
            v.push(rust.join(n));
        }
    } else {
        for e in std::fs::read_dir(rust)? {
            let e = e?;
            if e.file_type()?.is_dir() && e.file_name().to_string_lossy().ends_with("-cargo") {
                v.push(e.path());
            }
        }
        v.sort();
    }
    Ok(v)
}
pub fn call_span(text: &str, start: usize) -> Result<usize> {
    let Some(i) = text[start..].find('(') else {
        return Err(Error::new("call has no opening parenthesis"));
    };
    let mut depth = 0usize;
    for (off, c) in text[start + i..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(start + i + off + 1);
                }
            }
            _ => {}
        }
    }
    Err(Error::new("unterminated call"))
}
pub fn rc_object_depends(cmd: &str, raw: &str) -> Option<Vec<String>> {
    let a = args(raw);
    if cmd == "add_rc_deps" && a.first().is_some_and(|x| x.ends_with(".rc")) {
        return Some(a[1..].to_vec());
    }
    if cmd == "set_source_files_properties" && a.len() >= 4 {
        if let Some(i) = a.iter().position(|x| x == "PROPERTIES") {
            if a[..i].iter().all(|x| x.ends_with(".rc"))
                && a.get(i + 1).is_some_and(|x| x == "OBJECT_DEPENDS")
            {
                return Some(
                    a[i + 2..]
                        .iter()
                        .map(|x| x.trim_matches('"').to_owned())
                        .collect(),
                );
            }
        }
    }
    None
}
fn extra_deps(out: &str, target: &str, deps: &[String]) -> (String, usize) {
    let re = rx(&format!(
        r"EXTRA_DEPENDS ([^)\n]*\b{}\b[^)\n]*)",
        regex::escape(target)
    ));
    let mut n = 0;
    let v = re
        .replace_all(out, |c: &Captures| {
            n += 1;
            let mut names: Vec<String> = c[1]
                .split_whitespace()
                .filter(|x| *x != target)
                .map(str::to_owned)
                .collect();
            for d in deps {
                if !names.contains(d) {
                    names.push(d.clone());
                }
            }
            format!("EXTRA_DEPENDS {}", names.join(" "))
        })
        .into_owned();
    (v, n)
}
const COMMENTS: [&str; 3] = [
    "# How cmake compiled the target's sources, now on the OBJECT libraries.\n",
    "# Generated inputs those compiles read, ordered before them.\n",
    "# The C still compiled, in an OBJECT library whose compile the cargo link replays.\n",
];
fn remove_comment(mut gap: String, prefixes: &[&str]) -> String {
    let mut lines: Vec<_> = gap.split('\n').collect();
    while lines.len() > 1
        && prefixes
            .iter()
            .any(|p| lines[lines.len() - 2].trim_start().starts_with(p))
    {
        let i = lines.len() - 2;
        lines.remove(i);
    }
    gap = lines.join("\n");
    gap
}
pub fn drop_shims_in(
    text: &str,
    cmake: &Path,
    target: &str,
    generated: &[String],
) -> Result<String> {
    let (mut out, mut pos, mut deps) = (String::new(), 0, Vec::new());
    for (s, e, cmd, raw) in statements(text) {
        let a = args(&raw);
        if a.first().is_some_and(|x| x == target) {
            if cmd == "add_dependencies" {
                deps.extend_from_slice(&a[1..]);
            }
            if cmd == "target_link_libraries" {
                deps.extend(
                    a[1..]
                        .iter()
                        .filter(|x| {
                            !["PRIVATE", "PUBLIC", "INTERFACE"].contains(&x.as_str())
                                && !x.starts_with("${")
                        })
                        .cloned(),
                );
            }
            let gap = text[pos..s].to_owned();
            out.push_str(&if cmd == "add_library" {
                remove_comment(gap, &["# The C still compiled", "# whose compile"])
            } else {
                gap
            });
            pos = e;
        } else {
            out.push_str(&text[pos..e]);
            pos = e;
        }
    }
    out.push_str(&text[pos..]);
    for c in COMMENTS {
        out = out.replace(c, "");
    }
    deps.extend_from_slice(generated);
    let (out, n) = extra_deps(&out, target, &deps);
    if n == 0 || rx(&format!(r"\b{}\b", regex::escape(target))).is_match(&out) {
        return Err(Error::new(format!(
            "{}: {} still named after the rewrite",
            cmake.display(),
            target
        )));
    }
    Ok(rx(r"\n{3,}").replace_all(&out, "\n\n").into_owned())
}
pub fn drop_res_in(text: &str, cmake: &Path, target: &str) -> Result<String> {
    let (mut out, mut pos, mut deps) = (String::new(), 0, Vec::new());
    for (s, e, cmd, raw) in statements(text) {
        let a = args(&raw);
        let rd = rc_object_depends(&cmd, &raw);
        if a.first().is_some_and(|x| x == target) || rd.is_some() {
            if cmd == "add_dependencies" {
                deps.extend_from_slice(&a[1..]);
            }
            if let Some(d) = rd {
                deps.extend(
                    d.into_iter()
                        .filter(|x| x.contains("CMAKE_CURRENT_BINARY_DIR")),
                );
            }
            let gap = text[pos..s].to_owned();
            out.push_str(&if cmd == "add_library" {
                remove_comment(gap, &["# The resources are", "# whose windres command"])
            } else {
                gap
            });
            pos = e;
        } else {
            out.push_str(&text[pos..e]);
            pos = e;
        }
    }
    out.push_str(&text[pos..]);
    let re = rx(r"file\(GLOB (\w+)[^)]*\)\n");
    let matches: Vec<_> = re
        .captures_iter(&out)
        .map(|c| (c[0].to_owned(), c[1].to_owned()))
        .collect();
    for (whole, var) in matches {
        if !out.contains(&format!("${{{var}}}")) {
            out = out.replacen(&whole, "", 1);
        }
    }
    for c in COMMENTS {
        if let Some(i) = out.find(c) {
            let escaped = regex::escape(c);
            if !rx(&format!(r"{}\s*\S*_shims", escaped)).is_match(&out) {
                let next = &out[i + c.len()..];
                if !rx(r"^(?:\s*if\([^)]*\)\s*)?\S+\(\s*\S*_shims").is_match(next) {
                    out = out.replacen(c, "", 1);
                }
            }
        }
    }
    deps.push("psdk".into());
    let (out, n) = extra_deps(&out, target, &deps);
    if n == 0 && rx(&format!(r"\b{}\b", regex::escape(target))).is_match(&out) {
        return Err(Error::new(format!(
            "{}: {} still named after the rewrite",
            cmake.display(),
            target
        )));
    }
    Ok(rx(r"\n{3,}")
        .replace_all(&out, "\n\n")
        .trim_start_matches('\n')
        .to_owned())
}
/// Plan remapping in crate order, resolving archives using the current source graph.
pub fn remap(root: &Path, crates: Option<&[String]>) -> Result<Plan> {
    let mut plan = Plan::default();
    let index = library_index(root)?;
    let cmakes = cmakelists(root)?;
    let archive = rx(r#""([^"\n]+\.a)""#);
    let comment = rx(r"//[^\n]*");
    for dir in crate_dirs(root, crates)? {
        let rs = dir.join("build.rs");
        if !rs.exists() {
            continue;
        }
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let mut text = current(&plan, &rs)?;
        let stripped = comment.replace_all(&text, "");
        let raw: BTreeSet<String> = archive
            .captures_iter(&stripped)
            .map(|c| c[1].to_owned())
            .collect();
        let mapping: BTreeMap<_, _> = raw
            .into_iter()
            .filter_map(|x| engine_lib(&x, &index).map(|v| (x, v)))
            .collect();
        if mapping.is_empty() {
            continue;
        }
        let pat = rx(&format!(
            r"add_cargo_module\(\S+ {}\b",
            regex::escape(&name)
        ));
        let mut cm = None;
        for p in &cmakes {
            if pat.is_match(&current(&plan, p)?) {
                cm = Some(p.clone());
                break;
            }
        }
        let Some(cm) = cm else {
            plan.skipped
                .push(format!("skipped {name}: not an add_cargo_module cell"));
            continue;
        };
        let mut cml = current(&plan, &cm)?;
        for (lib, (engine, _)) in &mapping {
            text = text.replace(&format!("\"{lib}\""), &format!("\"{engine}\""));
            let pattern = format!(
                r"(?m)^[ \t]*(?:list\(APPEND \w+ )?\$\{{CMAKE_BINARY_DIR\}}/{}\)?\n",
                regex::escape(lib)
            );
            cml = rx(&pattern)
                .replace_all(&cml, |c: &Captures| {
                    if c[0].trim_end().ends_with(')') && !c[0].contains("list(APPEND") {
                        ")\n"
                    } else {
                        ""
                    }
                })
                .into_owned();
        }
        let empty = rx(r"set\((\w+_REUSED)\s*\)");
        let vars: Vec<_> = empty.captures_iter(&cml).map(|c| c[1].to_owned()).collect();
        for var in vars {
            cml = rx(&format!(r"set\({}\s*\)\n", regex::escape(&var)))
                .replace_all(&cml, "")
                .into_owned();
            cml = cml.replace(&format!(" ${{{var}}}"), "");
        }
        cml = rx("(?m)^if\\(ARCH STREQUAL \"\\w+\"\\)\\nendif\\(\\)\\n")
            .replace_all(&cml, "")
            .into_owned();
        let deps: BTreeSet<_> = mapping
            .values()
            .flat_map(|(_, d)| d.iter().cloned())
            .collect();
        if !deps.is_empty() {
            let re = rx(r"(?m)^(\s*EXTRA_DEPENDS)([^)\n]*)\)");
            if let Some(c) = re.captures(&cml) {
                let mut have: Vec<String> = c[2].split_whitespace().map(str::to_owned).collect();
                for d in deps {
                    if !have.contains(&d) {
                        have.push(d);
                    }
                }
                let replacement = format!("{} {})", &c[1], have.join(" "));
                let m = c.get(0).unwrap();
                cml.replace_range(m.start()..m.end(), &replacement);
            } else {
                return Err(Error::new(format!(
                    "{name}: no EXTRA_DEPENDS to add {:?} to",
                    deps
                )));
            }
        }
        plan.edits.insert(rs, text);
        plan.edits.insert(cm, cml);
        plan.changed.push(name);
    }
    Ok(plan)
}
fn insert_bindings(text: &str, bindings: Vec<String>, rc: bool) -> Result<String> {
    let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    let at = lines
        .iter()
        .position(|x| x.trim().starts_with("let arch = env::var("))
        .ok_or_else(|| Error::new("build.rs: no architecture binding"))?;
    let count = bindings.len();
    let mut insert = vec![String::new()];
    if rc {
        insert.extend(bindings.into_iter().take(count - 1));
    } else {
        insert.extend(bindings);
    }
    lines.splice(at + 1..at + 1, insert);
    if rc
        && lines
            .get(at + 1 + count)
            .is_some_and(|x| !x.trim().is_empty())
    {
        lines.insert(at + 1 + count, String::new());
    }
    Ok(lines.join("\n"))
}
/// Plan conversion of cmake-replayed C shims to Linker::c_compiled.
pub fn own_c(root: &Path, crates: Option<&[String]>) -> Result<Plan> {
    let mut plan = Plan::default();
    let cmakes = cmakelists(root)?;
    let shims = rx(r#"\.c_from_compile_db\(\s*"(\w+)""#);
    for dir in crate_dirs(root, crates)? {
        let rs = dir.join("build.rs");
        if !rs.exists() {
            continue;
        }
        let mut text = current(&plan, &rs)?;
        let Some(m) = shims.captures(&text) else {
            continue;
        };
        let name = m[1].to_owned();
        let start = m.get(0).unwrap().start();
        if ["rtl", "blrtl", "libcntpr", "win32k_residue"].contains(&name.as_str()) {
            continue;
        }
        let plain = rx(&format!(
            r"add_library\(\s*{}\s+OBJECT\s+",
            regex::escape(&name)
        ));
        let mut hits = Vec::new();
        for p in &cmakes {
            let c = current(&plan, p)?;
            if let Some(x) = plain.find(&c) {
                if !rx(r"^\$\{\w+_SHIMS\}").is_match(&c[x.end()..]) {
                    hits.push(p.clone());
                }
            }
        }
        let crate_name = dir.file_name().unwrap().to_string_lossy().to_string();
        if hits.len() != 1 {
            plan.skipped.push(format!(
                "left on replay: {crate_name} ({name}: {} plain definitions)",
                hits.len()
            ));
            continue;
        }
        let (srcs, flags) = c_lib_for(&name, root)?;
        let end = call_span(&text, start)?;
        text.replace_range(
            start..end,
            &format!(".c_compiled(\"{name}\", c_sources, c_includes, c_flags, c_defines)"),
        );
        text = insert_bindings(&text, c_binding(&srcs, &flags), false)?;
        let generated: BTreeSet<String> = ARCHES
            .into_iter()
            .flat_map(|a| srcs[a].iter())
            .filter_map(|x| {
                x.strip_prefix("@build/")
                    .map(|s| format!("${{REACTOS_BINARY_DIR}}/{s}"))
            })
            .collect();
        let cm = &hits[0];
        let cml = drop_shims_in(
            &current(&plan, cm)?,
            cm,
            &name,
            &generated.into_iter().collect::<Vec<_>>(),
        )?;
        plan.edits.insert(rs, text);
        plan.edits.insert(cm.clone(), cml);
        plan.changed.push(crate_name);
    }
    Ok(plan)
}
/// A configured sibling's resource flags, with the sibling path rewritten; returns provenance.
pub fn rc_flags_by_analogy(
    rc: &str,
    root: &Path,
) -> Result<(BTreeMap<String, super::RcFlags>, String)> {
    let mine = Path::new(rc).parent().unwrap_or(Path::new(""));
    let canon = super::constants()["CANON"]
        .as_str()
        .ok_or_else(|| Error::new("missing CANON constant"))?;
    let pat = rx(r"/CMakeFiles/\w+_res\.dir/");
    for e in rc_commands(root, "amd64")? {
        let src = e["file"]
            .as_str()
            .ok_or_else(|| Error::new("resource command lacks file"))?
            .strip_prefix(&format!("{canon}/"))
            .unwrap_or(e["file"].as_str().unwrap());
        let srcpath = Path::new(src);
        let parent = srcpath.parent().unwrap_or(Path::new(""));
        if parent.parent() != mine.parent()
            || !pat.is_match(
                e["command"]
                    .as_str()
                    .ok_or_else(|| Error::new("resource command lacks command"))?,
            )
        {
            continue;
        }
        let marker = format!("{}/CMakeFiles/", parent.display());
        let mut flags = rc_flags_for(src, &marker, root)?;
        let old = parent.to_string_lossy();
        let new = mine.to_string_lossy();
        for f in flags.values_mut() {
            for x in &mut f.includes {
                if x == old.as_ref() {
                    *x = new.to_string();
                } else if x.ends_with(&format!("/{old}")) {
                    let n = x.len() - old.len();
                    x.replace_range(n.., &new);
                }
            }
        }
        return Ok((flags, src.to_owned()));
    }
    Err(Error::new(format!(
        "{rc}: no windres command, and no configured sibling to take one from"
    )))
}
/// Plan resource library removals, consulting planned prior edits as well as disk.
pub fn drop_res_library(plan: &mut Plan, root: &Path, target: &str) -> Result<()> {
    let pat = rx(&format!(
        r"add_library\(\s*{}\s+OBJECT",
        regex::escape(target)
    ));
    let mut hits = Vec::new();
    for p in cmakelists(root)? {
        if pat.is_match(&current(plan, &p)?) {
            hits.push(p);
        }
    }
    if hits.is_empty() {
        return Err(Error::new(format!("{target}: defined in no CMakeLists")));
    }
    for p in hits {
        let c = drop_res_in(&current(plan, &p)?, &p, target)?;
        plan.edits.insert(p, c);
    }
    Ok(())
}
/// Plan conversion of resource compile replays and removal of unused CMake OBJECT libraries.
pub fn own_rc(root: &Path, crates: Option<&[String]>) -> Result<Plan> {
    let mut plan = Plan::default();
    let rust = root.join("sdk/lib/rust");
    let re = rx(r#"\.resource_from_rc\("([^"]+)",\s*"([^"]+)"\)"#);
    let mut targets = BTreeSet::new();
    for dir in crate_dirs(root, crates)? {
        let rs = dir.join("build.rs");
        if !rs.exists() {
            continue;
        }
        let mut text = current(&plan, &rs)?;
        let calls: Vec<_> = re
            .captures_iter(&text)
            .map(|c| (c[1].to_owned(), c[2].to_owned()))
            .collect();
        if calls.is_empty() {
            continue;
        }
        if calls.len() > 1 {
            return Err(Error::new(format!(
                "{}: {} resource_from_rc calls",
                rs.display(),
                calls.len()
            )));
        }
        let (rc, target) = &calls[0];
        let flags = match rc_flags_for(rc, &format!("/CMakeFiles/{target}.dir/"), root) {
            Ok(x) => x,
            Err(e) => {
                let (f, source) = rc_flags_by_analogy(rc, root)?;
                plan.provenance.push(format!(
                    "{rc}: configured sibling {source} (direct lookup: {e})"
                ));
                f
            }
        };
        text = insert_bindings(&text, rc_binding(&flags), true)?;
        text = re
            .replacen(
                &text,
                1,
                format!(".rc(\"{rc}\", rc_includes, rc_flags, rc_defines)"),
            )
            .into_owned();
        plan.edits.insert(rs, text);
        targets.insert(target.clone());
        plan.changed
            .push(dir.file_name().unwrap().to_string_lossy().to_string());
    }
    let mut still = BTreeSet::new();
    for entry in std::fs::read_dir(rust)? {
        let entry = entry?;
        let p = entry.path().join("build.rs");
        if p.is_file() {
            for c in re.captures_iter(&current(&plan, &p)?) {
                still.insert(c[2].to_owned());
            }
        }
    }
    let dep = rx(r"EXTRA_DEPENDS ([^)\n]*)");
    for p in cmakelists(root)? {
        for c in dep.captures_iter(&current(&plan, &p)?) {
            targets.extend(
                c[1].split_whitespace()
                    .filter(|x| x.ends_with("_res"))
                    .map(str::to_owned),
            );
        }
    }
    for t in targets.difference(&still) {
        drop_res_library(&mut plan, root, t)?;
    }
    Ok(plan)
}
