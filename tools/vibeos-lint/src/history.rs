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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_override_uses_the_last_change_parent_and_handles_new_files() {
        let root = std::env::temp_dir().join(format!("vibeos-lint-history-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let run = |args: &[&str]| {
            let o = git(&root, args).unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            o
        };
        run(&["init", "-q", "-b", "history-fixture"]);
        run(&["config", "user.name", "lint fixture"]);
        run(&["config", "user.email", "lint@example.invalid"]);
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_definitions(-D_WIN32_WINNT=0x502)\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("new")).unwrap();
        std::fs::write(
            root.join("new/CMakeLists.txt"),
            "add_definitions(-D_WIN32_WINNT=0x600)\n",
        )
        .unwrap();
        run(&["add", "CMakeLists.txt", "new/CMakeLists.txt"]);
        run(&["commit", "-qm", "initial version profiles"]);
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_cargo_module(example example-cargo example.dll)\n",
        )
        .unwrap();
        run(&["add", "CMakeLists.txt"]);
        run(&["commit", "-qm", "replace C link"]);
        let snapshot = VersionHistory::capture(&root).unwrap();
        assert!(snapshot
            .ever_versioned_cmake
            .contains(Path::new("CMakeLists.txt")));
        assert!(snapshot
            .ever_versioned_cmake
            .contains(Path::new("new/CMakeLists.txt")));
        assert_eq!(
            snapshot.last_parent_text[Path::new("CMakeLists.txt")],
            "add_definitions(-D_WIN32_WINNT=0x502)\n"
        );
        assert!(snapshot.last_parent_revision[Path::new("CMakeLists.txt")].ends_with('^'));
        assert!(!snapshot
            .last_parent_text
            .contains_key(Path::new("new/CMakeLists.txt")));
    }
}
