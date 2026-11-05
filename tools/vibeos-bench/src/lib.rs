//! Portable physical-bench services shared by the Windows LAN host and Linux control plane.
use std::{collections::BTreeMap, fmt};
#[derive(Debug, Clone, serde::Serialize)]
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
    pub fn classified(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: Some(kind.into()),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
impl From<String> for Error {
    fn from(s: String) -> Self {
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
impl From<ctrlc::Error> for Error {
    fn from(e: ctrlc::Error) -> Self {
        Self::new(e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub type Env = BTreeMap<String, String>;
pub mod bench_http;
pub mod bench_tftp;
#[cfg(unix)]
pub mod lease;
pub mod serial;
pub mod serial_times;
