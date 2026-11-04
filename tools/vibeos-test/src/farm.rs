//! Native soak ownership, durable evidence and corpus aggregation.
//! VM algorithms execute in the native smoke worker; reporting stays a Host adapter.
use crate::{callback::Host, Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::{
        fs::PermissionsExt,
        process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn flag(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}
fn integer(v: &Value, key: &str, default: i64) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(default)
}
fn say(host: &mut dyn Host, text: impl AsRef<str>) -> Result<()> {
    host.call("print", json!({"text":text.as_ref(),"flush":true}))?;
    Ok(())
}
fn text(path: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&fs::read(path)?).into_owned())
}
fn base_arch(arch: &str) -> String {
    arch.replace("-net", "")
        .replace("-uefi", "")
        .replace("-bios", "")
        .replace("-test", "")
}
fn env_map(a: &Value) -> Result<BTreeMap<String, String>> {
    a.get("env")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()
        .map(|v| v.unwrap_or_else(|| std::env::vars().collect()))
        .map_err(Into::into)
}
fn farm_dir(a: &Value, env: &BTreeMap<String, String>) -> PathBuf {
    a.get("farm_dir")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env.get("HOME").cloned().unwrap_or_default()).join("vibeos-flake-farm")
        })
}
pub(crate) fn now_tag() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
        return t.to_string();
    }
    let mut buf = [0u8; 64];
    let f = b"%Y%m%d-%H%M%S\0";
    let n = unsafe { libc::strftime(buf.as_mut_ptr().cast(), buf.len(), f.as_ptr().cast(), &tm) };
    String::from_utf8_lossy(&buf[..n]).into_owned()
}
fn provenance(root: &Path, op: &str, args: Value) -> Result<Value> {
    vibeos_build::provenance::dispatch(
        root,
        &json!({"op":format!("provenance_{op}"),"kwargs":args}),
    )
}
pub(crate) fn git_state(repo: &Path, env: &BTreeMap<String, String>) -> Result<(String, bool)> {
    let run = |cmd: Vec<&str>| -> Result<Vec<u8>> {
        serde_json::from_value(provenance(
            repo,
            "run_bytes",
            json!({"cmd":cmd,"cwd":repo,"ok_codes":[0],"env":env}),
        )?)
        .map_err(Into::into)
    };
    let sha = run(vec!["git", "rev-parse", "--short", "HEAD"])?;
    let status = run(vec!["git", "status", "--porcelain"])?;
    Ok((
        String::from_utf8_lossy(&sha).trim().to_owned(),
        !status.is_empty(),
    ))
}

