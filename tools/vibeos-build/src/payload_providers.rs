//! Native payload stage providers.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
fn home_path(s: &str) -> PathBuf {
    if s == "~" {
        PathBuf::from(env::var_os("HOME").unwrap_or_default())
    } else if let Some(rest) = s.strip_prefix("~/") {
        PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(rest)
    } else {
        PathBuf::from(s)
    }
}
fn entry(src: &Path, dest: String) -> Value {
    json!({"src":src.to_string_lossy(),"dest":dest})
}
fn wifi(out: &mut Vec<Value>, provider: &str) -> Result<()> {
    if let Ok(wifi) = env::var("VIBEOS_WIFI_CONFIG") {
        if !wifi.is_empty() {
            let cfg = home_path(&wifi);
            if !cfg.is_file() {
                return Err(Error::new(format!(
                    "{provider}: VIBEOS_WIFI_CONFIG={wifi} is not a file"
                )));
            }
            out.push(entry(&cfg, "bin/wifi.txt".into()));
            return Ok(());
        }
    }
    if provider == "piwifi" {
        eprintln!("piwifi: VIBEOS_WIFI_CONFIG is unset, so this image joins no network (set it for the BUILD, not the boot)")
    }
    Ok(())
}
fn preferred_inf(path: &Path) -> Result<PathBuf> {
    let raw = fs::read(path)?;
    let utf16 = raw.starts_with(&[0xff, 0xfe]) || raw.starts_with(&[0xfe, 0xff]);
    let big = raw.starts_with(&[0xfe, 0xff]);
    let text = if utf16 {
        let words: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|b| {
                if big {
                    u16::from_be_bytes([b[0], b[1]])
                } else {
                    u16::from_le_bytes([b[0], b[1]])
                }
            })
            .collect();
        String::from_utf16(&words).map_err(|e| Error::new(e.to_string()))?
    } else {
        raw.iter().map(|b| char::from(*b)).collect()
    };
    let re = regex::Regex::new(r"(?im)^(\s*DriverVer\s*=\s*)[0-9/]+")?;
    if !re.is_match(&text) {
        return Err(Error::new(format!(
            "vendordrv: VIBEOS_VENDOR_PREFER set but {} has no DriverVer line",
            path.file_name().unwrap_or_default().to_string_lossy()
        )));
    }
    let text = re.replace_all(&text, "${1}12/31/2099");
    let re = regex::Regex::new(r"(?m)^(\s*%[^%\r\n]+%\s*=\s*[^,;\r\n]+?)\s*,\s*,")?;
    let text = re.replace_all(&text, "${1},");
    let dir = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_path("~/.cache"))
        .join("vibeos/vendordrv");
    fs::create_dir_all(&dir)?;
    let out = dir.join(
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase(),
    );
    let bytes = if utf16 {
        let mut b = vec![0xff, 0xfe];
        for w in text.encode_utf16() {
            b.extend(w.to_le_bytes())
        }
        b
    } else {
        text.chars().map(|c| c as u8).collect()
    };
    fs::write(&out, bytes)?;
    Ok(out)
}
fn vendordrv() -> Result<Vec<Value>> {
    let root=env::var("VIBEOS_VENDOR_DRIVERS").map_err(|_|Error::new("vendordrv: set VIBEOS_VENDOR_DRIVERS to the driver package directory (the .inf and its files)"))?;
    let root = home_path(&root);
    let root = fs::canonicalize(&root).unwrap_or(root);
    let mut files = if root.is_dir() {
        fs::read_dir(&root)?
            .filter_map(|x| x.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    files.sort();
    if !files
        .iter()
        .any(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("inf")))
    {
        return Err(Error::new(format!(
            "vendordrv: no .inf in {}",
            root.display()
        )));
    }
    let mut out = Vec::new();
    for p in files {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let ext = p
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if name == "ndisprobe.txt" {
            out.push(entry(&p, "bin/ndisprobe.txt".into()));
            continue;
        }
        let src = if env::var("VIBEOS_VENDOR_PREFER").as_deref() == Ok("1") && ext == "inf" {
            preferred_inf(&p)?
        } else {
            p.clone()
        };
        out.push(entry(&src, format!("inf/{name}")));
        if ext == "sys" {
            out.push(entry(&p, format!("system32/drivers/{name}")))
        } else if ext == "dll" {
            out.push(entry(&p, format!("system32/{name}")))
        }
    }
    wifi(&mut out, "vendordrv")?;
    Ok(out)
}
fn hostapps(root: &Path) -> Result<Vec<Value>> {
    let dir = home_path(
        &env::var("VIBEOS_HOST_APPS_ROOT").unwrap_or_else(|_| "/mnt/c/Program Files".into()),
    );
    if !dir.is_dir() {
        return Err(Error::new(format!(
            "hostapps: {} is not a directory (set VIBEOS_HOST_APPS_ROOT)",
            dir.display()
        )));
    }
    let mut out = Vec::new();
    for line in crate::read(&root.join("tools/payloads/hostapps/apps.txt"))?.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<_> = line.split('|').collect();
        if parts.len() != 3 {
            return Err(Error::new(format!("hostapps: malformed apps.txt: {line}")));
        }
        let src = dir.join(parts[1]);
        for pat in parts[2].split(',').map(str::trim) {
            let mut hits = crate::provenance::glob_stable(&src, pat)?
                .into_iter()
                .filter(|p| p.is_file())
                .collect::<Vec<_>>();
            hits.sort();
            if hits.is_empty() {
                return Err(Error::new(format!(
                    "hostapps: {}/{pat} matched nothing (apps.txt: {})",
                    src.display(),
                    parts[0]
                )));
            }
            for p in hits {
                out.push(entry(
                    &p,
                    format!(
                        "bin/{}/{}",
                        parts[0],
                        p.file_name().unwrap_or_default().to_string_lossy()
                    ),
                ))
            }
        }
    }
    Ok(out)
}
fn piwifi() -> Result<Vec<Value>> {
    let dir = crate::payload_firmware::default_dest();
    for line in crate::payload_firmware::fetch(&dir).map_err(|e| {
        Error::new(format!(
            "piwifi: the firmware fetch failed, so this image would have no Pi Wi-Fi: {e}"
        ))
    })? {
        eprintln!("piwifi: {line}");
    }
    let mut out = crate::payload_firmware::FILES
        .iter()
        .map(|(_, name, _)| entry(&dir.join(name), format!("system32/drivers/cyw43455/{name}")))
        .collect::<Vec<_>>();
    wifi(&mut out, "piwifi")?;
    Ok(out)
}

