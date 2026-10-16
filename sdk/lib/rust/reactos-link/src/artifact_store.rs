//! Immutable body archive consumer. The build supervisor owns the lease through
//! the outer link; this short-lived build script never releases it.
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The builtin triple each architecture's eligible bodies build on. A body on a custom
/// target JSON (the i386 kernel's `i686-kernel`) needs build-std, which the store's
/// worker recipe does not model, so it keeps the ordinary path.
fn builtin_target(arch: &str) -> Option<&'static str> {
    match arch {
        "amd64" => Some("x86_64-pc-windows-gnu"),
        "i386" => Some("i686-pc-windows-gnu"),
        "arm64" => Some("aarch64-pc-windows-gnullvm"),
        _ => None,
    }
}

/// The exact RUSTFLAGS the worker's recipe compiles an eligible body with, by arch and
/// kernel mode: tools/artifact_executor.py FLAGS, which tools/test_artifact_config.py
/// holds this copy to. A body built with anything else keeps the ordinary path.
fn audited_flags(arch: &str, kernel: bool) -> Option<&'static str> {
    Some(match (arch, kernel) {
        ("amd64" | "arm64", false) => "",
        ("amd64" | "arm64", true) => "-C force-frame-pointers=yes",
        ("i386", false) => "-C target-feature=-sse,-sse2",
        ("i386", true) => "-C target-feature=-sse,-sse2 -C force-frame-pointers=yes",
        _ => return None,
    })
}

/// The helper's answer: a leased archive with its watches, or that the store does not
/// model this body's graph (a build script, a registry crate, ...), which then builds
/// the ordinary way.
#[derive(Debug, PartialEq)]
enum Response {
    Archive(PathBuf, Vec<PathBuf>),
    Ineligible(String),
}

fn response(text: &str) -> Result<Response, String> {
    if let Some(reason) = text.strip_prefix("ineligible\t") {
        let reason = reason.strip_suffix('\n').ok_or("unterminated ineligible response")?;
        if reason.is_empty() || reason.contains(['\t', '\r', '\n', '\0']) {
            return Err("invalid ineligible response".into());
        }
        return Ok(Response::Ineligible(reason.to_string()));
    }
    let mut archive = None;
    let mut watches = Vec::new();
    for line in text.lines() {
        let (kind, value) = line.split_once('\t').ok_or("invalid artifact helper response")?;
        if value.contains(['\t', '\r', '\0']) || !Path::new(value).is_absolute() {
            return Err("artifact helper returned an invalid path".into());
        }
        match kind {
            "archive" if archive.is_none() => archive = Some(PathBuf::from(value)),
            "watch" => watches.push(PathBuf::from(value)),
            _ => return Err("unknown or duplicate artifact response field".into()),
        }
    }
    let archive = archive.ok_or("artifact helper returned no archive")?;
    if watches.is_empty() { return Err("artifact helper returned no dependency watches".into()); }
    Ok(Response::Archive(archive, watches))
}

pub fn get(
    repo: &Path, package: &str, body: &str, arch: &str, target: &str,
    features: &[&str], default_features: bool, kernel: bool, staticlib: bool,
    flags: &str, other_flags: bool, cargo_cwd: &Path,
) -> Option<PathBuf> {
    println!("cargo:rerun-if-env-changed=VIBEOS_ARTIFACT_STORE_MODE");
    println!("cargo:rerun-if-env-changed=VIBEOS_ARTIFACT_STORE_ROOT");
    println!("cargo:rerun-if-env-changed=VIBEOS_ARTIFACT_VERIFY_NONCE");
    println!("cargo:rerun-if-env-changed=VIBEOS_ARTIFACT_TOOLCHAIN_ID");
    match env::var("VIBEOS_ARTIFACT_STORE_MODE").as_deref() {
        Err(_) | Ok("OFF") | Ok("") => return None,
        Ok("ON") => {},
        _ => panic!("VIBEOS_ARTIFACT_STORE_MODE must be ON or OFF"),
    }
    // Every body goes to the store when its build is one the worker's recipe models;
    // the helper's discovery then decides whether its graph is. The rest (a custom
    // target JSON, other flags, a crate-type override, IR emission) build privately.
    if builtin_target(arch) != Some(target)
        || audited_flags(arch, kernel) != Some(flags.trim())
        || staticlib
        || other_flags
        || env::var("VIBEOS_EMIT_LLVM_IR").as_deref() == Ok("1")
    {
        return None;
    }
    // The store is on by default, so a raw `ninja` (canon-run.sh) reaches here with
    // no vibeos-cli supervisor to own the lease: build this body the ordinary way.
    let Some(supervisor) = env::var_os("VIBEOS_ARTIFACT_SUPERVISOR") else {
        println!("cargo:warning=artifact store: no vibeos-cli build supervisor; building {body} privately");
        return None;
    };
    let python = env::var_os("VIBEOS_ARTIFACT_PYTHON")
        .expect("artifact store supervisor did not supply its Python interpreter");
    let consumer = format!("cargo:{package}|{}|{body}|{features:?}|{default_features}|{kernel}",
        env::var("OUT_DIR").expect("OUT_DIR"));
    let mut command = Command::new(python);
    command.current_dir(repo).args(["-m", "tools.artifact_runtime", "request"])
        .arg("--supervisor").arg(supervisor)
        .arg("--cargo-cwd").arg(cargo_cwd)
        .arg("--body").arg(body).arg("--consumer").arg(consumer)
        .arg("--features").arg(features.join(","))
        .arg(format!("--flags={flags}"))
        .arg("--arch").arg(arch)
        .arg("--target").arg(target);
    if !default_features { command.arg("--no-default-features"); }
    if kernel { command.arg("--kernel"); }
    let output = command.output().expect("launch immutable artifact helper");
    assert!(output.status.success(), "immutable artifact helper failed: {}",
        String::from_utf8_lossy(&output.stderr));
    let text = std::str::from_utf8(&output.stdout).expect("artifact helper emitted non-UTF8 response");
    let (archive, watches) = match response(text).unwrap_or_else(|error| panic!("{error}")) {
        Response::Archive(archive, watches) => (archive, watches),
        Response::Ineligible(_) => return None,
    };
    assert!(archive.is_file(), "artifact helper returned a missing archive: {}", archive.display());
    for watch in watches { println!("cargo:rerun-if-changed={}", watch.display()); }
    Some(archive)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incomplete_or_ambiguous_responses() {
        for text in ["", "archive\t/a\n", "watch\t/a\n", "archive\ta\nwatch\t/b\n",
            "archive\t/a\narchive\t/b\nwatch\t/c\n", "archive\t/a\nwatch\t/b\tx\n"] {
            assert!(response(text).is_err(), "{text:?}");
        }
        assert_eq!(
            response("archive\t/store/blob\nwatch\t/source/dir\n").unwrap(),
            Response::Archive(PathBuf::from("/store/blob"), vec![PathBuf::from("/source/dir")])
        );
        assert_eq!(
            response("ineligible\tbuild script unsupported\n").unwrap(),
            Response::Ineligible("build script unsupported".into())
        );
        for text in ["ineligible\t\n", "ineligible\tno newline", "ineligible\ta\tb\n"] {
            assert!(response(text).is_err(), "{text:?}");
        }
    }
}
