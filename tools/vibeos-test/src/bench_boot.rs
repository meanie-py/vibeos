//! Native lease-scoped Raspberry Pi boot orchestration.
use crate::{bench_helpers as h, callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
type Env = BTreeMap<String, String>;
fn strval<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn flag(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
}
struct Context<'a> {
    root: &'a Path,
    board: &'a str,
    args: &'a Value,
    env: Env,
    run_dir: PathBuf,
    host: &'a mut dyn Host,
}
impl Context<'_> {
    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
        self.host.call(op, a)
    }
    fn stage(&mut self, name: &str, status: &str, detail: &str, salient: Value) -> Result<()> {
        self.call(
            "stage",
            json!({"name":name,"status":status,"detail":detail,"salient":salient}),
        )?;
        Ok(())
    }
    fn class(&mut self, c: &str) -> Result<()> {
        self.call("set_run", json!({"classification":c}))?;
        Ok(())
    }
    fn extra(&mut self, key: &str, value: Value) -> Result<()> {
        self.call("set_run", json!({"extra":{key:value}}))?;
        Ok(())
    }
    fn finish(&mut self) -> Result<Value> {
        self.call("finalize", json!({}))
    }
    fn command(&mut self, script: &str, argv: &[&str], timeout: u64, stage: &str) -> Result<Value> {
        let mut a = vec![
            "bash".to_string(),
            self.root
                .join("tools")
                .join(self.board)
                .join(script)
                .to_string_lossy()
                .into_owned(),
        ];
        a.extend(argv.iter().map(|s| s.to_string()));
        self.call(
            "command",
            json!({"argv":a,"cwd":self.root,"env":self.env,"timeout":timeout,"stage":stage}),
        )
    }
    fn fail(&mut self, class: &str, name: &str, detail: &str, salient: Value) -> Result<Value> {
        self.class(class)?;
        self.stage(name, "fail", detail, salient)?;
        self.finish()
    }
    fn save_serial(&mut self, text: &str) -> Result<()> {
        let path = self.run_dir.join("serial.log");
        fs::write(&path, text)?;
        self.extra("serial_log", json!(path))
    }
    fn stale(&mut self, request: &Value) -> Result<bool> {
        let cfg = request
            .get("cfgs")
            .and_then(|v| v.get("arm64"))
            .ok_or("missing arm64 provenance configuration")?;
        let st = vibeos_build::provenance::dispatch(
            self.root,
            &json!({"op":"provenance_check_staleness_inner","kwargs":{"repo":self.root,"arch":"arm64","cfg":cfg,"env":self.env,"store_root":request.get("store_root")}}),
        )?;
        let stale = !flag(&st, "fresh") || flag(&st, "media_missing");
        let warm = flag(&st, "warm");
        if stale && !flag(self.args, "allow_stale") {
            self.class("STALE_MEDIA")?;
            self.stage(
                "staleness",
                "fail",
                "stale",
                st.get("reasons").cloned().unwrap_or(json!([])),
            )?;
            return Ok(false);
        }
        self.stage(
            "staleness",
            if stale || warm { "warn" } else { "ok" },
            if warm {
                "WARM build"
            } else if stale {
                "stale"
            } else {
                "current"
            },
            json!([]),
        )?;
        Ok(true)
    }
}
fn lease_error(e: vibeos_bench::Error) -> Error {
    Error::new(e.message)
}
pub fn boot(root: &Path, board: &str, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = request.get("args").ok_or("missing boot arguments")?;
    if board == "pi5" && flag(args, "drop_medium") && !flag(args, "stortest") {
        host.call(
            "print",
            json!({"text":"pi5 boot: --drop-medium needs --stortest"}),
        )?;
        return Ok(json!(2));
    }
    let mut env: Env = serde_json::from_value(request.get("env").cloned().unwrap_or(json!({})))?;
    env.insert("VIBEOS_BENCH_BOARD".into(), board.into());
    let owner = vibeos_bench::lease::owner_of(root, &env).map_err(lease_error)?;
    let held = vibeos_bench::lease::current(&env, None)
        .map_err(lease_error)?
        .get("owner")
        .and_then(Value::as_str)
        == Some(owner.as_str());
    let timeout = args.get("timeout").and_then(Value::as_f64).unwrap_or(300.0);
    if !timeout.is_finite() || timeout < 0.0 {
        return Err("invalid desktop timeout".into());
    }
    let minutes = (timeout / 60.0 + 20.0).max(30.0) + if flag(args, "ladder") { 30.0 } else { 0.0 };
    let run = host.call("get_run", json!({}))?;
    let run_dir = PathBuf::from(strval(&run, "dir"));
    let (ok, lease) = vibeos_bench::lease::take(
        &owner,
        minutes,
        &format!("vibeos-cli {board} boot"),
        &env,
        std::process::id(),
    )
    .map_err(lease_error)?;
    let mut c = Context {
        root,
        board,
        args,
        env,
        run_dir,
        host,
    };
    if !ok {
        let detail = vibeos_bench::lease::describe(&lease, None).map_err(lease_error)?;
        return c.fail("BENCH_BUSY", "lease", "busy", json!([detail]));
    }
    let result = (|| -> Result<Value> {
        c.stage(
            "lease",
            "ok",
            if held { "held" } else { "taken" },
            json!([]),
        )?;
        if !c.stale(request)? {
            return c.finish();
        }
        if board == "pi4" {
            pi4(&mut c)
        } else {
            pi5(&mut c)
        }
    })();
    if !held {
        let released = vibeos_bench::lease::drop(&owner, false, &c.env).map_err(lease_error)?;
        if !released.0 {
            return Err(Error::new(
                "failed to release bench lease: ownership changed",
            ));
        }
    }
    result
}
fn head(root: &Path, short: bool) -> Result<String> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("rev-parse");
    if short {
        cmd.arg("--short=12");
    }
    cmd.arg("HEAD").current_dir(root);
    let r = vibeos_build::artifact_executor::supervised_output(
        &mut cmd,
        Some(std::time::Duration::from_secs(10)),
    )?;
    if r.code != 0 {
        return Err(Error::new(format!("git rev-parse failed: {}", r.stderr)));
    }
    Ok(r.stdout.trim().into())
}
fn serial_state(c: &Context<'_>) -> Value {
    h::read_json(&crate::bench_readonly::bench(&c.env, c.board).join("serial-current.json"))
}
fn command_ok(v: &Value) -> bool {
    v.get("returncode").and_then(Value::as_i64) == Some(0) && !flag(v, "timed_out")
}
fn command_tail(v: &Value) -> Vec<String> {
    let out = if strval(v, "stderr").is_empty() {
        strval(v, "stdout")
    } else {
        strval(v, "stderr")
    };
    out.lines()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(str::to_owned)
        .collect()
}
fn captured(c: &mut Context<'_>, serial: &Path, start: u64, text: &str) -> Result<()> {
    c.extra("live_serial", json!(serial))?;
    c.extra("serial_offset", json!(start))?;
    c.save_serial(text)
}
fn finish_desktop(
    c: &mut Context<'_>,
    serial: &Path,
    start: u64,
    timeout: f64,
    boot_started: std::time::Instant,
) -> Result<Value> {
    let (text, reached) = h::wait_desktop(serial, start, timeout);
    captured(c, serial, start, &text)?;
    if reached {
        c.extra(
            "desktop_s",
            json!((boot_started.elapsed().as_secs_f64() * 10.0).round() / 10.0),
        )?;
        c.stage("desktop", "ok", h::MARKER, json!([]))?;
        if flag(c.args, "ladder") {
            ladder(c, serial, start)?;
        } else {
            await_until(c, serial, start)?;
        }
    } else {
        let v = crate::verdict::classify(
            &text,
            false,
            None,
            None,
            "",
            &crate::verdict::known_flakes(),
        );
        let class = strval(&v, "classification");
        let mut salient = v.get("salient").cloned().unwrap_or(json!([]));
        if ["BUGCHECK", "KNOWN_FLAKE"].contains(&class) {
            if let Some(sym) = crate::verdict::symbolicate_pc(c.root, "arm64", &text) {
                if let Some(a) = salient.as_array_mut() {
                    a.push(json!(sym));
                }
            }
        }
        c.class(class)?;
        c.stage("desktop", "fail", class, salient)?;
    }
    screenshot(c)?;
    c.finish()
}
struct DoctorPrefix<'a>(&'a mut dyn Host);
impl Host for DoctorPrefix<'_> {
    fn call(&mut self, op: &str, mut args: Value) -> Result<Value> {
        if op == "stage" {
            if let Some(name) = args.get("name").and_then(Value::as_str).map(str::to_owned) {
                if name == "payload" {
                    return Ok(Value::Null);
                }
                args["name"] = json!(format!("doctor:{name}"));
            }
        }
        self.0.call(op, args)
    }
}
fn pi4(c: &mut Context<'_>) -> Result<Value> {
    if !flag(c.args, "skip_doctor") {
        let mut prefix = DoctorPrefix(&mut *c.host);
        crate::bench_readonly::doctor(c.root, &c.env, "pi4", None, &mut prefix)?;
        if strval(&c.call("get_run", json!({}))?, "verdict") == "FAIL" {
            c.class("BENCH_UNHEALTHY")?;
            return c.finish();
        }
    }
    let (rc, out) = crate::bench_readonly::run(c.root, &c.env, "pi4", "power.sh", &["status"], 60)?;
    let power = h::parse_power(&out);
    if rc != 0
        || power.get("on").is_none()
        || power["on"] == true && power["watts"].as_f64().unwrap_or(f64::INFINITY) > h::PI_WATTS_MAX
    {
        return c.fail(
            "BENCH_UNHEALTHY",
            "plug",
            "plug identity or power draw unsafe",
            json!([out]),
        );
    }
    let (rc, out) = crate::bench_readonly::run(c.root, &c.env, "pi4", "pico.sh", &["usb"], 60)?;
    let usb = h::parse_usb(&out);
    if rc != 0
        || usb.get("error").is_some()
        || (usb.get("target_powered").is_some() && usb["target_powered"] != power["on"])
        || (power["on"] == false && usb["mounted"] == true && usb["suspended"] == false)
    {
        return c.fail(
            "BENCH_UNHEALTHY",
            "pico",
            "Pico identity or USB state unsafe",
            json!([out]),
        );
    }
    let kernel = if strval(c.args, "kernel").is_empty() {
        "mp"
    } else {
        strval(c.args, "kernel")
    }
    .to_owned();
    c.env.insert("PI4_KERNEL".into(), kernel.clone());
    let stage_args = if flag(c.args, "allow_stale") {
        vec!["stage", "--allow-stale"]
    } else {
        vec!["stage"]
    };
    let v = c.command("bench.sh", &stage_args, 300, "stage")?;
    let manifest =
        h::read_json(&crate::bench_readonly::bench(&c.env, "pi4").join("payload/manifest.json"));
    let head = head(c.root, false)?;
    if !command_ok(&v) || strval(&manifest, "source_head") != head {
        return c.fail(
            "STAGE_FAIL",
            "stage",
            &format!("exit {}", v["returncode"]),
            json!(command_tail(&v)),
        );
    }
    c.extra("payload", manifest)?;
    c.stage(
        "stage",
        "ok",
        &format!(
            "{} kernel={kernel}",
            head.chars().take(12).collect::<String>()
        ),
        json!([]),
    )?;
    let state = serial_state(c);
    let mode = c
        .env
        .get("PI4_SERIAL")
        .map(String::as_str)
        .unwrap_or("pico");
    let host = c
        .env
        .get("PI4_PICO_HOST")
        .map(String::as_str)
        .unwrap_or("picousb");
    let port = c
        .env
        .get("PICO_SERIAL_PORT")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8082);
    if mode == "pico"
        && !h::listener_valid(
            &state,
            &crate::bench_readonly::bench(&c.env, "pi4"),
            host,
            port,
        )
    {
        return c.fail(
            "BENCH_UNHEALTHY",
            "serial",
            "listener identity does not match bench",
            json!([]),
        );
    }
    let Some(serial) = h::serial_file(&state) else {
        return c.fail("BENCH_UNHEALTHY", "serial", "no capture file", json!([]));
    };
    let start = fs::metadata(&serial).map(|m| m.len()).unwrap_or(0);
    c.extra("live_serial", json!(serial))?;
    c.extra("serial_offset", json!(start))?;
    let boot_started = std::time::Instant::now();
    let v = c.command("netboot.sh", &[], 600, "netboot")?;
    if !command_ok(&v) {
        let text = h::serial_slice(&serial, start);
        c.save_serial(&text)?;
        c.class("NETBOOT_FAIL")?;
        c.stage(
            "netboot",
            "fail",
            &format!("exit {}", v["returncode"]),
            json!(command_tail(&v)),
        )?;
        screenshot(c)?;
        return c.finish();
    }
    c.stage(
        "netboot",
        "ok",
        command_tail(&v).last().map(String::as_str).unwrap_or(""),
        json!([]),
    )?;
    let timeout = c
        .args
        .get("timeout")
        .and_then(Value::as_f64)
        .unwrap_or(300.0);
    finish_desktop(c, &serial, start, timeout, boot_started)
}
fn screenshot(c: &mut Context<'_>) -> Result<()> {
    let shot = c.run_dir.join("hdmi.png");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut captured = false;
    loop {
        let path = shot.to_string_lossy().into_owned();
        let (rc, _) = crate::bench_readonly::run(c.root, &c.env, c.board, "hdmi.sh", &[&path], 60)?;
        if rc == 0 && shot.is_file() {
            captured = true;
            c.extra("screenshot", json!(shot))?;
            if h::brightness(&shot).is_none_or(|level| level > 12.0) {
                return Ok(());
            }
        }
        if std::time::Instant::now() > deadline {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    c.stage(
        "screenshot",
        "warn",
        if captured {
            "screen still dark after 30s"
        } else {
            "no frame (capture card busy?)"
        },
        json!([]),
    )
}
fn await_until(c: &mut Context<'_>, serial: &Path, start: u64) -> Result<()> {
    let needle = strval(c.args, "until").to_owned();
    if needle.is_empty() {
        return Ok(());
    }
    let timeout = c
        .args
        .get("until_timeout")
        .and_then(Value::as_f64)
        .unwrap_or(180.0);
    if !timeout.is_finite() || timeout < 0.0 {
        return Err("invalid until timeout".into());
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs_f64(timeout);
    loop {
        let text = h::serial_slice(serial, start);
        if let Some(hit) = text.lines().find(|l| l.contains(&needle)) {
            c.save_serial(&text)?;
            return c.stage(
                "until",
                "ok",
                &hit.trim().chars().take(200).collect::<String>(),
                json!([]),
            );
        }
        if std::time::Instant::now() > deadline {
            c.save_serial(&text)?;
            c.class("HANG_TIMEOUT")?;
            return c.stage(
                "until",
                "fail",
                &format!("{needle:?} not seen {timeout:.0}s after the desktop"),
                json!(text
                    .lines()
                    .rev()
                    .take(6)
                    .map(|l| l.trim().chars().take(200).collect::<String>())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()),
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
struct BoardTail {
    path: PathBuf,
    pos: u64,
    t0: std::time::Instant,
    times: BTreeMap<String, f64>,
    previous: String,
}
impl BoardTail {
    fn stamp(&mut self, text: &str, config: &crate::ladder::Config) {
        let window = format!("{}{}", self.previous, text);
        let now = (self.t0.elapsed().as_secs_f64() * 10.0).round() / 10.0;
        for (name, _) in config.ladder.iter().chain(config.optional.iter()) {
            for (event, marker) in [
                ("launch", format!("{name}.exe (")),
                ("exit", format!("{name}.exe exit = ")),
            ] {
                if window.contains(&marker) {
                    self.times.entry(format!("{name} {event}")).or_insert(now);
                }
            }
        }
        if window.contains(crate::ladder::PROGRAM_DONE) {
            self.times.entry("program_done".into()).or_insert(now);
        }
        self.previous = text
            .chars()
            .rev()
            .take(200)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
    }
}
impl crate::ladder_watch::Stream for BoardTail {
    fn now(&mut self) -> Result<f64> {
        Ok(self.t0.elapsed().as_secs_f64())
    }
    fn read_serial(&mut self, timeout: f64) -> Result<String> {
        std::thread::sleep(std::time::Duration::from_secs_f64(timeout));
        let mut bytes = Vec::new();
        if let Ok(mut file) = fs::File::open(&self.path) {
            use std::io::{Read, Seek, SeekFrom};
            if file.seek(SeekFrom::Start(self.pos)).is_ok() {
                let _ = file.read_to_end(&mut bytes);
            }
        }
        self.pos += bytes.len() as u64;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        self.stamp(&text, &crate::ladder::Config::default());
        Ok(text)
    }
}
fn ladder(c: &mut Context<'_>, serial: &Path, start: u64) -> Result<()> {
    use crate::{ladder as l, ladder_watch::Stream};
    let t0 = std::time::Instant::now();
    let mut tail = BoardTail {
        path: serial.into(),
        pos: start,
        t0,
        times: BTreeMap::new(),
        previous: String::new(),
    };
    let seen = tail.read_serial(0.0)?;
    let cfg = l::Config::default();
    let staged = l::media_optional_staged(&cfg, c.root, "arm64", None);
    let floor = l::drain_floor(c.env.get("VIBEOS_LADDER_DRAIN_FLOOR").map(String::as_str))?;
    let text = format!(
        "{}{}",
        seen,
        crate::ladder_watch::watch(
            &mut tail,
            &cfg,
            l::TIMEOUT_SECONDS,
            &seen,
            &staged,
            true,
            floor,
            l::OPTIONAL_EXTRA_SECONDS,
            l::WINETEST_EXTRA_SECONDS
        )?
    );
    let durations: BTreeMap<String, f64> = cfg
        .ladder
        .iter()
        .chain(cfg.optional.iter())
        .filter_map(|(name, _)| {
            Some((
                name.clone(),
                ((tail.times.get(&format!("{name} exit"))?
                    - tail.times.get(&format!("{name} launch"))?)
                    * 10.0)
                    .round()
                    / 10.0,
            ))
        })
        .collect();
    c.extra(
        "ladder_drain_s",
        json!((t0.elapsed().as_secs_f64() * 10.0).round() / 10.0),
    )?;
    c.extra("ladder_times", json!(tail.times))?;
    c.extra("ladder_durations", json!(durations))?;
    c.extra("ladder_verdicts", json!(l::verdicts(&text)?))?;
    c.save_serial(&text)?;
    let result = l::check(&cfg, &text, &staged)?;
    if !result.is_null() {
        let class = strval(&result, "classification");
        c.class(class)?;
        c.stage(
            "ladder",
            "fail",
            class,
            result.get("salient").cloned().unwrap_or(json!([])),
        )
    } else {
        c.stage("ladder", "ok", &l::summary(&cfg, &text)?, json!([]))
    }
}
fn pi5(c: &mut Context<'_>) -> Result<Value> {
    if !crate::bench_readonly::ssh(c.root, &c.env)? {
        if !crate::bench_readonly::configured(c.root, &c.env)? {
            return c.fail(
                "BENCH_UNHEALTHY",
                "stager",
                "no SSH, no plug named",
                json!(["power-cycle the Pi 5 by hand, or set PI5_PLUG"]),
            );
        }
        let (rc, out) =
            crate::bench_readonly::run(c.root, &c.env, "pi5", "power.sh", &["status"], 60)?;
        let power = h::parse_power(&out);
        if rc != 0
            || power.get("on").is_none()
            || power["on"] == true
                && power["watts"].as_f64().unwrap_or(f64::INFINITY) > h::PI_WATTS_MAX
        {
            return c.fail(
                "BENCH_UNHEALTHY",
                "plug",
                "plug identity or power draw unsafe",
                json!([out]),
            );
        }
        let cycle = c.command("power.sh", &["cycle", "5"], 60, "power-cycle")?;
        if !command_ok(&cycle) {
            return c.fail(
                "BENCH_UNHEALTHY",
                "power-cycle",
                "failed",
                json!(command_tail(&cycle)),
            );
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(150);
        while !crate::bench_readonly::ssh(c.root, &c.env)? {
            if std::time::Instant::now() > deadline {
                c.class("BENCH_UNHEALTHY")?;
                c.stage("stager", "fail", "no SSH 150s after power-on", json!([]))?;
                screenshot(c)?;
                return c.finish();
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
    c.stage("stager", "ok", "Linux up", json!([]))?;
    let state = serial_state(c);
    let host = c
        .env
        .get("PI5_PICO_HOST")
        .map(String::as_str)
        .unwrap_or("none.invalid");
    let serial = if h::listener_valid(
        &state,
        &crate::bench_readonly::bench(&c.env, "pi5"),
        host,
        c.env
            .get("PICO_SERIAL_PORT")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8082),
    ) {
        h::serial_file(&state)
    } else {
        None
    };
    let start = serial
        .as_ref()
        .and_then(|p| fs::metadata(p).ok())
        .map(|m| m.len())
        .unwrap_or(0);
    let stortest = flag(c.args, "stortest");
    if stortest && serial.is_none() {
        return c.fail(
            "BENCH_UNHEALTHY",
            "stortest",
            "no serial console",
            json!(["tools/pi5/serial.sh up"]),
        );
    }
    if stortest {
        let (rc, out) = crate::bench_readonly::run(c.root, &c.env, "pi5", "pico.sh", &["usb"], 60)?;
        let usb = h::parse_usb(&out);
        if rc != 0
            || usb.get("error").is_some()
            || usb["mounted"] != true
            || usb
                .get("target_powered")
                .is_some_and(|v| v.as_bool() != Some(true))
        {
            return c.fail(
                "BENCH_UNHEALTHY",
                "pico",
                "Pico USB is not mounted on the powered board",
                json!([out]),
            );
        }
    }
    let image = if stortest {
        let path = c.run_dir.join("vibeusb.img");
        h::stortest_image(&path)?;
        Some(path)
    } else {
        None
    };
    let mut options: Vec<String> = c
        .args
        .get("options")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    if stortest {
        options.push("/STORTEST".into())
    }
    if !options.is_empty() {
        c.env
            .insert("VIBEOS_LOADER_OPTIONS".into(), options.join(" "));
    }
    let stage_args = if flag(c.args, "allow_stale") {
        vec!["--allow-stale"]
    } else {
        vec![]
    };
    let result = c.command("stage.sh", &stage_args, 600, "stage")?;
    let manifest = h::read_json(&crate::bench_readonly::bench(&c.env, "pi5").join("payload.json"));
    let head = head(c.root, true)?;
    if !command_ok(&result) || strval(&manifest, "commit") != head {
        return c.fail(
            "STAGE_FAIL",
            "stage",
            &format!("exit {}", result["returncode"]),
            json!(command_tail(&result)),
        );
    }
    c.extra("payload", manifest)?;
    c.stage(
        "stage",
        "ok",
        &format!("{head} (tryboot) {}", options.join(" ")),
        json!([]),
    )?;
    let Some(image) = image else {
        return pi5_await(c, serial.as_deref(), start, None);
    };
    let mut server =
        h::ImageServer::start(c.root, &image, &c.run_dir.join("serve-image.log"), &c.env)?;
    std::thread::sleep(std::time::Duration::from_secs(5));
    if server.poll()? {
        let (rc, _) = server.lost.unwrap_or((-1, 0.0));
        return c.fail(
            "BENCH_UNHEALTHY",
            "serve-image",
            &format!("exit {rc}"),
            json!([format!("see {}", server.log.display())]),
        );
    }
    c.stage("serve-image", "ok", "VIBEUSB writable", json!([]))?;
    let result = pi5_await(c, serial.as_deref(), start, Some(&mut server));
    server.stop()?;
    let result = result?;
    let state = c.call("get_run", json!({}))?;
    if !strval(&state, "classification").is_empty() {
        return Ok(result);
    }
    if flag(c.args, "drop_medium") {
        return c.finish();
    }
    let read = h::stortest_readback(&image, &c.env)?;
    if read.as_deref() != Some(h::STORTEST_WROTE) {
        return c.fail(
            "STORTEST_FAIL",
            "readback",
            if read.is_none() {
                "WRITETST.TXT missing"
            } else {
                "WRITETST.TXT differs"
            },
            json!([read]),
        );
    }
    c.stage(
        "readback",
        "ok",
        "WRITETST.TXT on the image matches",
        json!([]),
    )?;
    c.finish()
}
fn pi5_await(
    c: &mut Context<'_>,
    serial: Option<&Path>,
    start: u64,
    mut medium: Option<&mut h::ImageServer>,
) -> Result<Value> {
    let timeout = c
        .args
        .get("timeout")
        .and_then(Value::as_f64)
        .unwrap_or(300.0);
    if let Some(serial) = serial {
        c.extra("live_serial", json!(serial))?;
        c.extra("serial_offset", json!(start))?;
        let waiting = h::wait_desktop_with_medium(
            serial,
            start,
            timeout,
            medium.as_deref_mut(),
            flag(c.args, "drop_medium"),
        );
        let (text, reached) = match waiting {
            Ok(v) => v,
            Err(e) => {
                c.class("BENCH_UNHEALTHY")?;
                c.stage("serve-image", "fail", &e.to_string(), json!([]))?;
                screenshot(c)?;
                return c.finish();
            }
        };
        c.save_serial(&text)?;
        if reached {
            c.stage("desktop", "ok", h::MARKER, json!([]))?;
            if let Some(server) = medium.as_deref_mut() {
                await_stortest(c, serial, start, server)?;
            } else {
                await_until(c, serial, start)?;
            }
        } else {
            let verdict = crate::verdict::classify(
                &text,
                false,
                None,
                None,
                "",
                &crate::verdict::known_flakes(),
            );
            let class = strval(&verdict, "classification");
            let mut salient = verdict.get("salient").cloned().unwrap_or(json!([]));
            if ["BUGCHECK", "KNOWN_FLAKE"].contains(&class) {
                if let Some(sym) = crate::verdict::symbolicate_pc(c.root, "arm64", &text) {
                    if let Some(a) = salient.as_array_mut() {
                        a.push(json!(sym));
                    }
                }
            }
            c.class(class)?;
            c.stage("desktop", "fail", class, salient)?;
        }
        screenshot(c)?;
        if medium.is_some() && strval(&c.call("get_run", json!({}))?, "classification").is_empty() {
            return Ok(json!(0));
        }
        return c.finish();
    }
    let frames = c.run_dir.join("hdmi-frames");
    fs::create_dir_all(&frames)?;
    let shot = c.run_dir.join("hdmi.png");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs_f64(timeout);
    let mut n = 0;
    let mut share = None;
    while std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_secs(10));
        let frame = frames.join(format!("hdmi-{n:03}.png"));
        let path = frame.to_string_lossy().into_owned();
        let (rc, _) = crate::bench_readonly::run(c.root, &c.env, "pi5", "hdmi.sh", &[&path], 60)?;
        if rc == 0 {
            n += 1;
            fs::copy(&frame, &shot)?;
            c.extra("screenshot", json!(shot))?;
            share = h::desktop_share(&shot);
            if share.is_some_and(|s| s >= 0.2) {
                c.stage(
                    "desktop",
                    "warn",
                    &format!("seen on HDMI ({:.0}% desktop)", share.unwrap() * 100.0),
                    json!(["no serial console: HDMI verdict only (tools/pi5/serial.sh up)"]),
                )?;
                return c.finish();
            }
        }
    }
    c.fail(
        "HANG_TIMEOUT",
        "desktop",
        "no desktop on HDMI",
        json!([share
            .map(|v| format!("last frame {:.0}% desktop", v * 100.0))
            .unwrap_or("no frame".into())]),
    )
}
fn await_stortest(
    c: &mut Context<'_>,
    serial: &Path,
    start: u64,
    server: &mut h::ImageServer,
) -> Result<()> {
    let drop_requested = flag(c.args, "drop_medium");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
    loop {
        let text = h::serial_slice(serial, start);
        let (verdict, lines) = h::stortest_verdict(&text);
        if drop_requested && !server.dropped && h::drop_marker(&text) && !server.poll()? {
            server.drop_medium()?;
        }
        let lost = server.poll()?;
        if verdict.is_some() || std::time::Instant::now() > deadline || lost && !server.dropped {
            c.save_serial(&text)?;
            c.extra("stortest", json!(lines))?;
            if drop_requested {
                let grade = h::grade_drop(&text, server.dropped, lost);
                let class = strval(&grade, "classification");
                if !class.is_empty() {
                    c.class(class)?;
                }
                return c.stage(
                    strval(&grade, "stage"),
                    strval(&grade, "status"),
                    strval(&grade, "detail"),
                    grade["salient"].clone(),
                );
            }
            if lost && verdict != Some("ok") {
                c.class("BENCH_UNHEALTHY")?;
                let (rc, elapsed) = server.lost.unwrap_or((-1, 0.));
                return c.stage("serve-image","fail",&format!("image server exited {rc} {elapsed:.0}s after the reboot: the Pico dropped the medium"),json!([format!("see {}",server.log.display())]));
            }
            if verdict == Some("ok") {
                return c.stage(
                    "stortest",
                    "ok",
                    lines.last().map(String::as_str).unwrap_or("ALL OK"),
                    json!([]),
                );
            }
            c.class(if verdict.is_some() {
                "STORTEST_FAIL"
            } else {
                "HANG_TIMEOUT"
            })?;
            return c.stage(
                "stortest",
                "fail",
                if verdict.is_some() {
                    "failed"
                } else {
                    "no verdict 240s after the desktop"
                },
                json!(lines
                    .iter()
                    .rev()
                    .take(6)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()),
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(if drop_requested {
            500
        } else {
            5000
        }));
    }
}
