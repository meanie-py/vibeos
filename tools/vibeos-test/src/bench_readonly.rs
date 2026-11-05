//! Read-only board status and doctor probes. No hardware mutation occurs here.
use crate::{bench_helpers as h, callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
type Env = BTreeMap<String, String>;
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
pub fn run(
    root: &Path,
    env: &Env,
    board: &str,
    script: &str,
    args: &[&str],
    timeout: u64,
) -> Result<(i32, String)> {
    let mut cmd = Command::new("bash");
    cmd.arg(root.join("tools").join(board).join(script))
        .args(args)
        .current_dir(root)
        .env_clear()
        .envs(env);
    let out = vibeos_build::artifact_executor::supervised_output(
        &mut cmd,
        Some(Duration::from_secs(timeout)),
    )?;
    Ok((
        out.code,
        format!("{}{}", out.stdout, out.stderr).trim().to_owned(),
    ))
}
pub fn ssh(root: &Path, env: &Env) -> Result<bool> {
    let script = root.join("tools/pi5/env.sh");
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(". \"$1\"; pi5_ssh true")
        .arg("bench-ssh")
        .arg(script)
        .current_dir(root)
        .env_clear()
        .envs(env);
    Ok(
        vibeos_build::artifact_executor::supervised_output(
            &mut cmd,
            Some(Duration::from_secs(20)),
        )?
        .code
            == 0,
    )
}
pub fn bench(env: &Env, board: &str) -> PathBuf {
    PathBuf::from(
        env.get(if board == "pi5" {
            "PI5_BENCH"
        } else {
            "PI4_BENCH"
        })
        .cloned()
        .unwrap_or_else(|| format!("/mnt/d/home/vibeos-r{board}-setup")),
    )
}
fn stage(
    host: &mut dyn Host,
    name: &str,
    status: &str,
    detail: &str,
    salient: Value,
) -> Result<()> {
    host.call(
        "stage",
        json!({"name":name,"status":status,"detail":detail,"salient":salient}),
    )?;
    Ok(())
}
fn print(host: &mut dyn Host, text: String) -> Result<()> {
    host.call("print", json!({"text":text}))?;
    Ok(())
}
pub fn configured(root: &Path, env: &Env) -> Result<bool> {
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(". \"$1\"; printf %s \"$PI4_PLUG\"")
        .arg("bench-plug")
        .arg(root.join("tools/pi5/env.sh"))
        .env_clear()
        .envs(env)
        .current_dir(root);
    let result = vibeos_build::artifact_executor::supervised_output(
        &mut cmd,
        Some(Duration::from_secs(10)),
    )?;
    if result.code != 0 {
        return Err(Error::new(format!(
            "Pi 5 environment failed: {}",
            result.stderr
        )));
    }
    Ok(!result.stdout.trim().is_empty() && result.stdout.trim() != "none.invalid")
}
fn lease_err(e: vibeos_bench::Error) -> Error {
    Error::new(e.message)
}
pub fn status(root: &Path, env: &Env, board: &str, host: &mut dyn Host) -> Result<Value> {
    let lease = vibeos_bench::lease::current(env, None).map_err(lease_err)?;
    print(
        host,
        format!(
            "lease  : {}",
            vibeos_bench::lease::describe(&lease, None).map_err(lease_err)?
        ),
    )?;
    if board == "pi5" && !configured(root, env)? {
        print(host, "power  : no plug named (tools/pi5/env.sh)".into())?;
    } else {
        let (rc, out) = run(root, env, board, "power.sh", &["status"], 60)?;
        print(
            host,
            format!("power  : {}", if rc == 0 { &out } else { "unreachable" }),
        )?;
    }
    let (_, out) = run(root, env, board, "pico.sh", &["usb"], 60)?;
    print(
        host,
        format!("pico   : {}", out.lines().last().unwrap_or("no answer")),
    )?;
    let dir = bench(env, board);
    let serial = h::read_json(&dir.join("serial-current.json"));
    if board == "pi4" {
        print(
            host,
            format!(
                "serial : {} connected={} {}",
                serial
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("none"),
                serial
                    .get("connected")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "None".into()),
                s(&serial, "directory")
            ),
        )?;
    } else {
        print(
            host,
            format!(
                "serial : {} {}",
                serial
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("none"),
                s(&serial, "directory")
            ),
        )?;
    }
    if board == "pi5" {
        print(
            host,
            format!("stager : {}", if ssh(root, env)? { "up" } else { "no SSH" }),
        )?;
    }
    let manifest = h::read_json(&dir.join(if board == "pi5" {
        "payload.json"
    } else {
        "payload/manifest.json"
    }));
    let commit = manifest
        .get(if board == "pi5" {
            "commit"
        } else {
            "source_head"
        })
        .and_then(Value::as_str)
        .unwrap_or("none");
    print(
        host,
        format!(
            "payload: {} staged {}",
            commit.chars().take(12).collect::<String>(),
            manifest
                .get("staged")
                .and_then(Value::as_str)
                .unwrap_or("-")
        ),
    )?;
    Ok(json!(0))
}
pub fn doctor(
    root: &Path,
    env: &Env,
    board: &str,
    run_dir: Option<&Path>,
    host: &mut dyn Host,
) -> Result<()> {
    let mut power = json!({});
    if board == "pi5" && !configured(root, env)? {
        stage(
            host,
            "plug",
            "warn",
            "no plug named",
            json!(["set PI5_PLUG before power cycling"]),
        )?;
    } else {
        let (rc, out) = run(root, env, board, "power.sh", &["status"], 60)?;
        power = h::parse_power(&out);
        if rc != 0 || power.get("on").is_none() {
            stage(host, "plug", "fail", "unreachable", json!([out]))?;
        } else if power["on"] == true && power["watts"].as_f64().unwrap_or(0.) > h::PI_WATTS_MAX {
            stage(
                host,
                "plug",
                "fail",
                "draws over 20 W: not a Pi",
                json!(["check this board's plug identity"]),
            )?;
        } else {
            stage(
                host,
                "plug",
                "ok",
                &format!(
                    "{} {:.1} W",
                    if power["on"] == true { "on" } else { "off" },
                    power["watts"].as_f64().unwrap_or(0.)
                ),
                json!([]),
            )?;
        }
    }
    let (_, out) = run(root, env, board, "pico.sh", &["usb"], 60)?;
    let usb = h::parse_usb(&out);
    if let Some(error) = usb.get("error") {
        stage(
            host,
            "pico",
            if board == "pi5" { "warn" } else { "fail" },
            error.as_str().unwrap_or("unreachable"),
            json!([]),
        )?;
    } else if board == "pi4"
        && power.get("on").is_some()
        && ((usb.get("target_powered").is_some() && usb["target_powered"] != power["on"])
            || (power["on"] == false && usb["mounted"] == true && usb["suspended"] == false))
    {
        stage(
            host,
            "pico",
            "fail",
            "attached elsewhere",
            json!(["plug the Pico's USB into the Pi before sending input"]),
        )?;
    } else if board == "pi4" && power["on"] == true && usb["mounted"] != true {
        stage(host, "pico", "warn", "not enumerated", json!([]))?;
    } else {
        stage(
            host,
            "pico",
            "ok",
            &format!("mounted={}", usb["mounted"]),
            json!([]),
        )?;
    }
    let dir = bench(env, board);
    let serial = h::read_json(&dir.join("serial-current.json"));
    let mode = env.get("PI4_SERIAL").map(String::as_str).unwrap_or("pico");
    let pico_host = env
        .get(if board == "pi5" {
            "PI5_PICO_HOST"
        } else {
            "PI4_PICO_HOST"
        })
        .map(String::as_str)
        .unwrap_or(if board == "pi5" {
            "none.invalid"
        } else {
            "picousb"
        });
    let port = env
        .get("PICO_SERIAL_PORT")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8082);
    let alive = h::listener_valid(&serial, &dir, pico_host, port);
    let serial_ok = if board == "pi5" {
        h::serial_file(&serial).is_some() && alive
    } else {
        s(&serial, "status") == "LISTENING"
            && (mode != "pico" || (alive && serial["connected"] == true))
    };
    stage(
        host,
        "serial",
        if serial_ok {
            "ok"
        } else if board == "pi5" {
            "warn"
        } else {
            "fail"
        },
        if serial_ok {
            "listening"
        } else {
            "no listener"
        },
        json!([format!(
            "tools/{board}/{} up",
            if board == "pi5" {
                "serial.sh"
            } else {
                "bench.sh"
            }
        )]),
    )?;
    if board == "pi5" {
        let up = ssh(root, env)?;
        stage(
            host,
            "stager",
            if up { "ok" } else { "warn" },
            if up { "Linux up, SSH ok" } else { "no SSH" },
            json!([]),
        )?;
        let path = dir.join("hdmi-doctor.png");
        let (rc, _) = run(root, env, board, "hdmi.sh", &[&path.to_string_lossy()], 60)?;
        stage(
            host,
            "hdmi",
            if rc == 0 { "ok" } else { "fail" },
            if rc == 0 { "frame ok" } else { "no frame" },
            json!([]),
        )?;
    } else {
        let (_, out) = run(root, env, board, "bench.sh", &["status"], 90)?;
        for (name, key) in [("tftp", "tftp server"), ("http", "http server")] {
            let value = out
                .lines()
                .find_map(|l| {
                    l.trim_start()
                        .strip_prefix(key)
                        .and_then(|s| s.trim_start().strip_prefix(':'))
                        .map(str::trim)
                })
                .unwrap_or("");
            stage(
                host,
                name,
                if value.is_empty() || value == "down" {
                    "fail"
                } else {
                    "ok"
                },
                if value.is_empty() { "down" } else { value },
                json!([]),
            )?;
        }
        if let Some(dir) = run_dir {
            let path = dir.join("hdmi-doctor.png");
            let mut captured = false;
            let mut output = String::new();
            for attempt in 0..3 {
                let (rc, out) = run(root, env, board, "hdmi.sh", &[&path.to_string_lossy()], 60)?;
                output = out;
                if rc == 0 && path.is_file() {
                    captured = true;
                    break;
                }
                if attempt < 2 {
                    std::thread::sleep(Duration::from_secs(2));
                }
            }
            stage(
                host,
                "hdmi",
                if captured { "ok" } else { "warn" },
                if captured {
                    "frame captured"
                } else {
                    "no frame"
                },
                if captured {
                    json!([])
                } else {
                    json!(["capture card busy? close the Windows Camera app", output])
                },
            )?;
        }
        let manifest = h::read_json(&dir.join("payload/manifest.json"));
        if manifest.as_object().is_none_or(|m| m.is_empty()) {
            stage(host, "payload", "warn", "none staged", json!([]))?;
        } else {
            let mut git = Command::new("git");
            git.args(["rev-parse", "HEAD"])
                .current_dir(root)
                .env_clear()
                .envs(env);
            let result = vibeos_build::artifact_executor::supervised_output(
                &mut git,
                Some(Duration::from_secs(10)),
            )?;
            if result.code != 0 {
                return Err(Error::new(format!(
                    "git rev-parse failed: {}",
                    result.stderr
                )));
            }
            let staged = s(&manifest, "source_head");
            if staged != result.stdout.trim() || manifest["source_dirty"] == true {
                stage(
                    host,
                    "payload",
                    "warn",
                    &format!(
                        "staged from {}",
                        staged.chars().take(12).collect::<String>()
                    ),
                    json!(["not this tree: `vibeos-cli pi4 boot` restages it"]),
                )?;
            } else {
                stage(host, "payload", "ok", "this tree", json!([]))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::symlink, process::Stdio};
    #[derive(Default)]
    struct Recorder(Vec<Value>);
    impl Host for Recorder {
        fn call(&mut self, op: &str, args: Value) -> Result<Value> {
            assert_eq!(op, "stage");
            self.0.push(args);
            Ok(Value::Null)
        }
    }
    #[test]
    fn doctor_rejects_pico_connected_to_another_host() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let scripts = root.join("tools/pi4");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("power.sh"), "echo 'on 4.0 W'\n").unwrap();
        fs::write(
            scripts.join("pico.sh"),
            "echo 'uptime=5s mounted=True suspended=False target_powered=False'\n",
        )
        .unwrap();
        fs::write(
            scripts.join("bench.sh"),
            "printf 'tftp server : 12\\nhttp server : 13\\n'\n",
        )
        .unwrap();
        let bench = root.join("bench");
        let cap = bench.join("serial-test");
        fs::create_dir_all(&cap).unwrap();
        let exe = root.join("vibeos-bench");
        symlink("/bin/sh", &exe).unwrap();
        let mut child = Command::new(&exe)
            .args(["-c", "read x", "serial", bench.to_str().unwrap(), "picousb"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
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
        fs::write(bench.join("serial-current.json"),serde_json::to_vec(&json!({"pid":child.id(),"directory":cap,"port":"picousb:8082 (picousb)","status":"LISTENING","connected":true,"engine":"vibeos-bench","process_start_ticks":ticks})).unwrap()).unwrap();
        let env = Env::from([
            ("PATH".into(), std::env::var("PATH").unwrap()),
            ("PI4_BENCH".into(), bench.to_string_lossy().into_owned()),
        ]);
        let mut rec = Recorder::default();
        doctor(root, &env, "pi4", None, &mut rec).unwrap();
        let pico = rec.0.iter().find(|v| v["name"] == "pico").unwrap();
        assert_eq!(pico["status"], "fail");
        assert_eq!(pico["detail"], "attached elsewhere");
        assert_eq!(
            rec.0.iter().find(|v| v["name"] == "serial").unwrap()["status"],
            "ok"
        );
        child.kill().unwrap();
        child.wait().unwrap();
    }
}

#[cfg(test)]
mod path_controls {
    use super::*;
    #[test]
    fn maintained_bench_defaults() {
        assert_eq!(
            bench(&Env::new(), "pi4"),
            PathBuf::from("/mnt/d/home/vibeos-rpi4-setup")
        );
        assert_eq!(
            bench(&Env::new(), "pi5"),
            PathBuf::from("/mnt/d/home/vibeos-rpi5-setup")
        );
    }
}
