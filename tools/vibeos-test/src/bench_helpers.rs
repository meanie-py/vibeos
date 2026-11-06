//! Pure board-controller helpers. Device actions remain in tools/pi{4,5}.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

pub const MARKER: &str = "shell ready";
pub const EXPLORER_MARKER: &str = "[EXPLORER] _tWinMain entered";
pub fn desktop_reached(text: &str) -> bool {
    // Win32k emits shell ready before Explorer starts. Both are needed to
    // establish the desktop; a crash in the same slice always takes precedence.
    text.contains(MARKER)
        && text.contains(EXPLORER_MARKER)
        && !text.contains("*** BUGCHECK")
        && !text.contains("panicked at")
}
pub const PI_WATTS_MAX: f64 = 20.0;
pub const STORTEST_MARKER: &[u8] = b"vibeos usb e2e marker\r\n";
pub const STORTEST_WROTE: &[u8] = b"vibeos wrote this through fastfat+usbstor\r\n";
pub const HARD_ERROR_INLINE: &str = "completed inline: the requester has APCs disabled";

pub fn parse_usb(text: &str) -> Value {
    let line = text.trim().lines().last().unwrap_or("");
    if line.starts_with('{') {
        return json!({"error": serde_json::from_str::<Value>(line).ok().and_then(|v| v.get("error").cloned()).unwrap_or(json!("unknown"))});
    }
    let mut out = Map::new();
    let re = Regex::new(r"(\w+)=(\[[^\]]*\]|\S+)").unwrap();
    for c in re.captures_iter(line) {
        let v = match &c[2] {
            "True" => json!(true),
            "False" => json!(false),
            s => json!(s),
        };
        out.insert(c[1].into(), v);
    }
    if out.contains_key("mounted") {
        Value::Object(out)
    } else {
        json!({"error":if line.is_empty(){"no output"}else{line}})
    }
}
pub fn parse_power(text: &str) -> Value {
    let re = Regex::new(r"\b(on|off)\s+([\d.]+)\s*W").unwrap();
    match re.captures(text).and_then(|c| {
        c[2].parse::<f64>()
            .ok()
            .map(|n| json!({"on":&c[1]=="on","watts":n}))
    }) {
        Some(v) => v,
        None => json!({}),
    }
}
pub fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok())
        .unwrap_or(json!({}))
}
pub fn serial_file(state: &Value) -> Option<PathBuf> {
    let dir = state.get("directory")?.as_str()?;
    if dir.is_empty() {
        return None;
    }
    let dir = if dir.starts_with('/') {
        dir.to_owned()
    } else {
        dir.replace("D:\\home", "/mnt/d/home").replace('\\', "/")
    };
    Some(PathBuf::from(dir).join("serial.bin"))
}
pub fn pid_runs(pid: &Value, needle: &str) -> bool {
    let pid = pid.as_u64().or_else(|| pid.as_str()?.parse().ok());
    pid.and_then(|p| fs::read(format!("/proc/{p}/cmdline")).ok())
        .is_some_and(|s| String::from_utf8_lossy(&s).contains(needle))
}
pub fn serial_slice(path: &Path, start: u64) -> String {
    let mut bytes = Vec::new();
    if let Ok(mut f) = fs::File::open(path) {
        if f.seek(SeekFrom::Start(start)).is_ok() {
            let _ = f.read_to_end(&mut bytes);
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
pub fn wait_desktop(path: &Path, start: u64, timeout: f64) -> (String, bool) {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout.max(0.0));
    loop {
        let text = serial_slice(path, start);
        if desktop_reached(&text) {
            return (text, true);
        }
        if text.contains("*** BUGCHECK") || text.contains("panicked at") {
            thread::sleep(Duration::from_secs(3));
            return (serial_slice(path, start), false);
        }
        if Instant::now() > deadline {
            return (text, false);
        }
        thread::sleep(Duration::from_secs(2));
    }
}
pub fn stortest_verdict(text: &str) -> (Option<&'static str>, Vec<String>) {
    let lines: Vec<_> = text
        .lines()
        .filter(|l| l.contains("[STORTEST]"))
        .map(|l| l.trim().to_owned())
        .collect();
    let verdict = if lines.iter().any(|l| l.contains("[STORTEST] ALL OK")) {
        Some("ok")
    } else if lines
        .iter()
        .any(|l| l.contains("[STORTEST] FAILURES") || l.contains("no VIBEUSB volume"))
    {
        Some("fail")
    } else {
        None
    };
    (verdict, lines)
}
pub fn drop_marker(text: &str) -> bool {
    Regex::new(r"PiEnumerateDevice USBSTOR\\Disk&Ven_PICOUSB_[^\n]*: 1 children")
        .unwrap()
        .is_match(text)
}
pub fn desktop_share(path: &Path) -> Option<f64> {
    let mut decoder = png::Decoder::new(fs::File::open(path).ok()?);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut data).ok()?;
    let (w, h) = (info.width, info.height);
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return None,
    };
    if w == 0 || h == 0 {
        return None;
    }
    // Match the original 192x108 sampling; nearest pixels avoid another image dependency.
    let mut near = 0usize;
    for y in 0..108 {
        for x in 0..192 {
            let i = (((y * h as usize / 108) * w as usize) + x * w as usize / 192) * channels;
            if (data[i] as i32 - 50).abs() < 20
                && (data[i + 1] as i32 - 121).abs() < 20
                && (data[i + 2] as i32 - 187).abs() < 25
            {
                near += 1;
            }
        }
    }
    Some(near as f64 / (192.0 * 108.0))
}
pub fn stortest_image(path: &Path) -> Result<()> {
    fs::File::create(path)?.set_len(32 << 20)?;
    let marker = path.with_file_name("MARKER.TXT");
    fs::write(&marker, STORTEST_MARKER)?;
    for (cmd, args) in [
        (
            "mkfs.fat",
            vec![
                "-n".into(),
                "VIBEUSB".into(),
                path.to_string_lossy().into_owned(),
            ],
        ),
        (
            "mcopy",
            vec![
                "-i".into(),
                path.to_string_lossy().into_owned(),
                marker.to_string_lossy().into_owned(),
                "::MARKER.TXT".into(),
            ],
        ),
    ] {
        let mut command = Command::new(cmd);
        command.args(args);
        let output = vibeos_build::artifact_executor::supervised_output(
            &mut command,
            Some(Duration::from_secs(30)),
        )?;
        if output.code != 0 {
            return Err(Error::new(format!("{cmd} failed: {}", output.stderr)));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn device_parsing_preserves_identity_data() {
        let usb = parse_usb(
            "uptime=5s mounted=True suspended=False target_powered=False completed=[3, 0, 0]",
        );
        assert_eq!(usb["target_powered"], false);
        assert_eq!(usb["completed"], "[3, 0, 0]");
        assert_eq!(parse_power("on  4.0 W"), json!({"on":true,"watts":4.0}));
        assert_eq!(parse_usb(""), json!({"error":"no output"}));
    }
    #[test]
    fn only_pico_partition_triggers_drop() {
        assert!(drop_marker(
            r"PiEnumerateDevice USBSTOR\Disk&Ven_PICOUSB_&Prod_REMOTE_DISK: 1 children"
        ));
        assert!(!drop_marker(
            r"PiEnumerateDevice USBSTOR\Disk&Ven_Generic_: 1 children"
        ));
        assert!(!drop_marker(
            r"PiEnumerateDevice USBSTOR\Disk&Ven_PICOUSB_: 0 children"
        ));
    }
    #[test]
    fn desktop_needs_explorer_and_shell_without_a_crash() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("serial.bin");
        for (text, expected) in [
            ("[WIN32K] shell ready", false),
            (EXPLORER_MARKER, false),
            ("[WIN32K] shell ready\n[EXPLORER] _tWinMain entered", true),
            (
                "[EXPLORER] _tWinMain entered\nshell ready\n*** BUGCHECK: 0x1E",
                false,
            ),
            (
                "shell ready\n[EXPLORER] _tWinMain entered\npanicked at bounds check",
                false,
            ),
        ] {
            assert_eq!(desktop_reached(text), expected);
        }
        fs::write(&p, "[WIN32K] shell ready").unwrap();
        assert!(!wait_desktop(&p, 0, 0.0).1);
        assert!(!wait_desktop_with_medium(&p, 0, 0.0, None, false).unwrap().1);
        fs::write(&p, "shell ready\n[EXPLORER] _tWinMain entered").unwrap();
        assert!(wait_desktop(&p, 0, 0.0).1);
        assert!(wait_desktop_with_medium(&p, 0, 0.0, None, false).unwrap().1);
        assert!(!wait_desktop(&p, fs::metadata(&p).unwrap().len(), 0.0).1);
    }
    #[test]
    fn serial_and_verdict_are_boot_scoped() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("serial.bin");
        fs::write(&p, b"old shell ready\n[STORTEST] ALL OK\n").unwrap();
        assert!(!wait_desktop(&p, fs::metadata(&p).unwrap().len(), 0.0).1);
        assert_eq!(
            stortest_verdict("[STORTEST] FAIL: no VIBEUSB volume found").0,
            Some("fail")
        );
    }
}

