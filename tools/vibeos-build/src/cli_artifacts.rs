//! Shared CLI artifact/reporting mechanisms. Compatibility callers exchange DTOs;
//! all filesystem publication and status/rendering decisions are native.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
fn text<'a>(v: &'a Value, key: &str, default: &'a str) -> &'a str {
    v[key].as_str().unwrap_or(default)
}
fn mono() -> f64 {
    let mut t: libc::timespec = unsafe { std::mem::zeroed() };
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut t) };
    t.tv_sec as f64 + t.tv_nsec as f64 / 1e9
}
fn epoch() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn path(v: &Value, key: &str) -> Result<PathBuf> {
    v[key]
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing CLI {key} path").into())
}
fn pretty(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().ok_or("summary has no directory")?;
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| Error::new(e.to_string()))?;
    Ok(())
}
fn locked_append(path: &Path, bytes: &[u8]) -> Result<()> {
    let shared = matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("full.log" | "commands.jsonl")
    );
    let guard = if shared {
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path.parent().unwrap().join("report.lock"))?,
        )
    } else {
        None
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let fd = guard.as_ref().map_or(file.as_raw_fd(), |f| f.as_raw_fd());
    if unsafe { libc::flock(fd, libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

fn stamp(ts: i64) -> Result<String> {
    let t = ts as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
        return Err("cannot format run timestamp".into());
    }
    let mut out = [0u8; 64];
    let n = unsafe {
        libc::strftime(
            out.as_mut_ptr().cast(),
            out.len(),
            c"%Y%m%d-%H%M%S".as_ptr(),
            &tm,
        )
    };
    Ok(String::from_utf8_lossy(&out[..n]).into_owned())
}
pub fn marker(dir: &Path, command: &str, arch: &str, owner: u32) -> Result<()> {
    pretty(
        &dir.join("summary.json"),
        &json!({"run_id":dir.file_name().unwrap().to_string_lossy(),"command":command,"arch":arch,"verdict":"INCOMPLETE","classification":"RUN_INTERRUPTED","incomplete":true,"pid":owner,"started_epoch":epoch(),"note":"This run never reached finalize(): it was killed, or it is still running. Either way it has no verdict and must not be read as one."}),
    )
}
pub fn create(a: &Value) -> Result<Value> {
    let root = path(a, "artifact_root")?;
    fs::create_dir_all(&root)?;
    let command = text(a, "command", "");
    let arch = text(a, "arch", "");
    for part in [command, arch] {
        if part.is_empty() || part.contains('/') || part.contains('\0') {
            return Err("invalid artifact command/architecture".into());
        }
    }
    let prefix = format!(
        "{}-",
        stamp(a["ts"].as_i64().unwrap_or_else(|| epoch() as i64))?
    );
    let suffix = format!("-{command}-{arch}");
    let dir = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(&suffix)
        .tempdir_in(root)?
        .keep();
    marker(
        &dir,
        command,
        arch,
        a["owner"].as_u64().unwrap_or(std::process::id() as u64) as u32,
    )?;
    Ok(json!({"dir":dir,"start":mono()}))
}
fn copied(src: &Path, dst: &Path) -> Result<()> {
    fs::copy(src, dst)?;
    if let Ok(t) = fs::metadata(src).and_then(|m| m.modified()) {
        File::options().write(true).open(dst)?.set_modified(t)?;
    }
    Ok(())
}
pub fn archive_serial(dir: &Path, live: Option<&str>, extra: &mut Value) -> Result<()> {
    let Some(live) = live.filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    extra["live_log"] = json!(live);
    let src = Path::new(live);
    if !src.is_file() {
        return Ok(());
    }
    let dst = dir.join("serial.log");
    let bytes = fs::read(src)?;
    let mut out = tempfile::NamedTempFile::new_in(dir)?;
    out.write_all(&bytes)?;
    out.persist(&dst).map_err(|e| Error::new(e.to_string()))?;
    extra["serial_log"] = json!(dst);
    Ok(())
}
fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let out =
        crate::process::capture(Command::new("git").arg("-C").arg(repo).args(args), 30).ok()?;
    if out.code != 0 {
        None
    } else {
        Some(out.stdout.trim().into())
    }
}
pub fn head(repo: &Path) -> Option<String> {
    git(repo, &["rev-parse", "--short", "HEAD"])
}
pub fn dirty(repo: &Path) -> Option<bool> {
    git(repo, &["status", "--porcelain"]).map(|s| !s.is_empty())
}
fn sha(path: &Path) -> Result<String> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = f.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        h.update(&buffer[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn snapshot(repo: &Path, farm: &Path, arch: &str, mp: bool) -> Result<Value> {
    let base = repo.join(format!("build-{arch}/ntoskrnl"));
    let kernel = if mp {
        base.join("ntkrnlmp/ntkrnlmp.exe")
    } else {
        base.join("ntoskrnl.exe")
    };
    if !kernel.is_file() {
        return Ok(json!({"kernel_sha12":null,"kernels_dir":null}));
    }
    let digest = sha(&kernel)?;
    let key = &digest[..12];
    let dst = farm.join("kernels").join(key);
    if !dst.is_dir() {
        let parent = dst.parent().unwrap();
        fs::create_dir_all(parent)?;
        let tmp = tempfile::Builder::new()
            .prefix(&format!("{key}.partial-"))
            .tempdir_in(parent)?;
        let target = tmp.path().join(kernel.file_name().unwrap());
        copied(&kernel, &target)?;
        if sha(&target)? != digest {
            return Err("kernel changed while preserving failure evidence".into());
        }
        let index = repo.join(format!(
            "qemu/symbols-{arch}{}.tsv",
            if mp { "-mp" } else { "" }
        ));
        if index.is_file() {
            copied(&index, &tmp.path().join(index.file_name().unwrap()))?;
        }
        let hal = repo.join(format!(
            "build-{arch}/hal/{}/{}",
            if mp { "halmp" } else { "hal" },
            if mp { "halmp.dll" } else { "hal.dll" }
        ));
        if hal.is_file() {
            copied(&hal, &tmp.path().join(hal.file_name().unwrap()))?;
        }
        match fs::rename(tmp.path(), &dst) {
            Ok(()) => {}
            Err(_) if dst.is_dir() => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(json!({"kernel_sha12":key,"kernels_dir":dst}))
}
pub fn preserve(
    dir: &Path,
    farm: &Path,
    arch: &str,
    class: &str,
    repo: Option<&Path>,
) -> Result<Option<PathBuf>> {
    let mut serials = Vec::new();
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        let name = p.file_name().unwrap().to_string_lossy();
        if name.contains("serial") && name.ends_with(".log") && p.is_file() {
            serials.push(p)
        }
    }
    serials.sort();
    if serials.is_empty() {
        return Ok(None);
    }
    let mp = serials.iter().any(|p| {
        fs::read_to_string(p)
            .unwrap_or_default()
            .contains("ntkrnlmp.exe")
    });
    let tag = regex::Regex::new(r"[^A-Za-z0-9_.-]+")?
        .replace_all(if class.is_empty() { "FAIL" } else { class }, "_")
        .trim_matches('_')
        .to_owned();
    let dst = farm.join("fails/auto").join(format!(
        "{}-{tag}",
        dir.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(&dst)?;
    for p in serials {
        copied(&p, &dst.join(p.file_name().unwrap()))?;
    }
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        let n = p.file_name().unwrap().to_string_lossy();
        if p.is_file()
            && (matches!(
                n.as_ref(),
                "summary.json" | "full.log" | "hang-cpustate.txt"
            ) || n.ends_with("-results.txt")
                || n.ends_with(".png"))
        {
            copied(&p, &dst.join(p.file_name().unwrap()))?;
        }
    }
    let mut manifest = json!({"run_id":dir.file_name().unwrap().to_string_lossy(),"arch":arch,"classification":class,"mp_kernel":mp,"head":repo.and_then(head),"dirty":repo.and_then(dirty)});
    if let Some(repo) = repo {
        let snap = snapshot(repo, farm, arch, mp)?;
        manifest
            .as_object_mut()
            .unwrap()
            .extend(snap.as_object().unwrap().clone());
    }
    pretty(&dst.join("manifest.json"), &manifest)?;
    Ok(Some(dst))
}
pub fn ledger(repo: Option<&Path>, env: &BTreeMap<String, String>, a: &Value) -> Result<()> {
    let Some(repo) = repo else { return Ok(()) };
    let root = crate::build_policy::fleet_root(repo, env)?;
    if !root.is_dir() {
        return Ok(());
    }
    let mut row = json!({"ts":epoch()as u64,"cmd":a["cmd"],"arch":a["arch"],"verdict":a["verdict"],"classification":a.get("classification").cloned().unwrap_or(json!("")),"duration_ms":a.get("duration_ms").cloned().unwrap_or(json!(0)),"head":head(repo).unwrap_or_default(),"worktree":repo.file_name().unwrap_or_default().to_string_lossy()});
    if let Some(extra) = a["extra"].as_object() {
        row.as_object_mut().unwrap().extend(extra.clone());
    }
    locked_append(
        &root.join("vibeos-ledger.jsonl"),
        format!("{}\n", serde_json::to_string(&row)?).as_bytes(),
    )
}
pub fn render(state: &Value, dir: &Path, seconds: f64) -> String {
    let command = text(state, "command", "");
    let arch = text(state, "arch", "");
    let verdict = text(state, "verdict", "OK");
    let class = text(state, "classification", "");
    let time = if seconds >= 60. {
        format!("{}m{:02}s", seconds as u64 / 60, seconds as u64 % 60)
    } else {
        format!("{seconds:.0}s")
    };
    let mut output = format!(
        "{command} {arch}  —  {verdict}{}   ({time})\n",
        if class.is_empty() {
            String::new()
        } else {
            format!(": {class}")
        }
    );
    let stages = state["stages"].as_array();
    if let Some(stages) = stages.filter(|s| !s.is_empty()) {
        let rows = stages
            .iter()
            .map(|s| {
                let status = text(s, "status", "ok");
                let glyph = match status {
                    "ok" => "✓",
                    "warn" => "⚠",
                    "fail" => "✗",
                    "skip" => "·",
                    _ => "?",
                };
                let detail = text(s, "detail", "");
                format!(
                    "{glyph} {}{}",
                    text(s, "name", ""),
                    if !detail.is_empty() && matches!(status, "warn" | "fail") {
                        format!("({detail})")
                    } else {
                        String::new()
                    }
                )
            })
            .collect::<Vec<_>>();
        output.push_str(&format!("  {}\n", rows.join("   ")));
        for stage in stages {
            if let Some(lines) = stage["salient"].as_array() {
                for line in lines.iter().take(4) {
                    output.push_str(&format!(
                        "  > {}\n",
                        line.as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| line.to_string())
                    ));
                }
            }
        }
    }
    output.push_str(&format!("full log:  {}\n", dir.join("full.log").display()));
    for key in ["serial_log", "live_log", "screenshot"] {
        if let Some(value) = state["extra"].get(key) {
            output.push_str(&format!(
                "{:<10} {}\n",
                key.replace('_', " "),
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string())
            ));
        }
    }
    output
}
pub fn finalize(a: &Value) -> Result<Value> {
    let dir = path(a, "dir")?;
    let repo = a["repo"].as_str().map(Path::new);
    let mut state = a["state"].clone();
    let duration = a["elapsed"]
        .as_f64()
        .unwrap_or_else(|| mono() - a["start"].as_f64().unwrap_or_else(mono))
        .max(0.);
    let mut extra = state.get("extra").cloned().unwrap_or(json!({}));
    let live = extra["live_log"].as_str().map(str::to_owned);
    if let Err(e) = archive_serial(&dir, live.as_deref(), &mut extra) {
        extra["serial_archive_error"] = json!(e.to_string());
    }
    state["extra"] = extra.clone();
    let full = dir.join("full.log");
    if !full.exists() {
        locked_append(
            &full,
            b"(no stage ran a command; see summary.json for the stages)\n",
        )?;
    }
    let mut summary = json!({"run_id":dir.file_name().unwrap().to_string_lossy(),"command":state["command"],"arch":state["arch"],"worktree":repo.map(|p|p.canonicalize().unwrap_or_else(|_|p.to_owned())),"verdict":state["verdict"],"classification":state["classification"],"duration_ms":(duration*1000.)as u64,"stages":state["stages"],"command_timings":a.get("command_timings").cloned().unwrap_or(json!([])),"artifacts":{"full_log":full}});
    summary.as_object_mut().unwrap().extend(
        extra
            .as_object()
            .ok_or("run extra must be an object")?
            .clone(),
    );
    pretty(&dir.join("summary.json"), &summary)?;
    if !matches!(text(&state, "verdict", "OK"), "OK" | "WARN") {
        if let Some(farm) = a["farm"].as_str() {
            match preserve(
                &dir,
                Path::new(farm),
                text(&state, "arch", ""),
                text(&state, "classification", ""),
                repo,
            ) {
                Ok(Some(path)) => {
                    summary["preserved"] = json!(path);
                    pretty(&dir.join("summary.json"), &summary)?;
                }
                Ok(None) => {}
                Err(error) => {
                    summary["preservation_error"] = json!(error.to_string());
                    pretty(&dir.join("summary.json"), &summary)?;
                }
            }
        }
    }
    Ok(
        json!({"code":if matches!(text(&state,"verdict","OK"),"OK"|"WARN"){0}else{1},"summary":summary,"extra":extra,"output":render(&state,&dir,duration),"seconds":duration}),
    )
}
fn merge(base: &Value, over: &Value) -> Value {
    if let (Some(base), Some(over)) = (base.as_object(), over.as_object()) {
        let mut out = base.clone();
        for (k, v) in over {
            out.insert(
                k.clone(),
                out.get(k).map(|b| merge(b, v)).unwrap_or_else(|| v.clone()),
            );
        }
        Value::Object(out)
    } else {
        over.clone()
    }
}
fn set(data: &mut Value, key: &str, value: Value) {
    let mut current = data;
    let mut keys = key.split('.').peekable();
    while let Some(k) = keys.next() {
        if !current.is_object() {
            *current = json!({})
        }
        if keys.peek().is_none() {
            current[k] = value;
            return;
        }
        if !current[k].is_object() {
            current[k] = json!({})
        }
        current = current.get_mut(k).unwrap();
    }
}
pub fn defaults() -> Value {
    json!({"qemu":{"backend":"windows-host","windows":{"qemu_dir":"/mnt/c/qemu-11.1.0","staging_dir":"/mnt/c/temp/vibeos-qemu","accel":"tcg","host_addr":"127.0.0.1"}}})
}
const ENV_MAP: &[(&str, &str)] = &[
    ("VIBEOS_QEMU_BACKEND", "qemu.backend"),
    ("VIBEOS_QEMU_WINDOWS_DIR", "qemu.windows.qemu_dir"),
    ("VIBEOS_QEMU_WINDOWS_STAGING", "qemu.windows.staging_dir"),
    ("VIBEOS_QEMU_WINDOWS_ACCEL", "qemu.windows.accel"),
    ("VIBEOS_QEMU_WINDOWS_HOST_ADDR", "qemu.windows.host_addr"),
];
pub fn configuration(a: &Value) -> Result<Value> {
    match text(a, "action", "") {
        "defaults" => Ok(defaults()),
        "merge" => Ok(merge(&a["base"], &a["over"])),
        "set" => {
            let mut data = a["data"].clone();
            set(&mut data, text(a, "key", ""), a["value"].clone());
            Ok(data)
        }
        "get" => {
            let mut value = &a["data"];
            for key in text(a, "key", "").split('.') {
                let Some(next) = value.get(key) else {
                    return Ok(a.get("default").cloned().unwrap_or(Value::Null));
                };
                value = next;
            }
            Ok(value.clone())
        }
        "env" => {
            let mut data = json!({});
            for (env, key) in ENV_MAP {
                if let Some(value) = a["env"]
                    .get(env)
                    .filter(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                {
                    set(&mut data, key, value.clone());
                }
            }
            Ok(data)
        }
        "local" => {
            let repo = path(a, "repo")?;
            Ok(fs::read(repo.join(".vibeos-cli.local.json"))
                .ok()
                .and_then(|v| serde_json::from_slice::<Value>(&v).ok())
                .filter(Value::is_object)
                .unwrap_or(json!({})))
        }
        "load" => {
            let repo = path(a, "repo")?;
            let over = fs::read(repo.join(".vibeos-cli.local.json"))
                .ok()
                .and_then(|v| serde_json::from_slice::<Value>(&v).ok())
                .filter(Value::is_object)
                .unwrap_or(json!({}));
            let mut data = merge(&a.get("defaults").cloned().unwrap_or_else(defaults), &over);
            for (env, key) in ENV_MAP {
                if let Some(value) = a["env"]
                    .get(env)
                    .filter(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                {
                    set(&mut data, key, value.clone());
                }
            }
            Ok(data)
        }
        _ => Err("unknown CLI configuration operation".into()),
    }
}
pub fn dispatch(_root: &Path, request: &Value) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    let op = text(request, "op", "").trim_start_matches("cli_artifacts_");
    match op {
        "create" => create(a),
        "marker" => {
            marker(
                &path(a, "dir")?,
                text(a, "command", ""),
                text(a, "arch", ""),
                a["owner"].as_u64().unwrap_or(std::process::id() as u64) as u32,
            )?;
            Ok(Value::Null)
        }
        "log" => {
            let dir = path(a, "dir")?;
            let mut value = String::new();
            if let Some(h) = a["header"].as_str() {
                value.push_str(&format!("\n=== {h} ===\n"));
            }
            value.push_str(text(a, "text", ""));
            if !value.ends_with('\n') {
                value.push('\n')
            }
            locked_append(&dir.join("full.log"), value.as_bytes())?;
            Ok(Value::Null)
        }
        "finalize" => finalize(a),
        "render" => Ok(json!(render(
            &a["state"],
            &path(a, "dir")?,
            a["elapsed"].as_f64().unwrap_or(0.)
        ))),
        "archive_serial" => {
            let mut extra = a["extra"].clone();
            archive_serial(&path(a, "dir")?, a["live_log"].as_str(), &mut extra)?;
            Ok(extra)
        }
        "sha12" => Ok(sha(&path(a, "path")?)
            .ok()
            .map(|s| json!(&s[..12]))
            .unwrap_or(Value::Null)),
        "snapshot_kernel" => snapshot(
            &path(a, "repo")?,
            &path(a, "farm")?,
            text(a, "arch", ""),
            a["mp"] == true,
        ),
        "preserve_failure" => Ok(preserve(
            &path(a, "dir")?,
            &path(a, "farm")?,
            text(a, "arch", ""),
            text(a, "classification", ""),
            a["repo"].as_str().map(Path::new),
        )
        .ok()
        .flatten()
        .map(|p| json!(p))
        .unwrap_or(Value::Null)),
        "head" => Ok(a["repo"]
            .as_str()
            .and_then(|s| head(Path::new(s)))
            .map(|s| json!(s))
            .unwrap_or(Value::Null)),
        "dirty" => Ok(a["repo"]
            .as_str()
            .and_then(|s| dirty(Path::new(s)))
            .map(|s| json!(s))
            .unwrap_or(Value::Null)),
        "ledger" => {
            let env: BTreeMap<String, String> =
                serde_json::from_value(a.get("env").cloned().unwrap_or(json!({})))?;
            ledger(a["repo"].as_str().map(Path::new), &env, a)?;
            Ok(Value::Null)
        }
        "stage_verdict" => Ok(json!(if a["stage"]["status"] == "fail" {
            "FAIL"
        } else if a["stage"]["status"] == "warn" && a["verdict"] == "OK" {
            "WARN"
        } else {
            text(a, "verdict", "OK")
        })),
        "config" => configuration(a),
        "command" => command(a),
        _ => Err(format!("unknown CLI artifact operation {op}").into()),
    }
}

pub fn command(a: &Value) -> Result<Value> {
    let dir = a["dir"].as_str().map(PathBuf::from).unwrap_or_else(|| {
        std::env::temp_dir()
            .join("vibeos-cli")
            .join("gate-commands")
    });
    let argv: Vec<String> = serde_json::from_value(a["argv"].clone())?;
    if argv.is_empty() {
        return Err("empty CLI command".into());
    }
    fs::create_dir_all(&dir)?;
    let commands = dir.join("commands");
    fs::create_dir_all(&commands)?;
    let sub = tempfile::Builder::new()
        .prefix("cmd-")
        .tempdir_in(&commands)?
        .keep();
    let out_path = sub.join("stdout.log");
    let err_path = sub.join("stderr.log");
    let receipt = sub.join("exit.json");
    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.arg("--root")
        .arg(a["repo"].as_str().unwrap_or("."))
        .arg("cli-command-worker")
        .arg(&receipt)
        .args(&argv);
    if let Some(cwd) = a["cwd"].as_str() {
        cmd.current_dir(cwd);
    }
    if let Some(env) = a["env"].as_object() {
        cmd.env_clear();
        for (k, v) in env {
            cmd.env(k, v.as_str().ok_or("invalid command environment")?);
        }
    }
    let timeout = a["timeout"]
        .as_f64()
        .map(|s| {
            if s.is_finite() && s >= 0. {
                Duration::try_from_secs_f64(s)
                    .map_err(|_| Error::new("command timeout exceeds supported duration"))
            } else {
                Err(Error::new("command timeout must be finite and nonnegative"))
            }
        })
        .transpose()?;
    let started = mono();
    let result = crate::artifact_executor::supervised_spool(
        &mut cmd,
        File::create(&out_path)?,
        File::create(&err_path)?,
        timeout,
    );
    let mut launch_error = None;
    let (mut code, timed_out) = match result {
        Ok(value) => value,
        Err(error) if error.kind.as_deref() == Some("LaunchError") => {
            launch_error = Some(error.message.clone());
            fs::write(&err_path, error.message)?;
            (127, false)
        }
        Err(error) => return Err(error),
    };
    if timed_out {
        code = 124
    } else if let Ok(bytes) = fs::read(&receipt) {
        if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
            launch_error = value["launch_error"].as_str().map(str::to_owned);
            if let Some(raw) = value["exit_code"].as_i64() {
                code = i32::try_from(raw).map_err(|_| Error::new("invalid child exit code"))?;
            }
        }
    }
    let seconds = mono() - started;
    let label = a["stage"].as_str().unwrap_or(&argv[0]);
    let timing = json!({"stage":label,"start_seconds":started-a["run_start"].as_f64().unwrap_or(started),"seconds":seconds,"exit":code,"timed_out":timed_out});
    let lock = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("report.lock"))?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("full.log"))?;
    writeln!(
        log,
        "\n=== {label}: {} — exit {code}, {seconds:.1}s ===",
        argv.join(" ")
    )?;
    std::io::copy(&mut File::open(&out_path)?, &mut log)?;
    if fs::metadata(&err_path)?.len() > 0 {
        log.write_all(b"\n--- stderr ---\n")?;
        std::io::copy(&mut File::open(&err_path)?, &mut log)?;
    }
    log.flush()?;
    let mut journal = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("commands.jsonl"))?;
    let mut entry = timing.clone();
    entry["cmd"] = json!(argv);
    entry["cwd"] = a.get("cwd").cloned().unwrap_or(json!(""));
    entry["stdout_path"] = json!(out_path);
    entry["stderr_path"] = json!(err_path);
    writeln!(journal, "{}", serde_json::to_string(&entry)?)?;
    drop(journal);
    drop(log);
    drop(lock);
    let capture = a["capture"] != false;
    Ok(
        json!({"returncode":code,"timed_out":timed_out,"launch_error":launch_error,"stdout":if capture{crate::read(&out_path)?}else{String::new()},"stderr":if capture{crate::read(&err_path)?}else{String::new()},"stdout_path":out_path,"stderr_path":err_path,"timing":timing}),
    )
}
pub fn worker(argv: &[String]) -> Result<u8> {
    let (receipt, command) = argv
        .split_first()
        .ok_or("CLI worker requires receipt and command")?;
    let mut receipt = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(receipt)?;
    crate::process_guard::controller(command, &mut receipt)
}
