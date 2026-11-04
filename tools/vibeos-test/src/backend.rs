//! QEMU backend transformations and verified Windows-visible media publication.
use crate::{Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime},
};

pub fn run_capture(
    program: &str,
    args: &[String],
    timeout: Duration,
) -> Result<(i32, String, String)> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().ok_or("missing child stdout")?;
    let mut stderr = child.stderr.take().ok_or("missing child stderr")?;
    let output = thread::spawn(move || {
        let mut v = Vec::new();
        stdout.read_to_end(&mut v).map(|_| v)
    });
    let errors = thread::spawn(move || {
        let mut v = Vec::new();
        stderr.read_to_end(&mut v).map(|_| v)
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            return Err(Error::new(format!(
                "{program} timed out after {}s",
                timeout.as_secs_f64()
            )));
        }
        thread::sleep(Duration::from_millis(20));
    };
    let out = output
        .join()
        .map_err(|_| Error::new("stdout reader panicked"))??;
    let err = errors
        .join()
        .map_err(|_| Error::new("stderr reader panicked"))??;
    use std::os::unix::process::ExitStatusExt;
    Ok((
        status
            .code()
            .unwrap_or_else(|| -status.signal().unwrap_or(1)),
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    ))
}
fn ps(script: &str) -> Result<String> {
    let (code, out, err) = run_capture(
        "powershell.exe",
        &["-NoProfile".into(), "-Command".into(), script.into()],
        Duration::from_secs(30),
    )?;
    if code != 0 {
        return Err(Error::new(format!("PowerShell exited {code}: {err}")));
    }
    Ok(out)
}
pub fn win_path(path: &Path) -> Result<String> {
    let s = path.to_str().ok_or("non-UTF8 Windows staging path")?;
    let b = s.as_bytes();
    if b.len() < 7 || !s.starts_with("/mnt/") || !b[5].is_ascii_alphabetic() || b[6] != b'/' {
        return Err(Error::new(format!(
            "path {s:?} is not under /mnt/<drive>; not reachable by Windows QEMU"
        )));
    }
    Ok(format!(
        "{}:/{}",
        (b[5] as char).to_ascii_uppercase(),
        &s[7..]
    ))
}

pub use crate::publication::{prune_media_cache, MediaLock};

pub fn stage_file(src: &Path, staging: &Path) -> Result<String> {
    win_path(&crate::publication::stage_file(src, staging)?)
}
pub fn stage_readonly_file(src: &Path, staging: &Path) -> Result<String> {
    win_path(&crate::publication::stage_readonly_file(src, staging)?)
}

pub fn windows_qemu_bin(native: &str, dir: &Path, accel: &str) -> String {
    let name = if accel == "whpx" && native == "qemu-system-i386" {
        "qemu-system-x86_64"
    } else {
        native
    };
    dir.join(format!("{name}.exe"))
        .to_string_lossy()
        .into_owned()
}
pub fn windows_qemu_version(exe: &str) -> String {
    match run_capture(exe, &["--version".into()], Duration::from_secs(30)) {
        Ok((_, s, _)) => s
            .lines()
            .next()
            .unwrap_or("")
            .strip_prefix("QEMU emulator version ")
            .unwrap_or(s.lines().next().unwrap_or(""))
            .trim()
            .to_owned(),
        Err(_) => String::new(),
    }
}
pub fn known_bad_windows_qemu(version: &str) -> Option<&'static str> {
    (version=="11.0.50 (v11.0.0-12631-g54e84cdc7a)").then_some("development snapshot whose TCG sets CF/ZF wrong when SAHF follows a logic op, so i386 x87 float compares (fnstsw; sahf; jcc under -march=pentium) branch the wrong way: tools/payloads/atodbl fails here and passes on 11.1.0 and real hardware")
}
pub fn resolve_accel(arch: &str, configured: &str, whpx_ok: Option<bool>) -> String {
    let whpx_ok = if configured == "whpx" && !arch.starts_with("arm64") {
        whpx_ok.or_else(whpx_available)
    } else {
        whpx_ok
    };
    if arch.starts_with("arm64") || configured == "whpx" && whpx_ok == Some(false) {
        "tcg".into()
    } else {
        configured.into()
    }
}
pub fn whpx_available() -> Option<bool> {
    match ps("(Get-CimInstance -ClassName Win32_ComputerSystem).HypervisorPresent")
        .ok()?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}
