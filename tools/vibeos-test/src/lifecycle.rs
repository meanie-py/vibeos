//! Scoped legacy ledger reaping and Linux CPU observation.
use crate::{backend, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Copy)]
struct Identity {
    pid: u32,
    parent: u32,
    group: u32,
    start: u64,
    cpu: u64,
    exited: bool,
    uid: u32,
}
fn identity(pid: u32) -> Result<Option<Identity>> {
    let dir = PathBuf::from(format!("/proc/{pid}"));
    // A process exiting mid-scan is gone, not an error. Its /proc entry can open and then
    // fail the READ with ESRCH, which is not ErrorKind::NotFound: that aborted the whole
    // group stop in reap, leaving an overdue QEMU running (test_native_reaper flake under
    // a 1333-test pytest load).
    let gone = |e: &std::io::Error| {
        e.kind() == std::io::ErrorKind::NotFound || e.raw_os_error() == Some(libc::ESRCH)
    };
    let stat = match fs::read_to_string(dir.join("stat")) {
        Ok(s) => s,
        Err(e) if gone(&e) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let fields = stat
        .rsplit_once(')')
        .ok_or("malformed proc stat")?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    let num = |i: usize| -> Result<u64> {
        fields
            .get(i)
            .ok_or("truncated proc stat")?
            .parse()
            .map_err(|_| "invalid proc stat".into())
    };
    // A process found during a group scan can vanish between stat and metadata.
    // Its disappearance must not abort termination of an unrelated owned QEMU.
    let uid = match fs::metadata(dir) {
        Ok(metadata) => metadata.uid(),
        Err(error) if gone(&error) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    Ok(Some(Identity {
        pid,
        parent: num(1)? as u32,
        group: num(2)? as u32,
        start: num(19)?,
        cpu: num(11)? + num(12)?,
        exited: matches!(fields.first().copied(), Some("Z" | "X")),
        uid,
    }))
}
fn ancestors() -> Result<BTreeSet<u32>> {
    let mut result = BTreeSet::new();
    let mut pid = std::process::id();
    while result.insert(pid) {
        let Some(id) = identity(pid)? else {
            break;
        };
        if id.parent == 0 {
            break;
        }
        pid = id.parent;
    }
    Ok(result)
}
fn signal(id: Identity, sig: i32) -> Result<()> {
    if id.uid != unsafe { libc::geteuid() } {
        return Err("refusing QEMU owned by another UID".into());
    }
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, id.pid, 0) } as i32;
    if fd < 0 {
        let e = std::io::Error::last_os_error();
        if e.raw_os_error() == Some(libc::ESRCH) {
            return Ok(());
        }
        return Err(e.into());
    }
    let result = (|| {
        if !identity(id.pid)?
            .is_some_and(|current| current.start == id.start && current.uid == id.uid)
        {
            return Err("QEMU PID incarnation changed".into());
        }
        if unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                fd,
                sig,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            )
        } < 0
        {
            let e = std::io::Error::last_os_error();
            if e.raw_os_error() != Some(libc::ESRCH) {
                return Err(e.into());
            }
        }
        Ok(())
    })();
    unsafe { libc::close(fd) };
    result
}
fn stop_linux(pid: u32, start: u64) -> Result<()> {
    let protected = ancestors()?;
    if protected.contains(&pid) {
        return Err("refusing to stop our own ancestor".into());
    }
    let Some(leader) = identity(pid)? else {
        return Ok(());
    };
    if leader.start != start {
        return Err("recorded QEMU PID has been reused; retaining ledger".into());
    }
    if leader.uid != unsafe { libc::geteuid() } {
        return Err("QEMU belongs to another UID".into());
    }
    let mut members = BTreeMap::new();
    members.insert(pid, leader);
    if leader.group == pid {
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let Ok(member) = entry.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            if let Some(id) = identity(member)? {
                if id.group == pid && id.uid == leader.uid {
                    if protected.contains(&member) {
                        return Err("QEMU process group includes our ancestor".into());
                    }
                    members.insert(member, id);
                }
            }
        }
    }
    for id in members.values() {
        signal(*id, libc::SIGTERM)?;
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let live = members
            .values()
            .filter(|id| {
                identity(id.pid)
                    .ok()
                    .flatten()
                    .is_some_and(|v| v.start == id.start && !v.exited)
            })
            .count();
        if live == 0 {
            return Ok(());
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    for id in members.values() {
        if identity(id.pid)?.is_some_and(|v| v.start == id.start && !v.exited) {
            signal(*id, libc::SIGKILL)?;
        }
    }
    // A zombie is dead; its parent may reap it later. Signal delivery cannot create reuse risk.
    Ok(())
}
fn pid_alive(pid: u32) -> bool {
    unsafe {
        libc::kill(pid as i32, 0) == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
}
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn uid_of(info: &Value) -> Option<String> {
    if let Some(uid) = info["win_uid"]
        .as_str()
        .or_else(|| info["uid"].as_str())
        .filter(|v| !v.is_empty())
    {
        return Some(uid.into());
    }
    let argv: Vec<String> = if let Some(values) = info["cmd"].as_array() {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect()
    } else {
        shlex::split(info["cmd"].as_str().unwrap_or("")).unwrap_or_default()
    };
    for (index, arg) in argv.iter().enumerate() {
        if arg == "-name" {
            return argv.get(index + 1).cloned();
        }
        if let Some(uid) = arg.strip_prefix("-name=") {
            return Some(uid.into());
        }
    }
    None
}

fn reap_one(path: &Path, manifest: bool, dry: bool) -> Result<Option<String>> {
    let bytes = fs::read(path)?;
    let info: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if !info.is_object() || info.get("stopped_at").is_some_and(|v| !v.is_null()) {
        return Ok(None);
    }
    let owner = info["launcher_pid"]
        .as_u64()
        .or_else(|| info["owner_pid"].as_u64())
        .or_else(|| info["parent_pid"].as_u64());
    let expired = info["deadline"].as_f64().is_some_and(|t| now() >= t);
    let owner_tick = info["launcher_start_tick"]
        .as_u64()
        .or_else(|| info["owner_start_tick"].as_u64());
    let owner_alive = owner.is_some_and(|p| match owner_tick {
        Some(t) => identity(p as u32)
            .ok()
            .flatten()
            .is_some_and(|i| i.start == t),
        None => pid_alive(p as u32),
    });
    if !expired && (owner_alive || manifest && owner.is_none()) {
        return Ok(None);
    }
    let pid = info["qemu_pid"].as_u64().map(|p| p as u32);
    let uid = uid_of(&info);
    if dry {
        return Ok(Some(format!(
            "DRY-RUN would reap qemu pid={pid:?} ({})",
            if expired {
                "exceeded lifetime cap"
            } else {
                "owner dead"
            }
        )));
    }
    if let Some(uid) = uid {
        if backend::kill_by_uid(&uid)? != Some(0) {
            return Err(Error::new(format!(
                "Windows QEMU stop unconfirmed for {uid}; retaining {}",
                path.display()
            )));
        }
    }
    if let Some(pid) = pid {
        if identity(pid)?.is_some() {
            let start = info["qemu_start_tick"]
                .as_u64()
                .or_else(|| info["qemu_start_time"].as_u64())
                .ok_or("live QEMU lacks recorded start-tick proof; retaining ledger")?;
            stop_linux(pid, start)?;
        }
    }
    // Do not erase a new owner's atomic replacement of the same ledger path.
    if fs::read(path)? != bytes {
        return Err("QEMU ledger changed while reaping; retaining replacement".into());
    }
    if manifest {
        let mut info = info;
        info["stopped_at"] = json!(now());
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, serde_json::to_vec_pretty(&info)?)?;
        fs::rename(temporary, path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(Some(format!(
        "killed orphan qemu pid={pid:?} ({})",
        if expired {
            "exceeded lifetime cap"
        } else {
            "owner dead"
        }
    )))
}
pub fn reap(root: &Path, env: &BTreeMap<String, String>, dry: bool) -> Result<Vec<String>> {
    let locks = env
        .get("VIBEOS_QEMU_LOCK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| "/tmp/vibeos-qemu-locks".into());
    let runs = env
        .get("RUNS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("qemu/runs"));
    let mut paths = Vec::new();
    if locks.is_dir() {
        for entry in fs::read_dir(locks)? {
            let path = entry?.path();
            if path.extension().is_some_and(|v| v == "json") {
                paths.push((path, false));
            }
        }
    }
    if runs.is_dir() {
        for entry in fs::read_dir(runs)? {
            let path = entry?.path().join("manifest.json");
            if path.is_file() {
                paths.push((path, true));
            }
        }
    }
    paths.sort();
    let mut actions = Vec::new();
    for (path, manifest) in paths {
        match reap_one(&path, manifest, dry) {
            Ok(Some(line)) => actions.push(line),
            Ok(None) => {}
            Err(e) => actions.push(format!("FAILED to reap {}: {}", path.display(), e.message)),
        }
    }
    Ok(actions)
}
pub fn process_cpu_pct(pid: u32, window: f64) -> Result<f64> {
    if !window.is_finite() || window < 0.0 {
        return Err("CPU observation window must be finite and nonnegative".into());
    }
    let Some(before) = identity(pid)? else {
        return Ok(0.0);
    };
    let begin = Instant::now();
    thread::sleep(Duration::from_secs_f64(window));
    let Some(after) = identity(pid)? else {
        return Ok(0.0);
    };
    if before.start != after.start {
        return Err("CPU sample PID incarnation changed".into());
    }
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if hz <= 0 {
        return Err("cannot determine scheduler tick frequency".into());
    }
    Ok(after.cpu.saturating_sub(before.cpu) as f64
        / hz as f64
        / begin.elapsed().as_secs_f64().max(f64::EPSILON)
        * 100.0)
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    Ok(match request["op"].as_str().unwrap_or("") {
        "vm_reap_orphan_qemus" => {
            let env = serde_json::from_value(a.get("env").cloned().unwrap_or_else(|| json!({})))?;
            json!(reap(root, &env, a["dry_run"].as_bool().unwrap_or(false))?)
        }
        "vm_process_cpu_pct" => json!(process_cpu_pct(
            a["pid"].as_u64().ok_or("missing PID")? as u32,
            a["window"].as_f64().unwrap_or(3.0)
        )?),
        "vm_per_arch_cap" => {
            let env: BTreeMap<String, String> =
                serde_json::from_value(a.get("env").cloned().unwrap_or_else(|| json!({})))?;
            let base = a["arch"]
                .as_str()
                .ok_or("missing arch")?
                .split('-')
                .next()
                .unwrap();
            let raw = env
                .get(&format!("VIBEOS_QEMU_CAP_{}", base.to_ascii_uppercase()))
                .or_else(|| env.get("VIBEOS_QEMU_PER_ARCH_CAP"));
            json!(if base == "arm64" && raw.is_none() {
                0
            } else {
                raw.map(String::as_str)
                    .unwrap_or("1")
                    .parse::<u64>()
                    .map_err(|_| Error::new("invalid per-arch cap"))?
            })
        }
        _ => return Err("unknown lifecycle operation".into()),
    })
}

/// The factory/start/stop are compatibility object callbacks; retry decisions and timing are native.
pub fn launch_retry(host: &mut dyn crate::callback::Host, a: &Value) -> Result<Value> {
    let attempts = a["attempts"].as_u64().unwrap_or(3);
    let pause = a["retry_seconds"].as_f64().unwrap_or(2.0);
    if attempts == 0 || !pause.is_finite() || pause < 0.0 {
        return Err("invalid launch retry policy".into());
    }
    let mut last = Value::Null;
    for attempt in 1..=attempts {
        host.call("launch_make", json!({"attempt":attempt}))?;
        let started = Instant::now();
        let result = host.call("launch_start", json!({"attempt":attempt}))?;
        if result.get("error").is_none() {
            let mut timings = result["timings"].as_object().cloned().unwrap_or_default();
            timings.insert(
                "start_seconds".into(),
                json!((started.elapsed().as_secs_f64() * 1000.0).round() / 1000.0),
            );
            if attempt > 1 {
                timings.insert("launch_attempts".into(), json!(attempt));
                host.call("launch_log",json!({"message":format!("{}: launched on attempt {attempt} (host-side flake)",a["label"].as_str().unwrap_or("launch"))}))?;
            }
            return Ok(json!({"attempts":attempt,"timings":timings,"error":null}));
        }
        last = result["error"].clone();
        host.call("launch_log",json!({"message":format!("{}: start failed (attempt {attempt}/{attempts}): {}",a["label"].as_str().unwrap_or("launch"),last.as_str().unwrap_or("unknown launch error"))}))?;
        host.call("launch_stop", json!({}))?;
        if attempt < attempts {
            thread::sleep(Duration::from_secs_f64(pause));
        }
    }
    Ok(json!({"attempts":attempts,"timings":{},"error":last}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_windows_uid_is_read_from_owned_argv() {
        assert_eq!(
            uid_of(&json!({"cmd":["qemu.exe","-name","vibeos-12-34"]})),
            Some("vibeos-12-34".into())
        );
        assert_eq!(
            uid_of(&json!({"cmd":"qemu.exe -name=vibeos-12-34"})),
            Some("vibeos-12-34".into())
        );
    }
    #[test]
    fn refuses_own_ancestor_before_signaling() {
        let pid = std::process::id();
        let current = identity(pid).unwrap().unwrap();
        assert!(stop_linux(pid, current.start)
            .unwrap_err()
            .message
            .contains("ancestor"));
    }
    #[test]
    fn reused_pid_refuses_before_signaling() {
        let child = std::process::Command::new("sleep")
            .arg("60")
            .spawn()
            .unwrap();
        let mut child = child;
        let id = identity(child.id()).unwrap().unwrap();
        let result = stop_linux(id.pid, id.start + 1);
        assert!(result.unwrap_err().message.contains("reused"));
        assert!(child.try_wait().unwrap().is_none());
        child.kill().unwrap();
        child.wait().unwrap();
    }
}

#[cfg(test)]
mod terminal_process_controls {
    use super::*;
    #[test]
    fn unreaped_children_are_exited_while_their_pid_still_exists() {
        let mut child = std::process::Command::new("/bin/true").spawn().unwrap();
        let pid = child.id();
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // Observe exit without reaping: deliberately retain the /proc zombie.
        assert_eq!(
            unsafe { libc::waitid(libc::P_PID, pid, &mut info, libc::WEXITED | libc::WNOWAIT) },
            0
        );
        let zombie = identity(pid).unwrap().unwrap();
        assert!(zombie.exited);
        assert!(!identity(std::process::id()).unwrap().unwrap().exited);
        stop_linux(pid, zombie.start).unwrap();
        assert!(child.wait().unwrap().success());
    }
}
