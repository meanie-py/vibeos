//! Scripted authorities for testing the shipping native controllers without a VM.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
struct ScriptHost {
    root: PathBuf,
    responses: BTreeMap<String, VecDeque<Value>>,
    chunks: VecDeque<String>,
    calls: Vec<Value>,
    state: Value,
    config: Value,
    created: u64,
    reads: u64,
}
impl ScriptHost {
    fn new(root: &Path, a: &Value) -> Result<Self> {
        Ok(Self{root:root.into(),responses:serde_json::from_value(a.get("responses").cloned().unwrap_or(json!({})))?,chunks:serde_json::from_value(a.get("chunks").cloned().unwrap_or(json!([])))?,calls:Vec::new(),state:a.get("run").cloned().unwrap_or(json!({"classification":"","extra":{},"stages":[],"verdict":"OK"})),config:a.get("config").cloned().unwrap_or(json!({"boot_marker":"READY","boot_timeout":0.03,"hang_silence_seconds":0.0,"desktop_settle_seconds":0.0,"required_milestones":[]})),created:0,reads:0})
    }
    fn update(&mut self, a: &Value) {
        for key in ["classification", "verdict"] {
            if let Some(value) = a.get(key) {
                self.state[key] = value.clone()
            }
        }
        if let Some(extra) = a["extra"].as_object() {
            if !self.state["extra"].is_object() {
                self.state["extra"] = json!({})
            }
            self.state["extra"]
                .as_object_mut()
                .unwrap()
                .extend(extra.clone())
        }
    }
}
impl Host for ScriptHost {
    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
        self.calls.push(json!({"op":op,"kwargs":a}));
        if let Some(row) = self.responses.get_mut(op).and_then(VecDeque::pop_front) {
            if let Some(error) = row["error"].as_str() {
                return Err(Error::new(error));
            }
            return Ok(row.get("value").cloned().unwrap_or(row));
        }
        let root = self.root.clone();
        let request = json!({"op":op,"kwargs":a});
        Ok(match op {
            "vm_poll_serial" => {
                self.reads += 1;
                if let Some(chunk) = self.chunks.pop_front() {
                    json!(chunk)
                } else {
                    thread::sleep(Duration::from_millis(1));
                    json!("")
                }
            }
            "vm_create" => {
                self.created += 1;
                json!({"id":self.created,"config":self.config})
            }
            "vm_start" => json!({"qemu_pid":999,"backend":"fixture","gdb_port":1234}),
            "vm_stop" | "vm_close" | "log" | "print" | "env" => Value::Null,
            "vm_screendump" => return Err("script has no framebuffer".into()),
            "vm_read_debugcon" => json!([]),
            "vm_qmp_hmp"
            | "vm_stderr_tail"
            | "vm_capture_cpu_state"
            | "vm_get_hang_cpu_state"
            | "vm_report_replay_break" => {
                json!("")
            }
            "vm_detach" => json!({"qemu_pid":999,"backend":"fixture","gdb_port":1234}),
            "debug_owner" => json!({"owner_pid":111,"kernel_image":"fixture-kernel.exe"}),
            "stage" => {
                let status = a["status"].as_str().unwrap_or("");
                if status == "fail" {
                    self.state["verdict"] = json!("FAIL")
                } else if status == "warn" && self.state["verdict"] == "OK" {
                    self.state["verdict"] = json!("WARN")
                }
                self.state["stages"]
                    .as_array_mut()
                    .ok_or("fixture run stages must be an array")?
                    .push(a);
                Value::Null
            }
            "get_run" => self.state.clone(),
            "set_run" => {
                self.update(&a);
                self.state.clone()
            }
            "finalize" => {
                self.update(&a);
                json!(if self.state["verdict"] == "FAIL"
                    || self.state["classification"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty())
                {
                    1
                } else {
                    0
                })
            }
            "smoke_one_boot" => crate::smoke::one_boot(&root, &a, self)?,
            "payload_stamp" => json!([]),
            op if op.starts_with("verdict_debugcon_") => {
                crate::debugcon::dispatch(&root, &request)?
            }
            op if op.starts_with("verdict_") => crate::verdict::dispatch(&root, &request)?,
            op if op.starts_with("smoke_check_") => crate::smoke_checks::dispatch(&root, &request)?,
            op if op.starts_with("ladder_") => crate::ladder::dispatch(&root, &request)?,
            _ => {
                return Err(Error::new(format!(
                    "fixture requires an explicit response for {op}"
                )))
            }
        })
    }
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let a = crate::verdict::args(request);
    let mut host = ScriptHost::new(root, a)?;
    let started = Instant::now();
    let mode = a["mode"]
        .as_str()
        .unwrap_or(if request["op"] == "smoke_settle_script" {
            "settle"
        } else {
            "one_boot"
        });
    let options = a.get("options").cloned().unwrap_or(json!({}));
    let result = (|| -> Result<Value> {
        Ok(match mode {
            "settle" => {
                let (text, seen) = crate::smoke::settle(
                    &mut host,
                    1,
                    a["seconds"].as_f64().unwrap_or(0.1),
                    a["milestone"].as_str(),
                    a["grace"].as_f64().unwrap_or(0.1),
                )?;
                json!({"text":text,"seen":seen})
            }
            "one_boot" => crate::smoke::one_boot(root, &options, &mut host)?,
            "run" => crate::smoke_run::smoke(root, &options, &mut host)?,
            "boot" => crate::smoke_run::boot(root, &options, &mut host)?,
            "operation" => crate::smoke_run::dispatch(
                root,
                &json!({"op":a["operation"],"kwargs":options}),
                &mut host,
            )?,
            _ => return Err(Error::new(format!("unknown smoke fixture mode {mode}"))),
        })
    })();
    let (result, error) = match result {
        Ok(v) => (v, Value::Null),
        Err(e) if a["capture_error"].as_bool() == Some(true) => (Value::Null, json!(e.message)),
        Err(e) => return Err(e),
    };
    Ok(
        json!({"error":error,"result":result,"run":host.state,"calls":host.calls,"serial_reads":host.reads,"created":host.created,"seconds":started.elapsed().as_secs_f64()}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settles_keep_all_serial_and_require_only_requested_milestones() {
        let root = tempfile::tempdir().unwrap();
        let v=dispatch(root.path(),&json!({"op":"smoke_settle_script","kwargs":{"seconds":0.01,"grace":0.01,"milestone":"shell ready","chunks":["taskbar building\n","shell ready\n"]}})).unwrap();
        assert!(v["result"]["seen"].as_bool().unwrap());
        assert!(v["result"]["text"]
            .as_str()
            .unwrap()
            .contains("taskbar building"));
        let v=dispatch(root.path(),&json!({"op":"smoke_settle_script","kwargs":{"seconds":0.01,"grace":0.02,"milestone":"shell ready","chunks":["taskbar building\n"]}})).unwrap();
        assert_eq!(v["result"]["seen"], false);
        assert!(v["seconds"].as_f64().unwrap() >= 0.03);
        assert!(v["seconds"].as_f64().unwrap() < 1.);
    }
    #[test]
    fn a_failed_native_launch_preserves_attempts_and_closes_every_partial_vm() {
        let root = tempfile::tempdir().unwrap();
        let v=dispatch(root.path(),&json!({"op":"smoke_launch_script","kwargs":{"options":{"arch":"arm64","run_dir":root.path().join("run"),"launch_attempts":3,"launch_retry_seconds":0},"responses":{"vm_start":[{"error":"launch failed"},{"error":"launch failed"},{"error":"launch failed"}]}}})).unwrap();
        assert_eq!(v["result"]["classification"], "LAUNCH_FAIL");
        assert_eq!(v["result"]["launch_attempts"], 3);
        let calls = v["calls"].as_array().unwrap();
        assert_eq!(calls.iter().filter(|c| c["op"] == "vm_stop").count(), 3);
        assert_eq!(calls.iter().filter(|c| c["op"] == "vm_close").count(), 3);
    }
}
