//! Host-wide, crash-released disk admission, and the host free-space probe. Never deletes.
//!
//! Reservations bound estimated additional writes by cooperating CLI processes, not physical
//! allocation; legacy lanes and external writers stay unaccounted for. Policy is one host
//! file (`<hostlock>/disk/policy.json`), never a per-lane environment override.
//!
//! The decisions live here. The two flocks that must last as long as the command — the
//! admission lock around one claim, and the lease a claim leaves behind — are held by the
//! caller's process (`tests/qemu/cli/diskspace.py`), which passes this module the volumes it
//! measured so a test can stand in for the host.
//!
//! FULLNESS and ADMISSION HEADROOM are different numbers. `df` shows only the first; on
//! 2026-09-06 six lanes stalled with `df` fine and the unreserved headroom gone, so the probe
//! prints the admission number on the green path too.
use crate::{Error, Result};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
/// MEASURED (2026-09-11, fleet stopped, sampled every 20s across a cold build): i386 and
/// amd64 both peak at 7.44 GiB; arm64 build dirs are 2.8-3.0 GiB. 12 carries ~60% headroom for
/// the unmeasured shared-cache writes of a cache-miss-heavy build. The old 32 halted the merge
/// train (3 concurrent gate builds x 32 + 20 floor had to be free). If these drift,
/// re-measure; don't re-order admission — `reserve` deliberately takes disk while holding no
/// CPU permit.
pub const REQUESTS: [(&str, f64); 4] = [
    ("build", 12.0),
    ("check-tooling", 4.0),
    ("gate", 0.0),
    ("land", 0.0),
];
pub const BACKING_FILE: &str = "/etc/vibeos/disk-backing-mounts";
/// Below this, refuse to start a gate rather than fail somewhere unhelpful: a gate writes
/// three ISOs plus artifact dirs, and at 0 bytes the harness cannot even write summary.json.
pub const MIN_FREE_BYTES: u64 = 5 << 30;
/// "Can the harness still record a verdict at all."
pub const CRITICAL_FREE_BYTES: u64 = 256 << 20;

type Env = BTreeMap<String, String>;

fn refuse(message: impl Into<String>) -> Error {
    Error::classified("admission", message)
}
fn not_found(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::NotFound
}