pub fn rewrite_accel(
    args: &[String],
    accel: &str,
    arch: &str,
    env: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-accel" && i + 1 < args.len() {
            let v = if accel == "whpx" {
                format!(
                    "whpx,kernel-irqchip={}",
                    env.get("VIBEOS_WHPX_IRQCHIP")
                        .map(String::as_str)
                        .unwrap_or(if arch.starts_with("i386") {
                            "off"
                        } else {
                            "on"
                        })
                )
            } else {
                accel.into()
            };
            out.extend(["-accel".into(), v]);
            i += 2;
            continue;
        }
        if a == "-cpu" && args.get(i + 1).is_some_and(|x| x == "host") {
            i += 2;
            continue;
        }
        if a == "-device"
            && args.get(i + 1).is_some_and(|x| x.starts_with("ide-cd,"))
            && (accel == "whpx" || env.get("VIBEOS_AHCI_CD").is_some_and(|v| v == "1"))
        {
            out.extend([
                "-device".into(),
                "ich9-ahci,id=ahci0".into(),
                "-device".into(),
                args[i + 1].replacen("ide-cd,", "ide-cd,bus=ahci0.0,", 1),
            ]);
            i += 2;
            continue;
        }
        out.push(a.clone());
        i += 1;
    }
    out
}
pub fn rewrite_netdev(args: &[String], prefix: &str) -> Result<(Vec<String>, Vec<String>)> {
    let re = Regex::new(r"(guestfwd=tcp:[\d.]+:\d+)-cmd:cat")?;
    let mut ids = Vec::new();
    let mut out = Vec::new();
    for a in args {
        if a.starts_with("user,") && (a.contains("hostfwd=") || a.contains("guestfwd=")) {
            let v = a.replace("hostfwd=tcp:127.0.0.1:", "hostfwd=tcp:0.0.0.0:");
            let rewritten = re.replace_all(&v, |c: &regex::Captures<'_>| {
                let id = format!("{prefix}{}", ids.len());
                ids.push(id.clone());
                format!("{}-chardev:{id}", &c[1])
            });
            out.push(rewritten.into_owned());
        } else {
            out.push(a.clone());
        }
    }
    Ok((out, ids))
}
pub fn stage_media_paths(args: &[String], staging: &Path) -> Result<Vec<String>> {
    let re = Regex::new(r"file=(/[^,]+)")?;
    args.iter()
        .map(|a| {
            let Some(c) = re.captures(a) else {
                return Ok(a.clone());
            };
            let m = c.get(1).unwrap();
            let src = Path::new(m.as_str());
            let replacement = if src.is_file() {
                if a.split(',').any(|s| s == "readonly=on") {
                    stage_readonly_file(src, staging)?
                } else {
                    stage_file(src, staging)?
                }
            } else {
                win_path(src)?
            };
            Ok(format!(
                "{}{}{}",
                &a[..m.start()],
                replacement,
                &a[m.end()..]
            ))
        })
        .collect()
}
pub fn gateway_ip() -> Result<String> {
    let table = fs::read_to_string("/proc/net/route")?;
    gateway_from_table(&table)
}
pub fn gateway_from_table(table: &str) -> Result<String> {
    for line in table.lines().skip(1) {
        let c: Vec<_> = line.split_whitespace().collect();
        if c.len() > 3 && c[1] == "00000000" && u32::from_str_radix(c[3], 16).unwrap_or(0) & 2 != 0
        {
            let n = u32::from_str_radix(c[2], 16).map_err(|e| Error::new(e.to_string()))?;
            return Ok(n.to_le_bytes().map(|v| v.to_string()).join("."));
        }
    }
    Err(Error::new(format!(
        "no default gateway in /proc/net/route:\n{table}"
    )))
}
pub fn windows_free_tcp_port() -> Result<u16> {
    let out=ps("$l=[System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Any,0);$l.Start();$p=$l.LocalEndpoint.Port;$l.Stop();$p")?;
    let digits: String = out.chars().filter(|x| x.is_ascii_digit()).collect();
    let port: u16 = digits.parse().map_err(|_| {
        Error::new(format!(
            "windows_free_tcp_port: no port in powershell output {out:?}"
        ))
    })?;
    if port == 0 {
        return Err("Windows returned port zero".into());
    }
    Ok(port)
}
pub fn kill_by_uid(uid: &str) -> Result<Option<u64>> {
    if uid.is_empty()
        || !uid
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("invalid Windows QEMU uid".into());
    }
    let script=format!("$ErrorActionPreference='Stop';$m = {{ @(Get-CimInstance Win32_Process -Filter \"Name like 'qemu-system-%'\" | Where-Object {{ $_.CommandLine -match '(?:^|\\s)-name(?:\\s+|=){uid}(?=\\s|$)' }}) }};& $m | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }};$left = (& $m).Count; for ($i = 0; $i -lt 20 -and $left -gt 0; $i++) {{ Start-Sleep -Milliseconds 250; $left = (& $m).Count }};Write-Output \"left=$left\"");
    let out = match ps(&script) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let re = Regex::new(r"left=(\d+)")?;
    Ok(re.captures(&out).and_then(|c| c[1].parse().ok()))
}
pub fn windows_qemus() -> Result<Value> {
    let out=ps("$ErrorActionPreference='Stop';ConvertTo-Json -Compress -InputObject @(Get-CimInstance Win32_Process -Filter \"Name like 'qemu-system-%'\" | ForEach-Object { [pscustomobject]@{ pid = $_.ProcessId; name = $_.Name; cmd = [string]$_.CommandLine; created = $_.CreationDate.ToString('s') } })")?;
    if out.trim().is_empty() {
        return Ok(json!([]));
    }
    let value: Value = serde_json::from_str(&out)?;
    match value {
        Value::Array(_) => Ok(value),
        Value::Object(_) => Ok(json!([value])),
        _ => Err("unexpected Windows process listing".into()),
    }
}

