//! Build receipts: has this lane's code been built, so the gate's compiler caches are warm?
//!
//! WHY (2026-09-26): a merge that forced a full kernel recompile hit the merge queue's COLD
//! gate build with nothing in the shared ccache/sccache, took hours per arch, timed out, and
//! held nine lanes for ~80 minutes. Building in your own tree first fills the same shared
//! cache; every successful `vibeos-cli build` leaves a receipt, and `land` wants one per arch.
//!
//! A receipt is keyed on the WORKING TREE at build time — the tree `git add -A` would commit
//! — not HEAD, since agents build with uncommitted edits and commit after. It is computed in
//! a throwaway index seeded from the real one (so git's stat cache applies); the real index
//! is never touched.
//!
//! CLOSE ENOUGH COUNTS: a receipt covers a tree when every differing path is inert (the
//! caller's docs predicate — `tools/docs_only.py`, shared with the pre-push hook, so it stays
//! there and arrives here as data), or Python the build cannot reach
//! ([`build_reachable_python`]), or identical to master (the lane merged master after
//! building; master's own gate already warmed that code).
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const RECEIPT_DIRNAME: &str = "vibeos-build-receipts";
pub const MAX_AGE_S: f64 = 14.0 * 24.0 * 3600.0;
/// Tracked files that can make the build run Python: CMake, cargo build scripts and the
/// reactos-link crate they share, sdk/tools (build-time Python itself), shell drivers.
/// Comments count too — a false reference only reports cold, the safe direction.
const BUILD_GRAPH: [&str; 7] = [
    "*CMakeLists.txt",
    "*.cmake",
    "*build.rs",
    "sdk/lib/rust/reactos-link",
    "sdk/tools",
    "*.sh",
    "*.cmd",
];
/// How the build names a Python file: a path or bare basename (often behind a CMake
/// variable), or a module (`-m rustconv.c_oracle`). The same shape in POSIX ERE for git grep.
const PY_REF: &str = r"([A-Za-z0-9_-]+)\.py\b|-m\s+([A-Za-z_][\w.]*)";
const GREP_PY_REF: &str = r"[A-Za-z0-9_-]+\.py|-m[[:space:]]+[A-Za-z_][A-Za-z0-9_.]*";
const IMPORT: &str = r"(?m)^\s*(?:from\s+\.*([\w.]*)\s+import\s+([\w, ()]+)|import\s+([\w., ]+))";
const NEUTRAL_ROOTS: [&str; 2] = ["tools/", "tests/"];

fn git_with(repo: &Path, args: &[&str], index: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(repo);
    if let Some(index) = index {
        cmd.env("GIT_INDEX_FILE", index);
    }
    let out = crate::process::capture(&mut cmd, 600)?;
    if out.code != 0 {
        return Err(Error::new(format!(
            "git {} failed: {}",
            args.join(" "),
            out.stderr.trim()
        )));
    }
    Ok(out.stdout.trim().to_owned())
}
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    git_with(repo, args, None)
}
fn absolute(repo: &Path, p: String) -> PathBuf {
    let p = PathBuf::from(p);
    if p.is_absolute() {
        p
    } else {
        repo.join(p)
    }
}

pub fn receipts_dir(repo: &Path) -> Result<PathBuf> {
    let common = absolute(repo, git(repo, &["rev-parse", "--git-common-dir"])?);
    Ok(common
        .canonicalize()
        .unwrap_or(common)
        .join(RECEIPT_DIRNAME))
}

/// The tree `git add -A && git commit` would record right now.
pub fn worktree_tree(repo: &Path) -> Result<String> {
    let index = absolute(repo, git(repo, &["rev-parse", "--git-path", "index"])?);
    let tmp = tempfile::Builder::new()
        .prefix("vibeos-warmth-")
        .tempdir()?;
    let scratch = tmp.path().join("index");
    if index.is_file() {
        fs::copy(&index, &scratch)?;
    }
    git_with(repo, &["add", "-A"], Some(&scratch))?;
    git_with(repo, &["write-tree"], Some(&scratch))
}

/// Leave a receipt for a SUCCESSFUL build of `arch`. Returns the tree it names.
pub fn record(repo: &Path, arch: &str, now: f64) -> Result<String> {
    let tree = worktree_tree(repo)?;
    let d = receipts_dir(repo)?;
    fs::create_dir_all(&d)?;
    let rec = json!({
        "tree": tree,
        "arch": arch,
        "worktree": crate::prune::resolve(repo),
        "at": now as i64,
    });
    let tmp = d.join(format!(".{tree}.{arch}.tmp"));
    fs::write(&tmp, rec.to_string())?;
    fs::rename(&tmp, d.join(format!("{tree}.{arch}.json")))?;
    prune_receipts(&d, now);
    Ok(tree)
}

