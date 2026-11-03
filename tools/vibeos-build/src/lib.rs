//! Native build mechanisms. The CLI and broker retain scheduling and reporting ownership.
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

pub mod build_bench;

pub mod process_guard;
