//! The host broker's live read surface: who holds which lease (`host-status`), the waiter
//! queues and the admission shadow, whether a transcript starved on the broker, and the job
//! class and landing reserve that admission keys on.
//!
//! Lease ACQUISITION is not here. Native builds and boots take their leases through
//! `admission.rs` and `host-jobs`; the Python acquirers in `tests/qemu/cli/hostlock.py` serve
//! the land lock (`land --direct`), the build-dir lock and older lanes, and stay there
//! untouched. Everything below is read-only: probing a lock takes it non-blocking and drops it
//! at once, and the queue view never reaps an orphan ticket (the next acquirer does), so
//! `host-status` cannot perturb what it reports.
use crate::{Error, Result};
use serde_json::{json, Map, Value};
use std::os::fd::AsRawFd;
use std::path::Path;

/// Bump when lease semantics change incompatibly. Readers treat a missing field as v0 and
/// still cooperate, but `host-status` flags a live holder speaking another version: that is
/// another lane running divergent tooling against this shared state.
pub const PROTOCOL_VERSION: f64 = 1.0;

/// The queue/acquire announcement pair. A transcript with WAIT_ANNOUNCE and no
/// ACQUIRED_ANNOUNCE spent its whole budget queued on the broker and never ran its work.
pub const WAIT_ANNOUNCE: &str = "[hostlock] waiting for host CPU lease";
pub const ACQUIRED_ANNOUNCE: &str = "[hostlock] lease acquired";

/// ACQUIRED_ANNOUNCE is printed only after an announced wait, so the pair-absence is exact:
/// the run's flat timeouts are host starvation (another lane owns the leases), not a code
/// regression.
pub fn looks_starved(output: &str) -> bool {
    output.contains(WAIT_ANNOUNCE) && !output.contains(ACQUIRED_ANNOUNCE)
}

/// The job's class. Exactly one admission decision reads it: an `integration` job (a landing
/// gate) may take the boot permits `land_boot_reserve` holds back from everyone else.
pub fn job_class(value: Option<&str>) -> &str {
    match value.unwrap_or("interactive") {
        v @ ("interactive" | "integration" | "background") => v,
        _ => "unknown",
    }
}

/// How many boot permits only a LANDING gate may take: half the pool by default (one full
/// gate's concurrent boots) and never all of it, so lanes can always boot.
/// VIBEOS_LAND_BOOT_RESERVE overrides (0 disables).
pub fn land_boot_reserve(leases: i64, override_: Option<&str>) -> Result<i64> {
    let want = match override_ {
        Some(v) => v.trim().parse::<i64>().map_err(|_| Error {
            kind: Some("value".into()),
            message: format!("VIBEOS_LAND_BOOT_RESERVE={v:?} is not an integer"),
        })?,
        None => leases.div_euclid(2),
    };
    Ok(want.min(leases - 1).max(0))
}

/// Whether anyone holds `path`'s flock: take it non-blocking and drop it at once.
pub fn is_held(path: &Path) -> bool {
    let Ok(f) = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
    else {
        return false;
    };
    let fd = f.as_raw_fd();
    if unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        unsafe { libc::flock(fd, libc::LOCK_UN) };
        false
    } else {
        true
    }
}

fn pid_alive(pid: i64) -> bool {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return false;
    };
    if pid <= 0 {
        return false;
    }
    if unsafe { libc::kill(pid, 0) } == 0 {
        return true;
    }
    // EPERM: it exists, owned by someone else.
    std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

