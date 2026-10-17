//! Port of tools/verify_match_consts.py (default CLI mode).
use crate::{Corpus, Rule, SourceFile, SourceSet};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

pub struct VerifyMatchConsts;
const INPUTS: &[SourceSet] = &[
    SourceSet::tree("match-consts-rust", &["."], &["rs"], &[]).with_pruning(&["target", ".git"]),
    SourceSet::tree("match-consts-manifests", &["."], &[], &["Cargo.toml"])
        .with_pruning(&["target", ".git"]),
];
impl Rule for VerifyMatchConsts {
    fn name(&self) -> &'static str {
        "verify_match_consts"
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        INPUTS
    }
    fn needs_module_tree(&self) -> bool {
        true
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let all: Vec<_> = c
            .sources("match-consts-rust")
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .collect();
        // The dependency closure and glob resolver use the same exported-name
        // parser. Compute its immutable per-file result once and share it across
        // every crate/source lookup in this run.
        let names_by_path: BTreeMap<PathBuf, BTreeSet<String>> = all
            .iter()
            .map(|f| (f.path.clone(), names_in_text(&f.text)))
            .collect();
        let default: Vec<_> = all
            .iter()
            .copied()
            .filter(|f| f.path.starts_with("sdk/lib/rust"))
            .collect();
        let mut universe = BTreeSet::new();
        for f in &default {
            universe.extend(names_by_path[&f.path].iter().cloned());
        }
        let by_path: BTreeMap<PathBuf, &SourceFile> =
            all.iter().map(|f| (f.path.clone(), *f)).collect();
        let manifests: BTreeMap<PathBuf, &SourceFile> = c
            .sources("match-consts-manifests")
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .map(|f| (f.path.clone(), f))
            .collect();
        // Index descendant files once. The original per-file universe walk tested every
        // corpus path with `starts_with` for every crate, then repeated that walk for
        // every source file in the crate. Keep its src-preferred/fallback semantics but
        // turn those repeated scans into a cached per-crate result.
        let mut descendants: BTreeMap<PathBuf, Vec<&SourceFile>> = BTreeMap::new();
        for f in &all {
            for dir in f.path.ancestors().skip(1) {
                descendants.entry(dir.to_owned()).or_default().push(*f);
            }
        }
        let mut deps_by_dir = BTreeMap::new();
        for dir in manifests.keys().filter_map(|p| p.parent()) {
            deps_by_dir.insert(dir.to_owned(), crate_path_deps(dir, &manifests));
        }
        let mut universe_cache: HashMap<PathBuf, Arc<BTreeSet<String>>> = HashMap::new();
        let mut results = Vec::new();
        let mut counts = [0usize; 4]; // high, medium, orphan, low
        for f in default {
            let arms = candidates(&f.text);
            if arms.is_empty() {
                continue;
            }
            let (local, has_glob) = local_names(&f.text);
            let crate_dir = crate_dir(&f.path, &manifests);
            let closure = crate_dir
                .as_ref()
                .map(|dir| {
                    universe_cache
                        .entry(dir.clone())
                        .or_insert_with(|| {
                            Arc::new(crate_universe_index(
                                dir,
                                &manifests,
                                &deps_by_dir,
                                &descendants,
                                &names_by_path,
                            ))
                        })
                        .clone()
                })
                .unwrap_or_else(|| Arc::new(BTreeSet::new()));
            let (glob, resolved) = crate_dir
                .as_ref()
                .map(|dir| {
                    glob_names_cached(
                        &f.text,
                        dir,
                        &manifests,
                        &by_path,
                        &deps_by_dir,
                        &names_by_path,
                    )
                })
                .unwrap_or_default();
            let root = c.module_tree().crate_root_of(&f.path).map(Path::to_owned);
            let is_live = root.as_ref().map_or(true, |r| {
                let live = c.module_tree().live_files(r);
                live.is_empty() || live.contains(&f.path)
            });
            let mut hits = Vec::new();
            for (line, id, _raw, indent) in &arms {
                if local.contains(id) || glob.contains(id) {
                    continue;
                }
                let (rank, why) = if !closure.contains(id) {
                    (0, "not defined in-file and not defined anywhere in this crate's dependency closure -> silent catch-all binding")
                } else if resolved {
                    (0, "defined in this crate but NOT exported by any of this file's (fully resolved) glob imports and never explicitly `use`d -> silent catch-all binding")
                } else if has_glob {
                    (3, "defined in this crate's dependency closure and this file has glob imports -> probably resolved through a `use ...::*` (likely FP)")
                } else {
                    (1, "defined in this crate's dependency closure but this file has NO glob import that could bring it into scope -> may be a silent binding")
                };
                let followed = arms
                    .iter()
                    .any(|(ln, _, _, ind2)| *ln > *line && *ln <= *line + 120 && *ind2 == *indent);
                hits.push((*line, id.clone(), rank, why, followed));
            }
            for (line, id, mut rank, why, followed) in hits {
                if !is_live {
                    rank = 2;
                }
                counts[rank] += 1;
                if rank == 0 || rank == 1 {
                    let conf = if rank == 0 { "HIGH" } else { "MEDIUM" };
                    let extra = if !is_live {
                        " [in a file NO `mod` declaration reaches — not compiled]"
                    } else {
                        ""
                    };
                    let smell = if followed {
                        "  [+followed-by-arms: unreachable smell]"
                    } else {
                        ""
                    };
                    results.push((
                        rank,
                        !followed,
                        f.path.clone(),
                        line,
                        format!(
                            "{}:{}: [{}] `{}` -- {}{}{}",
                            f.path.display(),
                            line,
                            conf,
                            id,
                            why,
                            extra,
                            smell
                        ),
                    ));
                }
            }
        }
        results.sort_by(|a, b| (a.0, a.1, &a.2, a.3).cmp(&(b.0, b.1, &b.2, b.3)));
        let mut out: Vec<String> = results.into_iter().map(|x| x.4).collect();
        out.push(String::new());
        out.push(format!(
            "SUMMARY: HIGH={}  MEDIUM={}  ORPHAN(dead file)={}  LOW(likely-FP)={}  shown={}",
            counts[0],
            counts[1],
            counts[2],
            counts[3],
            counts[0] + counts[1]
        ));
        Ok(out)
    }
}
fn regex(p: &'static str) -> &'static Regex {
    static R: OnceLock<BTreeMap<&'static str, Regex>> = OnceLock::new();
    let m = R.get_or_init(|| {
        let mut m = BTreeMap::new();
        for p in [
            r"^_*[A-Z][A-Za-z0-9_]*$",
            r"\b(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(?:mut\s+)?(_*[A-Z][A-Za-z0-9_]*)\b",
            r"(?s)enum\s+\w+[^\{]*\{(.*?)\}",
            r"(?m)^[ \t]*(?:#\[[^\]]*\]\s*)*pub\s+use\s+([^;]+);",
            r"\blet\s+(?:mut\s+)?(_*[A-Z][A-Za-z0-9_]*)\b",
            r"\buse\s+([^;]+);",
            r"^\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;",
            r"(?m)^\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;",
            r"(?:#\[[^\]]*\]\s*)*pub\s+use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;",
            r#"path\s*=\s*"([^"]+)""#,
            r"\bmod\s+([A-Za-z_]\w*)\s*;",
            r"(?m)^\s*(?:pub\s+)?const\s+",
            r"=>",
        ] {
            m.insert(p, crate::regexes::cached(p).unwrap());
        }
        m
    });
    m.get(p).unwrap()
}
fn constish(s: &str) -> bool {
    regex(r"^_*[A-Z][A-Za-z0-9_]*$").is_match(s)
        && ![
            "None", "Some", "Ok", "Err", "Ordering", "Less", "Greater", "Equal", "Cow", "Borrowed",
            "Owned",
        ]
        .contains(&s)
}
fn variants(body: &str, out: &mut BTreeSet<String>) {
    for item in body.split(',') {
        let t = item.trim();
        let n = t
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .next()
            .unwrap_or("");
        if constish(n) {
            out.insert(n.to_owned());
        }
    }
}
fn names_in_text(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for c in
        regex(r"\b(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(?:mut\s+)?(_*[A-Z][A-Za-z0-9_]*)\b")
            .captures_iter(text)
    {
        out.insert(c[1].to_owned());
    }
    for c in regex(r"(?s)enum\s+\w+[^\{]*\{(.*?)\}").captures_iter(text) {
        variants(&c[1], &mut out);
    }
    pub_uses(text, &mut out);
    out
}
fn pub_uses(text: &str, out: &mut BTreeSet<String>) {
    for c in regex(r"(?m)^[ \t]*(?:#\[[^\]]*\]\s*)*pub\s+use\s+([^;]+);").captures_iter(text) {
        let clause = &c[1];
        for group in clause
            .split('{')
            .skip(1)
            .filter_map(|x| x.split('}').next())
        {
            for token in group.split(',') {
                let t = token
                    .trim()
                    .split(" as ")
                    .last()
                    .unwrap_or("")
                    .split("::")
                    .last()
                    .unwrap_or("")
                    .trim();
                if constish(t) {
                    out.insert(t.into());
                }
            }
        }
        if !clause.contains('*') {
            let t = clause
                .split('{')
                .next()
                .unwrap_or("")
                .trim()
                .split(" as ")
                .last()
                .unwrap_or("")
                .split("::")
                .last()
                .unwrap_or("")
                .trim();
            if constish(t) {
                out.insert(t.into());
            }
        }
    }
}
fn local_names(text: &str) -> (BTreeSet<String>, bool) {
    let mut n = names_in_text(text);
    for c in regex(r"\blet\s+(?:mut\s+)?(_*[A-Z][A-Za-z0-9_]*)\b").captures_iter(text) {
        n.insert(c[1].into());
    }
    let mut glob = false;
    for c in regex(r"\buse\s+([^;]+);").captures_iter(text) {
        let cl = &c[1];
        if cl.contains('*') {
            glob = true;
        }
        for t in cl.split(|x| x == ',' || x == '{' || x == '}') {
            let s = t
                .trim()
                .split(" as ")
                .last()
                .unwrap_or("")
                .split("::")
                .last()
                .unwrap_or("")
                .trim();
            if constish(s) {
                n.insert(s.into());
            }
            let s = t
                .trim()
                .split(" as ")
                .next()
                .unwrap_or("")
                .split("::")
                .last()
                .unwrap_or("")
                .trim();
            if constish(s) {
                n.insert(s.into());
            }
        }
    }
    (n, glob)
}
fn strip_comment(line: &str) -> String {
    let mut quoted = false;
    let mut charq = false;
    let mut esc = false;
    let cs: Vec<char> = line.chars().collect();
    let mut o = String::new();
    let mut i = 0;
    while i < cs.len() {
        let ch = cs[i];
        if esc {
            o.push(ch);
            esc = false;
            i += 1;
            continue;
        }
        if quoted || charq {
            o.push(ch);
            if ch == '\\' {
                esc = true;
            } else if (quoted && ch == '"') || (charq && ch == '\'') {
                quoted = false;
                charq = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            quoted = true;
            o.push(ch);
            i += 1;
            continue;
        }
        if ch == '\'' {
            charq = true;
            o.push(ch);
            i += 1;
            continue;
        }
        if ch == '/' && cs.get(i + 1) == Some(&'/') {
            break;
        }
        o.push(ch);
        i += 1;
    }
    o
}
fn candidates(text: &str) -> Vec<(usize, String, String, usize)> {
    let mut arms = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        if let Some((left, _)) = strip_comment(raw).split_once("=>") {
            let s = left.trim().trim_end_matches(',').trim();
            if constish(s) {
                let ind = raw.len() - raw.trim_start().len();
                arms.push((i + 1, s.into(), raw.into(), ind));
            }
        }
    }
    arms
}
fn crate_dir(path: &Path, manifests: &BTreeMap<PathBuf, &SourceFile>) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|p| manifests.contains_key(&p.join("Cargo.toml")))
        .map(Path::to_owned)
}
fn crate_path_deps(dir: &Path, manifests: &BTreeMap<PathBuf, &SourceFile>) -> Vec<PathBuf> {
    let Some(m) = manifests.get(&dir.join("Cargo.toml")) else {
        return vec![];
    };
    regex(r#"path\s*=\s*"([^"]+)""#)
        .captures_iter(&m.text)
        .map(|c| normalize(&dir.join(c[1].trim_matches('"'))))
        .filter(|p| manifests.contains_key(&p.join("Cargo.toml")))
        .collect()
}
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            x => out.push(x.as_os_str()),
        }
    }
    out
}
fn crate_universe_index(
    dir: &Path,
    manifests: &BTreeMap<PathBuf, &SourceFile>,
    deps_by_dir: &BTreeMap<PathBuf, Vec<PathBuf>>,
    descendants: &BTreeMap<PathBuf, Vec<&SourceFile>>,
    names_by_path: &BTreeMap<PathBuf, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut seen = HashSet::new();
    let mut todo = vec![dir.to_owned()];
    while let Some(d) = todo.pop() {
        if !seen.insert(d.clone()) {
            continue;
        }
        // Match Python/Rust legacy behavior: use crate/src when it has any Rust
        // inputs; otherwise include the whole crate subtree.
        let src = d.join("src");
        let files = descendants
            .get(&src)
            .filter(|files| !files.is_empty())
            .or_else(|| descendants.get(&d));
        if let Some(files) = files {
            for f in files {
                if let Some(names) = names_by_path.get(&f.path) {
                    out.extend(names.iter().cloned());
                }
            }
        }
        if let Some(deps) = deps_by_dir.get(&d) {
            // Preserve the old validation against known manifests. This is normally
            // redundant because deps_by_dir was built from those manifests, but keeps
            // the membership condition explicit if the snapshot has odd paths.
            todo.extend(
                deps.iter()
                    .filter(|p| manifests.contains_key(&p.join("Cargo.toml")))
                    .cloned(),
            );
        }
    }
    out
}