fn mtime(p: &Path) -> Option<f64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(p)
        .ok()
        .map(|m| m.mtime() as f64 + m.mtime_nsec() as f64 / 1e9)
}
fn prune_receipts(d: &Path, now: f64) {
    for e in fs::read_dir(d).into_iter().flatten().flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "json")
            && mtime(&p).is_some_and(|m| now - m > MAX_AGE_S)
        {
            let _ = fs::remove_file(&p);
        }
    }
}

/// Trees with a receipt for `arch`, newest first.
fn receipt_trees(repo: &Path, arch: &str) -> Vec<String> {
    let Ok(d) = receipts_dir(repo) else {
        return Vec::new();
    };
    let suffix = format!(".{arch}.json");
    let mut found: Vec<(f64, String)> = fs::read_dir(d)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.ends_with(&suffix) {
                return None;
            }
            let tree = name.split('.').next().unwrap_or("").to_owned();
            Some((mtime(&e.path())?, tree))
        })
        .collect();
    found.sort_by(|a, b| b.0.total_cmp(&a.0));
    found.into_iter().map(|(_, t)| t).collect()
}

fn changed(repo: &Path, a: &str, b: &str) -> Option<BTreeSet<String>> {
    git(repo, &["diff", "--name-only", a, b]).ok().map(|out| {
        out.lines()
            .filter(|l| !l.trim().is_empty())
            .map(str::to_owned)
            .collect()
    })
}

fn names(dotted: &str) -> Vec<String> {
    let split = Regex::new(r"[.,\s()]+").expect("static regex");
    split
        .split(dotted)
        .filter(|p| !p.is_empty() && *p != "as")
        .map(str::to_owned)
        .collect()
}

/// The tracked tools/ and tests/ `.py` files the build might execute, or None when that
/// cannot be worked out (then nothing is treated as neutral).
///
/// Roots are the Python every [`BUILD_GRAPH`] file names, plus THIS worktree's generated
/// build-*/build.ninja (never a sibling's); then everything a reachable file imports, to a
/// fixpoint. Matching is by NAME, so a same-named file anywhere makes every file of that name
/// reachable: over-approximation only ever reports cold, the safe direction.
pub fn build_reachable_python(repo: &Path) -> Option<BTreeSet<String>> {
    let tracked = git(repo, &["ls-files", "--", "tools/*.py", "tests/*.py"]).ok()?;
    let mut grep = Command::new("git");
    grep.args(["grep", "-h", "-o", "-I", "-E", GREP_PY_REF, "--"])
        .args(BUILD_GRAPH)
        .current_dir(repo);
    let out = crate::process::capture(&mut grep, 600).ok()?;
    if out.code != 0 && out.code != 1 {
        return None; // 1 is "no match": a real answer
    }
    let mut text = vec![out.stdout];
    let mut ninjas: Vec<PathBuf> = fs::read_dir(repo)
        .ok()?
        .flatten()
        .map(|e| e.path().join("build.ninja"))
        .filter(|p| {
            p.parent()
                .and_then(|d| d.file_name())
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("build-"))
                && p.is_file()
        })
        .collect();
    ninjas.sort();
    for n in ninjas {
        text.push(String::from_utf8_lossy(&fs::read(n).ok()?).into_owned());
    }
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in tracked.lines().filter(|l| !l.is_empty()) {
        let stem = p.strip_suffix(".py").unwrap_or(p);
        let parts: Vec<&str> = stem.split('/').collect();
        let mut keys = BTreeSet::from([parts[parts.len() - 1].to_owned()]);
        if parts[parts.len() - 1] == "__init__" && parts.len() > 1 {
            keys.insert(parts[parts.len() - 2].to_owned());
        }
        for k in keys {
            by_name.entry(k).or_default().push(p.to_owned());
        }
    }
    let py_ref = Regex::new(PY_REF).expect("static regex");
    let mut wanted = BTreeSet::new();
    for c in py_ref.captures_iter(&text.join("\n")) {
        match (c.get(1), c.get(2)) {
            (Some(file), _) => {
                wanted.insert(file.as_str().to_owned());
            }
            (None, Some(module)) => wanted.extend(names(module.as_str())),
            _ => {}
        }
    }
    let import = Regex::new(IMPORT).expect("static regex");
    let mut reachable = BTreeSet::new();
    let mut pending: Vec<String> = wanted
        .iter()
        .flat_map(|n| by_name.get(n).cloned().unwrap_or_default())
        .collect();
    while let Some(p) = pending.pop() {
        if !reachable.insert(p.clone()) {
            continue;
        }
        let Ok(bytes) = fs::read(repo.join(&p)) else {
            continue;
        };
        let src = String::from_utf8_lossy(&bytes);
        for c in import.captures_iter(&src) {
            let joined = (1..=3)
                .filter_map(|i| c.get(i).map(|m| m.as_str()))
                .collect::<Vec<_>>()
                .join(" ");
            for name in names(&joined) {
                for q in by_name.get(&name).into_iter().flatten() {
                    if !reachable.contains(q) {
                        pending.push(q.clone());
                    }
                }
            }
        }
    }
    Some(reachable)
}