/// The only process stopped by a medium drop is the child we started.
/// Its child handle remains owned until reaped, preventing PID reuse races.
pub struct ImageServer {
    child: std::process::Child,
    pub log: PathBuf,
    started: Instant,
    pub lost: Option<(i32, f64)>,
    pub dropped: bool,
}
impl ImageServer {
    pub fn start(
        repo: &Path,
        image: &Path,
        log: &Path,
        env: &std::collections::BTreeMap<String, String>,
    ) -> Result<Self> {
        let output = fs::File::create(log)?;
        let child = Command::new("bash")
            .arg(repo.join("tools/pi5/pico.sh"))
            .args(["serve-image", &image.to_string_lossy(), "--writable"])
            .current_dir(repo)
            .env_clear()
            .envs(env)
            .stdout(output.try_clone()?)
            .stderr(output)
            .spawn()
            .map_err(|e| Error::new(format!("start Pico image server: {e}")))?;
        Ok(Self {
            child,
            log: log.into(),
            started: Instant::now(),
            lost: None,
            dropped: false,
        })
    }
    pub fn poll(&mut self) -> Result<bool> {
        if self.lost.is_none() {
            if let Some(status) = self.child.try_wait()? {
                self.lost = Some((
                    status.code().unwrap_or(-1),
                    self.started.elapsed().as_secs_f64(),
                ));
            }
        }
        Ok(self.lost.is_some())
    }
    pub fn drop_medium(&mut self) -> Result<()> {
        if self.poll()? {
            return Err("image server exited before requested medium drop".into());
        }
        // The child has not exited and has not been reaped, so this PID cannot be reused.
        if unsafe { libc::kill(self.child.id() as i32, libc::SIGTERM) } != 0 {
            return Err(Error::new(format!(
                "terminate image server: {}",
                std::io::Error::last_os_error()
            )));
        }
        self.dropped = true;
        Ok(())
    }
    pub fn stop(&mut self) -> Result<()> {
        if self.poll()? {
            return Ok(());
        }
        if unsafe { libc::kill(self.child.id() as i32, libc::SIGTERM) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) || !self.poll()? {
                return Err(Error::new(format!("terminate image server: {error}")));
            }
            return Ok(());
        }
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if self.poll()? {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        self.child.kill()?;
        self.child.wait()?;
        Ok(())
    }
}
impl Drop for ImageServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Grade the medium drop without conflating a lost server with a kernel result.
pub fn grade_drop(text: &str, dropped: bool, server_lost: bool) -> Value {
    let crash = crate::verdict::post_boot_check(text, "", &crate::verdict::known_flakes());
    if !crash.is_null() {
        return json!({"classification":crash["classification"],"stage":"drop-medium","status":"fail","detail":"guest crashed during medium removal","salient":crash["salient"]});
    }
    let (verdict, lines) = stortest_verdict(text);
    let tail: Vec<_> = lines
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .cloned()
        .collect();
    if server_lost && !dropped {
        return json!({"classification":"BENCH_UNHEALTHY", "stage":"serve-image", "status":"fail", "detail":"image server exited before the requested drop", "salient":tail});
    }
    if !dropped {
        return json!({"classification":"HANG_TIMEOUT", "stage":"drop-medium", "status":"fail", "detail":"the Pico's disk never reported its partition: the medium was never dropped", "salient":tail});
    }
    if verdict.is_none() {
        return json!({"classification":"HANG_TIMEOUT", "stage":"drop-medium", "status":"fail", "detail":"no stortest verdict 240s after the medium went", "salient":tail});
    }
    match text.lines().find(|line| line.contains(HARD_ERROR_INLINE)) {
        None => {
            json!({"classification":"NOT_EXERCISED", "stage":"drop-medium", "status":"fail", "detail":"stortest finished but no hard error completed inline", "salient":tail})
        }
        Some(line) => {
            json!({"classification":"", "stage":"drop-medium", "status":"ok", "detail":format!("{}; stortest: {}", line.trim().chars().take(120).collect::<String>(), lines.last().map(|s| s.chars().take(80).collect::<String>()).unwrap_or_default()), "salient":[]})
        }
    }
}

