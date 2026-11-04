//! Tracked historical Cargo path closures, without changing a checkout.
use crate::{Error, Result};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    use std::{
        io::Read,
        process::Stdio,
        time::{Duration, Instant},
    };
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().ok_or("missing git stdout")?;
    let mut stderr = child.stderr.take().ok_or("missing git stderr")?;
    let out = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let err = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let started = Instant::now();
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (status, false);
        }
        if started.elapsed() >= Duration::from_secs(120) {
            let _ = child.kill();
            break (child.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let stdout = out
        .join()
        .map_err(|_| Error::new("git stdout reader failed"))??;
    let stderr = err
        .join()
        .map_err(|_| Error::new("git stderr reader failed"))??;
    if timed_out {
        return Err("git historical read timed out after 120s".into());
    }
    if !status.success() {
        return Err(Error::new(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&stderr).trim()
        )));
    }
    String::from_utf8(stdout)
        .map_err(|e| Error::new(format!("historical Git output is not UTF-8: {e}")))
}

fn normalized(path: &Path) -> Result<PathBuf> {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err("historical dependency leaves Rust workspace".into());
                }
            }
            _ => return Err("historical dependency leaves Rust workspace".into()),
        }
    }
    if !out.starts_with("sdk/lib/rust") || out == Path::new("sdk/lib/rust") {
        return Err("historical dependency leaves Rust workspace".into());
    }
    Ok(out)
}
/// Extract only the tracked selected package and its normal path dependency closure.
pub fn capture(repo: &Path, revision: &str, body: &str, destination: &Path) -> Result<String> {
    let commit = git(
        repo,
        &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
    )?
    .trim()
    .to_owned();
    fn visit(
        repo: &Path,
        commit: &str,
        path: &Path,
        packages: &mut BTreeSet<PathBuf>,
    ) -> Result<()> {
        let path = normalized(path)?;
        if !packages.insert(path.clone()) {
            return Ok(());
        }
        let text = git(
            repo,
            &["show", &format!("{commit}:{}/Cargo.toml", path.display())],
        )?;
        let manifest: toml::Value = toml::from_str(&text).map_err(|e| Error::new(e.to_string()))?;
        if let Some(deps) = manifest.get("dependencies") {
            let deps = deps
                .as_table()
                .ok_or("historical dependencies must be a table")?;
            for value in deps.values() {
                if let Some(child) = value.get("path") {
                    let child = child
                        .as_str()
                        .ok_or("historical dependency path must be a string")?;
                    visit(repo, commit, &path.join(child), packages)?;
                }
            }
        }
        Ok(())
    }
    let mut packages = BTreeSet::new();
    visit(
        repo,
        &commit,
        &Path::new("sdk/lib/rust").join(body),
        &mut packages,
    )?;
    let temporary = tempfile::Builder::new()
        .prefix("artifact-history-")
        .tempdir()?;
    let archive = temporary.path().join("tracked.tar");
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(["archive", "--format=tar", "--output"])
        .arg(&archive)
        .arg(&commit)
        .arg("--")
        .args(["sdk/lib/rust/Cargo.toml", "sdk/lib/rust/Cargo.lock"])
        .args(&packages);
    let result = crate::process::capture(&mut command, 120)?;
    if result.code != 0 {
        return Err(Error::new(format!(
            "git archive failed: {}",
            result.stderr.trim()
        )));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(destination)?;
    let mut stream = tar::Archive::new(fs::File::open(archive)?);
    for member in stream.entries()? {
        let mut member = member?;
        let relative = member.path()?.into_owned();
        if relative.is_absolute()
            || relative.components().any(|p| {
                matches!(
                    p,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err("unsafe historical archive path".into());
        }
        let target = destination.join(relative);
        let kind = member.header().entry_type();
        if kind.is_pax_global_extensions() {
            continue;
        }
        if kind.is_dir() {
            fs::create_dir_all(target)?;
        } else if kind.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)?;
            std::io::copy(&mut member, &mut output)?;
        } else {
            return Err("historical source symlinks/special files unsupported".into());
        }
    }
    Ok(commit)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("sdk/lib/rust");
        fs::create_dir_all(base.join("body/src")).unwrap();
        fs::create_dir_all(base.join("dep/src")).unwrap();
        fs::write(
            base.join("Cargo.toml"),
            "[workspace]\nmembers=[\"body\",\"dep\"]\n",
        )
        .unwrap();
        fs::write(base.join("Cargo.lock"), "version=4\n").unwrap();
        fs::write(
            base.join("body/Cargo.toml"),
            "[package]\nname=\"body\"\nversion=\"0.1.0\"\n[dependencies]\ndep={path=\"../dep\"}\n",
        )
        .unwrap();
        fs::write(
            base.join("dep/Cargo.toml"),
            "[package]\nname=\"dep\"\nversion=\"0.1.0\"\n",
        )
        .unwrap();
        fs::write(base.join("body/src/lib.rs"), "pub fn body(){}\n").unwrap();
        fs::write(base.join("dep/src/lib.rs"), "pub fn dep(){}\n").unwrap();
        git(root.path(), &["init", "-q"]).unwrap();
        git(root.path(), &["add", "."]).unwrap();
        git(
            root.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ],
        )
        .unwrap();
        root
    }
    #[test]
    fn historical_capture_uses_committed_closure_and_refuses_existing_destination() {
        let root = setup();
        fs::write(root.path().join("sdk/lib/rust/dep/src/lib.rs"), "dirty\n").unwrap();
        let dest = root.path().join("capture");
        let commit = capture(root.path(), "HEAD", "body", &dest).unwrap();
        assert_eq!(commit.len(), 40);
        assert_eq!(
            fs::read_to_string(dest.join("sdk/lib/rust/dep/src/lib.rs")).unwrap(),
            "pub fn dep(){}\n"
        );
        assert!(capture(root.path(), "HEAD", "body", &dest).is_err());
        assert!(capture(
            root.path(),
            "HEAD",
            "../../../escape",
            &root.path().join("bad")
        )
        .is_err());
    }
    #[test]
    fn invalid_utf8_manifest_is_refused_without_a_partial_destination() {
        let root = setup();
        fs::write(
            root.path().join("sdk/lib/rust/body/Cargo.toml"),
            b"[package]\nname=\"\xff\"\n",
        )
        .unwrap();
        git(root.path(), &["add", "."]).unwrap();
        git(
            root.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "invalid manifest",
            ],
        )
        .unwrap();
        let destination = root.path().join("capture");
        let error = capture(root.path(), "HEAD", "body", &destination).unwrap_err();
        assert!(error.message.contains("not UTF-8"));
        assert!(!destination.exists());
    }
    #[test]
    fn historical_symlinks_are_not_extracted() {
        let root = setup();
        std::os::unix::fs::symlink("/etc/passwd", root.path().join("sdk/lib/rust/body/link"))
            .unwrap();
        git(root.path(), &["add", "."]).unwrap();
        git(
            root.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "link",
            ],
        )
        .unwrap();
        let error = capture(root.path(), "HEAD", "body", &root.path().join("capture")).unwrap_err();
        assert!(error.message.contains("symlinks/special files"));
    }
}