pub fn extract_evidence(run_dir: &Path, salient: &[String]) -> Value {
    let raw = text(&run_dir.join("full.log")).unwrap_or_default();
    let hay = format!("{raw}\n{}", salient.join("\n"));
    let stop = Regex::new(r"BUGCHECK: 0x([0-9A-Fa-f]{8})")
        .expect("static stop regex")
        .captures(&hay)
        .map(|c| format!("0x{}", c[1].to_ascii_uppercase()));
    let mut symbols = Vec::<String>::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("***") && line.contains('=') && !symbols.iter().any(|x| x == line) {
            symbols.push(line.to_owned());
        }
        if symbols.len() >= 12 {
            break;
        }
    }
    let top = symbols
        .iter()
        .find(|s| s.contains("TrapFrame.Rip"))
        .or_else(|| {
            symbols
                .iter()
                .find(|s| s.contains(".exe+") || s.contains(".sys+"))
        })
        .and_then(|s| s.rsplit('=').next())
        .map(str::trim)
        .map(str::to_owned);
    json!({"stop_code":stop,"symbols":symbols,"top_symbol":top})
}
fn safe_tag(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == '\0' {
                '_'
            } else {
                c
            }
        })
        .collect()
}
pub fn preserve_failure(farm: &Path, run_dir: &Path, record: &Value) -> Result<PathBuf> {
    let tag = if s(record, "classification").is_empty() {
        s(record, "outcome")
    } else {
        s(record, "classification")
    };
    let dst = farm.join("fails").join(format!(
        "{}-{}-{}",
        s(record, "ts"),
        safe_tag(s(record, "arch")),
        safe_tag(tag)
    ));
    fs::create_dir_all(&dst)?;
    for name in ["full.log", "summary.json"] {
        let src = run_dir.join(name);
        if src.is_file() {
            fs::copy(src, dst.join(name))?;
        }
    }
    if run_dir.is_dir() {
        for entry in fs::read_dir(run_dir)? {
            let src = entry?.path();
            let name = src.file_name().unwrap_or_default().to_string_lossy();
            if src.is_file() && name.contains("serial") && name.ends_with(".log") {
                fs::copy(&src, dst.join(name.as_ref()))?;
            }
        }
    }
    if let Some(shot) = record["screenshot"].as_str() {
        let src = Path::new(shot);
        if src.is_file() {
            fs::copy(
                src,
                dst.join(src.file_name().ok_or("screenshot lacks filename")?),
            )?;
        }
    }
    if let Some(worker_log) = record["worker_log"].as_str() {
        let src = Path::new(worker_log);
        if src.is_file() {
            fs::copy(
                src,
                dst.join(src.file_name().ok_or("worker log lacks filename")?),
            )?;
        }
    }
    Ok(dst)
}
pub fn append_record(path: &Path, record: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;
    use std::os::fd::AsRawFd;
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut bytes = serde_json::to_vec(record)?;
    bytes.push(b'\n');
    file.write_all(&bytes)?;
    file.sync_data()?;
    Ok(())
}
struct Worker {
    child: Child,
    done: bool,
}
impl Worker {
    fn spawn(
        command: &[String],
        repo: &Path,
        env: &BTreeMap<String, String>,
        log: &Path,
    ) -> Result<Self> {
        let (program, args) = command.split_first().ok_or("empty farm worker command")?;
        let file = File::create(log)?;
        let mut cmd = Command::new(program);
        cmd.args(args)
            .current_dir(repo)
            .env_clear()
            .envs(env)
            .stdin(Stdio::null())
            .stdout(file.try_clone()?)
            .stderr(file);
        let parent = std::process::id();
        unsafe {
            cmd.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::getppid() != parent as i32 {
                    libc::_exit(125);
                }
                Ok(())
            });
        }
        Ok(Self {
            child: cmd.spawn()?,
            done: false,
        })
    }
    fn wait(&mut self, cancel: &AtomicBool) -> Result<i32> {
        loop {
            if let Some(status) = self.child.try_wait()? {
                self.done = true;
                return Ok(status
                    .code()
                    .unwrap_or_else(|| -status.signal().unwrap_or(1)));
            }
            if cancel.load(Ordering::SeqCst) {
                self.child.kill()?;
                let _ = self.child.wait();
                self.done = true;
                return Err(Error::classified("KeyboardInterrupt", "farm cancelled"));
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
struct Boot {
    arch: String,
    index: u64,
    result: Value,
    run_dir: PathBuf,
    duration: f64,
    log: PathBuf,
}
enum Event {
    Quiet {
        reply: mpsc::SyncSender<Result<Option<String>>>,
    },
    Boot(Boot),
    Failed(String),
    Notice(String),
    Finished,
}
struct FarmConfig {
    repo: PathBuf,
    env: BTreeMap<String, String>,
    worker_prefix: Vec<String>,
    work: PathBuf,
    deadline: Instant,
    max_boots: u64,
    polite: bool,
    cancel: Arc<AtomicBool>,
    owners: Arc<Mutex<BTreeSet<u32>>>,
}
fn arch_worker(cfg: Arc<FarmConfig>, arch: String, events: mpsc::SyncSender<Event>) {
    let result = (|| -> Result<()> {
        let mut index = 0;
        let mut announced = String::new();
        while Instant::now() < cfg.deadline
            && index < cfg.max_boots
            && !cfg.cancel.load(Ordering::SeqCst)
        {
            if cfg.polite {
                let (tx, rx) = mpsc::sync_channel(1);
                events
                    .send(Event::Quiet { reply: tx })
                    .map_err(|_| Error::new("farm reporting owner closed"))?;
                if let Some(why) = rx
                    .recv()
                    .map_err(|_| Error::new("farm quiet query owner closed"))??
                {
                    if why != announced {
                        events
                            .send(Event::Notice(format!(
                                "[farm {arch}] polite: waiting, {why}"
                            )))
                            .map_err(|_| Error::new("farm reporting owner closed"))?;
                        announced = why;
                    }
                    let end = Instant::now() + Duration::from_secs(30);
                    while Instant::now() < end
                        && Instant::now() < cfg.deadline
                        && !cfg.cancel.load(Ordering::SeqCst)
                    {
                        thread::sleep(Duration::from_millis(100));
                    }
                    continue;
                }
            }
            index += 1;
            let stem = format!("{arch}-{index}");
            let request = cfg.work.join(format!("{stem}.request.json"));
            let output = cfg.work.join(format!("{stem}.result.json"));
            let log = cfg.work.join(format!("{stem}.worker.log"));
            fs::write(
                &request,
                serde_json::to_vec(
                    &json!({"op":"smoke_one_boot","repo":cfg.repo,"command":"farm","arch":arch,"owner_pid":std::process::id(),"kwargs":{"until":"","fail_on":null,"idx":0,"total":1}}),
                )?,
            )?;
            let mut command = cfg.worker_prefix.clone();
            command.extend([
                "--request".into(),
                request.to_string_lossy().into_owned(),
                "--result".into(),
                output.to_string_lossy().into_owned(),
            ]);
            let start = Instant::now();
            let mut worker = Worker::spawn(&command, &cfg.repo, &cfg.env, &log)?;
            let pid = worker.child.id();
            cfg.owners
                .lock()
                .map_err(|_| Error::new("farm owner index poisoned"))?
                .insert(pid);
            let status = worker.wait(&cfg.cancel);
            cfg.owners
                .lock()
                .map_err(|_| Error::new("farm owner index poisoned"))?
                .remove(&pid);
            let status = status?;
            let value = fs::read(&output)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
            let (mut result, run_dir) = match value {
                Some(v) if v.get("result").is_some_and(Value::is_object) => (
                    v["result"].clone(),
                    v["run_dir"]
                        .as_str()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| cfg.work.clone()),
                ),
                _ => (
                    json!({"ok":false,"outcome":"farm_error","classification":"FARM_ERROR","salient":[format!("worker exited {status} without a valid result; see {}",log.display())]}),
                    cfg.work.clone(),
                ),
            };
            if status != 0 && flag(&result, "ok") {
                result["ok"] = json!(false);
                result["outcome"] = json!("farm_error");
                result["classification"] = json!("FARM_ERROR");
                result["salient"] = json!([format!(
                    "reporting worker exited {status} after a claimed success"
                )]);
            }
            events
                .send(Event::Boot(Boot {
                    arch: arch.clone(),
                    index,
                    result,
                    run_dir,
                    duration: start.elapsed().as_secs_f64(),
                    log,
                }))
                .map_err(|_| Error::new("farm reporting owner closed"))?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        let _ = events.send(Event::Failed(format!("[farm {arch}] worker failed: {e}")));
    }
    let _ = events.send(Event::Finished);
}
fn sample_record(boot: &Boot, sha: &str, dirty: bool, env: &BTreeMap<String, String>) -> Value {
    let r = &boot.result;
    let limited = |key: &str, n: usize| {
        r.get(key)
            .and_then(Value::as_array)
            .map(|v| v.iter().take(n).cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    };
    let mut rec = json!({"ts":now_tag(),"arch":boot.arch,"sha":sha,"dirty":dirty,"tripwire":env.get("VIBEOS_TRIPWIRE").cloned().unwrap_or_default(),"smp":env.get("VIBEOS_QEMU_SMP").and_then(|s|s.parse::<u64>().ok()).unwrap_or(0),"ahci":env.get("VIBEOS_AHCI_CD").is_some_and(|s|s=="1"),"ok":flag(r,"ok"),"outcome":s(r,"outcome"),"classification":s(r,"classification"),"hang_kind":r.get("hang_kind"),"census_stuck":r.get("census_stuck"),"lockdep_hits":r.get("lockdep_hits"),"cswitch_hits":r.get("cswitch_hits"),"census_lines":limited("census_lines",6),"salient":limited("salient",8),"screenshot":r.get("screenshot"),"run_dir":boot.run_dir,"duration_s":(boot.duration*10.0).round()/10.0,"timings":r.get("timings").cloned().unwrap_or_else(||json!({})),"worker_log":boot.log});
    let salient: Vec<String> = rec["salient"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    for (k, v) in extract_evidence(&boot.run_dir, &salient)
        .as_object()
        .unwrap()
    {
        rec[k] = v.clone();
    }
    rec
}
pub fn run(root: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = a.get("args").unwrap_or(a);
    let mut env = env_map(a)?;
    let dir = farm_dir(a, &env);
    let corpus = a
        .get("corpus")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| dir.join("corpus.jsonl"));
    let arches: Vec<String> = s(args, "arches")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(base_arch)
        .collect();
    let arches = if arches.is_empty() {
        vec!["amd64".into()]
    } else {
        arches
    };
    let minutes = args.get("minutes").and_then(Value::as_f64).unwrap_or(0.0);
    if !minutes.is_finite() || minutes < 0.0 {
        return Err("farm minutes must be finite and nonnegative".into());
    }
    let boots = integer(args, "boots", 0);
    if boots < 0 {
        return Err("farm boots must be nonnegative".into());
    }
    let max_boots = if boots > 0 {
        boots as u64
    } else if minutes > 0.0 {
        1_000_000_000
    } else {
        5
    };
    let seconds = if minutes > 0.0 { minutes * 60.0 } else { 1e9 };
    if !seconds.is_finite() || seconds > u64::MAX as f64 {
        return Err("farm duration is not representable".into());
    }
    let deadline = Instant::now()
        .checked_add(Duration::from_secs_f64(seconds))
        .ok_or("farm deadline is not representable")?;
    let tripwire = s(args, "tripwire").trim();
    if !tripwire.is_empty() {
        env.insert("VIBEOS_TRIPWIRE".into(), tripwire.into());
    }
    let smp = integer(args, "smp", 0);
    if smp < 0 {
        return Err("farm SMP must be nonnegative".into());
    }
    if smp > 0 {
        env.insert("VIBEOS_QEMU_SMP".into(), smp.to_string());
    }
    if flag(args, "ahci") {
        env.insert("VIBEOS_AHCI_CD".into(), "1".into());
    }
    for arch in &arches {
        let cfg = a
            .get("cfgs")
            .and_then(|m| m.get(arch))
            .ok_or_else(|| Error::new(format!("missing architecture config for {arch}")))?;
        let st = provenance(
            root,
            "check_staleness",
            json!({"repo":root,"arch":arch,"cfg":cfg,"env":env}),
        )?;
        if (!flag(&st, "fresh") || flag(&st, "media_missing")) && !flag(args, "allow_stale") {
            say(
                host,
                format!("[farm] {arch}: STALE media — build first (or --allow-stale)"),
            )?;
            return Ok(json!(1));
        }
    }
    let require: Vec<String> = args
        .get("require_probe")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .collect();
    if !require.is_empty() {
        for arch in &arches {
            let missing = provenance(
                root,
                "probes_missing_from_media",
                json!({"repo":root,"arch":arch,"cfg":a["cfgs"][arch],"literals":require,"env":env}),
            )?;
            let missing = missing.as_array().ok_or("invalid probe response")?;
            if !missing.is_empty() {
                say(
                    host,
                    format!("[farm] {arch}: PROBE NOT IN MEDIA — refusing to spend the boots"),
                )?;
                for line in missing {
                    say(
                        host,
                        format!(
                            "[farm]   {}",
                            line.as_str().unwrap_or("invalid probe error")
                        ),
                    )?;
                }
                return Ok(json!(1));
            }
        }
        say(
            host,
            format!(
                "[farm] probe-check OK: {} literal(s) in every boot image",
                require.len()
            ),
        )?;
    }
    let (sha, dirty) = git_state(root, &env)?;
    say(
        host,
        format!(
            "[farm] arches={} sha={}{}{}{}{}{} budget={} -> {}",
            arches.join(","),
            sha,
            if dirty { " DIRTY" } else { "" },
            if !tripwire.is_empty() {
                format!(" tripwire={tripwire:?}")
            } else {
                String::new()
            },
            if smp > 0 {
                format!(" -smp {smp}")
            } else {
                String::new()
            },
            if flag(args, "ahci") { " ahci" } else { "" },
            if flag(args, "polite") { " polite" } else { "" },
            if minutes > 0.0 {
                format!("{minutes:.0}min")
            } else {
                format!("{max_boots} boots/arch")
            },
            corpus.display()
        ),
    )?;
    fs::create_dir_all(dir.join("workers"))?;
    let work = tempfile::Builder::new()
        .prefix(&format!("{}-", std::process::id()))
        .tempdir_in(dir.join("workers"))?;
    fs::set_permissions(work.path(), fs::Permissions::from_mode(0o700))?;
    let prefix: Vec<String> = a
        .get("worker_command")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?
        .unwrap_or_else(|| {
            vec![
                a["python"].as_str().unwrap_or("python3").into(),
                "-m".into(),
                "tests.qemu.native_worker".into(),
            ]
        });
    let cancel = Arc::new(AtomicBool::new(false));
    let owners = Arc::new(Mutex::new(BTreeSet::from([std::process::id()])));
    let cfg = Arc::new(FarmConfig {
        repo: root.to_owned(),
        env: env.clone(),
        worker_prefix: prefix,
        work: work.path().to_owned(),
        deadline,
        max_boots,
        polite: flag(args, "polite"),
        cancel: cancel.clone(),
        owners: owners.clone(),
    });
    let (tx, rx) = mpsc::sync_channel(16);
    let mut workers = Vec::new();
    for arch in &arches {
        let (cfg, tx, arch) = (cfg.clone(), tx.clone(), arch.clone());
        workers.push(thread::spawn(move || arch_worker(cfg, arch, tx)));
    }
    drop(tx);
    let (mut done, mut total, mut fails) = (0, 0u64, 0u64);
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    let mut fatal = None;
    while done < workers.len() {
        let event = rx
            .recv()
            .map_err(|_| Error::new("farm workers closed without completion"))?;
        let result = (|| -> Result<()> {
            match event {
                Event::Finished => done += 1,
                Event::Notice(message) => say(host, &message)?,
                Event::Failed(message) => {
                    say(host, &message)?;
                    fatal = Some(message);
                    cancel.store(true, Ordering::SeqCst);
                }
                Event::Quiet { reply } => {
                    let owner_pids: Vec<u32> = owners
                        .lock()
                        .map_err(|_| Error::new("farm owner index poisoned"))?
                        .iter()
                        .copied()
                        .collect();
                    let response = host.call(
                        "host_quiet",
                        json!({"owner_pid":std::process::id(),"owner_pids":owner_pids}),
                    );
                    let why = response.and_then(|v| {
                        Ok(quiet_reason(
                            &v,
                            &owner_pids.iter().copied().collect(),
                            std::process::id(),
                        ))
                    });
                    let _ = reply.send(why);
                }
                Event::Boot(boot) => {
                    let mut rec = sample_record(&boot, &sha, dirty, &env);
                    if !flag(&rec, "ok") {
                        match preserve_failure(&dir, &boot.run_dir, &rec) {
                            Ok(path) => rec["preserved"] = json!(path),
                            Err(e) => {
                                rec["preserved"] = Value::Null;
                                rec["preservation_error"] = json!(e.to_string());
                                fatal = Some(format!("cannot preserve failed boot evidence: {e}"));
                                cancel.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                    append_record(&corpus, &rec)?;
                    total += 1;
                    if !flag(&rec, "ok") {
                        fails += 1;
                    }
                    let state = if flag(&rec, "ok") {
                        "ok"
                    } else if !s(&rec, "classification").is_empty() {
                        s(&rec, "classification")
                    } else {
                        s(&rec, "outcome")
                    };
                    *by.entry(state.into()).or_default() += 1;
                    say(
                        host,
                        format!(
                            "[farm {}] boot {}: {} ({:.0}s){}{}",
                            boot.arch,
                            boot.index,
                            state,
                            boot.duration,
                            rec["stop_code"]
                                .as_str()
                                .map(|v| format!(" stop={v}"))
                                .unwrap_or_default(),
                            rec["top_symbol"]
                                .as_str()
                                .map(|v| format!(" @ {v}"))
                                .unwrap_or_default()
                        ),
                    )?;
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            fatal = Some(e.to_string());
            cancel.store(true, Ordering::SeqCst);
        }
    }
    for worker in workers {
        if worker.join().is_err() {
            fatal = Some("farm worker panicked".into());
        }
    }
    say(
        host,
        format!(
            "[farm] done: {total} boots, {fails} failures — {}",
            serde_json::to_string(&by)?
        ),
    )?;
    say(host, "[farm] report: ./vibeos-cli farm-report")?;
    if let Some(error) = fatal {
        return Err(Error::new(error));
    }
    Ok(json!(0))
}
pub fn report(a: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = a.get("args").unwrap_or(a);
    let env = env_map(a)?;
    let corpus = a
        .get("corpus")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| farm_dir(a, &env).join("corpus.jsonl"));
    if !corpus.is_file() {
        say(
            host,
            format!(
                "no corpus at {} — run `vibeos-cli farm` first",
                corpus.display()
            ),
        )?;
        return Ok(json!(1));
    }
    let since = s(args, "since");
    let sha = s(args, "sha");
    let mut records = Vec::new();
    for line in text(&corpus)?.lines() {
        let Ok(r) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if !since.is_empty() && s(&r, "ts") < since {
            continue;
        }
        if !sha.is_empty() && s(&r, "sha") != sha {
            continue;
        }
        records.push(r);
    }
    if records.is_empty() {
        say(host, "no records match")?;
        return Ok(json!(1));
    }
    let fails: Vec<&Value> = records.iter().filter(|r| !flag(r, "ok")).collect();
    say(
        host,
        format!(
            "corpus: {} boots, {} failures ({:.1}%)",
            records.len(),
            fails.len(),
            100.0 * fails.len() as f64 / records.len() as f64
        ),
    )?;
    let mut per_arch: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for r in &records {
        per_arch
            .entry(
                if s(r, "arch").is_empty() {
                    "?"
                } else {
                    s(r, "arch")
                }
                .into(),
            )
            .or_default()
            .push(r);
    }
    for (arch, rs) in per_arch {
        let bad = rs.iter().filter(|r| !flag(r, "ok")).count();
        say(
            host,
            format!(
                "  {arch}: {} boots, {bad} failures ({:.1}%)",
                rs.len(),
                100.0 * bad as f64 / rs.len() as f64
            ),
        )?;
    }
    let mut families: Vec<((String, String, String, String), Vec<&Value>)> = Vec::new();
    for r in fails {
        let choose = |key: &str, default: &str| {
            if s(r, key).is_empty() {
                default.to_owned()
            } else {
                s(r, key).to_owned()
            }
        };
        let key = (
            choose("arch", "?"),
            choose("classification", &choose("outcome", "?")),
            choose("stop_code", "-"),
            choose("top_symbol", "-")
                .split('+')
                .next()
                .unwrap_or("-")
                .into(),
        );
        if let Some((_, rows)) = families.iter_mut().find(|(k, _)| k == &key) {
            rows.push(r);
        } else {
            families.push((key, vec![r]));
        }
    }
    families.sort_by_key(|(_, rs)| std::cmp::Reverse(rs.len()));
    if !families.is_empty() {
        say(host, "\nfailure families (most frequent first):")?;
        for ((arch, class, stop, symbol), rows) in families {
            let last = rows.last().unwrap();
            say(
                host,
                format!(
                    "  {:3}x  {arch:6} {class:14} stop={stop:12} {symbol}",
                    rows.len()
                ),
            )?;
            if let Some(symbols) = last["symbols"].as_array() {
                for value in symbols.iter().take(3) {
                    say(host, format!("          {}", value.as_str().unwrap_or("")))?;
                }
            }
            say(
                host,
                format!(
                    "          latest: {}",
                    last["preserved"]
                        .as_str()
                        .or_else(|| last["run_dir"].as_str())
                        .unwrap_or("None")
                ),
            )?;
        }
    }
    let mut trips: Vec<((String, String), Vec<&Value>)> = Vec::new();
    for r in &records {
        for kind in ["lockdep", "cswitch"] {
            if r[format!("{kind}_hits")].as_u64().unwrap_or(0) > 0 {
                let key = (
                    if s(r, "arch").is_empty() {
                        "?"
                    } else {
                        s(r, "arch")
                    }
                    .into(),
                    kind.into(),
                );
                if let Some((_, rows)) = trips.iter_mut().find(|(k, _)| k == &key) {
                    rows.push(r);
                } else {
                    trips.push((key, vec![r]));
                }
            }
        }
    }
    trips.sort_by_key(|(_, rows)| std::cmp::Reverse(rows.len()));
    if !trips.is_empty() {
        say(
            host,
            "\ntripwire hits (on green + failed boots — detector fired):",
        )?;
        for ((arch, kind), rows) in trips {
            let hits: u64 = rows
                .iter()
                .map(|r| r[format!("{kind}_hits")].as_u64().unwrap_or(0))
                .sum();
            let last = rows.last().unwrap();
            say(
                host,
                format!(
                    "  {:3} boots {arch:6} {kind:8} {hits} total hits",
                    rows.len()
                ),
            )?;
            if let Some(lines) = last["census_lines"].as_array() {
                for value in lines.iter().take(3) {
                    say(host, format!("          {}", value.as_str().unwrap_or("")))?;
                }
            }
            say(
                host,
                format!(
                    "          latest: {}",
                    last["preserved"]
                        .as_str()
                        .or_else(|| last["run_dir"].as_str())
                        .unwrap_or("None")
                ),
            )?;
        }
    }
    Ok(json!(0))
}
pub fn dispatch(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    match s(request, "op") {
        "farm_run" => run(root, a, host),
        "farm_report" => report(a, host),
        "farm_extract_evidence" => Ok(extract_evidence(
            Path::new(s(a, "run_dir")),
            &serde_json::from_value::<Vec<String>>(
                a.get("salient").cloned().unwrap_or_else(|| json!([])),
            )?,
        )),
        "farm_preserve_failure" => {
            let env = env_map(a)?;
            Ok(json!(preserve_failure(
                &farm_dir(a, &env),
                Path::new(s(a, "run_dir")),
                a.get("record").ok_or("missing farm record")?
            )?))
        }
        "farm_append" => {
            let env = env_map(a)?;
            append_record(
                &a.get("corpus")
                    .and_then(Value::as_str)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| farm_dir(a, &env).join("corpus.jsonl")),
                a.get("record").ok_or("missing farm record")?,
            )?;
            Ok(Value::Null)
        }
        "farm_git_state" => Ok(json!(git_state(root, &env_map(a)?)?)),
        _ => Err(Error::new(format!(
            "unknown farm operation {}",
            s(request, "op")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Output(Vec<String>);
    impl Host for Output {
        fn call(&mut self, op: &str, args: Value) -> Result<Value> {
            if op != "print" {
                return Err(format!("unexpected callback {op}").into());
            }
            self.0.push(s(&args, "text").into());
            Ok(Value::Null)
        }
    }
    #[test]
    fn evidence_prefers_faulting_instruction_and_is_durable() {
        let d = tempfile::tempdir().unwrap();
        let run = d.path().join("run");
        fs::create_dir(&run).unwrap();
        fs::write(run.join("full.log"),"*** BUGCHECK: 0x0000001e\n*** P3 = innocent.sys+0x20\n*** TrapFrame.Rip = culprit.sys+0x10\n").unwrap();
        fs::write(run.join("guest_serial.log"), "serial evidence").unwrap();
        let ev = extract_evidence(&run, &[]);
        assert_eq!(ev["stop_code"], "0x0000001E");
        assert_eq!(ev["top_symbol"], "culprit.sys+0x10");
        let record =
            json!({"classification":"BUGCHECK","outcome":"bugcheck","ts":"test","arch":"arm64"});
        let kept = preserve_failure(&d.path().join("farm"), &run, &record).unwrap();
        assert_eq!(
            fs::read(kept.join("guest_serial.log")).unwrap(),
            b"serial evidence"
        );
    }
    #[test]
    fn report_separates_tripwire_signals_from_red_boots() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("corpus.jsonl");
        append_record(&p,&json!({"arch":"amd64","ok":true,"lockdep_hits":2,"census_lines":["ABBA"],"run_dir":"good"})).unwrap();
        append_record(&p,&json!({"arch":"arm64","ok":false,"classification":"BUGCHECK","stop_code":"0x1E","top_symbol":"kernel+0x1","run_dir":"bad"})).unwrap();
        let mut host = Output(Vec::new());
        assert_eq!(
            report(&json!({"farm_dir":d.path(),"args":{}}), &mut host).unwrap(),
            0
        );
        let output = host.0.join("\n");
        assert!(output.contains("2 boots, 1 failures"));
        assert!(output.contains("tripwire hits"));
        assert!(output.contains("2 total hits"));
        assert!(output.contains("BUGCHECK"));
    }
}

// Integrate helper into farm.rs. In Event::Quiet response closure, replace the
// inline holder-policy block with:
// Ok(quiet_reason(&v, &owner_pids.iter().copied().collect(), std::process::id()))
// Policy is tested natively; Python only returns existing raw holder metadata.
fn quiet_reason(response: &Value, owners: &BTreeSet<u32>, owner_pid: u32) -> Option<String> {
    let holder = &response["land_holder"];
    let other = response["foreign_boot_holders"]
        .as_array()
        .map(|holders| {
            holders
                .iter()
                .filter(|meta| {
                    let pid = meta
                        .get("client_pid")
                        .and_then(Value::as_u64)
                        .or_else(|| meta.get("pid").and_then(Value::as_u64));
                    pid != Some(owner_pid as u64)
                        && !pid.is_some_and(|p| owners.contains(&(p as u32)))
                })
                .count()
        })
        .unwrap_or(0);
    if !holder.is_null() && holder.as_object().is_some_and(|m| !m.is_empty()) {
        Some(format!("a land is gating ({})", s(holder, "kind")))
    } else if other > 0 {
        Some(format!("{other} other boot(s) running"))
    } else {
        None
    }
}
#[cfg(test)]
mod quiet_controls {
    use super::*;
    #[test]
    fn land_then_foreign_boot_then_quiet_never_admits_early() {
        let own = BTreeSet::from([12, 13]);
        assert!(quiet_reason(
            &json!({"land_holder":{"kind":"land"},"foreign_boot_holders":[]}),
            &own,
            11
        )
        .unwrap()
        .contains("land"));
        assert!(quiet_reason(
            &json!({"land_holder":{},"foreign_boot_holders":[{"pid":99}]}),
            &own,
            11
        )
        .unwrap()
        .contains("1 other"));
        assert_eq!(
            quiet_reason(
                &json!({"land_holder":{},"foreign_boot_holders":[]}),
                &own,
                11
            ),
            None
        );
    }
    #[test]
    fn native_api_leases_are_attributed_to_their_own_worker_clients() {
        assert_eq!(
            quiet_reason(
                &json!({"land_holder":{},"foreign_boot_holders":[{"pid":999,"client_pid":12}]}),
                &BTreeSet::from([12]),
                11
            ),
            None
        );
        assert!(quiet_reason(
            &json!({"land_holder":{},"foreign_boot_holders":[{"pid":999,"client_pid":98}]}),
            &BTreeSet::from([12]),
            11
        )
        .is_some());
    }
    #[test]
    fn own_parallel_workers_are_not_foreign_and_unknown_owners_remain_busy() {
        let own = BTreeSet::from([12, 13]);
        assert_eq!(
            quiet_reason(
                &json!({"land_holder":{},"foreign_boot_holders":[{"pid":11},{"pid":12},{"pid":13}]}),
                &own,
                11
            ),
            None
        );
        assert!(quiet_reason(
            &json!({"land_holder":{},"foreign_boot_holders":[{}]}),
            &own,
            11
        )
        .is_some());
    }
    #[test]
    fn expired_worker_never_requests_permission_or_launches_a_boot() {
        let d = tempfile::tempdir().unwrap();
        let (tx, rx) = mpsc::sync_channel(4);
        let cfg = Arc::new(FarmConfig {
            repo: d.path().into(),
            env: BTreeMap::new(),
            worker_prefix: vec!["/must-never-launch".into()],
            work: d.path().into(),
            deadline: Instant::now() - Duration::from_secs(1),
            max_boots: 5,
            polite: true,
            cancel: Arc::new(AtomicBool::new(false)),
            owners: Arc::new(Mutex::new(BTreeSet::new())),
        });
        arch_worker(cfg, "arm64".into(), tx);
        assert!(matches!(rx.recv().unwrap(), Event::Finished));
        assert!(rx.try_recv().is_err());
        assert_eq!(fs::read_dir(d.path()).unwrap().count(), 0);
    }
}
