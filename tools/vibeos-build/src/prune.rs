//! `vibeos-cli prune`: decide which idle worktrees' build + Rust target dirs may be reclaimed.
//!
//! Build trees are ephemeral scratch — ccache/sccache rehydrate a pruned tree fast — so an
//! idle worktree's 26-42 G can be reclaimed and regenerated on demand. Everything here is the
//! DECISION; the Python facade (`tests/qemu/cli/prune.py`) still owns the maintenance lock,
//! the printout and the delete loop, which rechecks [`live_qemu_worktrees`] and
//! [`active_build_worktrees`] before every directory.
//!
//! A worktree is protected, most-protective first, when it is the current tree (unless
//! `--self`), the merge-queue integration tree (unless opted in), has a live QEMU, has a
//! running compiler/linker/ninja, or was touched inside the idle window. Only
//! [`PRUNE_RELDIRS`] are ever candidates — never source, never the worktree itself.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// The global lock dir vm.py / qemu/start.sh / `vibeos-cli ps` use.
pub const LOCK_DIR: &str = "/tmp/vibeos-qemu-locks";

/// Per-worktree dirs treated as disposable build scratch.
pub const PRUNE_RELDIRS: [&str; 5] = [
    "build-i386",
    "build-amd64",
    "build-arm64",
    "sdk/lib/rust/target",
    "ntoskrnl/rust-kernel-target",
];

/// Process names that mean "a build is in flight" — never prune their worktree.
const BUILD_PROCS: [&str; 13] = [
    "ninja",
    "cargo",
    "rustc",
    "cc1",
    "cc1plus",
    "clang",
    "clang++",
    "lld",
    "ld.lld",
    "cmake",
    "make",
    "mcopy",
    "mkramdisk.sh",
];

const INTEGRATION_DIRNAME: &str = "vibeos-integration";
const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

fn refused(message: impl Into<String>) -> Error {
    Error::classified("maintenance", message)
}
fn not_found(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::NotFound
}

/// `pathlib.Path.resolve()` (non-strict): the longest existing prefix canonicalized, the
/// rest appended lexically.
pub fn resolve(path: &Path) -> PathBuf {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut clean = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::ParentDir => {
                clean.pop();
            }
            Component::CurDir => {}
            c => clean.push(c),
        }
    }
    let mut tail = Vec::new();
    let mut head = clean.as_path();
    loop {
        if let Ok(real) = head.canonicalize() {
            return tail.iter().rev().fold(real, |p: PathBuf, n| p.join(n));
        }
        match (head.parent(), head.file_name()) {
            (Some(parent), Some(name)) => {
                tail.push(name.to_os_string());
                head = parent;
            }
            _ => return clean,
        }
    }
}

/// The integrator's tree: `vibeos-integration` or `vibeos-integration-<n>`.
pub fn is_integration_dirname(name: &str) -> bool {
    name == INTEGRATION_DIRNAME
        || name
            .strip_prefix(INTEGRATION_DIRNAME)
            .and_then(|r| r.strip_prefix('-'))
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// All worktree roots from `git worktree list --porcelain`; empty when git cannot answer.
pub fn git_worktrees(repo: &Path) -> Vec<PathBuf> {
    let Ok(out) = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo)
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .map(PathBuf::from)
        .collect()
}

/// Which worktree (if any) contains this path. Longest match wins, so a nested worktree is
/// not mis-attributed to its parent.
pub fn attribute(path: &Path, worktrees: &[PathBuf]) -> Option<PathBuf> {
    let p = resolve(path);
    let mut best: Option<(&PathBuf, usize)> = None;
    for wt in worktrees {
        let wtr = resolve(wt);
        let len = wtr.as_os_str().len();
        if p.starts_with(&wtr) && best.is_none_or(|(_, l)| len > l) {
            best = Some((wt, len));
        }
    }
    best.map(|(wt, _)| wt.clone())
}

fn pid_alive(pid: i64) -> bool {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return false;
    };
    // SAFETY: signal 0 only probes for existence and permission.
    if unsafe { libc::kill(pid, 0) } == 0 {
        return true;
    }
    io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}
fn is_qemu(pid: i64) -> bool {
    fs::read_to_string(format!("/proc/{pid}/comm")).is_ok_and(|c| c.contains("qemu"))
}

