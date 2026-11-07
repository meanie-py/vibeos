//! `prune --trim` and the post-build trim: cut cargo target dirs back to what a build can
//! still read. Everything planned here is unreachable by the CURRENT build configuration, so
//! it runs on live lanes and never costs a warm build. The removal classes:
//!
//! * SUPERSEDED UNITS: cargo names outputs `<crate>-<hash>` where the hash covers toolchain,
//!   deps, flags and features but not source; a toolchain/dep/flag change orphans the old
//!   outputs forever (1-4 G per lane). Units are compared by cargo's own fingerprint identity
//!   (target, path, profile, features, compile kind): two units differing in any of those are
//!   BOTH live — one host `release/` holds `crt-shared-rust` with two feature sets.
//! * AD-HOC TARGET TREES: a hand-run `cargo check --target <triple>`, `target/debug`, or
//!   `cargo_orchestrate`'s private dirs (up to 8.4 G in one lane).
//! * DEAD CUSTOM TARGETS: a per-module `<triple-json-stem>/` whose cell no longer has that
//!   `<name>.json` (215 dirs / 27 G after the i386 kernel cells left `i686-kernel.json`).
//! * RETIRED LAYOUTS: `build-<arch>/**/rust-*-target` dirs build.ninja no longer names, the
//!   private `-cargo-link` / `-rustlib` dirs an arch stops reading once its cache enables
//!   link/body shards, and body-storage shards no build has acquired for three days.
//!
//! Nothing younger than the idle window (default 24h) is touched. Removing a still-wanted
//! unit costs at most one recompile (an sccache hit): cargo checks a unit's outputs exist
//! before calling it fresh.
//!
//! It also trims rustconv PORT STAGING caches (`~/vibeos-rust-staging/port/<stem>/target-*`):
//! pure cargo output beside paid artifacts that are FILES (state.json, banked fixes,
//! responses) and are never planned.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cargo target roots inside a worktree (the non-build-* half of prune's reldirs).
pub const TARGET_RELDIRS: [&str; 2] = ["sdk/lib/rust/target", "ntoskrnl/rust-kernel-target"];
const ARCHES: [&str; 3] = ["i386", "amd64", "arm64"];
// The fingerprint fields that make two units DIFFERENT things rather than versions of one.
// Left out on purpose: rustc, deps, rustflags, config, local — exactly what changes when a
// unit is superseded.
const IDENTITY: [&str; 5] = ["target", "path", "profile", "features", "compile_kind"];
const HASHED_SUBDIRS: [&str; 4] = ["deps", "build", "incremental", ".fingerprint"];
const MODULE_DIR_OTHER: [&str; 4] = ["release", "debug", "CACHEDIR.TAG", "tmp"];
pub const BODY_SHARD_IDLE_SECONDS: f64 = 72.0 * 3600.0;

fn re(p: &str) -> Regex {
    Regex::new(p).expect("static trim regex")
}
static ADHOC_TRIPLE: LazyLock<Regex> = LazyLock::new(|| {
    re(r"^(x86_64|i[3-6]86|aarch64|arm64ec|armv7|thumbv7a)-[a-z0-9_]+-[a-z0-9_]+(-[a-z0-9_]+)?$")
});
static ADHOC_ORCHESTRATE: LazyLock<Regex> =
    LazyLock::new(|| re(r"^orchestrate-(?:i386|amd64|arm64)-[0-9a-f]{12}$"));
