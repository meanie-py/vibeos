//! Safe Ninja closure planning; deletion is a separate explicit operation.
use crate::{read, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub outputs: Vec<String>,
    pub inputs: Vec<String>,
}
pub fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '$' {
            if let Some(n) = it.next() {
                if !matches!(n, ' ' | ':' | '$') {
                    out.push('$');
                }
                out.push(n);
            } else {
                out.push('$');
            }
        } else {
            out.push(c);
        }
    }
    out
}
pub fn split(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cur = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '$' {
            cur.push(c);
            if let Some(n) = it.next() {
                cur.push(n);
            }
        } else if c == ' ' {
            if !cur.is_empty() {
                result.push(unescape(&cur));
                cur.clear();
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        result.push(unescape(&cur));
    }
    result
}
pub fn logical_lines(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for line in s.split('\n') {
        if line.ends_with('$') && !line.ends_with("$$") {
            buf.push_str(&line[..line.len() - 1]);
        } else {
            buf.push_str(line);
            out.push(std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}
pub fn colon(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'$' => i += 2,
            b':' => return Some(i),
            _ => i += 1,
        }
    }
    None
}
/// Follow every include and fail on missing, unreadable, or malformed build statements.
pub fn parse_edges(manifest: &Path) -> Result<Vec<Edge>> {
    let base = manifest
        .parent()
        .ok_or_else(|| Error::new("manifest has no parent"))?;
    let base_canonical = base.canonicalize()?;
    let mut edges = Vec::new();
    let mut seen = BTreeSet::new();
    let mut todo = vec![manifest.to_owned()];
    while let Some(path) = todo.pop() {
        let canonical = path
            .canonicalize()
            .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
        if !canonical.starts_with(&base_canonical) {
            return Err(Error::new(format!(
                "manifest include escapes build directory: {}",
                path.display()
            )));
        }
        if !seen.insert(canonical) {
            continue;
        }
        let data = read(&path)?;
        for (line_no, line) in logical_lines(&data).iter().enumerate() {
            if line.starts_with("include ") || line.starts_with("subninja ") {
                let name = line.split_once(' ').unwrap().1.trim();
                if name.is_empty() {
                    return Err(Error::new(format!(
                        "{}:{}: empty include",
                        path.display(),
                        line_no + 1
                    )));
                }
                todo.push(base.join(unescape(name)));
                continue;
            }
            let Some(body) = line.strip_prefix("build ") else {
                continue;
            };
            let c = colon(body).ok_or_else(|| {
                Error::new(format!(
                    "{}:{}: malformed build statement",
                    path.display(),
                    line_no + 1
                ))
            })?;
            let outs = split(&body[..c].replace(" | ", " "))
                .into_iter()
                .filter(|x| x != "|")
                .collect::<Vec<_>>();
            let rest = body[c + 1..].trim().split(" |@ ").next().unwrap();
            let Some(rule) = rest.split_whitespace().next() else {
                return Err(Error::new(format!(
                    "{}:{}: missing build rule",
                    path.display(),
                    line_no + 1
                )));
            };
            if outs.is_empty() {
                return Err(Error::new(format!(
                    "{}:{}: missing build output",
                    path.display(),
                    line_no + 1
                )));
            }
            let ins = rest[rule.len()..].trim_start();
            edges.push(Edge {
                outputs: outs,
                inputs: split(ins)
                    .into_iter()
                    .filter(|t| t != "|" && t != "||")
                    .collect(),
            });
        }
    }
    Ok(edges)
}
pub fn closure_outputs<F: Fn(&str) -> String>(
    edges: &[Edge],
    targets: &[String],
    norm: F,
) -> (BTreeSet<String>, Vec<String>) {
    let mut index = BTreeMap::new();
    for (i, e) in edges.iter().enumerate() {
        for o in &e.outputs {
            index.insert(norm(o), i);
        }
    }
    let mut keep = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut q = VecDeque::new();
    let mut unknown = Vec::new();
    for t in targets {
        let n = norm(t);
        if index.contains_key(&n) {
            q.push_back(n)
        } else {
            unknown.push(t.clone())
        }
    }
    while let Some(n) = q.pop_front() {
        let Some(&i) = index.get(&n) else { continue };
        if !visited.insert(i) {
            continue;
        }
        for o in &edges[i].outputs {
            keep.insert(norm(o));
        }
        for x in &edges[i].inputs {
            q.push_back(norm(x));
        }
    }
    (keep, unknown)
}
pub fn logged_outputs(build: &Path) -> Result<Vec<String>> {
    let log = build.join(".ninja_log");
    if !log.exists() {
        return Ok(vec![]);
    }
    let text = read(&log)?;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for l in text.lines() {
        if l.starts_with('#') {
            continue;
        }
        let parts: Vec<_> = l.split('\t').collect();
        if parts.len() >= 4 && seen.insert(parts[3].to_owned()) {
            out.push(parts[3].to_owned());
        }
    }
    Ok(out)
}
fn normalize(p: &str) -> String {
    let mut stack = Vec::new();
    let absolute = p.starts_with('/');
    for part in p.split('/') {
        match part {
            "" | "." => {}
            ".." if stack.last().is_some_and(|x| *x != "..") => {
                stack.pop();
            }
            ".." if !absolute => stack.push(".."),
            ".." => {}
            _ => stack.push(part),
        }
    }
    let mut s = stack.join("/");
    if absolute {
        s.insert(0, '/');
    }
    if s.is_empty() {
        s.push('.');
    }
    s
}
fn safe_relative(build: &Path, rel: &str) -> Result<Option<PathBuf>> {
    let p = Path::new(rel);
    if p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(Error::new(format!("unsafe output path: {rel}")));
    }
    let mut cur = build.to_owned();
    for part in p.components() {
        cur.push(part.as_os_str());
        match std::fs::symlink_metadata(&cur) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(Error::new(format!(
                        "refusing symlink traversal: {}",
                        cur.display()
                    )));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(if cur.is_file() { Some(cur) } else { None })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub build: PathBuf,
    pub victims: Vec<String>,
    pub unknown: Vec<String>,
}
pub fn plan(build: &Path, targets: &[String], canon_root: Option<&str>) -> Result<Plan> {
    let build = build.canonicalize()?;
    if !build.is_dir() {
        return Err(Error::new(format!(
            "{} is not a directory",
            build.display()
        )));
    }
    let edges = parse_edges(&build.join("build.ninja"))?;
    let mut roots = vec![format!("{}/", build.display())];
    if let Some(c) = canon_root {
        roots.push(format!("{}/", c.trim_end_matches('/')));
    }
    let norm = |p: &str| {
        for r in &roots {
            if let Some(v) = p.strip_prefix(r) {
                return v.to_owned();
            }
        }
        normalize(p)
    };
    let (keep, unknown) = closure_outputs(&edges, targets, &norm);
    let mut victims = BTreeSet::new();
    for p in logged_outputs(&build)? {
        let rel = norm(&p);
        if rel.starts_with('/') || rel.starts_with("..") {
            continue;
        }
        if keep.contains(&rel) {
            continue;
        }
        if safe_relative(&build, &rel)?.is_some() {
            victims.insert(rel);
        }
    }
    Ok(Plan {
        build,
        victims: victims.into_iter().collect(),
        unknown,
    })
}
/// Refuse unknown targets and preflight every victim before deleting any.
pub fn delete(build: &Path, plan: &Plan) -> Result<usize> {
    if !plan.unknown.is_empty() {
        return Err(Error::new(format!(
            "unknown target(s): {}",
            plan.unknown.join(" ")
        )));
    }
    let build = build.canonicalize()?;
    if build != plan.build {
        return Err(Error::new(
            "cleanup plan belongs to a different build directory",
        ));
    }
    let mut paths = Vec::new();
    for rel in &plan.victims {
        if let Some(p) = safe_relative(&build, rel)? {
            paths.push(p);
        }
    }
    for p in &paths {
        std::fs::remove_file(p)?;
    }
    Ok(paths.len())
}

/// Hard fail when `build/build.ninja` predates an input of its own regeneration edge
/// (a CMakeLists.txt, a .cmake module, CMakeCache.txt), i.e. when ninja would re-run cmake
/// before building anything. Readers that derive state from build.ninja (the cell and
/// media manifests, the cell generator) otherwise emit a plan of the PREVIOUS configure
/// without a word: after a merge, `cell-manifest --write` silently wrote master's cells
/// and dropped a wave's 61. The regeneration edge is restat, so ninja's own record of
/// build.ninja's mtime (.ninja_log) can be newer than the file; the newer of the two
/// counts, as in ninja. A build.ninja with no regeneration edge (a test fixture) is
/// accepted. Each build directory is checked once per process.
pub fn require_configured(build: &Path, canon_root: &Path) -> Result<()> {
    use std::sync::{Mutex, OnceLock};
    static DONE: OnceLock<Mutex<BTreeSet<PathBuf>>> = OnceLock::new();
    let done = DONE.get_or_init(|| Mutex::new(BTreeSet::new()));
    if done.lock().unwrap().contains(build) {
        return Ok(());
    }
    let manifest = build.join("build.ninja");
    let text = read(&manifest)?;
    let Some(edge) = logical_lines(&text)
        .into_iter()
        .find(|l| l.starts_with("build build.ninja:"))
    else {
        done.lock().unwrap().insert(build.to_owned());
        return Ok(());
    };
    let implicit = edge.split_once(" | ").map(|(_, r)| r).unwrap_or("");
    let implicit = implicit.split(" || ").next().unwrap_or("");
    let mtime = |p: &Path| -> Option<u128> {
        let t = std::fs::metadata(p).ok()?.modified().ok()?;
        Some(t.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos())
    };
    let mut reference = mtime(&manifest).unwrap_or(0);
    if let Ok(log) = std::fs::read_to_string(build.join(".ninja_log")) {
        for line in log.lines() {
            let f: Vec<_> = line.split('\t').collect();
            if f.len() == 5 && f[3] == "build.ninja" {
                if let Ok(m) = f[2].parse::<u128>() {
                    reference = reference.max(m);
                }
            }
        }
    }
    let mut newer = Vec::new();
    for raw in split(implicit) {
        let input = unescape(&raw);
        let path = match input.strip_prefix("/tmp/vibeos/") {
            Some(rel) => canon_root.join(rel),
            None if Path::new(&input).is_absolute() => PathBuf::from(&input),
            None => build.join(&input),
        };
        match mtime(&path) {
            Some(m) if m <= reference => {}
            Some(_) => newer.push(input),
            None => newer.push(format!("{input} (missing)")),
        }
    }
    if !newer.is_empty() {
        let name = build
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return Err(Error::new(format!(
            "{name}/build.ninja is older than {} of its cmake inputs ({}{}): it describes the \
             previous configure. Reconfigure first: (cd {name} && bash ../tools/canon-run.sh \
             ninja build.ninja)",
            newer.len(),
            newer.iter().take(3).cloned().collect::<Vec<_>>().join(", "),
            if newer.len() > 3 { ", ..." } else { "" }
        )));
    }
    done.lock().unwrap().insert(build.to_owned());
    Ok(())
}

#[cfg(test)]
mod configured_tests {
    use super::require_configured;
    use std::{fs, thread::sleep, time::Duration};

    fn tree(name: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("m")).unwrap();
        fs::write(root.path().join("m/CMakeLists.txt"), "add_library(m)\n").unwrap();
        sleep(Duration::from_millis(20));
        let build = root.path().join(name);
        fs::create_dir_all(&build).unwrap();
        fs::write(
            build.join("build.ninja"),
            "build build.ninja: RERUN_CMAKE CMakeFiles/cmake.verify_globs | \
             /tmp/vibeos/m/CMakeLists.txt CMakeCache.txt\n",
        )
        .unwrap();
        fs::write(build.join("CMakeCache.txt"), "").unwrap();
        // The cache is written after build.ninja here; date build.ninja through the log.
        sleep(Duration::from_millis(20));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        fs::write(
            build.join(".ninja_log"),
            format!("# ninja log v5\n0\t1\t{now}\tbuild.ninja\tabc\n"),
        )
        .unwrap();
        root
    }

    #[test]
    fn a_configure_newer_than_every_input_passes() {
        let root = tree("build-amd64");
        require_configured(&root.path().join("build-amd64"), root.path()).unwrap();
    }

    #[test]
    fn a_cmakelists_edited_after_the_configure_is_refused() {
        let root = tree("build-i386");
        sleep(Duration::from_millis(20));
        fs::write(root.path().join("m/CMakeLists.txt"), "add_library(m n.c)\n").unwrap();
        let e = require_configured(&root.path().join("build-i386"), root.path())
            .unwrap_err()
            .to_string();
        assert!(e.contains("/tmp/vibeos/m/CMakeLists.txt"), "{e}");
        assert!(e.contains("canon-run.sh ninja build.ninja"), "{e}");
    }

    #[test]
    fn a_build_ninja_without_a_regeneration_edge_is_not_a_configure() {
        let root = tempfile::tempdir().unwrap();
        let build = root.path().join("build-arm64");
        fs::create_dir_all(&build).unwrap();
        fs::write(build.join("build.ninja"), "build a.exe: LINK a.o\n").unwrap();
        require_configured(&build, root.path()).unwrap();
    }
}
