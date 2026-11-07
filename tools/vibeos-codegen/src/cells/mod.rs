//! Shared model and recipe contracts for Cargo-owned module generation.
mod types;
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::OnceLock,
};
pub use types::*;
#[derive(Debug, Clone, serde::Serialize)]
pub struct Error {
    pub message: String,
    pub kind: String,
    pub target: Option<String>,
    pub arch: Option<String>,
    pub why: Option<String>,
}
impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: "GenError".into(),
            target: None,
            arch: None,
            why: None,
        }
    }
    pub fn no_link(target: &str, arch: &str, why: &str, message: String) -> Self {
        Self {
            message,
            kind: "NoLinkEdge".into(),
            target: Some(target.into()),
            arch: Some(arch.into()),
            why: Some(why.into()),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
impl From<String> for Error {
    fn from(v: String) -> Self {
        Self::new(v)
    }
}
impl From<&str> for Error {
    fn from(v: &str) -> Self {
        Self::new(v)
    }
}
impl From<std::io::Error> for Error {
    fn from(v: std::io::Error) -> Self {
        Self::new(v.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(v: serde_json::Error) -> Self {
        Self::new(v.to_string())
    }
}
impl From<regex::Error> for Error {
    fn from(v: regex::Error) -> Self {
        Self::new(v.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn constants() -> &'static serde_json::Value {
    static DATA: OnceLock<serde_json::Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("constants.json")).expect("committed cell constants")
    })
}
pub fn read(p: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&std::fs::read(p)?).into_owned())
}
pub fn local(root: &Path, p: &Path) -> Result<PathBuf> {
    Ok(crate::probe::local(root, p)?)
}
pub fn compile_db(root: &Path, arch: &str) -> Result<Vec<serde_json::Value>> {
    // compile_commands.json is written at configure time, like build.ninja: after a merge
    // it describes the PREVIOUS tree. clib_recipes_fresh read one and reported 45 recipes
    // as drifted that a reconfigure turned green (batch 113). Refuse with the reconfigure
    // command instead. (A fixture with no build.ninja has no configure to be stale against.)
    let build = root.join(format!("build-{arch}"));
    if build.join("build.ninja").is_file() {
        vibeos_build::ninja::require_configured(&local(root, &build)?, root)
            .map_err(|e| Error::new(e.to_string()))?;
    }
    let file = local(
        root,
        &root.join(format!("build-{arch}/compile_commands.json")),
    )?;
    Ok(serde_json::from_str(&read(&file)?)?)
}

pub mod pe;

pub mod graph;
pub mod interface;
pub mod mutate;
pub mod proof;
pub mod recipes;
pub mod render;
pub mod rewrite;

pub mod api;
pub mod driver;