fn glob_names(
    text: &str,
    dir: &Path,
    manifests: &BTreeMap<PathBuf, &SourceFile>,
    files: &BTreeMap<PathBuf, &SourceFile>,
) -> (BTreeSet<String>, bool) {
    let deps_by_dir = manifests
        .keys()
        .filter_map(|p| p.parent())
        .map(|dir| (dir.to_owned(), crate_path_deps(dir, manifests)))
        .collect();
    let names_by_path = files
        .iter()
        .map(|(path, file)| (path.clone(), names_in_text(&file.text)))
        .collect();
    glob_names_cached(text, dir, manifests, files, &deps_by_dir, &names_by_path)
}

fn glob_names_cached(
    text: &str,
    dir: &Path,
    manifests: &BTreeMap<PathBuf, &SourceFile>,
    files: &BTreeMap<PathBuf, &SourceFile>,
    deps_by_dir: &BTreeMap<PathBuf, Vec<PathBuf>>,
    names_by_path: &BTreeMap<PathBuf, BTreeSet<String>>,
) -> (BTreeSet<String>, bool) {
    let mut all = BTreeSet::new();
    let mut ok = true;
    for m in regex(r"(?m)^\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;")
        .captures_iter(text)
    {
        let segs = m[1].split("::").collect::<Vec<_>>();
        if segs.first() != Some(&"crate")
            && !deps_by_dir.get(dir).into_iter().flatten().any(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().replace('-', "_") == segs[0])
            })
        {
            // The original top-level glob resolver does not resolve `super` or
            // sibling module paths. It fails open for those imports.
            ok = false;
            continue;
        }
        let mut seen = HashSet::new();
        let (names, resolved) = exported_module_names(
            dir,
            &[],
            &m[1].split("::").map(str::to_owned).collect::<Vec<_>>(),
            manifests,
            files,
            deps_by_dir,
            names_by_path,
            &mut seen,
            0,
        );
        all.extend(names);
        ok &= resolved;
    }
    (all, ok)
}

