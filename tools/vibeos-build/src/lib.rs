//! Native build mechanisms, CLI composition and reporting; host-jobs owns admission.
use std::{
    fmt,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, serde::Serialize)]
pub struct Error {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}
impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: None,
        }
    }
}
impl Error {
    pub fn classified(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: Some(kind.into()),
        }
    }
}
impl Error {
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(e.to_string())
    }
}
impl From<regex::Error> for Error {
    fn from(e: regex::Error) -> Self {
        Self::new(e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub const ARCHES: [&str; 3] = ["i386", "amd64", "arm64"];
/// Copy a git index to a scratch GIT_INDEX_FILE keeping its mtime. git treats an entry whose
/// file mtime is not older than the index as "racily clean" and re-hashes it; a plain copy
/// stamps the scratch index NOW, so a same-size edit made in the same second as the last
/// `git add` looks clean to it once a second has passed, and the tree it writes is stale.
pub fn copy_git_index(real: &Path, scratch: &Path) -> std::io::Result<()> {
    std::fs::copy(real, scratch)?;
    let modified = std::fs::metadata(real)?.modified()?;
    std::fs::File::options()
        .write(true)
        .open(scratch)?
        .set_modified(modified)
}
pub fn read(path: &Path) -> Result<String> {
    std::fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| Error::new(format!("{}: {e}", path.display())))
}
pub fn local(root: &Path, path: &Path) -> Result<PathBuf> {
    let root = root.canonicalize()?;
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let resolved = path
        .canonicalize()
        .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
    if !resolved.starts_with(&root) {
        return Err(Error::new(format!(
            "{} is outside selected checkout {}",
            path.display(),
            root.display()
        )));
    }
    Ok(resolved)
}
pub fn arch(arch: &str) -> Result<&str> {
    if ARCHES.contains(&arch) {
        Ok(arch)
    } else {
        Err(Error::new(format!("unsupported architecture: {arch}")))
    }
}
pub mod cargo;
pub mod cell_manifest;
pub mod depfile;
pub mod helper_api;
pub mod json;
pub mod link_guard;
pub mod manifest_api;
pub mod media_manifest;
pub mod ninja;
pub mod process;
pub mod sdk_api;
pub mod sdk_recipe;

pub mod artifact_config;
pub mod artifact_fs;

pub mod artifact_api;
pub mod artifact_inputs;
pub mod artifact_worker;

pub mod artifact_economics;
pub mod artifact_history;

pub mod artifact_store;
pub mod artifact_store_api;

pub mod artifact_executor;
pub mod artifact_runtime;

pub mod artifact_lifetime;

pub mod artifact_execution_api;

pub mod media_stage;

pub mod cargo_orchestrate;

pub mod build_policy;
pub mod diskspace;
pub mod prune;
pub mod trim;

pub mod build_bench;

pub mod process_guard;
pub mod provenance;

pub mod payload_build;
pub mod payload_closure;
pub mod payload_modrun;
pub mod payload_pe;
pub mod payload_providers;
pub mod payload_spec;

pub mod payload_firmware;

#[cfg(test)]
mod index_copy_tests {
    use std::{fs, path::Path, process::Command, thread::sleep, time::Duration};

    fn git(dir: &Path, args: &[&str], index: Option<&Path>) -> String {
        let mut c = Command::new("git");
        c.current_dir(dir).args(args);
        c.env("GIT_AUTHOR_NAME", "t").env("GIT_AUTHOR_EMAIL", "t@t");
        c.env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t");
        if let Some(i) = index {
            c.env("GIT_INDEX_FILE", i);
        }
        let o = c.output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap().trim().to_owned()
    }

    #[test]
    fn a_scratch_index_sees_a_same_size_edit_made_with_the_last_git_add() {
        let repo = tempfile::tempdir().unwrap();
        let r = repo.path();
        git(r, &["init", "-q"], None);
        fs::write(r.join("a.txt"), "aaaa\n").unwrap();
        git(r, &["add", "a.txt"], None);
        // Same second as the add, same size: only a re-hash can see it.
        fs::write(r.join("a.txt"), "bbbb\n").unwrap();
        sleep(Duration::from_millis(1100));
        let tmp = tempfile::tempdir().unwrap();
        let scratch = tmp.path().join("index");
        super::copy_git_index(&r.join(".git/index"), &scratch).unwrap();
        git(r, &["add", "-A"], Some(&scratch));
        let tree = git(r, &["write-tree"], Some(&scratch));
        let blob = git(r, &["rev-parse", &format!("{tree}:a.txt")], None);
        let want = git(r, &["hash-object", "a.txt"], None);
        assert_eq!(blob, want, "the scratch index kept the pre-edit blob");
    }
}

pub mod cli_artifacts;

pub mod cli_arguments;
pub mod tooling_receipts;

pub mod cli_build;
pub mod cli_build_checks;

pub mod cli_build_helpers;
pub mod cli_canon;

pub mod cli_build_entry;
