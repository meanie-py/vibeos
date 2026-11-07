//! Tooling check planning and parallel execution, with content receipts and full evidence.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    time::Instant,
};
use vibeos_build::{cli_artifacts as artifacts, tooling_receipts as receipts};
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v[k].as_str().unwrap_or(d)
}
fn policy() -> Value {
    serde_json::from_str(include_str!("tooling_checks.json")).unwrap()
}
pub fn plan(repo: &Path, a: &Value) -> Result<Value> {
    let p = policy();
    let python = s(a, "python", "python3");
    let bin = Path::new(python).parent().unwrap_or(Path::new("."));
    let ruff = bin.join("ruff").to_string_lossy().into_owned();
    let pyright = bin.join("pyright").to_string_lossy().into_owned();
    let mut checks = vec![];
    let tools = {
        let mut v = std::fs::read_dir(repo.join("tools"))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|s| s == "py"))
            .map(|p| p.strip_prefix(repo).unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        v.sort();
        v
    };
    for (prefix, flags) in [("ruff", vec!["check"]), ("fmt", vec!["format", "--check"])] {
        for root in p["_RUFF_ROOTS"].as_array().unwrap() {
            let root = root.as_str().unwrap();
            let mut cmd = vec![ruff.clone()];
            cmd.extend(flags.iter().map(|s| s.to_string()));
            cmd.push(root.into());
            checks.push(json!({"label":format!("{prefix}:{root}"),"cmd":cmd}));
        }
        let mut cmd = vec![ruff.clone()];
        cmd.extend(flags.iter().map(|s| s.to_string()));
        cmd.extend(tools.clone());
        checks.push(json!({"label":format!("{prefix}:tools/*.py"),"cmd":cmd}));
    }
    if a["no_types"] != true {
        for root in p["_PYRIGHT_ROOTS"].as_array().unwrap() {
            checks.push(json!({"label":format!("pyright:{}",root.as_str().unwrap()),"cmd":[pyright,"--pythonpath",python,root]}));
        }
    }
    checks.push(json!({"label":"generated:ddk_enum_constants","cmd":[python,"tools/rustconv/gen_enum_constants.py","--check"]}));
    if a["no_tests"] != true {
        for name in ["bench", "test", "build", "codegen", "lint"] {
            let dir = format!("tools/vibeos-{name}/Cargo.toml");
            let target = match name {
                "test" | "build" => ".cache/vibeos-build/target".to_string(),
                "lint" => ".cache/vibeos-lint/test-target".to_string(),
                _ => format!(".cache/vibeos-{name}/target"),
            };
            let mut cmd = vec![
                "cargo".into(),
                "test".into(),
                "--offline".into(),
                "--locked".into(),
            ];
            if matches!(name, "bench" | "test" | "build") {
                cmd.extend(["--jobs".into(), "1".into()]);
            }
            cmd.extend(["--manifest-path".into(), dir, "--target-dir".into(), target]);
            if matches!(name, "bench" | "test" | "build") {
                cmd.extend(["--".into(), "--test-threads=1".into()]);
            }
            checks.push(json!({"label":format!("rust:vibeos-{name}"),"cmd":cmd}));
        }
        for suite in p["_PYTEST_SUITES"].as_array().unwrap() {
            let pkg = suite[0].as_str().unwrap();
            let pp = suite[1].as_str().unwrap();
            let target = suite[2].as_str().unwrap();
            let label = if pkg == "." {
                target.to_owned()
            } else {
                format!("{pkg}/{target}")
            };
            let mut cmd = vec![
                python.into(),
                "-m".into(),
                "pytest".into(),
                target.into(),
                "-q".into(),
            ];
            if target == "tools/test_build_invalidation.py"
                || (pkg == "tools/rustconv" && target == "tests")
            {
                cmd.extend([
                    "-n".into(),
                    a["jobs"].as_u64().unwrap_or(1).min(8).to_string(),
                ]);
            }
            if target == "tests/qemu/cli" {
                cmd.push("--ignore=tests/qemu/cli/test_portlease.py".into());
            }
            checks.push(json!({"label":format!("pytest:{label}"),"cmd":cmd,"cwd":repo.join(pkg),"env_pp":pp}));
        }
    }
    let mut skipped = vec![];
    if a["skip_generated"] == true {
        checks.retain(|c| {
            let skip = receipts::GENERATED_INPUT_CHECKS.contains(&s(c, "label", ""));
            if skip {
                skipped.push(c["label"].clone());
            }
            !skip
        });
    }
    Ok(json!({"checks":checks,"skipped":skipped}))
}
pub fn check(repo: &Path, a: &Value) -> Result<Value> {
    let label = s(a, "label", "");
    let mut cmd = a["cmd"].clone();
    let temp = if label.starts_with("pytest:") {
        let t = tempfile::Builder::new()
            .prefix("pytest-tooling-")
            .tempdir()?;
        cmd.as_array_mut()
            .ok_or("invalid tooling argv")?
            .extend([json!("--basetemp"), json!(t.path().join("run"))]);
        Some(t)
    } else {
        None
    };
    let mut args = a.clone();
    args["argv"] = cmd;
    args["repo"] = json!(repo);
    args["cwd"] = a.get("cwd").cloned().unwrap_or(json!(repo));
    args["stage"] = json!(label);
    if let Some(pp) = a["env_pp"].as_str() {
        let mut env: BTreeMap<String, String> = serde_json::from_value(
            a.get("environment")
                .cloned()
                .unwrap_or_else(|| json!(std::env::vars().collect::<BTreeMap<_, _>>())),
        )?;
        env.insert(
            "PYTHONPATH".into(),
            repo.join(pp).to_string_lossy().into_owned(),
        );
        args["env"] = json!(env);
    }
    let mut out = artifacts::command(&args)?;
    let output = format!("{}{}", s(&out, "stdout", ""), s(&out, "stderr", ""))
        .trim()
        .to_owned();
    out["result"] = json!([label, out["returncode"] == 0, output]);
    drop(temp);
    Ok(out)
}
fn log(dir: &Path, header: &str, text: &str) -> Result<()> {
    artifacts::dispatch(
        dir,
        &json!({"op":"cli_artifacts_log","kwargs":{"dir":dir,"header":header,"text":text}}),
    )?;
    Ok(())
}
pub fn run(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let dir = PathBuf::from(s(a, "dir", ""));
    let plan = if a["checks"].is_array() {
        json!({"checks":a["checks"],"skipped":a.get("skipped").cloned().unwrap_or(json!([]))})
    } else {
        plan(repo, a)?
    };
    let checks = plan["checks"].as_array().unwrap();
    // Provision once, before the parallel checks, outside individual test budgets.
    if checks
        .iter()
        .any(|c| c["label"] == "pytest:tools/test_lint_facts.py")
    {
        host.call("tooling_provision_lint", json!({"repo":repo}))?;
    }
    let cache = a["cache"] != false;
    let before = if cache {
        json!([
            receipts::source(repo)?,
            host.call("tooling_runtime", json!({"repo":repo}))?
        ])
    } else {
        Value::Null
    };
    let session = Mutex::new(if cache {
        Some(receipts::Session::new(repo, before)?)
    } else {
        None
    });
    let next = AtomicUsize::new(0);
    let outputs = Mutex::new(vec![]);
    std::thread::scope(|scope| {
        for _ in 0..a["jobs"]
            .as_u64()
            .unwrap_or(1)
            .max(1)
            .min(checks.len().max(1) as u64)
        {
            let outputs = &outputs;
            let next = &next;
            let session = &session;
            let dir = &dir;
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= checks.len() {
                    break;
                }
                let r = execute(repo, a, dir, &checks[i], session, false);
                outputs.lock().unwrap().push((i, r));
            });
        }
    });
    let mut outputs = outputs.into_inner().unwrap();
    outputs.sort_by_key(|(i, _)| *i);
    let mut rows = outputs
        .into_iter()
        .map(|(_, r)| r)
        .collect::<Result<Vec<_>>>()?;
    let mut receipt_error = None;
    if cache {
        let stale = session.lock().unwrap().as_mut().unwrap().stale()?;
        for label in stale.as_array().unwrap() {
            if let Some(i) = checks.iter().position(|c| c["label"] == *label) {
                rows[i] = execute(repo, a, &dir, &checks[i], &session, true)?;
                session
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .mark_rerun(label.as_str().unwrap());
            }
        }
        let after = json!([
            receipts::source(repo)?,
            host.call("tooling_runtime", json!({"repo":repo}))?
        ]);
        if let Err(e) = session.lock().unwrap().as_mut().unwrap().finish(&after) {
            receipt_error = Some(e.to_string())
        }
    }
    let hits = session
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.hits().clone())
        .unwrap_or_default();
    let mut stages = vec![];
    let mut timings = vec![];
    let mut lines = String::new();
    let mut failed = vec![];
    let width = rows
        .iter()
        .map(|r| s(&r["stage"], "name", "").len())
        .max()
        .unwrap_or(1);
    for r in &rows {
        let st = &r["stage"];
        let label = s(st, "name", "");
        let ok = st["status"] == "ok";
        lines.push_str(&format!(
            "  {} {label:width$}\n",
            if ok { "✓" } else { "✗" }
        ));
        if !ok {
            failed.push((
                label.to_owned(),
                r["output"].as_str().unwrap_or("").to_owned(),
            ));
        }
        stages.push(st.clone());
        if let Some(t) = r.get("timing") {
            timings.push(t.clone())
        }
    }
    if let Some(error) = receipt_error {
        log(&dir, "receipt-validation", &error)?;
        failed.push(("receipt-validation".into(), error.clone()));
        stages.push(json!({"name":"receipt-validation","exit":1,"seconds":0.,"status":"fail","detail":error,"salient":[]}));
    }
    if !hits.is_empty() {
        lines.push_str(&format!(
            "Reused {}/{} tooling checks with matching inputs.\n",
            hits.len(),
            checks.len()
        ));
    }
    if failed.is_empty() {
        lines.push_str(&format!(
            "\nOK — all {} tooling checks passed.\n",
            checks.len()
        ));
    } else {
        lines.push_str(&format!(
            "\nFAIL — {}/{} checks failed:\n",
            failed.len(),
            checks.len()
        ));
        for (label, output) in &failed {
            lines.push_str(&format!(
                "\n=== {label} ===\n{}\n",
                output
                    .lines()
                    .rev()
                    .take(25)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
    Ok(
        json!({"code":if failed.is_empty(){0}else{1},"stages":stages,"command_timings":timings,"checks":{"total":rows.len(),"passed":rows.iter().filter(|r|r["stage"]["status"]=="ok").count(),"failed":rows.iter().filter(|r|r["stage"]["status"]!="ok").count(),"skipped":plan["skipped"],"reused":hits},"output":lines}),
    )
}
fn execute(
    repo: &Path,
    a: &Value,
    dir: &Path,
    c: &Value,
    session: &Mutex<Option<receipts::Session>>,
    revalidate: bool,
) -> Result<Value> {
    let started = Instant::now();
    let label = s(c, "label", "");
    let lookup = if !revalidate {
        let mut guard = session.lock().unwrap();
        guard
            .as_mut()
            .map(|s| s.lookup(c))
            .transpose()?
            .unwrap_or(json!({"eligible":false}))
    } else {
        json!({"eligible":false})
    };
    let reused = lookup["result"].is_array();
    let out = if reused {
        log(
            dir,
            &format!("{label}: reused"),
            lookup["result"][2].as_str().unwrap_or(""),
        )?;
        json!({"result":lookup["result"]})
    } else {
        let mut args = c.clone();
        args["dir"] = json!(dir);
        args["run_start"] = a["run_start"].clone();
        args["environment"] = a["environment"].clone();
        let out = check(repo, &args)?;
        if lookup["eligible"] == true {
            session
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .put(&json!({"key":lookup["key"],"result":out["result"]}))?;
        }
        out
    };
    let ok = out["result"][1] == true;
    let output = out["result"][2].as_str().unwrap_or("");
    let salient = if ok {
        vec![]
    } else {
        output
            .lines()
            .rev()
            .take(25)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    };
    let mut value = json!({"stage":{"name":if revalidate{format!("{label}:revalidation")}else{label.into()},"exit":out["returncode"].as_i64().unwrap_or(if ok{0}else{1}),"seconds":started.elapsed().as_secs_f64(),"status":if ok{"ok"}else{"fail"},"detail":if revalidate{"revalidated"}else if reused{"reused"}else{"executed"},"salient":salient},"output":output});
    if !out["timing"].is_null() {
        value["timing"] = out["timing"].clone()
    }
    Ok(value)
}
pub fn dispatch(repo: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    match r["op"].as_str().unwrap_or("") {
        "tooling_policy" => Ok(policy()),
        "tooling_plan" => plan(repo, &r["kwargs"]),
        "tooling_check" => check(repo, &r["kwargs"]),
        "tooling_run" => run(repo, &r["kwargs"], host),
        op => Err(Error::new(format!("unknown tooling operation {op}"))),
    }
}
