//! Compile-based detector for Rust match patterns that silently bind unresolved constants.
use crate::binary::{BinaryContext, CheckOutput};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const DEFAULT_TARGET: &str = "x86_64-pc-windows-gnu";
const FORCE_WARN: &[&str] = &[
    "--force-warn",
    "unreachable_patterns",
    "--force-warn",
    "unused_variables",
    "--force-warn",
    "non_snake_case",
];
const ARM64_EXCLUDES: &[&str] = &["kdvm-cargo", "hal-rust", "rtl-difftest", "cnt-difftest"];
fn regex(pattern: &str) -> &'static regex::Regex {
    static MAP: OnceLock<
        std::sync::Mutex<std::collections::BTreeMap<String, &'static regex::Regex>>,
    > = OnceLock::new();
    let mut map = MAP
        .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
        .lock()
        .unwrap();
    *map.entry(pattern.to_owned())
        .or_insert_with(|| Box::leak(Box::new(regex::Regex::new(pattern).unwrap())))
}

fn primary_span(diag: &Value) -> String {
    diag.get("spans")
        .and_then(Value::as_array)
        .and_then(|spans| {
            spans
                .iter()
                .find(|s| s.get("is_primary").and_then(Value::as_bool) == Some(true))
        })
        .and_then(|s| {
            Some(format!(
                "{}:{}",
                s.get("file_name")?.as_str()?,
                s.get("line_start")?.as_u64()?
            ))
        })
        .unwrap_or_else(|| "?:?".into())
}
fn const_like(name: &str) -> bool {
    regex(r"^[A-Z][A-Z0-9_]{2,}$").is_match(name)
}
fn is_match_arm(workspace: &Path, location: &str) -> bool {
    let Some((filename, line)) = location.rsplit_once(':') else {
        return true;
    };
    let Ok(line) = line.parse::<usize>() else {
        return true;
    };
    if line == 0 {
        return true;
    }
    let path = if Path::new(filename).is_absolute() {
        PathBuf::from(filename)
    } else {
        workspace.join(filename)
    };
    let Ok(bytes) = std::fs::read(path) else {
        return true;
    };
    let contents = String::from_utf8_lossy(&bytes);
    contents
        .lines()
        .nth(line - 1)
        .map_or(true, |text| text.contains("=>"))
}
fn warning_name(message: &str, pattern: &str) -> Option<String> {
    regex(pattern).captures(message).map(|c| c[1].to_owned())
}