static HASH: LazyLock<Regex> = LazyLock::new(|| re(r"-([0-9a-f]{16})(?:\.|$)"));
static CELL_LINK: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(?P<cell>.+?)-(?:i386|amd64|arm64)-cargo-link$"));
static CELL_RUSTLIB: LazyLock<Regex> = LazyLock::new(|| re(r"^(?P<cell>.+?-cargo)-.+-rustlib$"));
static BODY_SHARD: LazyLock<Regex> =
    LazyLock::new(|| re(r"^body-storage-v1-(?:i386|amd64|arm64)-[0-9a-f]{16}-\d+-rustlib$"));

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TrimPlan {
    pub paths: Vec<(PathBuf, u64)>,
    pub superseded_units: usize,
    pub adhoc_dirs: usize,
    pub dead_targets: usize,
    pub retired_dirs: usize,
}
impl TrimPlan {
    pub fn total(&self) -> u64 {
        self.paths.iter().map(|(_, s)| s).sum()
    }
    pub fn to_json(&self) -> Value {
        json!({
            "paths": self.paths.iter().map(|(p, s)| json!([p, s])).collect::<Vec<_>>(),
            "superseded_units": self.superseded_units,
            "adhoc_dirs": self.adhoc_dirs,
            "dead_targets": self.dead_targets,
            "retired_dirs": self.retired_dirs,
            "total": self.total(),
        })
    }
    pub fn from_json(v: &Value) -> Result<Self> {
        let paths = v["paths"]
            .as_array()
            .ok_or("trim plan needs paths")?
            .iter()
            .map(|row| {
                Ok((
                    PathBuf::from(row[0].as_str().ok_or("trim path must be a string")?),
                    row[1].as_u64().ok_or("trim size must be an integer")?,
                ))
            })
            .collect::<Result<_>>()?;
        let n = |k: &str| v[k].as_u64().unwrap_or(0) as usize;
        Ok(Self {
            paths,
            superseded_units: n("superseded_units"),
            adhoc_dirs: n("adhoc_dirs"),
            dead_targets: n("dead_targets"),
            retired_dirs: n("retired_dirs"),
        })
    }
}

fn not_found(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::NotFound
}
fn name_of(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}
fn is_symlink(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}
/// A real (non-symlink) directory.
fn real_dir(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.is_dir())
}
fn mtime(m: &fs::Metadata) -> f64 {
    m.mtime() as f64 + m.mtime_nsec() as f64 / 1e9
}
pub fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

/// `os.walk` (top-down, followlinks=False, unreadable dirs skipped): `visit(dir, subdirs)`
/// sees every directory entry that is a dir or a symlink to one, and prunes descent by
/// removing names. Symlinked dirs are listed but never entered.
fn walk(top: &Path, visit: &mut dyn FnMut(&Path, &mut Vec<String>)) {
    let Ok(entries) = fs::read_dir(top) else {
        return;
    };
    let mut dirs = Vec::new();
    for e in entries.flatten() {
        let p = e.path();
        if fs::metadata(&p).is_ok_and(|m| m.is_dir()) {
            dirs.push(name_of(&p));
        }
    }
    visit(top, &mut dirs);
    for d in dirs {
        let p = top.join(&d);
        if !is_symlink(&p) {
            walk(&p, visit);
        }
    }
}
/// Every entry below `top` (not `top` itself), without following symlinks.
fn each_below(top: &Path, f: &mut dyn FnMut(&Path) -> io::Result<()>) -> io::Result<()> {
    let Ok(entries) = fs::read_dir(top) else {
        return Ok(());
    };
    for e in entries.flatten() {
        let p = e.path();
        f(&p)?;
        if real_dir(&p) {
            each_below(&p, f)?;
        }
    }
    Ok(())
}

