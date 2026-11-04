//! Capture a verified source projection independently of the mutable checkout.
use crate::{artifact_fs as af, artifact_inputs as inputs, Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Instant,
};
#[derive(Default)]
pub struct CaptureOptions {
    pub expected_projection_key: Option<String>,
    pub discovery: inputs::DiscoverOptions,
    pub additional_inputs: serde_json::Map<String, Value>,
}
pub fn verified_copy(source: &Path, output: &Path, expected: &Value) -> Result<u64> {
    let payload = af::verified_read(source, "snapshot input")?;
    if expected["bytes"].as_u64() != Some(payload.len() as u64)
        || expected["sha256"].as_str() != Some(&format!("{:x}", Sha256::digest(&payload)))
    {
        return Err(Error::new(format!(
            "snapshot input differs from discovery: {}",
            source.display()
        )));
    }
    fs::create_dir_all(
        output
            .parent()
            .ok_or("snapshot destination has no parent")?,
    )?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    f.write_all(&payload)?;
    drop(f);
    if Sha256::digest(fs::read(output)?).as_slice() != Sha256::digest(&payload).as_slice() {
        return Err(Error::new(format!(
            "snapshot destination verification failed: {}",
            output.display()
        )));
    }
    Ok(payload.len() as u64)
}
pub fn capture(
    repo: &Path,
    body: &str,
    context: Value,
    destination: &Path,
    options: &CaptureOptions,
) -> Result<Value> {
    capture_with(repo, body, context, destination, options, |_| Ok(()))
}
fn capture_with<F: FnMut(&Path) -> Result<()>>(
    repo: &Path,
    body: &str,
    context: Value,
    destination: &Path,
    options: &CaptureOptions,
    mut after_copy: F,
) -> Result<Value> {
    let repo = af::resolve(repo)?;
    let destination = std::path::absolute(destination)?;
    if af::resolve(&destination)?.starts_with(&repo) {
        return Err("snapshot destination must be outside the live source repository".into());
    }
    let start = Instant::now();
    let discovery = inputs::discover(&repo, body, context.clone(), &options.discovery);
    if discovery["eligible"] != true {
        return Err(Error::new(format!(
            "source discovery ineligible: {}",
            discovery["reasons"]
                .as_array()
                .map(|r| r
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("; "))
                .unwrap_or_default()
        )));
    }
    if options
        .expected_projection_key
        .as_ref()
        .is_some_and(|key| discovery["source_projection_key"].as_str() != Some(key))
    {
        return Err("source projection changed before source capture".into());
    }
    let mut reads = discovery["provenance"]["reads"]
        .as_object()
        .ok_or("discovery reads missing")?
        .clone();
    for (relative, entry) in &options.additional_inputs {
        if reads.get(relative).is_some_and(|v| v != entry) {
            return Err(Error::new(format!(
                "additional input conflicts with discovery: {relative}"
            )));
        }
        reads.insert(relative.clone(), entry.clone());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(&destination)?;
    let mut copied = 0;
    for (relative, entry) in &reads {
        let path = Path::new(relative);
        if !af::safe_relative(path) {
            return Err(Error::new(format!(
                "invalid snapshot logical path: {relative}"
            )));
        }
        let source = repo.join(path);
        if !af::resolve(&source)?.starts_with(&repo) {
            return Err(Error::new(format!(
                "snapshot source escapes repository: {relative}"
            )));
        }
        copied += verified_copy(&source, &destination.join(path), entry)?;
        after_copy(&source)?;
    }
    let packages = discovery["descriptor"]["packages"]
        .as_object()
        .ok_or("discovery packages missing")?;
    for (root, package) in packages {
        for directory in package["inventory"]["directories"]
            .as_array()
            .ok_or("discovery directories missing")?
        {
            let relative = PathBuf::from(root).join(directory.as_str().ok_or("invalid directory")?);
            if !af::safe_relative(&relative) {
                return Err("invalid snapshot directory".into());
            }
            fs::create_dir_all(destination.join(relative))?;
        }
    }
    let mut snapshot_options = options.discovery.clone();
    if let Some(workspace) = &snapshot_options.workspace {
        snapshot_options.workspace = Some(
            destination.join(
                af::resolve(workspace)?
                    .strip_prefix(&repo)
                    .map_err(|_| Error::new("workspace escapes repository"))?,
            ),
        );
    }
    let captured = inputs::discover(&destination, body, context.clone(), &snapshot_options);
    if captured["eligible"] != true
        || captured["source_projection_key"] != discovery["source_projection_key"]
    {
        return Err("copied source projection does not match the discovered byte inventory".into());
    }
    let current = inputs::discover(&repo, body, context, &options.discovery);
    if current["eligible"] != true
        || current["source_projection_key"] != captured["source_projection_key"]
        || current["provenance"]["reads"] != discovery["provenance"]["reads"]
    {
        return Err("source input inventory changed during source capture".into());
    }
    let report = json!({"schema":"vibeos-body-source-capture-v1-spike","source_projection_key":captured["source_projection_key"],"action_key":captured["action_key"],"checkout_consistency":"captured bytes only; original checkout not claimed atomic","copied_files":reads.len(),"copied_bytes":copied,"reads":reads,"source_manifest_policy":"raw manifests preserved; semantic projection rederived","execution_status":"source capture only; compiler isolation and ABI command unproven","elapsed_seconds":start.elapsed().as_secs_f64()});
    fs::write(
        destination.join("artifact-source-capture.json"),
        inputs::canonical(&report)?,
    )?;
    fn readonly(path: &Path) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if entry.file_type()?.is_dir() {
                readonly(&p)?;
            } else {
                fs::set_permissions(&p, fs::Permissions::from_mode(0o444))?;
            }
        }
        fs::set_permissions(path, fs::Permissions::from_mode(0o555))?;
        Ok(())
    }
    readonly(&destination)?;
    Ok(report)
}

