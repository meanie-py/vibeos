//! Native boot watching and smoke sequencing; Python retains CLI presentation.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn number(v: &Value, key: &str, default: f64) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(default)
}
fn flag(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}
fn tail(s: &str, n: usize) -> String {
    s.chars()
        .rev()
        .take(n)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}
fn limit(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
fn duration(n: f64) -> Result<Duration> {
    if !n.is_finite() || n < 0.0 {
        return Err("invalid smoke duration".into());
    }
    Ok(Duration::from_secs_f64(n))
}
fn rounded(n: f64) -> f64 {
    (n * 1000.0).round() / 1000.0
}
fn request(host: &mut dyn Host, op: &str, kwargs: Value) -> Result<Value> {
    host.call(op, kwargs)
}
fn vm(host: &mut dyn Host, op: &str, id: u64, mut kwargs: Value) -> Result<Value> {
    kwargs
        .as_object_mut()
        .ok_or("VM arguments must be an object")?
        .insert("id".into(), json!(id));
    request(host, op, kwargs)
}
fn log(host: &mut dyn Host, header: &str, message: impl AsRef<str>) -> Result<()> {
    request(
        host,
        "log",
        json!({"header":header,"text":message.as_ref()}),
    )?;
    Ok(())
}
fn add(host: &mut dyn Host, name: &str, status: &str, detail: &str, salient: Value) -> Result<()> {
    request(
        host,
        "stage",
        json!({"name":name,"status":status,"detail":detail,"salient":salient}),
    )?;
    Ok(())
}
fn serial(host: &mut dyn Host, id: u64, timeout: f64) -> Result<String> {
    let value = vm(host, "vm_poll_serial", id, json!({"timeout":timeout}))?;
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::new("serial read did not return text"))
}