/// The host policy. `initialize` (only under the admission lock) pins the defaults for the
/// whole host the first time, so later lane versions read it rather than choosing their own.
/// Anything malformed fails closed.
pub fn policy(root: &Path, initialize: bool) -> Result<Value> {
    let path = root.join("policy.json");
    let requests: Map<String, Value> = REQUESTS
        .iter()
        .map(|(k, v)| ((*k).into(), json!(*v as u64)))
        .collect();
    let default = json!({"version": 1, "min_free_gib": 20, "requests_gib": requests});
    if !path.exists() {
        if !initialize {
            return Ok(default);
        }
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        f.write_all(default.to_string().as_bytes())?;
        f.sync_all()?;
    }
    let bad = |why: String| {
        refuse(format!(
            "invalid host disk policy {}: {why}",
            path.display()
        ))
    };
    let text = fs::read_to_string(&path).map_err(|e| bad(e.to_string()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| bad(e.to_string()))?;
    let keys = |v: &Value| -> Option<Vec<String>> {
        let mut k: Vec<String> = v.as_object()?.keys().cloned().collect();
        k.sort();
        Some(k)
    };
    if keys(&value) != keys(&default) || value["version"].as_f64() != Some(1.0) {
        return Err(bad("expected version, min_free_gib, requests_gib".into()));
    }
    let amounts = &value["requests_gib"];
    if keys(amounts) != keys(&default["requests_gib"]) {
        return Err(bad(
            "requests_gib must name build, check-tooling, gate, land".into(),
        ));
    }
    let mut all = vec![&value["min_free_gib"]];
    all.extend(amounts.as_object().into_iter().flat_map(|o| o.values()));
    for amount in all {
        let Some(n) = amount.as_f64().filter(|_| amount.is_number()) else {
            return Err(bad("budgets must be finite numbers".into()));
        };
        if !n.is_finite() || n < 0.0 {
            return Err(bad("budgets must be nonnegative and finite".into()));
        }
    }
    let n = |v: &Value| v.as_f64().unwrap_or(0.0);
    if n(&value["min_free_gib"]) < 1.0
        || n(&amounts["build"]) <= 0.0
        || n(&amounts["check-tooling"]) <= 0.0
    {
        return Err(bad(
            "keep at least 1 GiB headroom and positive writer reservations".into(),
        ));
    }
    Ok(value)
}

fn budget_bytes(policy: &Value, kind: &str) -> Result<u64> {
    let gib = policy["requests_gib"][kind]
        .as_f64()
        .ok_or_else(|| refuse(format!("no disk request for {kind}")))?;
    Ok((gib * GIB) as u64)
}
fn floor_bytes(policy: &Value) -> u64 {
    (policy["min_free_gib"].as_f64().unwrap_or(0.0) * GIB) as u64
}

/// Host volumes BACKING our filesystem, which `df /` cannot see.
///
/// A WSL ext4 lives in a sparse .vhdx on a Windows volume, so the guest reports the VHDX
/// CEILING while the real constraint is the host volume underneath. On 2026-09-13 that gap ate
/// the fleet: a 1 TiB ceiling over a C: with 620 GiB free, seven concurrent builds filled the
/// HOST while `df /` still showed room. Only a DYNAMIC .vhdx's host belongs here.
///
/// Read from the host file on every call — as an exported variable, a layout change reached
/// no running session. The file wins over `VIBEOS_DISK_BACKING_MOUNTS` (colon-separated) so a
/// stale export cannot override it. Neither means unarmed, right for a bare-metal host.
pub fn backing_mounts(file: &Path, env: &Env) -> Result<Vec<PathBuf>> {
    match fs::read_to_string(file) {
        Ok(text) => Ok(text
            .lines()
            .map(|l| l.split('#').next().unwrap_or("").trim())
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .collect()),
        Err(e) if not_found(&e) => Ok(env
            .get("VIBEOS_DISK_BACKING_MOUNTS")
            .map(|raw| {
                raw.split(':')
                    .filter(|p| !p.is_empty())
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default()),
        Err(e) => Err(e.into()),
    }
}

/// Free bytes on the filesystem holding `path` (statvfs `f_bavail`, as `shutil.disk_usage`).
pub fn free_bytes(path: &Path) -> io::Result<u64> {
    let c = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    // SAFETY: statvfs writes only into the zeroed struct we own.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(st.f_bavail as u64 * st.f_frsize as u64)
}

fn nearest_existing(p: &Path) -> PathBuf {
    let mut p = crate::prune::resolve(p);
    while !p.exists() {
        match p.parent() {
            Some(parent) => p = parent.to_path_buf(),
            None => break,
        }
    }
    p
}

/// Every volume a build writes to, each device once: the tree, both compiler caches, the temp
/// dir, the artifact root, and the backing mounts — each a distinct `st_dev` folded in as one
/// more volume, so every floor/reservation check applies to it unchanged.
pub fn filesystems(paths: &[PathBuf]) -> Result<Vec<Value>> {
    let mut order: Vec<String> = Vec::new();
    let mut devices: BTreeMap<String, Value> = BTreeMap::new();
    for p in paths {
        let path = nearest_existing(p);
        let dev = fs::metadata(&path)?.dev().to_string();
        if !devices.contains_key(&dev) {
            order.push(dev.clone());
        }
        // Last path naming a device wins, as the dict assignment it replaces did.
        devices.insert(
            dev.clone(),
            json!({"device": dev, "path": path, "free_bytes": free_bytes(&path)?}),
        );
    }
    Ok(order
        .into_iter()
        .filter_map(|d| devices.remove(&d))
        .collect())
}

pub fn volume_paths(repo: &Path, env: &Env, tmpdir: &Path, backing: &Path) -> Result<Vec<PathBuf>> {
    let cache = crate::build_policy::cache_make(repo, env)?;
    let mut paths = vec![
        repo.to_path_buf(),
        cache.ccache_dir,
        cache.sccache_dir,
        tmpdir.to_path_buf(),
        PathBuf::from(
            env.get("VIBEOS_ARTIFACT_ROOT")
                .map_or("/tmp/vibeos-cli", String::as_str),
        ),
    ];
    paths.extend(backing_mounts(backing, env)?);
    Ok(paths)
}

fn try_lock(f: &File) -> io::Result<bool> {
    // SAFETY: flock on a descriptor we own.
    if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        return Ok(true);
    }
    let e = io::Error::last_os_error();
    if e.raw_os_error() == Some(libc::EWOULDBLOCK) {
        Ok(false)
    } else {
        Err(e)
    }
}

/// Live leases: a lease file whose flock someone holds. One nobody holds is an orphan of a
/// crashed process (the kernel released its lock) and is reaped when `reap`. A live lease
/// whose metadata cannot be read fails closed — an unknown protocol is not free space.
pub fn live(root: &Path, reap: bool) -> Result<Vec<Value>> {
    let mut names: Vec<PathBuf> = match fs::read_dir(root) {
        Ok(it) => it
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("lease-") && n.ends_with(".json"))
            })
            .collect(),
        Err(e) if not_found(&e) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    names.sort();
    let mut leases = Vec::new();
    for path in names {
        let mut f = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(f) => f,
            Err(e) if not_found(&e) => continue,
            Err(e) => return Err(e.into()),
        };
        if try_lock(&f)? {
            if reap {
                let _ = fs::remove_file(&path);
            }
            continue;
        }
        let mut text = String::new();
        let bad = |why: &str| {
            refuse(format!(
                "unreadable live disk lease {}: {why}",
                path.display()
            ))
        };
        f.read_to_string(&mut text)
            .map_err(|e| bad(&e.to_string()))?;
        let meta: Value = serde_json::from_str(&text).map_err(|e| bad(&e.to_string()))?;
        if meta["version"].as_f64() != Some(1.0) || !meta["reservations"].is_object() {
            return Err(bad("invalid reservation metadata"));
        }
        if meta["reservations"]
            .as_object()
            .is_some_and(|r| r.values().any(|n| n.as_u64().is_none()))
        {
            return Err(bad("invalid reserved byte count"));
        }
        leases.push(meta);
    }
    Ok(leases)
}

