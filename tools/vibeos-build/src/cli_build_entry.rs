//! Build entry, resource sequencing and immutable-body supervision policy.
use crate::{cli_build::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v[k].as_str().unwrap_or(d)
}
fn stage(
    host: &mut dyn Host,
    name: &str,
    status: &str,
    detail: impl Into<String>,
    salient: Value,
) -> Result<()> {
    host.call("stage",json!({"name":name,"status":status,"detail":detail.into(),"exit":0,"seconds":0.,"salient":salient}))?;
    Ok(())
}
fn fail(
    host: &mut dyn Host,
    class: &str,
    name: &str,
    detail: impl Into<String>,
    salient: Value,
) -> Result<Value> {
    host.call("set_run", json!({"classification":class}))?;
    stage(host, name, "fail", detail, salient)?;
    host.call("finalize", json!({}))
}
fn fact(host: &mut dyn Host, name: &str, args: Value) -> Result<Value> {
    host.call("build_entry_fact", json!({"name":name,"arguments":args}))
}
fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
fn source_fail(
    host: &mut dyn Host,
    class: &str,
    name: &str,
    detail: impl Into<String>,
    salient: Value,
) -> Result<Value> {
    host.call("set_run", json!({"classification":class}))?;
    stage(host, name, "fail", detail, salient)?;
    Ok(json!(false))
}
pub fn hold_source(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = &a["args"];
    let arch = s(a, "arch", "");
    if arch != "amd64" || args["no_wow64_stage"] == true {
        return Ok(json!(true));
    }
    let source = repo.join("build-i386");
    let start = Instant::now();
    let mut acquired = host.call(
        "build_source_acquire",
        json!({"build":source,"nonblocking":true}),
    )? == true;
    if !acquired {
        host.call("print",json!({"text":"[build] an i386 build holds build-i386, which this amd64 media stages SysWOW64 from: waiting for it (--no-wow64-stage builds beside it instead)","flush":true}))?;
        acquired = host.call(
            "build_source_acquire",
            json!({"build":source,"nonblocking":false}),
        )? == true;
    }
    if !acquired {
        return source_fail(
            host,
            "BUILD_POLICY_REFUSED",
            "wow64-source",
            "timed out waiting for the i386 build this media stages from",
            json!(["or build with --no-wow64-stage and restage after i386 finishes"]),
        );
    }
    if !source.is_dir() {
        return source_fail(host,"NO_BUILD_DIR","wow64-source","build-i386 missing: amd64 media stages SysWOW64 from it",json!(["build i386 first (`vibeos-cli build --arch i386`), or pass --no-wow64-stage and restage after it; `gate` orders this itself"]));
    }
    let elapsed = start.elapsed().as_secs_f64();
    stage(
        host,
        "wow64-source",
        "ok",
        if elapsed >= 1. {
            format!("build-i386 held (waited {elapsed:.0}s for an i386 build)")
        } else {
            "build-i386 held".into()
        },
        json!([]),
    )?;
    Ok(json!(true))
}
pub fn entry(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let mut args = a["args"].clone();
    let arch = s(a, "arch", "");
    crate::arch(arch)?;
    let build = repo.join(format!("build-{arch}"));
    let headers = args["headers_only"] == true;
    if headers {
        host.call("set_run", json!({"extra":{"partial":"headers-only"}}))?;
        if arch == "amd64" {
            args["no_wow64_stage"] = json!(true)
        }
        let ignored = [
            "clean",
            "extra_target",
            "check",
            "no_check",
            "strict",
            "keep_going",
            "artifact_store",
            "artifact_store_verify",
            "no_artifact_store",
            "rust_storage_shards",
            "rust_body_shards",
            "incremental",
        ];
        let requested = ignored
            .into_iter()
            .filter(|k| {
                !args[*k].is_null()
                    && args[*k] != false
                    && args[*k] != json!(0)
                    && args[*k] != json!([])
            })
            .map(|k| format!("--{}", k.replace('_', "-")))
            .collect::<Vec<_>>();
        if !requested.is_empty() {
            return fail(
                host,
                "HEADERS_OPTIONS_REFUSED",
                "preflight",
                format!("--headers-only cannot honor {}", requested.join(", ")),
                json!([]),
            );
        }
    }
    host.call("build_arguments", json!({"args":args}))?;
    if !build.is_dir() && fact(host, "canon_available", json!({}))? != true {
        return fail(
            host,
            "NO_BUILD_DIR",
            "preflight",
            "build dir missing",
            json!([format!(
                "{} does not exist — configure it first",
                build.display()
            )]),
        );
    }
    if arch == "arm64" && a["env"]["VIBEOS_FEX"] == "1" {
        let missing = fact(host, "fex_prerequisites", json!({}))?;
        if missing.as_array().is_some_and(|v| !v.is_empty()) {
            return fail(
                host,
                "FEX_PREREQ_MISSING",
                "fex-preflight",
                format!(
                    "VIBEOS_FEX=1 needs {} thing(s) this tree does not have",
                    missing.as_array().unwrap().len()
                ),
                missing,
            );
        }
    }
    let admission = host.call("build_resources_open", json!({"build":build,"arch":arch}))?;
    if let Some(error) = admission["error"].as_str() {
        return fail(host, "BUILD_POLICY_REFUSED", "admission", error, json!([]));
    }
    host.call(
        "set_run",
        json!({"extra":{"admission":admission["admission"]}}),
    )?;
    let body = (|| -> Result<Value> {
        if hold_source(repo, &json!({"args":args,"arch":arch}), host)? != true {
            return host.call("finalize", json!({}));
        }
        host.call("build_reservation_open", json!({}))?;
        let result = if headers {
            match remove(&build.join(crate::provenance::STAMP_NAME)) {
                Ok(()) => host.call("build_locked", json!({})),
                Err(error) => fail(
                    host,
                    "HEADERS_STAMP_INVALIDATION_FAIL",
                    "preflight",
                    error.to_string(),
                    json!([]),
                ),
            }
        } else {
            host.call("build_supervised", json!({}))
        };
        host.call("build_reservation_close", json!({}))?;
        let result = result?;
        if result == 0 && !headers {
            fact(host, "trim", json!({}))?;
            fact(host, "store_gc", json!({}))?;
        }
        Ok(result)
    })();
    let closed = host.call("build_resources_close", json!({}));
    match (body, closed) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(result), Ok(_)) => Ok(result),
    }
}
pub fn default_store(a: &Value) -> PathBuf {
    a["env"]["VIBEOS_ARTIFACT_STORE_ROOT"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(s(a, "home", ".")).join("vibeos-artifact-store"))
}
pub fn supervised(repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let mut args = a["args"].clone();
    let verify = args["artifact_store_verify"] == true;
    let incremental = args["incremental"] == true
        || matches!(
            s(&a["env"], "VIBEOS_RUST_INCREMENTAL", "")
                .to_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
    let mut store = args["artifact_store"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    if store.is_none() && !verify && !incremental && args["no_artifact_store"] != true {
        store = Some(default_store(a));
        args["artifact_store"] = json!(store);
        host.call("build_arguments", json!({"args":args}))?;
    }
    if store.is_none() && !verify {
        return host.call("build_locked", json!({}));
    }
    let mut completed = None;
    let result = (|| -> Result<Value> {
        let store = store.ok_or("--artifact-store-verify requires --artifact-store")?;
        if !store.is_absolute() {
            return Err("--artifact-store must name an absolute directory".into());
        }
        if incremental {
            return Err("immutable body store does not support incremental builds".into());
        }
        fs::create_dir_all(&store)?;
        if fact(host, "toolchain", json!({}))? != true {
            return Err(".build-tools/llvm-mingw missing and no sibling llvm-mingw-* dir".into());
        }
        fact(host, "broker", json!({}))?;
        let state = host.call("get_run", json!({}))?;
        let output = Path::new(s(&state, "dir", ".")).join("artifact-supervisor");
        let toolchain = Path::new(s(a, "home", "."))
            .join(".rustup/toolchains")
            .join(crate::artifact_executor::PIN);
        host.call("set_run",json!({"extra":{"artifact_store":{"root":store,"report":output.join("summary.json"),"verify":verify,"clean":a["cold"]==true}}}))?;
        let opened=host.call("build_supervisor_open",json!({"store":store,"toolchain":toolchain,"output":output,"clean":a["cold"]==true,"verify":verify,"arch":a["arch"]}))?;
        if let Some(error) = opened["error"].as_str() {
            return Err(error.to_string().into());
        }
        let supervisor = opened["path"]
            .as_str()
            .ok_or("supervisor did not return a path")?;
        let body = (|| -> Result<Value> {
            let config: Value = serde_json::from_slice(&fs::read(supervisor)?)?;
            let rc=host.call("build_locked",json!({"capture_errors":true,"artifact_env":{"VIBEOS_ARTIFACT_SUPERVISOR":supervisor,"VIBEOS_ARTIFACT_TOOL":repo.join("tools/build-infra"),"VIBEOS_ARTIFACT_TOOLCHAIN_ID":config["toolchain_key"]}}))?;
            if let Some(error) = rc["error"].as_str() {
                return Err(error.to_owned().into());
            }
            completed = rc.as_i64();
            host.call(
                "build_supervisor_record",
                json!({"path":supervisor,"code":rc}),
            )?;
            Ok(rc)
        })();
        let closed = host.call("build_supervisor_close", json!({}))?;
        if let Some(error) = closed["error"].as_str() {
            return Err(error.to_string().into());
        }
        body
    })();
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            if completed == Some(0) {
                let build = a["build"]
                    .as_str()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| repo.join(format!("build-{}", s(a, "arch", ""))));
                remove(&build.join(crate::provenance::STAMP_NAME))?;
            }
            fail(
                host,
                "ARTIFACT_STORE_FAIL",
                "artifact-store",
                error.to_string(),
                json!([]),
            )
        }
    }
}
pub fn dispatch(repo: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    match r["op"].as_str().unwrap_or("") {
        "cli_build_entry_run" => entry(repo, &r["kwargs"], host),
        "cli_build_entry_supervised" => supervised(repo, &r["kwargs"], host),
        "cli_build_entry_hold_source" => hold_source(repo, &r["kwargs"], host),
        "cli_build_entry_default_store" => Ok(json!(default_store(&r["kwargs"]))),
        op => Err(Error::new(format!("unknown build entry operation {op}"))),
    }
}
