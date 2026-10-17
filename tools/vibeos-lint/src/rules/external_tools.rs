//! Checks whose authority is an external syntax checker, driven by the Rust runner.
use crate::{Corpus, Rule};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct Shellcheck;
pub struct PythonSyntax;

fn checked_output(mut command: Command, label: &str) -> Result<Vec<String>, String> {
    let result = command.output().map_err(|e| format!("{label}: {e}"))?;
    let stdout = String::from_utf8_lossy(&result.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.stderr);
    if !result.status.success() && stdout.trim().is_empty() {
        return Err(format!("{label} exited {}: {stderr}", result.status));
    }
    if !result.status.success() && label == "shellcheck" && result.status.code() != Some(1) {
        return Err(format!(
            "{label} exited {}: {stdout}{stderr}",
            result.status
        ));
    }
    Ok(vec![stdout])
}
fn scripts(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for dir in ["qemu", "qemu/lib"] {
        let path = root.join(dir);
        if !path.is_dir() {
            continue;
        }
        let mut files = std::fs::read_dir(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        files.retain(|p| p.extension().is_some_and(|e| e == "sh"));
        files.sort();
        paths.extend(files);
    }
    Ok(paths)
}
impl Rule for Shellcheck {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "shellcheck"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let files = scripts(&c.root)?;
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut command = Command::new(
            std::env::var_os("VIBEOS_LINT_SHELLCHECK").unwrap_or_else(|| "shellcheck".into()),
        );
        command
            .current_dir(&c.root)
            .args(["-S", "warning", "-f", "gcc"])
            .args(files);
        checked_output(command, "shellcheck")
    }
}
impl Rule for PythonSyntax {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "py-syntax"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut command = Command::new(
            std::env::var_os("VIBEOS_LINT_PYTHON").unwrap_or_else(|| "python3".into()),
        );
        command
            .current_dir(&c.root)
            .args(["-m", "compileall", "-q", "tests/qemu", "tools"]);
        let result = command.output().map_err(|e| format!("compileall: {e}"))?;
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        if !result.status.success() && text.trim().is_empty() {
            return Err(format!(
                "compileall exited {} without diagnostics",
                result.status
            ));
        }
        Ok(vec![text])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shellcheck_errors_are_not_findings_or_clean_passes() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "printf 'unavailable\\n' >&2; exit 2"]);
        assert!(checked_output(cmd, "shellcheck").is_err());
        let mut cmd = Command::new("sh");
        cmd.args([
            "-c",
            "printf 'fixture.sh:1:1: warning: bad [SC0000]\\n'; exit 1",
        ]);
        assert!(checked_output(cmd, "shellcheck").unwrap()[0].contains("SC0000"));
    }
}