pub struct WatchState {
    pub buffer: String,
    marker: String,
    fail: Option<fancy_regex::Regex>,
    timeout: f64,
    silence: f64,
    last_activity: f64,
}
impl WatchState {
    pub fn new(marker: &str, fail: Option<&str>, timeout: f64, silence: f64) -> Result<Self> {
        duration(timeout)?;
        duration(silence)?;
        Ok(Self {
            buffer: String::new(),
            marker: marker.into(),
            fail: fail
                .filter(|s| !s.is_empty())
                .map(fancy_regex::Regex::new)
                .transpose()
                .map_err(|e| Error::new(format!("invalid fail-on expression: {e}")))?,
            timeout,
            silence,
            last_activity: 0.0,
        })
    }
    pub fn feed(&mut self, now: f64, chunk: &str) -> Result<Option<&'static str>> {
        if !chunk.is_empty() {
            self.buffer.push_str(chunk);
            self.last_activity = now;
            // Preserve marker priority within a read; post-marker triage catches crashes.
            if self.buffer.contains(&self.marker) {
                return Ok(Some("reached"));
            }
            if let Some(re) = &self.fail {
                if re
                    .is_match(&self.buffer)
                    .map_err(|e| Error::new(format!("fail-on match: {e}")))?
                {
                    return Ok(Some("failmarker"));
                }
            }
        } else if self.silence > 0.0
            && !self.buffer.is_empty()
            && now - self.last_activity > self.silence
        {
            return Ok(Some("timeout"));
        }
        Ok((now >= self.timeout).then_some("timeout"))
    }
    pub fn diagnostics(&self, now: f64) -> Value {
        let last = self
            .buffer
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");
        json!({"silent_s":(now-self.last_activity).max(0.0),"last_line":limit(last,160),
            "early_hang":self.silence>0.0 && !self.buffer.is_empty() && now-self.last_activity>self.silence && now<self.timeout})
    }
}
fn watch(
    host: &mut dyn Host,
    id: u64,
    marker: &str,
    fail: Option<&str>,
    timeout: f64,
    silence: f64,
) -> Result<(String, String, Value)> {
    let mut state = WatchState::new(marker, fail, timeout, silence)?;
    let start = Instant::now();
    loop {
        if start.elapsed().as_secs_f64() >= timeout {
            return Ok((
                "timeout".into(),
                state.buffer.clone(),
                state.diagnostics(start.elapsed().as_secs_f64()),
            ));
        }
        match serial(host, id, 0.5) {
            Ok(chunk) => {
                if let Some(outcome) = state.feed(start.elapsed().as_secs_f64(), &chunk)? {
                    let diag = if outcome == "timeout" {
                        state.diagnostics(start.elapsed().as_secs_f64())
                    } else {
                        json!({})
                    };
                    return Ok((outcome.into(), state.buffer, diag));
                }
            }
            Err(error) => {
                // The host failed to preserve explicitly requested evidence.
                // This is not a guest death verdict and must retain its I/O kind.
                if matches!(error.kind.as_deref(), Some("OSError" | "DiagnosticError")) {
                    return Err(error);
                }
                let mut diag = state.diagnostics(start.elapsed().as_secs_f64());
                diag["serial_error"] = json!(error.message);
                return Ok(("dead".into(), state.buffer, diag));
            }
        }
    }
}
pub(crate) fn settle(
    host: &mut dyn Host,
    id: u64,
    seconds: f64,
    milestone: Option<&str>,
    grace: f64,
) -> Result<(String, bool)> {
    let start = Instant::now();
    let mut out = String::new();
    while start.elapsed() < duration(seconds)? {
        out.push_str(&serial(host, id, 0.5)?);
    }
    if let Some(marker) = milestone {
        let until = Instant::now() + duration(grace)?;
        while !out.contains(marker) && Instant::now() < until {
            out.push_str(&serial(host, id, 0.5)?);
        }
        let found = out.contains(marker);
        Ok((out, found))
    } else {
        Ok((out, true))
    }
}
fn debug_text(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.into();
    }
    value
        .as_array()
        .map(|a| {
            a.iter()
                .map(|x| match x.as_u64() {
                    Some(n) if n < 128 => n as u8 as char,
                    _ => '\u{fffd}',
                })
                .collect()
        })
        .unwrap_or_default()
}
fn screenshot(
    host: &mut dyn Host,
    id: u64,
    work: &Path,
    root: &Path,
    arch: &str,
) -> Result<Option<String>> {
    let ppm = work.join("screen.ppm");
    if vm(host, "vm_screendump", id, json!({"path":ppm})).is_err() {
        return Ok(None);
    }
    let png = work.join("screen.png");
    match crate::screenshot::convert(&ppm, &png) {
        Ok(()) => {
            let canonical = root.join(format!("qemu/{arch}_smoke.png"));
            if let Some(parent) = canonical.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&png, canonical)?;
            Ok(Some(png.to_string_lossy().into()))
        }
        Err(e) => {
            log(
                host,
                "screenshot",
                format!(
                    "PNG capture failed: {}; retaining {}",
                    e.message,
                    ppm.display()
                ),
            )?;
            Ok(ppm.is_file().then(|| ppm.to_string_lossy().into()))
        }
    }
}
fn with_shot(mut result: Value, shot: &Option<String>) -> Value {
    if let Some(path) = shot {
        result["screenshot"] = json!(path);
    }
    result
}
fn merge_failure(outcome: &str, failure: Value, shot: &Option<String>) -> Value {
    let mut out = json!({"ok":false,"outcome":outcome});
    if let Some(map) = failure.as_object() {
        out.as_object_mut().unwrap().extend(map.clone());
    }
    with_shot(out, shot)
}
fn checks(
    host: &mut dyn Host,
    serial: &str,
    arch: &str,
    env: &Value,
    stderr: &str,
) -> Result<Value> {
    for name in [
        "smp",
        "ahci",
        "gic",
        "nvme",
        "xhci",
        "boot_nvme",
        "headless",
        "virtio_mmio",
        "ahci_irq",
        "ata_health",
        "kernel_assert",
        "legide",
    ] {
        let result = request(
            host,
            &format!("smoke_check_{name}"),
            json!({"text":if name == "ahci_irq" {stderr} else {serial},"serial":serial,"arch":arch,"env":env,"stderr":stderr,"stderr_text":stderr}),
        )?;
        if !result.is_null() {
            return Ok(result);
        }
    }
    Ok(Value::Null)
}
// Replace smoke.rs::ladder_watch; pass &env at its one_boot call site.
fn ladder_watch(
    host: &mut dyn Host,
    id: u64,
    seen: &str,
    optional: &Value,
    label: &str,
    env: &Value,
) -> Result<String> {
    struct QemuStream<'a> {
        host: &'a mut dyn Host,
        id: u64,
        start: Instant,
    }
    impl crate::ladder_watch::Stream for QemuStream<'_> {
        fn now(&mut self) -> Result<f64> {
            Ok(self.start.elapsed().as_secs_f64())
        }
        fn read_serial(&mut self, timeout: f64) -> Result<String> {
            serial(self.host, self.id, timeout)
        }
    }
    let constants = request(host, "ladder_constants", json!({}))?;
    let config: crate::ladder::Config = serde_json::from_value(
        constants
            .get("config")
            .cloned()
            .unwrap_or_else(|| serde_json::to_value(crate::ladder::Config::default()).unwrap()),
    )?;
    let ceiling = number(&constants, "timeout_seconds", 420.0);
    log(
        host,
        label,
        format!("draining up to {ceiling:.0}s for the WoW64 ladder"),
    )?;
    let floor = crate::ladder::drain_floor(env["VIBEOS_LADDER_DRAIN_FLOOR"].as_str())?;
    let staged: Vec<String> = serde_json::from_value(optional.clone())?;
    let started = Instant::now();
    let mut stream = QemuStream {
        host,
        id,
        start: started,
    };
    let output = crate::ladder_watch::watch(
        &mut stream,
        &config,
        ceiling,
        seen,
        &staged,
        true,
        floor,
        number(&constants, "optional_extra_seconds", 360.),
        number(&constants, "winetest_extra_seconds", 200.),
    )?;
    log(
        host,
        label,
        format!(
            "ladder drain: {:.1}s (ceiling {ceiling:.0}s)",
            started.elapsed().as_secs_f64()
        ),
    )?;
    Ok(output)
}

