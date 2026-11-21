//! Build command sequencing. Resource adapters keep leases and external tool handles;
//! this module owns stage ordering, early failure, reconciliation and stamping policy.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
pub trait Host {
    fn call(&mut self, op: &str, a: Value) -> Result<Value>;
}
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v[k].as_str().unwrap_or(d)
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}
pub fn keep_going(args: &Value) -> Vec<String> {
    if args["keep_going"] == true {
        vec!["-k0".into()]
    } else {
        vec![]
    }
}
const PRIME: &[&str] = &["host-tools", "asm", "psdk", "xdk", "bugcodes", "errcodes"];
struct Build<'a> {
    host: &'a mut dyn Host,
    repo: &'a Path,
    dir: PathBuf,
    arch: String,
    args: Value,
    wrapped: bool,
    env: Value,
    /// `-j` cap and ninja timeout: host facts read once by the caller, which owns
    /// the policy dir and environment the tests point elsewhere.
    jobs: Vec<String>,
    /// The caller's environment, which the build mechanisms read their knobs from.
    host_env: Value,
}
impl Build<'_> {
    /// A build mechanism, run in this session against the same host: facts and
    /// external commands still cross it, the stages land on this build's run.
    fn mechanism(&mut self, op: &str, mut kw: Value) -> Result<Value> {
        kw["arch"] = json!(self.arch);
        kw["build"] = json!(self.dir);
        kw["args"] = self.args.clone();
        // An explicit env (cache stats takes the build's) is that call's alone.
        let own = kw.get("env").is_none();
        if own {
            kw["env"] = self.host_env.clone();
        }
        let (result, env) = crate::cli_build_helpers::dispatch_env(
            self.repo,
            &json!({"op":op,"kwargs":kw}),
            self.host,
        )?;
        if own {
            self.host_env = env;
        }
        Ok(result)
    }
    fn helper(&mut self, name: &str, a: Value) -> Result<Value> {
        self.host
            .call("build_helper", json!({"name":name,"arguments":a}))
    }
    fn stage(
        &mut self,
        name: &str,
        status: &str,
        detail: impl Into<String>,
        code: i64,
        salient: Value,
    ) -> Result<()> {
        self.host.call("stage",json!({"name":name,"status":status,"detail":detail.into(),"exit":code,"seconds":0.,"salient":salient}))?;
        Ok(())
    }
    fn classify(&mut self, cls: &str) -> Result<()> {
        self.host.call("set_run", json!({"classification":cls}))?;
        Ok(())
    }
    fn command(
        &mut self,
        cmd: Vec<String>,
        stage: &str,
        timeout: u64,
        canonical: bool,
    ) -> Result<Value> {
        let cmd = if canonical && self.wrapped {
            strings(&self.helper("wrap", json!({"cmd":cmd}))?)
        } else {
            cmd
        };
        self.host.call("command",json!({"argv":cmd,"cwd":if canonical{self.dir.clone()}else{self.repo.to_owned()},"env":self.env,"stage":stage,"timeout":timeout}))
    }
    fn fail_command(&mut self, name: &str, cp: &Value, detail: &str) -> Result<()> {
        let classified = self.helper(
            "classify",
            json!({"stdout":cp["stdout"],"stderr":cp["stderr"],"rc":cp["returncode"]}),
        )?;
        let cls = classified[0].as_str().unwrap_or("BUILD_FAIL");
        self.classify(cls)?;
        let salient = if classified[1].as_array().is_some_and(|s| !s.is_empty()) {
            classified[1].clone()
        } else {
            json!(["see full log"])
        };
        self.stage(
            name,
            "fail",
            if detail.is_empty() {
                cls.into()
            } else {
                format!("{detail} ({cls})")
            },
            cp["returncode"].as_i64().unwrap_or(1),
            salient,
        )
    }
    fn finish(&mut self) -> Result<Value> {
        self.host.call("finalize", json!({}))
    }
    fn ninja(
        &mut self,
        targets: Vec<String>,
        stage: &str,
        timeout: u64,
        keep: bool,
    ) -> Result<Value> {
        let mut cmd = vec!["ninja".into()];
        cmd.extend(self.jobs.iter().cloned());
        if keep {
            cmd.extend(keep_going(&self.args));
        }
        cmd.extend(targets);
        self.command(cmd, stage, timeout, true)
    }
}
pub fn locked(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = a["args"].clone();
    let arch = s(a, "arch", "").to_owned();
    crate::arch(&arch)?;
    let dir = a["build_dir"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join(format!("build-{arch}")));
    let mut b = Build {
        host,
        repo,
        dir,
        arch,
        args: args.clone(),
        wrapped: false,
        env: Value::Null,
        jobs: strings(&a["ninja_jobs"]),
        host_env: a["host_env"].clone(),
    };
    let cfg = &a["cfg"];
    let cold = a["cold"] == true;
    let headers = args["headers_only"] == true;
    let artifact_env = &a["artifact_env"];
    // The tree `git add -A` would commit; None when git cannot say (never fatal here).
    let start = json!(crate::warmth::worktree_tree(b.repo).ok());
    b.helper("failure_hook", json!({"tree":start}))?;
    let iso = crate::cli_build_checks::isolation_verdict(b.repo, &b.dir);
    if iso["ok"] != true {
        let st = &iso["stage"];
        let (name, status, detail) = (s(st, "name", ""), s(st, "status", ""), s(st, "detail", ""));
        b.stage(name, status, detail, 0, st["salient"].clone())?;
        b.classify(s(&iso, "classification", ""))?;
        return b.finish();
    }
    let flags = json!({"VIBEOS_LOCKDEP":args["lockdep"]==true,"VIBEOS_CSWITCHDEP":args["cswitchdep"]==true,"VIBEOS_UP_DEFAULT_BOOT":args["up_kernel"]==true,"VIBEOS_PIC_HAL":args["hal_pic"]==true,"VIBEOS_ATA_STACK":args["ata_stack"]==true,"ENABLE_ROSTESTS":args["rostests"]==true,"VIBEOS_WINETEST_SWEEP":args["winetest_sweep"]==true,"VIBEOS_WOW64_NO_STAGE":args["no_wow64_stage"]==true});
    let sticky = [
        ("ENABLE_ROSTESTS_KMTESTS", "kmtests"),
        ("ENABLE_ROSTESTS_APITESTS", "apitests"),
    ]
    .into_iter()
    .filter(|(_, k)| args[*k] == true)
    .map(|(k, _)| k)
    .collect::<Vec<_>>();
    let values = json!({"VIBEOS_WINETEST_SKIP":s(&args,"winetest_skip","").trim(),"VIBEOS_WINETEST_ONLY":s(&args,"winetest_only","").trim(),"ROSTESTS_WINETESTS":s(&args,"winetests","").split_whitespace().collect::<Vec<_>>().join(" ")});
    let wrapped = b.mechanism(
        "canonize",
        json!({"flags":flags,"sticky":sticky,"values":values}),
    )?;
    if wrapped.is_null() {
        return b.finish();
    }
    b.wrapped = wrapped == true;
    let mut stamped = json!([]);
    if !headers {
        let mut options = crate::build_policy::storage_options(
            &b.dir,
            &args["rust_storage_shards"],
            &args["rust_body_shards"],
        )?;
        let active = artifact_env.is_object();
        let state = b.host.call("get_run", json!({}))?;
        for (key, val) in [
            (
                "VIBEOS_ARTIFACT_STORE_MODE",
                if active { "ON".into() } else { "OFF".into() },
            ),
            (
                "VIBEOS_ARTIFACT_STORE_ROOT",
                if active {
                    s(&args, "artifact_store", "").into()
                } else {
                    String::new()
                },
            ),
            (
                "VIBEOS_ARTIFACT_VERIFY_NONCE",
                if active && args["artifact_store_verify"] == true {
                    s(&state, "dir", "").into()
                } else {
                    String::new()
                },
            ),
            (
                "VIBEOS_ARTIFACT_TOOLCHAIN_ID",
                s(artifact_env, "VIBEOS_ARTIFACT_TOOLCHAIN_ID", "").into(),
            ),
        ] {
            if crate::build_policy::cmake_value(&b.dir, key) != val {
                options.push(format!("-D{key}={val}"));
            }
        }
        if !options.is_empty() {
            let mut cmd = vec!["cmake".into()];
            cmd.extend(options);
            cmd.push(".".into());
            let cp = b.command(cmd, "storage-config", 600, true)?;
            if cp["returncode"] != 0 {
                b.classify("CMAKE_CONFIGURE_FAIL")?;
                b.stage(
                    "storage-config",
                    "fail",
                    "",
                    cp["returncode"].as_i64().unwrap_or(1),
                    json!([]),
                )?;
                return b.finish();
            }
        }
        stamped = b.mechanism("stage_payloads", json!({}))?;
        if stamped.is_null() {
            return b.finish();
        }
    }
    if cold {
        b.mechanism("clean", json!({}))?;
        if artifact_env.is_object() {
            b.helper(
                "retire_build_refs",
                json!({"supervisor":artifact_env["VIBEOS_ARTIFACT_SUPERVISOR"]}),
            )?;
        }
    }
    // The adapter returns a complete environment. It reuses the existing native
    // cache policy and broker; the broker remains the only admission authority.
    let setup = b.mechanism(
        "environment",
        json!({"wrapped":b.wrapped,"artifact_env":artifact_env}),
    )?;
    if setup.is_null() {
        return b.finish();
    }
    b.env = setup["env"].clone();
    let cache = setup["cache"].clone();
    let caching = setup["caching"] == true;
    let mut deferred = vec![];
    // arm64 under VIBEOS_FEX=1 stages the same guests, so it refreshes them too: it used to
    // stage whatever build-fex-hello/ held, and an arm64 `smoke --ladder` graded a fault32.exe
    // two days older than Fault32.c. Its x86-64 guests link build-amd64's importlibs, which
    // an arm64 ninja never makes, so a deferral there stays deferred (and the ladder then
    // refuses those guests as stale, which names the fix).
    let stages_guests = b.arch == "amd64"
        || (b.arch == "arm64" && std::env::var("VIBEOS_FEX").as_deref() == Ok("1"));
    if stages_guests && !headers && args["no_wow64_stage"] != true {
        let r = b.mechanism("refresh_guests", json!({"stage":"wow64-guests"}))?;
        if r == false {
            return b.finish();
        }
        if b.arch == "amd64" {
            deferred = strings(&r);
        }
    }
    if b.arch == "arm64" && !headers && b.mechanism("refresh_cpu", json!({}))? != true {
        return b.finish();
    }
    let cp = b.ninja(
        PRIME.iter().map(|s| s.to_string()).collect(),
        "ninja-prime",
        1800,
        false,
    )?;
    if cp["returncode"] != 0 {
        b.fail_command("ninja-prime", &cp, "")?;
        return b.finish();
    }
    b.stage("ninja-prime", "ok", "generated headers", 0, json!([]))?;
    if headers {
        b.host
            .call("set_run", json!({"extra":{"partial":"headers-only"}}))?;
        b.stage(
            "headers-only",
            "warn",
            "partial build; no boot media or stamp",
            0,
            json!([]),
        )?;
        return b.finish();
    }
    let cp = b.command(
        vec!["ninja".into(), "-t".into(), "cleandead".into()],
        "ninja-cleandead",
        300,
        true,
    )?;
    if cp["returncode"] != 0 {
        b.fail_command("cleandead", &cp, "ninja -t cleandead failed")?;
        return b.finish();
    }
    let re = regex::Regex::new(r"(\d+) files?\.")?;
    let n = re
        .captures(s(&cp, "stdout", ""))
        .map(|m| m[1].to_string())
        .unwrap_or("?".into());
    b.stage(
        "cleandead",
        "ok",
        format!("{n} dead output(s) removed"),
        0,
        json!([]),
    )?;
    let mut targets = strings(&cfg["ninja_targets"]);
    for t in strings(&args["extra_target"]) {
        if !targets.contains(&t) {
            targets.push(t)
        }
    }
    if targets.iter().any(|t| t.is_empty() || t.starts_with('-')) {
        b.stage(
            "targets",
            "fail",
            "invalid extra Ninja target",
            0,
            json!([]),
        )?;
        return b.finish();
    }
    let cohab = strings(&b.helper("cohab_rels", json!({}))?)
        .into_iter()
        .filter(|r| !targets.contains(r))
        .collect::<Vec<_>>();
    if !cohab.is_empty() {
        b.host.call("log",json!({"header":"cohab","text":format!("force-building the boot-manifest set (self-heal):\n{}",cohab.join("\n"))}))?;
    }
    let mut full_targets = targets.clone();
    full_targets.extend(cohab.clone());
    let timeout = a["ninja_timeout"].as_u64().ok_or("invalid ninja timeout")?;
    let cp = b.ninja(full_targets.clone(), "ninja", timeout, true)?;
    if cp["returncode"] != 0 {
        b.fail_command("ninja", &cp, "")?;
        return b.finish();
    }
    b.stage("ninja", "ok", targets.join("+"), 0, json!([]))?;
    if b.mechanism("verify_flags", json!({}))? != true {
        return b.finish();
    }
    for target in strings(&cfg["ninja_targets_optional"]) {
        let cp = b.ninja(
            vec![target.clone()],
            &format!("ninja-optional:{target}"),
            900,
            false,
        )?;
        let name = format!("opt:{target}");
        if cp["returncode"] == 0 {
            b.stage(&name, "ok", "", 0, json!([]))?;
            continue;
        }
        let blob = format!("{}{}", s(&cp, "stdout", ""), s(&cp, "stderr", ""));
        if blob.to_lowercase().contains("unknown target") {
            b.stage(&name, "skip", "absent from ninja graph", 0, json!([]))?;
            continue;
        }
        b.fail_command(
            &name,
            &cp,
            "optional target FAILED to build — not absent, broken",
        )?;
        return b.finish();
    }
    let cp = b.ninja(full_targets.clone(), "ninja-reaffirm", 1800, true)?;
    if cp["returncode"] != 0 {
        b.fail_command(
            "ninja-reaffirm",
            &cp,
            "relink after the optional passes failed",
        )?;
        return b.finish();
    }
    let relinked = crate::build_policy::ninja_work(s(&cp, "stdout", ""))? as u64;
    if !deferred.is_empty() {
        let r = b.mechanism("refresh_guests", json!({"stage":"wow64-guests-x64"}))?;
        if r != true {
            let state = b.host.call("get_run", json!({}))?;
            if s(&state, "classification", "").is_empty() {
                b.classify("GUEST_BUILD_FAIL")?;
                b.stage(
                    "wow64-guests-x64",
                    "fail",
                    format!(
                        "x86-64 guests still unbuildable after the first ninja: {}",
                        deferred.join(", ")
                    ),
                    0,
                    json!([]),
                )?;
            }
            return b.finish();
        }
        let cp = b.ninja(full_targets.clone(), "ninja-restage", 1800, true)?;
        if cp["returncode"] != 0 {
            b.fail_command(
                "ninja-restage",
                &cp,
                "restaging the deferred x86-64 guests failed",
            )?;
            return b.finish();
        }
        b.stage("ninja-restage", "ok", "x86-64 guests staged", 0, json!([]))?;
    }
    b.stage(
        "ninja-reaffirm",
        "ok",
        if relinked > 0 {
            format!("relinked {relinked} target(s) invalidated by the optional passes")
        } else {
            "no-op — optional passes invalidated nothing".into()
        },
        0,
        json!([]),
    )?;
    let mut roots = vec!["build.ninja".to_owned()];
    roots.extend(PRIME.iter().map(|s| s.to_string()));
    roots.extend(full_targets);
    roots.extend(strings(&cfg["ninja_targets_optional"]));
    roots.extend(strings(
        &b.helper("payload_roots", json!({"stamped":stamped}))?,
    ));
    let prune = b.helper(
        "prune",
        json!({"roots":roots,"optional":cfg["ninja_targets_optional"]}),
    )?;
    if prune["error"].is_string() {
        b.classify("BUILD_FAIL")?;
        b.stage(
            "closure-prune",
            "fail",
            format!(
                "cannot resolve root(s) in build.ninja: {}",
                s(&prune, "error", "")
            ),
            0,
            json!([]),
        )?;
        return b.finish();
    }
    let victims = strings(&prune["victims"]);
    if !victims.is_empty() {
        b.host.call(
            "log",
            json!({"header":"closure-prune","text":victims.join("\n")}),
        )?;
    }
    b.stage(
        "closure-prune",
        "ok",
        format!("{} output(s) outside the closure removed", victims.len()),
        0,
        json!([]),
    )?;
    if caching {
        let env = b.env.clone();
        b.mechanism(
            "cache_stats",
            json!({"cache":cache,"env":env,"wrapped":b.wrapped}),
        )?;
    }
    if cfg["needs_mkramdisk"] == true {
        let ramdisk = repo.join("qemu/ramdisk.img");
        let mut refresh = ramdisk.is_file() && !cold;
        let mut cmd = vec![
            "bash".into(),
            repo.join("qemu/mkramdisk.sh")
                .to_string_lossy()
                .into_owned(),
        ];
        if refresh {
            cmd.push("--refresh".into())
        }
        if args["up_kernel"] == true {
            cmd.push("--up".into())
        }
        // Deliberately use the ambient environment here, as the original staging path did.
        let env = b.env.clone();
        b.env = Value::Null;
        let mut cp = b.command(cmd.clone(), "mkramdisk", 900, false)?;
        if cp["returncode"] != 0
            && refresh
            && format!("{}{}", s(&cp, "stdout", ""), s(&cp, "stderr", "")).contains("Disk full")
        {
            b.stage(
                "mkramdisk-refresh",
                "warn",
                "refresh hit Disk full on a stale image; recreating it",
                cp["returncode"].as_i64().unwrap_or(1),
                json!([]),
            )?;
            for p in [ramdisk, repo.join("qemu/ramdisk.payloads")] {
                match std::fs::remove_file(p) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
            }
            refresh = false;
            cmd.retain(|s| s != "--refresh");
            cp = b.command(cmd, "mkramdisk", 900, false)?;
        }
        b.env = env;
        if cp["returncode"] != 0 {
            b.classify("RESTAGE_FAIL")?;
            let lines = s(&cp, "stderr", "")
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>();
            b.stage(
                "mkramdisk",
                "fail",
                "ramdisk re-stage",
                cp["returncode"].as_i64().unwrap_or(1),
                json!(lines),
            )?;
            return b.finish();
        }
        b.stage(
            "mkramdisk",
            "ok",
            if refresh { "--refresh" } else { "full" },
            0,
            json!([]),
        )?;
    }
    if args["no_check"] == true {
        b.stage(
            "binary-lints",
            "skip",
            "--no-check: the caller's own lint lane (gate/land) runs them",
            0,
            json!([]),
        )?;
    } else {
        b.helper("binary_lints", json!({"strict":args["strict"]==true}))?;
    }
    let index = repo.join("qemu/build-index.sh");
    if index.is_file() {
        let env = b.env.clone();
        b.env = Value::Null;
        let cp = b.command(
            vec![
                "bash".into(),
                index.to_string_lossy().into_owned(),
                b.arch.clone(),
            ],
            "build-index",
            300,
            false,
        )?;
        b.env = env;
        b.stage(
            "build-index",
            if cp["returncode"] == 0 { "ok" } else { "warn" },
            "",
            cp["returncode"].as_i64().unwrap_or(1),
            json!([]),
        )?;
    }
    if args["check"] == true {
        b.helper("source_lints", json!({}))?;
    }
    let moved = b.mechanism("inputs_moved", json!({"tree":start}))?;
    if moved.as_array().is_some_and(|s| !s.is_empty()) {
        b.classify("BUILD_INPUTS_MOVED")?;
        let paths = strings(&moved);
        b.stage("inputs-moved","fail",format!("{} build input(s) changed while this build ran, so it built neither tree; no stamp or warm receipt written. Rebuild.",paths.len()),0,json!(paths.into_iter().take(8).collect::<Vec<_>>()))?;
        return b.finish();
    }
    b.helper("stamp", json!({"cold":cold}))?;
    b.stage(
        "stamp",
        "ok",
        if cold { "cold" } else { "warm" },
        0,
        json!([]),
    )?;
    let st = b.helper("staleness", json!({}))?;
    if st["media_missing"] == true || st["stale"] == true {
        b.classify("BUILD_INCOMPLETE")?;
        b.stage(
            "artifact-check",
            "fail",
            "boot media missing or stale after a 'successful' build",
            0,
            json!(strings(&st["reasons"])
                .into_iter()
                .take(4)
                .collect::<Vec<_>>()),
        )?;
        return b.finish();
    }
    b.stage(
        "artifact-check",
        "ok",
        "media present + fresh",
        0,
        json!([]),
    )?;
    let receipt = match crate::warmth::record(b.repo, &b.arch, crate::trim::now()) {
        Ok(tree) => json!({"tree": tree}),
        Err(error) => json!({"error": error.to_string()}),
    };
    if let Some(tree) = receipt["tree"].as_str() {
        b.stage(
            "warm-receipt",
            "ok",
            format!("tree {}", tree.chars().take(12).collect::<String>()),
            0,
            json!([]),
        )?;
    } else {
        b.stage(
            "warm-receipt",
            "warn",
            format!("not recorded: {}", s(&receipt, "error", "unknown error")),
            0,
            json!([]),
        )?;
    }
    let dedup = b.mechanism("dedup", json!({}))?;
    b.host.call("stage", dedup)?;
    if !cold {
        b.stage(
            "warm-build",
            "warn",
            "cargo incremental may mask; --clean to verify",
            0,
            json!([]),
        )?;
    }
    b.finish()
}
pub fn dispatch(repo: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    match r["op"].as_str().unwrap_or("") {
        "cli_build_locked" => locked(repo, &r["kwargs"], host),
        op => Err(Error::new(format!(
            "unknown build coordinator operation {op}"
        ))),
    }
}