fn reserved_on(leases: &[Value], device: &str) -> u64 {
    leases
        .iter()
        .filter_map(|l| l["reservations"][device].as_u64())
        .sum()
}
fn gib(n: u64) -> String {
    format!("{:.1}", n as f64 / GIB)
}
fn vol<'a>(v: &'a Value) -> (&'a str, &'a str, u64) {
    (
        v["device"].as_str().unwrap_or(""),
        v["path"].as_str().unwrap_or(""),
        v["free_bytes"].as_u64().unwrap_or(0),
    )
}

/// Fail-fast half of a DEFERRED kind's admission: is the host too full for this at ALL?
/// Deliberately ignores other lanes' live reservations — counting contention here refuses a
/// build for space a concurrent build merely holds a claim on. Without it, a genuinely full
/// host would queue every build for a CPU permit just to refuse it after.
pub fn refuse_if_hopeless(root: &Path, kind: &str, volumes: &[Value]) -> Result<()> {
    let config = policy(root, false)?;
    let (budget, floor) = (budget_bytes(&config, kind)?, floor_bytes(&config));
    for v in volumes {
        let (_, path, free) = vol(v);
        if free < floor + budget {
            return Err(refuse(format!(
                "{kind}: {path} has {} GiB free; need {} headroom + {} requested, before any \
                 other lane is counted. Use vibeos-cli disk-status; reclaim completed \
                 disposable outputs.",
                gib(free),
                gib(floor),
                gib(budget)
            )));
        }
    }
    Ok(())
}