pub fn dispatch(_root: &Path, r: &Value) -> Result<Value> {
    let a = r.get("kwargs").unwrap_or(r);
    let s = |n: &str| -> Result<&str> {
        a[n].as_str()
            .ok_or_else(|| Error::new(format!("missing {n}")))
    };
    let args = || -> Result<Vec<String>> { Ok(serde_json::from_value(a["args"].clone())?) };
    let env = || -> Result<BTreeMap<String, String>> {
        Ok(a.get("env")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default())
    };
    Ok(match r["op"].as_str().unwrap_or("") {
        "backend_win_path" => json!(win_path(Path::new(s("path")?))?),
        "backend_publish_file" => json!(crate::publication::stage_file(
            Path::new(s("src")?),
            Path::new(s("staging")?)
        )?),
        "backend_publish_readonly_file" => json!(crate::publication::stage_readonly_file(
            Path::new(s("src")?),
            Path::new(s("staging")?)
        )?),
        "backend_stage_file" => json!(stage_file(Path::new(s("src")?), Path::new(s("staging")?))?),
        "backend_stage_readonly_file" => json!(stage_readonly_file(
            Path::new(s("src")?),
            Path::new(s("staging")?)
        )?),
        "backend_windows_qemu_bin" => json!(windows_qemu_bin(
            s("native_bin")?,
            Path::new(s("qemu_dir")?),
            s("accel")?
        )),
        "backend_windows_qemu_version" => json!(windows_qemu_version(s("exe")?)),
        "backend_known_bad_windows_qemu" => json!(known_bad_windows_qemu(s("version")?)),
        "backend_resolve_accel" => json!(resolve_accel(
            s("arch")?,
            s("configured")?,
            a["whpx_ok"].as_bool()
        )),
        "backend_rewrite_accel" => json!(rewrite_accel(
            &args()?,
            s("accel")?,
            a["arch"].as_str().unwrap_or("amd64"),
            &env()?
        )),
        "backend_rewrite_netdev" => {
            let (args, ids) = rewrite_netdev(&args()?, a["prefix"].as_str().unwrap_or("gf"))?;
            json!([args, ids])
        }
        "backend_stage_media_paths" => {
            json!(stage_media_paths(&args()?, Path::new(s("staging")?))?)
        }
        "backend_prune_media_cache" => {
            prune_media_cache(Path::new(s("cache")?), Path::new(s("keep")?))?;
            Value::Null
        }
        "backend_cleanup_staging" => cleanup_staging(
            Path::new(s("root")?),
            a["dry_run"].as_bool().unwrap_or(false),
            a["min_age"].as_f64().unwrap_or(3600.0),
        )?,
        "backend_whpx_available" => json!(whpx_available()),
        "backend_gateway_ip" => json!(gateway_ip()?),
        "backend_gateway_from_table" => json!(gateway_from_table(s("text")?)?),
        "backend_windows_free_tcp_port" => json!(windows_free_tcp_port()?),
        "backend_kill_by_uid" => json!(kill_by_uid(s("uid")?)?),
        "backend_windows_qemus" => windows_qemus()?,
        _ => return Err("unknown backend operation".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scoped_transformations() {
        assert_eq!(
            win_path(Path::new("/mnt/c/test file")).unwrap(),
            "C:/test file"
        );
        assert!(win_path(Path::new("/tmp/no")).is_err());
        let args = vec!["-accel".into(), "kvm".into(), "-cpu".into(), "host".into()];
        assert_eq!(
            rewrite_accel(&args, "whpx", "i386", &BTreeMap::new()),
            vec!["-accel", "whpx,kernel-irqchip=off"]
        );
        assert_eq!(resolve_accel("arm64", "whpx", Some(true)), "tcg");
        let (a, ids) = rewrite_netdev(
            &[
                "user,id=n,guestfwd=tcp:10.0.2.2:42-cmd:cat,guestfwd=tcp:10.0.2.2:43-cmd:cat"
                    .into(),
            ],
            "gf",
        )
        .unwrap();
        assert_eq!(ids, vec!["gf0", "gf1"]);
        assert!(a[0].contains("chardev:gf1"));
    }
}

/// Caller holds the existing host-wide port handshake lock.
pub fn cleanup_staging(root: &Path, dry: bool, min_age: f64) -> Result<Value> {
    if !min_age.is_finite() || min_age < 0.0 {
        return Err("staging minimum age must be finite and nonnegative".into());
    }
    let mut protected = Vec::new();
    let mut candidates = Vec::new();
    let mut removed = Vec::new();
    let mut eligible = Vec::new();
    if !root.is_dir() {
        return Ok(json!({"protected":[],"eligible":[],"removed":[]}));
    }
    let owner_re = Regex::new(r"^(\d+)-")?;
    let flat = [
        "livecd.iso",
        "OVMF_CODE_4M.fd",
        "OVMF32_CODE.fd",
        "ovmf_vars.qcow2",
        "ovmf32_vars.qcow2",
        "scratch.qcow2",
        "virt.dtb",
        "QEMU_EFI.fd",
        "efivars.qcow2",
        "boot.img",
        "ramdisk.img",
    ];
    let owner_alive = |pid: u32| unsafe {
        libc::kill(pid as i32, 0) == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    };
    let old = |m: &fs::Metadata| {
        m.modified()
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .is_some_and(|age| age.as_secs_f64() >= min_age)
    };
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let m = fs::symlink_metadata(&path)?;
        if m.file_type().is_symlink() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let owner = owner_re
            .captures(&name)
            .and_then(|c| c[1].parse::<u32>().ok());
        let known = m.is_dir() && (owner.is_some() || name.starts_with("dbg-"))
            || m.is_file() && flat.contains(&name.as_ref());
        if !known {
            continue;
        }
        if !old(&m) || owner.is_some_and(owner_alive) {
            protected.push(path);
        } else {
            candidates.push(path);
        }
    }
    let mut commands = Vec::new();
    if !candidates.is_empty() {
        let processes = windows_qemus()?;
        for info in processes
            .as_array()
            .ok_or("invalid Windows process response")?
        {
            let command = info["cmd"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("cannot verify Windows QEMU command line; staging cleanup refused")?;
            commands.push(command.to_lowercase().replace('\\', "/"));
        }
    }
    for path in candidates {
        let mut needle = win_path(&path)?.to_lowercase();
        if path.is_dir() {
            needle.push('/');
        }
        if commands.iter().any(|cmd| cmd.contains(&needle)) {
            protected.push(path);
            continue;
        }
        eligible.push(path.clone());
        if !dry {
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            removed.push(path);
        }
    }
    if !dry {
        let _guard = MediaLock::acquire("cache.lock", None)?;
        let cache = root.join("media-cache-v1");
        if cache.is_dir() {
            let fill = Regex::new(r"^\.populate-v2-(\d+)-")?;
            for entry in fs::read_dir(&cache)? {
                let entry = entry?;
                let path = entry.path();
                let m = match fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(e.into()),
                };
                if m.file_type().is_symlink() || !m.is_dir() {
                    continue;
                }
                let name = entry.file_name();
                let name = name.to_string_lossy();
                let owner = fill.captures(&name).and_then(|c| c[1].parse::<u32>().ok());
                if owner.is_none() && !name.starts_with(".fill-") {
                    continue;
                }
                if owner.is_some_and(owner_alive) {
                    protected.push(path);
                } else if old(&m) {
                    fs::remove_dir_all(&path)?;
                    removed.push(path);
                }
            }
            prune_media_cache(&cache, &cache)?;
        }
    }
    Ok(json!({"protected":protected,"eligible":eligible,"removed":removed}))
}
