//! Gate child command policy and durable verdict recovery.
use crate::{callback::Host, gate_run, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Instant, UNIX_EPOCH},
};
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v[k].as_str().unwrap_or(d)
}
fn text_lines(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|s| {
            s.as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| s.to_string())
        })
        .collect()
}
pub fn last_words(text: &str, limit: usize) -> String {
    text.lines()
        .rev()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .map(|s| s.chars().take(limit).collect())
        .unwrap_or_else(|| "(no output at all)".into())
}
pub fn summary(root: &Path) -> Value {
    let mut hits = fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path().join("summary.json")))
        .filter(|p| p.is_file())
        .collect::<Vec<_>>();
    hits.sort();
    let Some(p) = hits.last() else {
        return json!({});
    };
    let mut data = fs::read(p)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .unwrap_or(json!({}));
    if data["incomplete"] == true || !data.is_object() {
        return json!({});
    }
    data["_dir"] = json!(p.parent());
    data
}
pub fn salient(summary: &Value) -> String {
    for st in summary["stages"].as_array().into_iter().flatten() {
        if matches!(st["status"].as_str(), Some("warn" | "fail")) {
            if let Some(v) = st["salient"].as_array().and_then(|a| a.first()) {
                return v
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string());
            }
        }
    }
    String::new()
}
pub fn classification(stdout: &str) -> String {
    if stdout
        .lines()
        .any(|s| s.starts_with("DISK_ADMISSION_FAIL:"))
    {
        return "DISK_ADMISSION_FAIL".into();
    }
    for line in stdout.lines() {
        if let Some(path) = line
            .strip_prefix("full log:")
            .map(str::trim)
            .and_then(|s| s.strip_suffix("/full.log"))
        {
            let file = Path::new(path).join("summary.json");
            return fs::read(file)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .map(|v| s(&v, "classification", "").to_owned())
                .unwrap_or_default();
        }
    }
    String::new()
}
fn execute(
    host: &mut dyn Host,
    repo: &Path,
    root: &Path,
    cmd: Value,
    env: Value,
    timeout: Value,
) -> Result<Value> {
    host.call(
        "gate_command",
        json!({"argv":cmd,"cwd":repo,"root":root,"env":env,"timeout":timeout}),
    )
}
fn console(cp: &Value) -> String {
    format!(
        "{}{}",
        s(cp, "stdout", ""),
        if s(cp, "stderr", "").is_empty() {
            String::new()
        } else {
            format!("\n--- stderr ---\n{}", s(cp, "stderr", ""))
        }
    )
}
fn env(a: &Value) -> Value {
    a.get("env")
        .cloned()
        .unwrap_or_else(|| json!(std::env::vars().collect::<BTreeMap<_, _>>()))
}
pub fn smoke(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let root = PathBuf::from(s(a, "root", ""));
    fs::create_dir_all(&root)?;
    let arch = s(a, "arch", "");
    let mut cmd = vec![
        repo.join("vibeos-cli").to_string_lossy().into_owned(),
        "smoke".into(),
        "--arch".into(),
        arch.into(),
    ];
    cmd.extend(text_lines(&a["extra"]));
    let mut env = env(a);
    env["VIBEOS_ARTIFACT_ROOT"] = json!(root);
    let start = Instant::now();
    let cp = execute(host, repo, &root, json!(cmd), env, Value::Null)?;
    let seconds = start.elapsed().as_secs_f64();
    let console = console(&cp);
    let log = root.join("smoke-console.log");
    fs::write(&log, &console)?;
    let summary = summary(&root);
    let rc = cp["returncode"].as_i64().unwrap_or(1);
    if summary.as_object().is_none_or(|s| s.is_empty()) {
        return Ok(
            json!({"arch":arch,"verdict":"FAIL","classification":"SMOKE_NO_SUMMARY","ok":false,"retryable":true,"exit":rc,"seconds":seconds,"run_dir":root,"salient":format!("smoke --arch {arch} exited {rc} after {seconds:.1}s without recording a verdict — it died before the boot could return one. Last output: {}  (full: {})",last_words(&console,240),log.display())}),
        );
    }
    let verdict = s(&summary, "verdict", if rc == 0 { "OK" } else { "FAIL" });
    let class = s(
        &summary,
        "classification",
        if rc == 0 { "" } else { "FAIL" },
    );
    Ok(
        json!({"arch":arch,"verdict":verdict,"classification":class,"ok":matches!(verdict,"OK"|"WARN"),"retryable":gate_run::retryable(class,&summary),"hang_kind":summary["hang_kind"],"wedge_confirmed":summary["wedge_confirmed"]==true,"census_stuck":summary.get("census_stuck").cloned().unwrap_or(json!(0)),"exit":rc,"seconds":seconds,"run_dir":summary.get("_dir").cloned().unwrap_or(json!(root)),"flake_rate":summary["flake_rate"],"screenshot":summary["screenshot"],"salient":salient(&summary)}),
    )
}
pub fn conformance(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let root = PathBuf::from(s(a, "root", ""));
    fs::create_dir_all(&root)?;
    let arch = s(a, "arch", "");
    let start = Instant::now();
    let baseline = repo.join(format!("tools/conformance-baseline/{arch}/kmtest.txt"));
    if !baseline.is_file() {
        return Ok(
            json!({"arch":format!("conf:{arch}"),"verdict":"FAIL","classification":"CONFORMANCE_NO_BASELINE","ok":false,"retryable":false,"seconds":0.,"run_dir":root,"salient":format!("no baseline at {}",baseline.display())}),
        );
    }
    let cp = execute(
        host,
        repo,
        &root,
        json!([
            repo.join("vibeos-cli"),
            "conformance",
            "--arch",
            arch,
            "--gate",
            "--no-build"
        ]),
        env(a),
        Value::Null,
    )?;
    let output = format!("{}{}", s(&cp, "stdout", ""), s(&cp, "stderr", ""));
    fs::write(root.join("conformance.log"), &output)?;
    let ok = cp["returncode"] == 0;
    let mut salient = output
        .lines()
        .filter(|s| s.contains("NEW failing assertion"))
        .map(str::trim)
        .take(6)
        .collect::<Vec<_>>();
    if !ok && salient.is_empty() {
        salient = output
            .lines()
            .filter(|s| s.contains("->") && s.contains(':'))
            .map(str::trim)
            .take(6)
            .collect()
    }
    Ok(
        json!({"arch":format!("conf:{arch}"),"verdict":if ok{"PASS"}else{"RATCHET"},"classification":if ok{""}else{"CONFORMANCE_RED"},"ok":ok,"retryable":!ok,"seconds":start.elapsed().as_secs_f64(),"run_dir":root,"salient":if salient.is_empty(){if ok{String::new()}else{format!("conformance exited {}",cp["returncode"])}}else{salient.join("; ")}}),
    )
}
pub fn stortest(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let root = PathBuf::from(s(a, "root", ""));
    fs::create_dir_all(&root)?;
    let arch = s(a, "arch", "");
    let vm_arch = if arch == "amd64" {
        "amd64-uefi"
    } else if arch == "i386" {
        "i386-uefi"
    } else {
        arch
    };
    let start = Instant::now();
    let result = root.join("stortest.json");
    let mut env = env(a);
    env["VIBEOS_QEMU_STORTEST"] = json!("1");
    env["VIBEOS_STORTEST_RESULT"] = json!(result);
    env["VIBEOS_ARTIFACT_DIR"] = json!(root);
    let cp = execute(
        host,
        repo,
        &root,
        json!([
            s(a, "python", "python3"),
            "-m",
            "pytest",
            repo.join("tests/qemu/test_stortest.py"),
            "--arch",
            vm_arch,
            "--skip-clean-build",
            // This lane already owns a supervised 900s process-tree deadline.
            // Pytest's global 300s also counts bounded fixture launch retries,
            // so it can interrupt a healthy boot after port-lock contention.
            "--timeout=0",
            "-q",
            "-p",
            "no:cacheprovider"
        ]),
        env,
        json!(900),
    )?;
    let out = format!("{}{}", s(&cp, "stdout", ""), s(&cp, "stderr", ""));
    let rc = if cp["timed_out"] == true {
        -1
    } else {
        cp["returncode"].as_i64().unwrap_or(1)
    };
    let log = root.join("stortest.log");
    fs::write(&log, &out)?;
    let verdict = fs::read(&result)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    let (cls, sal) = if let Some(v) = &verdict {
        (
            s(v, "classification", "").to_string(),
            text_lines(&v["salient"])
                .into_iter()
                .take(4)
                .collect::<Vec<_>>()
                .join("; "),
        )
    } else if rc == 0 {
        (
            "STORTEST_NO_VERDICT".into(),
            "pytest passed but recorded no verdict".into(),
        )
    } else if out.contains("Boot failed") {
        let v = host.call("gate_triage", json!({"text":out}))?;
        (
            s(&v, "classification", "STORTEST_BOOT_FAIL").into(),
            if text_lines(&v["salient"]).is_empty() {
                last_words(&out, 240)
            } else {
                text_lines(&v["salient"])
                    .into_iter()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("; ")
            },
        )
    } else {
        (
            "STORTEST_ERROR".into(),
            out.lines()
                .filter_map(|l| l.strip_prefix("E "))
                .map(str::trim)
                .find(|s| !s.is_empty())
                .map(|s| s.chars().take(300).collect())
                .unwrap_or_else(|| last_words(&out, 240)),
        )
    };
    let ok = rc == 0 && verdict.is_some() && cls.is_empty();
    Ok(
        json!({"arch":format!("stor:{arch}"),"verdict":if ok{"PASS"}else{"FAIL"},"classification":if ok{""}else{&cls},"ok":ok,"retryable":!ok&&verdict.is_none(),"exit":rc,"seconds":start.elapsed().as_secs_f64(),"run_dir":root,"salient":if ok{String::new()}else{format!("{sal}  (log: {})",log.display())}}),
    )
}
pub fn net(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let root = PathBuf::from(s(a, "root", ""));
    fs::create_dir_all(&root)?;
    let test = repo.join("tests/qemu/test_net.py");
    if !test.exists() {
        return Ok(
            json!({"arch":"net","verdict":"FAIL","classification":"NET_UNAVAILABLE","ok":false,"retryable":false,"seconds":0.,"run_dir":root,"salient":"tests/qemu/test_net.py is missing"}),
        );
    }
    let mut env = env(a);
    env["VIBEOS_ARTIFACT_DIR"] = json!(root);
    let start = Instant::now();
    let cp = execute(
        host,
        repo,
        &root,
        json!([
            s(a, "python", "python3"),
            "-m",
            "pytest",
            test,
            "--arch",
            "amd64-uefi-net",
            "--timeout",
            "600",
            "-q",
            "-p",
            "no:cacheprovider",
            "--skip-clean-build"
        ]),
        env,
        json!(1200),
    )?;
    if cp["timed_out"] == true {
        return Ok(
            json!({"arch":"net","verdict":"FAIL","classification":"NET_HANG","ok":false,"retryable":false,"seconds":start.elapsed().as_secs_f64(),"run_dir":root,"salient":"tests/qemu/test_net.py hung > 1200s"}),
        );
    }
    let output = format!("{}\n{}", s(&cp, "stdout", ""), s(&cp, "stderr", ""));
    fs::write(root.join("pytest.out"), &output)?;
    let ok = cp["returncode"] == 0;
    let summary = s(&cp, "stdout", "")
        .lines()
        .rev()
        .find(|s| s.contains(" passed") || s.contains(" failed") || s.contains(" error"))
        .or_else(|| s(&cp, "stdout", "").trim().lines().last())
        .unwrap_or("");
    let starved = !ok && host.call("gate_starved", json!({"text":output}))? == true;
    Ok(
        json!({"arch":"net","verdict":if ok{"OK"}else{"FAIL"},"classification":if ok{""}else if starved{"HOST_STARVED"}else{"NET_FLAKE"},"ok":ok,"retryable":!ok,"seconds":start.elapsed().as_secs_f64(),"run_dir":root,"salient":if ok{""}else if starved{"net lane never booted — queued on host CPU leases the whole run (§9.84); `vibeos-cli ps` names the holder"}else{summary.trim_matches(['=',' '])}}),
    )
}
pub fn build(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let arch = s(a, "arch", "");
    let args = &a["args"];
    let mut cmd = vec![
        repo.join("vibeos-cli").to_string_lossy().into_owned(),
        "build".into(),
        "--arch".into(),
        arch.into(),
    ];
    let restage = a["restage"] == true;
    if args["no_lint"] != true {
        cmd.push("--no-check".into())
    }
    if args["clean"] == true && !restage {
        cmd.push("--clean".into())
    }
    if let Some(store) = args["artifact_store"].as_str().filter(|s| !s.is_empty()) {
        cmd.extend(["--artifact-store".into(), store.into()]);
        if args["artifact_store_verify"] == true && !restage {
            cmd.push("--artifact-store-verify".into())
        }
    }
    if a["defer_wow64"] == true {
        cmd.push("--no-wow64-stage".into())
    }
    if args["keep_going"] == true {
        cmd.push("--keep-going".into())
    }
    let mut env = Value::Null;
    if args["no_conformance"] != true {
        cmd.push("--kmtests".into());
        env = self::env(a);
        env["VIBEOS_PAYLOAD"] = json!("kmtest");
    }
    let start = Instant::now();
    let cp=host.call("command",json!({"argv":cmd,"cwd":repo,"stage":format!("build:{arch}{}",if restage{":restage"}else{""}),"env":env}))?;
    Ok(
        json!({"arch":arch,"ok":cp["returncode"]==0,"exit":cp["returncode"],"seconds":start.elapsed().as_secs_f64(),"classification":classification(s(&cp,"stdout",""))}),
    )
}
pub fn check(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let root = PathBuf::from(s(a, "root", ""));
    fs::create_dir_all(&root)?;
    let kind = s(a, "kind", "");
    let arch = s(a, "arch", "");
    let mut cmd = vec![repo.join("vibeos-cli").to_string_lossy().into_owned()];
    match kind {
        "lint" => {
            cmd.extend(["check".into(), "--strict".into(), "--scope".into()]);
            if arch.starts_with("source") {
                cmd.push(arch.into())
            } else {
                cmd.extend(["arch".into(), "--arch".into(), arch.into()]);
            }
        }
        "tooling" => cmd.push("check-tooling".into()),
        "difftest" => cmd.push("difftest".into()),
        _ => return Err("unknown gate check kind".into()),
    }
    let mut env = env(a);
    env["VIBEOS_ARTIFACT_ROOT"] = json!(root);
    let start = Instant::now();
    let cp = execute(host, repo, &root, json!(cmd), env, Value::Null)?;
    let console = console(&cp);
    let log = root.join(if kind == "tooling" {
        "check-tooling-console.log"
    } else if kind == "difftest" {
        "difftest-console.log"
    } else {
        "lint-console.log"
    });
    fs::write(&log, &console)?;
    let mut sal = s(&cp, "stdout", "")
        .lines()
        .find(|l| {
            if kind == "difftest" {
                l.trim().starts_with('>') || l.contains('✗')
            } else {
                l.contains('✗') || l.contains("FAIL")
            }
        })
        .map(|l| l.trim().trim_start_matches(['>', ' ']).to_owned())
        .unwrap_or_default();
    let mut cls = classification(s(&cp, "stdout", ""));
    let mut inventory = true;
    if kind == "lint" {
        let summary = summary(&root);
        cls = s(&summary, "classification", "").into();
        let error = host.call("gate_scanner_error", cp.clone())? == true;
        if error {
            cls = "SCANNER_ERROR".into()
        }
        if matches!(arch, "source-early" | "source-deferred") {
            let expected = host.call("gate_source_inventory", json!({"arch":arch}))?;
            inventory = summary["checked_source_checks"] == expected;
            if !inventory {
                sal = "missing or incorrect source-partition coverage inventory".into();
                cls = "SCANNER_ERROR".into();
            }
        }
    }
    if sal.is_empty() && cp["returncode"] != 0 {
        sal = format!(
            "exit {}: {}  (full: {})",
            cp["returncode"],
            last_words(&console, 240),
            log.display()
        )
    }
    Ok(
        json!({"arch":arch,"ok":cp["returncode"]==0&&inventory&&cls!="SCANNER_ERROR","seconds":start.elapsed().as_secs_f64(),"salient":sal,"exit":cp["returncode"],"classification":cls}),
    )
}
pub fn cohab(_repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let root = PathBuf::from(s(a, "root", ""));
    fs::create_dir_all(&root)?;
    let start = Instant::now();
    let cp = host.call("gate_binary_cohab", json!({"arch":a["arch"]}))?;
    let console = console(&cp);
    let log = root.join("cohab-console.log");
    fs::write(&log, &console)?;
    let mut sal = s(&cp, "stdout", "")
        .lines()
        .find(|s| s.contains(".rs:"))
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    if sal.is_empty() && cp["returncode"] != 0 {
        sal = format!(
            "exit {}: {}  (full: {})",
            cp["returncode"],
            last_words(&console, 240),
            log.display()
        )
    }
    Ok(
        json!({"arch":a["arch"],"ok":cp["returncode"]==0,"seconds":start.elapsed().as_secs_f64(),"salient":sal,"exit":cp["returncode"]}),
    )
}
pub fn kmtest(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let dir = repo.join("tools/payloads/kmtest");
    let script = dir.join("build.sh");
    if !script.is_file() {
        return Ok(json!([false, format!("no {}", script.display())]));
    }
    let newest = [dir.join("sweep.c"), script.clone()]
        .into_iter()
        .filter_map(|p| fs::metadata(p).ok()?.modified().ok())
        .max()
        .unwrap_or(UNIX_EPOCH);
    let mut missing = vec![];
    for arch in text_lines(&a["arches"]) {
        let exe = dir.join(format!("kmtest-sweep-{arch}.exe"));
        if fs::metadata(exe)
            .and_then(|m| m.modified())
            .map_or(true, |t| t < newest)
        {
            missing.push(arch)
        }
    }
    if missing.is_empty() {
        return Ok(json!([true, ""]));
    }
    let tc = repo.join(".build-tools/llvm-mingw");
    let sibling = fs::read_dir(repo.parent().unwrap_or(repo))
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().starts_with("llvm-mingw-"));
    if !tc.exists() && !sibling {
        return Ok(json!([
            false,
            format!(
                "missing {} (pinned clang), needed to build {}",
                tc.display(),
                missing.join(", ")
            )
        ]));
    }
    for arch in missing {
        let cp = execute(
            host,
            repo,
            repo,
            json!(["bash", script, arch]),
            Value::Null,
            Value::Null,
        )?;
        if cp["returncode"] != 0 {
            let output = if s(&cp, "stderr", "").is_empty() {
                s(&cp, "stdout", "")
            } else {
                s(&cp, "stderr", "")
            };
            return Ok(json!([
                false,
                format!("build.sh {arch} failed: {}", last_words(output, 10000))
            ]));
        }
    }
    Ok(json!([true, ""]))
}
pub fn dispatch(repo: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = &r["kwargs"];
    Ok(match r["op"].as_str().unwrap_or("") {
        "gate_child_smoke" => smoke(repo, a, host)?,
        "gate_child_conformance" => conformance(repo, a, host)?,
        "gate_child_stortest" => stortest(repo, a, host)?,
        "gate_child_net" => net(repo, a, host)?,
        "gate_child_build" => build(repo, a, host)?,
        "gate_child_check" => check(repo, a, host)?,
        "gate_child_cohab" => cohab(repo, a, host)?,
        "gate_child_kmtest" => kmtest(repo, a, host)?,
        "gate_child_summary" => summary(Path::new(s(a, "root", ""))),
        "gate_child_salient" => json!(salient(&a["summary"])),
        "gate_child_last_words" => json!(last_words(
            s(a, "console", ""),
            a["limit"].as_u64().unwrap_or(240) as usize
        )),
        "gate_child_classification" => json!(classification(s(a, "stdout", ""))),
        op => return Err(Error::new(format!("unknown gate child operation {op}"))),
    })
}