#[cfg(test)]
mod drop_tests {
    use super::*;
    #[test]
    fn grade_requires_drop_verdict_and_inline_completion() {
        let verdict = "[STORTEST] FAIL: no VIBEUSB volume found\n";
        assert_eq!(
            grade_drop(verdict, false, true)["classification"],
            "BENCH_UNHEALTHY"
        );
        assert_eq!(
            grade_drop(verdict, false, false)["classification"],
            "HANG_TIMEOUT"
        );
        assert_eq!(
            grade_drop(verdict, true, false)["classification"],
            "NOT_EXERCISED"
        );
        assert_eq!(
            grade_drop(&format!("{HARD_ERROR_INLINE}\n{verdict}"), true, false)["classification"],
            ""
        );
    }
}

/// Wait for the desktop while watching the exact owned medium server. This poll
/// starts before the desktop, so the requested drop cannot miss the partition line.
pub fn wait_desktop_with_medium(
    path: &Path,
    start: u64,
    timeout: f64,
    mut medium: Option<&mut ImageServer>,
    request_drop: bool,
) -> Result<(String, bool)> {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout.max(0.0));
    loop {
        let text = serial_slice(path, start);
        if let Some(server) = medium.as_deref_mut() {
            if server.poll()? && !server.dropped {
                return Err(
                    "image server exited before the desktop; the Pico dropped the medium".into(),
                );
            }
            if request_drop && !server.dropped && drop_marker(&text) {
                server.drop_medium()?;
            }
        }
        if desktop_reached(&text) {
            return Ok((text, true));
        }
        if text.contains("*** BUGCHECK") || text.contains("panicked at") {
            thread::sleep(Duration::from_secs(3));
            return Ok((serial_slice(path, start), false));
        }
        if Instant::now() > deadline {
            return Ok((text, false));
        }
        thread::sleep(Duration::from_millis(500));
    }
}
pub fn brightness(path: &Path) -> Option<f64> {
    let mut decoder = png::Decoder::new(fs::File::open(path).ok()?);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut bytes = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut bytes).ok()?;
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        _ => return None,
    };
    let mut sum = 0f64;
    let mut count = 0usize;
    for px in bytes[..info.buffer_size()].chunks_exact(channels) {
        sum += if channels <= 2 {
            px[0] as f64
        } else {
            0.299 * px[0] as f64 + 0.587 * px[1] as f64 + 0.114 * px[2] as f64
        };
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(sum / count as f64)
    }
}
/// Validate the process and receipt against this exact bench and Pico.
pub fn listener_valid(state: &Value, bench: &Path, host: &str, port: u16) -> bool {
    let Some(pid) = state
        .get("pid")
        .and_then(|p| p.as_u64().or_else(|| p.as_str()?.parse().ok()))
    else {
        return false;
    };
    let Ok(bytes) = fs::read(format!("/proc/{pid}/cmdline")) else {
        return false;
    };
    let argv: Vec<_> = bytes
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    let Ok(expected) = bench.canonicalize() else {
        return false;
    };
    let same_path = |p: &str| {
        let path = Path::new(p);
        let resolved = if path.is_absolute() {
            path.to_owned()
        } else {
            PathBuf::from(format!("/proc/{pid}/cwd")).join(path)
        };
        resolved.canonicalize().is_ok_and(|p| p == expected)
    };
    let exe = argv
        .first()
        .and_then(|s| Path::new(s).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let native = ["bench-infra", "vibeos-bench"].contains(&exe)
        && argv
            .windows(3)
            .any(|v| v[0] == "serial" && same_path(&v[1]) && v[2].eq_ignore_ascii_case(host));
    let legacy = argv
        .iter()
        .position(|s| {
            Path::new(s)
                .file_name()
                .is_some_and(|n| n == "pico_serial.py")
        })
        .and_then(|i| argv.get(i + 1..i + 3))
        .is_some_and(|v| same_path(&v[0]) && v[1].eq_ignore_ascii_case(host));
    if native {
        let live = fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .and_then(|s| {
                s.rsplit_once(')')
                    .and_then(|(_, rest)| rest.split_whitespace().nth(19))
                    .and_then(|v| v.parse::<u64>().ok())
            });
        if state["engine"] != "vibeos-bench"
            || live.is_none()
            || live != state["process_start_ticks"].as_u64()
        {
            return false;
        }
    }
    let directory = state
        .get("directory")
        .and_then(Value::as_str)
        .and_then(|p| Path::new(p).canonicalize().ok())
        .is_some_and(|p| p.starts_with(&expected) && p != expected);
    let expected_port = format!("{host}:{port} (picousb)");
    let actual_port = state.get("port").and_then(Value::as_str).unwrap_or("");
    (native || legacy) && directory && actual_port.eq_ignore_ascii_case(&expected_port)
}
pub fn tail_read(
    path: &Path,
    pos: u64,
    t0: f64,
    previous: &str,
    times: &Value,
    timeout: f64,
) -> Result<Value> {
    use std::io::{Read, Seek, SeekFrom};
    if !timeout.is_finite() || timeout < 0.0 {
        return Err("invalid serial read timeout".into());
    }
    thread::sleep(Duration::from_secs_f64(timeout));
    let mut raw = Vec::new();
    if let Ok(mut f) = fs::File::open(path) {
        if f.seek(SeekFrom::Start(pos)).is_ok() {
            let _ = f.read_to_end(&mut raw);
        }
    }
    let text = String::from_utf8_lossy(&raw).into_owned();
    let window = format!("{previous}{text}");
    let mut ts = times.as_object().cloned().unwrap_or_default();
    let mut clock = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, clock.as_mut_ptr()) } != 0 {
        return Err(Error::new(format!(
            "monotonic clock: {}",
            std::io::Error::last_os_error()
        )));
    }
    let clock = unsafe { clock.assume_init() };
    let now = ((clock.tv_sec as f64 + clock.tv_nsec as f64 / 1e9 - t0) * 10.0).round() / 10.0;
    let cfg = crate::ladder::Config::default();
    for (name, _) in cfg.ladder.iter().chain(cfg.optional.iter()) {
        for (event, marker) in [
            ("launch", format!("{name}.exe (")),
            ("exit", format!("{name}.exe exit = ")),
        ] {
            if window.contains(&marker) {
                ts.entry(format!("{name} {event}")).or_insert(json!(now));
            }
        }
    }
    if window.contains(crate::ladder::PROGRAM_DONE) {
        ts.entry("program_done").or_insert(json!(now));
    }
    let tail = text
        .chars()
        .rev()
        .take(200)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    Ok(json!({"text":text,"pos":pos+raw.len() as u64,"times":ts,"tail":tail}))
}
pub fn tail_durations(times: &Value) -> Value {
    let cfg = crate::ladder::Config::default();
    let mut out = Map::new();
    for (name, _) in cfg.ladder.iter().chain(cfg.optional.iter()) {
        if let (Some(a), Some(b)) = (
            times
                .get(format!("{name} launch").as_str())
                .and_then(Value::as_f64),
            times
                .get(format!("{name} exit").as_str())
                .and_then(Value::as_f64),
        ) {
            out.insert(name.clone(), json!(((b - a) * 10.0).round() / 10.0));
        }
    }
    Value::Object(out)
}
pub fn until(path: &Path, start: u64, needle: &str, timeout: f64) -> Result<Value> {
    if !timeout.is_finite() || timeout < 0.0 {
        return Err("invalid until timeout".into());
    }
    let deadline = Instant::now() + Duration::from_secs_f64(timeout);
    loop {
        let text = serial_slice(path, start);
        if let Some(hit) = text.lines().find(|l| l.contains(needle)) {
            return Ok(
                json!({"text":text,"classification":"","stage":{"name":"until","detail":hit.trim().chars().take(200).collect::<String>()}}),
            );
        }
        if Instant::now() > deadline {
            let tail: Vec<_> = text
                .lines()
                .rev()
                .take(6)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|l| l.trim().chars().take(200).collect::<String>())
                .collect();
            return Ok(
                json!({"text":text,"classification":"HANG_TIMEOUT","stage":{"name":"until","status":"fail","detail":format!("{needle:?} not seen {timeout:.0}s after the desktop"),"salient":tail}}),
            );
        }
        thread::sleep(Duration::from_secs(2));
    }
}
/// Compatibility adapter: the host supplies only image-server/drop state; Rust
/// owns the polling, deadline and verdict decision.
pub fn await_stortest_host(
    path: &Path,
    start: u64,
    has_medium: bool,
    has_drop: bool,
    host: &mut dyn crate::callback::Host,
) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(240);
    loop {
        let text = serial_slice(path, start);
        let (verdict, lines) = stortest_verdict(&text);
        let drop_state = if has_drop {
            host.call("bench_drop_state", json!({}))?
        } else {
            json!({"dropped":false})
        };
        let dropped = drop_state["dropped"].as_bool().unwrap_or(false);
        let medium = if has_medium {
            host.call("bench_medium_state", json!({}))?
        } else {
            json!({"lost":false})
        };
        let lost = medium["lost"].as_bool().unwrap_or(false);
        if verdict.is_some() || Instant::now() > deadline || (lost && !dropped) {
            let grade = if has_drop {
                grade_drop(&text, dropped, lost)
            } else if lost && verdict != Some("ok") {
                json!({"classification":"BENCH_UNHEALTHY","stage":"serve-image","status":"fail","detail":format!("image server exited {} {:.0}s after the reboot: the Pico dropped the medium",medium["returncode"],medium["lost_after"].as_f64().unwrap_or(0.)),"salient":medium.get("salient").cloned().unwrap_or(json!([]))})
            } else if verdict == Some("ok") {
                json!({"classification":"","stage":"stortest","status":"ok","detail":lines.last().cloned().unwrap_or_default(),"salient":[]})
            } else {
                json!({"classification":if verdict.is_some(){"STORTEST_FAIL"}else{"HANG_TIMEOUT"},"stage":"stortest","status":"fail","detail":if verdict.is_some(){"failed"}else{"no verdict 240s after the desktop"},"salient":lines.iter().rev().take(6).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>()})
            };
            return Ok(json!({"text":text,"lines":lines,"grade":grade}));
        }
        thread::sleep(Duration::from_secs(5));
    }
}

