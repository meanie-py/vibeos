//! Apitest characterization: own each round's evidence before attributing a guest fault.
use crate::{
    callback::Host,
    conformance::{self, TestResult},
    Error, Result,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub const SAME_SUITE_LIMIT: usize = 3;
pub fn should_quarantine(bugcheck: Option<&str>, harness: &str) -> bool {
    bugcheck.is_some_and(|s| !s.is_empty()) || harness.contains("[payload] drain-end: idle")
}
pub fn archive(serial: &Path, run: &Path, round: u64) -> Result<PathBuf> {
    fs::create_dir_all(run)?;
    let dest = run.join(format!(
        "round{round:02}-{}",
        serial
            .file_name()
            .ok_or("serial has no filename")?
            .to_string_lossy()
    ));
    // No fallback to a moving pytest path if durable evidence cannot be written.
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&dest)?;
    std::io::copy(&mut fs::File::open(serial)?, &mut out)?;
    out.sync_all()?;
    Ok(dest)
}
fn emit(host: &mut dyn Host, text: impl Into<String>) -> Result<()> {
    host.call("print", json!({"text":text.into()}))?;
    Ok(())
}
fn command(
    host: &mut dyn Host,
    root: &Path,
    argv: Vec<String>,
    env: &BTreeMap<String, String>,
    timeout: u64,
    stage: &str,
) -> Result<Value> {
    host.call(
        "command",
        json!({"argv":argv,"cwd":root,"env":env,"timeout":timeout,"stage":stage}),
    )
}
fn success(v: &Value) -> bool {
    v["returncode"].as_i64() == Some(0) && !v["timed_out"].as_bool().unwrap_or(false)
}
fn arg_path(args: &Value, key: &str) -> Option<PathBuf> {
    args[key].as_str().map(PathBuf::from)
}