pub fn executor_requirements() -> Value {
    serde_json::from_str(include_str!("artifact_requirements.json"))
        .expect("static requirements JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path) {
        fs::create_dir_all(root.join("body/src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nresolver=\"2\"\nmembers=[\"body\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("Cargo.lock"),
            "version=4\n[[package]]\nname=\"body\"\nversion=\"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            root.join("body/Cargo.toml"),
            "[package]\nname=\"body\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        )
        .unwrap();
        fs::write(root.join("body/src/lib.rs"), "pub fn value() {}\n").unwrap();
    }
    #[test]
    fn capture_rejects_membership_changes() {
        for empty_directory in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let repo = temp.path().join("repo");
            fixture(&repo);
            let options = CaptureOptions {
                discovery: inputs::DiscoverOptions {
                    workspace: Some(repo.clone()),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut changed = false;
            let error = capture_with(
                &repo,
                "body",
                json!({}),
                &temp.path().join("captured"),
                &options,
                |_| {
                    if !changed {
                        let path = repo.join("body/added-during-capture");
                        if empty_directory {
                            fs::create_dir(path)?;
                        } else {
                            fs::write(path, "membership changed")?;
                        }
                        changed = true;
                    }
                    Ok(())
                },
            )
            .unwrap_err();
            assert!(error.message.contains("inventory changed"), "{error}");
        }
    }
    #[test]
    fn verified_copy_refuses_mismatched_bytes_and_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let output = temp.path().join("output");
        fs::write(&source, "actual").unwrap();
        assert!(verified_copy(&source, &output, &json!({"bytes":5,"sha256":"wrong"})).is_err());
        assert!(!output.exists());
        let link = temp.path().join("link");
        std::os::unix::fs::symlink(&source, &link).unwrap();
        assert!(verified_copy(
            &link,
            &output,
            &json!({"bytes":6,"sha256":format!("{:x}",Sha256::digest(b"actual"))})
        )
        .is_err());
        assert!(!output.exists());
    }
}
