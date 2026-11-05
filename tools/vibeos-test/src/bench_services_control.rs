//! Lease-scoped lifecycle for the Pi 4 bench host services.
//! Linux owns the detached Pico serial capture; Windows owns LAN servers.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
type Env = BTreeMap<String, String>;
const POWERSHELL: &str = "/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe";
fn err(e: vibeos_bench::Error) -> Error {
    Error::new(e.message)
}
fn value<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn setting<'a>(env: &'a Env, key: &str, default: &'a str) -> &'a str {
    env.get(key).map(String::as_str).unwrap_or(default)
}
fn psquote(s: &str) -> Result<String> {
    if s.contains('\0') || s.contains('\r') || s.contains('\n') {
        return Err("invalid Windows argument".into());
    }
    Ok(format!("'{}'", s.replace('\'', "''")))
}
fn windows_arg(s: &str) -> Result<String> {
    if s.contains('\0') || s.contains('\r') || s.contains('\n') {
        return Err("invalid Windows argument".into());
    }
    let mut out = String::from("\"");
    let mut slashes = 0usize;
    for c in s.chars() {
        match c {
            '\\' => slashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(slashes * 2 + 1));
                out.push('"');
                slashes = 0
            }
            _ => {
                out.push_str(&"\\".repeat(slashes));
                slashes = 0;
                out.push(c)
            }
        }
    }
    out.push_str(&"\\".repeat(slashes * 2));
    out.push('"');
    Ok(out)
}
/// Windows command line splitting follows CommandLineToArgvW's quote/backslash rules.
fn split_windows(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut args = Vec::new();
    while i < chars.len() {
        while i < chars.len() && chars[i].is_ascii_whitespace() {
            i += 1
        }
        if i == chars.len() {
            break;
        }
        let mut arg = String::new();
        let mut quoted = false;
        while i < chars.len() && (quoted || !chars[i].is_ascii_whitespace()) {
            let mut n = 0;
            while i < chars.len() && chars[i] == '\\' {
                n += 1;
                i += 1
            }
            if i < chars.len() && chars[i] == '"' {
                arg.push_str(&"\\".repeat(n / 2));
                if n % 2 == 1 {
                    arg.push('"');
                    i += 1
                } else if quoted && i + 1 < chars.len() && chars[i + 1] == '"' {
                    arg.push('"');
                    i += 2
                } else {
                    quoted = !quoted;
                    i += 1
                }
            } else {
                arg.push_str(&"\\".repeat(n));
                if i < chars.len() && (quoted || !chars[i].is_ascii_whitespace()) {
                    arg.push(chars[i]);
                    i += 1
                }
            }
        }
        args.push(arg);
    }
    args
}
fn winpath_eq(a: &str, b: &str) -> bool {
    a.replace('/', "\\")
        .trim_end_matches('\\')
        .eq_ignore_ascii_case(b.replace('/', "\\").trim_end_matches('\\'))
}
fn proc_ticks(stat: &str) -> Option<u64> {
    stat.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}
