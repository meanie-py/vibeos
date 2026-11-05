//! Shared bench ownership. The legacy Pi 4/OptiPlex alias is preserved conservatively.
use crate::{Env, Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    os::fd::AsRawFd,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub const DEFAULT_MINUTES: f64 = 30.;
pub const REFUSED: u8 = 75;
pub const BOARDS: [&str; 3] = ["pi4", "pi5", "optiplex"];
fn clock() -> Result<f64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::new("bench clock precedes epoch"))?
        .as_secs_f64())
}
fn when(now: Option<f64>) -> Result<f64> {
    let v = match now {
        Some(v) => v,
        None => clock()?,
    };
    if !v.is_finite() {
        return Err("bench clock must be finite".into());
    }
    Ok(v)
}
pub fn board(env: &Env) -> Result<String> {
    let name = env
        .get("VIBEOS_BENCH_BOARD")
        .map(String::as_str)
        .unwrap_or("pi4");
    if !BOARDS.contains(&name) {
        return Err(Error::new(format!(
            "VIBEOS_BENCH_BOARD={name:?}: choose from {}",
            BOARDS.join(", ")
        )));
    }
    Ok(name.into())
}
pub fn lease_name(env: &Env) -> Result<String> {
    let b = board(env)?;
    Ok(if b == "pi4" { "optiplex".into() } else { b })
}
pub fn lock_dir(env: &Env) -> PathBuf {
    env.get("VIBEOS_HOSTLOCK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/vibeos-qemu-locks/host"))
}
pub fn lease_file(env: &Env) -> Result<PathBuf> {
    Ok(lock_dir(env).join(format!("{}-lease.json", lease_name(env)?)))
}
pub fn owner_of(root: &Path, env: &Env) -> Result<String> {
    if let Some(owner) = env.get("VIBEOS_PI4_OWNER").filter(|s| !s.is_empty()) {
        return Ok(owner.clone());
    }
    let mut child = match Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => Some(c),
        Err(_) => None,
    };
    if let Some(child) = child.as_mut() {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait()? {
                if status.success() {
                    let mut text = String::new();
                    child
                        .stdout
                        .take()
                        .ok_or("git stdout missing")?
                        .read_to_string(&mut text)?;
                    if !text.trim().is_empty() {
                        return Ok(text.trim().into());
                    }
                }
                break;
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(root.canonicalize()?.to_string_lossy().into_owned())
}
struct Lock(fs::File);
impl Drop for Lock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
fn locked(env: &Env) -> Result<Lock> {
    let dir = lock_dir(env);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}-lease.lock", lease_name(env)?));
    let f = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o666)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)?;
    loop {
        if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) } == 0 {
            return Ok(Lock(f));
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error.into());
        }
    }
}
fn corrupt(path: &Path, message: impl std::fmt::Display) -> Error {
    Error::classified("LeaseStateError",format!("cannot trust bench lease {}: {message}; inspect it and use explicit force-drop to recover",path.display()))
}
pub fn current(env: &Env, now: Option<f64>) -> Result<Value> {
    let path = lease_file(env)?;
    let data = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(json!({})),
        Err(e) => return Err(corrupt(&path, e)),
    };
    // Unlike the former Python scanner, corrupt or unreadable state cannot grant ownership.
    let value: Value = serde_json::from_slice(&data).map_err(|e| corrupt(&path, e))?;
    let expires = value["expires"]
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| corrupt(&path, "expires must be finite"))?;
    if expires <= when(now)? {
        return Ok(json!({}));
    }
    if value["owner"].as_str().is_none_or(str::is_empty) {
        return Err(corrupt(&path, "owner is missing"));
    }
    Ok(value)
}
fn occupied(value: &Value) -> bool {
    value.as_object().is_some_and(|v| !v.is_empty())
}
pub fn describe(lease: &Value, now: Option<f64>) -> Result<String> {
    if !occupied(lease) {
        return Ok("free".into());
    }
    let expires = lease["expires"].as_f64().ok_or("lease expiry missing")?;
    let left = (expires - when(now)?).max(0.) as u64;
    let why = lease["why"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| format!(" - {s}"))
        .unwrap_or_default();
    Ok(format!(
        "held by {} for {}m{:02}s more{why}",
        lease["owner"].as_str().ok_or("lease owner missing")?,
        left / 60,
        left % 60
    ))
}
static SERIAL: AtomicU64 = AtomicU64::new(0);
fn publish(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().ok_or("lease has no directory")?;
    let tmp = parent.join(format!(
        ".bench-lease-{}-{}.tmp",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<()> {
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&tmp)?;
        serde_json::to_writer(&mut f, value)?;
        f.write_all(b"\n")?;
        f.sync_all()?;
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}
pub fn take(owner: &str, minutes: f64, why: &str, env: &Env, pid: u32) -> Result<(bool, Value)> {
    if owner.is_empty() || !minutes.is_finite() || minutes <= 0. {
        return Err("bench lease needs an owner and finite positive minutes".into());
    }
    let _lock = locked(env)?;
    let now = clock()?;
    let held = current(env, Some(now))?;
    if occupied(&held) && held["owner"] != owner {
        return Ok((false, held));
    }
    let expires = (now + minutes * 60.).max(held["expires"].as_f64().unwrap_or(0.));
    if !expires.is_finite() {
        return Err("bench lease expiration overflows".into());
    }
    let reason = if why.is_empty() {
        held["why"].as_str().unwrap_or("")
    } else {
        why
    };
    let value = json!({"owner":owner,"why":reason,"since":held["since"].as_f64().unwrap_or(now),"expires":expires,"pid":pid});
    publish(&lease_file(env)?, &value)?;
    Ok((true, value))
}
pub fn drop(owner: &str, force: bool, env: &Env) -> Result<(bool, Value)> {
    let _lock = locked(env)?;
    let held = match current(env, None) {
        Ok(v) => v,
        Err(_) if force => json!({}),
        Err(e) => return Err(e),
    };
    if occupied(&held) && held["owner"] != owner && !force {
        return Ok((false, held));
    }
    let path = lease_file(env)?;
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok((true, held))
}
pub fn check(owner: &str, env: &Env) -> Result<(bool, Value)> {
    let held = current(env, None)?;
    Ok((!occupied(&held) || held["owner"] == owner, held))
}
pub fn dispatch(root: &Path, r: &Value) -> Result<Value> {
    let a = &r["kwargs"];
    let op = r["op"]
        .as_str()
        .ok_or("missing lease op")?
        .trim_start_matches("bench_lease_");
    let env: Env = match a.get("env") {
        Some(v) => serde_json::from_value(v.clone())?,
        None => std::env::vars().collect(),
    };
    let owner = || {
        a["owner"]
            .as_str()
            .map(str::to_owned)
            .map(Ok)
            .unwrap_or_else(|| owner_of(root, &env))
    };
    Ok(match op {
        "board" => json!(board(&env)?),
        "lease_name" => json!(lease_name(&env)?),
        "lock_dir" => json!(lock_dir(&env)),
        "lease_file" => json!(lease_file(&env)?),
        "owner_of" => json!(owner_of(
            a["cwd"].as_str().map(Path::new).unwrap_or(root),
            &env
        )?),
        "current" => current(&env, a["now"].as_f64())?,
        "describe" => json!(describe(&a["lease"], a["now"].as_f64())?),
        "take" => json!(take(
            &owner()?,
            a["minutes"].as_f64().unwrap_or(DEFAULT_MINUTES),
            a["why"].as_str().unwrap_or(""),
            &env,
            a["pid"]
                .as_u64()
                .unwrap_or(unsafe { libc::getppid() } as u64) as u32
        )?),
        "drop" => json!(drop(&owner()?, a["force"] == true, &env)?),
        "check" => json!(check(&owner()?, &env)?),
        "main" => {
            let verb = a["verb"].as_str().unwrap_or("show");
            let owner = owner()?;
            let (code, out, err) = match verb {
                "show" => (
                    0,
                    format!("{}\n", describe(&current(&env, None)?, None)?),
                    String::new(),
                ),
                "check" | "touch" => {
                    let (ok, held) = if verb == "touch" {
                        let minutes = env
                            .get("VIBEOS_PI4_LEASE_MINUTES")
                            .map(|v| {
                                v.parse::<f64>()
                                    .map_err(|_| Error::new("invalid lease minutes"))
                            })
                            .transpose()?
                            .unwrap_or(DEFAULT_MINUTES);
                        take(&owner, minutes, "", &env, unsafe { libc::getppid() } as u32)?
                    } else {
                        check(&owner, &env)?
                    };
                    if ok {
                        (0, String::new(), String::new())
                    } else {
                        (REFUSED as i32,String::new(),format!("{} bench is {}.\n  Wait for it, or coordinate with that lane; `vibeos-cli {} status` shows it.\n",board(&env)?,describe(&held,None)?,board(&env)?))
                    }
                }
                _ => (
                    2,
                    String::new(),
                    "usage: pi4lease.py show|check|touch\n".into(),
                ),
            };
            json!({"code":code,"stdout":out,"stderr":err})
        }
        _ => return Err(Error::new(format!("unknown bench lease operation {op}"))),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn env(dir: &Path, board: &str) -> Env {
        [
            (
                "VIBEOS_HOSTLOCK_DIR".into(),
                dir.to_string_lossy().into_owned(),
            ),
            ("VIBEOS_BENCH_BOARD".into(), board.into()),
        ]
        .into_iter()
        .collect()
    }
    #[test]
    fn aliases_and_board_isolation() {
        let d = tempfile::tempdir().unwrap();
        let pi4 = env(d.path(), "pi4");
        let opt = env(d.path(), "optiplex");
        let pi5 = env(d.path(), "pi5");
        assert!(take("a", 1., "boot", &pi4, 1).unwrap().0);
        assert!(!take("b", 1., "", &opt, 2).unwrap().0);
        assert!(take("b", 1., "", &pi5, 2).unwrap().0);
        assert!(!drop("b", false, &opt).unwrap().0);
        assert!(drop("a", false, &opt).unwrap().0);
    }
    #[test]
    fn renewal_does_not_shorten_and_corruption_refuses() {
        let d = tempfile::tempdir().unwrap();
        let e = env(d.path(), "pi4");
        let (_, first) = take("a", 10., "reason", &e, 1).unwrap();
        let (_, second) = take("a", 1., "", &e, 1).unwrap();
        assert_eq!(first["expires"], second["expires"]);
        assert_eq!(second["why"], "reason");
        fs::write(lease_file(&e).unwrap(), b"{broken").unwrap();
        assert!(current(&e, None).is_err());
        assert!(take("b", 1., "", &e, 2).is_err());
        assert!(drop("a", true, &e).unwrap().0);
        assert_eq!(current(&e, None).unwrap(), json!({}));
    }
    #[test]
    fn cross_thread_writers_have_exactly_one_owner() {
        let d = tempfile::tempdir().unwrap();
        let e = env(d.path(), "pi5");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let threads: Vec<_> = (0..8)
            .map(|i| {
                let e = e.clone();
                let b = barrier.clone();
                std::thread::spawn(move || {
                    b.wait();
                    take(&format!("lane-{i}"), 1., "", &e, 1).unwrap().0
                })
            })
            .collect();
        assert_eq!(
            threads
                .into_iter()
                .map(|t| t.join().unwrap())
                .filter(|ok| *ok)
                .count(),
            1
        );
    }
}
