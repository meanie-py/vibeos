//! Build diagnostics and filesystem isolation guards, independent of presentation.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::{DirEntryExt, MetadataExt},
    path::{Path, PathBuf},
};
pub fn decoration(sym: &str) -> String {
    let hint = if Regex::new(r"^@\w+@\d+$").unwrap().is_match(sym) {
        "i386 fastcall — wrong extern or missing #[no_mangle]"
    } else if Regex::new(r"^_\w+@\d+$").unwrap().is_match(sym) {
        "i386 stdcall — likely missing #[no_mangle] or wrong extern"
    } else if Regex::new(r"^_\w+$").unwrap().is_match(sym) {
        "i386 cdecl — missing symbol / wrong ABI"
    } else {
        "undefined"
    };
    format!("{sym}: {hint}")
}
pub fn failed_edges(blob: &str) -> Vec<(String, Vec<String>)> {
    let status = Regex::new(r"^\[\d+/\d+\] ").unwrap();
    let mut edges: Vec<(String, Vec<String>)> = vec![];
    let mut active = false;
    for line in blob.lines() {
        if let Some(edge) = line.strip_prefix("FAILED: ") {
            edges.push((edge.trim().to_string(), vec![]));
            active = true;
        } else if active {
            if status.is_match(line) || line.starts_with("ninja: ") {
                active = false
            } else {
                edges.last_mut().unwrap().1.push(line.to_string())
            }
        }
    }
    for (_, lines) in &mut edges {
        if !lines.is_empty() {
            lines.remove(0);
        }
    }
    edges
}
pub fn enospc(blob: &str) -> bool {
    let low = blob.to_lowercase();
    [
        "no space left on device",
        "errno 28",
        "enospc",
        "disk quota exceeded",
        "write error: no space",
    ]
    .iter()
    .any(|m| low.contains(m))
}
pub fn classify(out: &str, err: &str, rc: Option<i64>, timeout: u64, disk: &[String]) -> Value {
    let full = format!("{out}\n{err}");
    let edges = failed_edges(&full);
    let blob = if edges.is_empty() {
        full.clone()
    } else {
        edges
            .iter()
            .flat_map(|(_, l)| l)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };
    let low = blob.to_lowercase();
    if rc == Some(124) {
        return json!([
            "BUILD_TIMEOUT",
            [
                format!("stage hit its {timeout}s time budget (exit 124): killed, not failed"),
                "re-run (ninja resumes), or raise the budget with VIBEOS_BUILD_TIMEOUT=<seconds>"
            ]
        ]);
    }
    if enospc(&full) {
        return json!(["DISK_FULL", disk]);
    }
    if low.contains("cmake error") {
        return json!([
            "CMAKE_CONFIGURE_FAIL",
            blob.lines()
                .filter(|s| s.contains("CMake Error"))
                .map(str::trim)
                .take(3)
                .collect::<Vec<_>>()
        ]);
    }
    let captures = |pattern: &str| {
        let re = Regex::new(pattern).unwrap();
        let mut seen = BTreeSet::new();
        re.captures_iter(&blob)
            .map(|m| m[1].to_string())
            .filter(|s| seen.insert(s.clone()))
            .collect::<Vec<_>>()
    };
    let undef = captures(r"(?i)undefined (?:symbol|reference)[^\w]*[`']?([@\w]+)");
    let dup = captures(r"(?i)error:\s*duplicate symbol[:\s]+[`']?([@\w]+)");
    if !undef.is_empty() || !dup.is_empty() || low.contains("ld.lld: error") {
        let mut sal = undef
            .iter()
            .take(3)
            .map(|s| decoration(s))
            .collect::<Vec<_>>();
        sal.extend(dup.iter().take(3).map(|s| format!("duplicate symbol: {s}")));
        if sal.is_empty() {
            sal = blob
                .lines()
                .filter(|s| s.contains("ld.lld"))
                .map(|s| s.trim().to_string())
                .take(3)
                .collect()
        }
        return json!(["LINK_FAIL", sal]);
    }
    if low.contains("error[e") || (low.contains("error:") && low.contains("cargo")) {
        return json!([
            "CARGO_CHECK_FAIL",
            blob.lines()
                .filter(|s| s.contains("error[") || s.contains("error:"))
                .map(str::trim)
                .take(3)
                .collect::<Vec<_>>()
        ]);
    }
    if let Some((edge, lines)) = edges.first() {
        let mut sal = vec![format!("FAILED: {edge}")
            .chars()
            .take(200)
            .collect::<String>()];
        sal.extend(
            lines
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .rev()
                .take(2)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(str::to_string),
        );
        return json!(["BUILD_FAIL", sal]);
    }
    json!([
        "BUILD_FAIL",
        blob.lines()
            .filter(|s| s.to_lowercase().contains("error"))
            .map(str::trim)
            .take(3)
            .collect::<Vec<_>>()
    ])
}
fn walk(root: &Path, prune: &BTreeSet<PathBuf>, visit: &mut impl FnMut(fs::DirEntry) -> bool) {
    let mut stack = vec![root.to_owned()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let Ok(t) = e.file_type() else { continue };
            if t.is_dir() {
                if !prune.contains(&p) {
                    stack.push(p)
                }
            } else if t.is_file() && !visit(e) {
                return;
            }
        }
    }
}
fn candidates(root: &Path, over: u64, prune: &BTreeSet<PathBuf>, max: usize) -> Vec<PathBuf> {
    let mut out = vec![];
    walk(root, prune, &mut |e| {
        if e.metadata().is_ok_and(|m| m.nlink() > over) {
            out.push(e.path());
        }
        out.len() < max
    });
    out
}
fn foreign(repo: &Path, candidates: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut info = BTreeMap::new();
    for p in candidates {
        if let Ok(m) = fs::metadata(&p) {
            info.entry((m.dev(), m.ino())).or_insert((m.nlink(), p));
        }
    }
    if info.is_empty() {
        return vec![];
    }
    let inodes = info.keys().map(|(_, ino)| *ino).collect::<BTreeSet<_>>();
    let mut counts = BTreeMap::new();
    walk(repo, &BTreeSet::new(), &mut |e| {
        if inodes.contains(&e.ino()) {
            if let Ok(m) = e.metadata() {
                let key = (m.dev(), m.ino());
                if info.contains_key(&key) {
                    *counts.entry(key).or_insert(0u64) += 1;
                }
            }
        }
        true
    });
    info.into_iter()
        .filter_map(|(key, (n, p))| (counts.get(&key).copied().unwrap_or(0) < n).then_some(p))
        .collect()
}
pub fn isolation(repo: &Path, build: &Path) -> Vec<String> {
    let mut hits = vec![];
    let mut roots = vec![];
    walk(build, &BTreeSet::new(), &mut |e| {
        if e.file_name() == "CACHEDIR.TAG" {
            if let Some(p) = e.path().parent() {
                roots.push(p.to_owned());
            }
        }
        true
    });
    roots.sort();
    let mut top = vec![];
    for d in roots {
        if !top.iter().any(|t: &PathBuf| d.starts_with(t)) {
            top.push(d)
        }
    }
    let prune = top.iter().cloned().collect();
    if let Some(p) = candidates(build, 1, &prune, 1).first() {
        hits.push(format!("{} (links 2+ in a cmake tree)", p.display()))
    }
    for root in top {
        for p in foreign(repo, candidates(&root, 2, &BTreeSet::new(), 200)) {
            hits.push(format!(
                "{} (links 3+ in a cargo dir, link escapes this worktree)",
                p.display()
            ))
        }
    }
    for p in foreign(
        repo,
        candidates(&repo.join("sdk/lib/rust/target"), 2, &BTreeSet::new(), 200),
    ) {
        hits.push(format!(
            "{} (links 3+ in a cargo dir, link escapes this worktree)",
            p.display()
        ))
    }
    hits
}
pub fn timeout(raw: &str) -> Result<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(3600);
    }
    let parsed = Regex::new(r"^[+-]?[0-9](?:_?[0-9])*$")?
        .is_match(raw)
        .then(|| raw.replace('_', "").parse::<i128>().ok())
        .flatten();
    let Some(value) = parsed else {
        return Err(format!("VIBEOS_BUILD_TIMEOUT={raw:?}: expected whole seconds").into());
    };
    if value <= 0 {
        return Err(format!("VIBEOS_BUILD_TIMEOUT={raw:?}: must be positive").into());
    }
    u64::try_from(value)
        .map_err(|_| Error::new("VIBEOS_BUILD_TIMEOUT exceeds the supported time budget"))
}
pub fn dispatch(_repo: &Path, r: &Value) -> Result<Value> {
    let a = &r["kwargs"];
    let s = |k: &str| a[k].as_str().unwrap_or("");
    match r["op"].as_str().unwrap_or("") {
        "cli_build_check_timeout" => Ok(json!(timeout(s("raw"))?)),
        "cli_build_check_keep" => Ok(json!(crate::cli_build::keep_going(&a["args"]))),
        "cli_build_check_decoration" => Ok(json!(decoration(s("symbol")))),
        "cli_build_check_edges" => Ok(json!(failed_edges(s("text")))),
        "cli_build_check_classify" => Ok(classify(
            s("stdout"),
            s("stderr"),
            a["rc"].as_i64(),
            a["timeout"].as_u64().unwrap_or(3600),
            &serde_json::from_value::<Vec<String>>(a.get("disk").cloned().unwrap_or(json!([])))?,
        )),
        "cli_build_check_enospc" => Ok(json!(enospc(s("text")))),
        "cli_build_check_isolation" => {
            let hits = isolation(Path::new(s("repo")), Path::new(s("build")));
            if hits.is_empty() {
                return Ok(json!({"ok":true}));
            }
            let mut salient = hits
                .iter()
                .map(|h| format!("first hit: {h}"))
                .collect::<Vec<_>>();
            salient.push("fix: recreate the affected private build outputs before retrying".into());
            salient.push("cause: a cross-worktree hardlink dedup ran — see tools/dedup-build-cache.sh SAFETY".into());
            Ok(
                json!({"ok":false,"classification":"ISOLATION_FAIL","stage":{"name":"isolation","status":"fail","detail":"build tree shares inodes with another worktree — refusing to build","salient":salient}}),
            )
        }
        op => Err(Error::new(format!("unknown build check operation {op}"))),
    }
}