/// One admission decision; the caller holds the admission lock across it and the lease it
/// then writes. Short only because of other jobs' live reservations is `contention` (space
/// that frees itself); short regardless is `admission`.
pub fn claim(root: &Path, kind: &str, volumes: &[Value]) -> Result<Value> {
    fs::create_dir_all(root)?;
    let config = policy(root, true)?;
    let (budget, floor) = (budget_bytes(&config, kind)?, floor_bytes(&config));
    let leases = live(root, true)?;
    let mut reservations = Map::new();
    for v in volumes {
        let (device, path, free) = vol(v);
        let reserved = reserved_on(&leases, device);
        if free < floor + reserved + budget {
            let class = if free >= floor + budget {
                "contention"
            } else {
                "admission"
            };
            return Err(Error::classified(
                class,
                format!(
                    "{kind}: {path} has {} GiB free; need {} headroom + {} live reservations \
                     + {} requested. Use vibeos-cli disk-status; reclaim completed disposable \
                     outputs or wait for reservations to finish.",
                    gib(free),
                    gib(floor),
                    gib(reserved),
                    gib(budget)
                ),
            ));
        }
        reservations.insert(device.into(), json!(budget));
    }
    Ok(json!({"kind": kind, "reservations": reservations}))
}

/// `disk-status`: read-only — never creates the policy or reaps a lease.
pub fn status(
    root: &Path,
    hostlock: &Path,
    volumes: &[Value],
    backing: &[PathBuf],
    backing_file: &Path,
    footprints: Option<&[PathBuf]>,
) -> Result<Value> {
    let config = policy(root, false)?;
    let leases = live(root, false)?;
    let floor = floor_bytes(&config);
    let volumes: Vec<Value> = volumes
        .iter()
        .map(|v| {
            let (device, _, free) = vol(v);
            let reserved = reserved_on(&leases, device);
            let mut v = v.clone();
            v["reserved_bytes"] = json!(reserved);
            v["unreserved_above_floor_bytes"] = json!(free.saturating_sub(reserved + floor));
            v
        })
        .collect();
    let retention = hostlock.join("retention.json");
    let timer_active = retention.exists()
        && !hostlock.join("maintenance-disabled").exists()
        && crate::process::capture(
            std::process::Command::new("systemctl").args([
                "--user",
                "is-active",
                "--quiet",
                "vibeos-retention.timer",
            ]),
            5,
        )
        .map_err(|e| refuse(e.to_string()))?
        .code
            == 0;
    let mut result = json!({
        "policy_path": root.join("policy.json"),
        "policy": config,
        "filesystems": volumes,
        "leases": leases,
        "coverage": "cooperating CLI build/tooling jobs only; not a physical quota",
        // Say ARMED/UNARMED positively: an absent guard that reports nothing reads
        // identically to a guard that found nothing to worry about.
        "backing_mounts": if backing.is_empty() {
            json!(format!("UNARMED: list the host volumes in {}", backing_file.display()))
        } else {
            json!(backing)
        },
        "automatic_retention": timer_active,
        "retention_policy_path": retention,
    });
    if let Some(paths) = footprints {
        let mut sizes = Vec::new();
        for p in paths.iter().filter(|p| p.is_dir()) {
            sizes.push(json!({"path": p, "bytes": crate::prune::disk_usage_bytes(p)?}));
        }
        result["footprints"] = json!(sizes);
    }
    Ok(result)
}

/// Bytes -> short human string. Exact enough to act on, short enough to log.
pub fn human(n: u64) -> String {
    let mut v = n as f64;
    for unit in ["B", "K", "M", "G", "T"] {
        if v.abs() < 1024.0 || unit == "T" {
            return if unit == "B" {
                format!("{v:.0}{unit}")
            } else {
                format!("{v:.1}{unit}")
            };
        }
        v /= 1024.0;
    }
    unreachable!()
}

