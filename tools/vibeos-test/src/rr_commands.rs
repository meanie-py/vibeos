//! Record/replay command composition. Execution and display are transport leaves;
//! trace decisions, refusal gates, finalisation and result policy live here.
use crate::{callback::Host, record_replay as rr, replay_sample, rr_cli, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
const PINNED: &[&str] = &[
    "VIBEOS_QEMU_SMP",
    "VIBEOS_QEMU_GIC",
    "VIBEOS_QEMU_SMMU",
    "VIBEOS_QEMU_USB_DISK",
    "VIBEOS_QEMU_NVME",
    "VIBEOS_QEMU_NVME_BRIDGE",
    "VIBEOS_QEMU_XHCI",
    "VIBEOS_QEMU_XHCI_MSI",
    "VIBEOS_QEMU_CPU",
    "VIBEOS_QEMU_HEADLESS",
    "VIBEOS_QEMU_BOOT_NVME",
    "VIBEOS_QEMU_EXTRA_ARGS",
    "VIBEOS_TRIPWIRE",
    "VIBEOS_UP_DEFAULT_BOOT",
    "VIBEOS_QEMU_BOOT_TRANSPORT",
];
fn s<'a>(v: &'a Value, k: &str, default: &'a str) -> &'a str {
    v[k].as_str().unwrap_or(default)
}
fn n(v: &Value, k: &str, default: u64) -> u64 {
    v[k].as_u64().unwrap_or(default)
}
fn show(host: &mut dyn Host, text: impl Into<String>, stderr: bool) -> Result<()> {
    host.call("print", json!({"text":text.into(),"stderr":stderr}))?;
    Ok(())
}
fn version() -> Result<String> {
    let v = vibeos_build::process::capture(
        &mut Command::new("qemu-system-aarch64").arg("--version"),
        30,
    )?;
    if v.code != 0 {
        return Err(format!("cannot read QEMU version: {}", v.stderr.trim()).into());
    }
    Ok(v.stdout.lines().next().unwrap_or("unknown").into())
}
pub fn boot_command(
    repo: &Path,
    arch: &str,
    payload: &str,
    smp: u64,
    build: bool,
    timeout: u64,
) -> Vec<String> {
    let mut cmd = vec![
        repo.join("vibeos-cli").to_string_lossy().into_owned(),
        if payload == "kmtest" {
            "conformance"
        } else {
            "smoke"
        }
        .into(),
        "--arch".into(),
        arch.into(),
        "--smp".into(),
        smp.to_string(),
    ];
    if payload == "kmtest" {
        cmd.extend(["--timeout".into(), timeout.to_string()]);
        if !build {
            cmd.push("--no-build".into())
        }
    } else {
        cmd.push("--no-net".into())
    }
    cmd
}
struct Context<'a> {
    repo: &'a Path,
    root: PathBuf,
    options: Value,
    env: BTreeMap<String, String>,
    host: &'a mut dyn Host,
}
impl Context<'_> {
    fn trace(&self) -> Result<PathBuf> {
        rr_cli::resolve_trace(&self.root, s(&self.options, "trace", ""))
    }
    fn execute(&mut self, cmd: &[String], env: &BTreeMap<String, String>) -> Result<i32> {
        self.host
            .call("rr_execute", json!({"argv":cmd,"cwd":self.repo,"env":env}))?
            .as_i64()
            .map(|v| v as i32)
            .ok_or_else(|| "RR executor omitted return code".into())
    }
    fn keep(&mut self, trace: &Path, name: &str, mode: &str) -> Result<Option<PathBuf>> {
        let v = rr_cli::keep_serial(trace, name, mode)?;
        if let Some(w) = v["warning"].as_str() {
            show(self.host, w, true)?;
        }
        Ok(v["path"].as_str().map(PathBuf::from))
    }
    fn record(&mut self) -> Result<i32> {
        let arch = s(&self.options, "arch", "").to_owned();
        if !matches!(
            arch.as_str(),
            "i386" | "i386-uefi" | "amd64" | "amd64-uefi" | "amd64-bios" | "arm64"
        ) {
            return Err("record requires an explicit valid architecture".into());
        }
        let payload = s(&self.options, "payload", "kmtest").to_owned();
        let name = self.options["name"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "{}-{arch}-{payload}",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                )
            });
        let trace = self.root.join(name);
        if trace.exists() {
            show(
                self.host,
                format!("[rr] trace already exists: {}", trace.display()),
                true,
            )?;
            return Ok(2);
        }
        if let Some(parent) = trace.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir(&trace)?;
        let mut env = self.env.clone();
        env.insert(
            "VIBEOS_QEMU_RR".into(),
            format!("record:{}", trace.display()),
        );
        let shift = n(&self.options, "shift", rr::DEFAULT_SHIFT);
        env.insert("VIBEOS_QEMU_RR_SHIFT".into(), shift.to_string());
        if shift != rr::DEFAULT_SHIFT {
            show(self.host,format!("[rr] WARNING: --shift {shift} is not the default ({}). REPLAY PROBABLY WILL NOT work. Measured on arm64: shift 1 and shift 5 stall at the first 4,096-byte buffer; shift 3 reaches desktop. See docs/engineering/qemu-record-replay.md.",rr::DEFAULT_SHIFT),true)?;
        }
        let smp = n(&self.options, "smp", 1);
        env.insert("VIBEOS_QEMU_SMP".into(), smp.to_string());
        if let Some(period) = self.options["sample"].as_f64().filter(|p| *p > 0.) {
            env.insert("VIBEOS_QEMU_RR_SAMPLE".into(), period.to_string());
        }
        env.insert("VIBEOS_QEMU_BACKEND".into(), "native".into());
        let cmd = boot_command(
            self.repo,
            &arch,
            &payload,
            smp,
            self.options["build"] == true,
            n(&self.options, "timeout", 6000),
        );
        show(
            self.host,
            format!(
                "[rr] recording -> {}\n[rr] boot: {}",
                trace.display(),
                cmd.join(" ")
            ),
            false,
        )?;
        let started = Instant::now();
        let rc = self.execute(&cmd, &env)?;
        let elapsed = started.elapsed().as_secs_f64();
        let size = rr_cli::await_quiescence(&trace, 6., 420.)?;
        // The trace is useful even when its guest bugchecked. Only quiescent,
        // non-empty traces can publish pristine media and the finalisation flag.
        let finalised = size > 0;
        if finalised {
            rr::snapshot_pristine(&trace)?;
            self.keep(&trace, "record-serial.log", "record")?;
        }
        let path = trace.join("manifest.json");
        let mut meta = if path.is_file() {
            rr::load_manifest(&trace)?
        } else {
            json!({})
        };
        let head = vibeos_build::process::capture(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(self.repo),
            30,
        )?;
        let commit = if head.code == 0 {
            head.stdout.trim()
        } else {
            "unknown"
        };
        let mut pinned: BTreeMap<String, String> = PINNED
            .iter()
            .filter_map(|k| {
                self.env
                    .get(*k)
                    .filter(|v| !v.is_empty())
                    .map(|v| ((*k).into(), v.clone()))
            })
            .collect();
        pinned.insert("VIBEOS_QEMU_SMP".into(), smp.to_string());
        meta.as_object_mut().ok_or("invalid trace manifest")?.extend(json!({"boot_cmd":cmd,"boot_rc":rc,"record_seconds":(elapsed*10.).round()/10.,"payload":payload,"commit":commit,"finalised":finalised,"env":pinned}).as_object().unwrap().clone());
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&meta)?))?;
        show(
            self.host,
            format!(
                "[rr] boot rc={rc} in {elapsed:.0}s; trace {:.2}MB at {}",
                size as f64 / 1e6,
                trace.display()
            ),
            false,
        )?;
        if size == 0 {
            show(
                self.host,
                if meta.get("argv").is_none() {
                    "[rr] NO TRACE WRITTEN — the boot refused before launching QEMU; see its error above."
                } else {
                    "[rr] NO TRACE WRITTEN — QEMU did not flush a complete recording. Let the boot end on its own or via QMP quit."
                },
                true,
            )?;
            return Ok(1);
        }
        let diagnostics = rr_cli::validate_record_diagnostics(
            &trace,
            env.get("VIBEOS_QEMU_RR_SAMPLE")
                .is_some_and(|s| !s.trim().is_empty()),
            env.get("VIBEOS_QEMU_PC_SAMPLE")
                .is_some_and(|s| !s.trim().is_empty()),
        )?;
        if diagnostics["ok"] != true {
            show(
                self.host,
                format!(
                    "[rr] requested diagnostics failed: {}; trace retained at {}",
                    diagnostics["error"],
                    trace.display()
                ),
                true,
            )?;
            return Ok(2);
        }
        show(
            self.host,
            format!(
                "[rr] replay it: ./vibeos-cli rr replay {}",
                trace.file_name().unwrap().to_string_lossy()
            ),
            false,
        )?;
        Ok(0)
    }
    fn replay(&mut self) -> Result<i32> {
        let trace = self.trace()?;
        let m = rr::load_manifest(&trace)?;
        let problems = rr::check_trace(&trace, &version()?)?;
        let mut fatal = false;
        for p in &problems {
            let refused = p.contains(rr::FATAL_MARKER);
            fatal |= refused;
            show(
                self.host,
                format!("[rr] {}: {p}", if refused { "REFUSING" } else { "WARNING" }),
                true,
            )?;
        }
        if fatal {
            show(
                self.host,
                "[rr] not replaying — re-record with `./vibeos-cli rr record`.",
                true,
            )?;
            return Ok(2);
        }
        rr::prepare_replay(&trace)?;
        let mut env = self.env.clone();
        env.insert(
            "VIBEOS_QEMU_RR".into(),
            format!("replay:{}", trace.display()),
        );
        env.insert(
            "VIBEOS_QEMU_RR_SHIFT".into(),
            n(&m, "icount_shift", rr::DEFAULT_SHIFT).to_string(),
        );
        env.insert("VIBEOS_QEMU_BACKEND".into(), "native".into());
        if let Some(pinned) = m["env"].as_object() {
            for (k, v) in pinned {
                env.insert(
                    k.clone(),
                    v.as_str()
                        .ok_or("invalid pinned replay environment")?
                        .into(),
                );
            }
        }
        if let Some(ic) = self.options["break_at"].as_u64() {
            env.insert("VIBEOS_QEMU_RR_BREAK".into(), ic.to_string());
        }
        if self.options["gdb"] == true {
            env.insert("VIBEOS_QEMU_GDB".into(), "wait".into());
            show(self.host,"[rr] NOTE: do NOT stop a replay from the debugger. stepi, attaching mid-run and even a breakpoint HIT abort QEMU with 'Playback shouldn't have to iowait'. Use QMP replay-break (`--break-at <icount>`) at a legal instruction boundary, THEN attach gdb to the already-paused guest.",true)?;
        }
        let mut cmd: Vec<String> = if m["boot_cmd"].is_array() {
            serde_json::from_value(m["boot_cmd"].clone())?
        } else {
            boot_command(
                self.repo,
                s(&m, "arch", ""),
                s(&m, "payload", "kmtest"),
                1,
                false,
                6000,
            )
        };
        cmd.retain(|a| a != "--build");
        if cmd.iter().any(|a| a == "conformance") && !cmd.iter().any(|a| a == "--no-build") {
            cmd.push("--no-build".into());
        }
        if cmd.iter().any(|a| a == "smoke") && !cmd.iter().any(|a| a == "--allow-stale") {
            cmd.push("--allow-stale".into());
        }
        // The preceding check_trace authorizes the TRACE's backing media. This bypasses
        // only the unrelated live-tree build stamp, never the trace's input guards.
        show(
            self.host,
            format!(
                "[rr] replaying {} (recorded {})\n[rr] boot: {}",
                trace.file_name().unwrap().to_string_lossy(),
                m["created"],
                cmd.join(" ")
            ),
            false,
        )?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let observed = trace.clone();
        let progress = thread::spawn(move || {
            let mut p = rr_cli::Progress {
                total: fs::metadata(observed.join("replay.bin")).map_or(0, |v| v.len()),
                ..Default::default()
            };
            let mut last = None;
            let mut next = Instant::now() + Duration::from_secs(20);
            while !stopped.load(Ordering::Acquire) {
                if Instant::now() >= next {
                    if let Some(pos) = rr::trace_position(&observed) {
                        p.max_pos = p.max_pos.max(pos);
                        p.stalled_for = if last == Some(pos) {
                            p.stalled_for + 20.
                        } else {
                            0.
                        };
                        last = Some(pos);
                    }
                    next = Instant::now() + Duration::from_secs(20);
                }
                thread::sleep(Duration::from_millis(100));
            }
            p
        });
        let executed = self.execute(&cmd, &env);
        stop.store(true, Ordering::Release);
        let p = progress
            .join()
            .map_err(|_| Error::new("trace progress worker failed"))?;
        let rc = executed?;
        show(self.host, format!("[rr] replay rc={rc}"), false)?;
        if let Some(note) = rr_cli::progress_report(&p) {
            show(self.host, format!("[rr] {note}"), false)?;
        }
        Ok(rc)
    }
    fn verify(&mut self) -> Result<i32> {
        let trace = self.trace()?;
        let recorded = trace.join("record-serial.log");
        if !recorded.is_file() {
            show(
                self.host,
                format!(
                    "[rr] {} has no record-serial.log — re-record before verifying.",
                    trace.display()
                ),
                true,
            )?;
            return Ok(2);
        }
        let rc = self.replay()?;
        let Some(replayed) = self.keep(&trace, "replay-serial.log", "replay")? else {
            show(
                self.host,
                "[rr] the replay produced no serial transcript",
                true,
            )?;
            return Ok(2);
        };
        let a = rr_cli::guest_lines(&recorded)?;
        let b = rr_cli::guest_lines(&replayed)?;
        let div = rr_cli::serial_divergence(&a, &b);
        if let Some((line, detail)) = div {
            show(
                self.host,
                format!(
                    "[rr] DIVERGED at serial line {line}:\n{detail}\n[rr] Transcripts: {} and {}",
                    recorded.display(),
                    replayed.display()
                ),
                false,
            )?;
            return Ok(1);
        }
        if a.len() == b.len() {
            show(
                self.host,
                format!(
                    "[rr] VERIFIED — {} lines of guest serial, identical to the recording",
                    a.len()
                ),
                false,
            )?;
            return Ok(0);
        }
        show(
            self.host,
            format!(
                "[rr] PARTIAL — agreed for {} lines with no divergence; record has {}, replay {}.",
                a.len().min(b.len()),
                a.len(),
                b.len()
            ),
            false,
        )?;
        if b.len() < a.len() {
            show(
                self.host,
                "[rr] The REPLAY stopped short, so the rest of the boot is unproven.",
                false,
            )?;
            Ok(if rc == 0 { 1 } else { rc })
        } else {
            Ok(rc)
        }
    }
    fn diverge(&mut self) -> Result<i32> {
        let trace = self.trace()?;
        let samples = trace.join(replay_sample::SAMPLES_FILE);
        if !samples.is_file() || fs::read_to_string(samples)?.trim().is_empty() {
            show(
                self.host,
                format!(
                    "[rr] {} has no fingerprints; re-record with `rr record ... --sample 10`.",
                    trace.display()
                ),
                true,
            )?;
            return Ok(2);
        }
        let path = trace.join(replay_sample::VERDICT_FILE);
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        };
        self.env
            .insert("VIBEOS_QEMU_RR_SAMPLE".into(), "check".into());
        self.options["gdb"] = json!(false);
        self.options["break_at"] = Value::Null;
        let rc = self.replay()?;
        if !path.is_file() {
            show(
                self.host,
                format!("[rr] diverge: no verdict written (replay rc={rc})"),
                true,
            )?;
            return Ok(1);
        }
        let v: Value = serde_json::from_slice(&fs::read(&path)?)?;
        show(
            self.host,
            format!(
                "[rr] diverge: compared {} of {} fingerprints",
                v["compared"], v["samples"]
            ),
            false,
        )?;
        for (key, label) in [("skipped", "skipped"), ("inconclusive", "inconclusive")] {
            if let Some(rows) = v[key].as_array() {
                for row in rows {
                    show(
                        self.host,
                        format!("[rr]   {label} icount {}: {}", row["icount"], row["why"]),
                        false,
                    )?;
                }
            }
        }
        show(
            self.host,
            format!("[rr]   last match: {}", v["last_match"]),
            false,
        )?;
        if let Some(mm) = v.get("first_mismatch").filter(|v| !v.is_null()) {
            show(
                self.host,
                format!(
                    "[rr]   FIRST MISMATCH: icount {}\n{}\n[rr]   full detail: {}",
                    mm["icount"],
                    serde_json::to_string_pretty(mm)?,
                    path.display()
                ),
                false,
            )?;
            return Ok(1);
        }
        if let Some(error) = v.get("error").filter(|v| !v.is_null()) {
            show(self.host, format!("[rr]   ERROR: {error}"), false)?;
            if let Some(regs) = v["wedge_regs"].as_str() {
                show(
                    self.host,
                    regs.lines().take(4).collect::<Vec<_>>().join("\n"),
                    false,
                )?;
            }
            return Ok(1);
        }
        let status = replay_sample::comparison_status(&v)?;
        show(
            self.host,
            format!("[rr]   {}", s(&status, "message", "")),
            false,
        )?;
        Ok(n(&status, "exit", 2) as i32)
    }
    fn list(&mut self) -> Result<i32> {
        let rows = rr_cli::inventory(&self.root)?;
        if rows.is_empty() {
            show(
                self.host,
                format!("no traces under {}", self.root.display()),
                false,
            )?;
            return Ok(0);
        }
        show(
            self.host,
            format!(
                "{:<40} {:<6} {:<8} {:<4} {:<6} {:<7} created",
                "trace", "arch", "payload", "rc", "rr", "total"
            ),
            false,
        )?;
        for row in rows {
            let m = &row["manifest"];
            show(
                self.host,
                format!(
                    "{:<40} {:<6} {:<8} {:<4} {:<6.0} {:<7.0} {}",
                    s(&row, "name", "?"),
                    s(m, "arch", "?"),
                    s(m, "payload", "?"),
                    m["boot_rc"],
                    n(&row, "bytes", 0) as f64 / 1e6,
                    n(&row, "total", 0) as f64 / 1e6,
                    m["created"]
                ),
                false,
            )?;
        }
        Ok(0)
    }
    fn prune(&mut self) -> Result<i32> {
        let names: Vec<String> = serde_json::from_value(
            self.options
                .get("name")
                .filter(|v| !v.is_null())
                .cloned()
                .unwrap_or(json!([])),
        )?;
        let keep = self.options["keep"].as_i64().unwrap_or(3).max(0) as usize;
        let apply = self.options["apply"] == true;
        let result = rr_cli::prune(&self.root, keep, &names, apply)?;
        let doomed = result["doomed"].as_array().unwrap();
        if doomed.is_empty() {
            show(
                self.host,
                format!(
                    "{} trace(s), keeping {keep} — nothing to prune",
                    result["count"]
                ),
                false,
            )?;
            return Ok(0);
        }
        for row in doomed {
            show(
                self.host,
                format!(
                    "{} {:<34} {:8.0}MB",
                    if apply { "DELETE" } else { "would delete" },
                    s(row, "name", ""),
                    n(row, "total", 0) as f64 / 1e6
                ),
                false,
            )?;
        }
        show(
            self.host,
            format!(
                "{} {:.2}GB across {} trace(s)",
                if apply { "freed" } else { "would free" },
                n(&result, "bytes", 0) as f64 / 1e9,
                doomed.len()
            ),
            false,
        )?;
        if !apply {
            show(self.host, "re-run with --apply to actually delete", false)?;
        }
        Ok(0)
    }
    fn info(&mut self) -> Result<i32> {
        let trace = self.trace()?;
        let m = rr::load_manifest(&trace)?;
        show(self.host, format!("trace:   {}", trace.display()), false)?;
        for key in [
            "arch",
            "payload",
            "created",
            "commit",
            "qemu_version",
            "icount_shift",
            "boot_rc",
            "record_seconds",
        ] {
            if let Some(value) = m.get(key) {
                let rendered = value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string());
                show(
                    self.host,
                    format!("{:<16} {rendered}", format!("{key}:")),
                    false,
                )?;
            }
        }
        show(
            self.host,
            format!(
                "{:<16} {}",
                "env:",
                m.get("env").cloned().unwrap_or(json!({}))
            ),
            false,
        )?;
        show(self.host, format!("{:<16}", "media:"), false)?;
        if let Some(media) = m["media"].as_object() {
            for (name, info) in media {
                let source = info.as_str().unwrap_or_else(|| s(info, "source", ""));
                let how = if info.get("copied") == Some(&json!(true)) {
                    "copy of"
                } else {
                    "overlay over"
                };
                show(self.host, format!("    {name:<24} {how} {source}"), false)?;
            }
        }
        let warnings = rr::check_trace(&trace, &version()?)?;
        if warnings.is_empty() {
            show(
                self.host,
                "warnings:        none — trace guards passed",
                false,
            )?;
        } else {
            show(
                self.host,
                format!("warnings:\n    {}", warnings.join("\n    ")),
                false,
            )?;
        }
        Ok(0)
    }
    fn breaks(&mut self) -> Result<i32> {
        let trace = self.trace()?;
        let bounds = rr::boundaries(&trace)?;
        if bounds.is_empty() {
            show(
                self.host,
                format!(
                    "{} records no instruction chunks — there is nowhere to break",
                    trace.display()
                ),
                false,
            )?;
            return Ok(1);
        }
        if let Some(near) = self.options["near"].as_u64() {
            let i = bounds.partition_point(|n| *n < near);
            let exact = bounds.get(i) == Some(&near);
            let count = n(&self.options, "count", 5) as usize;
            for b in &bounds[i.saturating_sub(count)
                ..bounds.len().min(i.saturating_add(count).saturating_add(1))]
            {
                let mark = if *b == near {
                    "   <- requested (legal)"
                } else if !exact && bounds.get(i) == Some(b) {
                    "   <- snaps here"
                } else {
                    ""
                };
                show(self.host, format!("  {b}{mark}"), false)?;
            }
            if exact {
                show(
                    self.host,
                    format!("\n{near} is a legal break icount."),
                    false,
                )?;
            } else if i == bounds.len() {
                show(self.host,format!("\n{near} is past the end of the recording (last boundary {}) — a break there would never fire.",bounds.last().unwrap()),false)?;
                return Ok(1);
            } else {
                let prev = if i == 0 { 0 } else { bounds[i - 1] };
                show(self.host,format!("\n{near} is NOT legal: it sits {} instructions into a {}-instruction chunk, leaving {}. Breaking there aborts the replay. Use {}.",near-prev,bounds[i]-prev,bounds[i]-near,bounds[i]),false)?;
            }
        } else {
            let mut gaps: Vec<_> = bounds.windows(2).map(|w| w[1] - w[0]).collect();
            gaps.sort_unstable();
            show(
                self.host,
                format!(
                    "trace: {}\nboundaries: {}\nicount range: {} .. {}",
                    trace.display(),
                    bounds.len(),
                    bounds[0],
                    bounds.last().unwrap()
                ),
                false,
            )?;
            if !gaps.is_empty() {
                show(
                    self.host,
                    format!(
                        "chunk size: min {} median {} max {}",
                        gaps[0],
                        gaps[gaps.len() / 2],
                        gaps.last().unwrap()
                    ),
                    false,
                )?;
            }
            show(self.host,"A replay-break must land on one of these instruction boundaries. Use --near <icount> to pick one.",false)?;
        }
        Ok(0)
    }
    fn wedge(&mut self) -> Result<i32> {
        let trace = self.trace()?;
        let mut offset = self.options["at"].as_u64();
        let mut source = "--at".to_owned();
        if let Some(ic) = self.options["icount"].as_u64() {
            offset = None;
            rr::walk(&trace, false, |e| {
                if e.icount >= ic {
                    offset = Some(e.offset);
                    Ok(false)
                } else {
                    Ok(true)
                }
            })?;
            if offset.is_none() {
                show(
                    self.host,
                    format!("icount {ic} is past the last event in {}", trace.display()),
                    true,
                )?;
                return Ok(1);
            }
            source = format!("--icount {ic}");
        }
        if offset.is_none() {
            offset = rr::trace_position(&trace);
            source = "live reader".into();
        }
        let Some(offset) = offset else {
            show(self.host,format!("nobody has {}/replay.bin open; pass --at <offset> to inspect a recorded position.",trace.display()),true)?;
            return Ok(1);
        };
        let size = fs::metadata(trace.join("replay.bin"))?.len();
        let count = n(&self.options, "count", 8) as usize;
        let (before, after) = rr::around(&trace, offset, count, count)?;
        show(
            self.host,
            format!(
                "trace: {}\noffset: {offset} of {size} ({:.2}%, from {source})",
                trace.display(),
                if size == 0 {
                    0.
                } else {
                    100. * offset as f64 / size as f64
                }
            ),
            false,
        )?;
        if before.is_empty() && after.is_empty() {
            show(self.host, "no events decode around that offset", false)?;
            return Ok(1);
        }
        for (label, rows) in [
            ("BEFORE (the stall is at or before one of these)", before),
            ("AFTER (context; not yet acted on)", after),
        ] {
            show(self.host, format!("\n{label}:"), false)?;
            for e in rows {
                show(
                    self.host,
                    format!(
                        "  @{:<10} icount {:<14} {}",
                        e.offset,
                        e.icount,
                        rr::event_name(e.eid)
                    ),
                    false,
                )?;
            }
        }
        show(self.host,"\nThe offset is a BUFFERED kernel read position, so it runs ahead of what\nthe replay has acted on. Treat BEFORE as the evidence.",false)?;
        Ok(0)
    }
}
pub fn dispatch(a: &Value, host: &mut dyn Host) -> Result<Value> {
    if let Some(options) = a.get("args") {
        for key in [
            "smp", "shift", "timeout", "count", "near", "break_at", "icount", "at",
        ] {
            if options
                .get(key)
                .is_some_and(|value| !value.is_null() && value.as_u64().is_none())
            {
                return Err(format!("RR {key} requires a nonnegative integer").into());
            }
        }
        if options
            .get("sample")
            .is_some_and(|v| !v.is_null() && !v.as_f64().is_some_and(|n| n.is_finite() && n > 0.))
        {
            return Err("RR sample period must be finite and positive".into());
        }
    }

    if a["action"] == "boot_command" {
        return Ok(json!(boot_command(
            Path::new(s(a, "repo", "")),
            s(a, "arch", ""),
            s(a, "payload", "kmtest"),
            n(a, "smp", 1),
            a["build"] == true,
            n(a, "timeout", 6000)
        )));
    }
    let mut c = Context {
        repo: Path::new(s(a, "repo", "")),
        root: PathBuf::from(s(a, "trace_root", "")),
        options: a.get("args").cloned().unwrap_or(json!({})),
        env: serde_json::from_value(a.get("env").cloned().unwrap_or(json!({})))?,
        host,
    };
    let code = match s(a, "verb", "") {
        "record" => c.record(),
        "replay" => c.replay(),
        "verify" => c.verify(),
        "diverge" => c.diverge(),
        "list" => c.list(),
        "prune" => c.prune(),
        "info" => c.info(),
        "breaks" => c.breaks(),
        "wedge" => c.wedge(),
        other => Err(format!("unknown rr verb: {other}").into()),
    }?;
    Ok(json!(code))
}