/// The receipts `missing` would weigh for `arch`: None when `tree` itself has one, else up to
/// `scan` (newest first) with the paths each differs by (None when git cannot diff them).
fn candidates(
    repo: &Path,
    tree: &str,
    arch: &str,
    scan: usize,
) -> Option<Vec<Option<BTreeSet<String>>>> {
    let trees = receipt_trees(repo, arch);
    if trees.iter().any(|t| t == tree) {
        return None;
    }
    Some(
        trees
            .iter()
            .take(scan)
            .map(|t| changed(repo, t, tree))
            .collect(),
    )
}

/// Every path the caller's inert predicate must judge for [`missing`].
pub fn diff_paths(repo: &Path, tree: &str, arches: &[String], scan: usize) -> BTreeSet<String> {
    arches
        .iter()
        .filter_map(|a| candidates(repo, tree, a, scan))
        .flatten()
        .flatten()
        .flatten()
        .collect()
}

/// The arches with no receipt close enough to `tree` to have warmed the cache for it.
/// `inert` is the caller's verdict on [`diff_paths`].
pub fn missing(
    repo: &Path,
    tree: &str,
    arches: &[String],
    inert: &BTreeSet<String>,
    master: Option<&str>,
    scan: usize,
) -> Vec<String> {
    // Only the lane's OWN changes can be cold: what master changed, master's gate built.
    let own = master.and_then(|m| changed(repo, m, tree));
    let mut reach: Option<Option<BTreeSet<String>>> = None; // a repo grep: computed on need
    let mut neutral = |p: &str| -> bool {
        if !(p.ends_with(".py") && NEUTRAL_ROOTS.iter().any(|r| p.starts_with(r))) {
            return false;
        }
        reach
            .get_or_insert_with(|| build_reachable_python(repo))
            .as_ref()
            .is_some_and(|r| !r.contains(p))
    };
    let mut out = Vec::new();
    for arch in arches {
        let Some(diffs) = candidates(repo, tree, arch, scan) else {
            continue;
        };
        let warm = diffs.into_iter().flatten().any(|diff| {
            diff.iter().all(|p| {
                inert.contains(p) || own.as_ref().is_some_and(|o| !o.contains(p)) || neutral(p)
            })
        });
        if !warm {
            out.push(arch.clone());
        }
    }
    out
}

fn path_arg(a: &Value, k: &str) -> Result<PathBuf> {
    a[k].as_str()
        .map(PathBuf::from)
        .ok_or_else(|| Error::new(format!("missing {k}")))
}
fn strings(v: &Value) -> Vec<String> {
    serde_json::from_value(v.clone()).unwrap_or_default()
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let scan = a["scan"].as_u64().unwrap_or(20) as usize;
    let tree = || a["tree"].as_str().ok_or_else(|| Error::new("missing tree"));
    match request["op"].as_str().unwrap_or("") {
        "warmth_worktree_tree" => Ok(json!(worktree_tree(&path_arg(a, "repo")?)?)),
        "warmth_record" => Ok(json!(record(
            &path_arg(a, "repo")?,
            a["arch"]
                .as_str()
                .ok_or_else(|| Error::new("missing arch"))?,
            a["now"].as_f64().unwrap_or_else(crate::trim::now),
        )?)),
        "warmth_receipts_dir" => Ok(json!(receipts_dir(&path_arg(a, "repo")?)?)),
        "warmth_build_reachable_python" => Ok(json!(build_reachable_python(&path_arg(a, "repo")?))),
        "warmth_diff_paths" => Ok(json!(diff_paths(
            &path_arg(a, "repo")?,
            tree()?,
            &strings(&a["arches"]),
            scan
        ))),
        "warmth_missing" => Ok(json!(missing(
            &path_arg(a, "repo")?,
            tree()?,
            &strings(&a["arches"]),
            &strings(&a["inert"]).into_iter().collect(),
            a["master"].as_str(),
            scan,
        ))),
        op => Err(format!("unknown warmth operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_and_import_names_split_like_the_python() {
        assert_eq!(names("rustconv.c_oracle"), ["rustconv", "c_oracle"]);
        assert_eq!(names("a as b, (c)"), ["a", "b", "c"]);
        let import = Regex::new(IMPORT).unwrap();
        let src = "import os, json\nfrom .warmth import (missing, record)\n  from tools import x\n";
        let got: Vec<String> = import
            .captures_iter(src)
            .flat_map(|c| {
                (1..=3)
                    .filter_map(|i| c.get(i).map(|m| m.as_str().to_owned()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            got,
            ["os, json", "warmth", "(missing, record)", "tools", "x"]
        );
        let py = Regex::new(PY_REF).unwrap();
        let refs: Vec<String> = py
            .captures_iter("${SRC}/tools/gen_cflags.py; python -m rustconv.c_oracle x.pyc")
            .map(|c| c.get(1).or(c.get(2)).unwrap().as_str().to_owned())
            .collect();
        assert_eq!(refs, ["gen_cflags", "rustconv.c_oracle"]);
    }
}
