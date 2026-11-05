//! Raspberry Pi board controllers and native helper API.
use crate::{
    bench_boot, bench_helpers as h, bench_lease_commands, bench_readonly, callback::Host, Error,
    Result,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn board(a: &Value) -> Result<&str> {
    match s(a, "board") {
        "pi4" => Ok("pi4"),
        "pi5" => Ok("pi5"),
        other => Err(Error::new(format!("invalid board {other}"))),
    }
}
fn path(a: &Value, k: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(
        a.get(k)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new(format!("missing {k}")))?,
    ))
}
struct Stages(Vec<Value>);
impl Host for Stages {
    fn call(&mut self, op: &str, args: Value) -> Result<Value> {
        if op == "stage" {
            self.0.push(args);
            Ok(Value::Null)
        } else {
            Err(Error::new(format!("unexpected doctor callback {op}")))
        }
    }
}
pub fn dispatch(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = request.get("kwargs").unwrap_or(request);
    let op = s(request, "op");
    let mut env: BTreeMap<String, String> =
        serde_json::from_value(a.get("env").cloned().unwrap_or(json!({})))?;
    match op {
        "bench_parse_usb" => Ok(h::parse_usb(s(a, "text"))),
        "bench_parse_power" => Ok(h::parse_power(s(a, "text"))),
        "bench_serial_file" => Ok(json!(h::serial_file(&a["state"]))),
        "bench_listener_state" => Ok(h::read_json(&path(a, "bench")?.join("serial-current.json"))),
        "bench_pid_runs" => Ok(json!(h::pid_runs(&a["pid"], s(a, "needle")))),
        "bench_payload_manifest" => {
            let b = board(a)?;
            Ok(h::read_json(&path(a, "bench")?.join(if b == "pi5" {
                "payload.json"
            } else {
                "payload/manifest.json"
            })))
        }
        "bench_wait_desktop" => {
            let (text, reached) = h::wait_desktop(
                &path(a, "serial")?,
                a["start"].as_u64().unwrap_or(0),
                a["timeout"].as_f64().unwrap_or(0.),
            );
            Ok(json!([text, reached]))
        }
        "bench_tail_read" => h::tail_read(
            &path(a, "serial")?,
            a["pos"].as_u64().unwrap_or(0),
            a["t0"].as_f64().unwrap_or(0.0),
            s(a, "tail"),
            &a["times"],
            a["timeout"].as_f64().unwrap_or(0.5),
        ),
        "bench_tail_durations" => Ok(h::tail_durations(&a["times"])),
        "bench_until" => h::until(
            &path(a, "serial")?,
            a["start"].as_u64().unwrap_or(0),
            s(a, "needle"),
            a["timeout"].as_f64().unwrap_or(180.0),
        ),
        "bench_serial_slice" => Ok(json!(h::serial_slice(
            &path(a, "serial")?,
            a["start"].as_u64().unwrap_or(0)
        ))),
        "bench_brightness" => Ok(json!(h::brightness(&path(a, "path")?))),
        "bench_desktop_share" => Ok(json!(h::desktop_share(&path(a, "path")?))),
        "bench_await_stortest" => h::await_stortest_host(
            &path(a, "serial")?,
            a["start"].as_u64().unwrap_or(0),
            a["has_medium"].as_bool().unwrap_or(false),
            a["has_drop"].as_bool().unwrap_or(false),
            host,
        ),
        "bench_stortest_verdict" => {
            let (verdict, lines) = h::stortest_verdict(s(a, "text"));
            Ok(json!([verdict, lines]))
        }
        "bench_drop_marker" => Ok(json!(h::drop_marker(s(a, "text")))),
        "bench_grade_drop" => Ok(h::grade_drop(
            s(a, "text"),
            a["dropped"].as_bool().unwrap_or(false),
            a["server_lost"].as_bool().unwrap_or(false),
        )),
        "bench_stortest_readback" => Ok(json!(h::stortest_readback(&path(a, "path")?, &env)?)),
        "bench_stortest_image" => {
            h::stortest_image(&path(a, "path")?)?;
            Ok(Value::Null)
        }
        "bench_ssh_up" => Ok(json!(bench_readonly::ssh(root, &env)?)),
        "bench_plug_configured" => Ok(json!(bench_readonly::configured(root, &env)?)),
        "bench_status" | "bench_lease" | "bench_doctor" | "bench_doctor_stages" | "bench_boot" => {
            let b = board(a)?;
            env.insert("VIBEOS_BENCH_BOARD".into(), b.into());
            match op {
                "bench_status" => bench_readonly::status(root, &env, b, host),
                "bench_lease" => bench_lease_commands::command(root, &a["args"], &env, host),
                "bench_doctor_stages" => {
                    let mut stages = Stages(Vec::new());
                    let dir = a.get("run_dir").and_then(Value::as_str).map(PathBuf::from);
                    bench_readonly::doctor(root, &env, b, dir.as_deref(), &mut stages)?;
                    Ok(json!(stages.0))
                }
                "bench_doctor" => {
                    let run = host.call("get_run", json!({}))?;
                    let run_dir = PathBuf::from(s(&run, "dir"));
                    bench_readonly::doctor(
                        root,
                        &env,
                        b,
                        if b == "pi4" { Some(&run_dir) } else { None },
                        host,
                    )?;
                    let run = host.call("get_run", json!({}))?;
                    if s(&run, "verdict") == "FAIL" {
                        host.call("set_run", json!({"classification":"BENCH_UNHEALTHY"}))?;
                    }
                    host.call("finalize", json!({}))
                }
                _ => bench_boot::boot(root, b, a, host),
            }
        }
        _ => Err(Error::new(format!("unknown bench operation {op}"))),
    }
}
