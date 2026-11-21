//! Existing hostlock/VM slot protocols, held by native VM owners.
//! This is not a second capacity policy: file names, ticket ordering and reserve
//! arithmetic are shared with the legacy hostlock clients and host-jobs broker.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    os::unix::io::AsRawFd,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn error(message: impl Into<String>) -> Error {
    Error {
        kind: Some("AdmissionError".into()),
        message: message.into(),
    }
}
fn env_num(env: &BTreeMap<String, String>, key: &str, default: usize) -> Result<usize> {
    env.get(key)
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| error(format!("{key} must be a nonnegative integer")))
        })
        .unwrap_or(Ok(default))
}
fn on(env: &BTreeMap<String, String>, key: &str) -> bool {
    !matches!(
        env.get(key).map(|s| s.to_ascii_lowercase()).as_deref(),
        Some("0" | "false" | "no" | "")
    )
}
fn lock_file(path: &Path, exclusive: bool, nonblock: bool) -> Result<Option<File>> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    let f = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?;
    let flags = if exclusive {
        libc::LOCK_EX
    } else {
        libc::LOCK_SH
    } | if nonblock { libc::LOCK_NB } else { 0 };
    if unsafe { libc::flock(f.as_raw_fd(), flags) } == 0 {
        return Ok(Some(f));
    }
    let e = std::io::Error::last_os_error();
    if nonblock && matches!(e.raw_os_error(), Some(libc::EWOULDBLOCK)) {
        Ok(None)
    } else {
        Err(e.into())
    }
}
fn write_meta(file: &mut File, value: &Value) -> Result<()> {
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    serde_json::to_writer(&mut *file, value)?;
    file.flush()?;
    Ok(())
}
pub struct HeldLock {
    _file: File,
    pub path: PathBuf,
}
impl HeldLock {
    pub fn acquire(path: &Path, timeout: Duration, meta: &Value) -> Result<Self> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(mut f) = lock_file(path, true, true)? {
                write_meta(&mut f, meta)?;
                return Ok(Self {
                    _file: f,
                    path: path.into(),
                });
            }
            if Instant::now() >= deadline {
                return Err(error(format!("timed out waiting for {}", path.display())));
            }
            thread::sleep(Duration::from_millis(25));
        }
    }
}
struct Ticket {
    _file: File,
    path: PathBuf,
}
impl Ticket {
    fn take(root: &Path, pool: &str, enabled: bool, meta: &Value) -> Result<Option<Self>> {
        if !enabled {
            return Ok(None);
        }
        let dir = root.join(format!("queue-{pool}"));
        fs::create_dir_all(&dir)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| error(e.to_string()))?
            .as_nanos();
        let name = format!("{now:020}-{}-1", std::process::id());
        let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
        if unsafe { libc::flock(temp.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        serde_json::to_writer(&mut temp, meta)?;
        temp.flush()?;
        let path = dir.join(name);
        let file = temp.persist(&path).map_err(|e| error(e.to_string()))?;
        Ok(Some(Self { _file: file, path }))
    }
    fn first(&self) -> Result<bool> {
        let dir = self.path.parent().unwrap();
        let mut entries = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| !p.file_name().unwrap().to_string_lossy().starts_with('.'))
            .collect::<Vec<_>>();
        entries.sort();
        for p in entries {
            if p >= self.path {
                break;
            }
            if let Some(_orphan) = lock_file(&p, true, true)? {
                match fs::remove_file(&p) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub struct Admission {
    pub global_slot: usize,
    _global: HeldLock,
    _arch: Option<HeldLock>,
    _cpu: Option<HeldLock>,
    _policy_guard: Option<File>,
}
impl Admission {
    pub fn receipt(&self) -> Value {
        json!({"global_slot":self.global_slot,"arch_slot":self._arch.as_ref().map(|l|&l.path),"cpu_lease":self._cpu.as_ref().map(|l|&l.path),"policy_pinned":self._policy_guard.is_some()})
    }
    pub fn inherited_fds(&self) -> Vec<i32> {
        let mut result = vec![self._global._file.as_raw_fd()];
        if let Some(v) = &self._arch {
            result.push(v._file.as_raw_fd());
        }
        if let Some(v) = &self._cpu {
            result.push(v._file.as_raw_fd());
        }
        if let Some(v) = &self._policy_guard {
            result.push(v.as_raw_fd());
        }
        result
    }
    pub fn acquire(
        repo: &Path,
        arch: &str,
        backend: &str,
        accel: &str,
        work: &Path,
        env: &BTreeMap<String, String>,
    ) -> Result<Self> {
        let root = PathBuf::from(
            env.get("VIBEOS_QEMU_LOCK_DIR")
                .map(String::as_str)
                .unwrap_or("/tmp/vibeos-qemu-locks"),
        );
        let slots = root.join("slots");
        let timeout = env
            .get("VIBEOS_QEMU_QUEUE_TIMEOUT")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|_| error("invalid QEMU queue timeout"))
            })
            .transpose()?
            .unwrap_or(1800.0);
        if !timeout.is_finite() || timeout < 0.0 {
            return Err(error("QEMU queue timeout must be finite and nonnegative"));
        }
        let timeout = Duration::from_secs_f64(timeout);
        let meta = json!({"v":1,"pid":std::process::id(),"client_pid":env.get("VIBEOS_QEMU_CLIENT_PID").and_then(|v|v.parse::<u32>().ok()),"kind":format!("boot:{arch}"),"cwd":repo,"work_dir":work,"job_class":env.get("VIBEOS_JOB_CLASS").map(String::as_str).unwrap_or("interactive")});
        let cap = env_num(env, "VIBEOS_QEMU_MAX_CONCURRENT", 16)?;
        if cap == 0 {
            return Err(error("QEMU global cap must be positive"));
        }
        let deadline = Instant::now() + timeout;
        let (global_slot, global) = loop {
            let mut found = None;
            for i in 0..cap {
                let path = slots.join(format!("slot-{i:02}"));
                if let Some(mut f) = lock_file(&path, true, true)? {
                    write_meta(&mut f, &meta)?;
                    found = Some((i, HeldLock { _file: f, path }));
                    break;
                }
            }
            if let Some(v) = found {
                break v;
            }
            if Instant::now() >= deadline {
                return Err(error(format!(
                    "no free QEMU slot after {}s (concurrency cap={cap})",
                    timeout.as_secs_f64()
                )));
            }
            thread::sleep(Duration::from_millis(100));
        };
        let base = arch.split('-').next().unwrap_or(arch);
        let arch_cap = if backend == "windows-host" && accel != "whpx" {
            0
        } else {
            env_num(
                env,
                &format!("VIBEOS_QEMU_CAP_{}", base.to_ascii_uppercase()),
                if base == "arm64" {
                    0
                } else {
                    env_num(env, "VIBEOS_QEMU_PER_ARCH_CAP", 1)?
                },
            )?
        };
        let arch_lock = if arch_cap == 0 {
            None
        } else {
            let mut found = None;
            while found.is_none() {
                for i in 0..arch_cap {
                    let path = slots.join(format!("arch-{base}-{i:02}"));
                    if let Some(mut f) = lock_file(&path, true, true)? {
                        write_meta(&mut f, &meta)?;
                        found = Some(HeldLock { _file: f, path });
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(error(format!(
                        "no free same-arch ({base}) slot after {}s (per-arch cap={arch_cap})",
                        timeout.as_secs_f64()
                    )));
                }
                thread::sleep(Duration::from_millis(100));
            }
            found
        };
        let host = PathBuf::from(
            env.get("VIBEOS_HOSTLOCK_DIR")
                .map(String::as_str)
                .unwrap_or("/tmp/vibeos-qemu-locks/host"),
        );
        let tcg = if backend == "windows-host" {
            accel != "whpx"
        } else {
            accel != "kvm"
        };
        let mut policy_guard = None;
        let mut cpu = None;
        let policy_dir = match env.get("VIBEOS_BUILD_POLICY_DIR") {
            Some(p) => PathBuf::from(p),
            None => vibeos_build::build_policy::policy_dir(repo)?,
        };
        if !on(env, "VIBEOS_HOST_BROKER") {
            if vibeos_build::build_policy::read(&policy_dir)?.is_some()
                || policy_dir.join("build-policy.paused").exists()
            {
                return Err(error("active host build policy requires the CPU broker"));
            }
        } else if tcg {
            let windows = backend == "windows-host";
            let mut capacity = if windows {
                let cpus = thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(0);
                env_num(env, "VIBEOS_BOOT_CPU_LEASES", (cpus / 3).clamp(4, 12))?.max(1)
            } else {
                env_num(env, "VIBEOS_HOST_CPU_LEASES", 4)?.max(1)
            };
            if !windows {
                if let Some(f) = lock_file(&policy_dir.join("build-policy.lock"), false, false)? {
                    let policy = vibeos_build::build_policy::read(&policy_dir)?;
                    if let Some(p) = policy {
                        capacity =
                            p["leases"].as_u64().ok_or("invalid host policy leases")? as usize;
                    }
                    policy_guard = Some(f);
                }
                if policy_dir.join("build-policy.paused").exists() {
                    return Err(error("CPU admission paused for fleet drain; use host-policy --resume after cutover"));
                }
            }
            let landing = env
                .get("VIBEOS_JOB_CLASS")
                .is_some_and(|v| v == "integration");
            let reserve = if windows {
                env_num(env, "VIBEOS_LAND_BOOT_RESERVE", capacity / 2)?.min(capacity - 1)
            } else {
                0
            };
            let pool = if windows {
                if landing {
                    "bootcpu-land"
                } else {
                    "bootcpu"
                }
            } else {
                "cpuheavy"
            };
            let prefix = if windows { "bootcpu" } else { "cpuheavy" };
            let indices: Vec<usize> = if windows && landing {
                (capacity - reserve..capacity)
                    .chain(0..capacity - reserve)
                    .collect()
            } else {
                (0..capacity - reserve).collect()
            };
            let ticket = Ticket::take(&host, pool, on(env, "VIBEOS_HOSTLOCK_FAIR"), &meta)?;
            let budget = env
                .get("VIBEOS_HOST_LOCK_TIMEOUT")
                .map(|s| {
                    s.parse::<f64>()
                        .map_err(|_| error("invalid host lock timeout"))
                })
                .transpose()?
                .unwrap_or(3600.0);
            if !budget.is_finite() || budget < 0.0 {
                return Err(error("host lock timeout must be finite and nonnegative"));
            }
            let end = Instant::now() + Duration::from_secs_f64(budget);
            loop {
                if !windows && policy_dir.join("build-policy.paused").exists() {
                    return Err(error("CPU admission paused for fleet drain"));
                }
                if ticket
                    .as_ref()
                    .map(Ticket::first)
                    .transpose()?
                    .unwrap_or(true)
                {
                    for i in &indices {
                        let path = host.join(format!("{prefix}-{i:02}"));
                        if let Some(mut f) = lock_file(&path, true, true)? {
                            let mut m = meta.clone();
                            m["boot_pool"] = json!(windows);
                            m["landing"] = json!(landing);
                            write_meta(&mut f, &m)?;
                            cpu = Some(HeldLock { _file: f, path });
                            break;
                        }
                    }
                }
                if cpu.is_some() {
                    break;
                }
                if Instant::now() >= end {
                    return Err(error(format!(
                        "CPU admission timed out for {arch}; refusing unlocked boot"
                    )));
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
        Ok(Self {
            global_slot,
            _global: global,
            _arch: arch_lock,
            _cpu: cpu,
            _policy_guard: policy_guard,
        })
    }
}

// In admission.rs replace port_handshake with these two definitions. Existing
// launchers retain their signature; legacy clients bind their diagnostic PID.
pub fn port_handshake(
    env: &BTreeMap<String, String>,
    repo: &Path,
    timeout: Duration,
) -> Result<HeldLock> {
    port_handshake_owned(env, repo, timeout, std::process::id() as u64)
}
pub fn port_handshake_owned(
    env: &BTreeMap<String, String>,
    repo: &Path,
    timeout: Duration,
    owner_pid: u64,
) -> Result<HeldLock> {
    let root = PathBuf::from(
        env.get("VIBEOS_HOSTLOCK_DIR")
            .map(String::as_str)
            .unwrap_or("/tmp/vibeos-qemu-locks/host"),
    );
    let path = root.join("qemu-port-handshake.lock");
    Instant::now().checked_add(timeout).ok_or_else(|| {
        Error::new("QEMU port handshake timeout is not representable").with_kind("ValueError")
    })?;
    let meta = json!({"pid":owner_pid,"cwd":repo,"since":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|error(e.to_string()))?.as_secs_f64()});
    HeldLock::acquire(&path, timeout, &meta).map_err(|e| {
        if !e.message.starts_with("timed out waiting for ") {
            return e;
        }
        // Bounded diagnostics never determine ownership; flock does. The lock
        // inode is permanent and each caller opens a new CLOEXEC description.
        use std::io::Read;
        let mut bytes = Vec::new();
        let holder = File::open(&path)
            .and_then(|f| f.take(4096).read_to_end(&mut bytes))
            .ok()
            .and_then(|_| serde_json::from_slice::<Value>(&bytes).ok())
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or_else(|| String::from_utf8_lossy(&bytes).trim().to_owned());
        Error::new(format!(
            "QEMU port handshake timed out after {}s waiting for {}; last holder: {}",
            timeout.as_secs_f64(),
            path.display(),
            if holder.is_empty() {
                "owner metadata unavailable"
            } else {
                &holder
            }
        ))
        .with_kind("TimeoutError")
    })
}

#[cfg(test)]
mod tests {
    //! Boot admission as native VM owners take it. These were the Python lease API's tests,
    //! re-homed onto the code that actually admits boots now.
    use super::*;
    use std::os::unix::io::AsRawFd;

    struct Host {
        dir: tempfile::TempDir,
    }
    impl Host {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().unwrap(),
            }
        }
        fn host(&self) -> PathBuf {
            self.dir.path().join("host")
        }
        fn env(&self, extra: &[(&str, &str)]) -> BTreeMap<String, String> {
            let p = self.dir.path();
            let mut env: BTreeMap<String, String> = [
                (
                    "VIBEOS_QEMU_LOCK_DIR",
                    p.join("qemu").to_string_lossy().into_owned(),
                ),
                (
                    "VIBEOS_HOSTLOCK_DIR",
                    self.host().to_string_lossy().into_owned(),
                ),
                (
                    "VIBEOS_BUILD_POLICY_DIR",
                    p.join("policy").to_string_lossy().into_owned(),
                ),
                ("VIBEOS_HOST_LOCK_TIMEOUT", "0.3".into()),
                ("VIBEOS_QEMU_QUEUE_TIMEOUT", "0.3".into()),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
            for (k, v) in extra {
                env.insert((*k).into(), (*v).into());
            }
            env
        }
        fn boot(&self, extra: &[(&str, &str)]) -> Result<Admission> {
            self.admit("windows-host", "amd64", extra)
        }
        fn admit(&self, backend: &str, arch: &str, extra: &[(&str, &str)]) -> Result<Admission> {
            let p = self.dir.path();
            Admission::acquire(p, arch, backend, "tcg", p, &self.env(extra))
        }
        /// A raw holder, as any other client (native or an older lane's Python) holds one.
        fn hold(&self, name: &str) -> File {
            lock_file(&self.host().join(name), true, true)
                .unwrap()
                .unwrap()
        }
    }

    fn lease(a: &Admission) -> String {
        let p = a.receipt()["cpu_lease"].as_str().unwrap().to_owned();
        Path::new(&p)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    const LANE: (&str, &str) = ("VIBEOS_JOB_CLASS", "interactive");
    const GATE: (&str, &str) = ("VIBEOS_JOB_CLASS", "integration");
    const EIGHT: (&str, &str) = ("VIBEOS_BOOT_CPU_LEASES", "8");

    #[test]
    fn lane_boots_cannot_take_the_landing_reserve() {
        // A lane's smokes fill only the unreserved share, so a landing gate never queues
        // behind them (the night-long 17-red-to-1-green queue did).
        let h = Host::new();
        let lanes: Vec<_> = (0..4).map(|_| h.boot(&[EIGHT, LANE]).unwrap()).collect();
        let full = h.boot(&[EIGHT, LANE]).err().unwrap().message;
        assert!(full.contains("CPU admission timed out"), "{full}");
        let gates: Vec<_> = (0..4).map(|_| h.boot(&[EIGHT, GATE]).unwrap()).collect();
        assert_eq!(lanes.len() + gates.len(), 8);
    }

    #[test]
    fn landing_takes_reserved_permits_first_and_can_overflow() {
        let h = Host::new();
        let gates: Vec<_> = (0..8).map(|_| h.boot(&[EIGHT, GATE]).unwrap()).collect();
        let first: Vec<String> = gates[..4].iter().map(lease).collect();
        assert_eq!(
            first,
            ["bootcpu-04", "bootcpu-05", "bootcpu-06", "bootcpu-07"]
        );
        assert_eq!(lease(&gates[4]), "bootcpu-00"); // an idle host lends landing everything
        let meta: Value =
            serde_json::from_str(&fs::read_to_string(h.host().join("bootcpu-04")).unwrap())
                .unwrap();
        assert_eq!(meta["job_class"], "integration");
        assert_eq!(meta["landing"], true);
    }

    #[test]
    fn the_landing_reserve_never_takes_the_whole_pool() {
        let h = Host::new();
        let r99 = ("VIBEOS_LAND_BOOT_RESERVE", "99");
        let four = ("VIBEOS_BOOT_CPU_LEASES", "4");
        let lane = h.boot(&[four, r99, LANE]).unwrap();
        assert_eq!(lease(&lane), "bootcpu-00");
        assert!(h.boot(&[four, r99, LANE]).is_err());
    }

    #[test]
    fn the_boot_pool_throttles_boot_against_boot() {
        let h = Host::new();
        let two = [
            ("VIBEOS_BOOT_CPU_LEASES", "2"),
            ("VIBEOS_LAND_BOOT_RESERVE", "0"),
            LANE,
        ];
        let (_a, _b) = (h.boot(&two).unwrap(), h.boot(&two).unwrap());
        assert!(h.boot(&two).is_err());
    }

    #[test]
    fn a_build_holding_every_cpu_permit_does_not_block_a_windows_boot() {
        // Builds and windows-host boots draw from separate pools: the build runs on WSL
        // cores, the boot on the Windows side.
        let h = Host::new();
        let _build: Vec<_> = (0..4)
            .map(|i| h.hold(&format!("cpuheavy-{i:02}")))
            .collect();
        assert!(h.boot(&[LANE]).is_ok());
    }

    #[test]
    fn native_tcg_boots_share_the_cpu_pool_with_builds() {
        let h = Host::new();
        let two = [("VIBEOS_HOST_CPU_LEASES", "2")];
        let a = h.admit("native", "arm64", &two).unwrap();
        let _b = h.admit("native", "arm64", &two).unwrap();
        assert_eq!(lease(&a), "cpuheavy-00");
        assert!(h.admit("native", "arm64", &two).is_err()); // pool exhausted
        drop(a);
        assert!(h.admit("native", "arm64", &two).is_ok()); // freed on release
    }

    fn ticket(h: &Host, pool: &str, name: &str) -> File {
        lock_file(
            &h.host().join(format!("queue-{pool}")).join(name),
            true,
            true,
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn an_earlier_waiter_is_never_barged_and_a_dead_ones_ticket_is_reaped() {
        let h = Host::new();
        // Someone arrived first and is still waiting: a free pool is still theirs first.
        let ahead = ticket(&h, "bootcpu", "00000000000000000001-999999-1");
        assert!(h.boot(&[LANE]).is_err());
        // The waiter dies: the kernel drops its flock, and the next acquirer reaps it.
        drop(ahead);
        let admitted = h.boot(&[LANE]).unwrap();
        let queue = h.host().join("queue-bootcpu");
        assert_eq!(
            fs::read_dir(&queue).unwrap().count(),
            0,
            "orphan and own ticket gone"
        );
        drop(admitted);
    }

    #[test]
    fn fifo_can_be_disabled() {
        let h = Host::new();
        let _ahead = ticket(&h, "bootcpu", "00000000000000000001-999999-1");
        assert!(h.boot(&[LANE, ("VIBEOS_HOSTLOCK_FAIR", "0")]).is_ok());
    }

    #[test]
    fn landing_waits_in_its_own_queue() {
        let h = Host::new();
        let _lane_waiter = ticket(&h, "bootcpu", "00000000000000000001-999999-1");
        assert!(h.boot(&[GATE]).is_ok());
    }

    #[test]
    fn broker_off_takes_no_cpu_lease_unless_a_policy_requires_one() {
        let h = Host::new();
        let off = [("VIBEOS_HOST_BROKER", "0")];
        let a = h.admit("native", "arm64", &off).unwrap();
        assert!(a.receipt()["cpu_lease"].is_null());
        let policy = h.dir.path().join("policy");
        fs::create_dir_all(&policy).unwrap();
        fs::write(policy.join("build-policy.paused"), "").unwrap();
        let e = h.admit("native", "arm64", &off).err().unwrap().message;
        assert!(e.contains("requires the CPU broker"), "{e}");
    }

    #[test]
    fn a_pause_releases_a_waiter_but_never_interrupts_a_holder() {
        let h = Host::new();
        let one = [
            ("VIBEOS_HOST_CPU_LEASES", "1"),
            ("VIBEOS_HOST_LOCK_TIMEOUT", "10"),
        ];
        let holder = h.admit("native", "arm64", &one).unwrap();
        let env = h.env(&one);
        let root = h.dir.path().to_path_buf();
        let waiter = std::thread::spawn(move || {
            Admission::acquire(&root, "arm64", "native", "tcg", &root, &env).err()
        });
        thread::sleep(Duration::from_millis(300));
        fs::write(h.dir.path().join("policy/build-policy.paused"), "").unwrap();
        let e = waiter
            .join()
            .unwrap()
            .expect("the waiter is refused")
            .message;
        assert!(e.contains("paused"), "{e}");
        assert!(holder.receipt()["cpu_lease"].is_string());
        // The holder still owns its permit: a probe cannot take it.
        let probe = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(h.host().join("cpuheavy-00"))
            .unwrap();
        assert_ne!(
            unsafe { libc::flock(probe.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
    }
}