fn cargo_command(ctx: &BinaryContext, workspace: &Path) -> Result<std::process::Output, String> {
    let target = if ctx.target.is_empty() {
        DEFAULT_TARGET
    } else {
        ctx.target.as_str()
    };
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace)
        .arg("check")
        .arg("--release")
        .arg("--message-format=json")
        .arg("--target")
        .arg(target)
        .arg("--manifest-path")
        .arg(workspace.join("Cargo.toml"));
    if ctx.packages.is_empty() {
        cmd.arg("--workspace");
        if target == "aarch64-pc-windows-gnullvm" {
            for p in ARM64_EXCLUDES {
                cmd.arg("--exclude").arg(p);
            }
        }
    } else {
        for p in &ctx.packages {
            cmd.arg("-p").arg(p);
        }
    }
    let mut rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    if !rustflags.is_empty() {
        rustflags.push(' ');
    }
    rustflags.push_str(&FORCE_WARN.join(" "));
    let target_dir = std::env::var_os("SCAN_BINDALL_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("scan-bind-all-target"));
    cmd.env("RUSTFLAGS", rustflags)
        .env("CARGO_TARGET_DIR", target_dir);
    // This is a stable-toolchain lint. Never inherit or recreate the Python port's
    // RUSTC_BOOTSTRAP=1 escape hatch; remove it from cargo and all its children.
    cmd.env_remove("RUSTC_BOOTSTRAP");
    cmd.output()
        .map_err(|e| format!("scan_bind_all_matches: cannot run cargo check: {e}"))
}
fn findings_from_json(workspace: &Path, stdout: &str) -> Vec<String> {
    let unused = r"unused variable: `([^`]+)`";
    let snake = r"variable `([^`]+)` should have a snake case name";
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    for line in stdout.lines() {
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if msg.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let diag = &msg["message"];
        let code = diag
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let text = diag.get("message").and_then(Value::as_str).unwrap_or("");
        let loc = primary_span(diag);
        if loc.contains("/.cargo/registry/") {
            continue;
        }
        let finding = if code == "unreachable_patterns" {
            Some(format!("{loc}: [UNREACHABLE] {text}"))
        } else if code == "unused_variables" {
            warning_name(text,unused).filter(|n|const_like(n)&&is_match_arm(workspace,&loc))
                .map(|name|format!("{loc}: [BIND-ALL?] pattern binds `{name}` (const-like name, unused) — likely unresolved constant"))
        } else if code == "non_snake_case" {
            warning_name(text,snake).filter(|n|const_like(n)&&is_match_arm(workspace,&loc))
                .map(|name|format!("{loc}: [CONST-BINDING] variable `{name}` is SCREAMING_CASE — check for unresolved constant in pattern"))
        } else {
            None
        };
        if let Some(f) = finding {
            if seen.insert(f.clone()) {
                findings.push(f);
            }
        }
    }
    findings
}
fn run_inner(ctx: &BinaryContext, quiet: bool) -> Result<CheckOutput, String> {
    let workspace = ctx.root.join("sdk/lib/rust");
    let manifest = workspace.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(format!(
            "scan_bind_all_matches: missing Rust workspace manifest {}",
            manifest.display()
        ));
    }
    let target = if ctx.target.is_empty() {
        DEFAULT_TARGET
    } else {
        ctx.target.as_str()
    };
    let output = cargo_command(ctx, &workspace)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail = stderr
            .lines()
            .rev()
            .take(15)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        // The old script reported interesting warnings and returned 1 even when cargo
        // later failed. That is incomplete coverage; fail hard rather than publish a
        // partial diagnostic set as a scan result.
        return Err(format!(
            "ERROR: cargo check failed (target {target}); scan coverage is incomplete:\n{tail}"
        ));
    }
    let findings = findings_from_json(&workspace, &String::from_utf8_lossy(&output.stdout));
    let mut stdout = String::new();
    if !quiet {
        for f in &findings {
            stdout.push_str(f);
            stdout.push('\n');
        }
    }
    let stderr = format!("\n{} finding(s) [{target}]\n", findings.len());
    Ok(CheckOutput {
        stdout,
        stderr,
        returncode: if findings.is_empty() { 0 } else { 1 },
    })
}
/// Native gate behavior: compile the selected target/package scope and fail on findings.
pub fn run(ctx: &BinaryContext) -> Result<CheckOutput, String> {
    run_inner(ctx, false)
}
/// Optional CLI variant preserving `-q/--quiet` output behavior.
pub fn run_with_quiet(ctx: &BinaryContext, quiet: bool) -> Result<CheckOutput, String> {
    run_inner(ctx, quiet)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn diagnostic(code: &str, message: &str, file: &str, line: u64) -> String {
        serde_json::json!({"reason":"compiler-message","message":{"code":{"code":code},"message":message,"spans":[{"is_primary":true,"file_name":file,"line_start":line}]}}).to_string()
    }
    #[test]
    fn const_name_warnings_are_kept_only_on_match_arms() {
        let d = std::env::temp_dir().join(format!("bind-all-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("x.rs"),"fn f(x:u32) { match x { IRP_MN_QUERY_ID => {}, other => {} } }\nfn g(IRP_MN_UNUSED: u32) {}\n").unwrap();
        let input = [
            diagnostic(
                "unused_variables",
                "unused variable: `IRP_MN_QUERY_ID`",
                "x.rs",
                1,
            ),
            diagnostic(
                "non_snake_case",
                "variable `IRP_MN_QUERY_ID` should have a snake case name",
                "x.rs",
                1,
            ),
            diagnostic(
                "unused_variables",
                "unused variable: `IRP_MN_UNUSED`",
                "x.rs",
                2,
            ),
            diagnostic("unused_variables", "unused variable: `ordinary`", "x.rs", 1),
        ]
        .join("\n");
        let got = findings_from_json(&d, &input);
        assert_eq!(got.len(), 2);
        assert!(got[0].contains("[BIND-ALL?]"));
        assert!(got[1].contains("[CONST-BINDING]"));
        let _ = std::fs::remove_dir_all(d);
    }
    #[test]
    fn unreachable_warnings_are_kept_but_registry_noise_is_skipped() {
        let d = std::env::temp_dir();
        let input = [
            diagnostic(
                "unreachable_patterns",
                "unreachable pattern",
                "src/lib.rs",
                12,
            ),
            diagnostic(
                "unreachable_patterns",
                "external",
                "/x/.cargo/registry/src/pkg/lib.rs",
                3,
            ),
            "not json".into(),
        ]
        .join("\n");
        let got = findings_from_json(&d, &input);
        assert_eq!(got.len(), 1);
        assert!(got[0].contains("[UNREACHABLE]"));
    }
    #[test]
    fn missing_source_line_is_kept_for_review() {
        assert!(is_match_arm(Path::new("/no/such/workspace"), "gone.rs:10"));
    }
}