/// Chrome for Testing, copied whole from the host folder `VIBEOS_CHROME_DIR` names (never
/// committed or hosted: the user accepts Google's terms by downloading it). Chrome finds its
/// .pak files, locales and versioned manifest beside chrome.exe, so the layout goes over intact.
fn chrome() -> Result<Vec<Value>> {
    let dir = env::var("VIBEOS_CHROME_DIR").ok().filter(|s| !s.is_empty()).ok_or_else(|| {
        Error::new("chrome: set VIBEOS_CHROME_DIR to a Chrome for Testing folder (the one holding chrome.exe)")
    })?;
    let src = home_path(&dir);
    if !src.join("chrome.exe").is_file() {
        return Err(Error::new(format!(
            "chrome: {} has no chrome.exe",
            src.display()
        )));
    }
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for e in fs::read_dir(dir)? {
            let p = e?.path();
            if p.is_dir() {
                walk(&p, out)?
            } else if p.is_file() {
                out.push(p)
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(&src, &mut files)?;
    files.sort();
    Ok(files
        .iter()
        .map(|p| {
            let rel = p.strip_prefix(&src).unwrap_or(p);
            let rel = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            entry(p, format!("bin/chrome/{rel}"))
        })
        .collect())
}

pub fn stage_entries(root: &Path, spec: &str, arch: &str) -> Result<Vec<Value>> {
    match spec {
        "vendordrv" | "tools/payloads/vendordrv/stage.py" => vendordrv(),
        "hostapps" | "tools/payloads/hostapps/stage.py" => hostapps(root),
        "piwifi" | "tools/payloads/piwifi/stage.py" => piwifi(),
        "chrome" | "tools/payloads/chrome/stage.py" => chrome(),
        "modrun" | "tools/payloads/modrun/modrun.py" => {
            crate::payload_modrun::stage_entries(&root.join("tools/payloads/modrun/suites"), arch)
        }
        _ => Err(Error::new(format!("unknown payload provider: {spec}"))),
    }
}
pub fn ninja_targets(root: &Path, spec: &str, arch: &str) -> Result<Vec<String>> {
    match spec {
        "modrun" | "tools/payloads/modrun/modrun.py" => {
            crate::payload_modrun::ninja_targets(&root.join("tools/payloads/modrun/suites"), arch)
        }
        "vendordrv"
        | "tools/payloads/vendordrv/stage.py"
        | "hostapps"
        | "tools/payloads/hostapps/stage.py"
        | "piwifi"
        | "tools/payloads/piwifi/stage.py"
        | "chrome"
        | "tools/payloads/chrome/stage.py" => Ok(Vec::new()),
        _ => Err(Error::new(format!("unknown ninja provider: {spec}"))),
    }
}
pub fn grade(root: &Path, path: &str, output: &str, arch: &str) -> Result<Value> {
    if path != "tools/payloads/modrun/modrun.py" && path != "modrun" {
        return Err(Error::new(format!("unknown payload grader: {path}")));
    }
    crate::payload_modrun::grade(root, path, output, arch)
}
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let op = request
        .get("op")
        .and_then(Value::as_str)
        .ok_or("missing op")?
        .trim_start_matches("payload_providers_");
    let kw = request.get("kwargs").unwrap_or(&Value::Null);
    let get = |key: &str| -> Result<&str> {
        kw.get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new(format!("missing argument {key}")))
    };
    let dir = kw
        .get("suites")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("tools/payloads/modrun/suites"));
    match op {
        "preferred_inf" => Ok(json!(preferred_inf(Path::new(get("src")?))?)),
        "stage_entries" => {
            if get("spec")? == "modrun" {
                crate::payload_modrun::stage_entries(&dir, get("arch")?).map(Value::Array)
            } else {
                stage_entries(root, get("spec")?, get("arch")?).map(Value::Array)
            }
        }
        "ninja_targets" => Ok(json!(crate::payload_modrun::ninja_targets(
            &dir,
            get("arch")?
        )?)),
        "grade" => {
            let (report, failures) =
                crate::payload_modrun::grade_dir(&dir, get("serial")?, get("arch")?)?;
            Ok(json!([report, failures]))
        }
        "case_file" => Ok(json!(crate::payload_modrun::case_file(&dir, get("arch")?)?)),
        "checksum" => Ok(json!(crate::payload_modrun::checksum(
            get("text")?,
            kw.get("seq").and_then(Value::as_u64).ok_or("missing seq")? as u32
        ))),
        "parse" => {
            let (out, exits) = crate::payload_modrun::parse(get("serial")?)?;
            Ok(json!([
                out.into_iter()
                    .map(|(k, v)| json!([k.0, k.1, v]))
                    .collect::<Vec<_>>(),
                exits
                    .into_iter()
                    .map(|(k, v)| json!([k.0, k.1, v]))
                    .collect::<Vec<_>>()
            ]))
        }
        "cases" => {
            let cases = crate::payload_modrun::cases(&dir, get("arch")?)?;
            Ok(json!(cases.into_iter().map(|c|json!({"id":c.id,"suite":c.suite,"variants":c.variants,"args":c.args,"stdin":c.stdin,"expect":c.expect,"timeout":c.timeout,"normalize":c.normalize,"xfail":c.xfail,"diverge":c.diverge})).collect::<Vec<_>>()))
        }
        "load_suites" => Ok(json!(crate::payload_modrun::load_suites(&dir)?)),
        _ => Err(Error::new(format!(
            "unknown payload provider operation: {op}"
        ))),
    }
}