/// Worktrees with a live QEMU, via the global lock files (the `ps` source of truth).
pub fn live_qemu_worktrees(lock_dir: &Path, worktrees: &[PathBuf]) -> BTreeSet<PathBuf> {
    let mut live = BTreeSet::new();
    let Ok(entries) = fs::read_dir(lock_dir) else {
        return live;
    };
    for e in entries.flatten() {
        let path = e.path();
        if path.extension().is_none_or(|x| x != "json") {
            continue;
        }
        let Some(info) = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        else {
            continue;
        };
        let Some(qpid) = info["qemu_pid"].as_i64().filter(|&p| p != 0) else {
            continue;
        };
        if !(pid_alive(qpid) && is_qemu(qpid)) {
            continue;
        }
        // Any path-bearing field attributes the run to its worktree.
        for key in ["live_log", "work_dir", "manifest_path"] {
            if let Some(val) = info[key].as_str().filter(|v| !v.is_empty()) {
                if let Some(wt) = attribute(Path::new(val), worktrees) {
                    live.insert(resolve(&wt));
                    break;
                }
            }
        }
    }
    live
}

/// A zombie (or dead) process: its build is over, whatever its cwd said. Unreadable but
/// present stays live.
fn exited(entry: &Path) -> bool {
    match fs::read_to_string(entry.join("status")) {
        Ok(status) => status
            .lines()
            .find_map(|l| l.strip_prefix("State:"))
            .and_then(|s| s.split_whitespace().next())
            .is_some_and(|s| s == "Z" || s == "X"),
        Err(_) => !entry.exists(),
    }
}

fn mountinfo(path: &Path) -> Vec<Vec<String>> {
    fs::read_to_string(path)
        .map(|t| {
            t.lines()
                .map(|l| l.split_whitespace().map(str::to_owned).collect())
                .collect()
        })
        .unwrap_or_default()
}

/// The host path bind-mounted at `canonical` in one process's mount namespace, or None.
///
/// mountinfo field 4 is the mount's root WITHIN its filesystem and field 3 the device, so the
/// host path is wherever the host mounts that device's root, joined with it. Every lane's
/// build runs at the one canonical path, so without this any single build made the whole
/// fleet look "in flight" and prune could reclaim nothing while the merge queue gated
/// (2026-10-12: 0 of 816 G eligible with only the integration trees building).
pub fn canonical_bind_source(
    mountinfo_path: &Path,
    canonical: &Path,
    host_mountinfo: &Path,
) -> Option<PathBuf> {
    let canonical = canonical.to_str()?;
    let f = mountinfo(mountinfo_path)
        .into_iter()
        .find(|f| f.len() >= 5 && f[4] == canonical)?;
    let (dev, root) = (&f[2], &f[3]);
    mountinfo(host_mountinfo)
        .into_iter()
        .find(|h| h.len() >= 5 && &h[2] == dev && h[3] == "/")
        .map(|h| Path::new(&h[4]).join(root.trim_start_matches('/')))
}

/// Worktrees with a running build process, read from `proc_root` (`/proc`).
///
/// An unattributable live compiler protects the whole fleet: it is not evidence of an idle
/// tree. A zombie protects nothing — mid-build, compilers exit constantly and each one's cwd
/// stops resolving before it is reaped, which once made one finished cc1 hold the fleet.
pub fn active_build_worktrees(
    proc_root: &Path,
    canonical: &Path,
    host_mountinfo: &Path,
    worktrees: &[PathBuf],
) -> io::Result<BTreeSet<PathBuf>> {
    let everything = || worktrees.iter().map(|w| resolve(w)).collect::<Vec<_>>();
    let mut active = BTreeSet::new();
    for e in fs::read_dir(proc_root)?.flatten() {
        let entry = e.path();
        let name = e.file_name();
        if !name
            .to_str()
            .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
        {
            continue;
        }
        let Ok(comm) = fs::read_to_string(entry.join("comm")) else {
            continue;
        };
        if !BUILD_PROCS.contains(&comm.trim()) {
            continue;
        }
        let cwd = match fs::read_link(entry.join("cwd")) {
            Ok(c) => c,
            Err(_) => {
                if entry.exists() && !exited(&entry) {
                    active.extend(everything());
                }
                continue;
            }
        };
        // Bind-mount builds report the canonical path, not their physical worktree; the bind
        // source in the process's own mount table names it. When that cannot be read,
        // protect the fleet rather than misclassify an orphan compiler as idle after its CLI
        // parent (and shared lease) has died.
        if cwd.starts_with(canonical) {
            match canonical_bind_source(&entry.join("mountinfo"), canonical, host_mountinfo) {
                // A tree outside `worktrees` (one `--only` left out) is no evidence about
                // the trees we were asked about.
                Some(source) => {
                    if let Some(wt) = attribute(&source, worktrees) {
                        active.insert(resolve(&wt));
                    }
                }
                None if entry.exists() && !exited(&entry) => active.extend(everything()),
                None => {}
            }
            continue;
        }
        if let Some(wt) = attribute(&cwd, worktrees) {
            active.insert(resolve(&wt));
        }
    }
    Ok(active)
}