pub fn one_boot(root: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let started = Instant::now();
    let arch = text(a, "arch");
    let boot_arch = if text(a, "boot_arch").is_empty() {
        match arch {
            "i386" => "i386-uefi",
            "amd64" => "amd64-uefi",
            _ => arch,
        }
    } else {
        text(a, "boot_arch")
    };
    let run = PathBuf::from(text(a, "run_dir"));
    if run.as_os_str().is_empty() {
        return Err("one boot requires an owned run directory".into());
    }
    let idx = a["idx"].as_u64().unwrap_or(0);
    let total = a["total"].as_u64().unwrap_or(1);
    let work = run.join(format!("boot{idx}"));
    fs::create_dir_all(&work)?;
    let label = if total > 1 {
        format!("boot {}/{total}", idx + 1)
    } else {
        "boot".into()
    };
    let cli = a.get("cli").cloned().unwrap_or_else(|| json!({}));
    let backend = cli
        .pointer("/qemu/backend")
        .and_then(Value::as_str)
        .unwrap_or("native");
    let windows = cli
        .pointer("/qemu/windows")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let env = a.get("env").cloned().unwrap_or_else(|| json!({}));
    struct Launch<'a> {
        host: &'a mut dyn Host,
        options: Value,
        session: Option<(u64, Value)>,
        label: &'a str,
    }
    impl Host for Launch<'_> {
        fn call(&mut self, op: &str, _arguments: Value) -> Result<Value> {
            match op {
                "launch_make" => {
                    let created = request(self.host, "vm_create", self.options.clone())?;
                    let id = created["id"]
                        .as_u64()
                        .ok_or("VM factory omitted session ID")?;
                    let config = created
                        .get("config")
                        .cloned()
                        .ok_or("VM factory omitted configuration")?;
                    self.session = Some((id, config));
                    Ok(Value::Null)
                }
                "launch_start" => {
                    let id = self.session.as_ref().ok_or("VM factory omitted session")?.0;
                    match vm(self.host, "vm_start", id, json!({})) {
                        Ok(handles) => {
                            // Diagnostic preparation can extend replay budgets at
                            // launch. The watch must use that effective config,
                            // rather than the snapshot taken before vm_start.
                            if let Some(config) = handles.get("config") {
                                if !config.is_object() {
                                    return Err("launched VM configuration is not an object".into());
                                }
                                self.session.as_mut().ok_or("VM factory omitted session")?.1 =
                                    config.clone();
                            }
                            Ok(
                                json!({"timings":handles.get("launch_timings").cloned().unwrap_or_else(||json!({}))}),
                            )
                        }
                        Err(error) => Ok(json!({"error":error.message})),
                    }
                }
                "launch_stop" => {
                    if let Some((id, _)) = self.session.take() {
                        vm(self.host, "vm_stop", id, json!({}))?;
                        vm(self.host, "vm_close", id, json!({}))?;
                    }
                    Ok(Value::Null)
                }
                "launch_log" => {
                    log(self.host, self.label, text(&_arguments, "message"))?;
                    Ok(Value::Null)
                }
                _ => Err("unknown VM launch binding".into()),
            }
        }
    }
    let mut launch = Launch {
        host,
        options: json!({"arch":boot_arch,"work_dir":work,"backend":backend,"windows":windows,"serial_log":work.join("serial.log"),"gdb":flag(a,"attach"),"env":env}),
        session: None,
        label: &label,
    };
    let launched = crate::lifecycle::launch_retry(
        &mut launch,
        &json!({"attempts":a["launch_attempts"].as_u64().unwrap_or(3),"retry_seconds":number(a,"launch_retry_seconds",2.0),"label":label}),
    )?;
    let session = launch.session.take();
    let mut timings = launched["timings"].as_object().cloned().unwrap_or_default();
    if !launched["error"].is_null() {
        return Ok(
            json!({"ok":false,"outcome":"launch_fail","classification":"LAUNCH_FAIL","launch_attempts":launched["attempts"],"salient":[format!("{} launch attempts, all failed",launched["attempts"]),limit(text(&launched,"error"),140)],"timings":{"total_seconds":rounded(started.elapsed().as_secs_f64())}}),
        );
    }
    let (id, config) = session.ok_or("successful VM launch omitted session")?;
    let body = (|| -> Result<Value> {
        let until = text(a, "until");
        let marker = if until.is_empty() {
            text(&config, "boot_marker")
        } else {
            until
        };
        let watching = Instant::now();
        let (outcome, mut output, diag) = watch(
            host,
            id,
            marker,
            a.get("fail_on").and_then(Value::as_str),
            number(&config, "boot_timeout", 300.0),
            number(&config, "hang_silence_seconds", 0.0),
        )?;
        let deadline_cpu_state = if outcome == "timeout" {
            crate::smoke_evidence::cpu_state_at_deadline(host, id)
        } else {
            None
        };
        timings.insert(
            "watch_seconds".into(),
            json!(rounded(watching.elapsed().as_secs_f64())),
        );
        log(host, &format!("{label} serial tail"), tail(&output, 8000))?;
        let mut shell_seen = true;
        let settle_seconds = number(&config, "desktop_settle_seconds", 0.0);
        if outcome == "reached" && settle_seconds > 0.0 {
            let need_shell = until.is_empty()
                && config["required_milestones"]
                    .as_array()
                    .is_some_and(|rows| {
                        rows.iter()
                            .any(|r| r.as_str().is_some_and(|s| s.contains("shell ready")))
                    })
                && !output.contains("shell ready");
            let settling = Instant::now();
            let (more, seen) = settle(
                host,
                id,
                settle_seconds,
                need_shell.then_some("shell ready"),
                60.0,
            )?;
            shell_seen = seen;
            timings.insert(
                "settle_seconds".into(),
                json!(rounded(settling.elapsed().as_secs_f64())),
            );
            if !more.is_empty() {
                output.push_str(&more);
                log(host, &format!("{label} settle serial"), tail(&more, 4000))?;
            }
        }
        let optional = if flag(a, "want_ladder") {
            request(host, "ladder_media_optional_staged", json!({"arch":arch}))?
        } else {
            json!([])
        };
        if flag(a, "want_ladder") && outcome == "reached" {
            let more = ladder_watch(host, id, &output, &optional, &label, &env)?;
            if !more.is_empty() {
                output.push_str(&more);
                log(host, &format!("{label} ladder serial"), tail(&more, 8000))?;
            }
        }
        if outcome == "reached" && env["VIBEOS_QEMU_RR_SAMPLE"] == "check" {
            let checking = Instant::now();
            vm(host, "vm_finish_diagnostics", id, json!({}))?;
            timings.insert(
                "diagnostics_seconds".into(),
                json!(rounded(checking.elapsed().as_secs_f64())),
            );
        }
        let shot = screenshot(host, id, &work, root, boot_arch)?;
        if arch != "arm64" {
            if let Ok(stats) = vm(host, "vm_qmp_hmp", id, json!({"command":"info irq"})) {
                if let Some(s) = stats.as_str().filter(|s| !s.trim().is_empty()) {
                    log(host, &format!("{label} info irq"), s)?;
                }
            }
        }
        let raw = vm(host, "vm_read_debugcon", id, json!({})).unwrap_or(Value::Null);
        let debugcon_raw = debug_text(&raw);
        if outcome == "reached" {
            let stderr = vm(host, "vm_stderr_tail", id, json!({}))
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            let failed = checks(host, &output, arch, &env, &stderr)?;
            if !failed.is_null() {
                return Ok(merge_failure("smp-check", failed, &shot));
            }
            let mut post = request(
                host,
                "verdict_post_boot_check",
                json!({"serial":output,"debugcon_raw":debugcon_raw}),
            )?;
            if post.is_null() && !shell_seen {
                let tails = output
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .collect::<Vec<_>>();
                let mut salient = vec![json!(
                    "boot marker reached but `shell ready` never followed (settle+60s grace)"
                )];
                salient.extend(
                    tails
                        .iter()
                        .rev()
                        .take(2)
                        .rev()
                        .map(|s| json!(format!("serial tail: {}", limit(s, 120)))),
                );
                post = json!({"classification":"SHELL_NOT_READY","salient":salient});
            }
            if !post.is_null() {
                return Ok(merge_failure("post-boot", post, &shot));
            }
            if flag(a, "want_ladder") {
                let bad = request(
                    host,
                    "ladder_check",
                    json!({"serial":output,"optional_staged":optional}),
                )?;
                if !bad.is_null() {
                    return Ok(merge_failure("ladder", bad, &shot));
                }
                let summary = request(host, "ladder_summary", json!({"serial":output}))?;
                log(
                    host,
                    &label,
                    format!("ladder: {}", summary.as_str().unwrap_or("")),
                )?;
            }
            let mut result = with_shot(
                json!({"ok":true,"outcome":"reached","classification":""}),
                &shot,
            );
            let ev = request(host, "verdict_wedge_evidence", json!({"serial":output}))?;
            if ["census_stuck", "lockdep", "cswitch"]
                .iter()
                .any(|k| ev[k].as_u64().unwrap_or(0) > 0)
            {
                for (dest, key) in [
                    ("census_stuck", "census_stuck"),
                    ("lockdep_hits", "lockdep"),
                    ("cswitch_hits", "cswitch"),
                    ("census_lines", "lines"),
                ] {
                    result[dest] = ev[key].clone();
                }
            }
            return Ok(result);
        }
        let debug_report = request(
            host,
            "verdict_debugcon_report",
            json!({"raw":raw,"arch":arch}),
        )
        .unwrap_or(Value::Null);
        let mut tri = request(
            host,
            "verdict_classify",
            json!({"serial":output,"reached":false,"fail_marker":if outcome=="failmarker"{a.get("fail_on").cloned().unwrap_or(Value::Null)}else{Value::Null},"debugcon":debug_report,"debugcon_raw":debugcon_raw}),
        )?;
        if let Ok(sym) = request(
            host,
            "verdict_symbolicate_pc",
            json!({"arch":arch,"serial":output}),
        ) {
            if let Some(s) = sym.as_str() {
                tri["salient"]
                    .as_array_mut()
                    .ok_or("triage omitted salient lines")?
                    .push(json!(s));
            }
        }
        if outcome == "timeout" {
            if let Some(state) = deadline_cpu_state.as_deref() {
                let lines=request(host,"verdict_symbolicate_cpu_state",json!({"arch":arch,"state":state,"mp":output.contains("ntkrnlmp.exe"),"serial":output})).unwrap_or_else(|_|json!([]));
                let path = work.join("hang-cpustate.txt");
                let mut data = state.to_string();
                if let Some(lines) = lines.as_array() {
                    for line in lines {
                        if let Some(line) = line.as_str() {
                            data.push('\n');
                            data.push_str(line);
                            tri["salient"]
                                .as_array_mut()
                                .ok_or("triage omitted lines")?
                                .push(json!(format!("hang: {line}")));
                        }
                    }
                }
                fs::write(&path, data)?;
                tri["cpu_state"] = json!(path);
            }
            let silent = number(&diag, "silent_s", 0.0);
            let mut hint = if silent < 5.0 {
                tri["hang_kind"] = json!("slow");
                format!("serial STILL FLOWING at deadline (silent {silent:.0}s) — likely SLOW/CONTENDED host; retry before assuming a wedge")
            } else {
                tri["hang_kind"] = json!("wedged");
                format!("serial SILENT {silent:.0}s before deadline — likely genuinely WEDGED")
            };
            if flag(&diag, "early_hang") {
                hint.push_str(
                    " (early-hang abort — failed fast, skipped the rest of boot_timeout)",
                );
            }
            let last = text(&diag, "last_line");
            if !last.is_empty() {
                hint.push_str(&format!("; last serial: {last}"));
            }
            tri["salient"]
                .as_array_mut()
                .ok_or("triage omitted lines")?
                .insert(0, json!(format!("timeout: {hint}")));
        }
        if let Ok(note) = vm(host, "vm_report_replay_break", id, json!({})) {
            if let Some(s) = note.as_str().filter(|s| !s.is_empty()) {
                tri["salient"]
                    .as_array_mut()
                    .ok_or("triage omitted lines")?
                    .insert(0, json!(s));
            }
        }
        Ok(merge_failure(&outcome, tri, &shot))
    })();
    let keep = flag(a, "attach") && body.as_ref().is_ok_and(|r| r["ok"] != true);
    if keep {
        let run_id = if total == 1 {
            run.file_name().unwrap().to_string_lossy().into_owned()
        } else {
            format!("{}-boot{idx}", run.file_name().unwrap().to_string_lossy())
        };
        let owner_data = request(host, "debug_owner", json!({"arch":arch,"run_id":run_id}))?;
        let owner = owner_data["owner_pid"]
            .as_u64()
            .ok_or("debug authority omitted owner PID")?;
        match vm(
            host,
            "vm_detach",
            id,
            json!({"owner_pid":owner,"run_id":run_id}),
        ) {
            Ok(handles) => {
                let mut manifest = handles
                    .as_object()
                    .ok_or("debug handles must be an object")?
                    .clone();
                manifest.extend(json!({"run_id":run_id,"arch":arch,"backend":backend,
                    "serial_log":work.join("serial.log"),"kernel_image":owner_data["kernel_image"],
                    "owner_pid":owner,"source":"smoke --attach-on-fail",
                    "kernel_runtime_base":request(host,"verdict_kernel_runtime_base",json!({"serial":fs::read_to_string(work.join("serial.log")).unwrap_or_default()}))?
                }).as_object().unwrap().clone());
                let dir = root.join("qemu/runs").join(&run_id);
                fs::create_dir_all(&dir)?;
                fs::write(
                    dir.join("manifest.json"),
                    serde_json::to_vec_pretty(&manifest)?,
                )?;
                let hint=format!("VM KEPT for debugging: run-id {run_id} (owner pid {owner}). Stop it: ./vibeos-cli gdb stop {run_id}");
                log(host, &label, &hint)?;
                request(host, "print", json!({"text":hint,"stderr":true}))?;
                add(
                    host,
                    "attach",
                    "warn",
                    &format!("VM kept: {run_id}"),
                    json!([hint]),
                )?;
                let mut attach = handles.as_object().unwrap().clone();
                attach.extend(
                    json!({"kept":true,"run_id":run_id,"owner_pid":owner})
                        .as_object()
                        .unwrap()
                        .clone(),
                );
                request(host, "set_run", json!({"extra":{"attach":attach}}))?;
            }
            Err(error) => {
                log(
                    host,
                    &label,
                    format!("attach-on-fail: VM not kept: {}", error.message),
                )?;
                request(
                    host,
                    "set_run",
                    json!({"extra":{"attach":{"kept":false,"reason":error.message}}}),
                )?;
                vm(host, "vm_stop", id, json!({}))?;
            }
        }
    } else {
        let stopping = Instant::now();
        let cleanup = vm(host, "vm_stop", id, json!({}));
        timings.insert(
            "stop_seconds".into(),
            json!(rounded(stopping.elapsed().as_secs_f64())),
        );
        if let Err(error) = cleanup {
            return Err(Error::new(format!(
                "boot VM cleanup failed: {}",
                error.message
            )));
        }
    }
    let mut result = body?;
    timings.insert(
        "total_seconds".into(),
        json!(rounded(started.elapsed().as_secs_f64())),
    );
    result["timings"] = Value::Object(timings);
    let serial = work.join("serial.log");
    if serial.is_file() {
        result["serial_log"] = json!(serial);
    }
    Ok(result)
}

