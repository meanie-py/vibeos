use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
};
use vibeos_test::{
    callback::{Callback, Host},
    vm::VmManager,
    Error, Result,
};

struct Runtime<'a> {
    root: &'a Path,
    vms: &'a mut VmManager,
    owner: u32,
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write,
}
impl Host for Runtime<'_> {
    fn call(&mut self, operation: &str, mut arguments: Value) -> Result<Value> {
        if operation == "vm_create" {
            let map = arguments
                .as_object_mut()
                .ok_or("VM arguments must be an object")?;
            let config = if map.get("config").is_some_and(Value::is_object) {
                map.get_mut("config").unwrap().as_object_mut().unwrap()
            } else {
                map
            };
            config.entry("owner_pid").or_insert(json!(self.owner));
        }
        let request = json!({"op":operation,"kwargs":arguments});
        match operation {
            "bench_medium_state" | "bench_drop_state" => Callback {
                input: self.input,
                output: self.output,
            }
            .call(operation, request["kwargs"].clone()),
            op if op.starts_with("bench_service_") => {
                vibeos_test::bench_services_control::dispatch(self.root, &request, self)
            }
            op if op.starts_with("bench_stage_") => {
                vibeos_test::bench_stage::dispatch(self.root, &request)
            }
            op if op.starts_with("bench_") => {
                vibeos_test::bench::dispatch(self.root, &request, self)
            }
            "tooling_runtime"
            | "tooling_provision_lint"
            | "rr_execute"
            | "diagnostic_hmp"
            | "diagnostic_begin_sample"
            | "diagnostic_end_sample"
            | "rr_cli_alive" => Callback {
                input: self.input,
                output: self.output,
            }
            .call(operation, request["kwargs"].clone()),
            "build_arguments"
            | "build_entry_fact"
            | "build_resources_open"
            | "build_source_acquire"
            | "build_reservation_open"
            | "build_reservation_close"
            | "build_resources_close"
            | "build_locked"
            | "build_supervised"
            | "build_supervisor_open"
            | "build_supervisor_close"
            | "build_supervisor_record" => Callback {
                input: self.input,
                output: self.output,
            }
            .call(operation, request["kwargs"].clone()),
            op if op.starts_with("cli_build_entry_") => {
                struct Adapter<'a>(&'a mut dyn Host);
                impl vibeos_build::cli_build::Host for Adapter<'_> {
                    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
                        self.0.call(op, a)
                    }
                }
                vibeos_build::cli_build_entry::dispatch(self.root, &request, &mut Adapter(self))
            }
            "build_fact"
            | "build_external"
            | "build_helper"
            | "gate_task_submit"
            | "gate_task_poll"
            | "gate_preflight"
            | "gate_tree_state"
            | "gate_kmtest"
            | "gate_ladder"
            | "gate_tooling_waits"
            | "gate_stamp"
            | "gate_command"
            | "gate_triage"
            | "gate_starved"
            | "gate_scanner_error"
            | "gate_source_inventory"
            | "gate_binary_cohab" => Callback {
                input: self.input,
                output: self.output,
            }
            .call(operation, request["kwargs"].clone()),
            op if op.starts_with("gate_child_") => {
                vibeos_test::gate_child::dispatch(self.root, &request, self)
            }
            op if op.starts_with("gate_stamp_") => {
                vibeos_test::gate_stamp::dispatch(self.root, &request)
            }
            op if op.starts_with("gate_") => {
                vibeos_test::gate_run::dispatch(self.root, &request, self)
            }
            op if op.starts_with("cli_canon_") => {
                Ok(vibeos_build::cli_canon::dispatch(self.root, &request)?)
            }
            op if op.starts_with("cli_build_helper_") => {
                struct Adapter<'a>(&'a mut dyn Host);
                impl vibeos_build::cli_build::Host for Adapter<'_> {
                    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
                        self.0.call(op, a)
                    }
                }
                vibeos_build::cli_build_helpers::dispatch(self.root, &request, &mut Adapter(self))
            }
            op if op.starts_with("cli_build_check_") => Ok(
                vibeos_build::cli_build_checks::dispatch(self.root, &request)?,
            ),
            op if op.starts_with("cli_build_") => {
                struct Adapter<'a>(&'a mut dyn Host);
                impl vibeos_build::cli_build::Host for Adapter<'_> {
                    fn call(&mut self, op: &str, a: Value) -> Result<Value> {
                        self.0.call(op, a)
                    }
                }
                vibeos_build::cli_build::dispatch(self.root, &request, &mut Adapter(self))
            }
            op if op.starts_with("tooling_receipts_") => Ok(
                vibeos_build::tooling_receipts::dispatch(self.root, &request)?,
            ),
            op if op.starts_with("cli_report_") => vibeos_test::cli_report::dispatch(&request),
            op if op.starts_with("tooling_") => {
                vibeos_test::tooling::dispatch(self.root, &request, self)
            }
            op if op.starts_with("cli_arguments_") => {
                Ok(vibeos_build::cli_arguments::dispatch(&request)?)
            }
            op if op.starts_with("cli_artifacts_") => {
                Ok(vibeos_build::cli_artifacts::dispatch(self.root, &request)?)
            }
            op if op.starts_with("record_replay_") => {
                vibeos_test::record_replay::dispatch(&request)
            }
            op if op.starts_with("pcsample_") => {
                vibeos_test::pc_sample::dispatch(self.root, &request, self)
            }
            op if op.starts_with("rrsample_") => vibeos_test::replay_sample::dispatch(&request),
            op if op.starts_with("diagnostics_") => vibeos_test::diagnostics::dispatch(&request),
            op if op.starts_with("difftest_") => vibeos_test::difftest::dispatch(&request),
            op if op.starts_with("stress_") => vibeos_test::stress::dispatch(&request),
            op if op.starts_with("debugcmd_") => vibeos_test::debugcmd::dispatch(&request),
            op if op.starts_with("winref_") => vibeos_test::winref::dispatch(&request),
            op if op.starts_with("rr_cli_") => vibeos_test::rr_cli::dispatch(&request, self),
            "payload_download"
            | "payload_print"
            | "payload_sleep"
            | "payload_qmp"
            | "payload_screendump"
            | "payload_cpu_state"
            | "payload_save_hang"
            | "payload_read"
            | "payload_clock"
            | "payload_on_output"
            | "payload_on_silence"
            | "command"
            | "log"
            | "stage"
            | "set_run"
            | "get_run"
            | "finalize"
            | "print"
            | "env"
            | "payload_stamp"
            | "host_quiet"
            | "debug_owner"
            | "ledger_append"
            | "create_run_dir"
            | "conformance_signatures"
            | "ladder_clock"
            | "ladder_read_serial"
            | "launch_make"
            | "launch_start"
            | "launch_stop"
            | "launch_log" => Callback {
                input: self.input,
                output: self.output,
            }
            .call(operation, request["kwargs"].clone()),
            "vm_reap_orphan_qemus" | "vm_process_cpu_pct" | "vm_per_arch_cap" => {
                vibeos_test::lifecycle::dispatch(self.root, &request)
            }
            // Add BEFORE the existing broad vm_/serial_/qmp_ arm in Runtime::call.
            // Callback borrows only the runtime's IO, while serial_with_probe borrows Manager.
            op if op.starts_with("serial_") && request["kwargs"]["alive_probe"] == true => {
                let mut callback = Callback {
                    input: self.input,
                    output: self.output,
                };
                let mut probe = || -> Result<bool> {
                    callback
                        .call("serial_alive", json!({}))?
                        .as_bool()
                        .ok_or_else(|| Error::new("serial_alive callback must return a boolean"))
                };
                self.vms.serial_with_probe(&request, &mut probe)
            }
            op if op.starts_with("portlease_")
                || op.starts_with("vm_")
                || op.starts_with("serial_")
                || op.starts_with("qmp_") =>
            {
                self.vms.handle(self.root, &request)
            }
            op if op.starts_with("verdict_debugcon_") => {
                vibeos_test::debugcon::dispatch(self.root, &request)
            }
            op if op.starts_with("verdict_") => vibeos_test::verdict::dispatch(self.root, &request),
            op if op.starts_with("payload_") => {
                vibeos_test::payload::dispatch(self.root, &request, self)
            }
            op if op.starts_with("backend_") => vibeos_test::backend::dispatch(self.root, &request),
            op if op.starts_with("conformance_parse_") => {
                vibeos_test::conformance::dispatch(self.root, &request)
            }
            op if op.starts_with("characterize_") => {
                vibeos_test::characterize::dispatch(self.root, &request, self)
            }
            op if op.starts_with("wine_failures_") => {
                vibeos_test::wine_failures::dispatch(self.root, &request)
            }
            op if op.starts_with("provenance_") => {
                vibeos_build::provenance::dispatch(self.root, &request)
            }
            "test_launch_retry" => vibeos_test::lifecycle::launch_retry(self, &request["kwargs"]),
            "smoke_fixture" | "smoke_settle_script" | "smoke_launch_script" => {
                vibeos_test::smoke_fixtures::dispatch(self.root, &request)
            }
            "smoke_one_boot" | "smoke_watch_script" => {
                vibeos_test::smoke::dispatch(self.root, &request, self)
            }
            op if op.starts_with("smoke_check_") => {
                vibeos_test::smoke_checks::dispatch(self.root, &request)
            }
            op if op.starts_with("smoke_") => {
                vibeos_test::smoke_run::dispatch(self.root, &request, self)
            }
            op if op.starts_with("checkpoint_") => {
                vibeos_test::checkpoint::dispatch(self.root, &request)
            }
            "ladder_watch" => vibeos_test::ladder_watch::dispatch(self.root, &request, self),
            op if op.starts_with("ladder_") => vibeos_test::ladder::dispatch(self.root, &request),
            op if op.starts_with("farm_") => vibeos_test::farm::dispatch(self.root, &request, self),
            op if op.starts_with("conformance_") => {
                vibeos_test::conformance_run::dispatch(self.root, &request, self)
            }

            _ => Err(Error::new(format!("unknown test operation {operation}"))),
        }
    }
}
fn dispatch(host: &mut Runtime<'_>, request: &Value) -> Result<Value> {
    let op = request["op"].as_str().ok_or("missing test operation")?;
    match op {
        "test_version" => Ok(json!({"version":1,"native":true})),
        _ => host.call(
            op,
            request.get("kwargs").cloned().unwrap_or_else(|| json!({})),
        ),
    }
}
fn run() -> Result<u8> {
    let mut args = std::env::args().skip(1);
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut lines = false;
    let mut owner = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = PathBuf::from(args.next().ok_or("--root requires a directory")?),
            "--client-pid" => {
                owner = Some(
                    args.next()
                        .ok_or("--client-pid requires a PID")?
                        .parse::<u32>()
                        .map_err(|_| Error::new("invalid client PID"))?,
                )
            }
            "--api-jsonl" | "--resource-jsonl" => lines = true,
            "--api-json" => {}
            "cli-command-worker" => {
                return Ok(vibeos_build::cli_artifacts::worker(
                    &args.collect::<Vec<_>>(),
                )?);
            }
            "owned-command-fd" => {
                return Ok(vibeos_build::process_guard::runner(
                    &args.collect::<Vec<_>>(),
                )?);
            }
            "vm-guardian" => {
                return vibeos_test::vm::guardian_worker(Path::new(
                    &args.next().ok_or("VM guardian requires descriptor")?,
                ))
            }
            "--diagnostic-worker" => {
                return vibeos_test::diagnostics::worker(
                    &args
                        .next()
                        .ok_or("--diagnostic-worker requires operation")?,
                )
            }
            "--help" | "-h" => {
                println!("Usage: tools/test-infra [--root DIR] [--client-pid PID] --api-json|--api-jsonl|--resource-jsonl");
                return Ok(0);
            }
            _ => return Err(Error::new(format!("unknown test option {arg}"))),
        }
    }
    let owner = owner.unwrap_or(unsafe { libc::getppid() } as u32);
    vibeos_build::artifact_lifetime::watch_owner(owner)?;
    let input = std::io::stdin();
    let mut input = input.lock();
    let output = std::io::stdout();
    let mut output = output.lock();
    let mut vms = VmManager::default();
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let result = match serde_json::from_str::<Value>(&line) {
            Ok(request) => dispatch(
                &mut Runtime {
                    root: &root,
                    vms: &mut vms,
                    owner,
                    input: &mut input,
                    output: &mut output,
                },
                &request,
            ),
            Err(error) => Err(error.into()),
        };
        let response = match result {
            Ok(value) => json!({"ok":value}),
            Err(error) => json!({"error":{"message":error.message,"kind":error.kind}}),
        };
        serde_json::to_writer(&mut output, &response)?;
        output.write_all(b"\n")?;
        output.flush()?;
        if !lines {
            break;
        }
    }
    vms.shutdown_all()?;
    Ok(0)
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(code) => std::process::ExitCode::from(code),
        Err(error) => {
            eprintln!("test-infra: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
