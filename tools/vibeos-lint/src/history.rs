//! Git facts captured centrally before rule execution.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Default)]
pub struct VersionHistory {
    pub ever_versioned_cmake: BTreeSet<PathBuf>,
    pub last_parent_text: BTreeMap<PathBuf, String>,
    pub last_parent_revision: BTreeMap<PathBuf, String>,
}
fn git(root: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("version history: git: {e}"))
}
impl VersionHistory {
    pub(crate) fn capture(root: &Path) -> Result<Self, String> {
        let paths = git(
            root,
            &[
                "log",
                "--format=",
                "--name-only",
                "-S_WIN32_WINNT",
                "--",
                "*CMakeLists.txt",
            ],
        )?;
        if !paths.status.success() {
            return Err(format!(
                "version history: {}",
                String::from_utf8_lossy(&paths.stderr)
            ));
        }
        let mut out = Self::default();
        out.ever_versioned_cmake = String::from_utf8_lossy(&paths.stdout)
            .split_whitespace()
            .map(PathBuf::from)
            .collect();
        for path in &out.ever_versioned_cmake {
            let p = path
                .to_str()
                .ok_or("version history: non-UTF-8 CMake path")?;
            let last = git(
                root,
                &["log", "-1", "--format=%h", "-S_WIN32_WINNT", "--", p],
            )?;
            if !last.status.success() {
                return Err(format!(
                    "version history: {}",
                    String::from_utf8_lossy(&last.stderr)
                ));
            }
            let commit = String::from_utf8_lossy(&last.stdout).trim().to_owned();
            if commit.is_empty() {
                continue;
            }
            let revision = format!("{commit}^");
            let old = git(root, &["show", &format!("{revision}:{p}")])?;
            // A newly created file has no copy in the parent. The old scanner
            // treats this as no historical override, not an empty source file.
            if old.status.success() {
                out.last_parent_text.insert(
                    path.clone(),
                    String::from_utf8_lossy(&old.stdout).into_owned(),
                );
                out.last_parent_revision.insert(path.clone(), revision);
            }
        }
        Ok(out)
    }
}
