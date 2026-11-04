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
