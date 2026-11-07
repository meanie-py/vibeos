//! Native diagnostic preparation and separately owned monitor sidecars.
use crate::protocol::QmpClient;
use crate::{backend, callback::Host};
use crate::{pc_sample, record_replay, replay_sample, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
fn s<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| Error::new(format!("missing diagnostic {k}")))
}
fn version(argv: &[String]) -> Result<String> {
    let exe = argv.first().ok_or("empty QEMU command line")?;
    let mut c = Command::new(exe);
    c.arg("--version");
    let out = vibeos_build::process::capture(&mut c, 30)?;
    if out.code != 0 {
        return Err(format!("cannot query QEMU version from {exe}: {}", out.stderr).into());
    }
    out.stdout
        .lines()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| Error::new("QEMU version response is empty"))
}
pub fn prepare(a: &Value) -> Result<Value> {
    let env: BTreeMap<String, String> =
        serde_json::from_value(a.get("env").cloned().unwrap_or(json!({})))?;
    let get = |k: &str| env.get(k).map(String::as_str).unwrap_or("");
    let mut argv: Vec<String> = serde_json::from_value(a["argv"].clone())?;
    let mut result = json!({"sidecars":[]});
    let mut messages = Vec::new();
    let req = record_replay::env_request(get("VIBEOS_QEMU_RR"))?;
    let sampler = get("VIBEOS_QEMU_RR_SAMPLE").trim();
    if !sampler.is_empty() && req.is_none() {
        return Err("VIBEOS_QEMU_RR_SAMPLE requires VIBEOS_QEMU_RR".into());
    }
    if let Some((mode, trace)) = req {
        if s(a, "backend")? != "native" {
            return Err("record/replay requires the native backend".into());
        }
        let current_version = version(&argv)?;
        if mode == "replay" {
            let problems = record_replay::check_trace(&trace, &current_version)?;
            if problems
                .iter()
                .any(|p| p.contains(record_replay::FATAL_MARKER))
            {
                return Err(problems.join("\n").into());
            }
        }
        let (transformed, media) = record_replay::transform(
            &argv,
            &mode,
            &trace,
            record_replay::env_shift(get("VIBEOS_QEMU_RR_SHIFT"))?,
            mode == "record",
        )?;
        argv = transformed;
        if mode == "record" {
            let media: serde_json::Map<_, _> = media.into_iter().collect();
            record_replay::write_manifest(
                &trace,
                &json!({"arch":a["arch"],"argv":argv,"media":media,"created":chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),"qemu_version":current_version,"extra":{"smp":if get("VIBEOS_QEMU_SMP").is_empty(){"1"}else{get("VIBEOS_QEMU_SMP")}}}),
            )?;
        } else {
            record_replay::assert_same_shape(&trace, &argv)?;
            if let Some(want) = record_replay::env_break(get("VIBEOS_QEMU_RR_BREAK"))? {
                let snapped = record_replay::snap_break(&trace, want)?;
                if snapped != want {
                    messages.push(format!(
                        "[rr-break] {want} is mid-chunk; using next boundary {snapped}"
                    ));
                }
                result["rr_break_icount"] = json!(snapped);
            }
        }
        record_replay::note_run(&trace, &mode, s(a, "live_log")?)?;
        result["boot_timeout_min"] = json!(1800.0);
        result["hang_silence_min"] = json!(600.0);
        if !sampler.is_empty() {
            if (mode == "replay") != (sampler == "check") {
                return Err("RR sampler expects seconds for record or check for replay".into());
            }
            if mode == "record" {
                let p: f64 = sampler
                    .parse()
                    .map_err(|_| Error::new("RR sample period must be a number"))?;
                if !p.is_finite() || p <= 0.0 {
                    return Err("RR sample period must be finite and positive".into());
                }
            }
            let sock = Path::new(s(a, "sock_dir")?).join("rrsample.sock");
            let sample_argv = argv.clone();
            argv.extend([
                "-qmp".into(),
                format!("unix:{},server,nowait", sock.display()),
            ]);
            result["sidecars"].as_array_mut().unwrap().push(json!({"kind":"rr","mode":mode,"value":sampler,"trace":trace,"argv":sample_argv,"sock":sock,"work":a["sock_dir"]}));
        }
    }
    if let Some(period) = pc_sample::env_period(get("VIBEOS_QEMU_PC_SAMPLE"))? {
        let mut sidecar = json!({"kind":"pc","period":period,"out":Path::new(s(a,"live_log")?).parent().unwrap_or(Path::new(".")).join("pcsamples.jsonl")});
        if s(a, "backend")? == "native" {
            let sock = Path::new(s(a, "sock_dir")?).join("pcsample.sock");
            argv.extend([
                "-qmp".into(),
                format!("unix:{},server,nowait", sock.display()),
            ]);
            sidecar["sock"] = json!(sock);
        } else {
            let reserved: BTreeSet<u16> =
                serde_json::from_value(a.get("reserved_ports").cloned().unwrap_or(json!([])))?;
            let mut port = None;
            for _ in 0..64 {
                let p = backend::windows_free_tcp_port()?;
                if !reserved.contains(&p) {
                    port = Some(p);
                    break;
                }
            }
            let port = port.ok_or("cannot select a unique diagnostic QMP port")?;
            argv.extend([
                "-qmp".into(),
                format!("tcp:0.0.0.0:{port},server=on,wait=off"),
            ]);
            sidecar["host"] = json!(backend::gateway_ip()?);
            sidecar["port"] = json!(port);
        }
        result["sidecars"].as_array_mut().unwrap().push(sidecar);
    }
    result["argv"] = json!(argv);
    if !messages.is_empty() {
        result["messages"] = json!(messages);
    }
    Ok(result)
}
struct Monitor {
    qmp: QmpClient,
}
impl Host for Monitor {
    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
        match op {
            "diagnostic_begin_sample" | "diagnostic_end_sample" => Ok(Value::Null),
            "diagnostic_hmp" => Ok(json!(self
                .qmp
                .hmp(s(&a, "command")?)
                .map_err(replay_sample::qerr)?)),
            _ => Err(format!("unknown monitor operation {op}").into()),
        }
    }
}
pub fn sidecar(a: &Value) -> Result<Value> {
    match s(a, "kind")? {
        "rr" => replay_sample::run(a),
        "pc" => {
            let mut host = Monitor {
                qmp: QmpClient::new(),
            };
            let deadline = Instant::now() + Duration::from_secs(60);
            loop {
                let connected = if let Some(sock) = a["sock"].as_str() {
                    host.qmp.connect(Path::new(sock), Duration::from_secs(60))
                } else {
                    host.qmp.connect_tcp(
                        s(a, "host")?,
                        a["port"]
                            .as_u64()
                            .filter(|v| *v > 0 && *v <= 65535)
                            .ok_or("invalid diagnostic port")? as u16,
                        Duration::from_secs(60),
                    )
                };
                match connected {
                    Ok(_) => break,
                    Err(e) if Instant::now() < deadline => {
                        let _ = e;
                        std::thread::sleep(Duration::from_millis(200));
                    }
                    Err(e) => return Err(e),
                }
            }
            let result = pc_sample::run_loop(
                &mut host,
                Path::new(s(a, "out")?),
                a["period"].as_f64().ok_or("missing sample period")?,
            );
            host.qmp.close();
            result
        }
        kind => Err(format!("unknown diagnostic sidecar {kind:?}").into()),
    }
}
pub fn dispatch(request: &Value) -> Result<Value> {
    match request["op"].as_str().unwrap_or("") {
        "diagnostics_prepare" => prepare(&request["kwargs"]),
        "diagnostics_sidecar" => sidecar(&request["kwargs"]),
        _ => Err("unknown diagnostics operation".into()),
    }
}

/// A VM-owned process: stdin is a DTO, stdout is one result, diagnostics use stderr.
pub fn worker(operation: &str) -> Result<u8> {
    use std::io::Write;
    let result = (|| -> Result<Value> {
        let request: Value = serde_json::from_reader(std::io::stdin().lock())?;
        match operation {
            "vm_diagnostic_prepare" => {
                let mut value = prepare(&request)?;
                if let Some(messages) = value.as_object_mut().and_then(|v| v.remove("messages")) {
                    if let Some(messages) = messages.as_array() {
                        for message in messages {
                            eprintln!("{}", message.as_str().unwrap_or(""));
                        }
                    }
                }
                Ok(value)
            }
            "vm_diagnostic_sidecar" => sidecar(&request),
            _ => Err(format!("unknown diagnostic worker {operation}").into()),
        }
    })();
    let (response, code) = match result {
        Ok(value) => (json!({"ok":value}), 0),
        Err(error) => (
            json!({"error":{"message":error.message,"kind":error.kind}}),
            2,
        ),
    };
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output, &response)?;
    output.write_all(b"\n")?;
    output.flush()?;
    Ok(code)
}