fn holder_pid(meta: &Map<String, Value>) -> Option<i64> {
    match meta.get("pid")? {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn read_meta(path: &Path) -> Map<String, Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(t.trim()).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn entry(path: &Path, name: &str) -> Value {
    let held = is_held(path);
    let meta = read_meta(path);
    let alive = match holder_pid(&meta) {
        Some(0) | None => Value::Null,
        Some(pid) => json!(pid_alive(pid)),
    };
    let speaks = meta
        .get("v")
        .map_or(Some(0.0), Value::as_f64)
        .is_some_and(|v| v == PROTOCOL_VERSION);
    json!({
        "name": name,
        "held": held,
        "holder": meta,
        "holder_alive": alive,
        "protocol_skew": held && !meta.is_empty() && !speaks,
    })
}

/// Clock readings, injected so a test can pin them. `mono` is CLOCK_MONOTONIC, the clock
/// Python's `time.monotonic()` (which the waiters stamp) reads.
#[derive(Clone, Copy)]
pub struct Now {
    pub wall: f64,
    pub mono: f64,
}

impl Now {
    pub fn read() -> Self {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64());
        Self {
            wall,
            mono: ts.tv_sec as f64 + ts.tv_nsec as f64 / 1e9,
        }
    }
}

/// Python's round(x, 1): the exact binary value rounded half-to-even, as Rust's formatter does.
fn round1(x: f64) -> f64 {
    format!("{x:.1}").parse().unwrap_or(x)
}

/// Live waiters for `pool`, oldest arrival first. A ticket file is named
/// `<arrival ns>-<pid>-<need>`; its owner holds its flock for the whole wait, so a ticket
/// nobody holds is an orphan and is left out (never deleted here).
pub fn queue_status(dir: &Path, pool: &str, now: Now) -> Vec<Value> {
    let qd = dir.join(format!("queue-{pool}"));
    let Ok(rd) = std::fs::read_dir(&qd) else {
        return Vec::new();
    };
    let mut names: Vec<String> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();
    names.sort();
    let mut waiters = Vec::new();
    for name in names {
        let path = qd.join(&name);
        if !is_held(&path) {
            continue;
        }
        let parts: Vec<&str> = name.split('-').collect();
        let mut waited = parts[0]
            .trim()
            .parse::<i64>()
            .map_or(0.0, |ns| (now.wall - ns as f64 / 1e9).max(0.0));
        let meta = read_meta(&path);
        let mut monotonic = false;
        if let Some(Value::Number(q)) = meta.get("queued_monotonic") {
            if let Some(q) = q.is_f64().then(|| q.as_f64()).flatten() {
                if q.is_finite() && 0.0 <= q && q <= now.mono {
                    monotonic = true;
                    waited = now.mono - q;
                }
            }
        }
        let need = parts
            .get(2)
            .filter(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
            .and_then(|p| p.parse::<i64>().ok())
            .unwrap_or(1);
        waiters.push(json!({
            "pid": parts.get(1).copied().unwrap_or("?"),
            "need": need,
            "waited_s": round1(waited),
            "job_class": meta.get("job_class").cloned().unwrap_or(json!("unknown")),
            "clock": if monotonic { "monotonic" } else { "legacy-wall" },
        }));
    }
    waiters
}

/// Read-only candidate: give integration a bounded 120 s head start. Older ordinary work
/// wins once its age exceeds that head start. Unknown or legacy metadata suppresses the
/// proposal rather than silently classifying another lane. This is NOT admission policy:
/// FIFO and every lock operation stay as they are.
pub fn admission_shadow(dir: &Path, waiters: &[Value]) -> Value {
    let mut result = json!({"mode": "shadow", "enforced": false, "boost_s": 120, "changed": false});
    if dir.join("admission-shadow.disabled").exists() {
        result["mode"] = json!("disabled");
        result["reason"] = json!("host-wide kill switch");
        return result;
    }
    let known = |w: &Value| {
        w["clock"] == "monotonic"
            && matches!(
                w["job_class"].as_str(),
                Some("interactive" | "integration" | "background")
            )
    };
    if !waiters.iter().all(known) {
        result["reason"] = json!("unknown or legacy waiter metadata; FIFO only");
        return result;
    }
    let key = |w: &Value| {
        w["waited_s"].as_f64().unwrap_or(0.0)
            + if w["job_class"] == "integration" {
                120.0
            } else {
                0.0
            }
    };
    let mut order: Vec<usize> = (0..waiters.len()).collect();
    order.sort_by(|&a, &b| key(&waiters[b]).total_cmp(&key(&waiters[a])));
    result["fifo"] = json!(waiters.iter().map(|w| w["pid"].clone()).collect::<Vec<_>>());
    result["proposed"] = json!(order
        .iter()
        .map(|&i| waiters[i]["pid"].clone())
        .collect::<Vec<_>>());
    result["changed"] = json!(order.iter().enumerate().any(|(i, &o)| i != o));
    result
}

/// Inputs `status` cannot read for itself: whether the broker is on (that check can refuse,
/// and the refusal belongs to the caller), the host build policy, and the default pool sizes.
pub struct StatusFacts<'a> {
    pub dir: &'a Path,
    pub enabled: bool,
    pub policy: &'a Value,
    pub cpu_leases: i64,
    pub boot_leases: i64,
    pub now: Now,
}

/// Snapshot of the host broker for `host-status`: which permits and locks are held, by
/// whom, whether that holder is still alive, and who is queued.
pub fn status(f: &StatusFacts) -> Value {
    let mut out = json!({
        "dir": f.dir.to_string_lossy(),
        "enabled": f.enabled,
        "cpuheavy": [], "bootcpu": [], "builddir": [], "land": [],
        "cpu_leases_total": f.cpu_leases,
        "cpu_leases_held": 0,
        "boot_leases_total": f.boot_leases,
        "boot_leases_held": 0,
    });
    if !f.dir.is_dir() {
        return out; // no lock dir yet: nothing held
    }
    let mut names: Vec<String> = std::fs::read_dir(f.dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();
    names.sort();
    for name in names {
        let path = f.dir.join(&name);
        if path.is_dir() {
            continue; // queue-<pool>/, reported by queue_status
        }
        let e = entry(&path, &name);
        let pool = if name.starts_with("cpuheavy-") {
            "cpuheavy"
        } else if name.starts_with("bootcpu-") {
            "bootcpu"
        } else if name.starts_with("builddir-") {
            "builddir"
        } else if name == "land" {
            "land"
        } else {
            continue;
        };
        out[pool].as_array_mut().unwrap().push(e);
    }
    let held = |pool: &str, out: &Value| {
        out[pool]
            .as_array()
            .map_or(0, |a| a.iter().filter(|e| e["held"] == true).count())
    };
    out["build_policy"] = f.policy.clone();
    out["cpu_leases_total"] = match f.policy.get("leases") {
        Some(n) if !f.policy.is_null() => n.clone(),
        _ => json!(f.cpu_leases),
    };
    out["cpu_leases_held"] = json!(held("cpuheavy", &out));
    out["boot_leases_held"] = json!(held("bootcpu", &out));
    let mut queue = Map::new();
    for pool in ["land", "cpuheavy", "bootcpu"] {
        queue.insert(pool.into(), json!(queue_status(f.dir, pool, f.now)));
    }
    let cpu: Vec<Value> = queue["cpuheavy"].as_array().cloned().unwrap_or_default();
    out["queue"] = Value::Object(queue);
    out["admission_shadow"] = admission_shadow(f.dir, &cpu);
    out
}

/// Python's `str()` of a JSON value, for holder fields printed as-is.
fn py_str(v: Option<&Value>, default: &str) -> String {
    match v {
        None => default.into(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) => "None".into(),
        Some(Value::Bool(b)) => if *b { "True" } else { "False" }.into(),
        Some(other) => other.to_string(),
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64() != Some(0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

fn plural(n: i64) -> &'static str {
    if n > 1 {
        "s"
    } else {
        ""
    }
}

/// The `vibeos-cli host-status` body.
pub fn render_status(st: &Value, reserve: i64) -> String {
    let num = |k: &str| st[k].as_i64().unwrap_or(0);
    let boot_total = num("boot_leases_total");
    let mut lines = vec![format!(
        "host broker: {}  —  legacy/native(cpuheavy) {}/{} · boot(windows-host) {}/{}{}",
        if st["enabled"] == true {
            "ON"
        } else {
            "OFF (VIBEOS_HOST_BROKER=0)"
        },
        st["cpu_leases_held"],
        st["cpu_leases_total"],
        num("boot_leases_held"),
        boot_total,
        if boot_total != 0 {
            format!(" ({reserve} reserved for landing)")
        } else {
            String::new()
        }
    )];
    let held = |pool: &str| -> Vec<&Value> {
        st[pool]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|e| e["held"] == true)
            .collect()
    };
    let stale = |e: &Value| {
        if truthy(e.get("holder_alive")) {
            ""
        } else {
            "  ⚠STALE(holder gone)"
        }
    };
    let cpu = held("cpuheavy");
    if cpu.is_empty() {
        lines.push("  cpuheavy : FREE — no legacy build / native TCG reservation".into());
    } else {
        // Group permits by holder pid, so an exclusive build (all K permits) is one line.
        let mut by_pid: Vec<(Value, Vec<&Value>)> = Vec::new();
        for e in cpu {
            let pid = e["holder"].get("pid").cloned().unwrap_or(Value::Null);
            match by_pid.iter_mut().find(|(p, _)| *p == pid) {
                Some((_, v)) => v.push(e),
                None => by_pid.push((pid, vec![e])),
            }
        }
        for (pid, entries) in by_pid {
            let h = &entries[0]["holder"];
            let n = entries.len() as i64;
            let tag = if truthy(h.get("exclusive")) {
                "EXCLUSIVE".to_owned()
            } else {
                format!("{n} permit{}", plural(n))
            };
            lines.push(format!(
                "  cpuheavy : {:<14} pid={} ({tag}){}  {}",
                py_str(h.get("kind"), "?"),
                py_str(Some(&pid), ""),
                stale(entries[0]),
                py_str(h.get("cwd"), ""),
            ));
        }
    }
    for e in held("bootcpu") {
        let h = &e["holder"];
        lines.push(format!(
            "  bootcpu  : {:<14} pid={}{}",
            py_str(h.get("kind"), "?"),
            py_str(h.get("pid"), "?"),
            stale(e)
        ));
    }
    for e in held("builddir") {
        let h = &e["holder"];
        lines.push(format!(
            "  builddir : {}  (pid={})",
            py_str(h.get("build_dir"), "?"),
            py_str(h.get("pid"), "?")
        ));
    }
    // Exactly one lane may be inside fetch→merge→gate→push at a time, so "who is landing
    // right now" answers most of "why is master moving under me".
    if let Some(e) = held("land").first() {
        let h = &e["holder"];
        lines.push(format!(
            "  LANDING  : {:<14} pid={}{}  {}",
            py_str(h.get("kind"), "?"),
            py_str(h.get("pid"), "?"),
            stale(e),
            py_str(h.get("cwd"), "")
        ));
    }
    // The queue, oldest first: starvation used to read as "my build is slow", not "I am
    // 3rd in line behind 20 minutes of waiting".
    if let Some(queue) = st["queue"].as_object() {
        for pool in ["land", "cpuheavy", "bootcpu"].into_iter().chain(
            queue
                .keys()
                .map(String::as_str)
                .filter(|k| !["land", "cpuheavy", "bootcpu"].contains(k)),
        ) {
            for (i, w) in queue
                .get(pool)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                let head = if i == 0 {
                    "NEXT".to_owned()
                } else {
                    format!("#{}  ", i + 1)
                };
                let need = w["need"].as_i64().unwrap_or(1);
                let want = if pool == "land" {
                    "to land".to_owned()
                } else {
                    format!("needs {need} permit{}", plural(need))
                };
                lines.push(format!(
                    "  queued   : {head} {pool:<9} pid={} {want}, waiting {:.0}s",
                    py_str(w.get("pid"), "?"),
                    w["waited_s"].as_f64().unwrap_or(0.0)
                ));
            }
        }
    }
    let skewed = ["cpuheavy", "bootcpu", "builddir"]
        .iter()
        .flat_map(|p| st[*p].as_array().into_iter().flatten())
        .filter(|e| e["protocol_skew"] == true)
        .count();
    if truthy(st.get("admission_shadow")) {
        lines.push(format!(
            "  admission: FIFO active; {}",
            vibeos_build::json::dumps(&st["admission_shadow"], true)
        ));
    }
    if skewed > 0 {
        lines.push(format!(
            "  ⚠ PROTOCOL SKEW: {skewed} live holder(s) speak a different lease protocol \
             (v{} here) — another lane runs divergent tooling; cooperate but expect \
             surprises until it merges",
            PROTOCOL_VERSION as i64
        ));
    }
    lines.push("  (QEMU boots: `vibeos-cli ps`)".into());
    lines.join("\n")
}

/// Holder metadata of every held boot permit NOT taken by `own_pid`: what a
/// `farm --polite` worker checks before each boot so a background soak never competes with
/// another agent's smoke or gate.
pub fn foreign_boot_holders(dir: &Path, own_pid: i64, leases: i64) -> Vec<Value> {
    (0..leases.max(0))
        .map(|i| dir.join(format!("bootcpu-{i:02}")))
        .filter(|p| is_held(p))
        .map(|p| read_meta(&p))
        // Python's `meta.get("pid") != own_pid`: only a NUMBER equal to ours is ours.
        .filter(|m| !matches!(m.get("pid"), Some(Value::Number(n)) if n.as_f64() == Some(own_pid as f64)))
        .map(Value::Object)
        .collect()
}

pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let text = |k: &str| a[k].as_str().unwrap_or("");
    let dir = Path::new(text("dir"));
    let now = || {
        let live = Now::read();
        Now {
            wall: a["now_wall"].as_f64().unwrap_or(live.wall),
            mono: a["now_mono"].as_f64().unwrap_or(live.mono),
        }
    };
    let reserve = |leases: i64| land_boot_reserve(leases, a["reserve_override"].as_str());
    match request["op"].as_str().unwrap_or("") {
        "hostlock_looks_starved" => Ok(json!(looks_starved(text("text")))),
        "hostlock_job_class" => Ok(json!(job_class(a["value"].as_str()))),
        "hostlock_land_boot_reserve" => Ok(json!(reserve(
            a["leases"].as_i64().ok_or("hostlock: leases")?
        )?)),
        "hostlock_is_held" => Ok(json!(is_held(Path::new(text("path"))))),
        "hostlock_queue_status" => Ok(json!(queue_status(dir, text("pool"), now()))),
        "hostlock_admission_shadow" => Ok(admission_shadow(
            dir,
            a["waiters"].as_array().map_or(&[][..], Vec::as_slice),
        )),
        "hostlock_status" => Ok(status(&StatusFacts {
            dir,
            enabled: a["enabled"] == true,
            policy: &a["policy"],
            cpu_leases: a["cpu_leases"].as_i64().ok_or("hostlock: cpu_leases")?,
            boot_leases: a["boot_leases"].as_i64().ok_or("hostlock: boot_leases")?,
            now: now(),
        })),
        "hostlock_render_status" => {
            let total = a["status"]["boot_leases_total"].as_i64().unwrap_or(0);
            Ok(json!(render_status(&a["status"], reserve(total)?)))
        }
        "hostlock_foreign_boot_holders" => Ok(json!(foreign_boot_holders(
            dir,
            a["own_pid"].as_i64().unwrap_or(0),
            a["leases"].as_i64().ok_or("hostlock: leases")?
        ))),
        op => Err(format!("unknown hostlock operation {op}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn hold(path: &Path, meta: &str) -> fs::File {
        fs::write(path, meta).unwrap();
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        assert_eq!(
            unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        f
    }

    #[test]
    fn a_starved_transcript_queued_and_never_acquired() {
        assert!(looks_starved(&format!("x\n{WAIT_ANNOUNCE} (build)...\n")));
        assert!(!looks_starved(&format!(
            "{WAIT_ANNOUNCE}\n{ACQUIRED_ANNOUNCE}"
        )));
        assert!(!looks_starved("FAILED test_x"));
    }

    #[test]
    fn the_landing_reserve_is_half_and_never_the_whole_pool() {
        assert_eq!(land_boot_reserve(8, None).unwrap(), 4);
        assert_eq!(land_boot_reserve(1, None).unwrap(), 0);
        assert_eq!(land_boot_reserve(4, Some("9")).unwrap(), 3);
        assert_eq!(land_boot_reserve(4, Some("-2")).unwrap(), 0);
        assert!(land_boot_reserve(4, Some("half")).is_err());
    }

    #[test]
    fn status_reports_live_holders_skew_and_never_reaps_an_orphan_ticket() {
        let d = tempfile::tempdir().unwrap();
        let me = std::process::id();
        let _cpu = hold(
            &d.path().join("cpuheavy-00"),
            &format!(r#"{{"v": 1, "kind": "build", "pid": {me}, "cwd": "/w"}}"#),
        );
        let _old = hold(
            &d.path().join("bootcpu-00"),
            &format!(r#"{{"kind": "smoke", "pid": {me}}}"#),
        );
        fs::write(d.path().join("cpuheavy-01"), "").unwrap(); // free
        let q = d.path().join("queue-cpuheavy");
        fs::create_dir(&q).unwrap();
        let _w = hold(
            &q.join("1000000000-77-2"),
            r#"{"job_class": "integration", "queued_monotonic": 50.0}"#,
        );
        fs::write(q.join("2000000000-88-1"), "{}").unwrap(); // orphan
        let st = status(&StatusFacts {
            dir: d.path(),
            enabled: true,
            policy: &Value::Null,
            cpu_leases: 4,
            boot_leases: 8,
            now: Now {
                wall: 11.0,
                mono: 60.0,
            },
        });
        assert_eq!(st["cpu_leases_held"], 1);
        assert_eq!(st["cpuheavy"][0]["holder_alive"], true);
        assert_eq!(st["bootcpu"][0]["protocol_skew"], true);
        let waiters = st["queue"]["cpuheavy"].as_array().unwrap();
        assert_eq!(waiters.len(), 1);
        assert_eq!(waiters[0]["waited_s"], 10.0);
        assert_eq!(waiters[0]["clock"], "monotonic");
        assert!(q.join("2000000000-88-1").exists());
        let text = render_status(&st, 4);
        assert!(text.contains("  cpuheavy : build          pid="), "{text}");
        assert!(text.contains("needs 2 permits, waiting 10s"), "{text}");
        assert!(text.contains("PROTOCOL SKEW: 1 live holder(s)"), "{text}");
    }
}
