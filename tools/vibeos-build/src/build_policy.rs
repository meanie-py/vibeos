//! Build plans and cache/environment decisions. The host-jobs broker owns admission.
use crate::{artifact_fs, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
#[path = "../../host-jobs/src/build_policy.rs"]
pub mod capacity;
use capacity::Capacity;
pub type Environment = BTreeMap<String, String>;
fn refused(message: impl Into<String>) -> Error {
    Error::classified("PolicyError", message)
}
fn capacity(value: &Value) -> Result<Capacity> {
    let map = value
        .as_object()
        .ok_or_else(|| refused("invalid host build policy"))?;
    if map.len() != capacity::FIELDS.len() || !capacity::FIELDS.iter().all(|k| map.contains_key(*k))
    {
        return Err(refused("invalid host build policy"));
    }
    let mut values = [0; 5];
    for (index, key) in capacity::FIELDS.iter().enumerate() {
        values[index] = map[*key]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| refused("invalid host build policy"))?;
    }
    Capacity::new(values).map_err(refused)
}
fn value(policy: Capacity) -> Value {
    Value::Object(
        capacity::FIELDS
            .into_iter()
            .zip(policy.values())
            .map(|(k, v)| (k.to_owned(), json!(v)))
            .collect(),
    )
}
pub fn read(root: &Path) -> Result<Option<Value>> {
    let path = root.join("build-policy.json");
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && !path.is_symlink() => {
            return Ok(None)
        }
        Err(e) => {
            return Err(refused(format!(
                "unreadable host build policy: {}: {e}",
                path.display()
            )))
        }
    };
    let policy: Value = serde_json::from_slice(&bytes).map_err(|e| {
        refused(format!(
            "unreadable host build policy: {}: {e}",
            path.display()
        ))
    })?;
    capacity(&policy).map_err(|e| refused(format!("{}: {}", e.message, path.display())))?;
    Ok(Some(policy))
}
pub fn main_worktree(repo: &Path) -> Result<PathBuf> {
    let dot = repo.join(".git");
    if dot.is_dir() {
        return Ok(repo.to_owned());
    }
    let text = match fs::read_to_string(dot) {
        Ok(s) => s,
        Err(_) => return Ok(repo.to_owned()),
    };
    if let Some(gitdir) = text.trim().strip_prefix("gitdir:") {
        let path = PathBuf::from(gitdir.trim());
        for p in path.ancestors().skip(1) {
            if p.file_name().is_some_and(|n| n == ".git") {
                return Ok(p.parent().ok_or("git directory has no parent")?.to_owned());
            }
        }
    }
    Ok(repo.to_owned())
}
pub fn policy_dir(repo: &Path) -> Result<PathBuf> {
    match std::env::var_os("VIBEOS_BUILD_POLICY_DIR").filter(|s| !s.is_empty()) {
        Some(p) => Ok(p.into()),
        None => main_worktree(repo),
    }
}
pub(crate) fn expand_user(path: &str, env: &Environment) -> Result<PathBuf> {
    if !path.starts_with('~') {
        return Ok(path.into());
    }
    let (user, tail) = path[1..].split_once('/').unwrap_or((&path[1..], ""));
    let home = if user.is_empty() {
        env.get("HOME").cloned()
    } else {
        None
    };
    let home = if let Some(home) = home {
        PathBuf::from(home)
    } else {
        use std::ffi::{CStr, CString};
        let user = CString::new(user).map_err(|_| Error::new("invalid home-directory username"))?;
        let mut buffer = vec![0u8; 16384];
        loop {
            let mut record: libc::passwd = unsafe { std::mem::zeroed() };
            let mut result = std::ptr::null_mut();
            let rc = unsafe {
                if user.as_bytes().is_empty() {
                    libc::getpwuid_r(
                        libc::getuid(),
                        &mut record,
                        buffer.as_mut_ptr().cast(),
                        buffer.len(),
                        &mut result,
                    )
                } else {
                    libc::getpwnam_r(
                        user.as_ptr(),
                        &mut record,
                        buffer.as_mut_ptr().cast(),
                        buffer.len(),
                        &mut result,
                    )
                }
            };
            if rc == libc::ERANGE {
                buffer.resize(
                    buffer
                        .len()
                        .checked_mul(2)
                        .ok_or("passwd buffer overflow")?,
                    0,
                );
                continue;
            }
            if rc != 0 || result.is_null() {
                return Err(Error::new(format!(
                    "cannot resolve home directory for {path:?}"
                )));
            }
            let bytes = unsafe { CStr::from_ptr(record.pw_dir) }.to_bytes();
            use std::os::unix::ffi::OsStrExt;
            break PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
        }
    };
    Ok(if tail.is_empty() {
        home
    } else {
        home.join(tail)
    })
}
pub fn fleet_root(repo: &Path, env: &Environment) -> Result<PathBuf> {
    if let Some(root) = env.get("VIBEOS_ROOT").filter(|s| !s.is_empty()) {
        return artifact_fs::resolve(&expand_user(root, env)?);
    }
    let path = artifact_fs::resolve(repo)?;
    let parts: Vec<_> = path.components().collect();
    let cut = parts
        .iter()
        .position(|c| c.as_os_str() == ".claude")
        .or_else(|| parts.iter().position(|c| c.as_os_str() == ".git"));
    let lane = if let Some(cut) = cut {
        parts[..cut].iter().collect::<PathBuf>()
    } else {
        path
    };
    Ok(lane.parent().unwrap_or(&lane).to_owned())
}
fn which(name: &str, env: &Environment) -> Result<Option<String>> {
    use std::os::unix::ffi::OsStrExt;
    let path = env
        .get("PATH")
        .cloned()
        .unwrap_or_else(|| "/bin:/usr/bin".into());
    for dir in std::env::split_paths(&path) {
        let candidate = if dir.as_os_str().is_empty() {
            PathBuf::from(".").join(name)
        } else {
            dir.join(name)
        };
        let name = std::ffi::CString::new(candidate.as_os_str().as_bytes())
            .map_err(|_| Error::new("cache executable path contains NUL"))?;
        if fs::metadata(&candidate).is_ok_and(|m| m.is_file())
            && unsafe { libc::access(name.as_ptr(), libc::X_OK) } == 0
        {
            return Ok(Some(candidate.to_string_lossy().into_owned()));
        }
    }
    Ok(None)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheEnv {
    pub root: PathBuf,
    pub ccache_dir: PathBuf,
    pub sccache_dir: PathBuf,
    pub ccache_bin: Option<String>,
    pub sccache_bin: Option<String>,
}
pub fn cache_make(repo: &Path, env: &Environment) -> Result<CacheEnv> {
    let root = fleet_root(repo, env)?;
    Ok(CacheEnv {
        ccache_dir: root.join("ccache"),
        sccache_dir: root.join("sccache"),
        root,
        ccache_bin: which("ccache", env)?,
        sccache_bin: which("sccache", env)?,
    })
}
pub fn cache_environment(cache: &CacheEnv) -> Environment {
    let mut e = Environment::new();
    if cache.ccache_bin.is_some() {
        for (k, v) in [
            ("CCACHE_DIR", cache.ccache_dir.display().to_string()),
            ("CCACHE_BASEDIR", cache.root.display().to_string()),
            ("CCACHE_NOHASHDIR", "true".into()),
            ("CCACHE_SLOPPINESS", "time_macros".into()),
            ("CCACHE_MAXSIZE", "50G".into()),
            (
                "CCACHE_TEMPDIR",
                cache.ccache_dir.join("tmp").display().to_string(),
            ),
        ] {
            e.insert(k.into(), v);
        }
    }
    if let Some(bin) = &cache.sccache_bin {
        for (k, v) in [
            (
                "RUSTC_WRAPPER",
                if bin.is_empty() {
                    "sccache".into()
                } else {
                    bin.clone()
                },
            ),
            ("SCCACHE_DIR", cache.sccache_dir.display().to_string()),
            ("SCCACHE_CACHE_SIZE", "150G".into()),
            ("CARGO_INCREMENTAL", "0".into()),
        ] {
            e.insert(k.into(), v);
        }
    }
    e
}
pub fn ninja_jobs(policy: Option<&Value>, env: &Environment, cpus: usize) -> Result<Vec<String>> {
    if let Some(policy) = policy {
        return Ok(vec!["-j".into(), capacity(policy)?.ninja_jobs.to_string()]);
    }
    let reserve: i64 = env
        .get("VIBEOS_BUILD_CORE_RESERVE")
        .map(String::as_str)
        .unwrap_or("0")
        .trim()
        .parse()
        .map_err(|_| Error::new("VIBEOS_BUILD_CORE_RESERVE must be an integer"))?;
    Ok(if reserve <= 0 || cpus as u64 <= reserve as u64 {
        vec![]
    } else {
        vec!["-j".into(), (cpus - reserve as usize).to_string()]
    })
}
pub fn build_environment(
    policy: Option<&Value>,
    env: Option<Environment>,
    ambient: &Environment,
) -> Result<Option<Environment>> {
    let Some(p) = policy else {
        return Ok(env);
    };
    let mut result = env.unwrap_or_else(|| ambient.clone());
    result.insert(
        "CARGO_BUILD_JOBS".into(),
        capacity(p)?.cargo_jobs.to_string(),
    );
    Ok(Some(result))
}
pub fn host_environment(
    repo: &Path,
    mut env: Environment,
    canonical: Option<&Path>,
    run_dir: Option<&Path>,
    owner: u32,
    class: &str,
    policy: Option<&Value>,
) -> Result<Environment> {
    let runner = canonical
        .unwrap_or(repo)
        .join(".cache/host-jobs/host-jobs")
        .display()
        .to_string();
    let inner = env
        .get("RUSTC_WRAPPER")
        .or_else(|| env.get("CARGO_BUILD_RUSTC_WRAPPER"))
        .cloned()
        .unwrap_or_default();
    if !inner.is_empty() && inner != runner {
        env.insert("VIBEOS_INNER_RUSTC_WRAPPER".into(), inner);
    } else {
        env.remove("VIBEOS_INNER_RUSTC_WRAPPER");
    }
    for (k, v) in [
        ("VIBEOS_HOST_JOBS", runner.clone()),
        ("RUSTC_WRAPPER", runner.clone()),
        ("CARGO_BUILD_RUSTC_WRAPPER", runner),
        ("CARGO_BUILD_JOBS", "1".into()),
        ("VIBEOS_JOB_REPO", repo.display().to_string()),
        ("VIBEOS_JOB_OWNER_PID", owner.to_string()),
        (
            "VIBEOS_JOB_RUN",
            run_dir.map(|p| p.display().to_string()).unwrap_or_default(),
        ),
        ("VIBEOS_JOB_CLASS", class.into()),
    ] {
        env.insert(k.into(), v);
    }
    policy_environment(&mut env, policy)?;
    Ok(env)
}
fn policy_environment(env: &mut Environment, policy: Option<&Value>) -> Result<()> {
    if let Some(p) = policy {
        let c = capacity(p)?;
        env.insert("VIBEOS_HOST_JOB_SLOTS".into(), c.slots().to_string());
        env.insert("VIBEOS_HOST_CPU_LEASES".into(), c.leases.to_string());
    }
    Ok(())
}
pub fn host_jobs_status(repo: &Path) -> Result<Value> {
    host_jobs_status_at(repo, &std::env::vars().collect(), &policy_dir(repo)?)
}
pub fn host_jobs_status_at(repo: &Path, ambient: &Environment, root: &Path) -> Result<Value> {
    let runner = repo.join(".cache/host-jobs/host-jobs");
    if !runner.exists() {
        return Ok(json!({"available":false}));
    }
    let mut env = ambient.clone();
    policy_environment(&mut env, read(root)?.as_ref())?;
    let result = crate::artifact_executor::supervised_output(
        Command::new(runner).arg("--status").env_clear().envs(env),
        Some(std::time::Duration::from_secs(30)),
    )?;
    if result.code != 0 {
        return Err(Error::new(format!(
            "host-jobs status failed: {}",
            result.stderr
        )));
    }
    Ok(serde_json::from_str(&result.stdout)?)
}
pub fn cmake_value(build: &Path, key: &str) -> String {
    let Ok(bytes) = fs::read(build.join("CMakeCache.txt")) else {
        return String::new();
    };
    for line in String::from_utf8_lossy(&bytes).lines() {
        if line.starts_with(&format!("{key}:")) {
            return line
                .split_once('=')
                .map(|(_, v)| v.trim().to_owned())
                .unwrap_or_default();
        }
    }
    String::new()
}
pub fn source_date_epoch(repo: &Path) -> Result<Option<String>> {
    let mut cmd = Command::new("git");
    cmd.args(["log", "-1", "--format=%ct"]).current_dir(repo);
    let result = match crate::artifact_executor::supervised_output(
        &mut cmd,
        Some(std::time::Duration::from_secs(30)),
    ) {
        Ok(result) => result,
        Err(error)
            if matches!(
                error.kind.as_deref(),
                Some("KeyboardInterrupt" | "InterruptedError")
            ) =>
        {
            return Err(error)
        }
        Err(_) => return Ok(None),
    };
    let text = result.stdout.trim();
    Ok(
        if result.code == 0 && !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
            Some(text.into())
        } else {
            None
        },
    )
}
fn sccache(text: &str) -> Value {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return json!({});
    };
    let stats = v.get("stats").unwrap_or(&v);
    if !stats.is_object() {
        return json!({});
    }
    fn total(v: &Value) -> i128 {
        let v = v.get("counts").unwrap_or(v);
        if let Some(m) = v.as_object() {
            return m
                .values()
                .filter_map(|v| {
                    v.as_number()
                        .and_then(|n| n.to_string().parse::<i128>().ok())
                })
                .sum();
        }
        v.as_number()
            .and_then(|n| n.to_string().parse().ok())
            .unwrap_or(0)
    }
    let hits = total(&stats["cache_hits"]);
    let misses = total(&stats["cache_misses"]);
    let denom = hits + misses;
    json!({"requests":total(&stats["compile_requests"]),"hits":hits,"misses":misses,"hit_rate":if denom==0 {0.0} else {hits as f64/denom as f64},
        "non_cacheable":total(&stats["requests_not_cacheable"]),"non_cacheable_reasons":stats.get("not_cached").cloned().unwrap_or_else(||json!({})),"cache_errors":total(&stats["cache_errors"]),"compilations":total(&stats["compilations"])})
}
fn ccache(text: &str) -> Value {
    let mut counts = BTreeMap::new();
    for line in text.lines() {
        let p: Vec<_> = line.split('\t').collect();
        if p.len() == 2 {
            if let Ok(n) = p[1].trim().parse::<u64>() {
                counts.insert(p[0].trim(), n as u128);
            }
        }
    }
    let hits = counts.get("direct_cache_hit").copied().unwrap_or(0)
        + counts.get("preprocessed_cache_hit").copied().unwrap_or(0);
    let misses = counts.get("cache_miss").copied().unwrap_or(0);
    json!({"hits":hits,"misses":misses,"hit_rate":if hits+misses==0 {0.0} else {hits as f64/(hits+misses) as f64}})
}
pub fn dispatch(request: &Value) -> Result<Value> {
    let a = &request["kwargs"];
    let path = |k: &str| -> Result<PathBuf> {
        Ok(serde_json::from_value(
            a.get(k)
                .cloned()
                .ok_or_else(|| Error::new(format!("missing {k}")))?,
        )?)
    };
    let env = || -> Result<Environment> { Ok(serde_json::from_value(a["environment"].clone())?) };
    match request["op"].as_str().ok_or("missing policy operation")? {
        "build_policy_constants" => Ok(
            json!({"version":capacity::VERSION,"trial":value(Capacity::trial(false)),"ccache_maxsize":"50G","sccache_maxsize":"150G"}),
        ),
        "build_policy_read" => Ok(read(&path("root")?)?.unwrap_or(Value::Null)),
        "build_policy_main_worktree" => Ok(json!(main_worktree(&path("repo")?)?)),
        "build_policy_cache_root" => Ok(json!(fleet_root(&path("repo")?, &env()?)?)),
        "build_policy_cache_make" => {
            Ok(serde_json::to_value(cache_make(&path("repo")?, &env()?)?)?)
        }
        "build_policy_cache_env" => Ok(json!(cache_environment(&serde_json::from_value(
            a["cache"].clone()
        )?))),
        "build_policy_cache_enabled" => {
            let enabled = if let Some(flag) = a["flag"].as_bool() {
                flag
            } else {
                !["0", "false", "no", "off"].contains(
                    &env()?
                        .get("VIBEOS_BUILD_CACHE")
                        .cloned()
                        .unwrap_or_default()
                        .to_lowercase()
                        .as_str(),
                )
            };
            Ok(json!(enabled))
        }
        "build_policy_sccache_stats" => {
            Ok(sccache(a["text"].as_str().ok_or("missing stats text")?))
        }
        "build_policy_ccache_stats" => Ok(ccache(a["text"].as_str().ok_or("missing stats text")?)),
        "build_policy_ninja_jobs" => Ok(json!(ninja_jobs(
            a.get("policy").filter(|v| !v.is_null()),
            &env()?,
            a["cpus"].as_u64().ok_or("missing CPU count")? as usize
        )?)),
        "build_policy_build_env" => Ok(json!(build_environment(
            a.get("policy").filter(|v| !v.is_null()),
            serde_json::from_value(a["env"].clone())?,
            &env()?
        )?)),
        "build_policy_host_env" => Ok(json!(host_environment(
            &path("repo")?,
            env()?,
            serde_json::from_value::<Option<PathBuf>>(a["canonical"].clone())?.as_deref(),
            serde_json::from_value::<Option<PathBuf>>(a["run_dir"].clone())?.as_deref(),
            a["owner"]
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or("invalid owner PID")?,
            a["class"].as_str().ok_or("missing job class")?,
            a.get("policy").filter(|v| !v.is_null())
        )?)),
        "build_policy_host_status" => host_jobs_status_at(&path("repo")?, &env()?, &path("root")?),
        "build_policy_ensure_host_jobs" => Ok(json!(crate::cargo_orchestrate::ensure_host_jobs(
            &path("repo")?
        )?)),
        "build_policy_cmake_value" => Ok(json!(cmake_value(
            &path("build")?,
            a["key"].as_str().ok_or("missing CMake key")?
        ))),
        "build_policy_source_date_epoch" => Ok(json!(source_date_epoch(&path("repo")?)?)),
        "build_policy_ninja_work" => {
            let text = a["text"].as_str().ok_or("missing Ninja output")?;
            Ok(json!(if text.contains("ninja: no work to do.") {
                0
            } else {
                regex::Regex::new(r"(?m)^\[([1-9]\d*)/\d+\]")?
                    .find_iter(text)
                    .count()
            }))
        }
        "build_policy_admission" => {
            let c = capacity(&a["policy"])?;
            if a["has_override"] == true && a["leases_override"].as_u64() != Some(c.leases as u64) {
                return Err(refused(
                    "per-client lease override conflicts with host policy",
                ));
            }
            let mut result = json!({"leases": c.leases});
            if a["build_share"] == true {
                result["n"] = json!(c.build_leases());
            }
            Ok(result)
        }
        "build_policy_storage_options" => {
            let mut result = Vec::new();
            let build = path("build")?;
            for (key, field) in [
                ("VIBEOS_RUST_STORAGE_SHARDS", "shards"),
                ("VIBEOS_RUST_BODY_SHARDS", "body_shards"),
            ] {
                if let Some(number) = a.get(field).filter(|v| !v.is_null()) {
                    let value = number
                        .as_i64()
                        .ok_or("storage shards must be integers")?
                        .to_string();
                    if cmake_value(&build, key) != value {
                        result.push(format!("-D{key}={value}"));
                    }
                }
            }
            Ok(json!(result))
        }
        "build_policy_trial" => {
            let c = Capacity::trial(a["rollback"].as_bool().unwrap_or(false));
            if c.slots() > a["cpus"].as_u64().ok_or("missing CPU count")? as usize {
                return Err(Error::new("trial job budget exceeds visible CPU count"));
            }
            Ok(value(c))
        }
        _ => Err(Error::new("unknown build policy operation")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_python_policy_cache_and_wrapper_plans() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/build-policy-python.json"))
                .unwrap();
        for row in fixture["cases"].as_array().unwrap() {
            assert_eq!(dispatch(row).unwrap(), row["expected"], "{}", row);
        }
    }
    #[test]
    fn policy_refusals_and_disposable_filesystem_boundaries() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(read(root).unwrap(), None);
        for invalid in [
            json!({}),
            json!([1, 6, 3, 6, 1]),
            json!({"version":1,"leases":6,"max_builds":3,"ninja_jobs":6,"cargo_jobs":true}),
            json!({"version":1,"leases":6,"max_builds":3,"ninja_jobs":6,"cargo_jobs":1,"extra":1}),
        ] {
            fs::write(root.join("build-policy.json"), invalid.to_string()).unwrap();
            assert_eq!(read(root).unwrap_err().kind.as_deref(), Some("PolicyError"));
        }
        fs::write(
            root.join("build-policy.json"),
            value(Capacity::trial(false)).to_string(),
        )
        .unwrap();
        assert_eq!(read(root).unwrap().unwrap(), value(Capacity::trial(false)));
        let req = json!({"op":"build_policy_admission","kwargs":{"policy":value(Capacity::trial(false)),"has_override":true,"leases_override":4,"build_share":true}});
        assert!(dispatch(&req)
            .unwrap_err()
            .message
            .contains("override conflicts"));
        fs::write(
            root.join("CMakeCache.txt"),
            "VIBEOS_RUST_STORAGE_SHARDS:STRING=4\nVIBEOS_RUST_BODY_SHARDS:STRING=2\n",
        )
        .unwrap();
        let req = json!({"op":"build_policy_storage_options","kwargs":{"build":root,"shards":4,"body_shards":2}});
        assert_eq!(dispatch(&req).unwrap(), json!([]));
        fs::remove_file(root.join("build-policy.json")).unwrap();
        std::os::unix::fs::symlink(root.join("missing"), root.join("build-policy.json")).unwrap();
        assert_eq!(read(root).unwrap_err().kind.as_deref(), Some("PolicyError"));
    }
}