fn proc_stat(pid: u32) -> Option<(u64, char)> {
    let s = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let tail = s.rsplit_once(')')?.1;
    Some((
        proc_ticks(&s)?,
        tail.split_whitespace().next()?.chars().next()?,
    ))
}
fn pidfd(pid: u32) -> Result<File> {
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
    if fd < 0 {
        return Err(Error::new(format!(
            "pidfd_open({pid}): {}",
            std::io::Error::last_os_error()
        )));
    }
    use std::os::fd::FromRawFd;
    Ok(unsafe { File::from_raw_fd(fd as i32) })
}
fn pidfd_signal(fd: &File, signal: i32) -> Result<()> {
    use std::os::fd::AsRawFd;
    let rc = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd.as_raw_fd(),
            signal,
            0usize,
            0usize,
        )
    };
    if rc != 0 {
        return Err(Error::new(format!(
            "pidfd_send_signal: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}
fn pidfd_exited(fd: &File) -> Result<bool> {
    use std::os::fd::AsRawFd;
    let mut p = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let rc = unsafe { libc::poll(&mut p, 1, 0) };
    if rc < 0 {
        return Err(Error::new(format!(
            "poll pidfd: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(rc > 0)
}
fn read_json(path: &Path) -> Result<Option<Value>> {
    match fs::read(path) {
        Ok(b) => {
            let text = if b.starts_with(&[0xff, 0xfe]) {
                String::from_utf16(
                    &b[2..]
                        .chunks_exact(2)
                        .map(|v| u16::from_le_bytes([v[0], v[1]]))
                        .collect::<Vec<_>>(),
                )
                .map_err(|e| Error::new(format!("invalid receipt {}: {e}", path.display())))?
            } else if b.starts_with(&[0xfe, 0xff]) {
                String::from_utf16(
                    &b[2..]
                        .chunks_exact(2)
                        .map(|v| u16::from_be_bytes([v[0], v[1]]))
                        .collect::<Vec<_>>(),
                )
                .map_err(|e| Error::new(format!("invalid receipt {}: {e}", path.display())))?
            } else {
                String::from_utf8(b)
                    .map_err(|e| Error::new(format!("invalid receipt {}: {e}", path.display())))?
            };
            Ok(Some(
                serde_json::from_str(text.trim_start_matches('\u{feff}'))
                    .map_err(|e| Error::new(format!("invalid receipt {}: {e}", path.display())))?,
            ))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn atomic_json(path: &Path, v: &Value) -> Result<()> {
    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().ok_or("receipt has no parent")?)?;
    serde_json::to_writer(&mut tmp, v)?;
    tmp.write_all(b"\n")?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .map_err(|e| Error::new(format!("publish {}: {e}", path.display())))?;
    Ok(())
}
fn native_run(mut command: Command, timeout: u64) -> Result<vibeos_build::process::Output> {
    vibeos_build::artifact_executor::supervised_output(
        &mut command,
        Some(Duration::from_secs(timeout)),
    )
}
fn checked(command: Command, timeout: u64, what: &str) -> Result<String> {
    let out = native_run(command, timeout)?;
    if out.code != 0 {
        return Err(Error::new(format!(
            "{what} exited {}: {}",
            out.code,
            out.stderr.trim()
        )));
    }
    Ok(out.stdout.trim().to_owned())
}
fn powershell(script: &str) -> Result<String> {
    let mut c = Command::new(POWERSHELL);
    c.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        script,
    ])
    .stdin(Stdio::null());
    checked(c, 30, "PowerShell").map(|s| s.replace('\r', ""))
}
fn winpath(p: &Path) -> Result<String> {
    let mut c = Command::new("wslpath");
    c.args(["-w", &p.to_string_lossy()]);
    checked(c, 10, "wslpath")
}
fn emit(host: &mut dyn Host, text: impl Into<String>) -> Result<()> {
    host.call("print", json!({"text":text.into()}))?;
    Ok(())
}
struct Config {
    root: PathBuf,
    env: Env,
    board: String,
    bench: PathBuf,
    win_bench: String,
    win_payload: String,
    win_exe: String,
    win_python: String,
    host: String,
    http_bind: String,
    http_port: u16,
    serial_port: u16,
    mode: String,
    com: String,
}
impl Config {
    fn new(root: &Path, a: &Value, linux_only: bool) -> Result<Self> {
        let mut env: Env = serde_json::from_value(a.get("env").cloned().unwrap_or(json!({})))?;
        let board = a
            .get("board")
            .and_then(Value::as_str)
            .or_else(|| env.get("VIBEOS_BENCH_BOARD").map(String::as_str))
            .unwrap_or("pi4")
            .to_owned();
        if board != "pi4" && board != "pi5" {
            return Err(Error::new(format!("unsupported bench board {board:?}")));
        }
        if !linux_only && board != "pi4" {
            return Err("Windows bench services are available only for pi4".into());
        }
        env.insert("VIBEOS_BENCH_BOARD".into(), board.clone());
        let (bench_key, bench_default, host_key, host_default) = if board == "pi5" {
            (
                "PI5_BENCH",
                "/mnt/d/home/vibeos-rpi5-setup",
                "PI5_PICO_HOST",
                "none.invalid",
            )
        } else {
            (
                "PI4_BENCH",
                "/mnt/d/home/vibeos-rpi4-setup",
                "PI4_PICO_HOST",
                "picousb",
            )
        };
        let bench = PathBuf::from(setting(&env, bench_key, bench_default));
        let win_bench = if linux_only {
            String::new()
        } else {
            winpath(&bench)?
        };
        let wp = format!("{}\\payload", win_bench.trim_end_matches('\\'));
        let we = format!("{}\\bench-infra.exe", win_bench.trim_end_matches('\\'));
        let port = |name: &str, default: u16| -> Result<u16> {
            match env.get(name) {
                Some(s) => s
                    .parse::<u16>()
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or_else(|| Error::new(format!("invalid {name}"))),
                None => Ok(default),
            }
        };
        let mode = if linux_only {
            "pico"
        } else {
            setting(&env, "PI4_SERIAL", "pico")
        };
        if mode != "pico" && mode != "com" {
            return Err(Error::new(format!(
                "unsupported PI4_SERIAL {mode:?}; choose pico or com"
            )));
        }
        Ok(Self {
            root: root.into(),
            bench,
            win_payload: wp,
            win_exe: we,
            win_bench,
            win_python: setting(
                &env,
                "PI4_WINPY",
                r"C:\Users\Lambda\AppData\Local\Programs\Python\Python313\python.exe",
            )
            .into(),
            host: setting(&env, host_key, host_default).into(),
            http_bind: setting(&env, "PI4_HOST_IP", "192.168.1.183").into(),
            http_port: if linux_only {
                8069
            } else {
                port("PI4_HTTP_PORT", 8069)?
            },
            serial_port: port("PICO_SERIAL_PORT", 8082)?,
            mode: mode.into(),
            com: setting(&env, "PI4_COM", "COM5").into(),
            board,
            env,
        })
    }
    fn lease(&self, why: &str) -> Result<()> {
        let owner = vibeos_bench::lease::owner_of(&self.root, &self.env).map_err(err)?;
        let (ok, held) = vibeos_bench::lease::take(&owner, 30., why, &self.env, std::process::id())
            .map_err(err)?;
        if !ok {
            return Err(Error::classified(
                "BENCH_BUSY",
                format!(
                    "bench is {}",
                    vibeos_bench::lease::describe(&held, None).map_err(err)?
                ),
            ));
        }
        Ok(())
    }
    fn receipt(&self, kind: &str) -> PathBuf {
        self.bench.join(format!("{kind}-service.json"))
    }
    fn ps(&self) -> Result<Vec<WinProcess>> {
        let s=powershell("$ErrorActionPreference='Stop'; ConvertTo-Json -InputObject @(Get-CimInstance Win32_Process | Where-Object { $_.CreationDate -and $_.CommandLine -and $_.ExecutablePath } | ForEach-Object { [pscustomobject]@{pid=[int]$_.ProcessId; started=([datetime]$_.CreationDate).ToUniversalTime().Ticks; exe=[string]$_.ExecutablePath; command=[string]$_.CommandLine} }) -Compress")?;
        let v: Value = serde_json::from_str(s.trim_start_matches('\u{feff}'))
            .map_err(|e| Error::new(format!("process list JSON: {e}")))?;
        v.as_array()
            .ok_or("process list is not an array")?
            .iter()
            .map(WinProcess::parse)
            .collect()
    }
}
#[derive(Clone, Debug)]
struct WinProcess {
    pid: u32,
    started: i64,
    exe: String,
    command: String,
}
impl WinProcess {
    fn parse(v: &Value) -> Result<Self> {
        Ok(Self {
            pid: v["pid"]
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or("invalid Windows pid")?,
            started: v["started"]
                .as_i64()
                .ok_or("invalid Windows process start")?,
            exe: value(v, "exe").into(),
            command: value(v, "command").into(),
        })
    }
    fn json(&self) -> Value {
        json!({"pid":self.pid,"started":self.started,"exe":self.exe,"command":self.command})
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Http,
    Tftp,
    Com,
}
impl Kind {
    fn key(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Tftp => "tftp",
            Self::Com => "com",
        }
    }
}
fn recognized(c: &Config, p: &WinProcess, kind: Kind) -> bool {
    let args = split_windows(&p.command);
    if args.is_empty() {
        return false;
    }
    let (name, bind, port) = match kind {
        Kind::Http => ("http", c.http_bind.as_str(), c.http_port),
        Kind::Tftp => ("tftp", "0.0.0.0", 69),
        Kind::Com => ("com", "", 0),
    };
    if kind != Kind::Com && winpath_eq(&p.exe, &c.win_exe) {
        return args.len() == 5
            && winpath_eq(&args[0], &c.win_exe)
            && args[1].eq_ignore_ascii_case(name)
            && winpath_eq(&args[2], &c.win_payload)
            && args[3].eq_ignore_ascii_case(bind)
            && args[4] == port.to_string();
    }
    if kind == Kind::Com {
        let script = format!(
            "{}\\Capture-RpiSerial.ps1",
            c.win_bench.trim_end_matches('\\')
        );
        return winpath_eq(
            &p.exe,
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        ) && (winpath_eq(&args[0], &p.exe)
            || args[0].eq_ignore_ascii_case("powershell.exe"))
            && args.len() == 8
            && args[1].eq_ignore_ascii_case("-NoProfile")
            && args[2].eq_ignore_ascii_case("-ExecutionPolicy")
            && args[3].eq_ignore_ascii_case("Bypass")
            && args[4].eq_ignore_ascii_case("-File")
            && winpath_eq(&args[5], &script)
            && args[6].eq_ignore_ascii_case("-PortName")
            && args[7].eq_ignore_ascii_case(&c.com);
    }
    if !winpath_eq(&p.exe, &c.win_python) || !winpath_eq(&args[0], &c.win_python) {
        return false;
    }
    let script = format!("{}\\serve_{name}.py", c.win_bench.trim_end_matches('\\'));
    let i = if args.get(1).is_some_and(|s| s.eq_ignore_ascii_case("-u")) {
        2
    } else {
        1
    };
    if args.len() != i + 3
        && !(kind == Kind::Http && args.len() == i + 4)
        && !(kind == Kind::Tftp && args.len() == i + 4)
    {
        return false;
    }
    winpath_eq(&args[i], &script)
        && winpath_eq(&args[i + 1], &c.win_payload)
        && args[i + 2].eq_ignore_ascii_case(bind)
        && args
            .get(i + 3)
            .map(|s| s == &port.to_string())
            .unwrap_or(kind == Kind::Tftp && port == 69)
}
fn receipt_matches(v: &Value, p: &WinProcess) -> bool {
    v["pid"].as_u64() == Some(p.pid as u64)
        && v["started"].as_i64() == Some(p.started)
        && winpath_eq(value(v, "exe"), &p.exe)
        && value(v, "command") == p.command
}
fn selected(c: &Config, kind: Kind, processes: &[WinProcess]) -> Result<Vec<WinProcess>> {
    let receipt = read_json(&c.receipt(kind.key()))?;
    let mut found = Vec::new();
    for p in processes.iter().filter(|p| recognized(c, p, kind)) {
        let native = winpath_eq(&p.exe, &c.win_exe);
        if kind == Kind::Com
            && !receipt.as_ref().is_some_and(|r| receipt_matches(r, p))
            && !com_ready(c, p)?
        {
            return Err(Error::new(format!(
                "COM process {} has no matching serial receipt",
                p.pid
            )));
        }
        if native && !receipt.as_ref().is_some_and(|r| receipt_matches(r, p)) {
            return Err(Error::new(format!(
                "unowned {} process {} matches this bench; inspect its receipt",
                kind.key(),
                p.pid
            )));
        }
        found.push(p.clone());
    }
    Ok(found)
}
fn registered_worktree_roots(c: &Config) -> Result<Vec<PathBuf>> {
    let mut command = Command::new("git");
    command
        .args(["worktree", "list", "--porcelain", "-z"])
        .current_dir(&c.root)
        .env_clear()
        .envs(&c.env);
    let raw = checked(command, 10, "git worktree list")?;
    let roots: Vec<_> = raw
        .split('\0')
        .filter_map(|field| field.strip_prefix("worktree "))
        .filter_map(|path| Path::new(path).canonicalize().ok())
        .collect();
    if roots.is_empty() {
        return Err("git reported no accessible registered worktrees".into());
    }
    Ok(roots)
}

// Linux appends this suffix when a running executable's cache file is replaced.
// Canonicalize its parent, not the current binary, which is a different inode.
fn normalize_exe_link(target: &Path) -> Option<PathBuf> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let bytes = target.as_os_str().as_bytes();
    let original = bytes.strip_suffix(b" (deleted)").unwrap_or(bytes);
    let path = PathBuf::from(std::ffi::OsString::from_vec(original.to_vec()));
    Some(path.parent()?.canonicalize().ok()?.join(path.file_name()?))
}

fn native_location_matches(root: &Path, actual: &Path) -> bool {
    let expected = root.join(".cache/vibeos-bench/linux/vibeos-bench");
    expected
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .is_some_and(|parent| parent.join("vibeos-bench") == actual)
}

fn native_owner_matches(roots: &[PathBuf], root: &Path, actual: &Path) -> bool {
    roots.iter().any(|registered| registered == root) && native_location_matches(root, actual)
}

fn serial_args_match(c: &Config, pid: u32, receipt: &Value) -> Result<bool> {
    let path = format!("/proc/{pid}/cmdline");
    let b = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };
    let args: Vec<_> = b
        .split(|c| *c == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    let bench = c.bench.canonicalize()?;
    let resolve = |v: &str| {
        let p = Path::new(v);
        if p.is_absolute() {
            p.to_owned()
        } else {
            PathBuf::from(format!("/proc/{pid}/cwd")).join(p)
        }
    };
    let same_bench = |v: &str| resolve(v).canonicalize().is_ok_and(|p| p == bench);
    let exe = args
        .first()
        .and_then(|v| Path::new(v).file_name())
        .and_then(|v| v.to_str())
        .unwrap_or("");
    let actual_exe = fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .and_then(|p| normalize_exe_link(&p));
    let legacy_exe = c
        .env
        .get("PICO_PY")
        .map(PathBuf::from)
        .or_else(|| {
            c.env
                .get("HOME")
                .map(|h| PathBuf::from(h).join(".venvs/picousb/bin/python"))
        })
        .and_then(|p| p.canonicalize().ok());
    let is_native_shape = exe == "vibeos-bench"
        && args.len() == 6
        && args[1] == "--root"
        && args[3] == "serial"
        && same_bench(&args[4])
        && args[5].eq_ignore_ascii_case(&c.host);
    let is_legacy_exe = actual_exe.is_some() && actual_exe == legacy_exe;
    let roots = if is_native_shape || is_legacy_exe {
        registered_worktree_roots(c)?
    } else {
        Vec::new()
    };
    let native = is_native_shape
        && resolve(&args[2]).canonicalize().ok().is_some_and(|root| {
            actual_exe
                .as_deref()
                .is_some_and(|actual| native_owner_matches(&roots, &root, actual))
        });
    let legacy_scripts: Vec<_> = if is_legacy_exe {
        roots
            .iter()
            .filter_map(|root| root.join("tools/pi4/pico_serial.py").canonicalize().ok())
            .collect()
    } else {
        Vec::new()
    };
    let legacy = args
        .iter()
        .position(|v| {
            resolve(v)
                .canonicalize()
                .ok()
                .is_some_and(|p| legacy_scripts.contains(&p))
        })
        .and_then(|i| {
            if args.len() == i + 3 && (i == 1 || (i == 2 && args[1] == "-u")) {
                args.get(i + 1..i + 3)
            } else {
                None
            }
        })
        .is_some_and(|v| same_bench(&v[0]) && v[1].eq_ignore_ascii_case(&c.host));
    let state_pid = receipt["pid"].as_u64() == Some(pid as u64);
    let port = value(receipt, "port")
        .eq_ignore_ascii_case(&format!("{}:{} (picousb)", c.host, c.serial_port));
    let within = receipt
        .get("directory")
        .and_then(Value::as_str)
        .and_then(|p| Path::new(p).canonicalize().ok())
        .is_some_and(|p| p.starts_with(&bench) && p != bench);
    let ticks = proc_stat(pid).map(|x| x.0);
    let native_receipt = value(receipt, "engine") == "vibeos-bench"
        && receipt["process_start_ticks"].as_u64() == ticks
        && ticks.is_some();
    Ok(state_pid
        && port
        && within
        && ((native && native_receipt) || (legacy && value(receipt, "engine").is_empty())))
}
fn serial_process(c: &Config) -> Result<Option<u32>> {
    let state = read_json(&c.bench.join("serial-current.json"))?;
    let pid_path = c.bench.join("pico-serial.pid");
    let from_file = match fs::read_to_string(&pid_path) {
        Ok(s) => Some(
            s.trim()
                .parse::<u32>()
                .map_err(|_| Error::new(format!("invalid {}", pid_path.display())))?,
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };
    let from_state = state
        .as_ref()
        .filter(|v| value(v, "port").ends_with(" (picousb)"))
        .and_then(|v| v["pid"].as_u64())
        .and_then(|n| u32::try_from(n).ok());
    let mut found = None;
    for pid in [from_file, from_state].into_iter().flatten() {
        if found == Some(pid) {
            continue;
        }
        if proc_stat(pid).is_none_or(|(_, state)| state == 'Z') {
            continue;
        }
        let receipt = state.as_ref().ok_or("live serial pid has no receipt")?;
        if !serial_args_match(c, pid, receipt)? {
            return Err(Error::new(format!("serial pid {pid} does not match this bench, host and receipt; refusing to touch it")));
        }
        if found.is_some() {
            return Err("multiple serial listeners match this bench".into());
        }
        found = Some(pid);
    }
    Ok(found)
}
fn stop_serial(c: &Config, pid: u32) -> Result<()> {
    let fd = pidfd(pid)?;
    let before = proc_stat(pid).ok_or("serial process vanished")?.0;
    let receipt =
        read_json(&c.bench.join("serial-current.json"))?.ok_or("serial receipt disappeared")?;
    if !serial_args_match(c, pid, &receipt)? || proc_stat(pid).map(|x| x.0) != Some(before) {
        return Err("serial identity changed before stop".into());
    }
    pidfd_signal(&fd, libc::SIGTERM)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    while !pidfd_exited(&fd)? {
        if Instant::now() >= deadline {
            pidfd_signal(&fd, libc::SIGKILL)?;
            break;
        }
        thread::sleep(Duration::from_millis(100))
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while !pidfd_exited(&fd)? {
        if Instant::now() >= deadline {
            return Err("serial listener did not exit after SIGKILL".into());
        }
        thread::sleep(Duration::from_millis(100))
    }
    Ok(())
}
fn bootstrap(c: &Config, windows: bool) -> Result<PathBuf> {
    let mut command = Command::new(c.root.join("tools/bench-infra"));
    if windows {
        command.arg("--windows-path");
    } else {
        command.arg("--help");
    }
    command
        .current_dir(&c.root)
        .env_clear()
        .envs(&c.env)
        .stdin(Stdio::null());
    let text = checked(command, 600, "bench-infra bootstrap")?;
    let path = if windows {
        PathBuf::from(
            text.lines()
                .last()
                .ok_or("bench-infra did not print Windows binary path")?,
        )
    } else {
        c.root.join(".cache/vibeos-bench/linux/vibeos-bench")
    };
    if !path.is_file() {
        return Err(Error::new(format!(
            "bench-infra binary absent: {}",
            path.display()
        )));
    }
    Ok(path)
}
fn start_serial(c: &Config) -> Result<u32> {
    if c.mode != "pico" {
        return Err("start_serial only supports pico mode".into());
    }
    let binary = bootstrap(c, false)?;
    let bench = c.bench.canonicalize()?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(c.bench.join("pico-serial.log"))?;
    let mut cmd = Command::new(binary);
    cmd.arg("--root")
        .arg(&c.root)
        .arg("serial")
        .arg(&bench)
        .arg(&c.host)
        .current_dir(&c.root)
        .env_clear()
        .envs(&c.env)
        .env_remove("VIBEOS_ARTIFACT_OWNER_PID")
        .env_remove("VIBEOS_ARTIFACT_OWNER_START")
        .env_remove("VIBEOS_QEMU_OWNER_PID")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| Error::new(format!("start native serial listener: {e}")))?;
    let pid = child.id();
    let result = (|| -> Result<u32> {
        let fd = pidfd(pid)?;
        let mut temp = tempfile::NamedTempFile::new_in(&c.bench)?;
        writeln!(temp, "{pid}")?;
        temp.persist(c.bench.join("pico-serial.pid"))
            .map_err(|e| Error::new(format!("publish serial pid: {e}")))?;
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if child.try_wait()?.is_some() {
                return Err(Error::new(format!(
                    "serial listener exited; see {}",
                    c.bench.join("pico-serial.log").display()
                )));
            }
            if let Some(r) = read_json(&c.bench.join("serial-current.json"))? {
                if serial_args_match(c, pid, &r)?
                    && value(&r, "status") == "LISTENING"
                    && r["connected"] == true
                {
                    return Ok(pid);
                }
            }
            if Instant::now() >= deadline {
                pidfd_signal(&fd, libc::SIGTERM)?;
                return Err(Error::new(format!(
                    "serial did not connect within 30s; see {}",
                    c.bench.join("pico-serial.log").display()
                )));
            }
            thread::sleep(Duration::from_millis(200));
        }
    })();
    if result.is_err() {
        if child.try_wait()?.is_none() {
            child.kill()?;
            child.wait()?;
        }
    }
    result
}
// CIM CreationDate has microsecond precision; Process.StartTime retains 100ns ticks.
// Compare both at CIM precision while retaining the opened process handle for Kill.
fn stop_windows(p: &WinProcess) -> Result<()> {
    // Opening the Process handle before rechecking the CIM command and start time
    // binds Kill to that incarnation even if its numeric PID is later reused.
    let script=format!("$ErrorActionPreference='Stop'; $p=[System.Diagnostics.Process]::GetProcessById({pid}); $h=$p.Handle; if(($p.StartTime.ToUniversalTime().Ticks - ($p.StartTime.ToUniversalTime().Ticks % 10)) -ne {started}) {{ throw 'process incarnation changed' }}; $w=Get-CimInstance Win32_Process -Filter 'ProcessId={pid}'; if(-not $w -or ([datetime]$w.CreationDate).ToUniversalTime().Ticks -ne {started} -or [string]$w.ExecutablePath -cne {exe} -or [string]$w.CommandLine -cne {command}) {{ throw 'process identity changed' }}; $p.Kill(); if(-not $p.WaitForExit(10000)) {{ throw 'process failed to exit' }}",pid=p.pid,started=p.started,exe=psquote(&p.exe)?,command=psquote(&p.command)?);
    powershell(&script)?;
    Ok(())
}
fn endpoint(c: &Config, kind: Kind, p: &WinProcess) -> Result<bool> {
    let (name, bind, port) = match kind {
        Kind::Http => ("Get-NetTCPConnection", c.http_bind.as_str(), c.http_port),
        Kind::Tftp => ("Get-NetUDPEndpoint", "0.0.0.0", 69),
        Kind::Com => return Ok(true),
    };
    let filter = if kind == Kind::Http {
        " -State Listen"
    } else {
        ""
    };
    let script=format!("$ErrorActionPreference='Stop'; $p=[System.Diagnostics.Process]::GetProcessById({pid}); $h=$p.Handle; if(($p.StartTime.ToUniversalTime().Ticks - ($p.StartTime.ToUniversalTime().Ticks % 10)) -ne {started}) {{ throw 'process incarnation changed' }}; $w=Get-CimInstance Win32_Process -Filter 'ProcessId={pid}'; if(-not $w -or [string]$w.ExecutablePath -cne {exe} -or [string]$w.CommandLine -cne {command}) {{ throw 'process identity changed' }}; $x=@({name}{filter} -ErrorAction Stop | Where-Object {{ $_.LocalPort -eq {port} -and $_.OwningProcess -eq {pid} -and [string]$_.LocalAddress -eq {bind} }}); if($x.Count -gt 0 -and -not $p.HasExited) {{ 'ready' }} else {{ 'waiting' }}",pid=p.pid,started=p.started,exe=psquote(&p.exe)?,command=psquote(&p.command)?,bind=psquote(bind)?);
    Ok(powershell(&script)?.trim() == "ready")
}
fn firewall_script(c: &Config, kind: Kind) -> Result<String> {
    let (display, protocol, port) = match kind {
        Kind::Http => ("VibeOS Pi4 sanboot HTTP (bring-up)", "TCP", c.http_port),
        Kind::Tftp => ("VibeOS Pi4 TFTP (bring-up)", "UDP", 69),
        Kind::Com => return Err("COM does not have a LAN firewall rule".into()),
    };
    let script = format!(
        r#"$ErrorActionPreference='Stop'
$rules=@(Get-NetFirewallRule -DisplayName {display} -ErrorAction Stop)
if($rules.Count -ne 1) {{ throw 'expected exactly one named bench firewall rule' }}
$r=$rules[0]; if($r.Direction -ne 'Inbound' -or $r.Action -ne 'Allow' -or $r.Enabled -ne 'True') {{ throw 'bench firewall rule is not enabled inbound allow' }}
$port=@($r | Get-NetFirewallPortFilter); if($port.Count -ne 1 -or ([string]$port[0].Protocol -notin @({protocol},{number})) -or ([string]$port[0].LocalPort -notin @({port},{any}))) {{ throw 'bench firewall port filter does not match' }}
$app=@($r | Get-NetFirewallApplicationFilter); if($app.Count -ne 1) {{ throw 'bench firewall application filter is ambiguous' }}
$old=[string]$app[0].Program; if(-not [string]::Equals($old,{python},[StringComparison]::OrdinalIgnoreCase) -and -not [string]::Equals($old,{exe},[StringComparison]::OrdinalIgnoreCase)) {{ throw 'bench firewall rule points at an unexpected program' }}
$before=[pscustomobject]@{{rule=@($r | Select-Object Name,Enabled,Direction,Action,Profile); port=@($r | Get-NetFirewallPortFilter | Select-Object Protocol,LocalPort,RemotePort,IcmpType,DynamicTarget); address=@($r | Get-NetFirewallAddressFilter | Select-Object LocalAddress,RemoteAddress)}} | ConvertTo-Json -Depth 8 -Compress
if(-not [string]::Equals($old,{exe},[StringComparison]::OrdinalIgnoreCase)) {{ $r | Get-NetFirewallApplicationFilter | Set-NetFirewallApplicationFilter -Program {exe} -ErrorAction Stop }}
$r=Get-NetFirewallRule -Name $r.Name -ErrorAction Stop
$after=[pscustomobject]@{{rule=@($r | Select-Object Name,Enabled,Direction,Action,Profile); port=@($r | Get-NetFirewallPortFilter | Select-Object Protocol,LocalPort,RemotePort,IcmpType,DynamicTarget); address=@($r | Get-NetFirewallAddressFilter | Select-Object LocalAddress,RemoteAddress)}} | ConvertTo-Json -Depth 8 -Compress
if($before -cne $after -or -not [string]::Equals([string](($r | Get-NetFirewallApplicationFilter).Program),{exe},[StringComparison]::OrdinalIgnoreCase)) {{ throw 'firewall rule verification failed' }}
'updated'"#,
        display = psquote(display)?,
        protocol = psquote(protocol)?,
        number = psquote(if kind == Kind::Http { "6" } else { "17" })?,
        port = psquote(&port.to_string())?,
        any = psquote(if kind == Kind::Tftp {
            "Any"
        } else {
            "<not-any>"
        })?,
        python = psquote(&c.win_python)?,
        exe = psquote(&c.win_exe)?
    );
    Ok(script)
}
fn firewall(c: &Config, kind: Kind) -> Result<()> {
    powershell(&firewall_script(c, kind)?)?;
    Ok(())
}
fn install_binary(c: &Config) -> Result<Vec<Kind>> {
    let source = bootstrap(c, true)?;
    let dest = c.bench.join("bench-infra.exe");
    let data = fs::read(source)?;
    if fs::read(&dest).is_ok_and(|old| old == data) {
        return Ok(Vec::new());
    }
    // A running image cannot be replaced safely. Stop only the recognized,
    // incarnation-checked bench servers before publishing the new executable.
    let processes = c.ps()?;
    let mut stopped = Vec::new();
    for kind in [Kind::Tftp, Kind::Http] {
        let found = selected(c, kind, &processes)?;
        if !found.is_empty() {
            stopped.push(kind);
        }
        for p in found {
            stop_windows(&p)?;
        }
    }
    let mut temp = tempfile::NamedTempFile::new_in(&c.bench)?;
    temp.write_all(&data)?;
    temp.as_file().sync_all()?;
    temp.persist(&dest)
        .map_err(|e| Error::new(format!("deploy {}: {e}", dest.display())))?;
    Ok(stopped)
}
fn start_windows(c: &Config, kind: Kind) -> Result<WinProcess> {
    let (exe, args, log, errlog) = match kind {
        Kind::Http => (
            &c.win_exe,
            vec![
                "http".to_owned(),
                c.win_payload.clone(),
                c.http_bind.clone(),
                c.http_port.to_string(),
            ],
            "http-server.log",
            "http-server.err",
        ),
        Kind::Tftp => (
            &c.win_exe,
            vec![
                "tftp".to_owned(),
                c.win_payload.clone(),
                "0.0.0.0".into(),
                "69".into(),
            ],
            "tftp-server.log",
            "tftp-server.err",
        ),
        Kind::Com => {
            let ps = r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe";
            let script = format!(
                "{}\\Capture-RpiSerial.ps1",
                c.win_bench.trim_end_matches('\\')
            );
            return launch_windows(
                c,
                kind,
                ps,
                &[
                    "-NoProfile".into(),
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-File".into(),
                    script,
                    "-PortName".into(),
                    c.com.clone(),
                ],
                "com-serial.log",
                "com-serial.err",
            );
        }
    };
    launch_windows(c, kind, exe, &args, log, errlog)
}
// This child is the short-lived WSL interop launcher, not the persistent server.
// It may retain inherited Windows handles after PowerShell exits. Reap it only
// after the independent receipt proves the server's ownership and readiness.
struct WindowsLauncher {
    child: std::process::Child,
    descriptor: File,
}
impl WindowsLauncher {
    fn spawn(script: &str, log: &Path) -> Result<Self> {
        let errors = OpenOptions::new().create(true).append(true).open(log)?;
        let mut cmd = Command::new(POWERSHELL);
        cmd.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(errors);
        let parent = std::process::id();
        unsafe {
            cmd.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::getppid() != parent as i32 {
                    libc::_exit(125);
                }
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = cmd.spawn()?;
        let descriptor = match pidfd(child.id()) {
            Ok(fd) => fd,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        };
        Ok(Self { child, descriptor })
    }
    fn stop(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_none() {
            pidfd_signal(&self.descriptor, libc::SIGKILL)?;
            self.child.wait()?;
        }
        Ok(())
    }
}
impl Drop for WindowsLauncher {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn launch_windows(
    c: &Config,
    kind: Kind,
    exe: &str,
    args: &[String],
    log: &str,
    errlog: &str,
) -> Result<WinProcess> {
    let argline = args
        .iter()
        .map(|s| windows_arg(s))
        .collect::<Result<Vec<_>>>()?
        .join(" ");
    let out = format!("{}\\{log}", c.win_bench.trim_end_matches('\\'));
    let errout = format!("{}\\{errlog}", c.win_bench.trim_end_matches('\\'));
    // Keep the unique pathname, but close its Linux handle before Windows writes it.
    let attempt = tempfile::NamedTempFile::new_in(&c.bench)?.into_temp_path();
    let attempt_windows = winpath(&attempt)?;
    let receipt = winpath(&c.receipt(kind.key()))?;
    // Publish ownership before ending the launcher. PowerShell can otherwise
    // remain alive waiting for a detached Start-Process child under WSL interop.
    let script = format!(
        r#"$ErrorActionPreference='Stop'
$p=Start-Process -FilePath {exe} -ArgumentList {args} -WindowStyle Hidden -RedirectStandardOutput {out} -RedirectStandardError {errout} -PassThru
$handle=$p.Handle
$t=$p.StartTime.ToUniversalTime().Ticks; $t=$t-($t%10)
$w=Get-CimInstance Win32_Process -Filter "ProcessId=$($p.Id)"
if(-not $w -or $p.HasExited -or ([datetime]$w.CreationDate).ToUniversalTime().Ticks -ne $t -or -not [string]::Equals([string]$w.ExecutablePath,{exe},[StringComparison]::OrdinalIgnoreCase)) {{ throw 'launched process identity unavailable' }}
$r=[pscustomobject]@{{pid=[int]$p.Id;started=$t;exe=[string]$w.ExecutablePath;command=[string]$w.CommandLine}}
$json=$r | ConvertTo-Json -Compress
$receipt={receipt}; $temp=$receipt+'.'+$PID+'.tmp'
[IO.File]::WriteAllText($temp,$json,[Text.UTF8Encoding]::new($false))
if([IO.File]::Exists($receipt)) {{ [IO.File]::Replace($temp,$receipt,$null) }} else {{ [IO.File]::Move($temp,$receipt) }}
[IO.File]::WriteAllText({attempt},$json,[Text.UTF8Encoding]::new($false))
[Console]::Out.WriteLine($json); [Console]::Out.Flush()
[Environment]::Exit(0)"#,
        exe = psquote(exe)?,
        args = psquote(&argline)?,
        out = psquote(&out)?,
        errout = psquote(&errout)?,
        receipt = psquote(&receipt)?,
        attempt = psquote(&attempt_windows)?
    );

    let launch_log = c.bench.join(format!("{}-launch.err", kind.key()));
    let mut launcher = WindowsLauncher::spawn(&script, &launch_log)?;
    let handoff_deadline = Instant::now() + Duration::from_secs(15);
    let value: Value = loop {
        // An incomplete write is pending, never a valid ownership record.
        if let Ok(bytes) = fs::read(&attempt) {
            if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                break v;
            }
        }
        if launcher.child.try_wait()?.is_some() || Instant::now() >= handoff_deadline {
            return Err(Error::new(format!(
                "Windows launcher did not publish ownership; see {}: {}",
                launch_log.display(),
                fs::read_to_string(&launch_log)
                    .unwrap_or_default()
                    .chars()
                    .take(3000)
                    .collect::<String>()
            )));
        }
        thread::sleep(Duration::from_millis(50));
    };
    let pid = value["pid"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("launch did not report pid")?;
    let started = value["started"]
        .as_i64()
        .ok_or("launch did not report process start")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut owned: Option<WinProcess> = None;
    let result = (|| -> Result<WinProcess> {
        loop {
            let processes = c.ps()?;
            if let Some(p) = processes
                .iter()
                .find(|p| p.pid == pid && p.started == started)
            {
                if !recognized(c, p, kind) {
                    return Err(Error::new(format!(
                        "launched {} pid {pid} has unexpected arguments",
                        kind.key()
                    )));
                }
                if owned.is_none() {
                    owned = Some(p.clone());
                    atomic_json(&c.receipt(kind.key()), &p.json())?;
                }
                let ready = if kind == Kind::Com {
                    com_ready(c, p)?
                } else {
                    endpoint(c, kind, p)?
                };
                if ready {
                    launcher.stop()?;
                    return Ok(p.clone());
                }
            } else if owned.is_some() {
                return Err(Error::new(format!(
                    "{} process exited before readiness; see {errout}",
                    kind.key()
                )));
            }
            if Instant::now() >= deadline {
                return Err(Error::new(format!(
                    "{} listener was not ready in 30s; see {errout}",
                    kind.key()
                )));
            }
            thread::sleep(Duration::from_millis(250));
        }
    })();
    match result {
        Ok(p) => Ok(p),
        Err(error) => {
            if let Some(p) = owned.as_ref() {
                let still = c.ps()?.into_iter().any(|q| {
                    q.pid == p.pid
                        && q.started == p.started
                        && q.exe == p.exe
                        && q.command == p.command
                });
                if still {
                    stop_windows(p).map_err(|cleanup| {
                        Error::new(format!("{error}; cleanup also failed: {cleanup}"))
                    })?;
                }
            } else {
                return Err(Error::new(format!(
                    "{error}; could not verify the launched pid {pid}, inspect it before retrying"
                )));
            }
            Err(error)
        }
    }
}
fn com_ready(c: &Config, p: &WinProcess) -> Result<bool> {
    let Some(state) = read_json(&c.bench.join("serial-current.json"))? else {
        return Ok(false);
    };
    let path = value(&state, "directory").replace('/', "\\");
    let prefix = format!(
        "{}\\",
        c.win_bench.replace('/', "\\").trim_end_matches('\\')
    );
    Ok(state["pid"].as_u64() == Some(p.pid as u64)
        && value(&state, "status") == "LISTENING"
        && value(&state, "port").eq_ignore_ascii_case(&c.com)
        && path.to_lowercase().starts_with(&prefix.to_lowercase()))
}
fn ensure_server(c: &Config, kind: Kind) -> Result<u32> {
    firewall(c, kind)?;
    let found = selected(c, kind, &c.ps()?)?;
    if found.len() > 1 {
        return Err(Error::new(format!(
            "multiple {} processes match this bench",
            kind.key()
        )));
    }
    if let Some(p) = found.first() {
        if winpath_eq(&p.exe, &c.win_exe) && endpoint(c, kind, p)? {
            return Ok(p.pid);
        }
        stop_windows(p)?;
    }
    Ok(start_windows(c, kind)?.pid)
}
fn ensure_serial(c: &Config) -> Result<u32> {
    match c.mode.as_str() {
        "pico" => {
            if let Some(pid) = serial_process(c)? {
                let state = read_json(&c.bench.join("serial-current.json"))?
                    .ok_or("missing serial state")?;
                if value(&state, "engine") == "vibeos-bench"
                    && value(&state, "status") == "LISTENING"
                    && state["connected"] == true
                {
                    return Ok(pid);
                }
                stop_serial(c, pid)?;
            }
            start_serial(c)
        }
        "com" => {
            fs::copy(
                c.root.join("tools/pi4/Capture-RpiSerial.ps1"),
                c.bench.join("Capture-RpiSerial.ps1"),
            )?;
            let found = selected(c, Kind::Com, &c.ps()?)?;
            if found.len() > 1 {
                return Err("multiple COM listeners match this bench".into());
            }
            if let Some(p) = found.first() {
                if com_ready(c, p)? {
                    return Ok(p.pid);
                }
                stop_windows(p)?;
            }
            Ok(start_windows(c, Kind::Com)?.pid)
        }
        _ => Err(Error::new(format!(
            "unsupported PI4_SERIAL {:?}; choose pico or com",
            c.mode
        ))),
    }
}
fn display_status(c: &Config, host: &mut dyn Host) -> Result<Value> {
    let all = c.ps()?;
    let serial = match c.mode.as_str() {
        "pico" => serial_process(c)?
            .map(|p| p.to_string())
            .unwrap_or("down".into()),
        "com" => {
            let mut ready = Vec::new();
            for p in selected(c, Kind::Com, &all)? {
                if com_ready(c, &p)? {
                    ready.push(p.pid.to_string())
                }
            }
            ready.join(" ")
        }
        _ => return Err(Error::new(format!("unsupported PI4_SERIAL {:?}", c.mode))),
    };
    emit(
        host,
        format!(
            "serial listener : {} ({})",
            if serial.is_empty() { "down" } else { &serial },
            c.mode
        ),
    )?;
    match read_json(&c.bench.join("serial-current.json"))? {
        Some(s) => emit(
            host,
            format!(
                "  last state    : {} {} {}",
                value(&s, "status"),
                value(&s, "directory"),
                value(&s, "error")
            ),
        )?,
        None => emit(host, "  last state    : none")?,
    }
    for kind in [Kind::Tftp, Kind::Http] {
        let mut ready = Vec::new();
        for p in selected(c, kind, &all)? {
            if endpoint(c, kind, &p)? {
                ready.push(p.pid.to_string())
            }
        }
        emit(
            host,
            format!(
                "{} server     : {}",
                kind.key(),
                if ready.is_empty() {
                    "down".into()
                } else {
                    ready.join(" ")
                }
            ),
        )?;
    }
    let mut command = Command::new("bash");
    command
        .arg(c.root.join("tools/pi4/power.sh"))
        .arg("status")
        .current_dir(&c.root)
        .env_clear()
        .envs(&c.env);
    let power = native_run(command, 60)?;
    if power.code != 0 {
        return Err(Error::new(format!(
            "power status failed: {}",
            power.stderr.trim()
        )));
    }
    emit(host, format!("power           : {}", power.stdout.trim()))?;
    Ok(json!(0))
}
fn up(c: &Config, host: &mut dyn Host) -> Result<Value> {
    c.lease("vibeos-cli bench services up")?;
    fs::create_dir_all(c.bench.join("payload"))?;
    install_binary(c)?;
    let serial = ensure_serial(c)?;
    emit(host, format!("serial listener : {serial} ({})", c.mode))?;
    let tftp = ensure_server(c, Kind::Tftp)?;
    emit(host, format!("tftp server     : {tftp}"))?;
    let http = ensure_server(c, Kind::Http)?;
    emit(
        host,
        format!(
            "http server on {}:{} (pid {http})",
            c.http_bind, c.http_port
        ),
    )?;
    Ok(json!(0))
}
fn http(c: &Config, host: &mut dyn Host) -> Result<Value> {
    c.lease("vibeos-cli bench HTTP service")?;
    fs::create_dir_all(c.bench.join("payload"))?;
    let stopped = install_binary(c)?;
    if stopped.contains(&Kind::Tftp) {
        let pid = ensure_server(c, Kind::Tftp)?;
        emit(host, format!("tftp server     : {pid}"))?;
    }
    let pid = ensure_server(c, Kind::Http)?;
    emit(
        host,
        format!("http server on {}:{} (pid {pid})", c.http_bind, c.http_port),
    )?;
    Ok(json!(0))
}
fn down(c: &Config, host: &mut dyn Host) -> Result<Value> {
    c.lease("vibeos-cli bench services down")?;
    if let Some(pid) = serial_process(c)? {
        stop_serial(c, pid)?;
        emit(host, format!("stopped serial listener ({pid})"))?;
    }
    for kind in [Kind::Com, Kind::Tftp, Kind::Http] {
        for p in selected(c, kind, &c.ps()?)? {
            stop_windows(&p)?;
            emit(host, format!("stopped {} ({})", kind.key(), p.pid))?;
        }
    }
    Ok(json!(0))
}
fn serial_restart(c: &Config, host: &mut dyn Host) -> Result<Value> {
    c.lease("vibeos-cli bench serial restart")?;
    match c.mode.as_str() {
        "pico" => {
            if let Some(pid) = serial_process(c)? {
                stop_serial(c, pid)?;
                emit(host, format!("stopped serial listener ({pid})"))?
            }
        }
        "com" => {
            for p in selected(c, Kind::Com, &c.ps()?)? {
                stop_windows(&p)?;
                emit(host, format!("stopped COM serial listener ({})", p.pid))?
            }
        }
        _ => return Err(Error::new(format!("unsupported PI4_SERIAL {:?}", c.mode))),
    }
    up(c, host)
}
fn serial_up(c: &Config, host: &mut dyn Host) -> Result<Value> {
    if c.host.is_empty() || c.host.eq_ignore_ascii_case("none.invalid") {
        return Err(Error::new(format!(
            "{} Pico host is not configured",
            c.board
        )));
    }
    c.lease("vibeos-cli bench serial up")?;
    fs::create_dir_all(&c.bench)?;
    let pid = ensure_serial(c)?;
    emit(host, format!("serial listener: {pid} ({})", c.host))?;
    Ok(json!(0))
}
fn serial_down(c: &Config, host: &mut dyn Host) -> Result<Value> {
    c.lease("vibeos-cli bench serial down")?;
    if let Some(pid) = serial_process(c)? {
        stop_serial(c, pid)?;
        emit(host, format!("stopped serial listener ({pid})"))?;
    }
    Ok(json!(0))
}
fn serial_status(c: &Config, host: &mut dyn Host) -> Result<Value> {
    let pid = serial_process(c)?;
    emit(
        host,
        match pid {
            Some(pid) => format!("serial listener: {pid} ({})", c.host),
            None => "serial listener: down".into(),
        },
    )?;
    if let Some(state) = read_json(&c.bench.join("serial-current.json"))? {
        emit(host, serde_json::to_string_pretty(&state)?)?;
    }
    Ok(json!(0))
}
/// Explicit recovery for an interrupted launch, never an automatic adoption.
fn recover(c: &Config, a: &Value, host: &mut dyn Host) -> Result<Value> {
    c.lease("recover interrupted native bench launch")?;
    let pid = a["pid"].as_u64().ok_or("recover requires --pid")?;
    let started = a["started"]
        .as_i64()
        .ok_or("recover requires --started (CIM UTC ticks)")?;
    let processes = c.ps()?;
    let p = processes
        .iter()
        .find(|p| u64::from(p.pid) == pid && p.started == started)
        .ok_or("recovery process incarnation does not match")?;
    if !winpath_eq(&p.exe, &c.win_exe) {
        return Err("recovery requires this bench's native executable".into());
    }
    let kind = [Kind::Tftp, Kind::Http]
        .into_iter()
        .find(|k| recognized(c, p, *k))
        .ok_or("recovery process arguments do not match this bench and service")?;
    if let Some(old) = read_json(&c.receipt(kind.key()))? {
        if processes
            .iter()
            .any(|live| receipt_matches(&old, live) && !receipt_matches(&old, p))
        {
            return Err("another live process already owns the service receipt".into());
        }
    }
    atomic_json(&c.receipt(kind.key()), &p.json())?;
    emit(
        host,
        format!(
            "recovered {} ownership for pid {} at {}",
            kind.key(),
            p.pid,
            p.started
        ),
    )?;
    Ok(json!(0))
}

pub fn dispatch(root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = request.get("kwargs").unwrap_or(request);
    let op = value(request, "op");
    let linux_only = matches!(
        op,
        "bench_service_serial_up" | "bench_service_serial_down" | "bench_service_serial_status"
    );
    let c = Config::new(root, args, linux_only)?;
    // A lease is shared by commands from one worktree. Serialize its service
    // changes as well so two calls cannot publish competing listener receipts.
    let _mutation = if !op.ends_with("status") {
        use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
        c.lease("native bench service control")?;
        let dir = vibeos_bench::lease::lock_dir(&c.env);
        fs::create_dir_all(&dir)?;
        let name = vibeos_bench::lease::lease_name(&c.env).map_err(err)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(dir.join(format!("{name}-services.lock")))?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Some(lock)
    } else {
        None
    };

    match op {
        "bench_service_recover" => recover(&c, args, host),
        "bench_service_status" => display_status(&c, host),
        "bench_service_up" => up(&c, host),
        "bench_service_http" => http(&c, host),
        "bench_service_down" => down(&c, host),
        "bench_service_serial_restart" => serial_restart(&c, host),
        "bench_service_serial_up" => serial_up(&c, host),
        "bench_service_serial_down" => serial_down(&c, host),
        "bench_service_serial_status" => serial_status(&c, host),
        _ => Err(Error::new(format!("unknown bench service operation {op}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_launch_handoff_outlives_its_interop_launcher() {
        if !Path::new(POWERSHELL).is_file() {
            return;
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let win_temp = powershell("[IO.Path]::GetTempPath()").unwrap();
        let mut convert = Command::new("wslpath");
        convert.args(["-u", win_temp.trim()]);
        let parent = checked(convert, 10, "temporary Windows path").unwrap();
        let temp = tempfile::Builder::new()
            .prefix("vibeos-bench-handoff-")
            .tempdir_in(parent)
            .unwrap();
        let mut c = config(temp.path());
        c.root = root;
        c.env = std::env::vars().collect();
        c.win_bench = winpath(&c.bench).unwrap();
        c.win_payload = format!("{}\\payload", c.win_bench);
        c.win_exe = format!("{}\\bench-infra.exe", c.win_bench);
        c.http_bind = "127.0.0.1".into();
        c.http_port = powershell("$s=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$s.Start();$port=$s.LocalEndpoint.Port;$s.Stop();$port").unwrap().parse().unwrap();
        fs::create_dir_all(c.bench.join("payload")).unwrap();
        fs::write(c.bench.join("payload/image"), b"0123456789").unwrap();
        fs::copy(
            bootstrap(&c, true).unwrap(),
            c.bench.join("bench-infra.exe"),
        )
        .unwrap();
        let now = Instant::now();
        let started = start_windows(&c, Kind::Http);
        let p = match started {
            Ok(p) => p,
            Err(error) => {
                if let Ok(Some(receipt)) = read_json(&c.receipt("http")) {
                    for live in c.ps().unwrap_or_default() {
                        if receipt_matches(&receipt, &live) {
                            stop_windows(&live).unwrap();
                        }
                    }
                }
                let kept = temp.keep();
                panic!("{error}; fixture retained at {}", kept.display());
            }
        };
        let result = (|| -> Result<()> {
            if now.elapsed() >= Duration::from_secs(25) {
                return Err("launch waited for its persistent child".into());
            }
            if !receipt_matches(
                &read_json(&c.receipt("http"))?.ok_or("ownership not published")?,
                &p,
            ) {
                return Err("ownership receipt differs from live process".into());
            }
            let body = powershell(&format!("$r=[Net.WebRequest]::Create('http://127.0.0.1:{}/image');$r.Timeout=2000;$response=$r.GetResponse();try{{$reader=[IO.StreamReader]::new($response.GetResponseStream());$reader.ReadToEnd()}}finally{{$response.Dispose()}}", c.http_port))?;
            if body != "0123456789" {
                return Err("server did not survive launcher handoff".into());
            }
            Ok(())
        })();
        let cleanup = stop_windows(&p);
        result.unwrap();
        cleanup.unwrap();
    }

    fn config(root: &Path) -> Config {
        let bench = root.join("bench");
        fs::create_dir_all(&bench).unwrap();
        Config {
            root: root.into(),
            board: "pi4".into(),
            env: Env::from([("HOME".into(), root.to_string_lossy().into_owned())]),
            bench,
            win_bench: r"D:\home\bench".into(),
            win_payload: r"D:\home\bench\payload".into(),
            win_exe: r"D:\home\bench\bench-infra.exe".into(),
            win_python: r"C:\Python\python.exe".into(),
            host: "picousb".into(),
            http_bind: "192.168.1.183".into(),
            http_port: 8069,
            serial_port: 8082,
            mode: "pico".into(),
            com: "COM5".into(),
        }
    }
    fn process(command: &str, exe: &str) -> WinProcess {
        WinProcess {
            pid: 123,
            started: 77,
            exe: exe.into(),
            command: command.into(),
        }
    }
    #[test]
    fn native_executable_location_accepts_deleted_cache_and_binds_its_worktree() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first");
        let second = dir.path().join("second");
        for root in [&first, &second] {
            fs::create_dir_all(root.join(".cache/vibeos-bench/linux")).unwrap();
        }
        let target = first.join(".cache/vibeos-bench/linux/vibeos-bench");
        let deleted = PathBuf::from(format!("{} (deleted)", target.display()));
        let normalized = normalize_exe_link(&deleted).unwrap();
        assert!(native_location_matches(&first, &normalized));
        assert!(!native_location_matches(&second, &normalized));
        let registered = vec![first.clone()];
        assert!(native_owner_matches(&registered, &first, &normalized));
        assert!(!native_owner_matches(&registered, &second, &normalized));
        assert!(!native_owner_matches(&[], &first, &normalized));
        assert_eq!(normalize_exe_link(&target), Some(normalized));
    }

    #[test]
    fn pi5_serial_selection_never_falls_back_to_pi4() {
        let d = tempfile::tempdir().unwrap();
        let args = json!({"board":"pi5","env":{"PI4_BENCH":"/wrong/pi4","PI4_PICO_HOST":"picousb","VIBEOS_BENCH_BOARD":"pi4"}});
        let c = Config::new(d.path(), &args, true).unwrap();
        assert_eq!(c.board, "pi5");
        assert_eq!(c.bench, PathBuf::from("/mnt/d/home/vibeos-rpi5-setup"));
        assert_eq!(c.host, "none.invalid");
        assert_eq!(c.env["VIBEOS_BENCH_BOARD"], "pi5");
        let args =
            json!({"board":"pi5","env":{"PI5_BENCH":"/bench/five","PI5_PICO_HOST":"pico-five"}});
        let c = Config::new(d.path(), &args, true).unwrap();
        assert_eq!(c.bench, PathBuf::from("/bench/five"));
        assert_eq!(c.host, "pico-five");
    }
    #[test]
    fn windows_parser_roundtrips_spaces_quotes_and_trailing_backslashes() {
        let args = [
            r"D:\home\bench infra\bench-infra.exe",
            r"D:\payload path\\",
            r#"a\"b"#,
            "http",
        ];
        let cmd = args
            .iter()
            .map(|s| windows_arg(s).unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(split_windows(&cmd), args);
    }
    #[test]
    fn process_identity_includes_root_protocol_port_exe_and_incarnation() {
        let d = tempfile::tempdir().unwrap();
        let c = config(d.path());
        let good = process(
            r#""D:\home\bench\bench-infra.exe" "http" "D:\home\bench\payload" "192.168.1.183" "8069""#,
            &c.win_exe,
        );
        assert!(recognized(&c, &good, Kind::Http));
        assert!(!recognized(&c, &good, Kind::Tftp));
        assert!(receipt_matches(&good.json(), &good));
        let mut changed = good.clone();
        changed.started += 1;
        assert!(!receipt_matches(&good.json(), &changed));
        let other = process(&good.command.replace("8069", "8070"), &c.win_exe);
        assert!(!recognized(&c, &other, Kind::Http));
        let legacy = process(
            r#""C:\Python\python.exe" -u "D:\home\bench\serve_tftp.py" "D:\home\bench\payload" 0.0.0.0"#,
            &c.win_python,
        );
        assert!(recognized(&c, &legacy, Kind::Tftp));
        assert!(!recognized(&c, &legacy, Kind::Http));
        let foreign = process(
            &legacy
                .command
                .replace("D:\\home\\bench\\payload", "D:\\other\\payload"),
            &c.win_python,
        );
        assert!(!recognized(&c, &foreign, Kind::Tftp));
    }
    #[test]
    fn firewall_script_targets_only_named_existing_rules_and_changes_program() {
        let d = tempfile::tempdir().unwrap();
        let c = config(d.path());
        let s = firewall_script(&c, Kind::Tftp).unwrap();
        assert!(s.contains("VibeOS Pi4 TFTP (bring-up)"));
        assert!(s.contains("Set-NetFirewallApplicationFilter -Program"));
        assert!(s.contains("LocalAddress,RemoteAddress"));
        assert!(!s.contains("New-NetFirewallRule"));
    }
    #[test]
    fn proc_stat_accepts_spaces_and_close_parentheses() {
        let s =
            "19 (name with ) parentheses) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 4321 20";
        assert_eq!(proc_ticks(s), Some(4321));
    }
}