#[cfg(test)]
mod controller_fixtures {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::process::Stdio;
    #[test]
    fn listener_receipt_rejects_wrong_bench_host_and_path() {
        let d = tempfile::tempdir().unwrap();
        let bench = d.path().join("bench");
        fs::create_dir(&bench).unwrap();
        let capture = bench.join("serial-test");
        fs::create_dir(&capture).unwrap();
        let executable = d.path().join("vibeos-bench");
        symlink("/bin/sh", &executable).unwrap();
        let mut child = Command::new(&executable)
            .args([
                "-c",
                "printf ready; read x",
                "serial",
                bench.to_str().unwrap(),
                "PiCoUSB",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut ready = [0; 5];
        child
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(&mut ready)
            .unwrap();
        assert_eq!(&ready, b"ready");
        let ticks = fs::read_to_string(format!("/proc/{}/stat", child.id()))
            .unwrap()
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let state = json!({"pid":child.id(),"directory":capture,"port":"picousb:8082 (picousb)","engine":"vibeos-bench","process_start_ticks":ticks});
        let mut reused = state.clone();
        reused["process_start_ticks"] = json!(ticks + 1);
        assert!(!listener_valid(&reused, &bench, "picousb", 8082));
        assert!(listener_valid(&state, &bench, "picousb", 8082));
        assert!(!listener_valid(&state, &bench, "another-host", 8082));
        assert!(!listener_valid(&state, &bench, "picousb", 8083));
        let other = d.path().join("other");
        fs::create_dir(&other).unwrap();
        assert!(!listener_valid(&state, &other, "picousb", 8082));
        let mut forged = state.clone();
        forged["directory"] = json!(other);
        assert!(!listener_valid(&forged, &bench, "picousb", 8082));
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(!listener_valid(&state, &bench, "picousb", 8082));
    }
    #[test]
    fn drop_requires_this_pico_and_inline_completion() {
        let text="PiEnumerateDevice USBSTOR\\Disk&Ven_PICOUSB_&Prod_REMOTE_DISK: 1 children\n[STORTEST] FAIL: no VIBEUSB volume found\n";
        assert!(drop_marker(text));
        assert_eq!(
            grade_drop(text, true, false)["classification"],
            "NOT_EXERCISED"
        );
        assert_eq!(
            grade_drop(&format!("{text}{HARD_ERROR_INLINE}"), true, false)["classification"],
            ""
        );
        assert_eq!(
            grade_drop(text, false, true)["classification"],
            "BENCH_UNHEALTHY"
        );
    }
    #[test]
    fn serial_tail_does_not_count_old_boot_and_handles_split_verdict() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("serial.bin");
        fs::write(&p, b"hello32.exe exit = 0x600d\n").unwrap();
        let offset = fs::metadata(&p).unwrap().len();
        let first = tail_read(&p, offset, 0.0, "", &json!({}), 0.0).unwrap();
        assert_eq!(first["text"], "");
        use std::io::Write;
        let mut f = fs::OpenOptions::new().append(true).open(&p).unwrap();
        f.write_all(b"hello32.exe (guest)\nhello32.e").unwrap();
        let first = tail_read(&p, offset, 0.0, "", &json!({}), 0.0).unwrap();
        assert!(first["times"].get("hello32 launch").is_some());
        f.write_all(b"xe exit = 0x600d\n").unwrap();
        let second = tail_read(
            &p,
            first["pos"].as_u64().unwrap(),
            0.0,
            first["tail"].as_str().unwrap(),
            &first["times"],
            0.0,
        )
        .unwrap();
        assert!(second["times"].get("hello32 exit").is_some());
    }
}
/// Read back raw bytes, preserving even non-UTF-8 corruption for an exact grade.
pub fn stortest_readback(
    path: &Path,
    env: &std::collections::BTreeMap<String, String>,
) -> Result<Option<Vec<u8>>> {
    use std::process::Stdio;
    let mut child = Command::new("mtype")
        .args(["-i", &path.to_string_lossy(), "::WRITETST.TXT"])
        .env_clear()
        .envs(env)
        .env("MTOOLS_SKIP_CHECK", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::new(format!("mtype: {e}")))?;
    let mut stdout = child.stdout.take().ok_or("missing mtype stdout")?;
    let mut stderr = child.stderr.take().ok_or("missing mtype stderr")?;
    let out = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() > deadline {
            child.kill()?;
            child.wait()?;
            let _ = out.join();
            let _ = err.join();
            return Err("mtype timed out after 30s".into());
        }
        thread::sleep(Duration::from_millis(20));
    };
    let bytes = out
        .join()
        .map_err(|_| Error::new("mtype stdout reader failed"))??;
    let _stderr = err
        .join()
        .map_err(|_| Error::new("mtype stderr reader failed"))??;
    Ok(if status.success() { Some(bytes) } else { None })
}

#[cfg(test)]
mod late_storage_crash_tests {
    use super::*;
    #[test]
    fn crash_after_desktop_beats_storage_success_and_timeout() {
        let crash = "[EXPLORER] _tWinMain entered\nshell ready\n***   P4 = ntoskrnl.exe+0x6F7CC\n";
        for tail in ["", "[STORTEST] ALL OK drive D:\n"] {
            assert_eq!(
                grade_drop(&format!("{crash}{HARD_ERROR_INLINE}\n{tail}"), true, false)
                    ["classification"],
                "BUGCHECK"
            );
        }
    }
}
