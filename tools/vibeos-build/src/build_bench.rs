//! Reproducible build/lint benchmarking with independently sampled host resources.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const ENV_KEYS: &[&str] = &[
    "VIBEOS_BUILD_CACHE",
    "VIBEOS_HOST_JOBS",
    "VIBEOS_BUILD_CORE_RESERVE",
    "VIBEOS_BUILD_TIMEOUT",
    "CARGO_BUILD_JOBS",
    "MAKEFLAGS",
    "NINJAFLAGS",
    "CCACHE_DIR",
    "CCACHE_MAXSIZE",
    "SCCACHE_DIR",
    "SCCACHE_CACHE_SIZE",
    "RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "VIBEOS_HOST_MEMORY_MIB",
    "VIBEOS_HOST_MEMORY_HEADROOM_MIB",
    "VIBEOS_HOST_JOB_MEMORY_MIB",
    "VIBEOS_HOST_JOB_SLOTS",
    "VIBEOS_HOST_LINK_SLOTS",
    "VIBEOS_LINT_JOBS",
];
static NEXT: AtomicU64 = AtomicU64::new(0);
fn unique() -> String {
    format!(
        "{}-{}-{}",
        std::process::id(),
        now().to_bits(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn timestamp(compact: bool) -> String {
    let time = unsafe { libc::time(std::ptr::null_mut()) };
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    unsafe {
        libc::gmtime_r(&time, &mut tm);
    }
    if compact {
        format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    } else {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    }
}
pub fn read_meminfo(path: &Path) -> Result<BTreeMap<String, u64>> {
    let mut out = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let fields = rest.split_whitespace().collect::<Vec<_>>();
        if let Some(value) = fields.first().and_then(|s| s.parse::<u64>().ok()) {
            let multiplier = if fields.get(1) == Some(&"kB") {
                1024
            } else {
                1
            };
            out.insert(
                key.into(),
                value
                    .checked_mul(multiplier)
                    .ok_or("meminfo value overflows")?,
            );
        }
    }
    if out.is_empty() {
        return Err(Error::new(format!(
            "no memory readings in {}",
            path.display()
        )));
    }
    Ok(out)
}
pub fn read_vmstat(path: &Path) -> Result<BTreeMap<String, u64>> {
    let mut out = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() == 2
            && (fields[0].starts_with("oom") || fields[0].starts_with("pgmajfault"))
        {
            if let Ok(n) = fields[1].parse() {
                out.insert(fields[0].into(), n);
            }
        }
    }
    if out.is_empty() {
        return Err(Error::new(format!(
            "no OOM or major-fault counters in {}",
            path.display()
        )));
    }
    Ok(out)
}
pub fn read_pressure(path: &Path) -> Result<Value> {
    let mut out = serde_json::Map::new();
    for resource in ["memory", "cpu", "io"] {
        let data = match fs::read_to_string(path.join(resource)) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                out.insert(
                    resource.into(),
                    json!({"available":false,"reason":"kernel PSI file absent"}),
                );
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        let mut rows = serde_json::Map::new();
        for line in data.lines() {
            let mut words = line.split_whitespace();
            let Some(name) = words.next() else { continue };
            let mut values = serde_json::Map::new();
            for word in words {
                let (k, v) = word
                    .split_once('=')
                    .ok_or_else(|| Error::new(format!("malformed PSI entry {word}")))?;
                let number: f64 = v
                    .parse()
                    .map_err(|_| Error::new(format!("invalid PSI value {word}")))?;
                if !number.is_finite() {
                    return Err(Error::new("non-finite PSI value"));
                }
                values.insert(k.into(), json!(number));
            }
            rows.insert(name.into(), Value::Object(values));
        }
        out.insert(resource.into(), json!({"available":true,"values":rows}));
    }
    Ok(Value::Object(out))
}
fn process_tree(roots: &[u32]) -> Value {
    let mut parents = BTreeMap::<u32, u32>::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return json!({"available":false,"reason":"cannot enumerate /proc"});
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        let Some((_, tail)) = stat.rsplit_once(')') else {
            continue;
        };
        let Some(ppid) = tail
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        parents.insert(pid, ppid);
    }
    let mut selected: BTreeSet<u32> = roots.iter().copied().collect();
    loop {
        let old = selected.len();
        for (&pid, &parent) in &parents {
            if selected.contains(&parent) {
                selected.insert(pid);
            }
        }
        if selected.len() == old {
            break;
        }
    }
    let mut rows = Vec::new();
    let mut sum = 0u64;
    let mut missing = 0usize;
    for pid in selected {
        let Some(parent) = parents.get(&pid) else {
            missing += 1;
            continue;
        };
        let rss = fs::read_to_string(format!("/proc/{pid}/status"))
            .ok()
            .and_then(|s| {
                s.lines().find_map(|line| {
                    line.strip_prefix("VmRSS:")
                        .and_then(|v| v.split_whitespace().next())
                        .and_then(|v| v.parse::<u64>().ok())
                })
            })
            .and_then(|n| n.checked_mul(1024));
        if let Some(n) = rss {
            sum = sum.saturating_add(n);
        } else {
            missing += 1;
        }
        rows.push(json!({"pid":pid,"ppid":parent,"rss_bytes":rss}));
    }
    json!({"available":true,"rss_bytes":sum,"missing_rss_count":missing,"processes":rows})
}
fn cgroup_memory() -> Value {
    let body = match fs::read_to_string("/proc/self/cgroup") {
        Ok(s) => s,
        Err(e) => return json!({"available":false,"reason":e.to_string()}),
    };
    let Some(relative) = body.lines().find_map(|line| line.strip_prefix("0::")) else {
        return json!({"available":false,"reason":"unified cgroup not found"});
    };
    let path = Path::new(relative.trim_start_matches('/'));
    if path
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
        && !path.as_os_str().is_empty()
    {
        return json!({"available":false,"reason":"invalid cgroup path"});
    }
    let path = Path::new("/sys/fs/cgroup")
        .join(path)
        .join("memory.current");
    match fs::read_to_string(&path) {
        Ok(s) => match s.trim().parse::<u64>() {
            Ok(n) => json!({"available":true,"path":path,"current_bytes":n}),
            Err(_) => {
                json!({"available":false,"path":path,"reason":"invalid cgroup memory.current"})
            }
        },
        Err(e) => json!({"available":false,"path":path,"reason":e.to_string()}),
    }
}
fn atomic_write(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().ok_or("output has no parent")?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .ok_or("invalid output name")?
            .to_string_lossy(),
        unique()
    ));
    let operation = (|| -> Result<()> {
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        serde_json::to_writer_pretty(&mut f, value)?;
        f.write_all(b"\n")?;
        f.sync_all()?;
        fs::rename(&tmp, path)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if operation.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    operation
}
fn git_state(root: &Path) -> Result<Value> {
    fn git(root: &Path, args: &[&str]) -> Result<String> {
        let o = Command::new("git").args(args).current_dir(root).output()?;
        if !o.status.success() {
            return Err(Error::new(format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&o.stderr).trim()
            )));
        }
        Ok(String::from_utf8(o.stdout)
            .map_err(|e| Error::new(e.to_string()))?
            .trim()
            .into())
    }
    let status = git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    let diff = git(root, &["diff", "HEAD", "--binary"])?;
    Ok(
        json!({"head":git(root,&["rev-parse","HEAD"])?,"dirty":!status.is_empty(),"dirty_status":status,"diff_sha256":format!("{:x}",Sha256::digest(diff.as_bytes()))}),
    )
}
fn environment() -> Value {
    let m: BTreeMap<_, _> = ENV_KEYS
        .iter()
        .filter_map(|key| std::env::var(key).ok().map(|v| ((*key).to_owned(), v)))
        .collect();
    json!(m)
}
#[derive(Clone, Debug)]
pub struct Options {
    pub arches: Vec<String>,
    pub clean: bool,
    pub repeat: usize,
    pub output: Option<PathBuf>,
}
#[derive(Clone, Debug)]
pub struct ChildSpec {
    pub name: String,
    pub argv: Vec<String>,
}
/// Injectable only at library level, enabling deterministic phase/failure tests.
pub trait Host {
    fn git(&mut self, root: &Path) -> Result<Value> {
        git_state(root)
    }
    fn policy(&mut self, root: &Path) -> Result<Value>;
    fn broker(&mut self, root: &Path, policy: &Value) -> Result<Value>;
    fn children(
        &mut self,
        root: &Path,
        specs: &[ChildSpec],
        dir: &Path,
        pids: Arc<Mutex<BTreeSet<u32>>>,
    ) -> Result<Vec<Value>> {
        run_children(root, specs, dir, pids)
    }
}

