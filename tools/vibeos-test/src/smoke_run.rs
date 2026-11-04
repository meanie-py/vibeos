//! Smoke control flow, admission boundaries, and external suite verdict aggregation.
use crate::smoke_checks::{self, Env};
use crate::verdict::{args, py_lines, re};
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use yaml_rust::{Yaml, YamlLoader};
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn flag(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
}
fn base_arch(s: &str) -> String {
    s.replace("-net", "")
        .replace("-uefi", "")
        .replace("-bios", "")
        .replace("-test", "")
}
fn vm_arch(s: &str) -> Result<String> {
    if s == "amd64-bios" {
        return Ok(s.into());
    }
    Ok(match base_arch(s).as_str() {
        "i386" => "i386-uefi",
        "amd64" => "amd64-uefi",
        "arm64" => "arm64",
        _ => return Err(Error::new(format!("unknown smoke arch {s}"))),
    }
    .into())
}
fn summary(output: &str) -> String {
    let lines = py_lines(output.trim());
    lines
        .iter()
        .rev()
        .find(|l| l.contains(" passed") || l.contains(" failed") || l.contains(" error"))
        .or_else(|| lines.last())
        .map(|s| s.trim_matches(['=', ' ']).to_owned())
        .unwrap_or_default()
}
fn logfiles(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("_serial.log"))
        {
            files.push(p)
        }
    }
    files.sort();
    Ok(files)
}
fn read_lossy(p: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&fs::read(p)?).into_owned())
}
struct Context<'a> {
    root: &'a Path,
    a: &'a Value,
    host: &'a mut dyn Host,
    env: Env,
    run_dir: PathBuf,
}
impl<'a> Context<'a> {
    fn new(root: &'a Path, a: &'a Value, host: &'a mut dyn Host) -> Result<Self> {
        let env = serde_json::from_value(a.get("env").cloned().unwrap_or(json!({})))?;
        let dir = if text(a, "run_dir").is_empty() {
            let run = host.call("get_run", json!({}))?;
            PathBuf::from(text(&run, "dir"))
        } else {
            PathBuf::from(text(a, "run_dir"))
        };
        Ok(Self {
            root,
            a,
            host,
            env,
            run_dir: dir,
        })
    }
    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
        self.host.call(op, a)
    }
    fn state(&mut self) -> Result<Value> {
        self.call("get_run", json!({}))
    }
    fn update(&mut self, update: Value) -> Result<()> {
        self.call("set_run", update)?;
        Ok(())
    }
    fn classification(&mut self) -> Result<String> {
        Ok(text(&self.state()?, "classification").to_owned())
    }
    fn class(&mut self, class: &str) -> Result<()> {
        self.update(json!({"classification":class}))
    }
    fn stage(
        &mut self,
        name: &str,
        status: &str,
        detail: impl AsRef<str>,
        salient: Value,
    ) -> Result<()> {
        self.call(
            "stage",
            json!({"name":name,"status":status,"detail":detail.as_ref(),"salient":salient}),
        )?;
        Ok(())
    }
    fn log(&mut self, header: &str, message: &str) -> Result<()> {
        self.call("log", json!({"header":header,"text":message}))?;
        Ok(())
    }
    fn set_env(&mut self, key: &str, value: impl Into<String>) -> Result<()> {
        let value = value.into();
        self.env.insert(key.into(), value.clone());
        self.call("env", json!({"overlay":{key:value}}))?;
        Ok(())
    }
    fn token(&mut self, token: &str) -> Result<()> {
        let mut ts: Vec<_> = self
            .env
            .get("VIBEOS_TRIPWIRE")
            .map(|s| s.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default();
        if !ts.iter().any(|t| t == token) {
            ts.push(token.into());
            self.set_env("VIBEOS_TRIPWIRE", ts.join(" "))?
        }
        Ok(())
    }
    fn command(
        &mut self,
        argv: Vec<String>,
        env: &Env,
        timeout: u64,
        stage: &str,
    ) -> Result<Value> {
        self.call(
            "command",
            json!({"argv":argv,"cwd":self.root,"env":env,"timeout":timeout,"stage":stage}),
        )
    }
    fn provenance(&mut self, op: &str, arch: &str, extra: Value) -> Result<Value> {
        let mut a = extra.as_object().cloned().unwrap_or_default();
        a.insert("arch".into(), json!(arch));
        a.insert("env".into(), json!(self.env));
        if let Some(cfg) = self.a.get("cfgs").and_then(|v| v.get(arch)) {
            a.insert("cfg".into(), cfg.clone());
        }
        if let Some(store) = self.a.get("store_root") {
            a.insert("store_root".into(), store.clone());
        }
        self.call(op, Value::Object(a))
    }
    fn finish(&mut self) -> Result<Value> {
        self.call("finalize", json!({}))
    }
    fn python(&self) -> String {
        let p = text(self.a, "python");
        if p.is_empty() {
            "python3".into()
        } else {
            p.into()
        }
    }
}
pub fn payload_build_flags(root: &Path, names: &[String]) -> Result<String> {
    for name in names {
        let spec = root.join("tools/payloads").join(name).join("payload.yaml");
        if spec.is_file()
            && re(r"modules/rostests/(?:winetests|apitests)/").is_match(&read_lossy(&spec)?)
        {
            return Ok(" --rostests --keep-going".into());
        }
    }
    Ok(String::new())
}
pub fn guest_reached_desktop(results: &[Value]) -> bool {
    results.iter().any(|r| flag(r, "ok"))
        && results
            .iter()
            .all(|r| flag(r, "ok") || text(r, "classification") == "LAUNCH_FAIL")
}
pub fn payload_crash(root: &Path, arch: &str, dir: &Path, catalog: &Value) -> Result<Value> {
    for p in logfiles(dir)? {
        let serial = read_lossy(&p)?;
        let mut tri = crate::verdict::classify(&serial, true, None, None, "", catalog);
        if matches!(
            text(&tri, "classification"),
            "BUGCHECK" | "RUST_PANIC" | "KNOWN_FLAKE"
        ) {
            if let Some(sym) = crate::verdict::symbolicate_pc(root, arch, &serial) {
                tri["salient"]
                    .as_array_mut()
                    .ok_or("triage salient is not an array")?
                    .push(json!(sym))
            }
            return Ok(tri);
        }
    }
    Ok(Value::Null)
}
fn net_stage(c: &mut Context<'_>) -> Result<()> {
    let test = c.root.join("tests/qemu/test_net.py");
    if !test.exists() {
        return c.stage("net", "skip", "no test_net.py", json!([]));
    }
    let cmd = vec![
        c.python(),
        "-m".into(),
        "pytest".into(),
        test.to_string_lossy().into_owned(),
        "--arch".into(),
        "amd64-uefi-net".into(),
        "--timeout".into(),
        "600".into(),
        "-q".into(),
        "-p".into(),
        "no:cacheprovider".into(),
        "--skip-clean-build".into(),
    ];
    let env = smoke_checks::net_suite_env(&c.env, &c.run_dir.join("net").to_string_lossy());
    let proc = c.command(cmd, &env, 1200, "net")?;
    if proc
        .get("timed_out")
        .and_then(Value::as_bool)
        .unwrap_or(proc["returncode"].as_i64() == Some(124))
    {
        return c.stage(
            "net",
            "fail",
            "net suite timed out",
            json!(["tests/qemu/test_net.py hung > 1200s"]),
        );
    }
    let stdout = text(&proc, "stdout");
    let stderr = text(&proc, "stderr");
    c.log("network pytest", &format!("{stdout}{stderr}"))?;
    let summary = summary(stdout);
    if proc["returncode"].as_i64() == Some(0) {
        return c.stage("net", "ok", summary, json!([]));
    }
    let output = format!("{stdout}\n{stderr}");
    let starved = output.contains("[hostlock] waiting for host CPU lease")
        && !output.contains("[hostlock] lease acquired");
    let run = c.state()?;
    let stages: Vec<_> = run["stages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|s| text(s, "name") == "boot" || text(s, "name").starts_with("run"))
        .collect();
    let green = !stages.is_empty() && stages.iter().all(|s| text(s, "status") == "ok");
    if text(&run, "classification").is_empty() {
        if starved {
            c.class("HOST_STARVED")?
        } else if green {
            c.class("NET_FLAKE")?
        }
    }
    c.stage("net","fail","net suite FAILED",json!([if starved{"net lane never booted — queued on host CPU leases (§9.84); `vibeos-cli ps` names the holder"}else{&summary},"repro: pytest tests/qemu/test_net.py --arch amd64-uefi-net"]))
}
#[derive(Debug)]
struct PayloadPlan {
    pytest_arch: String,
    drain: f64,
    memory: String,
    extra: Vec<String>,
    refusal: Option<String>,
}
fn yaml_number(v: &Yaml) -> Result<f64> {
    match v {
        Yaml::Integer(n) => Ok(*n as f64),
        Yaml::Real(s) | Yaml::String(s) => s
            .parse()
            .map_err(|_| Error::new("payload launch_drain_secs is not numeric")),
        _ => Err("payload launch_drain_secs is not numeric".into()),
    }
}
fn yaml_string(v: &Yaml) -> String {
    match v {
        Yaml::String(s) | Yaml::Real(s) => s.clone(),
        Yaml::Integer(n) => n.to_string(),
        Yaml::Boolean(b) => if *b { "True" } else { "False" }.into(),
        Yaml::Null => "None".into(),
        Yaml::BadValue => String::new(),
        _ => format!("{v:?}"),
    }
}
fn yaml_truthy(v: &Yaml) -> bool {
    !matches!(v, Yaml::Null | Yaml::BadValue | Yaml::Boolean(false))
        && match v {
            Yaml::Integer(n) => *n != 0,
            Yaml::Real(s) => s.parse::<f64>().map(|n| n != 0.0).unwrap_or(true),
            Yaml::String(s) => !s.is_empty(),
            Yaml::Array(a) => !a.is_empty(),
            Yaml::Hash(h) => !h.is_empty(),
            _ => true,
        }
}
fn payload_plan(
    root: &Path,
    payloads: &str,
    arch: &str,
    boot_arch: Option<&str>,
    memory_override: &str,
) -> Result<PayloadPlan> {
    let mut p = PayloadPlan {
        pytest_arch: boot_arch
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .unwrap_or(vm_arch(arch)?),
        drain: 0.,
        memory: String::new(),
        extra: Vec::new(),
        refusal: None,
    };
    for name in payloads.split(',') {
        let name = name.trim();
        let path = root.join("tools/payloads").join(name).join("payload.yaml");
        if !path.is_file() {
            continue;
        }
        let docs = YamlLoader::load_from_str(&fs::read_to_string(&path)?)
            .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
        let doc = docs.first().unwrap_or(&Yaml::Null);
        if !matches!(doc, Yaml::Null | Yaml::Hash(_)) {
            return Err(Error::new(format!(
                "{} must contain a mapping",
                path.display()
            )));
        }
        let test = &doc["test"];
        if !matches!(test, Yaml::Null | Yaml::BadValue | Yaml::Hash(_)) {
            return Err(Error::new(format!(
                "{}.test must contain a mapping",
                path.display()
            )));
        }
        let drain = if matches!(test["launch_drain_secs"], Yaml::BadValue) {
            30.
        } else {
            yaml_number(&test["launch_drain_secs"])?
        };
        if !drain.is_finite() || drain < 0. {
            return Err("payload launch_drain_secs must be finite and non-negative".into());
        }
        p.drain = p.drain.max(drain);
        let memory = yaml_string(&test["vm_memory"]);
        if smoke_checks::mem_mb(&memory)? > smoke_checks::mem_mb(&p.memory)? {
            p.memory = memory
        }
        if yaml_truthy(&test["qemu_extra_args"]) {
            p.extra.push(yaml_string(&test["qemu_extra_args"]))
        }
        if yaml_truthy(&test["net"]) && !p.pytest_arch.ends_with("-net") {
            if p.pytest_arch == "amd64-bios" {
                p.refusal=Some(format!("payload {name} declares net: true and there is no amd64-bios-net boot config; run it with --arch amd64"));
                return Ok(p);
            }
            p.pytest_arch += "-net"
        }
    }
    if !memory_override.is_empty() {
        p.memory = memory_override.into()
    }
    Ok(p)
}
fn payload_stage(
    c: &mut Context<'_>,
    arch: &str,
    payloads: &str,
    idx: u64,
    total: u64,
    memory_override: &str,
    boot_arch: Option<&str>,
) -> Result<()> {
    let p = payload_plan(c.root, payloads, arch, boot_arch, memory_override)?;
    let stage = if total == 1 {
        "payload".into()
    } else {
        format!("payload{}", idx + 1)
    };
    if let Some(refusal) = p.refusal {
        return c.stage(&stage, "fail", refusal, json!([]));
    }
    let budget = 600. + p.drain;
    if budget > u64::MAX as f64 / 2. {
        return Err("payload timeout exceeds integer range".into());
    }
    let timeout = budget as u64;
    let outer = timeout.checked_mul(2).ok_or("payload timeout overflow")?;
    let dir = c.run_dir.join(if idx == 0 {
        "payload".into()
    } else {
        format!("payload{}", idx + 1)
    });
    fs::create_dir_all(&dir)?;
    let cmd = vec![
        c.python(),
        "-m".into(),
        "pytest".into(),
        c.root
            .join("tests/qemu/test_payload.py")
            .to_string_lossy()
            .into_owned(),
        "--arch".into(),
        p.pytest_arch.clone(),
        "--payload".into(),
        payloads.into(),
        "--skip-clean-build".into(),
        "--timeout".into(),
        timeout.to_string(),
        "-q".into(),
        "-p".into(),
        "no:cacheprovider".into(),
    ];
    let mut env = c.env.clone();
    env.insert(
        "VIBEOS_ARTIFACT_DIR".into(),
        dir.to_string_lossy().into_owned(),
    );
    env.insert(
        "VIBEOS_QEMU_MAX_LIFETIME".into(),
        smoke_checks::payload_vm_lifetime(600., p.drain, &c.env)?.to_string(),
    );
    if !p.memory.is_empty() {
        env.insert("VIBEOS_QEMU_MEMORY".into(), p.memory);
    }
    if !p.extra.is_empty() {
        let mut extra = vec![env
            .get("VIBEOS_QEMU_EXTRA_ARGS")
            .cloned()
            .unwrap_or_default()];
        extra.extend(p.extra);
        env.insert(
            "VIBEOS_QEMU_EXTRA_ARGS".into(),
            extra.join(" ").trim().into(),
        );
    }
    let proc = c.command(cmd, &env, outer, &stage)?;
    if proc
        .get("timed_out")
        .and_then(Value::as_bool)
        .unwrap_or(proc["returncode"].as_i64() == Some(124))
    {
        return c.stage(
            &stage,
            "fail",
            format!("payload suite ({payloads}) timed out"),
            json!([format!("tests/qemu/test_payload.py hung > {outer}s")]),
        );
    }
    fs::write(
        dir.join("pytest.log"),
        format!("{}{}", text(&proc, "stdout"), text(&proc, "stderr")),
    )?;
    let summary = summary(text(&proc, "stdout"));
    let mut assertion = Value::Null;
    for path in logfiles(&dir)? {
        let hit = smoke_checks::kernel_assert_check(&read_lossy(&path)?);
        if !hit.is_null() {
            assertion = hit;
            break;
        }
    }
    let passed = proc["returncode"].as_i64() == Some(0);
    if passed && !assertion.is_null() {
        if matches!(c.classification()?.as_str(), "" | "NET_FLAKE") {
            c.class(text(&assertion, "classification"))?
        }
        let mut salient = assertion["salient"].as_array().cloned().unwrap_or_default();
        salient.push(json!(format!("serial + pytest.log: {}", dir.display())));
        return c.stage(
            &stage,
            "fail",
            format!("payload suite ({payloads}) passed with a failed kernel assertion"),
            json!(salient),
        );
    }
    if passed {
        return c.stage(&stage, "ok", summary, json!([]));
    }
    let catalog =
        c.a.get("known_flakes")
            .cloned()
            .unwrap_or_else(crate::verdict::known_flakes);
    let crash = payload_crash(c.root, arch, &dir, &catalog)?;
    let crash = if crash.is_null() { assertion } else { crash };
    if !crash.is_null() && matches!(c.classification()?.as_str(), "" | "NET_FLAKE") {
        c.class(text(&crash, "classification"))?
    }
    let mut salient = crash["salient"].as_array().cloned().unwrap_or_default();
    salient.extend([
        json!(summary),
        json!(format!(
            "repro: pytest tests/qemu/test_payload.py --arch {} --payload {payloads}",
            p.pytest_arch
        )),
        json!(format!("serial + pytest.log: {}", dir.display())),
    ]);
    c.stage(
        &stage,
        "fail",
        format!("payload suite ({payloads}) FAILED"),
        json!(salient),
    )
}
fn qemu_preflight(c: &mut Context<'_>, boot_arch: &str) -> Result<Value> {
    let cli = c.a.get("cli").cloned().unwrap_or(json!({}));
    if cli
        .pointer("/qemu/backend")
        .and_then(Value::as_str)
        .unwrap_or("native")
        != "windows-host"
    {
        return Ok(Value::Null);
    }
    let win = cli.pointer("/qemu/windows").cloned().unwrap_or(json!({}));
    let configured = win["accel"].as_str().unwrap_or("tcg");
    let accel = c.call(
        "backend_resolve_accel",
        json!({"arch":boot_arch,"configured":configured}),
    )?;
    let qemu = if boot_arch.starts_with("arm64") {
        "qemu-system-aarch64"
    } else if boot_arch.starts_with("i386") {
        "qemu-system-i386"
    } else {
        "qemu-system-x86_64"
    };
    let exe = c.call(
        "backend_windows_qemu_bin",
        json!({"native_bin":qemu,"qemu_dir":win["qemu_dir"].as_str().unwrap_or(""),"accel":accel}),
    )?;
    let version = c.call("backend_windows_qemu_version", json!({"exe":exe}))?;
    let version = version.as_str().unwrap_or("");
    let bad = crate::backend::known_bad_windows_qemu(version);
    let exe = exe.as_str().ok_or("QEMU executable was not a string")?;
    Ok(if let Some(bad) = bad {
        if c.env
            .get("VIBEOS_QEMU_ALLOW_KNOWN_BAD")
            .is_some_and(|v| v == "1")
        {
            json!({"name":"qemu","status":"warn","detail":format!("KNOWN-BAD {version} allowed by VIBEOS_QEMU_ALLOW_KNOWN_BAD=1 ({exe})"),"salient":[]})
        } else {
            json!({"name":"qemu","status":"fail","detail":format!("{version} ({bad})"),"salient":[format!("{exe} is a known-bad QEMU build. Point VIBEOS_QEMU_WINDOWS_DIR (or qemu.windows.qemu_dir) at a released build that passes the atodbl payload on i386.")]})
        }
    } else {
        json!({"name":"qemu","status":"ok","detail":format!("{} ({exe})",if version.is_empty(){"version unknown"}else{version}),"salient":[]})
    })
}
fn boot_evidence(c: &mut Context<'_>, results: &[Value]) -> Result<()> {
    let selected = results
        .iter()
        .find(|r| !flag(r, "ok"))
        .or_else(|| results.first())
        .ok_or("no boots were attempted")?;
    let mut extra = json!({"boots":results});
    if !text(selected, "serial_log").is_empty() {
        extra["live_log"] = selected["serial_log"].clone()
    }
    c.update(json!({"extra":extra}))
}
fn fail_preflight(
    c: &mut Context<'_>,
    class: &str,
    name: &str,
    detail: impl AsRef<str>,
    salient: Value,
) -> Result<Value> {
    c.class(class)?;
    c.stage(name, "fail", detail, salient)?;
    c.finish()
}
pub fn smoke(root: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let mut c = Context::new(root, a, host)?;
    let options = a.get("args").unwrap_or(a);
    let raw_arch = text(options, "arch");
    let arch = base_arch(raw_arch);
    let boot_arch = vm_arch(raw_arch)?;
    if flag(options, "timing") {
        c.set_env("VIBEOS_SERIAL_TIMESTAMPS", "1")?
    }
    let memory = text(options, "vm_memory");
    if !memory.is_empty() {
        if !re(r"^[1-9][0-9]*[MG]$").is_match(memory) {
            return fail_preflight(
                &mut c,
                "BAD_ARGS",
                "args",
                format!("--vm-memory {memory:?}"),
                json!(["give a qemu -m size such as 768M or 2G"]),
            );
        }
        c.set_env("VIBEOS_QEMU_MEMORY", memory)?;
        c.update(json!({"extra":{"vm_memory":memory}}))?
    }
    if !text(options, "media").is_empty() {
        let image = PathBuf::from(text(options, "media"));
        let image = if image.is_absolute() {
            image
        } else {
            root.join(image)
        };
        let image = fs::canonicalize(&image).unwrap_or(image);
        let bad: Vec<_> = ["payload", "doom", "ladder", "require_probe", "tripwire"]
            .into_iter()
            .filter(|key| {
                options.get(*key).is_some_and(|v| match v {
                    Value::Null => false,
                    Value::Bool(b) => *b,
                    Value::String(s) => !s.is_empty(),
                    Value::Array(a) => !a.is_empty(),
                    _ => true,
                })
            })
            .map(|key| format!("--media cannot combine with --{}", key.replace('_', "-")))
            .collect();
        let problems = if !bad.is_empty() {
            json!(bad)
        } else {
            c.provenance(
                "provenance_external_media_problems",
                &arch,
                json!({"image":image}),
            )?
        };
        if problems.as_array().is_some_and(|p| !p.is_empty()) {
            return fail_preflight(
                &mut c,
                if bad.is_empty() {
                    "STALE_MEDIA"
                } else {
                    "BAD_ARGS"
                },
                "media",
                image.to_string_lossy(),
                problems,
            );
        }
        c.set_env("VIBEOS_BOOT_MEDIA", image.to_string_lossy())?;
        c.update(json!({"extra":{"media":image}}))?;
        c.stage(
            "media",
            "ok",
            format!("{} (stamp matches this tree)", image.display()),
            json!([]),
        )?
    }
    let st = c.provenance("provenance_check_staleness", &arch, json!({}))?;
    let stale = st["stale"].as_bool().unwrap_or(!flag(&st, "fresh"));
    let allow = flag(options, "allow_stale");
    let reasons = st["reasons"].as_array().cloned().unwrap_or_default();
    if stale && !allow {
        let mut salient = reasons;
        salient.push(json!("run `vibeos-cli build` (or --allow-stale)"));
        return fail_preflight(&mut c, "STALE_MEDIA", "staleness", "stale", json!(salient));
    }
    if flag(&st, "warm") {
        c.stage("staleness", "warn", "WARM build", json!([]))?
    } else if stale && allow {
        c.stage(
            "staleness",
            "warn",
            "stale (--allow-stale)",
            json!(reasons.into_iter().take(2).collect::<Vec<_>>()),
        )?
    } else {
        c.stage("staleness", "ok", "current", json!([]))?
    }
    if flag(options, "ladder") {
        let absent = c.call("ladder_media_missing_runtime", json!({"arch":arch}))?;
        if absent.as_array().is_some_and(|a| !a.is_empty()) {
            let mut salient = absent.as_array().cloned().unwrap();
            let count = salient.len();
            salient.push(json!(format!("build i386 and amd64 first, then the arch: ./vibeos-cli build --arch i386 && ./vibeos-cli build --arch amd64{}",if arch=="arm64"{" && VIBEOS_FEX=1 ./vibeos-cli build --arch arm64"}else{""})));
            return fail_preflight(
                &mut c,
                "LADDER_MEDIA_INCOMPLETE",
                "ladder-media",
                format!("{count} guest runtime file(s) not on the media"),
                json!(salient),
            );
        }
    }
    let require: Vec<_> = options["require_probe"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .collect();
    if !require.is_empty() {
        let missing = c.provenance(
            "provenance_probes_missing_from_media",
            &arch,
            json!({"literals":require}),
        )?;
        if missing.as_array().is_some_and(|a| !a.is_empty()) {
            let mut salient = missing.as_array().cloned().unwrap();
            let n = salient.len();
            salient.push(json!("rebuild so the probe reaches the media, then re-run"));
            return fail_preflight(
                &mut c,
                "PROBE_NOT_IN_MEDIA",
                "probe-check",
                format!("{n} of {} probe(s) absent", require.len()),
                json!(salient),
            );
        }
        c.stage(
            "probe-check",
            "ok",
            format!("{} probe(s) present in every boot image", require.len()),
            json!([]),
        )?
    }
    let qemu = qemu_preflight(&mut c, &boot_arch)?;
    if !qemu.is_null() {
        c.call("stage", qemu.clone())?;
        if text(&qemu, "status") == "fail" {
            c.class("QEMU_KNOWN_BAD")?;
            return c.finish();
        }
    }
    let mut wanted: Vec<String> = text(options, "payload")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    if flag(options, "doom") && !wanted.iter().any(|p| p == "doom") {
        wanted.push("doom".into())
    }
    if !wanted.is_empty() {
        let staged = c.call("payload_stamp", json!({"arch":arch}))?;
        let staged: Vec<String> = serde_json::from_value(staged)?;
        let absent: Vec<_> = wanted
            .iter()
            .filter(|p| !staged.contains(p))
            .cloned()
            .collect();
        if !absent.is_empty() {
            let flags = payload_build_flags(root, &wanted)?;
            return fail_preflight(
                &mut c,
                "PAYLOAD_NOT_IN_MEDIA",
                "payload-check",
                format!(
                    "media carries {}, not {}",
                    if staged.is_empty() {
                        "<none>".into()
                    } else {
                        staged.join(",")
                    },
                    absent.join(",")
                ),
                json!([
                    format!(
                        "VIBEOS_PAYLOAD={} ./vibeos-cli build --arch {arch}{flags}",
                        wanted.join(",")
                    ),
                    "then re-run this smoke"
                ]),
            );
        }
        c.stage("payload-check", "ok", wanted.join(","), json!([]))?
    }
    let total = options["repeat"].as_i64().unwrap_or(1).max(1) as u64;
    let mut actual_total = total;
    let until = text(options, "until");
    let fail_on = options.get("fail_on").cloned().unwrap_or(Value::Null);
    let tripwire = text(options, "tripwire").trim();
    if !tripwire.is_empty() {
        c.set_env("VIBEOS_TRIPWIRE", tripwire)?;
        c.stage("tripwire", "warn", tripwire, json!([]))?
    }
    let smp = options["smp"].as_i64().unwrap_or(0);
    if smp != 0 {
        c.set_env("VIBEOS_QEMU_SMP", smp.to_string())?;
        c.stage("smp", "warn", format!("-smp {smp}"), json!([]))?
    }
    if flag(options, "ahci") {
        c.set_env("VIBEOS_AHCI_CD", "1")?;
        c.stage(
            "ahci",
            "warn",
            if arch == "arm64" {
                "disk + CD drive on ich9-ahci"
            } else {
                "boot CD on ich9-ahci"
            },
            json!([]),
        )?
    }
    if flag(options, "isa_ide") {
        if arch != "i386" {
            return fail_preflight(
                &mut c,
                "BAD_ARGS",
                "isa-ide",
                "--isa-ide is i386-only (legide)",
                json!([]),
            );
        }
        c.set_env("VIBEOS_QEMU_ISA_IDE", "1")?;
        c.token("LEGIDE_DETECT")?;
        c.stage("isa-ide", "warn", "disk on ISA IDE 0x1E8 IRQ 11", json!([]))?
    }
    if let Some(gic) = options.get("gic").filter(|v| !v.is_null()) {
        let gic = gic.as_i64().ok_or("gic must be an integer")?;
        if arch != "arm64" && gic != 2 {
            return fail_preflight(
                &mut c,
                "BAD_ARGS",
                "gic",
                "--gic 3 is arm64-only (x86 has no GIC)",
                json!([]),
            );
        }
        c.set_env("VIBEOS_QEMU_GIC", gic.to_string())?;
        if gic == 3 {
            c.stage("gic", "warn", "GICv3 + ITS", json!([]))?
        }
    }
    if flag(options, "smmu") {
        if arch != "arm64" {
            return fail_preflight(
                &mut c,
                "BAD_ARGS",
                "smmu",
                "--smmu is arm64-only",
                json!([]),
            );
        }
        c.set_env("VIBEOS_QEMU_SMMU", "1")?;
        c.stage("smmu", "warn", "SMMUv3 in front of PCI", json!([]))?
    }
    for (name, key, value) in [
        (
            "cpu",
            "VIBEOS_QEMU_CPU",
            options.get("cpu").and_then(Value::as_str),
        ),
        (
            "headless",
            "VIBEOS_QEMU_HEADLESS",
            flag(options, "headless").then_some("1"),
        ),
        (
            "boot-nvme",
            "VIBEOS_QEMU_BOOT_NVME",
            flag(options, "boot_nvme").then_some("1"),
        ),
    ] {
        if let Some(value) = value {
            if arch != "arm64" {
                return fail_preflight(
                    &mut c,
                    "BAD_ARGS",
                    name,
                    format!("--{name} is arm64-only"),
                    json!([]),
                );
            }
            c.set_env(key, value)?;
            c.stage(name, "warn", format!("{key}={value}"), json!([]))?
        }
    }
    if !text(options, "xhci").is_empty() {
        c.set_env("VIBEOS_QEMU_XHCI", "1")?;
        c.set_env("VIBEOS_QEMU_XHCI_MSI", text(options, "xhci"))?;
        c.stage(
            "xhci",
            "warn",
            format!("xHCI rig on {}", text(options, "xhci")),
            json!([]),
        )?
    }
    let bridge = flag(options, "nvme_bridge") || flag(options, "nvme_segment");
    if bridge {
        if arch != "arm64" {
            return fail_preflight(
                &mut c,
                "BAD_ARGS",
                "nvme",
                format!(
                    "{} is arm64-only",
                    if flag(options, "nvme_segment") {
                        "--nvme-segment"
                    } else {
                        "--nvme-bridge"
                    }
                ),
                json!([]),
            );
        }
        c.set_env("VIBEOS_QEMU_NVME_BRIDGE", "port")?
    }
    if flag(options, "nvme_segment") {
        c.set_env("VIBEOS_QEMU_NVME_BRIDGE", "segment")?;
        c.token("PCI_SEGMENT_SPLIT")?
    }
    if smoke_checks::nvme_default_on(flag(options, "nvme") || bridge, raw_arch, &c.env) {
        c.set_env("VIBEOS_QEMU_NVME", "1")?;
        c.stage("nvme", "warn", "NVMe controller attached", json!([]))?
    }
    let mut results = Vec::new();
    for idx in 0..total {
        let mut request = a.as_object().cloned().unwrap_or_default();
        request.extend(json!({"arch":arch,"boot_arch":boot_arch,"run_dir":c.run_dir,"env":c.env,"until":until,"fail_on":fail_on,"idx":idx,"total":total,"want_ladder":flag(options,"ladder"),"attach":flag(options,"attach_on_fail")}).as_object().unwrap().clone());
        let r = c.call("smoke_one_boot", json!(request))?;
        let ok = flag(&r, "ok");
        let name = if total > 1 {
            format!("run{}", idx + 1)
        } else {
            "boot".into()
        };
        c.call("stage",json!({"name":name,"status":if ok{"ok"}else{"fail"},"seconds":r.pointer("/timings/total_seconds").cloned().unwrap_or(json!(0)),"detail":if ok{"reached"}else{r.get("classification").and_then(Value::as_str).unwrap_or(text(&r,"outcome"))},"salient":r.get("salient").cloned().unwrap_or(json!([]))}))?;
        if !ok && c.classification()?.is_empty() {
            c.class(r["classification"].as_str().unwrap_or("FAIL"))?;
            let mut extra = serde_json::Map::new();
            for key in [
                "hang_kind",
                "wedge_confirmed",
                "census_stuck",
                "lockdep_hits",
                "cswitch_hits",
                "screenshot",
            ] {
                if let Some(value) = r.get(key) {
                    if key != "screenshot" || value.as_str().is_some_and(|s| !s.is_empty()) {
                        extra.insert(key.into(), value.clone());
                    }
                }
            }
            c.update(json!({"extra":extra}))?
        }
        results.push(r);
        if c.state()?
            .pointer("/extra/attach/kept")
            .and_then(Value::as_bool)
            == Some(true)
        {
            actual_total = idx + 1;
            break;
        }
    }
    let state = c.state()?;
    if state["extra"].get("screenshot").is_none() {
        if let Some(screen) = results
            .iter()
            .map(|r| text(r, "screenshot"))
            .find(|s| !s.is_empty())
        {
            c.update(json!({"extra":{"screenshot":screen}}))?
        }
    }
    let passes = results.iter().filter(|r| flag(r, "ok")).count() as u64;
    c.update(json!({"extra":{"passes":passes,"runs":actual_total}}))?;
    boot_evidence(&mut c, &results)?;
    let max = |key: &str| {
        results
            .iter()
            .filter_map(|r| r[key].as_u64())
            .max()
            .unwrap_or(0)
    };
    let stuck = max("census_stuck");
    let lockdep = max("lockdep_hits");
    let cswitch = max("cswitch_hits");
    if stuck != 0 || lockdep != 0 || cswitch != 0 {
        c.update(
            json!({"extra":{"census_stuck":stuck,"lockdep_hits":lockdep,"cswitch_hits":cswitch}}),
        )?;
        let lines = results
            .iter()
            .filter_map(|r| r["census_lines"].as_array())
            .find(|l| !l.is_empty())
            .map(|l| l.iter().take(2).cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        c.stage(
            "health",
            "warn",
            format!(
                "{stuck} [CENSUS] STUCK, {lockdep} [LOCKDEP], {cswitch} [CSWITCH] on a green boot"
            ),
            json!(lines),
        )?
    }
    if boot_arch == "amd64-uefi" && passes == actual_total && !flag(options, "no_net") {
        net_stage(&mut c)?
    }
    let mut payloads = text(options, "payload").to_owned();
    if flag(options, "doom") && !payloads.split(',').any(|p| p == "doom") {
        payloads = format!("{payloads},doom").trim_matches(',').into()
    }
    if !payloads.is_empty() && guest_reached_desktop(&results) {
        for idx in 0..actual_total {
            payload_stage(
                &mut c,
                &arch,
                &payloads,
                idx,
                actual_total,
                memory,
                Some(&boot_arch),
            )?
        }
    }
    if actual_total > 1 {
        c.update(json!({"extra":{"flake_rate":format!("{passes}/{actual_total}")}}))?;
        if passes > 0 && passes < actual_total {
            if c.classification()?.is_empty() {
                c.class("FLAKY")?
            }
            c.update(json!({"verdict":"FAIL"}))?;
            c.stage(
                "flake",
                "fail",
                format!("{passes}/{actual_total} pass"),
                json!([]),
            )?
        }
    }
    if flag(options, "timing") {
        let state = c.state()?;
        let live = state
            .pointer("/extra/live_log")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or(c.run_dir.join("serial.log"));
        let tool = root.join("tools/boot_phase_report.py");
        if live.is_file() && tool.is_file() {
            let cmd = vec![
                c.python(),
                tool.to_string_lossy().into_owned(),
                live.to_string_lossy().into_owned(),
                "--sizes".into(),
                root.join(format!("build-{arch}"))
                    .to_string_lossy()
                    .into_owned(),
            ];
            let env = c.env.clone();
            let proc = c.command(cmd, &env, 120, "boot-phase timing")?;
            let report = text(&proc, "stdout").trim();
            if !report.is_empty() {
                let report: String = report.chars().take(12000).collect();
                c.log("boot-phase timing", &report)?
            }
        }
    }
    c.finish()
}
pub fn boot(root: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let mut c = Context::new(root, a, host)?;
    let options = a.get("args").unwrap_or(a);
    let arch = base_arch(text(options, "arch"));
    let boot_arch = vm_arch(text(options, "arch"))?;
    let mut request = a.as_object().cloned().unwrap_or_default();
    request.extend(json!({"arch":arch,"boot_arch":boot_arch,"run_dir":c.run_dir,"env":c.env,"until":text(options,"until"),"fail_on":options.get("fail_on").cloned().unwrap_or(Value::Null),"idx":0,"total":1}).as_object().unwrap().clone());
    let r = c.call("smoke_one_boot", json!(request))?;
    let ok = flag(&r, "ok");
    c.stage(
        "boot",
        if ok { "ok" } else { "fail" },
        if ok {
            "reached"
        } else {
            r["classification"].as_str().unwrap_or(text(&r, "outcome"))
        },
        r.get("salient").cloned().unwrap_or(json!([])),
    )?;
    if !ok {
        c.class(r["classification"].as_str().unwrap_or("FAIL"))?
    }
    boot_evidence(&mut c, &[r])?;
    c.finish()
}
pub fn dispatch(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = args(request);
    Ok(match text(request, "op") {
        "smoke_run" => smoke(root, a, host)?,
        "smoke_boot" => boot(root, a, host)?,
        "smoke_run_guest_reached_desktop" => json!(guest_reached_desktop(
            a["results"].as_array().ok_or("missing boot results")?
        )),
        "smoke_run_payload_build_flags" => json!(payload_build_flags(
            root,
            &serde_json::from_value::<Vec<String>>(a["names"].clone())?
        )?),
        "smoke_run_payload_crash" => payload_crash(
            root,
            text(a, "arch"),
            Path::new(text(a, "payload_dir")),
            a.get("known_flakes")
                .unwrap_or(&crate::verdict::known_flakes()),
        )?,
        "smoke_run_payload_plan" => {
            let p = payload_plan(
                root,
                text(a, "payloads"),
                text(a, "arch"),
                a["boot_arch"].as_str(),
                text(a, "vm_memory_override"),
            )?;
            json!({"pytest_arch":p.pytest_arch,"drain":p.drain,"vm_memory":p.memory,"extra_args":p.extra,"refusal":p.refusal})
        }
        "smoke_run_boot_evidence" => {
            let mut c = Context::new(root, a, host)?;
            boot_evidence(
                &mut c,
                a["results"].as_array().ok_or("missing boot results")?,
            )?;
            Value::Null
        }
        "smoke_run_net_stage" => {
            let mut c = Context::new(root, a, host)?;
            net_stage(&mut c)?;
            Value::Null
        }
        "smoke_run_payload_stage" => {
            let mut c = Context::new(root, a, host)?;
            payload_stage(
                &mut c,
                text(a, "arch"),
                text(a, "payloads"),
                a["idx"].as_u64().unwrap_or(0),
                a["total"].as_u64().unwrap_or(1),
                text(a, "vm_memory_override"),
                a["boot_arch"].as_str(),
            )?;
            Value::Null
        }
        "smoke_run_qemu_preflight" => {
            let mut c = Context::new(root, a, host)?;
            qemu_preflight(&mut c, text(a, "boot_arch"))?
        }
        "smoke_run_staged_payloads" => {
            host.call("payload_stamp", json!({"arch":text(a,"arch")}))?
        }
        op => return Err(Error::new(format!("unknown outer smoke operation {op}"))),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launch_failures_do_not_discard_payload_coverage() {
        assert!(guest_reached_desktop(&[
            json!({"ok":true}),
            json!({"ok":false,"classification":"LAUNCH_FAIL"})
        ]));
        assert!(!guest_reached_desktop(&[
            json!({"ok":true}),
            json!({"ok":false,"classification":"BUGCHECK"})
        ]));
        assert!(!guest_reached_desktop(&[
            json!({"ok":false,"classification":"LAUNCH_FAIL"})
        ]));
    }
    #[test]
    fn payload_yamls_drive_memory_and_budgets() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("tools/payloads/load");
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("payload.yaml"),"test:\n  launch_drain_secs: 7200\n  vm_memory: 2G\n  qemu_extra_args: '-device e1000'\n  net: true\n").unwrap();
        let plan = payload_plan(dir.path(), "load", "amd64", Some("amd64-bios"), "").unwrap();
        assert!(plan.refusal.is_some());
        let plan = payload_plan(dir.path(), "load", "amd64", None, "768M").unwrap();
        assert_eq!(plan.drain, 7200.);
        assert_eq!(plan.memory, "768M");
        assert_eq!(plan.pytest_arch, "amd64-uefi-net");
        assert_eq!(plan.extra, vec!["-device e1000"]);
    }
    struct PreflightHost {
        state: Value,
        fresh: bool,
        boots: usize,
    }
    impl Host for PreflightHost {
        fn call(&mut self, op: &str, a: Value) -> Result<Value> {
            Ok(match op {
                "get_run" => self.state.clone(),
                "set_run" => {
                    for (k, v) in a.as_object().unwrap() {
                        self.state[k] = v.clone()
                    }
                    self.state.clone()
                }
                "stage" => {
                    self.state["stages"].as_array_mut().unwrap().push(a);
                    Value::Null
                }
                "provenance_check_staleness" => {
                    json!({"fresh":self.fresh,"reasons":["media is stale"],"warm":false})
                }
                "payload_stamp" => json!([]),
                "finalize" => json!(if text(&self.state, "classification").is_empty() {
                    0
                } else {
                    1
                }),
                "smoke_one_boot" => {
                    self.boots += 1;
                    return Err("preflight admitted an unintended boot".into());
                }
                _ => return Err(Error::new(format!("unexpected callback {op}"))),
            })
        }
    }
    #[test]
    fn missing_payload_and_stale_media_refuse_before_a_vm_exists() {
        let dir = tempfile::tempdir().unwrap();
        for (fresh, expected) in [(false, "STALE_MEDIA"), (true, "PAYLOAD_NOT_IN_MEDIA")] {
            let mut host = PreflightHost {
                state: json!({"classification":"","stages":[],"extra":{}}),
                fresh,
                boots: 0,
            };
            let args = json!({"run_dir":dir.path(),"args":{"arch":"amd64","payload":"absent","no_net":true},"env":{}});
            assert_eq!(smoke(dir.path(), &args, &mut host).unwrap(), 1);
            assert_eq!(host.state["classification"], expected);
            assert_eq!(host.boots, 0);
        }
    }
}