pub fn dispatch(root: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = r.get("kwargs").unwrap_or(r);
    match text(r, "op") {
        "smoke_one_boot" => one_boot(root, a, host),
        "smoke_watch_script" => {
            let mut state = WatchState::new(
                text(a, "until"),
                a["fail_on"].as_str(),
                number(a, "timeout", 300.0),
                number(a, "hang_silence", 0.0),
            )?;
            for row in a["chunks"].as_array().ok_or("missing watch chunks")? {
                if let Some(outcome) = state.feed(number(row, "time", 0.0), text(row, "text"))? {
                    return Ok(
                        json!({"outcome":outcome,"text":state.buffer,"diag":state.diagnostics(number(row,"time",0.0))}),
                    );
                }
            }
            Ok(json!({"outcome":"pending","text":state.buffer}))
        }
        _ => Err("unknown smoke operation".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_priority_and_chunk_spanning_patterns_match_python() {
        let mut state = WatchState::new("shell ready", Some(r"(?<=BUG)CHECK"), 5.0, 1.0).unwrap();
        assert!(state.feed(0.1, "BUG").unwrap().is_none());
        assert_eq!(
            state.feed(0.2, "CHECK shell ready").unwrap(),
            Some("reached")
        );
        let mut state = WatchState::new("shell ready", None, 50.0, 1.0).unwrap();
        assert!(state.feed(0.1, "booting").unwrap().is_none());
        assert_eq!(state.feed(2.0, "").unwrap(), Some("timeout"));
        assert_eq!(state.diagnostics(2.0)["early_hang"], true);
    }
}

// Append to smoke.rs after integrating the OSError watch propagation branch.
#[cfg(test)]
mod serial_log_failure_controls {
    use super::*;
    struct FailedSerial {
        kind: &'static str,
    }
    impl Host for FailedSerial {
        fn call(&mut self, op: &str, _args: Value) -> Result<Value> {
            assert_eq!(op, "vm_poll_serial");
            Err(Error::new("serial tee /dev/full: No space left on device").with_kind(self.kind))
        }
    }
    #[test]
    fn host_log_io_failure_is_not_graded_as_guest_death() {
        let error = watch(
            &mut FailedSerial { kind: "OSError" },
            1,
            "READY",
            None,
            1.,
            0.,
        )
        .unwrap_err();
        assert_eq!(error.kind.as_deref(), Some("OSError"));
        assert!(error.message.contains("/dev/full"));
    }
    #[test]
    fn requested_diagnostic_failure_is_not_graded_as_guest_death() {
        let error = watch(
            &mut FailedSerial {
                kind: "DiagnosticError",
            },
            1,
            "READY",
            None,
            1.,
            0.,
        )
        .unwrap_err();
        assert_eq!(error.kind.as_deref(), Some("DiagnosticError"));
    }
    #[test]
    fn ordinary_serial_closure_retains_guest_death_behavior() {
        let (outcome, _, diag) = watch(
            &mut FailedSerial {
                kind: "ConnectionError",
            },
            1,
            "READY",
            None,
            1.,
            0.,
        )
        .unwrap();
        assert_eq!(outcome, "dead");
        assert!(diag["serial_error"].as_str().is_some());
    }
}

#[cfg(test)]
mod effective_launch_config_controls {
    use super::*;
    #[test]
    fn watcher_uses_post_launch_configuration() {
        let root = tempfile::tempdir().unwrap();
        let result=crate::smoke_fixtures::dispatch(root.path(),&json!({"op":"smoke_script","kwargs":{
            "mode":"one_boot", "options":{"arch":"i386","run_dir":root.path()},
            "config":{"boot_marker":"BEFORE","boot_timeout":0.01,"hang_silence_seconds":0.0},
            "responses":{"vm_start":[{"value":{"config":{"boot_marker":"AFTER","boot_timeout":0.1,"hang_silence_seconds":0.0}}}]},
            "chunks":["AFTER\n"]
        }})).unwrap();
        assert_eq!(result["result"]["ok"], true, "{result}");
    }
}
