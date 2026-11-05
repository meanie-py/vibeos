//! Declarative payload test execution. Python only supplies VM handles and pytest presentation.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    time::{Duration, Instant},
};

fn s<'a>(a: &'a Value, key: &str) -> &'a str {
    a[key].as_str().unwrap_or("")
}
fn number(a: &Value, key: &str, default: f64) -> Result<f64> {
    let n = match a.get(key) {
        Some(v) if !v.is_null() => v
            .as_f64()
            .ok_or_else(|| Error::new(format!("payload {key} must be numeric")))?,
        _ => default,
    };
    if !n.is_finite() || n < 0. {
        return Err(format!("payload {key} must be finite and nonnegative").into());
    }
    Ok(n)
}
fn assertion(message: impl Into<String>) -> Error {
    Error::classified("AssertionError", message)
}
fn say(h: &mut dyn Host, message: impl Into<String>) -> Result<()> {
    h.call("payload_print", json!({"message":message.into()}))?;
    Ok(())
}
fn sleep(h: &mut dyn Host, seconds: f64) -> Result<()> {
    h.call("payload_sleep", json!({"seconds":seconds}))?;
    Ok(())
}
fn qmp(h: &mut dyn Host, command: &str, arguments: Value) -> Result<Value> {
    let result = h.call(
        "payload_qmp",
        json!({"command":command,"arguments":arguments}),
    )?;
    if let Some(error) = result.get("error") {
        return Err(assertion(format!("QMP {command} failed: {error}")));
    }
    Ok(result)
}
fn shot(h: &mut dyn Host, path: &Path) -> Result<()> {
    let result = h.call("payload_screendump", json!({"path":path}))?;
    if let Some(e) = result.get("error") {
        return Err(Error::new(format!("payload screenshot: {e}")));
    }
    Ok(())
}
fn send(h: &mut dyn Host, events: Value) -> Result<()> {
    qmp(h, "input-send-event", json!({"events":events}))?;
    Ok(())
}
pub fn click_center(h: &mut dyn Host) -> Result<()> {
    for _ in 0..25 {
        send(
            h,
            json!([{"type":"rel","data":{"axis":"x","value":-50}},{"type":"rel","data":{"axis":"y","value":-40}}]),
        )?;
    }
    for _ in 0..10 {
        send(
            h,
            json!([{"type":"rel","data":{"axis":"x","value":40}},{"type":"rel","data":{"axis":"y","value":28}}]),
        )?;
    }
    for down in [true, false] {
        send(
            h,
            json!([{"type":"btn","data":{"button":"left","down":down}}]),
        )?;
    }
    sleep(h, 1.5)
}
fn key(h: &mut dyn Host, qcode: &str) -> Result<()> {
    for down in [true, false] {
        send(
            h,
            json!([{"type":"key","data":{"down":down,"key":{"type":"qcode","data":qcode}}}]),
        )?;
    }
    Ok(())
}
pub fn drive(h: &mut dyn Host, steps: &Value) -> Result<()> {
    for step in steps.as_array().ok_or("payload drive must be a list")? {
        if let Some(keys) = step.get("keys") {
            let gap = number(step, "gap", 0.5)?;
            for k in keys.as_array().ok_or("payload keys must be a list")? {
                key(h, k.as_str().ok_or("payload key must be a qcode string")?)?;
                sleep(h, gap)?;
            }
        } else if step.get("wait").is_some() {
            sleep(h, number(step, "wait", 0.)?)?;
        } else if let Some(focus) = step.get("focus") {
            if focus != "click-center" {
                return Err(assertion(format!("unknown focus mode {focus}")));
            }
            click_center(h)?;
        } else {
            return Err(assertion(format!(
                "unknown drive step {step} — add the verb to the native payload interpreter"
            )));
        }
    }
    Ok(())
}
/// Match Pillow's ImageChops.difference(...).convert("L") rounding and >8 threshold.
pub fn pixel_delta(a: &Path, b: &Path) -> Result<f64> {
    let (aw, ah, a) = crate::screenshot::load_rgb(a)?;
    let (bw, bh, b) = crate::screenshot::load_rgb(b)?;
    if (aw, ah) != (bw, bh) {
        return Err("payload screenshots changed dimensions".into());
    }
    let changed = a
        .chunks_exact(3)
        .zip(b.chunks_exact(3))
        .filter(|(a, b)| {
            let r = a[0].abs_diff(b[0]) as u32;
            let g = a[1].abs_diff(b[1]) as u32;
            let b = a[2].abs_diff(b[2]) as u32;
            ((19595 * r + 38470 * g + 7471 * b + 32768) >> 16) > 8
        })
        .count();
    Ok(changed as f64 / (aw as f64 * ah as f64))
}
fn frame_delta(h: &mut dyn Host, work: &Path, gap: f64) -> Result<f64> {
    let a = work.join("payload_a.ppm");
    let b = work.join("payload_b.ppm");
    shot(h, &a)?;
    sleep(h, gap)?;
    shot(h, &b)?;
    pixel_delta(&a, &b)
}
struct Clock {
    start: Instant,
    callback: bool,
}
impl Clock {
    fn now(&self, h: &mut dyn Host) -> Result<f64> {
        if self.callback {
            let t = h
                .call("payload_clock", json!({}))?
                .as_f64()
                .ok_or("payload clock must return seconds")?;
            if !t.is_finite() {
                return Err("payload clock must be finite".into());
            }
            Ok(t)
        } else {
            Ok(self.start.elapsed().as_secs_f64())
        }
    }
}
struct Observer<'a> {
    args: &'a Value,
    taken: usize,
    samples: usize,
}
impl Observer<'_> {
    fn output(&mut self, h: &mut dyn Host, text: &str) -> Result<()> {
        self.taken = take_shots(h, self.args, text, self.taken)?;
        Ok(())
    }
    fn silence(&mut self, h: &mut dyn Host, seconds: f64) -> Result<()> {
        self.samples = sample_silence(h, self.args, seconds, self.samples, 12)?;
        Ok(())
    }
}
/// Wall-clock bounded, with idle and incomplete marker-line fences. A caller-supplied
/// clock is only an I/O callback; all scheduling decisions remain here.
fn drain(h: &mut dyn Host, a: &Value, mut observer: Option<&mut Observer<'_>>) -> Result<Value> {
    let clock = Clock {
        start: Instant::now(),
        callback: a["callback_clock"] == true,
    };
    let deadline = clock.now(h)? + number(a, "budget_secs", 30.)?;
    let mut last = clock.now(h)?;
    let silence = number(a, "silence_secs", 0.)?;
    let mut next_silence = silence;
    let idle = number(a, "idle_secs", 0.)?;
    let mut line_deadline = None;
    let marker = s(a, "marker");
    if marker.is_empty() {
        return Err("payload drain needs a marker".into());
    }
    let fail = s(a, "fail_marker");
    let mut out = s(a, "output").to_owned();
    let reason = loop {
        if let Some(at) = out.find(marker) {
            if out[at..].contains('\n') {
                break "marker";
            }
            if let Some(end) = line_deadline {
                if clock.now(h)? >= end {
                    break "marker";
                }
            } else {
                line_deadline = Some(clock.now(h)? + 10.);
            }
        }
        if !fail.is_empty() && out.contains(fail) {
            break "fail";
        }
        if clock.now(h)? >= deadline {
            break "budget";
        }
        if idle > 0. && clock.now(h)? - last >= idle {
            break "idle";
        }
        if silence > 0. && clock.now(h)? - last >= next_silence {
            let seconds = clock.now(h)? - last;
            if let Some(ref mut observer) = observer {
                observer.silence(h, seconds)?;
            } else if a["on_silence"] == true {
                h.call("payload_on_silence", json!({"seconds":seconds}))?;
            }
            next_silence += silence;
        }
        let chunk = h
            .call("payload_read", json!({"timeout":1.0}))?
            .as_str()
            .ok_or("payload read must return text")?
            .to_owned();
        if !chunk.is_empty() {
            out.push_str(&chunk);
            last = clock.now(h)?;
            next_silence = silence;
            if let Some(ref mut observer) = observer {
                observer.output(h, &out)?;
            } else if a["on_output"] == true {
                h.call("payload_on_output", json!({"output":out}))?;
            }
        }
    };
    Ok(json!({"output":out,"reason":reason}))
}
fn take_shots(h: &mut dyn Host, a: &Value, text: &str, mut taken: usize) -> Result<usize> {
    let prefix = s(a, "prefix");
    if prefix.is_empty() {
        return Ok(taken);
    }
    let re = regex::Regex::new(&format!(
        r"{}([A-Za-z0-9_.\-]+)\r?\n",
        regex::escape(prefix)
    ))?;
    for cap in re.captures_iter(text).skip(taken) {
        taken += 1;
        let name = &cap[1];
        let ppm = Path::new(s(a, "artifact_dir")).join(format!("shot-{taken:02}-{name}.ppm"));
        let png = ppm.with_extension("png");
        let capture = (|| -> Result<()> {
            shot(h, &ppm)?;
            crate::screenshot::convert(&ppm, &png)?;
            fs::remove_file(&ppm)?;
            Ok(())
        })();
        match capture {
            Ok(()) => say(h, format!("[payload] screenshot {}", png.display()))?,
            Err(e) => say(h, format!("[payload] screenshot {name} FAILED: {e}"))?,
        }
    }
    Ok(taken)
}
fn sample_silence(
    h: &mut dyn Host,
    a: &Value,
    seconds: f64,
    taken: usize,
    limit: usize,
) -> Result<usize> {
    if taken >= limit {
        return Ok(taken);
    }
    let taken = taken + 1;
    let state = match h.call("payload_cpu_state", json!({})) {
        Ok(v) => {
            if let Some(error) = v.get("error") {
                format!("<capture failed: {error}>\n")
            } else {
                v["state"]
                    .as_str()
                    .unwrap_or("<invalid CPU capture>\n")
                    .to_owned()
            }
        }
        Err(e) => format!("<capture failed: {e}>\n"),
    };
    let pcs = crate::verdict::cpu_pcs(&state)
        .iter()
        .map(|(cpu, pc)| format!("CPU#{cpu}=0x{pc}"))
        .collect::<Vec<_>>()
        .join(" ");
    let head = format!(
        "=== silence sample #{taken}: serial silent {seconds:.0}s  pcs: {} ===\n",
        if pcs.is_empty() { "?" } else { &pcs }
    );
    say(h, format!("[payload] {}", head.trim()))?;
    // Diagnostic snapshots are deliberately best effort; their loss is reported,
    // and must not manufacture a guest failure or end the serial drain.
    let dest = Path::new(s(a, "artifact_dir")).join("silence-samples.txt");
    let result = (|| -> std::io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&dest)?;
        writeln!(f, "{head}{state}")
    })();
    if let Err(e) = result {
        say(
            h,
            format!("[payload] silence capture {} FAILED: {e}", dest.display()),
        )?;
    }
    Ok(taken)
}
fn require_net(t: &Value, arch: &str, name: &str) -> Result<()> {
    if t["net"] == true && !arch.ends_with("-net") {
        return Err(assertion(format!("payload {name:?} declares net: true — boot the -net variant (e.g. --arch {arch}-net) so the NIC + payload hostfwd exist")));
    }
    Ok(())
}
fn launch(root: &Path, h: &mut dyn Host, a: &Value) -> Result<Value> {
    let t = &a["spec"]["test"];
    let arch = s(a, "arch");
    let name = s(a, "name");
    require_net(t, arch, name)?;
    let marker = s(t, "launch_marker");
    if marker.is_empty() {
        return Ok(
            json!({"status":"skip","reason":format!("payload {name:?} declares no launch_marker")}),
        );
    }
    let fail = t["fail_marker"].as_str().unwrap_or("*** BUGCHECK");
    let mut out = s(a, "boot_output").to_owned();
    let observation = json!({"prefix":t["shot_prefix"],"artifact_dir":a["artifact_dir"]});
    let mut observer = Observer {
        args: &observation,
        taken: 0,
        samples: 0,
    };
    observer.output(h, &out)?;
    if !out.contains(marker) {
        let result = drain(
            h,
            &json!({"output":out,"marker":marker,"fail_marker":fail,"budget_secs":number(t,"launch_drain_secs",30.)?,"idle_secs":number(t,"launch_idle_secs",0.)?,"silence_secs":number(t,"silence_sample_secs",0.)?}),
            Some(&mut observer),
        )?;
        out = s(&result, "output").into();
        say(h, format!("[payload] drain-end: {}", s(&result, "reason")))?;
    }
    if !out.contains(marker) && !(!fail.is_empty() && out.contains(fail)) {
        h.call(
            "payload_save_hang",
            json!({"phase":format!("payload {name}: launch_marker never appeared")}),
        )?;
    }
    if !out.contains(marker) {
        let tail: String = out
            .chars()
            .rev()
            .take(800)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        return Err(assertion(format!("--payload {name} was passed but {marker:?} never appeared. Either the media was not built with VIBEOS_PAYLOAD={name} (fix: VIBEOS_PAYLOAD={name} ./vibeos-cli build --arch <arch>) or the launcher regressed (grep serial for CreateProcessW errors).\nserial tail: {tail:?}")));
    }
    if let Some(pass) = t["pass_marker"].as_str().filter(|v| !v.is_empty()) {
        if !out.contains(pass) {
            return Err(assertion(format!(
                "--payload {name} finished but its output lacks {pass:?}: {:?}",
                out.lines().find(|ln| ln.contains(marker)).unwrap_or("")
            )));
        }
    }
    if let Some(grader) = t["grader"].as_str().filter(|v| !v.is_empty()) {
        let grade = vibeos_build::payload_providers::grade(root, grader, &out, arch)?;
        let lines = |key: &str| {
            grade[key]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default()
        };
        say(
            h,
            format!("\n[payload] {name} grader:\n{}", lines("report")),
        )?;
        if grade["failures"].as_array().is_some_and(|v| !v.is_empty()) {
            return Err(assertion(format!(
                "--payload {name}: {} failed:\n{}",
                grade["failures"].as_array().unwrap().len(),
                lines("failures")
            )));
        }
    }
    Ok(json!({"status":"pass"}))
}
fn renders(h: &mut dyn Host, a: &Value) -> Result<Value> {
    let r = &a["spec"]["test"]["renders"];
    let name = s(a, "name");
    if r.is_null() || r == &json!({}) {
        return Ok(
            json!({"status":"skip","reason":format!("payload {name:?} declares no renders gate")}),
        );
    }
    sleep(h, number(r, "settle", 3.)?)?;
    let threshold = number(r, "frame_delta_min", 0.02)?;
    let delta = frame_delta(h, Path::new(s(a, "work_dir")), number(r, "gap", 2.)?)?;
    if delta < threshold {
        return Err(assertion(format!("screen delta {delta:.4} < {threshold} — {name} launched but nothing is animating (GDI blit path / app loop wedged?). Check qemu/{}_smoke.png and the serial log.",s(a,"arch"))));
    }
    say(
        h,
        format!(
            "\n[{name}] rendering confirmed — frame delta {:.1}%",
            delta * 100.
        ),
    )?;
    Ok(json!({"status":"pass","delta":delta}))
}
fn input(h: &mut dyn Host, a: &Value) -> Result<Value> {
    let t = &a["spec"]["test"];
    let i = &t["input"];
    let name = s(a, "name");
    let arch = s(a, "arch");
    if i.is_null() || i == &json!({}) {
        return Ok(
            json!({"status":"skip","reason":format!("payload {name:?} declares no input gate")}),
        );
    }
    if a["force"] != true {
        if let Some(x) = t["xfail"].as_object() {
            for (prefix, reason) in x {
                if arch.starts_with(prefix) {
                    return Ok(json!({"status":"xfail","reason":reason}));
                }
            }
        }
    }
    let oracle = &i["oracle"];
    let static_max = number(oracle, "frame_delta_static_max", 0.012)?;
    let gap = number(oracle, "gap", 1.5)?;
    let render_min = number(&t["renders"], "frame_delta_min", 0.02)?;
    sleep(h, number(i, "settle", 3.)?)?;
    if i["focus"].as_str().unwrap_or("click-center") == "click-center" {
        click_center(h)?;
    }
    let work = Path::new(s(a, "work_dir"));
    let pre = frame_delta(h, work, gap)?;
    if pre < render_min {
        return Err(assertion(format!("pre-input frame delta {pre:.4} too low — app not animating, input test would be meaningless")));
    }
    let retries = i["retries"].as_u64().unwrap_or(1);
    if retries == 0 {
        return Err("payload retries must be positive".into());
    }
    let mut post = 1.;
    for attempt in 0..retries {
        drive(
            h,
            i.get("drive")
                .filter(|v| !v.is_null())
                .unwrap_or(&json!([])),
        )?;
        post = frame_delta(h, work, gap)?;
        if post <= static_max {
            break;
        }
        say(
            h,
            format!(
                "[{name}] input attempt {}: delta {post:.4} — retry",
                attempt + 1
            ),
        )?;
        drive(
            h,
            i.get("on_retry")
                .filter(|v| !v.is_null())
                .unwrap_or(&json!([])),
        )?;
    }
    if post > static_max {
        let path = Path::new(s(a, "repo"))
            .join("qemu")
            .join(format!("{arch}_{name}_input_fail.ppm"));
        shot(h, &path)?;
        return Err(assertion(format!("post-drive frame delta {post:.4} > {static_max} — the screen kept animating, so the drive keys never reached {name} ({retries} attempts). Input chain is broken between the class driver and the foreground window (win32k routing/focus). Screenshot: {}",path.display())));
    }
    if oracle["survives"] != false {
        survives(h)?;
    }
    say(
        h,
        format!(
            "\n[{name}] PLAYABLE — drive script confirmed (delta {:.1}% -> {:.1}%)",
            pre * 100.,
            post * 100.
        ),
    )?;
    Ok(json!({"status":"pass","pre":pre,"post":post}))
}
fn survives(h: &mut dyn Host) -> Result<()> {
    if qmp(h, "query-status", json!({}))?["return"]["running"] != true {
        return Err(assertion("payload VM is no longer running"));
    }
    Ok(())
}
fn http_get(host: &str, port: u16, path: &str) -> Result<(u16, Vec<u8>)> {
    let addresses = (host, port).to_socket_addrs()?;
    let mut stream = None;
    let mut last = None;
    for addr in addresses {
        match TcpStream::connect_timeout(&addr, Duration::from_secs(20)) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => last = Some(e),
        }
    }
    let mut stream =
        stream.ok_or_else(|| Error::new(format!("payload HTTP connection: {last:?}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(20)))?;
    stream.set_write_timeout(Some(Duration::from_secs(20)))?;
    stream.write_all(format!("GET {path} HTTP/1.0\r\nHost: guest\r\n\r\n").as_bytes())?;
    let mut data = Vec::new();
    (&mut stream)
        .take(512 * 1024 * 1024 + 1)
        .read_to_end(&mut data)?;
    if data.len() > 512 * 1024 * 1024 {
        return Err("payload HTTP response exceeds 512 MiB".into());
    }
    let at = data
        .windows(4)
        .position(|b| b == b"\r\n\r\n")
        .ok_or("no header terminator in payload HTTP response")?;
    let status = std::str::from_utf8(&data[..at])
        .map_err(|_| Error::new("invalid HTTP header"))?
        .split_whitespace()
        .nth(1)
        .ok_or("missing HTTP status")?
        .parse::<u16>()
        .map_err(|_| Error::new("invalid HTTP status"))?;
    Ok((status, data[at + 4..].to_vec()))
}
fn tcp_hammer(h: &mut dyn Host, a: &Value) -> Result<Value> {
    let t = &a["spec"]["test"];
    let hammer = &t["tcp_hammer"];
    let name = s(a, "name");
    if hammer.is_null() || hammer == &json!({}) {
        return Ok(
            json!({"status":"skip","reason":format!("payload {name:?} declares no tcp_hammer gate")}),
        );
    }
    require_net(t, s(a, "arch"), name)?;
    let port = a["host_port"]
        .as_u64()
        .and_then(|v| u16::try_from(v).ok())
        .filter(|p| *p > 0)
        .ok_or_else(|| assertion("vm config has no payload_hostfwd_port (net config too old?)"))?;
    let echo = a["echo_port"].as_u64().unwrap_or(0);
    let host = s(a, "net_host");
    say(
        h,
        format!("[tcp_hammer] payload host_port={port} echo control port={echo}"),
    )?;
    if echo > 0 {
        let probe = (|| -> Result<Vec<u8>> {
            let address = (host, echo as u16)
                .to_socket_addrs()?
                .next()
                .ok_or("no echo address")?;
            let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))?;
            stream.set_read_timeout(Some(Duration::from_secs(10)))?;
            stream.set_write_timeout(Some(Duration::from_secs(10)))?;
            stream.write_all(b"hammer-control\n")?;
            let mut b = [0; 64];
            let n = stream.read(&mut b)?;
            Ok(b[..n].to_vec())
        })();
        say(
            h,
            match probe {
                Ok(got) => format!("[tcp_hammer] echo control OK ({got:?})"),
                Err(e) => {
                    format!("[tcp_hammer] echo control FAILED: {e} — hostfwd plumbing suspect")
                }
            },
        )?;
    }
    let threads = hammer["threads"].as_u64().unwrap_or(16);
    let requests = hammer["requests"].as_u64().unwrap_or(20);
    if threads == 0 || threads > 1024 || requests == 0 || requests.checked_mul(threads).is_none() {
        return Err("invalid payload hammer concurrency/request count".into());
    }
    let sizes = hammer
        .get("data_sizes")
        .cloned()
        .unwrap_or(json!([1, 512, 4096]));
    let sizes = sizes
        .as_array()
        .ok_or("payload data_sizes must be a list")?
        .iter()
        .map(|v| {
            v.as_u64()
                .filter(|n| *n <= 512 * 1024 * 1024)
                .ok_or_else(|| Error::new("invalid payload data size"))
        })
        .collect::<Result<Vec<_>>>()?;
    if sizes.is_empty() {
        return Err("payload data_sizes is empty".into());
    }
    let mut handles = Vec::new();
    for tid in 0..threads {
        let host = host.to_owned();
        let sizes = sizes.clone();
        handles.push(std::thread::spawn(move || {
            let mut ok = 0u64;
            let mut errors = Vec::new();
            for i in 0..requests {
                let size = sizes[((tid + i) % sizes.len() as u64) as usize];
                let path = if i % 4 == 0 {
                    "/".into()
                } else {
                    format!("/data?n={size}")
                };
                let result = (|| -> Result<()> {
                    let (status, body) = http_get(&host, port, &path)?;
                    if status != 200 {
                        return Err(format!("{path} -> {status}").into());
                    }
                    if path.starts_with("/data")
                        && (body.len() != size as usize
                            || body
                                .iter()
                                .enumerate()
                                .any(|(j, v)| *v != ((j.wrapping_mul(7) + 13) & 255) as u8))
                    {
                        return Err(
                            format!("{path} body mismatch: got {} bytes", body.len()).into()
                        );
                    }
                    Ok(())
                })();
                match result {
                    Ok(()) => ok += 1,
                    Err(e) => errors.push(format!("t{tid} req{i} {path}: {e}")),
                }
            }
            (ok, errors)
        }));
    }
    let mut ok = 0;
    let mut errors = Vec::new();
    for handle in handles {
        let (n, e) = handle
            .join()
            .map_err(|_| Error::new("payload hammer worker panicked"))?;
        ok += n;
        errors.extend(e);
    }
    let total = threads * requests;
    if !errors.is_empty() {
        return Err(assertion(format!(
            "tcp_hammer: {}/{total} requests failed ({ok} ok). First failures:\n{}",
            errors.len(),
            errors.into_iter().take(10).collect::<Vec<_>>().join("\n")
        )));
    }
    if hammer["status_check"] != false {
        let (status, body) = http_get(host, port, "/status")?;
        if status != 200 {
            return Err(assertion(format!("/status -> {status}")));
        }
        let text = String::from_utf8_lossy(&body);
        let count = text
            .split("requests=")
            .nth(1)
            .and_then(|s| s.split(' ').next())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or("invalid payload request counter")?;
        if count < ok {
            return Err(assertion(format!("guest counted {count} requests but host got {ok} 200s — phantom successes (responses without server-side accounting)")));
        }
    }
    survives(h)?;
    say(
        h,
        format!("\n[{name}] tcp_hammer PASS — {ok}/{total} requests verified, OS alive"),
    )?;
    Ok(json!({"status":"pass","requests":ok}))
}
pub fn dispatch(root: &Path, r: &Value, h: &mut dyn Host) -> Result<Value> {
    let a = &r["kwargs"];
    match s(r, "op") {
        "payload_firmware_fetch" => {
            let files = vibeos_build::payload_firmware::files_from_json(&a["files"])?;
            let dest = Path::new(s(a, "dest"));
            Ok(json!(vibeos_build::payload_firmware::fetch_with(
                dest,
                &files,
                |url| {
                    let bytes = h.call("payload_download", json!({"url":url}))?;
                    bytes
                        .as_array()
                        .ok_or("payload download must return bytes")?
                        .iter()
                        .map(|b| {
                            b.as_u64()
                                .and_then(|n| u8::try_from(n).ok())
                                .ok_or_else(|| Error::new("invalid download byte"))
                        })
                        .collect()
                }
            )?))
        }
        "payload_select" => {
            let mut names: Vec<_> = s(a, "payload")
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect();
            if a["doom"] == true && !names.iter().any(|n| n == "doom") {
                names.push("doom".into());
            }
            Ok(json!(names))
        }
        "payload_load_spec" => vibeos_build::payload_spec::load(root, s(a, "name")),
        "payload_drain" => drain(h, a, None),
        "payload_launch" => launch(root, h, a),
        "payload_renders" => renders(h, a),
        "payload_input" => input(h, a),
        "payload_tcp_hammer" => tcp_hammer(h, a),
        "payload_frame_delta" => Ok(json!(frame_delta(
            h,
            Path::new(s(a, "work_dir")),
            number(a, "gap", 2.)?
        )?)),
        "payload_pixel_delta" => Ok(json!(pixel_delta(
            Path::new(s(a, "a")),
            Path::new(s(a, "b"))
        )?)),
        "payload_drive" => {
            drive(h, &a["steps"])?;
            Ok(Value::Null)
        }
        "payload_key" => {
            key(h, s(a, "key"))?;
            Ok(Value::Null)
        }
        "payload_click" => {
            click_center(h)?;
            Ok(Value::Null)
        }
        "payload_send" => {
            send(h, a["events"].clone())?;
            Ok(Value::Null)
        }
        "payload_shots" => Ok(json!(take_shots(
            h,
            a,
            s(a, "output"),
            a["taken"].as_u64().unwrap_or(0) as usize
        )?)),
        "payload_sample" => Ok(json!(sample_silence(
            h,
            a,
            number(a, "seconds", 0.)?,
            a["taken"].as_u64().unwrap_or(0) as usize,
            a["limit"].as_u64().unwrap_or(12) as usize
        )?)),
        "payload_require_net" => {
            require_net(&a["spec"]["test"], s(a, "arch"), s(a, "name"))?;
            Ok(Value::Null)
        }
        _ => Err(format!("unknown payload operation {}", s(r, "op")).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Script {
        now: f64,
        step: f64,
        reads: usize,
        chunks: Vec<String>,
        calls: Vec<(String, Value)>,
    }
    impl Script {
        fn new(step: f64, chunks: &[&str]) -> Self {
            Self {
                now: 0.,
                step,
                reads: 0,
                chunks: chunks.iter().map(|s| s.to_string()).collect(),
                calls: Vec::new(),
            }
        }
    }
    impl Host for Script {
        fn call(&mut self, op: &str, a: Value) -> Result<Value> {
            self.calls.push((op.into(), a.clone()));
            Ok(match op {
                "payload_clock" => json!(self.now),
                "payload_read" => {
                    self.now += self.step;
                    self.reads += 1;
                    json!(self.chunks.get(self.reads - 1).cloned().unwrap_or_default())
                }
                "payload_qmp" => json!({"return":{"running":true}}),
                _ => Value::Null,
            })
        }
    }
    #[test]
    fn chatty_wall_clock_budget_and_idle_are_distinct() {
        let mut host = Script::new(0.02, &[]);
        host.chunks = vec!["chatter\n".into(); 31000];
        let result = drain(
            &mut host,
            &json!({"callback_clock":true,"marker":"DONE","budget_secs":600.}),
            None,
        )
        .unwrap();
        assert_eq!(result["reason"], "budget");
        assert!(host.now >= 600.);
        assert!(host.reads > 25000);
        let mut host = Script::new(1., &[]);
        let result = drain(
            &mut host,
            &json!({"callback_clock":true,"marker":"DONE","budget_secs":7200.,"idle_secs":600.}),
            None,
        )
        .unwrap();
        assert_eq!(result["reason"], "idle");
        assert_eq!(host.reads, 600);
    }
    #[test]
    fn verdict_line_is_finished_and_deadlines_bound_missing_newline() {
        let mut host = Script::new(0.02, &["noise\nDONE VER", "DICT=PASS\n", "never read"]);
        let out = drain(
            &mut host,
            &json!({"callback_clock":true,"marker":"DONE","budget_secs":600.}),
            None,
        )
        .unwrap();
        assert_eq!(out["reason"], "marker");
        assert!(s(&out, "output").contains("VERDICT=PASS"));
        assert_eq!(host.reads, 2);
        let mut host = Script::new(1., &[]);
        let out = drain(
            &mut host,
            &json!({"callback_clock":true,"marker":"DONE","output":"DONE VER","budget_secs":600.}),
            None,
        )
        .unwrap();
        assert_eq!(out["reason"], "marker");
        assert_eq!(host.reads, 10);
    }
    #[test]
    fn silence_samples_rearm_after_progress_and_crash_ends_drain() {
        let mut host = Script::new(1., &[]);
        host.chunks = vec![String::new(); 50];
        host.chunks[44] = "back\n".into();
        host.chunks[49] = "DONE\n".into();
        let out=drain(&mut host,&json!({"callback_clock":true,"marker":"DONE","budget_secs":600.,"silence_secs":20.,"on_silence":true}),None).unwrap();
        assert_eq!(out["reason"], "marker");
        let samples: Vec<_> = host
            .calls
            .iter()
            .filter(|(op, _)| op == "payload_on_silence")
            .map(|(_, a)| a["seconds"].clone())
            .collect();
        assert_eq!(samples, vec![json!(20.), json!(40.)]);
        let mut host = Script::new(0.02, &["*** BUGCHECK: 0x1e\n"]);
        let out=drain(&mut host,&json!({"callback_clock":true,"marker":"DONE","fail_marker":"*** BUGCHECK","budget_secs":7200.}),None).unwrap();
        assert_eq!(out["reason"], "fail");
        assert_eq!(host.reads, 1);
    }
    #[test]
    fn input_order_and_fail_loud_unknown_verbs() {
        let mut host = Script::new(1., &[]);
        drive(
            &mut host,
            &json!([{"keys":["esc","ret"],"gap":0.7},{"wait":2.}]),
        )
        .unwrap();
        let events: Vec<_> = host
            .calls
            .iter()
            .filter(|(op, _)| op == "payload_qmp")
            .map(|(_, a)| a["arguments"]["events"][0]["data"].clone())
            .collect();
        assert_eq!(events.len(), 4);
        assert_eq!(events[0]["key"]["data"], "esc");
        assert_eq!(events[0]["down"], true);
        assert_eq!(events[1]["down"], false);
        assert!(drive(&mut host, &json!([{"mystery":1}])).is_err());
        assert!(drive(&mut host, &json!([{"wait":-1}])).is_err());
    }
    #[test]
    fn delta_uses_luma_noise_threshold_and_exact_dimensions() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a.ppm");
        let b = tmp.path().join("b.ppm");
        fs::write(&a, b"P6\n3 1\n255\n\0\0\0\0\0\0\0\0\0").unwrap();
        fs::write(&b, b"P6\n3 1\n255\n\x08\x08\x08\x09\x09\x09\0\0\x10").unwrap();
        assert_eq!(pixel_delta(&a, &b).unwrap(), 1. / 3.);
        fs::write(&b, b"P6\n1 1\n255\n\0\0\0").unwrap();
        assert!(pixel_delta(&a, &b).is_err());
    }
    #[test]
    fn survival_and_missing_network_contract_are_required() {
        let mut host = Script::new(1., &[]);
        assert!(require_net(&json!({"net":true}), "arm64", "httpd").is_err());
        assert!(require_net(&json!({"net":true}), "arm64-net", "httpd").is_ok());
        survives(&mut host).unwrap();
    }
}
