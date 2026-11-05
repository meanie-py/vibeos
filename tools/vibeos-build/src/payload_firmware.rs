//! Pinned Pi Wi-Fi firmware acquisition, shared by media staging and the host CLI.
use crate::{Error, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub const COMMIT: &str = "3bab0f823f5b53150b76aab77093adef6655b920";
pub const BASE: &str = "https://raw.githubusercontent.com/RPi-Distro/firmware-nonfree/3bab0f823f5b53150b76aab77093adef6655b920/debian/added-firmware/";
pub const FILES: &[(&str, &str, &str)] = &[
    (
        "cypress/cyfmac43455-sdio-standard.bin",
        "cyfmac43455-sdio.bin",
        "d608f866582519c0a28d86db43040f4f1b98dd1d153e72e9752586546b4a36c3",
    ),
    (
        "cypress/cyfmac43455-sdio.clm_blob",
        "cyfmac43455-sdio.clm_blob",
        "9823842cae9fb9a5dd1e5fb31f595516ec7deee341354bef30bb3026eee29cc1",
    ),
    (
        "brcm/brcmfmac43455-sdio.txt",
        "brcmfmac43455-sdio.txt",
        "ca709be81a78bdb6932936374f39943acbd7af07fae6151011127599a3ce9e3d",
    ),
];
#[derive(Clone, Debug)]
pub struct FirmwareFile {
    pub upstream: String,
    pub name: String,
    pub sha256: String,
}
pub fn pinned_files() -> Vec<FirmwareFile> {
    FILES
        .iter()
        .map(|(u, n, h)| FirmwareFile {
            upstream: (*u).into(),
            name: (*n).into(),
            sha256: (*h).into(),
        })
        .collect()
}
pub fn files_from_json(v: &Value) -> Result<Vec<FirmwareFile>> {
    v.as_array()
        .ok_or("firmware files must be an array")?
        .iter()
        .map(|v| {
            let x = v.as_array().ok_or("firmware file must be a triple")?;
            if x.len() != 3 {
                return Err(Error::new("firmware file must be a triple"));
            }
            let get = |i: usize| {
                x[i].as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| Error::new("firmware file fields must be strings"))
            };
            Ok(FirmwareFile {
                upstream: get(0)?,
                name: get(1)?,
                sha256: get(2)?,
            })
        })
        .collect()
}
pub fn default_dest() -> PathBuf {
    env::var_os("VIBEOS_PI_WIFI_FIRMWARE")
        .filter(|x| !x.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache/vibeos/cyw43455")
        })
}
pub fn sha256(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
pub fn download(url: &str) -> Result<Vec<u8>> {
    let result = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "60",
            url,
        ])
        .output()
        .map_err(|e| Error::new(format!("firmware download: cannot launch curl: {e}")))?;
    if !result.status.success() {
        return Err(Error::new(format!(
            "firmware download failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )));
    }
    Ok(result.stdout)
}
pub fn fetch_with<F>(dest: &Path, files: &[FirmwareFile], mut get: F) -> Result<Vec<String>>
where
    F: FnMut(&str) -> Result<Vec<u8>>,
{
    fs::create_dir_all(dest)?;
    let mut done = Vec::new();
    for file in files {
        let target = dest.join(&file.name);
        if target.is_file() && sha256(&target)? == file.sha256 {
            done.push(format!("{}: present", file.name));
            continue;
        }
        let data = get(&format!("{BASE}{}", file.upstream))?;
        let got = format!("{:x}", Sha256::digest(&data));
        if got != file.sha256 {
            return Err(Error::new(format!("{}: sha256 {got}, pinned {}. Upstream changed under the pinned commit, or the download was altered; nothing written for it.",file.upstream,file.sha256)));
        }
        let mut tmp = tempfile::NamedTempFile::new_in(dest)?;
        tmp.write_all(&data)?;
        tmp.persist(&target)
            .map_err(|e| Error::new(format!("{}: {}", target.display(), e.error)))?;
        done.push(format!("{}: fetched ({} bytes)", file.name, data.len()));
    }
    Ok(done)
}
pub fn fetch(dest: &Path) -> Result<Vec<String>> {
    fetch_with(dest, &pinned_files(), download)
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let op = request
        .get("op")
        .and_then(Value::as_str)
        .ok_or("missing op")?
        .trim_start_matches("payload_firmware_");
    let kw = request.get("kwargs").unwrap_or(&Value::Null);
    let get = |name: &str| {
        kw.get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new(format!("missing argument {name}")))
    };
    let path = |name: &str| -> Result<PathBuf> {
        let p = PathBuf::from(get(name)?);
        Ok(if p.is_absolute() { p } else { root.join(p) })
    };
    match op {
        "metadata" => Ok(json!({"commit":COMMIT,"base":BASE,"files":FILES})),
        "default_dest" => Ok(json!(default_dest())),
        "sha256" => Ok(json!(sha256(&path("path")?)?)),
        "download" => Ok(json!(download(get("url")?)?)),
        "fetch" => {
            let files = kw
                .get("files")
                .map(files_from_json)
                .transpose()?
                .unwrap_or_else(pinned_files);
            Ok(json!(fetch_with(&path("dest")?, &files, download)?))
        }
        _ => Err(Error::new(format!(
            "unknown payload firmware operation: {op}"
        ))),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_hash_and_reuses_cache() {
        let d = tempfile::tempdir().unwrap();
        let bytes = b"firmware";
        let f = FirmwareFile {
            upstream: "source".into(),
            name: "target".into(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
        };
        let mut calls = 0;
        let result = fetch_with(d.path(), &[f.clone()], |_| {
            calls += 1;
            Ok(bytes.to_vec())
        })
        .unwrap();
        assert_eq!(calls, 1);
        assert!(result[0].contains("fetched"));
        let result =
            fetch_with(d.path(), &[f.clone()], |_| panic!("cached file downloaded")).unwrap();
        assert!(result[0].contains("present"));
        fs::write(d.path().join("target"), b"wrong").unwrap();
        assert!(fetch_with(d.path(), &[f], |_| Ok(b"tampered".to_vec())).is_err());
        assert_eq!(fs::read(d.path().join("target")).unwrap(), b"wrong");
    }
}