struct Sampler {
    stop: mpsc::Sender<()>,
    thread: Option<std::thread::JoinHandle<Result<(usize, Option<u64>)>>>,
}
impl Sampler {
    fn start(path: PathBuf, pids: Arc<Mutex<BTreeSet<u32>>>) -> Result<Self> {
        let (stop, recv) = mpsc::channel();
        let (ready, started) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || -> Result<(usize, Option<u64>)> {
            let run = || -> Result<(usize, Option<u64>)> {
                let mut stream = OpenOptions::new().write(true).create_new(true).open(path)?;
                let mut count = 0;
                let mut min = None;
                loop {
                    let mem = read_meminfo(Path::new("/proc/meminfo"))?;
                    let available = *mem.get("MemAvailable").ok_or("missing MemAvailable")?;
                    let roots: Vec<_> = pids
                        .lock()
                        .map_err(|_| Error::new("process registry poisoned"))?
                        .iter()
                        .copied()
                        .collect();
                    let row = json!({"time":now(),"meminfo_bytes":mem,"vmstat":read_vmstat(Path::new("/proc/vmstat"))?,"pressure":read_pressure(Path::new("/proc/pressure"))?,"process_tree":process_tree(&roots),"cgroup_memory":cgroup_memory()});
                    serde_json::to_writer(&mut stream, &row)?;
                    stream.write_all(b"\n")?;
                    stream.flush()?;
                    count += 1;
                    min = Some(min.map_or(available, |v: u64| v.min(available)));
                    if count == 1 {
                        let _ = ready.send(Ok(()));
                    }
                    match recv.recv_timeout(Duration::from_millis(200)) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => (),
                    }
                }
                stream.sync_all()?;
                Ok((count, min))
            };
            let result = run();
            if let Err(e) = &result {
                let _ = ready.send(Err(e.to_string()));
            }
            result
        });
        match started.recv() {
            Ok(Ok(())) => Ok(Self {
                stop,
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(Error::new(format!("resource sampler failed: {e}")))
            }
            Err(e) => {
                let _ = thread.join();
                Err(Error::new(format!("resource sampler failed: {e}")))
            }
        }
    }
    fn stop(mut self) -> Result<(usize, Option<u64>)> {
        let _ = self.stop.send(());
        self.thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| Error::new("resource sampler panicked"))?
            .map_err(|e| Error::new(format!("resource sampler failed: {e}")))
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Child helper runs exactly one supervised process group; its parent owns the log file.
pub fn worker_cli(argv: &[String]) -> Result<u8> {
    if argv.len() < 2 {
        return Err(Error::new("benchmark worker requires receipt and command"));
    }
    let receipt = Path::new(&argv[0]);
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(receipt)?;
    crate::process_guard::controller(&argv[1..], &mut file)
}
fn error_row(
    spec: &ChildSpec,
    log: &Path,
    error: impl std::fmt::Display,
    started: Instant,
) -> Value {
    let error = format!("{error}");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log) {
        let _ = writeln!(f, "{error}");
    }
    json!({"name":spec.name,"command":spec.argv,"log":log,"exit_code":127,"status":"FAILED","launch_error":error,"wall_seconds":started.elapsed().as_secs_f64()})
}
struct ChildGuard(Option<std::process::Child>);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            unsafe {
                libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(6);
            while Instant::now() < deadline {
                match child.try_wait() {
                    Ok(Some(_)) => return,
                    Err(_) => break,
                    _ => std::thread::sleep(Duration::from_millis(20)),
                }
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn run_child(root: &Path, spec: &ChildSpec, log: &Path, pids: Arc<Mutex<BTreeSet<u32>>>) -> Value {
    let started = Instant::now();
    let run = || -> Result<Value> {
        if spec.argv.is_empty() {
            return Err(Error::new("empty child command"));
        }
        let f = OpenOptions::new().create_new(true).write(true).open(log)?;
        let artifact_root = log.with_extension("artifacts");
        let receipt = log.with_extension("status.json");
        let mut cmd = Command::new(std::env::current_exe()?);
        cmd.arg("--root")
            .arg(root)
            .arg("build-bench-worker")
            .arg(&receipt)
            .args(&spec.argv)
            .current_dir(root)
            .env("VIBEOS_ARTIFACT_ROOT", &artifact_root)
            .stdout(Stdio::from(f.try_clone()?))
            .stderr(Stdio::from(f));
        let mut guard = ChildGuard(Some(cmd.spawn()?));
        let pid = guard.0.as_ref().unwrap().id();
        pids.lock()
            .map_err(|_| Error::new("process registry poisoned"))?
            .insert(pid);
        let status = guard.0.as_mut().unwrap().wait();
        if status.is_ok() {
            guard.0.take();
        }
        pids.lock()
            .map_err(|_| Error::new("process registry poisoned"))?
            .remove(&pid);
        let status = status?;
        use std::os::unix::process::ExitStatusExt;
        let worker_code = status
            .code()
            .unwrap_or_else(|| -status.signal().unwrap_or(1));
        let receipt_value = crate::artifact_fs::verified_read(&receipt, "benchmark worker receipt")
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        let receipt_code = receipt_value
            .as_ref()
            .and_then(|value| value["exit_code"].as_i64())
            .and_then(|value| i32::try_from(value).ok());
        let reported_worker = receipt_value
            .as_ref()
            .filter(|value| value["child_status_unknown"] == true)
            .and_then(|value| value["worker_exit_code"].as_i64())
            .and_then(|n| i32::try_from(n).ok())
            .filter(|&code| {
                (if code < 0 {
                    (128_i64 - i64::from(code)).min(255) as i32
                } else {
                    code.min(255)
                }) == worker_code
            })
            .unwrap_or(worker_code);
        let valid_code = receipt_code.filter(|&code| {
            let expected = if code < 0 {
                (128_i64 - i64::from(code)).min(255) as i32
            } else {
                code.min(255)
            };
            expected == worker_code
        });
        let mut code = valid_code.unwrap_or(1);
        let mut summaries = Vec::<PathBuf>::new();
        match fs::read_dir(&artifact_root) {
            Ok(entries) => {
                for entry in entries {
                    let path = entry?.path().join("summary.json");
                    if path.is_file() {
                        summaries.push(path);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
        summaries.sort();
        let mut artifact_error = if valid_code.is_none() {
            Some("worker exited without a valid child status receipt")
        } else {
            None
        };
        if code == 0 {
            if summaries.is_empty() {
                artifact_error = Some("missing or unsuccessful child summary");
            } else {
                for path in &summaries {
                    let v: Value = serde_json::from_slice(&fs::read(path)?)?;
                    if !matches!(v["verdict"].as_str(), Some("OK" | "WARN")) {
                        artifact_error = Some("missing or unsuccessful child summary");
                        break;
                    }
                }
            }
            if artifact_error.is_some() {
                code = 1;
            }
        }
        let mut row = json!({"name":spec.name,"command":spec.argv,"log":log,"exit_code":code,"summaries":summaries,"status":if code==0{"OK"}else{"FAILED"},"status_receipt":receipt,"wall_seconds":started.elapsed().as_secs_f64()});
        if let Some(e) = artifact_error {
            row["artifact_error"] = json!(e);
        }
        if valid_code.is_none() {
            row["worker_exit_code"] = json!(reported_worker);
        }
        if code == 127 {
            if let Some(message) = receipt_value.as_ref().and_then(|v| v.get("launch_error")) {
                row["launch_error"] = message.clone();
            }
        }
        Ok(row)
    };
    match run() {
        Ok(row) => row,
        Err(e) => error_row(spec, log, e, started),
    }
}
pub fn run_children(
    root: &Path,
    specs: &[ChildSpec],
    dir: &Path,
    pids: Arc<Mutex<BTreeSet<u32>>>,
) -> Result<Vec<Value>> {
    let mut rows = std::thread::scope(|scope| {
        let handles: Vec<_> = specs
            .iter()
            .map(|spec| {
                let pids = pids.clone();
                scope.spawn(move || {
                    run_child(root, spec, &dir.join(format!("{}.log", spec.name)), pids)
                })
            })
            .collect();
        handles.into_iter().enumerate().map(|(i,h)|match h.join(){Ok(v)=>v,Err(_)=>json!({"name":specs[i].name,"command":[],"log":null,"exit_code":127,"status":"FAILED","launch_error":"child worker panicked"})}).collect::<Vec<_>>()
    });
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(rows)
}
pub fn media_path(root: &Path, arch: &str) -> Result<PathBuf> {
    crate::arch(arch)?;
    Ok(if arch == "arm64" {
        root.join("qemu/ramdisk.img")
    } else {
        root.join(format!("build-{arch}/livecd.iso"))
    })
}
pub fn check_artifacts(root: &Path, arches: &[String], children: &mut [Value]) -> Result<Value> {
    let mut map = serde_json::Map::new();
    for arch in arches {
        let path = media_path(root, arch)?;
        let exists = path.is_file();
        let mut entry = json!({"path":path,"exists":exists,"size_bytes":if exists{Some(fs::metadata(&path)?.len())}else{None}});
        let row = children
            .iter_mut()
            .find(|r| r["name"] == format!("build-{arch}"))
            .ok_or_else(|| Error::new(format!("missing build child for {arch}")))?;
        if !exists {
            row["artifact_error"] = json!("expected boot media is absent");
        }
        if arch == "arm64" {
            let boot = root.join("qemu/boot.img");
            let bytes = if boot.is_file() {
                fs::metadata(&boot)?.len()
            } else {
                0
            };
            entry["boot_image"] = json!(boot);
            entry["boot_image_bytes"] = json!(bytes);
            if bytes == 0 {
                row["artifact_error"] = json!("arm64 boot image is absent or empty");
            }
        }
        map.insert(arch.clone(), entry);
    }
    Ok(Value::Object(map))
}

pub struct NativeHost;
impl Host for NativeHost {
    fn policy(&mut self, root: &Path) -> Result<Value> {
        Ok(serde_json::to_value(crate::build_policy::read(
            &crate::build_policy::policy_dir(root)?,
        )?)?)
    }
    fn broker(&mut self, root: &Path, _policy: &Value) -> Result<Value> {
        crate::build_policy::host_jobs_status(root)
    }
}
fn spec(name: impl Into<String>, argv: Vec<String>) -> ChildSpec {
    ChildSpec {
        name: name.into(),
        argv,
    }
}
fn validate_children(specs: &[ChildSpec], rows: &[Value]) -> Result<()> {
    let expected: BTreeSet<_> = specs.iter().map(|s| s.name.as_str()).collect();
    let actual: BTreeSet<_> = rows.iter().filter_map(|r| r["name"].as_str()).collect();
    if expected.len() != specs.len()
        || actual.len() != rows.len()
        || expected != actual
        || rows.iter().any(|r| r["exit_code"].as_i64().is_none())
    {
        return Err(Error::new(
            "child execution returned incomplete or invalid results",
        ));
    }
    Ok(())
}
fn children_ok(children: &[Value]) -> bool {
    children
        .iter()
        .all(|r| r["exit_code"] == 0 && r.get("artifact_error").is_none())
}
fn failure_text(e: &Error) -> String {
    format!("RuntimeError: {e}")
}
fn output_path(root: &Path, opt: &Options) -> Result<PathBuf> {
    let raw = if let Some(path) = &opt.output {
        let s = path.to_string_lossy();
        if s == "~" || s.starts_with("~/") {
            let home = std::env::var_os("HOME").ok_or("HOME is required for ~ expansion")?;
            PathBuf::from(home).join(s.trim_start_matches('~').trim_start_matches('/'))
        } else {
            path.clone()
        }
    } else {
        root.join(".cache/build-bench")
            .join(format!("{}-{}.json", timestamp(true), unique()))
    };
    crate::artifact_fs::resolve(&raw)
}
/// Run the full phase graph and keep the report durable through each transition.
pub fn run_with<H: Host>(root: &Path, opt: &Options, host: &mut H) -> Result<u8> {
    if opt.repeat == 0 {
        return Err(Error::new("--repeat must be at least 1"));
    }
    let mut arches = Vec::new();
    for arch in &opt.arches {
        crate::arch(arch)?;
        if !arches.contains(arch) {
            arches.push(arch.clone());
        }
    }
    if arches.is_empty() {
        return Err(Error::new("--arches requires at least one architecture"));
    }
    let before = host.git(root)?;
    if before["dirty"] != false {
        return Err(Error::new(
            "build-bench requires a clean committed tree for repeatable inputs",
        ));
    }
    let out = output_path(root, opt)?;
    fs::create_dir_all(out.parent().ok_or("output has no parent")?)?;
    match OpenOptions::new().create_new(true).write(true).open(&out) {
        Ok(f) => f.sync_all()?,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(Error::new(format!(
                "build-bench output already exists: {}",
                out.display()
            )))
        }
        Err(e) => return Err(e.into()),
    }
    let mut report = json!({"schema":1,"status":"INCOMPLETE","started_utc":timestamp(false),"repo":root.canonicalize()?,"git_before":before,"arches":arches,"clean_build_outputs":opt.clean,"cache_note":"--clean removes build outputs only; compiler caches are not cleared","repeat_requested":opt.repeat,"environment":environment(),"repeats":[],"build_policy":null,"broker_before":null});
    atomic_write(&out, &report)?;
    let init = (|| -> Result<()> {
        let policy = host.policy(root)?;
        let broker = host.broker(root, &policy)?;
        report["build_policy"] = policy;
        report["broker_before"] = broker;
        Ok(())
    })();
    if let Err(e) = init {
        report["status"] = json!("FAILED");
        report["exit_code"] = json!(1);
        report["error"] = json!(failure_text(&e));
        atomic_write(&out, &report)?;
        return Err(e);
    }
    atomic_write(&out, &report)?;
    let mut any_failed = false;
    for index in 1..=opt.repeat {
        let run_dir = out.parent().unwrap().join(format!(
            "{}-run{index:02}",
            out.file_stem()
                .ok_or("invalid output name")?
                .to_string_lossy()
        ));
        if let Err(e) = fs::create_dir(&run_dir) {
            report["status"] = json!("FAILED");
            report["error"] = json!(format!(
                "cannot create run directory {}: {e}",
                run_dir.display()
            ));
            report["exit_code"] = json!(1);
            atomic_write(&out, &report)?;
            return Ok(1);
        }
        let mut repeat = json!({"index":index,"status":"INCOMPLETE","children":[],"samples_jsonl":run_dir.join("samples.jsonl")});
        report["repeats"]
            .as_array_mut()
            .unwrap()
            .push(repeat.clone());
        atomic_write(&out, &report)?;
        let started = Instant::now();
        let mut children: Vec<Value> = Vec::new();
        let mut before_vm = BTreeMap::new();
        let mut after_vm = BTreeMap::new();
        let mut count = 0;
        let mut min = None;
        let mut sampler = None;
        let phase = (|| -> Result<(Value, bool)> {
            before_vm = read_vmstat(Path::new("/proc/vmstat"))?;
            let pids = Arc::new(Mutex::new(BTreeSet::new()));
            sampler = Some(Sampler::start(run_dir.join("samples.jsonl"), pids.clone())?);
            let cli = root.join("vibeos-cli").to_string_lossy().into_owned();
            let mut first = Vec::new();
            let defer = arches.contains(&"amd64".into()) && arches.contains(&"i386".into());
            for arch in &arches {
                let mut cmd = vec![
                    cli.clone(),
                    "build".into(),
                    "--arch".into(),
                    arch.clone(),
                    "--no-check".into(),
                ];
                if opt.clean {
                    cmd.push("--clean".into());
                }
                if defer && arch == "amd64" {
                    cmd.push("--no-wow64-stage".into());
                }
                first.push(spec(format!("build-{arch}"), cmd));
            }
            first.push(spec(
                "check-source-early",
                vec![
                    cli.clone(),
                    "check".into(),
                    "--strict".into(),
                    "--scope".into(),
                    "source-early".into(),
                ],
            ));
            let first_rows = host.children(root, &first, &run_dir, pids.clone())?;
            validate_children(&first, &first_rows)?;
            children.extend(first_rows.iter().cloned());
            let mut builds: Vec<Value> = first_rows
                .into_iter()
                .filter(|r| r["name"].as_str().is_some_and(|n| n.starts_with("build-")))
                .collect();
            let mut artifacts = check_artifacts(root, &arches, &mut builds)?;
            for updated in &builds {
                if let Some(row) = children.iter_mut().find(|r| r["name"] == updated["name"]) {
                    *row = updated.clone();
                }
            }
            let mut builds_ok = builds.len() == arches.len() && children_ok(&builds);
            if builds_ok && defer {
                let restage = host.children(
                    root,
                    &[spec(
                        "build-amd64-restage",
                        vec![
                            cli.clone(),
                            "build".into(),
                            "--arch".into(),
                            "amd64".into(),
                            "--no-check".into(),
                        ],
                    )],
                    &run_dir,
                    pids.clone(),
                )?;
                if restage.len() != 1
                    || restage[0]["name"] != "build-amd64-restage"
                    || restage[0]["exit_code"].as_i64().is_none()
                {
                    return Err(Error::new("restage returned incomplete or invalid results"));
                }
                builds_ok = children_ok(&restage);
                children.extend(restage);
                artifacts = check_artifacts(root, &arches, &mut builds)?;
                builds_ok &= children_ok(&builds);
                for updated in &builds {
                    if let Some(row) = children.iter_mut().find(|r| r["name"] == updated["name"]) {
                        *row = updated.clone();
                    }
                }
            }
            let mut follow = vec![spec(
                "check-source-deferred",
                vec![
                    cli.clone(),
                    "check".into(),
                    "--strict".into(),
                    "--scope".into(),
                    "source-deferred".into(),
                ],
            )];
            for arch in &arches {
                follow.push(spec(
                    format!("check-arch-{arch}"),
                    vec![
                        cli.clone(),
                        "check".into(),
                        "--strict".into(),
                        "--scope".into(),
                        "arch".into(),
                        "--arch".into(),
                        arch.clone(),
                    ],
                ));
            }
            if builds_ok {
                let follow_rows = host.children(root, &follow, &run_dir, pids)?;
                validate_children(&follow, &follow_rows)?;
                children.extend(follow_rows);
            } else {
                children.extend(follow.into_iter().map(|s|json!({"name":s.name,"command":s.argv,"log":null,"exit_code":null,"status":"SKIPPED","reason":"one or more builds failed"})));
            }
            Ok((artifacts, builds_ok))
        })();
        let sampler_result = match sampler.take() {
            Some(s) => s.stop(),
            None => Ok((0, None)),
        };
        if let Ok((n, m)) = &sampler_result {
            count = *n;
            min = *m;
        }
        let result = (|| -> Result<(Value, bool)> {
            let (artifacts, builds_ok) = phase?;
            sampler_result?;
            after_vm = read_vmstat(Path::new("/proc/vmstat"))?;
            if before_vm.keys().ne(after_vm.keys()) {
                return Err(Error::new("vmstat counter set changed during benchmark"));
            }
            if before_vm.iter().any(|(key, value)| after_vm[key] < *value) {
                return Err(Error::new("vmstat counter decreased during benchmark"));
            }
            let end = host.git(root)?;
            report["git_after"] = end.clone();
            if end != before {
                return Err(Error::new(format!(
                    "git state changed during benchmark: before={before}, after={end}"
                )));
            }
            Ok((artifacts, builds_ok))
        })();
        children.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        match result {
            Ok((artifacts, builds_ok)) => {
                let mut failures = BTreeSet::new();
                for row in &children {
                    if (!row["exit_code"].is_null() && row["exit_code"] != 0)
                        || row.get("artifact_error").is_some()
                    {
                        if let Some(name) = row["name"].as_str() {
                            failures.insert(name.to_owned());
                        }
                    }
                }
                if !builds_ok {
                    failures.insert("check-source-deferred".into());
                    for arch in &arches {
                        failures.insert(format!("check-arch-{arch}"));
                    }
                }
                let keys: BTreeSet<_> = before_vm
                    .keys()
                    .chain(after_vm.keys())
                    .filter(|k| k.starts_with("oom"))
                    .cloned()
                    .collect();
                let delta: BTreeMap<_, _> = keys
                    .into_iter()
                    .map(|k| {
                        let n = (*after_vm.get(&k).unwrap_or(&0) as i128)
                            - (*before_vm.get(&k).unwrap_or(&0) as i128);
                        (k, n)
                    })
                    .collect();
                repeat = json!({"index":index,"status":if failures.is_empty(){"OK"}else{"FAILED"},"children":children,"samples_jsonl":run_dir.join("samples.jsonl"),"wall_seconds":started.elapsed().as_secs_f64(),"artifacts":artifacts,"sample_count":count,"min_mem_available_bytes":min,"vmstat_before":before_vm,"vmstat_after":after_vm,"oom_delta":delta,"failed_children":failures});
            }
            Err(e) => {
                repeat["status"] = json!("FAILED");
                repeat["error"] = json!(failure_text(&e));
                repeat["children"] = json!(children);
                repeat["sample_count"] = json!(count);
                repeat["vmstat_before"] = json!(before_vm);
                repeat["vmstat_after"] = json!(after_vm);
            }
        }
        if repeat["status"] != "OK" {
            any_failed = true;
        }
        let entries = report["repeats"].as_array_mut().unwrap();
        entries[index - 1] = repeat.clone();
        report["status"] = json!("INCOMPLETE");
        report["updated_utc"] = json!(timestamp(false));
        atomic_write(&out, &report)?;
        println!(
            "build-bench repeat {index}/{}: {}; summary {}",
            opt.repeat,
            repeat["status"].as_str().unwrap_or("FAILED"),
            out.display()
        );
    }
    report["status"] = json!(if any_failed { "FAILED" } else { "OK" });
    report["exit_code"] = json!(u8::from(any_failed));
    match host.git(root) {
        Ok(state) => {
            report["git_after"] = state.clone();
            if state != before {
                report["status"] = json!("FAILED");
                report["exit_code"] = json!(1);
            }
        }
        Err(e) => {
            report["status"] = json!("FAILED");
            report["exit_code"] = json!(1);
            report["git_after_error"] = json!(failure_text(&e));
        }
    }
    match host.policy(root) {
        Ok(policy) => {
            report["build_policy_after"] = policy.clone();
            if policy != report["build_policy"] {
                report["status"] = json!("FAILED");
                report["exit_code"] = json!(1);
                report["policy_error"] = json!("build capacity changed during measurement");
            }
        }
        Err(error) => {
            report["status"] = json!("FAILED");
            report["exit_code"] = json!(1);
            report["policy_error"] = json!(failure_text(&error));
        }
    }
    match host.broker(root, &report["build_policy_after"]) {
        Ok(broker) => report["broker_after"] = broker,
        Err(error) => {
            report["status"] = json!("FAILED");
            report["exit_code"] = json!(1);
            report["broker_after_error"] = json!(failure_text(&error));
        }
    }
    report["updated_utc"] = json!(timestamp(false));
    atomic_write(&out, &report)?;
    Ok(report["exit_code"].as_u64().unwrap_or(1) as u8)
}

/// Native API for compatibility imports; required inputs fail loudly.
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let op = request["op"].as_str().ok_or("missing op")?;
    let kw = &request["kwargs"];
    let path = |key: &str| -> Result<PathBuf> {
        kw[key]
            .as_str()
            .map(PathBuf::from)
            .ok_or_else(|| Error::new(format!("missing {key}")))
    };
    match op {
        "build_bench_meminfo" => Ok(json!(read_meminfo(&path("path")?)?)),
        "build_bench_vmstat" => Ok(json!(read_vmstat(&path("path")?)?)),
        "build_bench_pressure" => read_pressure(&path("path")?),
        "build_bench_git" => git_state(root),
        "build_bench_run_child" => {
            let name = kw["name"].as_str().ok_or("missing name")?.to_owned();
            let argv: Vec<String> =
                serde_json::from_value(kw.get("command").cloned().ok_or("missing command")?)?;
            let log = path("log")?;
            Ok(run_child(
                root,
                &ChildSpec { name, argv },
                &log,
                Arc::new(Mutex::new(BTreeSet::new())),
            ))
        }
        "build_bench_media_path" => Ok(json!(media_path(
            root,
            kw["arch"].as_str().ok_or("missing arch")?
        )?)),
        _ => Err(Error::new(format!("unknown build-bench operation {op}"))),
    }
}
pub fn cli(root: &Path, raw: &[String]) -> Result<u8> {
    let mut normalized = Vec::new();
    for arg in raw {
        if let Some((key, value)) = arg.split_once('=') {
            if ["--arches", "--repeat", "--output"].contains(&key) {
                normalized.push(key.to_owned());
                normalized.push(value.to_owned());
                continue;
            }
        }
        normalized.push(arg.clone());
    }
    let args = &normalized;
    let mut arches = Vec::new();
    let mut clean = false;
    let mut repeat = 1usize;
    let mut output = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--arches" => {
                i += 1;
                let first = i;
                while i < args.len() && !args[i].starts_with('-') {
                    arches.push(args[i].clone());
                    i += 1;
                }
                if i == first {
                    return Err(Error::new("--arches requires values"));
                }
                continue;
            }
            "--clean" => clean = true,
            "--repeat" => {
                i += 1;
                repeat = args
                    .get(i)
                    .ok_or("--repeat requires value")?
                    .parse()
                    .map_err(|_| Error::new("invalid --repeat"))?;
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(args.get(i).ok_or("--output requires value")?));
            }
            "--help" | "-h" => {
                println!(
                    "usage: build-bench --arches ARCH... [--clean] [--repeat N] [--output FILE]"
                );
                return Ok(0);
            }
            s => return Err(Error::new(format!("unknown build-bench option {s}"))),
        }
        i += 1;
    }
    run_with(
        root,
        &Options {
            arches,
            clean,
            repeat,
            output,
        },
        &mut NativeHost,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proc_parsers_and_unavailable_psi() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path();
        fs::write(p.join("meminfo"), "MemTotal: 100 kB\nHugePages_Total: 2\n").unwrap();
        assert_eq!(
            read_meminfo(&p.join("meminfo")).unwrap(),
            BTreeMap::from([("MemTotal".into(), 102400), ("HugePages_Total".into(), 2)])
        );
        fs::write(p.join("vmstat"), "oom_kill 4\npgmajfault 9\npgfault 100\n").unwrap();
        assert_eq!(
            read_vmstat(&p.join("vmstat")).unwrap(),
            BTreeMap::from([("oom_kill".into(), 4), ("pgmajfault".into(), 9)])
        );
        let psi = read_pressure(p).unwrap();
        for name in ["memory", "cpu", "io"] {
            assert_eq!(
                psi[name],
                json!({"available":false,"reason":"kernel PSI file absent"})
            );
        }
        fs::write(p.join("meminfo"), "invalid\n").unwrap();
        assert!(read_meminfo(&p.join("meminfo")).is_err());
    }
    #[test]
    fn missing_media_and_arm_boot_are_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut rows = vec![
            json!({"name":"build-i386","exit_code":0}),
            json!({"name":"build-arm64","exit_code":0}),
        ];
        let artifacts = check_artifacts(root, &["i386".into(), "arm64".into()], &mut rows).unwrap();
        assert_eq!(artifacts["i386"]["exists"], false);
        assert!(rows.iter().all(|r| r.get("artifact_error").is_some()));
        assert_eq!(
            media_path(root, "arm64").unwrap(),
            root.join("qemu/ramdisk.img")
        );
    }
    #[derive(Default)]
    struct FakeHost {
        phases: Vec<Vec<ChildSpec>>,
        fail: bool,
        changed: bool,
        git_calls: usize,
        policy_calls: usize,
        change_policy: bool,
    }
    impl Host for FakeHost {
        fn git(&mut self, _: &Path) -> Result<Value> {
            self.git_calls += 1;
            Ok(json!({"head":if self.changed&&self.git_calls>1{"def"}else{"abc"},"dirty":false}))
        }
        fn policy(&mut self, _: &Path) -> Result<Value> {
            self.policy_calls += 1;
            Ok(if self.change_policy && self.policy_calls > 1 {
                json!({"changed":true})
            } else {
                json!({})
            })
        }
        fn broker(&mut self, _: &Path, _: &Value) -> Result<Value> {
            Ok(json!({}))
        }
        fn children(
            &mut self,
            root: &Path,
            specs: &[ChildSpec],
            _: &Path,
            _: Arc<Mutex<BTreeSet<u32>>>,
        ) -> Result<Vec<Value>> {
            self.phases.push(specs.to_vec());
            for s in specs {
                if let Some(arch) = s.name.strip_prefix("build-") {
                    if arch != "amd64-restage" {
                        let file = media_path(root, arch)?;
                        fs::create_dir_all(file.parent().unwrap())?;
                        fs::write(file, b"media")?;
                    } else {
                        fs::write(media_path(root, "amd64")?, b"restaged media")?;
                    }
                }
            }
            Ok(specs.iter().map(|s|json!({"name":s.name,"command":s.argv,"exit_code":if self.fail&&s.name=="build-i386"{1}else{0}})).collect())
        }
    }
    #[test]
    fn phases_restage_and_git_changes_fail_loudly() {
        for changed in [false, true] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            let out = root.join("report.json");
            let mut host = FakeHost {
                changed,
                ..Default::default()
            };
            let opts = Options {
                arches: vec!["amd64".into(), "i386".into()],
                clean: true,
                repeat: 1,
                output: Some(out.clone()),
            };
            assert_eq!(run_with(root, &opts, &mut host).unwrap(), u8::from(changed));
            assert_eq!(host.phases.len(), 3);
            assert!(host.phases[0][0].argv.contains(&"--no-wow64-stage".into()));
            assert_eq!(host.phases[1][0].name, "build-amd64-restage");
            assert!(!host.phases[1][0].argv.contains(&"--clean".into()));
            let report: Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
            if changed {
                assert_eq!(report["status"], "FAILED");
                assert!(report["repeats"][0]["error"]
                    .as_str()
                    .unwrap()
                    .contains("git state changed"));
            } else {
                assert_eq!(report["status"], "OK");
                assert_eq!(report["repeats"][0]["artifacts"]["amd64"]["size_bytes"], 14);
                let samples = fs::read_to_string(root.join("report-run01/samples.jsonl")).unwrap();
                let row: Value = serde_json::from_str(samples.lines().next().unwrap()).unwrap();
                assert!(row["meminfo_bytes"]["MemAvailable"].as_u64().is_some());
                assert!(row["process_tree"]["available"].as_bool().is_some());
            }
            assert!(run_with(root, &opts, &mut host).is_err());
        }
    }
    #[test]
    fn failed_build_skips_followups_and_persists_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let out = root.join("report.json");
        let mut host = FakeHost {
            fail: true,
            ..Default::default()
        };
        let opts = Options {
            arches: vec!["i386".into()],
            clean: false,
            repeat: 1,
            output: Some(out.clone()),
        };
        assert_eq!(run_with(root, &opts, &mut host).unwrap(), 1);
        assert_eq!(host.phases.len(), 1);
        let report: Value = serde_json::from_slice(&fs::read(out).unwrap()).unwrap();
        assert_eq!(report["status"], "FAILED");
        assert!(report["repeats"][0]["children"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["name"] == "check-source-deferred" && r["status"] == "SKIPPED"));
    }
    #[test]
    fn changed_policy_cannot_certify_a_comparable_trial() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("report.json");
        let mut host = FakeHost {
            change_policy: true,
            ..Default::default()
        };
        let opt = Options {
            arches: vec!["i386".into()],
            clean: false,
            repeat: 1,
            output: Some(out.clone()),
        };
        assert_eq!(run_with(tmp.path(), &opt, &mut host).unwrap(), 1);
        let report: Value = serde_json::from_slice(&fs::read(out).unwrap()).unwrap();
        assert_eq!(
            report["policy_error"],
            "build capacity changed during measurement"
        );
    }
    #[test]
    fn sampler_propagates_creation_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("samples");
        fs::write(&path, b"occupied").unwrap();
        let result = Sampler::start(path, Arc::new(Mutex::new(BTreeSet::new())));
        assert!(result.is_err());
    }
}