/// Check every distinct filesystem behind `paths`; `lines` already carry the numbers, so
/// callers print them verbatim — "DISK_FULL: 192K free on / (need 5.0G)" ends the
/// investigation in one line where four agents reading a RED with no failing test did not.
pub fn probe(paths: &[PathBuf], threshold: u64) -> Value {
    let mut seen: Vec<(u64, PathBuf, u64)> = Vec::new();
    for p in paths {
        let near = nearest_existing(p);
        let Ok(n) = free_bytes(&near) else { continue };
        let Ok(meta) = fs::metadata(&near) else {
            continue;
        };
        // One report per filesystem, attributed to the first path that named it.
        if !seen.iter().any(|(d, _, _)| *d == meta.dev()) {
            seen.push((meta.dev(), p.clone(), n));
        }
    }
    let mut lines = Vec::new();
    let (mut ok, mut critical) = (true, false);
    let mut by_free = seen.clone();
    by_free.sort_by_key(|(_, _, n)| *n);
    for (_, p, n) in &by_free {
        if *n < threshold {
            ok = false;
            lines.push(format!(
                "DISK_FULL: {} free on {} (need {})",
                human(*n),
                p.display(),
                human(threshold)
            ));
            if *n < CRITICAL_FREE_BYTES {
                critical = true;
                lines.push(format!(
                    "  CRITICAL: below {} — the harness may be unable to write summary.json \
                     or even run `true`, so any verdict from this host is unreliable rather \
                     than merely red.",
                    human(CRITICAL_FREE_BYTES)
                ));
            }
        }
    }
    let free: Map<String, Value> = seen
        .iter()
        .map(|(_, p, n)| (p.display().to_string(), json!(n)))
        .collect();
    json!({"ok": ok, "critical": critical, "lines": lines, "free": free})
}

/// The filesystems a gate writes to: the tree, its build dirs, and /tmp — where artifact
/// dirs and every command's stdout file live, and where the 2026-09-06 ENOSPC was raised.
pub fn gate_paths(repo: &Path, arches: &[String]) -> Vec<PathBuf> {
    let mut out = vec![repo.to_path_buf(), PathBuf::from("/tmp")];
    out.extend(arches.iter().map(|a| repo.join(format!("build-{a}"))));
    out
}

/// Salient lines for an ENOSPC classification — always carrying the number.
pub fn enospc_detail(repo: &Path) -> Vec<String> {
    let rep = probe(&gate_paths(repo, &[]), MIN_FREE_BYTES);
    let lines: Vec<String> = rep["lines"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| l.as_str().map(str::to_owned))
        .take(3)
        .collect();
    if !lines.is_empty() {
        return lines;
    }
    // Space was reclaimed between the failure and this probe: say so, rather than print a
    // reassuring "plenty free" that contradicts the error just seen.
    let now = rep["free"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(k, v)| format!("{k}={}", human(v.as_u64().unwrap_or(0))))
        .collect::<Vec<_>>()
        .join(", ");
    vec![format!(
        "ENOSPC reported by the toolchain but the filesystem now has headroom — space was \
         freed after the failure, or a different mount filled. now: {now}"
    )]
}

/// The admission headroom a run actually needs, printed on the GREEN path too: "df looks
/// fine" is a documented false negative for this class. A failure to read is reported, never
/// swallowed, and never blocks the free-space verdict.
pub fn admission_lines(status: Result<Value>, arch_count: usize) -> Vec<String> {
    let st = match status {
        Ok(st) => st,
        Err(e) => return vec![format!("admission headroom unavailable ({e})")],
    };
    let requests = &st["policy"]["requests_gib"];
    let floor = &st["policy"]["min_free_gib"];
    let n = arch_count.max(1);
    let wants = requests["build"].as_f64().unwrap_or(0.0) * n as f64
        + requests["check-tooling"].as_f64().unwrap_or(0.0);
    let num = |v: f64| {
        if v.fract() == 0.0 {
            format!("{}", v as i64)
        } else {
            format!("{v}")
        }
    };
    st["filesystems"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|fs| {
            let head = fs["unreserved_above_floor_bytes"].as_u64().unwrap_or(0);
            let mark = if head as f64 >= wants * GIB {
                ""
            } else {
                "  << SHORT"
            };
            format!(
                "admission: {} unreserved above the {}G floor on {} ({n}-arch run requests \
                 ~{}G){mark}",
                human(head),
                num(floor.as_f64().unwrap_or(0.0)),
                fs["path"].as_str().unwrap_or(""),
                num(wants)
            )
        })
        .collect()
}

