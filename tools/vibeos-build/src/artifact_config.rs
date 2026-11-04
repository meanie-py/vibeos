//! Fail-loud audit of ambient Cargo configuration before isolated artifact execution.
use crate::{artifact_fs as af, Error, Result, ARCHES};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
pub const TARGET: &str = "x86_64-pc-windows-gnu";
pub const CANONICAL_ROOT: &str = "/tmp/vibeos";
pub const KNOWN_TARGETS: [&str; 4] = [
    TARGET,
    "i686-pc-windows-gnu",
    "aarch64-pc-windows-gnullvm",
    "i686-kernel",
];
pub fn cargo_scope(repo: &Path) -> Result<Vec<PathBuf>> {
    let repo = af::resolve(repo)?;
    let workspace = repo.join("sdk/lib/rust");
    let mut paths: BTreeSet<_> = ARCHES
        .iter()
        .map(|a| repo.join(format!("build-{a}")))
        .collect();
    paths.insert(workspace.clone());
    if workspace.is_dir() {
        for entry in fs::read_dir(&workspace)? {
            let entry = entry?;
            let name = entry.file_name();
            if !["target", ".git", ".cargo"].iter().any(|s| name == *s)
                && entry.path().is_dir()
                && !entry.file_type()?.is_symlink()
            {
                paths.insert(entry.path());
            }
        }
    }
    Ok(paths.into_iter().collect())
}
pub fn mapped_cwd(
    repo: &Path,
    cwd: Option<&Path>,
    kernel: bool,
    canonical: &Path,
) -> Result<PathBuf> {
    let repo = af::resolve(repo)?;
    let default = repo.join(if kernel {
        "sdk/lib/rust"
    } else {
        "build-amd64"
    });
    let cwd = cwd.unwrap_or(&default);
    if !cwd.is_absolute() {
        return Err("actual nested Cargo cwd must be absolute".into());
    }
    let mut cwd = af::resolve(cwd)?;
    if cwd.starts_with(canonical) {
        if !af::same_file(canonical, &repo) {
            return Err("canonical Cargo cwd does not mount this source checkout".into());
        }
        cwd = repo.join(cwd.strip_prefix(canonical).unwrap());
    }
    if !cargo_scope(&repo)?.contains(&cwd) {
        return Err(Error::new(format!(
            "nested Cargo cwd is outside the supervisor's audited scope: {}",
            cwd.display()
        )));
    }
    Ok(cwd)
}
pub fn environment(env: &BTreeMap<String, String>, broker: Option<&Path>) -> Result<()> {
    for (key, value) in env {
        let reason = if key.starts_with("CARGO_PROFILE_") {
            Some("profile")
        } else if key.starts_with("CARGO_TARGET_") && key.ends_with("_RUSTFLAGS") {
            Some("target flags")
        } else if key == "RUSTC_BOOTSTRAP" && !value.is_empty() {
            return Err("unsupported original compiler environment: RUSTC_BOOTSTRAP".into());
        } else if key.starts_with("CARGO_UNSTABLE_")
            && ![
                "CARGO_UNSTABLE_BUILD_STD",
                "CARGO_UNSTABLE_BUILD_STD_FEATURES",
            ]
            .contains(&key.as_str())
        {
            Some("unstable")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(Error::new(format!(
                "unsupported original Cargo {reason} environment: {key}"
            )));
        }
        if !key.starts_with("CARGO_BUILD_")
            || [
                "CARGO_BUILD_TARGET",
                "CARGO_BUILD_RUSTFLAGS",
                "CARGO_BUILD_RUSTC",
            ]
            .contains(&key.as_str())
        {
            continue;
        }
        if key == "CARGO_BUILD_JOBS"
            && value
                .as_bytes()
                .first()
                .is_some_and(|b| matches!(b, b'1'..=b'9'))
            && value.bytes().all(|b| b.is_ascii_digit())
        {
            continue;
        }
        if key == "CARGO_BUILD_RUSTC_WRAPPER"
            && (value.is_empty() || broker.is_some_and(|b| af::same_file(Path::new(value), b)))
        {
            continue;
        }
        return Err(Error::new(format!(
            "unsupported original Cargo build environment: {key}"
        )));
    }
    Ok(())
}
pub fn allowed(
    config: &Value,
    path: &Path,
    requested: Option<&str>,
    rustflags_set: bool,
) -> Result<()> {
    let config = config
        .as_object()
        .ok_or("Cargo configuration must be an object")?;
    if config.keys().any(|k| k != "target") {
        return Err(Error::new(format!(
            "unsupported original Cargo configuration section: {}",
            path.display()
        )));
    }
    let empty = serde_json::Map::new();
    let targets = match config.get("target") {
        None => &empty,
        Some(v) => v.as_object().ok_or_else(|| {
            Error::new(format!(
                "invalid Cargo target configuration: {}",
                path.display()
            ))
        })?,
    };
    for (target, settings) in targets {
        let settings = settings.as_object().ok_or_else(|| {
            Error::new(format!(
                "invalid Cargo target table: {}: {target}",
                path.display()
            ))
        })?;
        if !KNOWN_TARGETS.contains(&target.as_str()) {
            return Err(Error::new(format!(
                "unaudited Cargo target selector: {}: {target}",
                path.display()
            )));
        }
        if Some(target.as_str()) == requested {
            if settings
                .keys()
                .any(|k| k != "linker" && !(k == "rustflags" && rustflags_set))
                || settings.get("linker").is_some_and(|v| !v.is_string())
            {
                return Err(Error::new(format!(
                    "unsupported Cargo target configuration for {target}: {}",
                    path.display()
                )));
            }
            continue;
        }
        if settings
            .keys()
            .any(|k| !["linker", "runner", "rustflags", "rustdocflags"].contains(&k.as_str()))
        {
            return Err(Error::new(format!(
                "unaudited Cargo target configuration: {}: {target}",
                path.display()
            )));
        }
        for (key, value) in settings {
            if !value.is_string()
                && !(key != "linker"
                    && value
                        .as_array()
                        .is_some_and(|a| a.iter().all(Value::is_string)))
            {
                return Err(Error::new(format!(
                    "invalid Cargo target setting: {}: {target}.{key}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}
pub fn audit(
    repo: &Path,
    cwds: &[PathBuf],
    env: Option<&BTreeMap<String, String>>,
    broker: Option<&Path>,
    requested: Option<&str>,
    rustflags_set: bool,
    canonical: &Path,
) -> Result<Value> {
    let inherited: BTreeMap<_, _> = std::env::vars().collect();
    let env = env.unwrap_or(&inherited);
    environment(env, broker)?;
    let mut dirs = BTreeSet::new();
    for cwd in cwds {
        for p in cwd.ancestors() {
            dirs.insert(p.join(".cargo"));
        }
    }
    for p in canonical.ancestors().skip(1) {
        dirs.insert(p.join(".cargo"));
    }
    let home = env.get("CARGO_HOME").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(
            env.get("HOME")
                .cloned()
                .unwrap_or_else(|| std::env::var("HOME").unwrap_or_else(|_| "/".into())),
        )
        .join(".cargo")
    });
    if !home.is_absolute() {
        return Err("effective CARGO_HOME must be absolute for the pilot".into());
    }
    dirs.insert(home);
    let mut files = BTreeMap::new();
    let mut checked = Vec::new();
    for dir in dirs {
        if dir.is_symlink() {
            return Err(Error::new(format!(
                "symlinked Cargo configuration directory is unaudited: {}",
                dir.display()
            )));
        }
        for name in ["config", "config.toml"] {
            let path = dir.join(name);
            checked.push(path.to_string_lossy().into_owned());
            if !path.exists() && !path.is_symlink() {
                continue;
            }
            if path.is_symlink() {
                return Err(Error::new(format!(
                    "symlinked Cargo configuration is unaudited: {}",
                    path.display()
                )));
            }
            let payload = af::verified_read(&path, "Cargo configuration")?;
            let parsed = (|| -> Result<Value> {
                let text = std::str::from_utf8(&payload).map_err(|e| Error::new(e.to_string()))?;
                let t: toml::Value = toml::from_str(text).map_err(|e| Error::new(e.to_string()))?;
                Ok(serde_json::to_value(t)?)
            })();
            let parsed = parsed.map_err(|e| {
                Error::new(format!(
                    "invalid original Cargo configuration: {}: {e}",
                    path.display()
                ))
            })?;
            allowed(&parsed, &path, requested, rustflags_set)?;
            files.insert(
                path.to_string_lossy().into_owned(),
                json!({"sha256":format!("{:x}",Sha256::digest(&payload)),"bytes":payload.len()}),
            );
        }
    }
    let _ = repo;
    Ok(
        json!({"scope_directories":cwds.len(),"checked_paths":checked,"watches":files.keys().collect::<Vec<_>>(),"files":files}),
    )
}
