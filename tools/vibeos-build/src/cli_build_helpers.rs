//! Build preparation and finishing mechanisms. External handles and facts cross
//! the host interface; all reconciliation and acceptance decisions remain native.
use crate::{build_policy, cli_build::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
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
struct Context<'a> {
    host: &'a mut dyn Host,
    repo: &'a Path,
    build: PathBuf,
    env: Value,
}
impl Context<'_> {
    fn fact(&mut self, kind: &str, a: Value) -> Result<Value> {
        self.host
            .call("build_fact", json!({"kind":kind,"arguments":a}))
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
    fn classify(&mut self, class: &str) -> Result<()> {
        self.host.call("set_run", json!({"classification":class}))?;
        Ok(())
    }
    fn command(
        &mut self,
        argv: Vec<String>,
        stage: &str,
        timeout: u64,
        cwd: &Path,
        env: Value,
        wrapped: bool,
    ) -> Result<Value> {
        let argv = if wrapped {
            strings(&self.fact("wrap", json!({"cmd":argv}))?)
        } else {
            argv
        };
        self.host.call(
            "command",
            json!({"argv":argv,"stage":stage,"timeout":timeout,"cwd":cwd,"env":env}),
        )
    }
    fn cmake(&mut self, argv: Vec<String>, stage: &str, timeout: u64) -> Result<Value> {
        self.command(argv, stage, timeout, &self.build.clone(), Value::Null, true)
    }
}
fn errors(cp: &Value) -> Vec<String> {
    format!("{}{}", s(cp, "stdout", ""), s(cp, "stderr", ""))
        .lines()
        .filter(|l| l.contains("Error"))
        .map(|s| s.trim().to_string())
        .take(3)
        .collect()
}
fn canonize(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let flags = a["flags"].as_object().ok_or("missing canonical flags")?;
    let values = a.get("values").and_then(Value::as_object);
    let sticky = strings(&a["sticky"]);
    if c.fact("canonical", json!({}))? == true {
        let mut stale = BTreeMap::new();
        for (k, on) in flags {
            let cached = c.fact("cache_value", json!({"key":k}))?;
            let cached = matches!(cached.as_str(), Some("ON" | "1" | "TRUE"));
            if (*on == true) != cached {
                stale.insert(k.clone(), *on == true);
            }
        }
        for key in sticky {
            let cached = c.fact("cache_value", json!({"key":key}))?;
            if !matches!(cached.as_str(), Some("ON" | "1" | "TRUE")) {
                stale.insert(key, true);
            }
        }
        let mut defs = stale
            .into_iter()
            .map(|(k, on)| format!("-D{k}={}", if on { "ON" } else { "OFF" }))
            .collect::<Vec<_>>();
        if let Some(values) = values {
            for (k, v) in values {
                if c.fact("cache_value", json!({"key":k}))? != *v {
                    defs.push(format!(
                        "-D{k}={}",
                        v.as_str().ok_or("invalid string instrumentation")?
                    ));
                }
            }
        }
        if !defs.is_empty() || !c.build.join("build.ninja").is_file() {
            let mut cmd = vec!["cmake".into()];
            cmd.extend(defs.clone());
            cmd.push(".".into());
            let cp = c.cmake(cmd, "instr-config", 600)?;
            if cp["returncode"] != 0 {
                c.classify("CMAKE_CONFIGURE_FAIL")?;
                c.stage(
                    "instr-config",
                    "fail",
                    format!("cmake {} reconfigure failed", defs.join(" ")),
                    cp["returncode"].as_i64().unwrap_or(1),
                    json!(errors(&cp)),
                )?;
                return Ok(Value::Null);
            }
            if !c.build.join("build.ninja").is_file() {
                c.classify("CMAKE_CONFIGURE_FAIL")?;
                c.stage(
                    "instr-config",
                    "fail",
                    "cmake produced no build.ninja",
                    0,
                    json!([]),
                )?;
                return Ok(Value::Null);
            }
            c.stage(
                "instr-config",
                "ok",
                if defs.is_empty() {
                    "recovered incomplete configure".into()
                } else {
                    defs.join(" ")
                },
                0,
                json!([]),
            )?;
        }
        return Ok(json!(true));
    }
    if c.fact("canon_available", json!({}))? != true {
        c.stage(
            "canon",
            "warn",
            "userns unavailable — legacy real-path build (no cross-worktree cache)",
            0,
            json!([]),
        )?;
        return Ok(json!(false));
    }
    if c.fact("toolchain", json!({}))? != true {
        c.classify("CMAKE_CONFIGURE_FAIL")?;
        c.stage(
            "canonize",
            "fail",
            "no llvm-mingw toolchain found",
            0,
            json!([".build-tools/llvm-mingw missing and no sibling llvm-mingw-* dir"]),
        )?;
        return Ok(Value::Null);
    }
    let mut carried = strings(&c.fact("carried", json!({}))?);
    for (k, on) in flags {
        if *on == true {
            carried.push(format!("-D{k}=ON"))
        }
    }
    if let Some(values) = values {
        for (k, v) in values {
            if let Some(value) = v.as_str().filter(|s| !s.is_empty()) {
                carried.push(format!("-D{k}={value}"))
            }
        }
    }
    for key in sticky {
        let def = format!("-D{key}=ON");
        if !carried.contains(&def) {
            carried.push(def)
        }
    }
    let fresh = !c.build.join("CMakeCache.txt").is_file();
    if c.build.exists() {
        let root = c.repo.canonicalize()?;
        let build = c.build.canonicalize()?;
        if build == root || !build.starts_with(root) {
            return Err(
                "canonical migration refuses a build directory outside its checkout".into(),
            );
        }
        fs::remove_dir_all(&c.build)?;
    }
    fs::create_dir_all(&c.build)?;
    let cmd = strings(&c.fact("configure_cmd", json!({"extra":carried}))?);
    let cp = c.cmake(cmd, "canonize", 1800)?;
    if cp["returncode"] != 0 {
        c.classify("CMAKE_CONFIGURE_FAIL")?;
        c.stage(
            "canonize",
            "fail",
            "fresh canonical configure failed",
            cp["returncode"].as_i64().unwrap_or(1),
            json!(errors(&cp)),
        )?;
        return Ok(Value::Null);
    }
    let path = c.fact("canon_path", json!({}))?;
    c.stage(
        "canonize",
        "ok",
        format!(
            "{} @ {}{}",
            if fresh {
                "fresh configure"
            } else {
                "migrated legacy dir"
            },
            path.as_str().unwrap_or("/tmp/vibeos"),
            if fresh {
                ""
            } else {
                " (one-time full rebuild)"
            }
        ),
        0,
        json!([]),
    )?;
    Ok(json!(true))
}
fn setup_cache(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let cache: build_policy::CacheEnv = serde_json::from_value(a["cache"].clone())?;
    fs::create_dir_all(&cache.ccache_dir)?;
    fs::create_dir_all(&cache.sccache_dir)?;
    let mut env = c
        .env
        .as_object()
        .ok_or("missing cache environment")?
        .clone();
    env.extend(
        serde_json::to_value(build_policy::cache_environment(&cache))?
            .as_object()
            .unwrap()
            .clone(),
    );
    if cache.ccache_bin.is_some()
        && !matches!(
            c.fact("cache_value", json!({"key":"ENABLE_CCACHE"}))?
                .as_str(),
            Some("ON" | "1" | "TRUE")
        )
    {
        let cp = c.command(
            vec!["cmake".into(), "-DENABLE_CCACHE=1".into(), ".".into()],
            "ccache-config",
            600,
            &c.build.clone(),
            Value::Null,
            a["wrapped"] == true,
        )?;
        if cp["returncode"] != 0 {
            c.classify("CMAKE_CONFIGURE_FAIL")?;
            let salient = s(&cp, "stderr", "")
                .lines()
                .filter(|l| l.contains("Error"))
                .map(str::trim)
                .take(3)
                .collect::<Vec<_>>();
            c.stage(
                "ccache-config",
                "fail",
                "reconfigure for ENABLE_CCACHE failed",
                cp["returncode"].as_i64().unwrap_or(1),
                json!(salient),
            )?;
            return Ok(Value::Null);
        }
        c.stage("ccache-config", "ok", "ENABLE_CCACHE=1", 0, json!([]))?;
    }
    let summary = format!(
        "root={}  {}  {}",
        cache.root.display(),
        if cache.ccache_bin.is_some() {
            format!("ccache→{}", cache.ccache_dir.display())
        } else {
            "ccache(absent)".into()
        },
        if cache.sccache_bin.is_some() {
            format!("sccache→{}", cache.sccache_dir.display())
        } else {
            "sccache(absent)".into()
        }
    );
    c.stage("cache", "ok", summary, 0, json!([]))?;
    Ok(json!(env))
}
fn cache_stats(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let cache: build_policy::CacheEnv = serde_json::from_value(a["cache"].clone())?;
    c.host.call("set_run",json!({"extra":{"cache_stats_scope":"shared cumulative snapshots; not per-build attribution"}}))?;
    let state = c.host.call("get_run", json!({}))?;
    let dir = PathBuf::from(s(&state, "dir", "."));
    for (name, cmd, cwd, wrapped) in [
        (
            "ccache",
            cache.ccache_bin.as_ref().map(|bin| {
                vec![
                    if bin.is_empty() {
                        "ccache".into()
                    } else {
                        bin.clone()
                    },
                    "--print-stats".into(),
                ]
            }),
            dir.clone(),
            false,
        ),
        (
            "sccache",
            cache.sccache_bin.as_ref().map(|bin| {
                vec![
                    if bin.is_empty() {
                        "sccache".into()
                    } else {
                        bin.clone()
                    },
                    "--show-stats".into(),
                    "--stats-format=json".into(),
                ]
            }),
            if a["wrapped"] == true {
                c.repo.into()
            } else {
                dir.clone()
            },
            a["wrapped"] == true,
        ),
    ] {
        let Some(cmd) = cmd else { continue };
        let cp = c.command(
            cmd,
            &format!("{name}-stats"),
            60,
            &cwd,
            a["env"].clone(),
            wrapped,
        )?;
        let stats = build_policy::dispatch(
            &json!({"op":if name=="ccache"{"build_policy_ccache_stats"}else{"build_policy_sccache_stats"},"kwargs":{"text":cp["stdout"]}}),
        )?;
        let mut extra = json!({});
        extra[name] = stats.clone();
        c.host.call("set_run", json!({"extra":extra}))?;
        if name == "ccache" || stats.as_object().is_some_and(|s| !s.is_empty()) {
            c.stage(
                &format!("{name}-stats"),
                "ok",
                format!(
                    "{}h/{}m  {:.0}% hit",
                    stats["hits"],
                    stats["misses"],
                    stats["hit_rate"].as_f64().unwrap_or(0.) * 100.
                ),
                0,
                json!([]),
            )?;
        }
    }
    Ok(Value::Null)
}
fn refresh_guests(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let stage = s(a, "stage", "wow64-guests");
    let todo = strings(&c.fact("guests_to_refresh", json!({}))?);
    if todo.is_empty() {
        c.stage(stage, "ok", "current", 0, json!([]))?;
        return Ok(json!(true));
    }
    let blocker = c.fact("refresh_blocker", json!({}))?;
    if let Some(blocker) = blocker.as_str().filter(|s| !s.is_empty()) {
        c.stage(
            stage,
            "skip",
            format!("{} guest(s) need building but {blocker}", todo.len()),
            0,
            json!([]),
        )?;
        return Ok(json!(true));
    }
    let deferred = strings(&c.fact("guests_deferred", json!({"todo":todo}))?);
    let builder = c.fact("guest_builder", json!({}))?;
    let script = builder.as_str().ok_or("missing guest builder")?;
    let cp = c.command(
        vec![
            "bash".into(),
            c.repo.join(script).to_string_lossy().into_owned(),
        ],
        stage,
        900,
        c.repo,
        Value::Null,
        false,
    )?;
    let left = strings(&c.fact("guests_to_refresh", json!({}))?)
        .into_iter()
        .filter(|g| !deferred.contains(g))
        .collect::<Vec<_>>();
    if cp["returncode"] != 0 || !left.is_empty() {
        c.classify("GUEST_BUILD_FAIL")?;
        let mut salient = s(&cp, "stdout", "")
            .lines()
            .filter(|s| s.starts_with("[!]"))
            .rev()
            .take(8)
            .collect::<Vec<_>>();
        salient.reverse();
        if salient.is_empty() {
            salient.push("see full log")
        }
        c.stage(
            stage,
            "fail",
            format!(
                "{script} left: {}",
                if left.is_empty() {
                    "nothing (script failed)".into()
                } else {
                    left.join(", ")
                }
            ),
            cp["returncode"].as_i64().unwrap_or(1),
            json!(salient),
        )?;
        return Ok(json!(false));
    }
    let built = todo
        .iter()
        .filter(|g| !deferred.contains(g))
        .cloned()
        .collect::<Vec<_>>();
    let mut detail = if built.is_empty() {
        "nothing buildable yet".into()
    } else {
        format!("built: {}", built.join(", "))
    };
    if !deferred.is_empty() {
        detail.push_str(&format!(
            "; deferred until build-amd64 has importlibs: {}",
            deferred.join(", ")
        ));
        c.stage(stage, "ok", detail, 0, json!([]))?;
        Ok(json!(deferred))
    } else {
        c.stage(stage, "ok", detail, 0, json!([]))?;
        Ok(json!(true))
    }
}
pub const FEX_OUT_OF_TREE: &[(&str, &str, &[&str], &str)] = &[
    ("fex", "build-fex-arm64", &[], "tools/fex/build-fex.sh"),
    (
        "wowx64-cpu",
        "build-fex-hello/wowx64fex",
        &[],
        "tools/fex/build-hello-compute.sh",
    ),
    (
        "fex-probes",
        "build-fex-hello",
        &["fexprobe", "wow64probe"],
        "tools/fex/build-hello-compute.sh",
    ),
];
pub const CPU_SOURCES: &[&str] = &[
    "tools/fex/wowx64fex",
    "sdk/include/reactos/wowx64/btcpu64.h",
];
pub const CPU_DLL: &str = "build-fex-hello/wowx64fex/libwowx64fex.dll";
fn newer(repo: &Path) -> Vec<String> {
    let dll = repo.join(CPU_DLL);
    let Ok(built) = fs::metadata(dll).and_then(|m| m.modified()) else {
        return vec![];
    };
    let mut newer = vec![];
    let mut stack = CPU_SOURCES.iter().map(|s| repo.join(s)).collect::<Vec<_>>();
    while let Some(p) = stack.pop() {
        if fs::symlink_metadata(&p).is_ok_and(|m| m.is_dir()) {
            if let Ok(entries) = fs::read_dir(p) {
                stack.extend(entries.flatten().map(|e| e.path()))
            }
        } else if p.is_file()
            && fs::metadata(&p)
                .and_then(|m| m.modified())
                .is_ok_and(|t| t > built)
        {
            newer.push(p.strip_prefix(repo).unwrap().to_string_lossy().into_owned())
        }
    }
    newer.sort();
    newer
}
fn prerequisites(repo: &Path, env: &Value) -> Vec<String> {
    let mut missing = vec![];
    for (_, tree, _, script) in FEX_OUT_OF_TREE {
        if !repo.join(tree).join("build.ninja").is_file() {
            missing.push(format!("{tree} not configured: run {script}"))
        }
    }
    let ntdll = env["VIBEOS_NTDLL32"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join("build-i386/dll/ntdll/ntdll.dll"));
    if !ntdll.is_file() {
        missing.push(format!(
            "{} absent (the i386 guests' SysWOW64 set): ./vibeos-cli build --arch i386",
            ntdll.display()
        ))
    }
    let gui = env["VIBEOS_TESTGUI64_EXE"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join("build-amd64/base/applications/testgui/testgui.exe"));
    if !gui.is_file() {
        missing.push(format!("{} absent (anchors the x86-64 guests' sysamd64 set): ./vibeos-cli build --arch amd64 (after i386)",gui.display()))
    }
    missing
}
fn refresh_cpu(c: &mut Context<'_>) -> Result<Value> {
    let mut env = c.env.clone();
    env["PATH"] = json!(format!(
        "{}:{}",
        c.repo.join(".build-tools/llvm-mingw/bin").display(),
        s(&env, "PATH", "")
    ));
    for (stage, tree, targets, script) in FEX_OUT_OF_TREE {
        if !c.repo.join(tree).join("build.ninja").is_file() {
            c.stage(
                stage,
                "skip",
                format!("{tree} not configured (run: {script})"),
                0,
                json!([]),
            )?;
            continue;
        }
        let mut cmd = vec![
            "ninja".into(),
            "-C".into(),
            c.repo.join(tree).to_string_lossy().into_owned(),
        ];
        cmd.extend(targets.iter().map(|s| s.to_string()));
        let cp = c.command(cmd, stage, 1800, c.repo, env.clone(), false)?;
        if cp["returncode"] != 0 {
            c.classify("CPU_LAYER_BUILD_FAIL")?;
            c.stage(
                stage,
                "fail",
                format!("ninja in {tree} failed"),
                cp["returncode"].as_i64().unwrap_or(1),
                json!([format!("run: {script}")]),
            )?;
            return Ok(json!(false));
        }
        c.stage(stage, "ok", "current", 0, json!([]))?;
    }
    let left = newer(c.repo);
    if !left.is_empty() {
        c.classify("CPU_LAYER_BUILD_FAIL")?;
        let mut salient = left.into_iter().take(4).collect::<Vec<_>>();
        salient.push("run: tools/fex/build-hello-compute.sh".into());
        c.stage(
            "wowx64-cpu",
            "fail",
            format!("{CPU_DLL} older than its sources after ninja"),
            0,
            json!(salient),
        )?;
        return Ok(json!(false));
    }
    Ok(json!(true))
}
fn verify_flags(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let nm = c.repo.join(".build-tools/llvm-mingw/bin/llvm-nm");
    let artifact = c.build.join("ntoskrnl/ntoskrnl.exe");
    if !nm.is_file() || !artifact.is_file() {
        return Ok(json!(true));
    }
    let mut ok = true;
    for (flag, wanted) in [
        (
            "VIBEOS_LOCKDEP",
            a["args"]["lockdep"] == true || a["args"]["cswitchdep"] == true,
        ),
        ("VIBEOS_CSWITCHDEP", a["args"]["cswitchdep"] == true),
    ] {
        let cp = c.host.call(
            "build_external",
            json!({"argv":[nm,artifact],"timeout":120}),
        )?;
        if cp["error"].is_string() {
            c.stage(
                "flag-verify",
                "warn",
                "llvm-nm failed for ntoskrnl/ntoskrnl.exe",
                0,
                json!([]),
            )?;
            continue;
        }
        let present = s(&cp, "stdout", "").lines().any(|line| {
            line.ends_with(&format!(" {flag}")) || line.ends_with(&format!(" _{flag}"))
        });
        if present == wanted {
            c.stage(
                "flag-verify",
                "ok",
                format!(
                    "{flag}={} ↔ {flag} {} in ntoskrnl.exe",
                    if wanted { "ON" } else { "OFF" },
                    if present { "present" } else { "absent" }
                ),
                0,
                json!([]),
            )?;
        } else {
            ok = false;
            let state = c.host.call("get_run", json!({}))?;
            if s(&state, "classification", "").is_empty() {
                c.classify("FLAG_VERIFY_FAIL")?;
            }
            c.stage("flag-verify","fail",if wanted{format!("{flag} requested but {flag} ABSENT from ntoskrnl/ntoskrnl.exe — the flag is silently dead (plumbing broke somewhere between cmake and cargo)")}else{format!("{flag} OFF but {flag} PRESENT in ntoskrnl/ntoskrnl.exe — instrumentation leaked into a production build")},0,json!([format!("artifact: {}",artifact.display())]))?;
        }
    }
    Ok(json!(ok))
}
fn clean(c: &mut Context<'_>) -> Result<Value> {
    let mut targets = strings(&c.fact("rust_targets", json!({}))?);
    targets.extend(strings(&c.fact("workspace_targets", json!({}))?));
    let archives = strings(&c.fact("rust_archives", json!({}))?);
    let outputs = strings(&c.fact("module_outputs", json!({}))?);
    for p in &targets {
        if Path::new(p).exists() {
            fs::remove_dir_all(p)?;
        }
    }
    for p in archives.iter().chain(&outputs) {
        match fs::remove_file(p) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    let mut paths = targets.clone();
    paths.extend(archives.clone());
    paths.extend(outputs.clone());
    c.host.call("log",json!({"header":"clean","text":format!("--clean removed {} rust target dirs + {} lib*_rust.a + {} cargo-module outputs:\n{}",targets.len(),archives.len(),outputs.len(),paths.join("\n"))}))?;
    c.stage(
        "clean",
        "ok",
        format!("{}t+{}a+{}o", targets.len(), archives.len(), outputs.len()),
        0,
        json!([]),
    )?;
    Ok(Value::Null)
}
fn stage_payloads(c: &mut Context<'_>) -> Result<Value> {
    let wanted = strings(&c.fact("payload_resolve", json!({}))?);
    if !wanted.is_empty() {
        let built = c.fact("payload_ensure", json!({"wanted":wanted}))?;
        if built["ok"] != true {
            c.classify("PAYLOAD_BUILD_FAIL")?;
            c.stage(
                "payloads",
                "fail",
                wanted.join(","),
                0,
                json!(strings(&built["why"])
                    .into_iter()
                    .take(6)
                    .collect::<Vec<_>>()),
            )?;
            return Ok(Value::Null);
        }
        let rebuilt = strings(&built["rebuilt"]);
        let detail = format!(
            "{}{}",
            wanted.join(","),
            if rebuilt.is_empty() {
                String::new()
            } else {
                format!(" (rebuilt {})", rebuilt.join(","))
            }
        );
        c.stage("payloads", "ok", detail, 0, json!([]))?;
    }
    c.fact("payload_stamp", json!({}))?;
    Ok(json!(wanted))
}
fn build_environment(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let epoch = c.fact("source_epoch", json!({}))?;
    if let Some(epoch) = epoch.as_str().filter(|s| !s.is_empty()) {
        if c.env.get("SOURCE_DATE_EPOCH").is_none() {
            c.env["SOURCE_DATE_EPOCH"] = json!(epoch);
            c.host
                .call("env", json!({"overlay":{"SOURCE_DATE_EPOCH":epoch}}))?;
        }
    }
    let cache = c.fact("cache_make", json!({}))?;
    let caching = c.fact("cache_enabled", json!({}))? == true;
    let mut env = Value::Null;
    if caching {
        let mut setup = a.clone();
        setup["cache"] = cache.clone();
        env = setup_cache(c, &setup)?;
        if env.is_null() {
            return Ok(Value::Null);
        }
    }
    let policy = c.fact("policy", json!({}))?;
    if policy.as_object().is_some_and(|p| !p.is_empty()) {
        env = c.fact("policy_environment", json!({"env":env}))?;
        c.host
            .call("set_run", json!({"extra":{"build_policy":policy}}))?;
    }
    let incremental = a["args"]["incremental"] == true
        || matches!(
            s(&c.env, "VIBEOS_RUST_INCREMENTAL", "")
                .to_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
    if incremental && !env.is_null() {
        for (k, v) in [
            ("CARGO_INCREMENTAL", "1"),
            ("RUSTC_WRAPPER", ""),
            ("CARGO_BUILD_RUSTC_WRAPPER", ""),
        ] {
            env[k] = json!(v)
        }
        c.stage(
            "incremental",
            "warn",
            "rustc INCREMENTAL on, Rust sccache OFF (fast-iterate; non-reproducible)",
            0,
            json!([]),
        )?;
    }
    env = c.fact("host_environment", json!({"env":env}))?;
    if let Some(overlay) = a["artifact_env"].as_object() {
        env.as_object_mut()
            .ok_or("host environment must be an object")?
            .extend(overlay.clone());
    }
    Ok(json!({"env":env,"caching":caching,"cache":cache}))
}
fn inputs_moved(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let Some(start) = a["tree"].as_str() else {
        return Ok(json!([]));
    };
    let end = json!(crate::warmth::worktree_tree(c.repo).ok());
    let Some(end) = end.as_str() else {
        return Ok(json!([]));
    };
    if end == start {
        return Ok(json!([]));
    }
    let mut cmd = vec![
        "git".to_owned(),
        "diff-tree".into(),
        "-r".into(),
        "--name-only".into(),
        start.into(),
        end.into(),
        "--".into(),
        ".".into(),
    ];
    cmd.extend(
        crate::provenance::NON_BUILD_PATHSPEC
            .iter()
            .map(|s| s.to_string()),
    );
    let out = c.host.call(
        "build_external",
        json!({"argv":cmd,"cwd":c.repo,"check":true}),
    )?;
    if out["error"].is_string() || out["returncode"] != 0 {
        return Ok(json!([]));
    }
    Ok(json!(s(&out, "stdout", "")
        .lines()
        .filter(|s| !s.is_empty() && !s.ends_with("Cargo.lock"))
        .collect::<Vec<_>>()))
}
fn failure_moved(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    let run = c.host.call("get_run", json!({}))?;
    if matches!(run["verdict"].as_str(), Some("OK" | "WARN"))
        || run["classification"] == "BUILD_INPUTS_MOVED"
    {
        return Ok(Value::Null);
    }
    let paths = strings(&inputs_moved(c, a)?);
    if paths.is_empty() {
        return Ok(Value::Null);
    }
    let was = s(&run, "classification", s(&run, "verdict", "FAIL"));
    c.classify("BUILD_INPUTS_MOVED")?;
    c.stage("inputs-moved","fail",format!("{} build input(s) changed while this build ran, so its {was} may be the edit's, not the tree's. Rebuild before diagnosing.",paths.len()),0,json!(paths.into_iter().take(8).collect::<Vec<_>>()))?;
    Ok(Value::Null)
}
fn dedup(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    if !Path::new(s(a, "unit", "/etc/systemd/system/vibeos-dedup.service")).exists() {
        return Ok(
            json!({"name":"dedup","status":"skip","detail":"no vibeos-dedup.service on this host"}),
        );
    }
    let out=c.host.call("build_external",json!({"argv":["sudo","-n","systemctl","start","--no-block","vibeos-dedup.service"],"check":true,"timeout":30}))?;
    if out["error"].is_string() || out["returncode"] != 0 {
        let error = s(&out, "error", s(&out, "stderr", "command failed"));
        return Ok(
            json!({"name":"dedup","status":"warn","detail":format!("could not start vibeos-dedup: {error}")}),
        );
    }
    Ok(json!({"name":"dedup","status":"ok","detail":"vibeos-dedup.service started"}))
}

fn trim(c: &mut Context<'_>) -> Result<Value> {
    if c.env["VIBEOS_TRIM"] == "0" {
        return Ok(Value::Null);
    }
    // A trim that cannot plan or delete never fails the build: everything it would remove
    // is unreachable by the current configuration, so skipping it only costs disk.
    let plan = match crate::trim::plan_worktree(c.repo, crate::trim::now(), 24.0) {
        Ok(plan) => plan,
        Err(error) => {
            c.host.call(
                "print",
                json!({"text":format!("[trim] skipped: {}", error.message)}),
            )?;
            return Ok(Value::Null);
        }
    };
    if plan.paths.is_empty() {
        return Ok(Value::Null);
    }
    match crate::trim::apply(c.repo, &plan) {
        Err(error) => {
            c.host.call(
                "print",
                json!({"text":format!("[trim] skipped: {}", error.message)}),
            )?;
        }
        Ok((freed, failures)) => {
            for line in &failures {
                c.host
                    .call("print", json!({"text":format!("  FAILED {line}")}))?;
            }
            c.host.call("print",json!({"text":format!("[trim] freed {:.1} G of superseded cargo output ({} units, {} idle ad-hoc dirs)",freed as f64/1024f64.powi(3),plan.superseded_units,plan.adhoc_dirs)}))?;
        }
    }
    Ok(Value::Null)
}
fn store_gc(c: &mut Context<'_>, a: &Value) -> Result<Value> {
    if a["args"]["no_artifact_store"] == true || c.env["VIBEOS_STORE_GC"] == "0" {
        return Ok(Value::Null);
    }
    let store = a["args"]["artifact_store"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            crate::cli_build_entry::default_store(&json!({"env":c.env,"home":s(&c.env,"HOME",".")}))
        });
    let report = c.fact("store_gc", json!({"store":store}))?;
    if let Some(error) = report["error"].as_str() {
        c.host.call(
            "print",
            json!({"text":format!("[store-gc] skipped: {error}")}),
        )?;
    } else if !report.is_null() {
        let count = |k: &str| report[k].as_array().map_or(0, Vec::len);
        c.host.call("print",json!({"text":format!("[store-gc] dropped {} dead refs, {} actions, {} blobs, {} staging dirs",count("dead_refs"),count("actions"),count("blobs"),count("staging"))}))?;
    }
    Ok(Value::Null)
}

pub fn dispatch(repo: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = &r["kwargs"];
    let arch = s(a, "arch", "").to_string();
    let build = a["build"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join(format!("build-{arch}")));
    let mut c = Context {
        host,
        repo,
        build,
        env: a.get("env").cloned().unwrap_or(json!({})),
    };
    match r["op"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches("cli_build_helper_")
    {
        "canonize" => canonize(&mut c, a),
        "setup_cache" => setup_cache(&mut c, a),
        "cache_stats" => cache_stats(&mut c, a),
        "refresh_guests" => refresh_guests(&mut c, a),
        "refresh_cpu" => refresh_cpu(&mut c),
        "verify_flags" => verify_flags(&mut c, a),
        "fex_prerequisites" => Ok(json!(prerequisites(repo, &c.env))),
        "cpu_stale" => Ok(json!(newer(repo))),
        "trim" => trim(&mut c),
        "store_gc" => store_gc(&mut c, a),
        "clean" => clean(&mut c),
        "stage_payloads" => stage_payloads(&mut c),
        "environment" => build_environment(&mut c, a),
        "inputs_moved" => inputs_moved(&mut c, a),
        "failure_moved" => failure_moved(&mut c, a),
        "dedup" => dedup(&mut c, a),
        "constants" => Ok(
            json!({"fex_out_of_tree":FEX_OUT_OF_TREE,"cpu_dll":CPU_DLL,"cpu_sources":CPU_SOURCES}),
        ),
        op => Err(Error::new(format!("unknown build helper operation {op}"))),
    }
}