fn path_arg(a: &Value, k: &str) -> Result<PathBuf> {
    a[k].as_str()
        .map(PathBuf::from)
        .ok_or_else(|| Error::new(format!("missing {k}")))
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
fn env_arg(a: &Value) -> Env {
    serde_json::from_value(a["environment"].clone()).unwrap_or_default()
}
fn volumes_arg(a: &Value) -> Vec<Value> {
    a["volumes"].as_array().cloned().unwrap_or_default()
}
fn backing_file(a: &Value) -> PathBuf {
    PathBuf::from(a["backing_file"].as_str().unwrap_or(BACKING_FILE))
}

/// Every failure leaves here classified, so the facade can name it DISK_ADMISSION_FAIL:
/// `contention` (wait and retry) or `admission`.
pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let op = request["op"].as_str().unwrap_or("");
    let run = || -> Result<Value> {
        match op {
            "disk_policy" => policy(&path_arg(a, "root")?, false),
            "disk_backing_mounts" => Ok(json!(backing_mounts(&backing_file(a), &env_arg(a))?)),
            "disk_filesystems" => {
                let p = volume_paths(
                    &path_arg(a, "repo")?,
                    &env_arg(a),
                    &path_arg(a, "tmpdir")?,
                    &backing_file(a),
                )?;
                Ok(json!(filesystems(&p)?))
            }
            "disk_live" => Ok(json!(live(&path_arg(a, "root")?, false)?)),
            "disk_refuse_if_hopeless" => {
                let kind = a["kind"].as_str().unwrap_or("");
                refuse_if_hopeless(&path_arg(a, "root")?, kind, &volumes_arg(a))
                    .map(|()| json!(null))
            }
            "disk_claim" => claim(
                &path_arg(a, "root")?,
                a["kind"].as_str().unwrap_or(""),
                &volumes_arg(a),
            ),
            "disk_status" => {
                let footprints = a["footprints"].as_array().map(|_| paths(&a["footprints"]));
                status(
                    &path_arg(a, "root")?,
                    &path_arg(a, "hostlock")?,
                    &volumes_arg(a),
                    &backing_mounts(&backing_file(a), &env_arg(a))?,
                    &backing_file(a),
                    footprints.as_deref(),
                )
            }
            "disk_probe" => Ok(probe(
                &paths(&a["paths"]),
                a["threshold"].as_u64().unwrap_or(MIN_FREE_BYTES),
            )),
            "disk_gate_paths" => {
                let arches: Vec<String> =
                    serde_json::from_value(a["arches"].clone()).unwrap_or_default();
                Ok(json!(gate_paths(&path_arg(a, "repo")?, &arches)))
            }
            "disk_enospc_detail" => Ok(json!(enospc_detail(&path_arg(a, "repo")?))),
            "disk_admission_lines" => {
                // The status that feeds these is the facade's (it measured the volumes);
                // a failure to produce it arrives as `status_error`.
                let st = match a["status_error"].as_str() {
                    Some(e) => Err(Error::new(e)),
                    None => Ok(a["status"].clone()),
                };
                Ok(json!(admission_lines(
                    st,
                    a["arch_count"].as_u64().unwrap_or(0) as usize
                )))
            }
            "disk_requests" => Ok(json!(REQUESTS
                .iter()
                .map(|(k, v)| ((*k).to_owned(), json!(*v as u64)))
                .collect::<Map<_, _>>())),
            "disk_human" => Ok(json!(human(a["n"].as_u64().unwrap_or(0)))),
            "disk_free_bytes" => Ok(json!(
                free_bytes(&nearest_existing(&path_arg(a, "path")?)).ok()
            )),
            _ => Err(Error::new(format!("unknown disk operation {op}"))),
        }
    };
    run().map_err(|e| match e.kind {
        Some(_) => e,
        None => e.with_kind("admission"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(free: u64) -> Vec<Value> {
        vec![json!({"device": "1", "path": "/fixture", "free_bytes": free})]
    }
    /// Room for exactly one build plus a check-tooling job, and not two builds.
    const ONE_BUILD: u64 = (20 + 12 + 4 + 1) << 30;

    fn hold(root: &Path, kind: &str, volumes: &[Value]) -> File {
        let meta = claim(root, kind, volumes).unwrap();
        let path = root.join(format!("lease-{kind}-{}.json", std::process::id()));
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        assert!(try_lock(&f).unwrap());
        let mut lease = meta.clone();
        lease["version"] = json!(1);
        (&f).write_all(lease.to_string().as_bytes()).unwrap();
        f
    }

    #[test]
    fn a_second_build_is_contention_and_a_full_disk_is_not() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("disk");
        let v = volume(ONE_BUILD);
        let _first = hold(&root, "build", &v);
        let second = claim(&root, "build", &v).unwrap_err();
        assert_eq!(second.kind.as_deref(), Some("contention"));
        assert!(
            second.to_string().contains("12.0 live reservations"),
            "{second}"
        );
        // A job that fits beside the live one is still admitted.
        assert!(claim(&root, "check-tooling", &v).is_ok());
        // Fail-fast admission ignores the live claim: it is not a full host.
        refuse_if_hopeless(&root, "build", &v).unwrap();
        let full = claim(&root, "build", &volume(5 << 30)).unwrap_err();
        assert_eq!(full.kind.as_deref(), Some("admission"));
        assert!(refuse_if_hopeless(&root, "build", &volume(0)).is_err());
    }

    #[test]
    fn a_crashed_holder_is_reaped_and_status_never_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("disk");
        let host = tmp.path();
        let st = status(&root, host, &volume(ONE_BUILD), &[], Path::new("/x"), None).unwrap();
        assert_eq!(st["leases"], json!([]));
        assert!(!root.exists(), "status is read-only");
        let v = volume(ONE_BUILD);
        let held = hold(&root, "build", &v);
        let st = status(&root, host, &v, &[], Path::new("/x"), None).unwrap();
        assert_eq!(st["filesystems"][0]["reserved_bytes"], json!(12u64 << 30));
        assert_eq!(
            st["backing_mounts"],
            json!("UNARMED: list the host volumes in /x")
        );
        drop(held); // the kernel releases the flock, as for a killed process
        assert_eq!(live(&root, false).unwrap().len(), 0);
        assert_eq!(
            fs::read_dir(&root).unwrap().count(),
            2,
            "policy + orphan, not reaped"
        );
        claim(&root, "build", &v).unwrap();
        assert_eq!(
            fs::read_dir(&root).unwrap().count(),
            1,
            "orphan reaped under claim"
        );
    }

    #[test]
    fn foreign_live_metadata_fails_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("disk");
        fs::create_dir_all(&root).unwrap();
        let f = File::create(root.join("lease-foreign.json")).unwrap();
        assert!(try_lock(&f).unwrap());
        (&f).write_all(br#"{"version":2,"reservations":{}}"#)
            .unwrap();
        let e = claim(&root, "build", &volume(ONE_BUILD)).unwrap_err();
        assert!(e.to_string().contains("unreadable live"), "{e}");
    }

    #[test]
    fn invalid_policies_fail_closed_and_edits_are_reloaded() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for bad in [
            "{}",
            "null",
            "{",
            r#"{"version":2}"#,
            r#"{"version":1,"min_free_gib":0,"requests_gib":{"build":32,"check-tooling":4,"gate":0,"land":0}}"#,
            r#"{"version":1,"min_free_gib":true,"requests_gib":{"build":32,"check-tooling":4,"gate":0,"land":0}}"#,
            r#"{"version":1,"min_free_gib":20,"requests_gib":{"build":32,"check-tooling":4,"gate":0}}"#,
        ] {
            fs::write(root.join("policy.json"), bad).unwrap();
            let e = claim(root, "build", &volume(ONE_BUILD)).unwrap_err();
            assert_eq!(e.kind.as_deref(), Some("admission"), "{bad}");
        }
        fs::remove_file(root.join("policy.json")).unwrap();
        claim(root, "build", &volume(ONE_BUILD)).unwrap();
        let mut p = policy(root, false).unwrap();
        assert_eq!(
            p["requests_gib"]["build"],
            json!(12),
            "the first claim pins the defaults"
        );
        p["min_free_gib"] = json!(50);
        fs::write(root.join("policy.json"), p.to_string()).unwrap();
        assert!(
            claim(root, "build", &volume(ONE_BUILD)).is_err(),
            "policy reloaded"
        );
    }

    #[test]
    fn the_host_file_names_the_backing_volumes_and_beats_a_stale_export() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("disk-backing-mounts");
        let env: Env = [(
            "VIBEOS_DISK_BACKING_MOUNTS".to_owned(),
            "/mnt/g:/mnt/h".to_owned(),
        )]
        .into_iter()
        .collect();
        assert_eq!(
            backing_mounts(&f, &env).unwrap(),
            [PathBuf::from("/mnt/g"), PathBuf::from("/mnt/h")]
        );
        assert!(
            backing_mounts(&f, &Env::new()).unwrap().is_empty(),
            "unarmed"
        );
        fs::write(
            &f,
            "# dynamic root vhdx host\n/mnt/c\n\n/mnt/x  # another\n",
        )
        .unwrap();
        assert_eq!(
            backing_mounts(&f, &env).unwrap(),
            [PathBuf::from("/mnt/c"), PathBuf::from("/mnt/x")]
        );
    }

    #[test]
    fn filesystems_count_each_device_once() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().to_path_buf();
        let v = filesystems(&[p.clone(), p.join("not/yet/created"), p.join("..")]).unwrap();
        let devs: std::collections::BTreeSet<String> =
            v.iter().map(|x| x["device"].to_string()).collect();
        assert_eq!(devs.len(), v.len());
    }

    #[test]
    fn the_probe_prints_the_number() {
        assert_eq!(human(1023), "1023B");
        assert_eq!(human(5 << 30), "5.0G");
        assert_eq!(human(192 << 10), "192.0K");
        let tmp = tempfile::tempdir().unwrap();
        let rep = probe(
            &[tmp.path().to_path_buf(), tmp.path().join("build-x")],
            u64::MAX,
        );
        assert_eq!(rep["ok"], json!(false));
        let lines = rep["lines"].as_array().unwrap();
        assert!(lines[0].as_str().unwrap().starts_with("DISK_FULL: "));
        assert_eq!(
            rep["free"].as_object().unwrap().len(),
            1,
            "one report per filesystem"
        );
        assert_eq!(probe(&[tmp.path().to_path_buf()], 0)["ok"], json!(true));
    }

    #[test]
    fn admission_lines_mark_short_headroom_and_report_failures() {
        let st = json!({
            "policy": {"min_free_gib": 20, "requests_gib": {"build": 12, "check-tooling": 4}},
            "filesystems": [
                {"path": "/a", "unreserved_above_floor_bytes": 100u64 << 30},
                {"path": "/b", "unreserved_above_floor_bytes": 10u64 << 30},
            ],
        });
        let lines = admission_lines(Ok(st), 3);
        assert_eq!(
            lines[0],
            "admission: 100.0G unreserved above the 20G floor on /a (3-arch run requests ~40G)"
        );
        assert!(lines[1].ends_with("~40G)  << SHORT"));
        let failed = admission_lines(Err(Error::new("boom")), 0);
        assert_eq!(failed, ["admission headroom unavailable (boom)"]);
    }
}