fn module_file(
    dir: &Path,
    parts: &[String],
    files: &BTreeMap<PathBuf, &SourceFile>,
) -> Option<PathBuf> {
    let mut path = dir.join("src");
    for part in parts {
        path.push(part);
    }
    if parts.is_empty() {
        for candidate in [dir.join("src/lib.rs"), dir.join("lib.rs")] {
            if files.contains_key(&candidate) {
                return Some(candidate);
            }
        }
        return None;
    }
    for candidate in [path.with_extension("rs"), path.join("mod.rs")] {
        if files.contains_key(&candidate) {
            return Some(candidate);
        }
    }
    // `#[path = "..." ] mod child;`, relative to the declaration file.
    let parent_parts = &parts[..parts.len() - 1];
    let parent = module_file(dir, parent_parts, files)?;
    let text = &files.get(&parent)?.text;
    let attr = crate::regexes::cached(
        r#"(?m)#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*;"#,
    )
    .unwrap();
    for capture in attr.captures_iter(text) {
        if capture[2] == parts[parts.len() - 1] {
            let candidate = normalize(&parent.parent()?.join(&capture[1]));
            if files.contains_key(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn exported_module_names(
    dir: &Path,
    module: &[String],
    request: &[String],
    manifests: &BTreeMap<PathBuf, &SourceFile>,
    files: &BTreeMap<PathBuf, &SourceFile>,
    deps_by_dir: &BTreeMap<PathBuf, Vec<PathBuf>>,
    names_by_path: &BTreeMap<PathBuf, BTreeSet<String>>,
    seen: &mut HashSet<PathBuf>,
    depth: usize,
) -> (BTreeSet<String>, bool) {
    if depth > 6 {
        return (BTreeSet::new(), false);
    }
    let (base, parts, child_module) = match request.first().map(String::as_str) {
        Some("crate") => (dir.to_owned(), request[1..].to_vec(), request[1..].to_vec()),
        Some("super") => {
            let mut target = module.to_vec();
            target.pop();
            target.extend_from_slice(&request[1..]);
            (dir.to_owned(), target.clone(), target)
        }
        Some(extern_name) => {
            if let Some(dep) = deps_by_dir.get(dir).into_iter().flatten().find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().replace('-', "_") == extern_name)
            }) {
                (dep.clone(), request[1..].to_vec(), request[1..].to_vec())
            } else {
                let mut target = module.to_vec();
                target.pop();
                target.extend_from_slice(request);
                (dir.to_owned(), target.clone(), target)
            }
        }
        None => return (BTreeSet::new(), false),
    };
    let Some(file) = module_file(&base, &parts, files) else {
        return (BTreeSet::new(), false);
    };
    exported_file_names(
        &base,
        &child_module,
        &file,
        manifests,
        files,
        deps_by_dir,
        names_by_path,
        seen,
        depth,
    )
}

fn exported_file_names(
    base: &Path,
    child_module: &[String],
    file: &Path,
    manifests: &BTreeMap<PathBuf, &SourceFile>,
    files: &BTreeMap<PathBuf, &SourceFile>,
    deps_by_dir: &BTreeMap<PathBuf, Vec<PathBuf>>,
    names_by_path: &BTreeMap<PathBuf, BTreeSet<String>>,
    seen: &mut HashSet<PathBuf>,
    depth: usize,
) -> (BTreeSet<String>, bool) {
    if depth > 6 {
        return (BTreeSet::new(), false);
    }
    if !seen.insert(file.to_owned()) {
        return (BTreeSet::new(), true);
    }
    let Some(source) = files.get(file) else {
        return (BTreeSet::new(), false);
    };
    let mut names = names_by_path
        .get(file)
        .cloned()
        .unwrap_or_else(|| names_in_text(&source.text));
    let attrs = crate::regexes::cached(
        r#"(?m)#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*;"#,
    )
    .unwrap();
    let mut path_mods: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for cap in attrs.captures_iter(&source.text) {
        path_mods
            .entry(cap[2].to_owned())
            .or_default()
            .push(normalize(
                &file.parent().unwrap_or(Path::new("")).join(&cap[1]),
            ));
    }
    let glob = crate::regexes::cached(
        r"(?m)^\s*(?:#\[[^\]]*\]\s*)*pub\s+use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;",
    )
    .unwrap();
    let mut resolved = true;
    for cap in glob.captures_iter(&source.text) {
        let segs = cap[1].split("::").map(str::to_owned).collect::<Vec<_>>();
        let (sub, ok) = if segs.len() == 1 && path_mods.contains_key(&segs[0]) {
            let mut sub = BTreeSet::new();
            let mut all_ok = true;
            for path in &path_mods[&segs[0]] {
                // Python's resolver follows each existing #[path] target, retaining
                // the current module context. A missing target is ignored (legacy
                // fail-open behavior), while nested unresolved globs still fail open.
                if files.contains_key(path) {
                    let (child, child_ok) = exported_file_names(
                        base,
                        child_module,
                        path,
                        manifests,
                        files,
                        deps_by_dir,
                        names_by_path,
                        seen,
                        depth + 1,
                    );
                    sub.extend(child);
                    all_ok &= child_ok;
                }
            }
            (sub, all_ok)
        } else {
            exported_module_names(
                base,
                child_module,
                &segs,
                manifests,
                files,
                deps_by_dir,
                names_by_path,
                seen,
                depth + 1,
            )
        };
        names.extend(sub);
        resolved &= ok;
    }
    (names, resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_binding_but_not_real_patterns() {
        assert_eq!(
            candidates("match x {\n FOO => {},\n Some(X) => {},\n A | B => {},\n _ => {},\n}")
                .iter()
                .map(|x| x.1.as_str())
                .collect::<Vec<_>>(),
            vec!["FOO"]
        );
    }
    #[test]
    fn multiline_enum_variants_are_in_the_name_universe() {
        let text = "pub enum I8042_DEVICE_TYPE {\n Unknown,\n Keyboard,\n Mouse,\n PhysicalDeviceObject,\n}";
        let names = names_in_text(text);
        assert!(names.contains("Keyboard"));
        assert!(names.contains("Mouse"));
        assert!(names.contains("Unknown"));
    }
    #[test]
    fn names_and_shadowing() {
        let exported = names_in_text("pub use dep::{_BIND as Bind, C};");
        assert!(exported.contains("Bind") && exported.contains("C"));
        let (n, g) = local_names(
            "const FOO: u32 = 1; use crate::x::{BAR as BAZ}; let mut QUUX=0; use crate::m::*;",
        );
        assert!(n.contains("FOO") && n.contains("BAR") && n.contains("BAZ") && n.contains("QUUX"));
        assert!(g);
    }
    #[test]
    fn parses_quoted_path_dependencies_followed_by_manifest_punctuation() {
        let owner = SourceFile::new(
            "sdk/lib/rust/a/Cargo.toml".into(),
            "[dependencies]\nb = { path = \"../b\", features = [\"x\"] }\nc = { path = \"../c\" }"
                .into(),
        );
        let b = SourceFile::new("sdk/lib/rust/b/Cargo.toml".into(), String::new());
        let c = SourceFile::new("sdk/lib/rust/c/Cargo.toml".into(), String::new());
        let manifests = BTreeMap::from([
            (owner.path.clone(), &owner),
            (b.path.clone(), &b),
            (c.path.clone(), &c),
        ]);
        assert_eq!(
            crate_path_deps(Path::new("sdk/lib/rust/a"), &manifests),
            vec![
                PathBuf::from("sdk/lib/rust/b"),
                PathBuf::from("sdk/lib/rust/c")
            ]
        );
    }

    #[test]
    fn resolves_dependency_glob_reexports_from_snapshot() {
        let rows = vec![
            SourceFile::new(
                "sdk/lib/rust/a/Cargo.toml".into(),
                "[package]\nname='a'".into(),
            ),
            SourceFile::new("sdk/lib/rust/a/src/lib.rs".into(), "pub mod api;".into()),
            SourceFile::new(
                "sdk/lib/rust/a/src/api.rs".into(),
                "pub use crate::values::*;".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/values.rs".into(),
                "pub const BAD: u32 = 1;".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/use.rs".into(),
                "use crate::api::*;".into(),
            ),
        ];
        let manifests = BTreeMap::from([(PathBuf::from("sdk/lib/rust/a/Cargo.toml"), &rows[0])]);
        let files = rows
            .iter()
            .map(|f| (f.path.clone(), f))
            .collect::<BTreeMap<_, _>>();
        let (names, ok) = glob_names(
            &rows[4].text,
            Path::new("sdk/lib/rust/a"),
            &manifests,
            &files,
        );
        assert!(ok);
        assert!(names.contains("BAD"));
    }
    #[test]
    fn path_module_reexports_follow_nested_globs_and_fail_open_on_inline_modules() {
        let rows = vec![
            SourceFile::new(
                "sdk/lib/rust/a/Cargo.toml".into(),
                "[package]\nname='a'\n[dependencies]\nntdll_sys = { path='../ntdll-sys' }".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/lib.rs".into(),
                "pub mod prelude; pub mod types;".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/prelude.rs".into(),
                "pub use crate::types::*;".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/types.rs".into(),
                "#[path=\"types_arch.rs\"] mod types_arch;\npub use types_arch::*;".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/types_arch.rs".into(),
                "pub use ntdll_sys::nt::*;".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/a/src/use.rs".into(),
                "use crate::prelude::*;\nmatch x {\n  WM_INITDIALOG => {},\n  _ => {},\n}".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/ntdll-sys/Cargo.toml".into(),
                "[package]\nname='ntdll-sys'".into(),
            ),
            SourceFile::new(
                "sdk/lib/rust/ntdll-sys/src/lib.rs".into(),
                "pub mod nt { pub const WM_INITDIALOG: u32 = 272; }".into(),
            ),
        ];
        let manifests = rows
            .iter()
            .filter(|f| f.path.file_name().is_some_and(|n| n == "Cargo.toml"))
            .map(|f| (f.path.clone(), f))
            .collect::<BTreeMap<_, _>>();
        let files = rows
            .iter()
            .filter(|f| f.path.extension().is_some_and(|e| e == "rs"))
            .map(|f| (f.path.clone(), f))
            .collect::<BTreeMap<_, _>>();
        let user = files.get(Path::new("sdk/lib/rust/a/src/use.rs")).unwrap();
        let (names, resolved) =
            glob_names(&user.text, Path::new("sdk/lib/rust/a"), &manifests, &files);
        assert!(!names.contains("WM_INITDIALOG"));
        assert!(
            !resolved,
            "inline dependency modules are unresolved by Python too: {:?}",
            names
        );
    }

    #[test]
    fn live_binding_fixture_reports_only_unknown_bare_match_arms() {
        let root = "sdk/lib/rust/unit-rust";
        let files = vec![
            SourceFile::new(format!("{root}/Cargo.toml").into(), "[package]\nname='unit-rust'\nversion='0.1.0'".into()),
            SourceFile::new(format!("{root}/src/lib.rs").into(), "mod bad; mod safe;".into()),
            SourceFile::new(format!("{root}/src/bad.rs").into(), "pub fn f(x:u32) { match x {\n MISSING => {},\n OTHER => {},\n _ => {},\n } }".into()),
            SourceFile::new(format!("{root}/src/safe.rs").into(), "const KNOWN:u32=1; enum E { VALUE } pub fn f(x:u32) { match x {\n KNOWN => {},\n VALUE => {},\n _ => {},\n } }".into()),
        ];
        let corpus = Corpus::new("/repo".into(), files);
        let lines = VerifyMatchConsts.run(&corpus).unwrap();
        assert!(lines
            .iter()
            .any(|line| line.contains("bad.rs:2: [HIGH] `MISSING`")));
        assert!(lines
            .iter()
            .any(|line| line.contains("bad.rs:3: [HIGH] `OTHER`")));
        assert!(!lines.iter().any(|line| line.contains("safe.rs:")));
        assert!(lines.last().unwrap().contains("SUMMARY: HIGH=2"));
    }
}
