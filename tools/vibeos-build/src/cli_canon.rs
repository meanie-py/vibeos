//! Canonical-path configuration and shared pinned toolchain provisioning.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};
type Env = BTreeMap<String, String>;
pub const CANON: &str = "/tmp/vibeos";
pub fn canon(env: &Env) -> String {
    env.get("VIBEOS_CANON")
        .cloned()
        .unwrap_or_else(|| CANON.into())
}
pub fn wrap(repo: &Path, argv: &[String]) -> Vec<String> {
    let mut cmd = vec![
        "bash".into(),
        repo.join("tools/canon-run.sh")
            .to_string_lossy()
            .into_owned(),
    ];
    cmd.extend_from_slice(argv);
    cmd
}
pub fn cmake_home(build: &Path) -> Option<String> {
    let bytes = fs::read(build.join("CMakeCache.txt")).ok()?;
    String::from_utf8_lossy(&bytes)
        .lines()
        .find(|s| s.starts_with("CMAKE_HOME_DIRECTORY:"))
        .and_then(|s| s.split_once('=').map(|(_, s)| s.trim().to_string()))
}
pub fn carried(build: &Path) -> Vec<String> {
    let mut opts = vec![];
    if let Ok(bytes) = fs::read(build.join("CMakeCache.txt")) {
        for line in String::from_utf8_lossy(&bytes).lines() {
            for var in [
                "CMAKE_BUILD_TYPE",
                "REACTOS_TEST_KERNEL",
                "SARCH",
                "OARCH",
                "TUNE",
            ] {
                if line.starts_with(&format!("{var}:")) {
                    if let Some((_, v)) = line.split_once('=') {
                        let v = v.trim();
                        if !v.is_empty() {
                            opts.push(format!("-D{var}={v}"))
                        }
                    }
                }
            }
        }
    }
    opts
}
pub fn configure(arch: &str, extra: &[String], env: &Env) -> Vec<String> {
    let canon = canon(env);
    let mut cmd = vec![
        "cmake".into(),
        "-G".into(),
        "Ninja".into(),
        format!("-DCMAKE_TOOLCHAIN_FILE={canon}/toolchain-llvm-mingw.cmake"),
        format!("-DARCH:STRING={arch}"),
        "-DENABLE_CCACHE=1".into(),
        "-DCMAKE_BUILD_TYPE=Debug".into(),
    ];
    cmd.extend_from_slice(extra);
    cmd.push(canon);
    cmd
}
pub fn ensure_toolchain(repo: &Path, env: &Env) -> Result<bool> {
    let link = repo.join(".build-tools/llvm-mingw");
    let mut target = None;
    if link.exists() {
        if !link.is_symlink() {
            match fs::remove_dir(&link) {
                Ok(()) => {}
                Err(_) => return Ok(fs::read_dir(&link)?.next().is_some()),
            }
        } else {
            target = Some(link.canonicalize()?);
            if fs::read_link(&link)?.is_absolute() {
                return Ok(true);
            }
        }
    }
    if target.is_none() {
        let parent = env
            .get("VIBEOS_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                repo.canonicalize()
                    .unwrap_or_else(|_| repo.into())
                    .parent()
                    .unwrap_or(repo)
                    .to_owned()
            });
        let mut hits = fs::read_dir(parent)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("llvm-mingw-"))
            .map(|e| e.path())
            .collect::<Vec<_>>();
        hits.sort();
        let Some(first) = hits.first() else {
            return Ok(false);
        };
        target = Some(first.canonicalize()?);
    }
    let parent = link.parent().unwrap();
    fs::create_dir_all(parent)?;
    let staging = tempfile::Builder::new()
        .prefix(".toolchain-link-")
        .tempdir_in(parent)?;
    let tmp = staging.path().join("llvm-mingw");
    symlink(target.unwrap(), &tmp)?;
    fs::rename(tmp, link)?;
    Ok(true)
}
pub fn available(repo: &Path) -> Result<bool> {
    let dir = repo.join(".cache/canon-probes");
    let out = crate::cli_artifacts::command(
        &json!({"dir":dir,"repo":repo,"cwd":repo,"argv":["unshare","--map-root-user","--mount","true"],"timeout":10}),
    )?;
    Ok(out["returncode"] == 0)
}
pub fn dispatch(repo: &Path, r: &Value) -> Result<Value> {
    let a = &r["kwargs"];
    let env: Env = serde_json::from_value(
        a.get("env")
            .cloned()
            .unwrap_or_else(|| json!(std::env::vars().collect::<Env>())),
    )?;
    let build = PathBuf::from(a["build"].as_str().unwrap_or("."));
    let root = a["repo"].as_str().map(Path::new).unwrap_or(repo);
    let argv = || {
        serde_json::from_value::<Vec<String>>(a.get("argv").cloned().unwrap_or(json!([])))
            .map_err(Error::from)
    };
    match r["op"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches("cli_canon_")
    {
        "path" => Ok(json!(canon(&env))),
        "wrap" => Ok(json!(wrap(root, &argv()?))),
        "home" => Ok(json!(cmake_home(&build))),
        "is_canonical" => Ok(json!(cmake_home(&build) == Some(canon(&env)))),
        "carried" => Ok(json!(carried(&build))),
        "configure" => Ok(json!(configure(
            a["arch"].as_str().unwrap_or(""),
            &argv()?,
            &env
        ))),
        "toolchain" => Ok(json!(ensure_toolchain(root, &env)?)),
        "available" => Ok(json!(available(root)?)),
        op => Err(Error::new(format!(
            "unknown canonical build operation {op}"
        ))),
    }
}