pub fn run(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = &request["args"];
    let arch = vibeos_build::arch(args["arch"].as_str().ok_or("missing arch")?)?;
    let pa = if arch == "arm64" {
        arch.to_owned()
    } else {
        format!("{arch}-uefi")
    };
    let rounds = args["rounds"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("rounds must be positive")?;
    let only: Vec<String> = if args["only"].is_null() {
        Vec::new()
    } else {
        serde_json::from_value(args["only"].clone())?
    };
    let ignore = args["ignore_quarantine"].as_bool().unwrap_or(false);
    let no_quarantine = ignore || args["no_quarantine"].as_bool().unwrap_or(false);
    let reproducing = ignore || !only.is_empty();
    let base = arg_path(args, "artifact_dir").unwrap_or_else(|| PathBuf::from("/tmp/vibeos-cli"));
    fs::create_dir_all(&base)?;
    // Atomic uniqueness also covers two invocations in the same second.
    let run_dir = tempfile::Builder::new()
        .prefix(&format!("characterize-{arch}-"))
        .tempdir_in(base)?
        .keep();
    emit(
        host,
        format!("[characterize] artifacts: {}", run_dir.display()),
    )?;
    if reproducing {
        emit(
            host,
            "[characterize] REPRODUCTION run — coverage is partial; do NOT bank its results table.",
        )?;
    }
    let env: BTreeMap<String, String> = serde_json::from_value(request["env"].clone())?;
    let python = request["python"]
        .as_str()
        .ok_or("missing Python test entry point")?;
    let mut merged: BTreeMap<String, TestResult> = BTreeMap::new();
    let mut culprits: Vec<String> = Vec::new();
    let mut evidence = Vec::new();
    let mut complete = false;
    let mut stop = "round limit".to_owned();
    for round in 1..=rounds {
        emit(
            host,
            format!("[characterize] ===== round {round}/{rounds} ====="),
        )?;
        let mut gen = vec![
            root.join("tools/codegen").to_string_lossy().into_owned(),
            "apitest-suites".into(),
            "--arch".into(),
            arch.into(),
        ];
        for name in &only {
            gen.extend(["--only".into(), name.clone()]);
        }
        if ignore {
            gen.push("--ignore-quarantine".into());
        }
        if !success(&command(
            host,
            root,
            gen,
            &env,
            300,
            "characterize:generate",
        )?) {
            return Err("apitest suite generation failed; characterization stopped".into());
        }
        let mut build_env = env.clone();
        build_env.insert("VIBEOS_PAYLOAD".into(), "apitest".into());
        if !success(&command(
            host,
            root,
            vec![
                root.join("vibeos-cli").to_string_lossy().into_owned(),
                "build".into(),
                "--arch".into(),
                arch.into(),
                "--apitests".into(),
            ],
            &build_env,
            14400,
            "characterize:build",
        )?) {
            return Err("apitest build failed; characterization stopped".into());
        }
        let boot_dir = run_dir.join(format!("round{round:02}"));
        fs::create_dir(&boot_dir)?;
        let mut boot_env = env.clone();
        boot_env.insert("VIBEOS_QEMU_MAX_LIFETIME".into(), "8400".into());
        boot_env.insert(
            "VIBEOS_ARTIFACT_DIR".into(),
            boot_dir.to_string_lossy().into_owned(),
        );
        let boot = command(
            host,
            root,
            vec![
                python.into(),
                "-m".into(),
                "pytest".into(),
                "tests/qemu/test_payload.py".into(),
                "--arch".into(),
                pa.clone(),
                "--payload".into(),
                "apitest".into(),
                "--skip-clean-build".into(),
                "--timeout".into(),
                "7800".into(),
                "-q".into(),
            ],
            &boot_env,
            8100,
            "characterize:boot",
        )?;
        let serial = boot_dir.join(format!("{pa}_serial.log"));
        if !serial.is_file() {
            return Err(Error::new(format!("no serial from this characterization round at {}; refusing to select another run's log",serial.display())));
        }
        let serial = archive(&serial, &run_dir, round)?;
        let (rows, meta) = conformance::parse_apitest(&vibeos_build::read(&serial)?)?;
        for row in &rows {
            let replace = merged.get(&row.name).is_none_or(|prev| {
                conformance::is_bad(row.status.as_deref())
                    && !conformance::is_bad(prev.status.as_deref())
            });
            if replace {
                merged.insert(row.name.clone(), row.clone());
            }
        }
        complete = meta["done"].as_str().is_some_and(|s| !s.is_empty());
        evidence.push(json!({"round":round,"serial":serial,"meta":meta,"returncode":boot["returncode"],"timed_out":boot["timed_out"]}));
        emit(
            host,
            format!(
                "[characterize] round {round}: {} subtests, {} known; sweep {}",
                rows.len(),
                merged.len(),
                if complete { "done" } else { "INCOMPLETE" }
            ),
        )?;
        if complete {
            stop = "sweep completed".into();
            break;
        }
        let Some(culprit) = rows
            .iter()
            .rev()
            // TIMEOUT/FAILSTART/ENUM-TIMEOUT are resolved by the sweep even
            // without an exit marker. They cannot explain a later bugcheck.
            .find(|r| {
                r.exit.is_none()
                    && matches!(r.status.as_deref(), Some("CRASH" | "NO-END" | "INCOMPLETE"))
            })
            .map(|r| r.name.clone())
        else {
            stop = "sweep incomplete but no in-flight subtest".into();
            break;
        };
        let harness = format!(
            "{}\n{}",
            boot["stdout"].as_str().unwrap_or(""),
            boot["stderr"].as_str().unwrap_or("")
        );
        if !should_quarantine(meta["bugcheck"].as_str(), &harness) {
            stop = format!("nothing positively implicates the guest; {culprit:?} was merely in flight and is NOT quarantined");
            break;
        }
        // Normalize enumeration before duplicate detection as well as publication.
        let culprit = culprit
            .strip_suffix(" <enumeration>")
            .unwrap_or(&culprit)
            .to_owned();
        if culprits.contains(&culprit) {
            stop = format!("{culprit} was already quarantined; attribution did not converge");
            break;
        }
        let suite = culprit.split_whitespace().next().unwrap_or("");
        if culprits
            .iter()
            .filter(|c| c.split_whitespace().next() == Some(suite))
            .count()
            >= SAME_SUITE_LIMIT - 1
        {
            stop = format!("{SAME_SUITE_LIMIT} rounds accused {suite}; triage the shared cause instead of spending a boot per victim");
            break;
        }
        let why = meta["bugcheck"]
            .as_str()
            .map(|b| format!("BUGCHECK {b}"))
            .unwrap_or_else(|| {
                "guest went SILENT here (idle cutoff) — a wedge, not a rig timeout".into()
            });
        if no_quarantine {
            stop = format!(
                "would have quarantined {culprit:?} ({why}); not written (--no-quarantine)"
            );
            culprits.push(culprit);
            break;
        }
        let quarantine = root.join(format!("tools/payloads/apitest/quarantine-{arch}.txt"));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(quarantine)?;
        writeln!(file,"\n# characterize.py round {round}: sweep stopped here — {why}.\n# serial: {}\n{culprit}",serial.display())?;
        file.sync_all()?;
        emit(
            host,
            format!("[characterize] quarantined {culprit:?} ({why})"),
        )?;
        culprits.push(culprit);
    }
    let out = arg_path(args, "out")
        .unwrap_or_else(|| root.join(format!("tools/conformance-baseline/{arch}/apitest.txt.new")));
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut text = "# apitest conformance results — parse_apitest_serial.py format\n# MERGED ACROSS ROUNDS by characterize.py: no single boot sees every\n# subtest while crashers remain, so counts are commentary and the\n# assertion identities are the floor.\n".to_owned();
    for row in merged.values() {
        text.push_str(&conformance::fmt(row, &BTreeSet::new()));
        text.push('\n');
    }
    fs::write(&out, text)?;
    fs::write(
        run_dir.join("characterize.json"),
        serde_json::to_vec_pretty(
            &json!({"complete":complete,"partial":reproducing,"stop":stop,"rounds":evidence,"culprits":culprits,"out":out}),
        )?,
    )?;
    emit(
        host,
        format!(
            "[characterize] {stop}; wrote {} — {} subtests",
            out.display(),
            merged.len()
        ),
    )?;
    host.call(
        "stage",
        json!({"name":"characterize","status":if complete {"ok"}else{"warn"},"detail":stop}),
    )?;
    Ok(json!({"code":0,"rounds":evidence,"complete":complete,"artifact_dir":run_dir,"out":out}))
}
pub fn dispatch(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = &request["kwargs"];
    match request["op"].as_str().unwrap_or("") {
        "characterize_run" => run(root, a, host),
        "characterize_should_quarantine" => Ok(json!(should_quarantine(
            a["bugcheck"].as_str(),
            a["harness"].as_str().ok_or("missing harness")?
        ))),
        "characterize_archive" => Ok(json!(archive(
            Path::new(a["serial"].as_str().ok_or("missing serial")?),
            Path::new(a["run_dir"].as_str().ok_or("missing run_dir")?),
            a["round"].as_u64().ok_or("missing round")?
        )?)),
        _ => Err("unknown characterization operation".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        boots: Vec<(String, String)>,
        commands: Vec<Value>,
    }
    impl Host for Fixture {
        fn call(&mut self, op: &str, a: Value) -> Result<Value> {
            if op != "command" {
                return Ok(Value::Null);
            }
            self.commands.push(a.clone());
            if a["stage"] != "characterize:boot" {
                return Ok(json!({"returncode":0,"timed_out":false}));
            }
            let (serial, harness) = self.boots.remove(0);
            let env = &a["env"];
            assert_eq!(env["VIBEOS_QEMU_MAX_LIFETIME"], "8400");
            assert!(a["argv"].as_array().unwrap().contains(&json!("7800")));
            if serial != "MISSING" {
                let dir = Path::new(env["VIBEOS_ARTIFACT_DIR"].as_str().unwrap());
                fs::write(dir.join("amd64-uefi_serial.log"), serial)?;
            }
            Ok(json!({"returncode":1,"stdout":harness,"stderr":"","timed_out":false}))
        }
    }
    fn request(base: &Path) -> Value {
        json!({"args":{"arch":"amd64","rounds":12,"artifact_dir":base.join("artifacts"),"out":base.join("result.txt")},"env":{},"python":"fixture-python"})
    }
    fn fault(name: &str) -> String {
        format!("[APITEST-SWEEP] launch {name}\n*** BUGCHECK: 0x1E\n")
    }
    fn root() -> tempfile::TempDir {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join("tools/payloads/apitest")).unwrap();
        t
    }
    #[test]
    fn harness_budget_never_accuses_a_guest_and_keeps_owned_evidence() {
        let t = root();
        let mut host = Fixture {
            boots: vec![(
                "[APITEST-SWEEP] launch x.exe test\n".into(),
                "[payload] drain-end: budget".into(),
            )],
            commands: vec![],
        };
        let result = run(t.path(), &request(t.path()), &mut host).unwrap();
        assert_eq!(result["complete"], false);
        assert_eq!(result["rounds"].as_array().unwrap().len(), 1);
        let serial = Path::new(result["rounds"][0]["serial"].as_str().unwrap());
        assert!(serial.is_file());
        assert!(!t
            .path()
            .join("tools/payloads/apitest/quarantine-amd64.txt")
            .exists());
    }
    #[test]
    fn later_skip_cannot_erase_the_crash_and_enumeration_quarantines_the_suite() {
        let t = root();
        let mut host = Fixture {
            boots: vec![
                (
                    "[APITEST-SWEEP] enumerate x.exe\n*** BUGCHECK: 0x1E\n".into(),
                    String::new(),
                ),
                (
                    "[APITEST-SWEEP] SKIP x.exe <enumeration>\n[APITEST-SWEEP] done total=1\n"
                        .into(),
                    String::new(),
                ),
            ],
            commands: vec![],
        };
        let result = run(t.path(), &request(t.path()), &mut host).unwrap();
        assert_eq!(result["complete"], true);
        let q = fs::read_to_string(t.path().join("tools/payloads/apitest/quarantine-amd64.txt"))
            .unwrap();
        assert!(q.ends_with("\nx.exe\n"));
        let text = fs::read_to_string(t.path().join("result.txt")).unwrap();
        assert!(text.contains("CRASH"));
        assert_eq!(result["rounds"].as_array().unwrap().len(), 2);
    }
    #[test]
    fn same_suite_limit_is_cumulative_across_an_unrelated_round() {
        let t = root();
        let mut host = Fixture {
            boots: ["x.exe A", "y.exe A", "x.exe B", "x.exe C"]
                .iter()
                .map(|n| (fault(n), String::new()))
                .collect(),
            commands: vec![],
        };
        let result = run(t.path(), &request(t.path()), &mut host).unwrap();
        assert_eq!(result["rounds"].as_array().unwrap().len(), 4);
        let q = fs::read_to_string(t.path().join("tools/payloads/apitest/quarantine-amd64.txt"))
            .unwrap();
        assert!(q.contains("x.exe A") && q.contains("x.exe B") && !q.contains("x.exe C"));
    }
    #[test]
    fn ignore_quarantine_forwards_selection_and_never_rewrites_the_quarantine() {
        let t = root();
        let mut a = request(t.path());
        a["args"]["ignore_quarantine"] = json!(true);
        a["args"]["only"] = json!(["x.exe"]);
        let mut host = Fixture {
            boots: vec![(fault("x.exe A"), String::new())],
            commands: vec![],
        };
        let r = run(t.path(), &a, &mut host).unwrap();
        assert_eq!(r["rounds"].as_array().unwrap().len(), 1);
        assert!(!t
            .path()
            .join("tools/payloads/apitest/quarantine-amd64.txt")
            .exists());
        let argv = host.commands[0]["argv"].as_array().unwrap();
        assert!(argv.contains(&json!("--ignore-quarantine")) && argv.contains(&json!("--only")));
    }
    #[test]
    fn a_missing_owned_serial_is_an_error_instead_of_using_an_older_log() {
        let t = root();
        let mut host = Fixture {
            boots: vec![("MISSING".into(), String::new())],
            commands: vec![],
        };
        let error = run(t.path(), &request(t.path()), &mut host).unwrap_err();
        assert!(error.message.contains("refusing to select another run"));
        assert!(!t.path().join("result.txt").exists());
    }
    #[test]
    fn a_resolved_timeout_is_not_accused_of_a_later_bugcheck() {
        let t = root();
        let mut host = Fixture {
            boots: vec![(
                "[APITEST-SWEEP] TIMEOUT x.exe A (60s) - terminated\n*** BUGCHECK: 0x1E\n".into(),
                String::new(),
            )],
            commands: vec![],
        };
        let mut args = request(t.path());
        args["args"]["rounds"] = json!(1);
        let result = run(t.path(), &args, &mut host).unwrap();
        assert_eq!(result["rounds"].as_array().unwrap().len(), 1);
        assert!(!t
            .path()
            .join("tools/payloads/apitest/quarantine-amd64.txt")
            .exists());
    }
    #[test]
    fn archival_refuses_to_overwrite_existing_evidence() {
        let t = root();
        let source = t.path().join("serial.log");
        fs::write(&source, "first").unwrap();
        let dest = archive(&source, &t.path().join("archive"), 1).unwrap();
        fs::write(&source, "second").unwrap();
        assert!(archive(&source, &t.path().join("archive"), 1).is_err());
        assert_eq!(fs::read_to_string(dest).unwrap(), "first");
    }
}