/// Allocated bytes of a path and everything below it, never following symlinks.
pub fn size(path: &Path) -> io::Result<u64> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if not_found(&e) => return Ok(0),
        Err(e) => return Err(e),
    };
    let mut total = meta.blocks() * 512;
    if !meta.is_dir() {
        return Ok(total);
    }
    each_below(path, &mut |p| {
        match fs::symlink_metadata(p) {
            Ok(m) => total += m.blocks() * 512,
            Err(e) if not_found(&e) => {}
            Err(e) => return Err(e),
        }
        Ok(())
    })?;
    Ok(total)
}
/// Newest lstat mtime of a path and everything below it. Missing `path` is an error (the
/// caller treats a vanished candidate as not idle); a vanished descendant is skipped.
fn newest_mtime(path: &Path) -> io::Result<f64> {
    let mut best = mtime(&fs::symlink_metadata(path)?);
    each_below(path, &mut |p| {
        match fs::symlink_metadata(p) {
            Ok(m) => best = best.max(mtime(&m)),
            Err(e) if not_found(&e) => {}
            Err(e) => return Err(e),
        }
        Ok(())
    })?;
    Ok(best)
}
/// Idle for at least `seconds`; a candidate that vanished is not idle.
fn idle_for(path: &Path, now: f64, seconds: f64) -> Result<bool> {
    match newest_mtime(path) {
        Ok(t) => Ok(now - t >= seconds),
        Err(e) if not_found(&e) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Canonical identity of one `.fingerprint/<pkg>-<hash>` dir, or None when it holds no
/// readable fingerprint json (cargo mid-write, or a layout we do not know).
pub fn unit_identity(fp_dir: &Path) -> Option<String> {
    let name = name_of(fp_dir);
    let pkg = name.rsplit_once('-').map_or(name.as_str(), |(p, _)| p);
    let mut files: Vec<PathBuf> = fs::read_dir(fp_dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let mut keys = Vec::new();
    for js in files {
        let data: Value = serde_json::from_str(&fs::read_to_string(&js).ok()?).ok()?;
        let obj = data.as_object()?;
        let mut key = vec![name_of(&js)];
        for k in IDENTITY {
            key.push(serde_json::to_string(obj.get(k)?).ok()?);
        }
        keys.push(key);
    }
    if keys.is_empty() {
        return None;
    }
    keys.sort();
    Some(serde_json::to_string(&json!([pkg, keys])).ok()?)
}

/// Unit hashes in one profile dir that a newer unit with the same identity replaced and
/// that are older than the idle window.
pub fn superseded_hashes(profile: &Path, now: f64, idle: f64) -> Result<BTreeSet<String>> {
    let mut groups: BTreeMap<String, Vec<(f64, String)>> = BTreeMap::new();
    for e in fs::read_dir(profile.join(".fingerprint"))?.flatten() {
        let fp = e.path();
        let n = name_of(&fp);
        let Some(m) = HASH.captures(&n) else { continue };
        if !fs::metadata(&fp).is_ok_and(|m| m.is_dir()) {
            continue;
        }
        let Some(id) = unit_identity(&fp) else {
            continue;
        };
        match newest_mtime(&fp) {
            Ok(t) => groups.entry(id).or_default().push((t, m[1].to_owned())),
            Err(e) if not_found(&e) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    let mut dead = BTreeSet::new();
    for units in groups.values_mut() {
        if units.len() < 2 {
            continue;
        }
        units.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let newest = units.last().unwrap().0;
        for (t, h) in &units[..units.len() - 1] {
            if *t < newest && now - t >= idle {
                dead.insert(h.clone());
            }
        }
    }
    Ok(dead)
}

fn profile_dirs(target_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(target_root, &mut |dir, dirs| {
        if dirs.iter().any(|d| d == ".fingerprint") {
            out.push(dir.to_path_buf());
        }
        dirs.retain(|d| !HASHED_SUBDIRS.contains(&d.as_str()));
    });
    out
}
fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)?.flatten().map(|e| e.path()).collect();
    v.sort();
    Ok(v)
}
fn under(p: &Path, d: &Path) -> bool {
    p != d && p.starts_with(d)
}

pub fn plan_target(target_root: &Path, now: f64, idle_hours: f64) -> Result<TrimPlan> {
    let mut plan = TrimPlan::default();
    if !target_root.is_dir() {
        return Ok(plan);
    }
    let idle = idle_hours * 3600.0;
    let mut adhoc = Vec::new();
    for entry in sorted_entries(target_root)? {
        let n = name_of(&entry);
        let named = n == "debug" || ADHOC_TRIPLE.is_match(&n) || ADHOC_ORCHESTRATE.is_match(&n);
        if named && real_dir(&entry) && idle_for(&entry, now, idle)? {
            adhoc.push(entry);
        }
    }
    for d in &adhoc {
        plan.paths.push((d.clone(), size(d)?));
        plan.adhoc_dirs += 1;
    }
    let dead_targets = dead_custom_targets(target_root, now, idle.min(3600.0))?;
    for d in &dead_targets {
        plan.paths.push((d.clone(), size(d)?));
        plan.dead_targets += 1;
    }
    adhoc.extend(dead_targets);
    for prof in profile_dirs(target_root) {
        if adhoc.iter().any(|d| prof == *d || under(&prof, d)) {
            continue; // already going whole
        }
        let dead = superseded_hashes(&prof, now, idle)?;
        if dead.is_empty() {
            continue;
        }
        plan.superseded_units += dead.len();
        for sub in HASHED_SUBDIRS {
            let sd = prof.join(sub);
            if !sd.is_dir() {
                continue;
            }
            for p in sorted_entries(&sd)? {
                if HASH
                    .captures(&name_of(&p))
                    .is_some_and(|m| dead.contains(&m[1]))
                {
                    plan.paths.push((p.clone(), size(&p)?));
                }
            }
        }
    }
    Ok(plan)
}

/// Custom-target subdirs of per-module dirs whose cell no longer has the target's json.
/// Judged against THIS worktree's source only; a dir whose cell cannot be found is not
/// judged. The idle window is capped at an hour: the rule is exact, the window only keeps
/// clear of a build that started before the json went.
fn dead_custom_targets(target_root: &Path, now: f64, idle: f64) -> Result<Vec<PathBuf>> {
    let Some(rust) = target_root.parent() else {
        return Ok(vec![]);
    };
    if name_of(rust) != "rust" || !target_root.is_dir() {
        return Ok(vec![]);
    }
    let mut dead = Vec::new();
    for entry in sorted_entries(target_root)? {
        let n = name_of(&entry);
        let Some(m) = CELL_LINK.captures(&n).or_else(|| CELL_RUSTLIB.captures(&n)) else {
            continue;
        };
        if !real_dir(&entry) {
            continue;
        }
        let cell = rust.join(&m["cell"]);
        if !cell.join("Cargo.toml").is_file() {
            continue;
        }
        for sub in sorted_entries(&entry)? {
            let s = name_of(&sub);
            if MODULE_DIR_OTHER.contains(&s.as_str()) || ADHOC_TRIPLE.is_match(&s) {
                continue;
            }
            if !real_dir(&sub) || cell.join(format!("{s}.json")).exists() {
                continue;
            }
            if idle_for(&sub, now, idle)? {
                dead.push(sub);
            }
        }
    }
    Ok(dead)
}

fn cache_value(build: &Path, name: &str) -> Option<String> {
    let bytes = fs::read(build.join("CMakeCache.txt")).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let pattern = Regex::new(&format!(r"(?m)^{}:[A-Z]+=(.*)$", regex::escape(name))).ok()?;
    pattern
        .captures(&text)
        .map(|m| m[1].trim_matches(|c: char| c.is_whitespace()).to_owned())
}
fn sharded(v: Option<String>) -> bool {
    v.is_some_and(|s| !s.is_empty() && s != "0")
}

fn retired_layouts(worktree: &Path, now: f64, idle: f64) -> Result<Vec<PathBuf>> {
    let mut dead = Vec::new();
    let builds: Vec<PathBuf> = ARCHES
        .iter()
        .map(|a| worktree.join(format!("build-{a}")))
        .filter(|b| b.is_dir())
        .collect();
    for build in &builds {
        let Ok(bytes) = fs::read(build.join("build.ninja")) else {
            continue; // unconfigured: nothing says what is live
        };
        let ninja = String::from_utf8_lossy(&bytes);
        let mut found = Vec::new();
        walk(build, &mut |parent, dirs| {
            dirs.retain(|name| {
                if name.starts_with("rust-") && name.ends_with("-target") {
                    found.push((parent.join(name), name.clone()));
                    false
                } else {
                    !(name == "CMakeFiles" || name == ".git" || name.ends_with("-target"))
                }
            });
        });
        for (path, name) in found {
            if !ninja.contains(&format!("/{name}"))
                && !is_symlink(&path)
                && idle_for(&path, now, idle)?
            {
                dead.push(path);
            }
        }
    }
    // Cells of an arch whose cache shares link dirs no longer read their own.
    let target = worktree.join("sdk/lib/rust/target");
    for build in &builds {
        let arch = name_of(build).trim_start_matches("build-").to_owned();
        if !sharded(cache_value(build, "VIBEOS_CARGO_LINK_SHARDS")) || !target.is_dir() {
            continue;
        }
        let suffix = format!("-{arch}-cargo-link");
        for entry in sorted_entries(&target)? {
            if name_of(&entry).ends_with(&suffix)
                && real_dir(&entry)
                && idle_for(&entry, now, idle)?
            {
                dead.push(entry);
            }
        }
    }
    dead.extend(idle_body_shards(&target, now)?);
    if !builds.is_empty()
        && target.is_dir()
        && builds
            .iter()
            .all(|b| sharded(cache_value(b, "VIBEOS_RUST_BODY_SHARDS")))
    {
        for entry in sorted_entries(&target)? {
            if CELL_RUSTLIB.is_match(&name_of(&entry))
                && real_dir(&entry)
                && idle_for(&entry, now, idle)?
            {
                dead.push(entry);
            }
        }
    }
    Ok(dead)
}

/// Body-storage shards whose `.body-storage.lock` (stamped on every acquire) is three days
/// old and which no build holds right now.
fn idle_body_shards(target_root: &Path, now: f64) -> Result<Vec<PathBuf>> {
    let mut dead = Vec::new();
    if !target_root.is_dir() {
        return Ok(dead);
    }
    for entry in sorted_entries(target_root)? {
        if !BODY_SHARD.is_match(&name_of(&entry)) || !real_dir(&entry) {
            continue;
        }
        let lock = entry.join(".body-storage.lock");
        let meta = match fs::metadata(&lock) {
            Ok(m) => m,
            Err(e) if not_found(&e) => continue, // no lock yet: not ours to judge
            Err(e) => return Err(e.into()),
        };
        if now - mtime(&meta) < BODY_SHARD_IDLE_SECONDS {
            continue;
        }
        let handle = match fs::File::open(&lock) {
            Ok(h) => h,
            Err(e) if not_found(&e) => continue,
            Err(e) => return Err(e.into()),
        };
        let fd = handle.as_raw_fd();
        if unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let e = std::io::Error::last_os_error();
            if e.raw_os_error() == Some(libc::EWOULDBLOCK) {
                continue; // a build holds it right now
            }
            return Err(e.into());
        }
        unsafe { libc::flock(fd, libc::LOCK_UN) };
        dead.push(entry);
    }
    Ok(dead)
}

pub fn plan_worktree(worktree: &Path, now: f64, idle_hours: f64) -> Result<TrimPlan> {
    let mut plan = TrimPlan::default();
    for rel in TARGET_RELDIRS {
        let sub = plan_target(&worktree.join(rel), now, idle_hours)?;
        plan.paths.extend(sub.paths);
        plan.superseded_units += sub.superseded_units;
        plan.adhoc_dirs += sub.adhoc_dirs;
        plan.dead_targets += sub.dead_targets;
    }
    let going: Vec<PathBuf> = plan.paths.iter().map(|(p, _)| p.clone()).collect();
    for d in retired_layouts(worktree, now, idle_hours * 3600.0)? {
        if going.iter().any(|g| *g == d || under(&d, g)) {
            continue;
        }
        // A unit already planned inside a retired dir goes with it.
        plan.paths.retain(|(p, _)| !under(p, &d));
        let sz = size(&d)?;
        plan.paths.push((d, sz));
        plan.retired_dirs += 1;
    }
    Ok(plan)
}

/// rustconv `port`'s shared state root (RUSTCONV_PORT_STATE overrides).
pub fn port_state_root() -> PathBuf {
    std::env::var_os("RUSTCONV_PORT_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("vibeos-rust-staging/port")
        })
}

/// Idle `<stem>/target-*` cargo dirs under the port state root, each going whole.
pub fn plan_port_staging(root: &Path, now: f64, idle_hours: f64) -> Result<TrimPlan> {
    let mut plan = TrimPlan::default();
    if !root.is_dir() {
        return Ok(plan);
    }
    for stem in sorted_entries(root)? {
        if !real_dir(&stem) {
            continue;
        }
        for t in sorted_entries(&stem)? {
            if !name_of(&t).starts_with("target-") || !real_dir(&t) {
                continue;
            }
            if !idle_for(&t, now, idle_hours * 3600.0)? {
                continue;
            }
            let sz = size(&t)?;
            plan.paths.push((t, sz));
            plan.adhoc_dirs += 1;
        }
    }
    Ok(plan)
}

/// Delete a plan's paths: (bytes freed, failure lines). Never follows a symlink out of
/// `root`; a path that resolves outside it is refused, not deleted.
pub fn apply(root: &Path, plan: &TrimPlan) -> Result<(u64, Vec<String>)> {
    let root = fs::canonicalize(root)?;
    let (mut freed, mut failures) = (0, Vec::new());
    for (p, sz) in &plan.paths {
        let resolved = match fs::canonicalize(p) {
            Ok(r) => r,
            Err(e) if not_found(&e) => continue,
            Err(e) => {
                failures.push(format!("{}: {e}", p.display()));
                continue;
            }
        };
        if !under(&resolved, &root) {
            failures.push(format!(
                "{}: resolves outside {}",
                p.display(),
                root.display()
            ));
            continue;
        }
        let removed = if real_dir(p) {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        };
        match removed {
            Ok(()) => freed += sz,
            Err(e) if not_found(&e) => {}
            Err(e) => failures.push(format!("{}: {e}", p.display())),
        }
    }
    Ok((freed, failures))
}

fn path_arg(a: &Value, k: &str) -> Result<PathBuf> {
    a[k].as_str()
        .map(PathBuf::from)
        .ok_or_else(|| Error::new(format!("missing {k}")))
}
fn idle_arg(a: &Value) -> Result<f64> {
    let idle = a["idle_hours"].as_f64().unwrap_or(24.0);
    if !idle.is_finite() || idle < 0.0 {
        return Err("idle_hours must be finite and nonnegative".into());
    }
    Ok(idle)
}
pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let now = a["now"].as_f64().unwrap_or_else(now);
    match request["op"].as_str().unwrap_or("") {
        "trim_plan_target" => {
            Ok(plan_target(&path_arg(a, "target_root")?, now, idle_arg(a)?)?.to_json())
        }
        "trim_plan_worktree" => {
            Ok(plan_worktree(&path_arg(a, "worktree")?, now, idle_arg(a)?)?.to_json())
        }
        "trim_plan_port_staging" => {
            let root = a["root"]
                .as_str()
                .map_or_else(port_state_root, PathBuf::from);
            let mut v = plan_port_staging(&root, now, idle_arg(a)?)?.to_json();
            v["root"] = json!(root);
            Ok(v)
        }
        "trim_apply" => {
            let (freed, failures) =
                apply(&path_arg(a, "root")?, &TrimPlan::from_json(&a["plan"])?)?;
            Ok(json!({"freed":freed,"failures":failures}))
        }
        op => Err(format!("unknown trim operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn apply_never_deletes_through_a_symlink_out_of_root() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, outside) = (tmp.path().join("wt"), tmp.path().join("precious"));
        fs::create_dir_all(root.join("sdk")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("state.json"), "{}").unwrap();
        symlink(&outside, root.join("sdk/escape")).unwrap();
        let inside = root.join("sdk/dead");
        fs::create_dir_all(&inside).unwrap();
        let plan = TrimPlan {
            paths: vec![
                (root.join("sdk/escape"), 1),
                (inside.clone(), 7),
                (root.join("gone"), 9),
            ],
            ..TrimPlan::default()
        };
        let (freed, failures) = apply(&root, &plan).unwrap();
        assert_eq!(
            freed, 7,
            "only the in-root dir counts; the vanished path is skipped"
        );
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("resolves outside"));
        assert!(outside.join("state.json").exists() && root.join("sdk/escape").exists());
        assert!(!inside.exists());
    }

    #[test]
    fn identity_ignores_what_supersedes_and_keeps_what_distinguishes() {
        let tmp = tempfile::tempdir().unwrap();
        let fp = |name: &str, body: Value| {
            let d = tmp.path().join(name);
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join("lib-syn.json"), body.to_string()).unwrap();
            d
        };
        let base = json!({"rustc":1,"features":"[\"default\"]","target":7,"profile":3,"path":5,"deps":[],"compile_kind":0});
        let mut newer = base.clone();
        newer["rustc"] = json!(2);
        newer["deps"] = json!([[1, "x"]]);
        let mut feature = base.clone();
        feature["features"] = json!("[\"full\"]");
        let a = unit_identity(&fp("syn-aaaaaaaaaaaaaaaa", base)).unwrap();
        assert_eq!(
            a,
            unit_identity(&fp("syn-bbbbbbbbbbbbbbbb", newer)).unwrap()
        );
        assert_ne!(
            a,
            unit_identity(&fp("syn-cccccccccccccccc", feature)).unwrap()
        );
        assert_ne!(a, unit_identity(&fp("quote-dddddddddddddddd", json!({"rustc":1,"features":"[\"default\"]","target":7,"profile":3,"path":5,"deps":[],"compile_kind":0}))).unwrap());
        fs::write(tmp.path().join("syn-eeeeeeeeeeeeeeee.json"), "").unwrap();
        let partial = fp("syn-ffffffffffffffff", json!({"rustc":1}));
        assert!(
            unit_identity(&partial).is_none(),
            "a fingerprint missing identity fields is never judged"
        );
    }
}