/// Python's `{x:g}`, enough for the hour counts this prints.
fn fmt_g(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e6 {
        format!("{}", x as i64)
    } else if x.abs() >= 1e6 {
        let s = format!("{x:.5e}");
        let (m, e) = s.split_once('e').unwrap_or((&s, "0"));
        let m = m.trim_end_matches('0').trim_end_matches('.');
        let e: i32 = e.parse().unwrap_or(0);
        format!("{m}e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
    } else {
        let s = format!("{x:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

/// What protects one worktree, if anything.
pub struct Gates<'a> {
    pub current: &'a Path,
    pub live_qemu: &'a BTreeSet<PathBuf>,
    pub active_build: &'a BTreeSet<PathBuf>,
    pub allow_self: bool,
    pub include_integration: bool,
    pub min_age_hours: f64,
    pub now: f64,
}

/// The eligibility decision: a skip reason, or None if prunable. Most-protective first.
pub fn classify(worktree: &Path, g: &Gates, newest_mtime: Option<f64>) -> Option<String> {
    let wtr = resolve(worktree);
    if wtr == resolve(g.current) && !g.allow_self {
        return Some("current worktree (use --self)".into());
    }
    // Preserve conductor scratch for ordinary manual pruning. Scheduled retention
    // explicitly includes it, subject to the same idle window and activity lock.
    let name = wtr
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    if is_integration_dirname(&name) && !g.allow_self && !g.include_integration {
        return Some("merge-queue integration tree (its warm caches gate every batch)".into());
    }
    if g.live_qemu.contains(&wtr) {
        return Some("live QEMU running".into());
    }
    if g.active_build.contains(&wtr) {
        return Some("build in flight".into());
    }
    if g.min_age_hours > 0.0 && newest_mtime.is_some_and(|m| (g.now - m) < g.min_age_hours * 3600.0)
    {
        return Some(format!("touched < {}h ago", fmt_g(g.min_age_hours)));
    }
    None
}

/// Allocated bytes under `path`, as `du -s -B1` reports them: st_blocks, each hardlinked inode
/// once (cargo hardlinks deps <-> final outputs), symlinks not followed, directories
/// included. A file that vanishes mid-walk is skipped (a live build rewrites its tree); any
/// other error is returned — a size that silently omits an unreadable subtree is wrong.
pub fn disk_usage_bytes(path: &Path) -> io::Result<u64> {
    let mut seen = HashSet::new();
    let mut total = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(d) = stack.pop() {
        match fs::symlink_metadata(&d) {
            Ok(m) => total += m.blocks() * 512,
            Err(e) if not_found(&e) => continue,
            Err(e) => return Err(e),
        }
        let entries = match fs::read_dir(&d) {
            Ok(it) => it,
            Err(e) if not_found(&e) => continue,
            Err(e) => return Err(e),
        };
        for e in entries {
            let e = e?;
            // d_type first: a directory is lstat'ed once, when it is popped.
            let is_dir = match e.file_type() {
                Ok(t) => t.is_dir(),
                Err(err) if not_found(&err) => continue,
                Err(err) => return Err(err),
            };
            if is_dir {
                stack.push(e.path());
                continue;
            }
            // DirEntry::metadata does not follow symlinks.
            let m = match e.metadata() {
                Ok(m) => m,
                Err(err) if not_found(&err) => continue,
                Err(err) => return Err(err),
            };
            if m.nlink() > 1 && !seen.insert((m.dev(), m.ino())) {
                continue;
            }
            total += m.blocks() * 512;
        }
    }
    Ok(total)
}

fn mtime(m: &fs::Metadata) -> f64 {
    m.mtime() as f64 + m.mtime_nsec() as f64 / 1e9
}
fn newest_below(dir: &Path, best: &mut f64) -> io::Result<()> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(()); // os.walk skips unreadable dirs
    };
    for e in entries.flatten() {
        let p = e.path();
        match fs::symlink_metadata(&p) {
            Ok(m) => {
                *best = best.max(mtime(&m));
                if m.is_dir() {
                    newest_below(&p, best)?;
                }
            }
            Err(err) if not_found(&err) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}
/// Newest descendant mtime across the prunable dirs: rewriting an object need not touch its
/// root dir. None when the worktree has none of them.
pub fn newest_mtime(worktree: &Path) -> io::Result<Option<f64>> {
    let mut best: Option<f64> = None;
    for rel in PRUNE_RELDIRS {
        let d = worktree.join(rel);
        if !d.exists() {
            continue;
        }
        let mut b = mtime(&fs::metadata(&d)?);
        if let Some(prev) = best {
            b = b.max(prev);
        }
        newest_below(&d, &mut b)?;
        best = Some(b);
    }
    Ok(best)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub worktree: PathBuf,
    pub dirs: Vec<(PathBuf, u64)>,
    pub eligible: bool,
    pub skip_reason: Option<String>,
}
impl Candidate {
    pub fn total(&self) -> u64 {
        self.dirs.iter().map(|(_, s)| s).sum()
    }
    fn to_json(&self) -> Value {
        json!({
            "worktree": self.worktree,
            "dirs": self.dirs.iter().map(|(p, s)| json!([p, s])).collect::<Vec<_>>(),
            "eligible": self.eligible,
            "skip_reason": self.skip_reason,
        })
    }
}

/// Retention budget: keep the smallest eligible outputs while the whole fleet fits in
/// `budget` GiB. Protected outputs stay protected even if they alone exceed it.
pub fn select_budget(candidates: &[Candidate], budget: Option<f64>) -> Result<Vec<Candidate>> {
    let Some(budget) = budget else {
        return Ok(candidates.to_vec());
    };
    if !budget.is_finite() || budget < 0.0 {
        return Err(Error::classified(
            "budget",
            "--retain-gib must be finite and nonnegative",
        ));
    }
    let mut remaining: u64 = candidates.iter().map(Candidate::total).sum();
    let mut sorted = candidates.to_vec();
    sorted.sort_by_key(|c| std::cmp::Reverse(c.total()));
    Ok(sorted
        .into_iter()
        .map(|mut c| {
            if c.eligible && remaining as f64 <= budget * GIB {
                c.eligible = false;
                c.skip_reason = Some("within retention budget".into());
            } else if c.eligible {
                remaining -= c.total();
            }
            c
        })
        .collect())
}

pub struct PlanRequest {
    pub repo: PathBuf,
    pub worktrees: Vec<PathBuf>,
    pub live: BTreeSet<PathBuf>,
    pub active: BTreeSet<PathBuf>,
    pub now: f64,
    pub allow_self: bool,
    pub include_integration: bool,
    pub idle_hours: f64,
    pub retain_gib: Option<f64>,
    pub min_free_gib: Option<f64>,
    /// Free bytes on the repo's filesystem; needed only with `min_free_gib`.
    pub free_bytes: Option<f64>,
}
pub struct Plan {
    pub candidates: Vec<Candidate>,
    pub budget: Option<f64>,
}

pub fn plan(r: &PlanRequest) -> Result<Plan> {
    let mut min_age = r.idle_hours;
    if (r.retain_gib.is_some() || r.min_free_gib.is_some()) && min_age == 0.0 {
        min_age = 24.0; // bounded fleet retention must not reclaim newly completed work
    }
    if !min_age.is_finite() || min_age < 0.0 {
        return Err(refused("--idle-hours must be finite and nonnegative"));
    }
    let gates = Gates {
        current: &r.repo,
        live_qemu: &r.live,
        active_build: &r.active,
        allow_self: r.allow_self,
        include_integration: r.include_integration,
        min_age_hours: min_age,
        now: r.now,
    };
    let mut candidates = Vec::new();
    for wt in &r.worktrees {
        let mut dirs = Vec::new();
        for rel in PRUNE_RELDIRS {
            let d = wt.join(rel);
            if d.exists() {
                let size = disk_usage_bytes(&d)
                    .map_err(|e| refused(format!("cannot measure {}: {e}", d.display())))?;
                dirs.push((d, size));
            }
        }
        if dirs.is_empty() {
            continue; // nothing to reclaim
        }
        let newest = if min_age > 0.0 {
            newest_mtime(wt)?
        } else {
            None
        };
        let reason = classify(wt, &gates, newest);
        candidates.push(Candidate {
            worktree: wt.clone(),
            dirs,
            eligible: reason.is_none(),
            skip_reason: reason,
        });
    }

    let mut budget = r.retain_gib;
    if let Some(min_free) = r.min_free_gib {
        if !min_free.is_finite() || min_free < 0.0 {
            return Err(refused("--min-free-gib must be finite and nonnegative"));
        }
        let device = fs::metadata(&r.repo)?.dev();
        for c in &candidates {
            for (d, _) in &c.dirs {
                if fs::metadata(d)?.dev() != device {
                    return Err(refused(
                        "headroom retention requires outputs on one filesystem",
                    ));
                }
            }
        }
        let free = r
            .free_bytes
            .ok_or_else(|| Error::new("min_free_gib needs free_bytes"))?;
        let shortfall = (min_free * GIB - free).max(0.0);
        let total: u64 = candidates.iter().map(Candidate::total).sum();
        let headroom = (total as f64 - shortfall).max(0.0) / GIB;
        budget = Some(budget.map_or(headroom, |b| b.min(headroom)));
    }
    let mut candidates = select_budget(&candidates, budget)?;
    candidates.sort_by_key(|c| std::cmp::Reverse(c.total()));
    Ok(Plan { candidates, budget })
}

fn paths(v: &Value) -> Vec<PathBuf> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}
/// A number, null, or a Python `repr(float)` string ("inf", "nan") — JSON has no spelling
/// for the non-finite values the validators exist to refuse.
fn num(v: &Value) -> Result<Option<f64>> {
    match v {
        Value::Null => Ok(None),
        Value::Number(n) => Ok(n.as_f64()),
        Value::Bool(b) => Ok(Some(f64::from(u8::from(*b)))),
        Value::String(s) => s
            .parse::<f64>()
            .map(Some)
            .map_err(|_| Error::new(format!("not a number: {s}"))),
        other => Err(Error::new(format!("not a number: {other}"))),
    }
}
fn path_arg(a: &Value, k: &str) -> Result<PathBuf> {
    a[k].as_str()
        .map(PathBuf::from)
        .ok_or_else(|| Error::new(format!("missing {k}")))
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let set = |s: BTreeSet<PathBuf>| json!(s.into_iter().collect::<Vec<_>>());
    match request["op"].as_str().unwrap_or("") {
        "prune_git_worktrees" => Ok(json!(git_worktrees(&path_arg(a, "repo")?))),
        "prune_attribute" => Ok(json!(attribute(
            &path_arg(a, "path")?,
            &paths(&a["worktrees"])
        ))),
        "prune_live_qemu" => {
            let lock_dir = a["lock_dir"].as_str().unwrap_or(LOCK_DIR);
            Ok(set(live_qemu_worktrees(
                Path::new(lock_dir),
                &paths(&a["worktrees"]),
            )))
        }
        "prune_active_builds" => Ok(set(active_build_worktrees(
            Path::new(a["proc_root"].as_str().unwrap_or("/proc")),
            &path_arg(a, "canonical")?,
            Path::new(
                a["host_mountinfo"]
                    .as_str()
                    .unwrap_or("/proc/self/mountinfo"),
            ),
            &paths(&a["worktrees"]),
        )?)),
        "prune_plan" => {
            let p = plan(&PlanRequest {
                repo: path_arg(a, "repo")?,
                worktrees: paths(&a["worktrees"]),
                live: paths(&a["live"]).into_iter().collect(),
                active: paths(&a["active"]).into_iter().collect(),
                now: num(&a["now"])?.unwrap_or_else(crate::trim::now),
                allow_self: a["allow_self"].as_bool().unwrap_or(false),
                include_integration: a["include_integration"].as_bool().unwrap_or(false),
                idle_hours: num(&a["idle_hours"])?.unwrap_or(0.0),
                retain_gib: num(&a["retain_gib"])?,
                min_free_gib: num(&a["min_free_gib"])?,
                free_bytes: num(&a["free_bytes"])?,
            })?;
            Ok(json!({
                "candidates": p.candidates.iter().map(Candidate::to_json).collect::<Vec<_>>(),
                "budget": p.budget,
            }))
        }
        op => Err(format!("unknown prune operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn set(items: &[&str]) -> BTreeSet<PathBuf> {
        items.iter().map(PathBuf::from).collect()
    }
    fn gates<'a>(
        current: &'a Path,
        live: &'a BTreeSet<PathBuf>,
        active: &'a BTreeSet<PathBuf>,
    ) -> Gates<'a> {
        Gates {
            current,
            live_qemu: live,
            active_build: active,
            allow_self: false,
            include_integration: false,
            min_age_hours: 0.0,
            now: 1000.0,
        }
    }

    #[test]
    fn current_live_and_building_trees_are_protected() {
        let (none, cur) = (set(&[]), Path::new("/wt/a"));
        let g = gates(cur, &none, &none);
        assert_eq!(
            classify(cur, &g, None).as_deref(),
            Some("current worktree (use --self)")
        );
        assert_eq!(
            classify(
                cur,
                &Gates {
                    allow_self: true,
                    ..g
                },
                None
            ),
            None
        );
        let b = Path::new("/wt/b");
        let live = set(&["/wt/b"]);
        assert_eq!(
            classify(b, &gates(cur, &live, &none), None).as_deref(),
            Some("live QEMU running")
        );
        assert_eq!(
            classify(b, &gates(cur, &none, &live), None).as_deref(),
            Some("build in flight")
        );
        assert_eq!(classify(b, &gates(cur, &none, &none), None), None);
    }

    #[test]
    fn idle_window_holds_recent_trees_and_zero_disables_it() {
        let none = set(&[]);
        let now = 100_000.0;
        let g = Gates {
            min_age_hours: 24.0,
            now,
            ..gates(Path::new("/wt/a"), &none, &none)
        };
        let b = Path::new("/wt/b");
        assert_eq!(
            classify(b, &g, Some(now - 3600.0)).as_deref(),
            Some("touched < 24h ago")
        );
        assert_eq!(classify(b, &g, Some(now - 48.0 * 3600.0)), None);
        assert_eq!(
            classify(
                b,
                &Gates {
                    min_age_hours: 0.0,
                    ..g
                },
                Some(now)
            ),
            None
        );
        assert_eq!(
            classify(
                b,
                &Gates {
                    min_age_hours: 0.5,
                    ..g
                },
                Some(now)
            )
            .as_deref(),
            Some("touched < 0.5h ago")
        );
    }

    #[test]
    fn integration_tree_needs_self_or_an_explicit_opt_in() {
        let none = set(&[]);
        let tree = Path::new("/fleet/vibeos-integration-2");
        let g = gates(Path::new("/fleet/other"), &none, &none);
        assert!(classify(tree, &g, None).is_some_and(|r| r.contains("integration")));
        assert_eq!(
            classify(
                tree,
                &Gates {
                    allow_self: true,
                    ..g
                },
                None
            ),
            None
        );
        let opted = Gates {
            include_integration: true,
            ..g
        };
        assert_eq!(classify(tree, &opted, None), None);
        let building = set(&["/fleet/vibeos-integration-2"]);
        assert_eq!(
            classify(
                tree,
                &Gates {
                    active_build: &building,
                    ..opted
                },
                None
            )
            .as_deref(),
            Some("build in flight")
        );
        assert!(!is_integration_dirname("vibeos-integration-x"));
        assert!(!is_integration_dirname("vibeos-integration-"));
        assert!(is_integration_dirname("vibeos-integration"));
    }

    #[test]
    fn attribution_prefers_the_longest_worktree() {
        let wts = vec![PathBuf::from("/wt"), PathBuf::from("/wt/nested")];
        let at = |p| attribute(Path::new(p), &wts);
        assert_eq!(
            at("/wt/nested/build-i386/x.o"),
            Some(PathBuf::from("/wt/nested"))
        );
        assert_eq!(at("/wt/build-i386/x.o"), Some(PathBuf::from("/wt")));
        assert_eq!(
            at("/wtx/build-i386/x.o"),
            None,
            "a name prefix is not containment"
        );
        assert_eq!(at("/elsewhere/x"), None);
    }

    #[test]
    fn budget_never_overrides_protection() {
        let c = |name: &str, gib: u64, eligible| Candidate {
            worktree: PathBuf::from("/fixture").join(name),
            dirs: vec![(
                PathBuf::from("/fixture").join(name).join("build-amd64"),
                gib << 30,
            )],
            eligible,
            skip_reason: None,
        };
        let all = vec![
            c("integration", 40, false),
            c("old", 30, true),
            c("warm", 20, true),
        ];
        let names = |r: Vec<Candidate>| {
            r.into_iter()
                .filter(|c| c.eligible)
                .map(|c| {
                    c.worktree
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(names(select_budget(&all, Some(60.0)).unwrap()), ["old"]);
        assert_eq!(
            all[2].skip_reason, None,
            "the planner does not mutate its input"
        );
        assert_eq!(
            names(select_budget(&all, Some(0.0)).unwrap()),
            ["old", "warm"]
        );
        for bad in [-1.0, f64::INFINITY, f64::NAN] {
            assert!(select_budget(&all, Some(bad)).is_err());
        }
    }

    #[test]
    fn age_sees_a_nested_object_write() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("build-amd64");
        let obj = root.join("nested/file.o");
        fs::create_dir_all(obj.parent().unwrap()).unwrap();
        fs::write(&obj, b"object").unwrap();
        let at = |p: &Path, t: i64| {
            let ts = libc::timespec {
                tv_sec: t,
                tv_nsec: 0,
            };
            let c = std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap();
            // SAFETY: valid NUL-terminated path and a two-element times array.
            assert_eq!(
                unsafe { libc::utimensat(libc::AT_FDCWD, c.as_ptr(), [ts, ts].as_ptr(), 0) },
                0
            );
        };
        at(&obj, 1000);
        at(obj.parent().unwrap(), 10);
        at(&root, 10);
        assert_eq!(newest_mtime(tmp.path()).unwrap(), Some(1000.0));
        assert_eq!(newest_mtime(&tmp.path().join("none")).unwrap(), None);
    }

    #[test]
    fn hardlinked_outputs_count_once() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("deps/lib.rlib");
        fs::create_dir_all(a.parent().unwrap()).unwrap();
        fs::write(&a, vec![7u8; 64 * 1024]).unwrap();
        let one = disk_usage_bytes(tmp.path()).unwrap();
        fs::hard_link(&a, tmp.path().join("lib.rlib")).unwrap();
        assert_eq!(disk_usage_bytes(tmp.path()).unwrap(), one);
        assert_eq!(disk_usage_bytes(&tmp.path().join("gone")).unwrap(), 0);
    }

    fn proc_entry(root: &Path, pid: &str, comm: &str, cwd: &str) -> PathBuf {
        let p = root.join(pid);
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("comm"), format!("{comm}\n")).unwrap();
        symlink(cwd, p.join("cwd")).unwrap();
        p
    }

    #[test]
    fn an_orphan_canonical_compiler_protects_the_fleet() {
        let tmp = tempfile::tempdir().unwrap();
        let proc = tmp.path().join("proc");
        proc_entry(&proc, "1001", "rustc", "/tmp/vibeos/sdk/lib/rust");
        let lanes = vec![tmp.path().join("one"), tmp.path().join("two")];
        let got = active_build_worktrees(
            &proc,
            Path::new("/tmp/vibeos"),
            &tmp.path().join("no-host-mountinfo"),
            &lanes,
        )
        .unwrap();
        assert_eq!(got, lanes.iter().map(|l| resolve(l)).collect());
    }

    #[test]
    fn a_canonical_compiler_is_attributed_by_its_bind_source() {
        let tmp = tempfile::tempdir().unwrap();
        let lanes = vec![tmp.path().join("one"), tmp.path().join("two")];
        for l in &lanes {
            fs::create_dir_all(l).unwrap();
        }
        let proc = tmp.path().join("proc");
        let p = proc_entry(&proc, "1001", "ninja", "/tmp/vibeos/build-i386");
        fs::write(
            p.join("mountinfo"),
            format!(
                "22 1 8:48 / / rw - ext4 /dev/sdd rw\n\
                 1063 1015 8:48 {} /tmp/vibeos rw,relatime - ext4 /dev/sdd rw\n",
                lanes[1].display()
            ),
        )
        .unwrap();
        let host = tmp.path().join("host-mountinfo");
        fs::write(&host, "22 1 8:48 / / rw - ext4 /dev/sdd rw\n").unwrap();
        let got = active_build_worktrees(&proc, Path::new("/tmp/vibeos"), &host, &lanes).unwrap();
        assert_eq!(got, [resolve(&lanes[1])].into_iter().collect());
    }

    #[test]
    fn a_zombie_compiler_protects_nothing_and_a_direct_cwd_is_attributed() {
        let tmp = tempfile::tempdir().unwrap();
        let lanes = vec![tmp.path().join("one"), tmp.path().join("two")];
        for l in &lanes {
            fs::create_dir_all(l.join("build-i386")).unwrap();
        }
        let proc = tmp.path().join("proc");
        let z = proc.join("1001");
        fs::create_dir_all(&z).unwrap();
        fs::write(z.join("comm"), "cc1\n").unwrap();
        fs::write(z.join("status"), "Name:\tcc1\nState:\tZ (zombie)\n").unwrap();
        proc_entry(&proc, "1002", "bash", lanes[0].to_str().unwrap());
        proc_entry(&proc, "self", "ninja", lanes[0].to_str().unwrap());
        let host = tmp.path().join("none");
        let canon = Path::new("/tmp/vibeos");
        assert!(active_build_worktrees(&proc, canon, &host, &lanes)
            .unwrap()
            .is_empty());
        proc_entry(
            &proc,
            "1003",
            "clang",
            lanes[1].join("build-i386").to_str().unwrap(),
        );
        assert_eq!(
            active_build_worktrees(&proc, canon, &host, &lanes).unwrap(),
            [resolve(&lanes[1])].into_iter().collect()
        );
    }

    #[test]
    fn headroom_selects_extra_outputs_but_recent_outputs_survive() {
        let tmp = tempfile::tempdir().unwrap();
        let (old, recent) = (tmp.path().join("old"), tmp.path().join("recent"));
        for lane in [&old, &recent] {
            fs::create_dir_all(lane.join("build-amd64")).unwrap();
            fs::write(lane.join("build-amd64/obj"), "disposable").unwrap();
        }
        let now = crate::trim::now();
        let request = PlanRequest {
            repo: tmp.path().to_path_buf(),
            worktrees: vec![old.clone(), recent.clone()],
            live: BTreeSet::new(),
            active: BTreeSet::new(),
            // `old` was last touched just now; seen from two days on it is idle, and `recent`
            // is protected by being rewritten below at that later time's clock.
            now: now + 48.0 * 3600.0,
            allow_self: false,
            include_integration: false,
            idle_hours: 24.0,
            retain_gib: Some(100.0),
            min_free_gib: Some(150.0),
            free_bytes: Some(149.0 * GIB),
        };
        let ts = libc::timespec {
            tv_sec: (now + 48.0 * 3600.0) as i64,
            tv_nsec: 0,
        };
        let c = std::ffi::CString::new(
            recent
                .join("build-amd64/obj")
                .into_os_string()
                .into_encoded_bytes(),
        )
        .unwrap();
        // SAFETY: valid NUL-terminated path and a two-element times array.
        assert_eq!(
            unsafe { libc::utimensat(libc::AT_FDCWD, c.as_ptr(), [ts, ts].as_ptr(), 0) },
            0
        );
        let p = plan(&request).unwrap();
        let eligible: Vec<_> = p
            .candidates
            .iter()
            .filter(|c| c.eligible)
            .map(|c| &c.worktree)
            .collect();
        assert_eq!(eligible, [&old]);
        let held = p.candidates.iter().find(|c| c.worktree == recent).unwrap();
        assert_eq!(held.skip_reason.as_deref(), Some("touched < 24h ago"));
        assert_eq!(
            p.budget,
            Some(0.0),
            "a 1 GiB shortfall exceeds the whole fleet"
        );
    }

    #[test]
    fn retention_defaults_the_idle_window_and_refuses_bad_knobs() {
        let tmp = tempfile::tempdir().unwrap();
        let lane = tmp.path().join("lane");
        fs::create_dir_all(lane.join("build-i386")).unwrap();
        let base = || PlanRequest {
            repo: tmp.path().to_path_buf(),
            worktrees: vec![lane.clone()],
            live: BTreeSet::new(),
            active: BTreeSet::new(),
            now: crate::trim::now(),
            allow_self: false,
            include_integration: false,
            idle_hours: 0.0,
            retain_gib: Some(0.0),
            min_free_gib: None,
            free_bytes: None,
        };
        let p = plan(&base()).unwrap();
        assert_eq!(
            p.candidates[0].skip_reason.as_deref(),
            Some("touched < 24h ago"),
            "bounded retention must not reclaim newly completed work"
        );
        for r in [
            PlanRequest {
                idle_hours: f64::NAN,
                ..base()
            },
            PlanRequest {
                idle_hours: -1.0,
                ..base()
            },
            PlanRequest {
                min_free_gib: Some(f64::INFINITY),
                ..base()
            },
        ] {
            assert_eq!(
                plan(&r).err().and_then(|e| e.kind).as_deref(),
                Some("maintenance")
            );
        }
        let bad = PlanRequest {
            retain_gib: Some(-1.0),
            ..base()
        };
        assert_eq!(
            plan(&bad).err().and_then(|e| e.kind).as_deref(),
            Some("budget")
        );
        assert_eq!(num(&json!("inf")).unwrap(), Some(f64::INFINITY));
        assert!(num(&json!("nan")).unwrap().unwrap().is_nan());
    }

    #[test]
    fn hour_counts_print_like_python() {
        assert_eq!(fmt_g(24.0), "24");
        assert_eq!(fmt_g(0.5), "0.5");
        assert_eq!(fmt_g(1.25), "1.25");
        assert_eq!(fmt_g(1e6), "1e+06");
    }
}
