//! Owned QEMU sessions and independent lifetime guardians.
use crate::{
    admission::{port_handshake, Admission},
    backend,
    protocol::{QmpClient, SerialClient},
};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::FromRawFd,
        unix::process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn dur(value: f64) -> Result<Duration> {
    if !value.is_finite() || value < 0.0 {
        return Err("timeout must be finite and nonnegative".into());
    }
    Ok(Duration::from_secs_f64(value))
}
fn tick(pid: u32) -> Option<u64> {
    let s = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    s.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}
fn alive(pid: u32, start: u64) -> bool {
    if tick(pid) != Some(start) {
        return false;
    }
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|s| {
            s.rsplit_once(')')
                .and_then(|(_, tail)| tail.split_whitespace().next())
                .map(|state| state != "Z" && state != "X")
        })
        .unwrap_or(false)
}
fn atomic_json(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().ok_or("receipt has no parent")?;
    fs::create_dir_all(parent)?;
    let mut f = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut f, value)?;
    f.write_all(b"\n")?;
    f.flush()?;
    f.persist(path)
        .map_err(|e| Error::new(format!("publish {}: {}", path.display(), e.error)))?;
    Ok(())
}
fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn active_group(group: u32) -> Result<bool> {
    for e in fs::read_dir("/proc")? {
        let e = e?;
        let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        if pid == group {
            continue;
        }
        let s = match fs::read_to_string(e.path().join("stat")) {
            Ok(v) => v,
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound
                    || e.raw_os_error() == Some(libc::ESRCH) =>
            {
                continue
            }
            Err(e) => return Err(e.into()),
        };
        let fields: Vec<_> = s
            .rsplit_once(')')
            .ok_or("invalid owned group process stat")?
            .1
            .split_whitespace()
            .collect();
        if fields.get(2).and_then(|s| s.parse::<u32>().ok()) == Some(group)
            && !matches!(fields.first(), Some(&"Z" | &"X"))
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn stop_group(group: u32) -> Result<()> {
    let start = Instant::now();
    loop {
        let signal = if start.elapsed() < Duration::from_secs(5) {
            libc::SIGTERM
        } else {
            libc::SIGKILL
        };
        if unsafe { libc::kill(-(group as i32), signal) } != 0
            && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
        {
            return Err(std::io::Error::last_os_error().into());
        }
        if !active_group(group)? {
            break;
        }
        if start.elapsed() > Duration::from_secs(12) {
            return Err(Error::new(format!("owned QEMU group {group} did not stop")));
        }
        thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}
fn child_exited(pid: u32) -> Result<bool> {
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let rc = unsafe {
        libc::waitid(
            libc::P_PID,
            pid,
            &mut info,
            libc::WEXITED | libc::WNOWAIT | libc::WNOHANG,
        )
    };
    if rc < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { info.si_pid() } != 0)
}

/// Independent guardian survives API-server SIGTERM/SIGKILL. QEMU is its private
/// session leader and remains unreaped until group cleanup is complete, reserving
/// the numeric group identity through signals. Only the recorded UID is touched
/// on Windows; an unconfirmed stop retains the global lock/staging evidence.
pub fn guardian_worker(descriptor: &Path) -> Result<u8> {
    let mut config = read_json(descriptor)?;
    let mut admission_files = Vec::new();
    if let Some(fds) = config["lease_fds"].as_array() {
        for fd in fds {
            let fd = fd.as_i64().ok_or("invalid inherited lease fd")? as i32;
            if fd < 3 {
                return Err("lease fd must be >=3".into());
            }
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
            if flags < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            admission_files.push(unsafe { File::from_raw_fd(fd) });
        }
    }
    let argv: Vec<String> = serde_json::from_value(config["argv"].clone())?;
    if argv.is_empty() {
        return Err("guardian requires QEMU argv".into());
    }
    let status = PathBuf::from(
        config["status"]
            .as_str()
            .ok_or("guardian status path missing")?,
    );
    let stderr = PathBuf::from(
        config["stderr"]
            .as_str()
            .ok_or("guardian stderr path missing")?,
    );
    if let Some(p) = stderr.parent() {
        fs::create_dir_all(p)?;
    }
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(File::create(&stderr)?);
    if let Some(cwd) = config["cwd"].as_str().filter(|s| !s.is_empty()) {
        command.current_dir(cwd);
    }
    if let Some(e) = config["env"].as_object() {
        for (k, v) in e {
            if let Some(v) = v.as_str() {
                command.env(k, v);
            }
        }
    }
    let guardian = std::process::id();
    unsafe {
        command.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::getppid() != guardian as i32 {
                libc::_exit(125);
            }
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            atomic_json(
                &status,
                &json!({"state":"failed","error":format!("cannot launch {}: {e}",argv[0]),"exit_code":127}),
            )?;
            return Ok(127);
        }
    };
    let pid = child.id();
    let start = tick(pid).ok_or("cannot read launched QEMU start tick")?;
    atomic_json(
        &status,
        &json!({"state":"running","pid":pid,"start_tick":start,"guardian_pid":guardian,"started":now()}),
    )?;
    let mut reason = "exit";
    loop {
        if child_exited(pid)? {
            break;
        }
        config = match read_json(descriptor) {
            Ok(c) => c,
            Err(e) => {
                reason = "descriptor unavailable";
                let _ = atomic_json(
                    &status,
                    &json!({"state":"stopping","pid":pid,"start_tick":start,"error":e.message}),
                );
                break;
            }
        };
        if config["detached"].as_bool() == Some(true) {
            admission_files.clear();
        }
        let owner = config["owner_pid"].as_u64().unwrap_or(0) as u32;
        let owner_start = config["owner_tick"].as_u64().unwrap_or(0);
        let server = config["server_pid"].as_u64().unwrap_or(0) as u32;
        let server_start = config["server_tick"].as_u64().unwrap_or(0);
        if config["stop"].as_bool() == Some(true) {
            reason = "stop requested";
            break;
        }
        if !alive(owner, owner_start) {
            if read_json(descriptor).is_ok_and(|latest| latest != config) {
                continue;
            }
            reason = "owner exited";
            break;
        }
        if config["detached"].as_bool() != Some(true) && !alive(server, server_start) {
            if read_json(descriptor).is_ok_and(|latest| latest != config) {
                continue;
            }
            reason = "API server exited";
            break;
        }
        if now() >= config["deadline"].as_f64().unwrap_or(0.0) {
            if read_json(descriptor).is_ok_and(|latest| latest != config) {
                continue;
            }
            reason = "lifetime exceeded";
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    // Signal while child is live or an unreaped zombie: PID/group reuse is impossible.
    let cleanup = stop_group(pid);
    if cleanup.is_err() {
        unsafe { libc::kill(-(pid as i32), libc::SIGKILL) };
    }
    let state = child.wait()?;
    cleanup?;
    let uid = config["uid"].as_str().unwrap_or("");
    let mut left = if !uid.is_empty() {
        backend::kill_by_uid(uid).unwrap_or(None)
    } else {
        Some(0)
    };
    while left != Some(0) {
        atomic_json(
            &status,
            &json!({"state":"stopping","pid":pid,"start_tick":start,"reason":reason,"windows_left":left,"confirmed":false}),
        )?;
        thread::sleep(Duration::from_secs(2));
        left = backend::kill_by_uid(uid).unwrap_or(None);
    }
    if let Some(p) = config["lock_path"].as_str() {
        let _ = fs::remove_file(p);
    }
    let code = state.code().unwrap_or_else(|| -state.signal().unwrap_or(1));
    atomic_json(
        &status,
        &json!({"state":"stopped","pid":pid,"start_tick":start,"exit_code":code,"reason":reason,"windows_left":left,"confirmed":left==Some(0)}),
    )?;
    // QEMU is confirmed dead: release admission, but preserve capture media until
    // export explicitly releases it. The guardian still handles owner/API death.
    admission_files.clear();
    while reason == "exit" && config["retain_staging_until_release"].as_bool() == Some(true) {
        config = match read_json(descriptor) {
            Ok(c) => c,
            Err(_) => break,
        };
        if config["stop"].as_bool() == Some(true)
            || !alive(
                config["owner_pid"].as_u64().unwrap_or(0) as u32,
                config["owner_tick"].as_u64().unwrap_or(0),
            )
            || !alive(
                config["server_pid"].as_u64().unwrap_or(0) as u32,
                config["server_tick"].as_u64().unwrap_or(0),
            )
            || now() >= config["deadline"].as_f64().unwrap_or(0.0)
        {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    if let Some(p) = config["staging"].as_str() {
        let _ = fs::remove_dir_all(p);
    }
    Ok(if left == Some(0) { 0 } else { 1 })
}

fn default_boot_timeout() -> f64 {
    360.0
}
fn default_qmp_timeout() -> f64 {
    10.0
}
fn default_marker() -> String {
    "[EXPLORER] _tWinMain entered".into()
}
fn default_backend() -> String {
    "native".into()
}
fn default_min_bytes() -> usize {
    1000
}
fn default_settle() -> f64 {
    15.0
}
fn default_lifetime() -> f64 {
    1800.0
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmConfig {
    pub arch: String,
    pub qemu_bin: String,
    #[serde(default)]
    pub arch_args: Vec<String>,
    #[serde(default)]
    pub common_args: Vec<String>,
    #[serde(default)]
    pub disk_args: Vec<String>,
    #[serde(default)]
    pub network_args: Vec<String>,
    #[serde(default)]
    pub serial_args: Vec<String>,
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default)]
    pub windows: Value,
    pub work_dir: PathBuf,
    #[serde(default)]
    pub live_log: PathBuf,
    #[serde(default = "default_marker")]
    pub boot_marker: String,
    #[serde(default = "default_boot_timeout")]
    pub boot_timeout: f64,
    #[serde(default = "default_qmp_timeout")]
    pub qmp_timeout: f64,
    #[serde(default = "default_min_bytes")]
    pub min_boot_bytes: usize,
    #[serde(default = "default_settle")]
    pub desktop_settle_seconds: f64,
    #[serde(default)]
    pub hang_silence_seconds: f64,
    #[serde(default = "default_lifetime")]
    pub max_lifetime: f64,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub owner_pid: u32,
    #[serde(default)]
    pub gdb: bool,
    #[serde(default)]
    pub gdb_wait: bool,
    #[serde(default)]
    pub gdb_port: u16,
    #[serde(default)]
    pub memory: String,
    #[serde(default)]
    pub required_milestones: Vec<String>,
}
struct EchoServer {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    port: u16,
}
impl EchoServer {
    fn new() -> Result<Self> {
        let listener = std::net::TcpListener::bind(("0.0.0.0", 0))?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let h = thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let closed = flag.clone();
                        thread::spawn(move || {
                            let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
                            let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
                            let mut buf = [0u8; 65536];
                            while !closed.load(Ordering::Acquire) {
                                match stream.read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        if stream.write_all(&buf[..n]).is_err() {
                                            break;
                                        }
                                    }
                                    Err(e)
                                        if matches!(
                                            e.kind(),
                                            std::io::ErrorKind::WouldBlock
                                                | std::io::ErrorKind::TimedOut
                                        ) => {}
                                    Err(_) => break,
                                }
                            }
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            stop,
            thread: Some(h),
            port,
        })
    }
}
impl Drop for EchoServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(h) = self.thread.take() {
            let _ = h.join();
        }
    }
}

pub struct Vm {
    pub config: VmConfig,
    pub serial: SerialClient,
    pub qmp: QmpClient,
    pub boot_output: String,
    pub launch_timings: BTreeMap<String, f64>,
    pub gdb_host: String,
    pub gdb_port: u16,
    pub net_host: String,
    pub pid: Option<u32>,
    pub uid: String,
    pub staging: Option<PathBuf>,
    repo: PathBuf,
    guardian: Option<Child>,
    descriptor: PathBuf,
    status: PathBuf,
    stderr: PathBuf,
    lock: Option<PathBuf>,
    admission: Option<Admission>,
    echo: Vec<EchoServer>,
    sock_dir: tempfile::TempDir,
    detached: bool,
    pending_media: bool,
    network: Value,
    pub hang_cpu_state: String,
    diagnostics: Value,
    sidecars: Vec<Child>,
    capture_restrict_network: bool,
    checkpoint: Value,
    capture_plan: Value,
    launch_ports: crate::config::LaunchPorts,
}
impl Vm {
    pub fn from_request(root: &Path, request: &Value) -> Result<Self> {
        let value = request
            .get("config")
            .or_else(|| request.get("kwargs").and_then(|a| a.get("config")))
            .unwrap_or(request);
        let pending_media = value.get("qemu_bin").is_none() && value.get("checkpoint").is_none();
        let mut config: VmConfig = if pending_media {
            crate::config::resolve(root, value)?
        } else {
            serde_json::from_value(value.clone())?
        };
        if let Some(overrides) = value.get("overrides").and_then(Value::as_object) {
            let mut merged = serde_json::to_value(&config)?;
            for key in [
                "boot_marker",
                "boot_timeout",
                "qmp_timeout",
                "min_boot_bytes",
                "desktop_settle_seconds",
                "hang_silence_seconds",
                "required_milestones",
                "memory",
            ] {
                if let Some(value) = overrides.get(key) {
                    merged[key] = value.clone();
                }
            }
            config = serde_json::from_value(merged)?;
            if value["timeout_is_up"].as_bool() == Some(true) {
                let (boot, silence) = crate::config::timeout_policy(
                    config.boot_timeout,
                    config.hang_silence_seconds,
                    &config.env,
                )?;
                config.boot_timeout = boot;
                config.hang_silence_seconds = silence;
            }
        }
        if config.owner_pid == 0 {
            config.owner_pid = unsafe { libc::getppid() } as u32;
        }
        config.env.insert(
            "VIBEOS_QEMU_CLIENT_PID".into(),
            config.owner_pid.to_string(),
        );
        if config.backend != "native" && config.backend != "windows-host" {
            return Err("QEMU backend must be native or windows-host".into());
        }
        for n in [
            config.boot_timeout,
            config.qmp_timeout,
            config.desktop_settle_seconds,
            config.hang_silence_seconds,
            config.max_lifetime,
        ] {
            dur(n)?;
        }
        if config.work_dir.is_relative() {
            config.work_dir = root.join(&config.work_dir);
        }
        fs::create_dir_all(&config.work_dir)?;
        if config.live_log.as_os_str().is_empty() {
            config.live_log = config.work_dir.join(format!("{}_serial.log", config.arch));
        } else if config.live_log.is_relative() {
            config.live_log = root.join(&config.live_log);
        }
        let sock_dir = tempfile::Builder::new().prefix("vibeos-vm-").tempdir()?;
        let descriptor = config.work_dir.join("native-vm-owner.json");
        let status = config.work_dir.join("native-vm-status.json");
        let stderr = config
            .live_log
            .with_file_name(format!("{}_qemu.stderr.log", config.arch));
        let gdb_port = config.gdb_port;
        Ok(Self {
            config,
            serial: SerialClient::new(),
            qmp: QmpClient::new(),
            boot_output: String::new(),
            launch_timings: BTreeMap::new(),
            gdb_host: "127.0.0.1".into(),
            gdb_port,
            net_host: "127.0.0.1".into(),
            pid: None,
            uid: String::new(),
            staging: None,
            repo: root.into(),
            guardian: None,
            descriptor,
            status,
            stderr,
            lock: None,
            admission: None,
            echo: Vec::new(),
            sock_dir,
            detached: false,
            pending_media,
            network: json!({}),
            hang_cpu_state: String::new(),
            diagnostics: json!({}),
            sidecars: Vec::new(),
            checkpoint: value.get("checkpoint").cloned().unwrap_or(Value::Null),
            capture_plan: Value::Null,
            launch_ports: crate::config::LaunchPorts::default(),
            capture_restrict_network: value["capture_restrict_network"].as_bool().unwrap_or(false),
        })
    }
    fn env(&self, name: &str) -> Option<&str> {
        self.config.env.get(name).map(String::as_str)
    }
    fn serial_args(&self) -> Vec<String> {
        if self.config.serial_args.is_empty() {
            vec!["-serial".into(), "chardev:ser0".into()]
        } else {
            self.config.serial_args.clone()
        }
    }
    fn make_checkpoint_command(&mut self) -> Result<Vec<String>> {
        let checkpoint = self.checkpoint.clone();
        let bundle = PathBuf::from(
            checkpoint["bundle"]
                .as_str()
                .ok_or("checkpoint bundle missing")?,
        );
        let manifest = &checkpoint["manifest"];
        let windows = self.config.backend == "windows-host";
        let staging = if windows {
            self.staging
                .as_deref()
                .ok_or("checkpoint Windows staging missing")?
        } else {
            &self.config.work_dir
        };
        let guest = crate::checkpoint::prepare_guest(
            &self.repo,
            &bundle,
            manifest,
            Some(staging),
            windows,
        )?;
        let executable = manifest["emulator"]["path"]
            .as_str()
            .ok_or("checkpoint emulator path missing")?;
        let mut command = vec![executable.into(), "-S".into()];
        if windows {
            let host = self.config.windows["host_addr"]
                .as_str()
                .unwrap_or("127.0.0.1");
            let serial = self
                .launch_ports
                .choose(|| self.serial.listen_tcp("0.0.0.0", 0))?;
            let qmp = self
                .launch_ports
                .choose(|| self.qmp.listen_tcp("0.0.0.0", 0))?;
            self.uid = format!("vibeos-{}-{serial}", std::process::id());
            command.extend([
                "-name".into(),
                self.uid.clone(),
                "-chardev".into(),
                format!("socket,id=qmp0,host={host},port={qmp},server=off"),
                "-mon".into(),
                "chardev=qmp0,mode=control".into(),
                "-chardev".into(),
                format!("socket,id=ser0,host={host},port={serial},server=off"),
            ]);
            self.net_host = backend::gateway_ip()?;
        } else {
            let serial = self.sock_dir.path().join("serial.sock");
            self.serial.listen(&serial)?;
            command.extend([
                "-qmp".into(),
                format!(
                    "unix:{},server,nowait",
                    self.sock_dir.path().join("qmp.sock").display()
                ),
                "-chardev".into(),
                format!("socket,id=ser0,path={},server=off", serial.display()),
            ]);
        }
        command.extend([
            "-serial".into(),
            "chardev:ser0".into(),
            "-monitor".into(),
            "none".into(),
        ]);
        if self.config.gdb {
            self.gdb_port = if self.config.gdb_port != 0 {
                self.config.gdb_port
            } else if windows {
                self.launch_ports.choose(backend::windows_free_tcp_port)?
            } else {
                self.launch_ports.choose(|| {
                    Ok(std::net::TcpListener::bind(("127.0.0.1", 0))?
                        .local_addr()?
                        .port())
                })?
            };
            self.gdb_host = if windows {
                backend::gateway_ip()?
            } else {
                "127.0.0.1".into()
            };
            command.extend([
                "-gdb".into(),
                format!(
                    "tcp:{}:{}",
                    if windows { "0.0.0.0" } else { "127.0.0.1" },
                    self.gdb_port
                ),
            ]);
        }
        if manifest["debugcon"].as_bool().unwrap_or(false) {
            let log = staging.join("debugcon.bin");
            command.extend([
                "-debugcon".into(),
                format!(
                    "file:{}",
                    if windows {
                        backend::win_path(&log)?
                    } else {
                        log.to_string_lossy().into_owned()
                    }
                ),
            ]);
        }
        command.extend(guest);
        Ok(command)
    }
    pub fn wait_exit(&mut self, timeout: f64) -> Result<i32> {
        let deadline = Instant::now() + dur(timeout)?;
        loop {
            if let Some(code) = self.poll_exit()? {
                return Ok(code);
            }
            if Instant::now() >= deadline {
                return Err("QEMU exit wait timed out".into());
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    fn make_command(&mut self, accel: &str) -> Result<Vec<String>> {
        self.serial = SerialClient::new();
        self.qmp = QmpClient::new();
        self.echo.clear();
        self.gdb_port = self.config.gdb_port;
        if self.gdb_port != 0 {
            self.launch_ports.reserve(self.gdb_port);
        }
        self.serial.set_tee(
            &self.config.live_log,
            &format!("# vibeos {} native live serial\n", self.config.arch),
        )?;
        if !self.checkpoint.is_null() {
            return self.make_checkpoint_command();
        }
        let mut common = self.config.common_args.clone();
        let serial = self.serial_args();
        let mut arch = self.config.arch_args.clone();
        let (network, ports) = crate::config::network_args_with(
            &self.config.arch,
            self.config.backend == "windows-host",
            &mut self.launch_ports,
        )?;
        self.network = ports;
        arch.extend(network);
        arch.extend(self.config.network_args.clone());
        if self.capture_restrict_network {
            arch.extend(["-nic".into(), "user,restrict=on".into()]);
        }
        let is_arm = self.config.arch.starts_with("arm64");
        if self.config.backend == "native" {
            let ser = self.sock_dir.path().join("serial.sock");
            self.serial.listen(&ser)?;
            let qmp = self.sock_dir.path().join("qmp.sock");
            let mut cmd = vec![
                self.config.qemu_bin.clone(),
                "-qmp".into(),
                format!("unix:{},server,nowait", qmp.display()),
                "-chardev".into(),
                format!("socket,id=ser0,path={},server=off", ser.display()),
            ];
            cmd.extend(serial);
            cmd.extend(self.config.disk_args.clone());
            cmd.append(&mut common);
            cmd.extend(["-monitor".into(), "none".into()]);
            if self.config.gdb {
                if self.gdb_port == 0 {
                    self.gdb_port = self.launch_ports.choose(|| {
                        Ok(std::net::TcpListener::bind(("127.0.0.1", 0))?
                            .local_addr()?
                            .port())
                    })?;
                    if self.gdb_port == 0 {
                        return Err("cannot select distinct GDB port".into());
                    }
                }
                cmd.extend(["-gdb".into(), format!("tcp:127.0.0.1:{}", self.gdb_port)]);
                if self.config.gdb_wait {
                    cmd.push("-S".into());
                }
            }
            if !is_arm {
                cmd.extend([
                    "-debugcon".into(),
                    format!(
                        "file:{}",
                        self.config.work_dir.join("debugcon.bin").display()
                    ),
                ]);
            }
            cmd.extend(arch);
            return crate::config::apply_stress_knobs(&cmd, &self.config.arch, &self.config.env);
        }
        let host = self.config.windows["host_addr"]
            .as_str()
            .unwrap_or("127.0.0.1")
            .to_owned();
        let ser = self
            .launch_ports
            .choose(|| self.serial.listen_tcp("0.0.0.0", 0))?;
        let qmp = self
            .launch_ports
            .choose(|| self.qmp.listen_tcp("0.0.0.0", 0))?;
        self.uid = format!("vibeos-{}-{ser}", std::process::id());
        let dir = PathBuf::from(
            self.config.windows["qemu_dir"]
                .as_str()
                .unwrap_or("/mnt/c/qemu-11.1.0"),
        );
        let bin = backend::windows_qemu_bin(&self.config.qemu_bin, &dir, accel);
        let version = backend::windows_qemu_version(&bin);
        if let Some(bad) = backend::known_bad_windows_qemu(&version) {
            if self.env("VIBEOS_QEMU_ALLOW_KNOWN_BAD") != Some("1") {
                return Err(Error::new(format!("{bin} is a known-bad QEMU build: {bad}; select a released build via VIBEOS_QEMU_WINDOWS_DIR")));
            }
        }
        let staging = self
            .staging
            .as_ref()
            .ok_or("Windows staging is not prepared")?
            .clone();
        arch = crate::config::acpitable_relative(&backend::stage_media_paths(&arch, &staging)?);
        let disks = backend::stage_media_paths(&self.config.disk_args, &staging)?;
        if !is_arm {
            arch = backend::rewrite_accel(&arch, accel, &self.config.arch, &self.config.env);
        }
        let (arch, ids) = backend::rewrite_netdev(&arch, "gf")?;
        let mut echo = Vec::new();
        for id in ids {
            let server = EchoServer::new()?;
            echo.extend([
                "-chardev".into(),
                format!("socket,id={id},host={host},port={},server=off", server.port),
            ]);
            self.echo.push(server);
        }
        let mut cmd = vec![
            bin,
            "-name".into(),
            self.uid.clone(),
            "-chardev".into(),
            format!("socket,id=qmp0,host={host},port={qmp},server=off"),
            "-mon".into(),
            "chardev=qmp0,mode=control".into(),
            "-chardev".into(),
            format!("socket,id=ser0,host={host},port={ser},server=off"),
        ];
        cmd.extend(serial);
        cmd.extend(disks);
        cmd.extend(common);
        cmd.extend(["-monitor".into(), "none".into()]);
        if !is_arm {
            cmd.extend([
                "-debugcon".into(),
                format!("file:{}", backend::win_path(&staging.join("debugcon.bin"))?),
            ]);
        }
        if self.config.gdb {
            if self.gdb_port == 0 {
                self.gdb_port = self.launch_ports.choose(backend::windows_free_tcp_port)?;
                if self.gdb_port == 0 {
                    return Err("cannot select distinct Windows GDB port".into());
                }
            }
            self.gdb_host = backend::gateway_ip()?;
            cmd.extend(["-gdb".into(), format!("tcp:0.0.0.0:{}", self.gdb_port)]);
            if self.config.gdb_wait {
                cmd.push("-S".into());
            }
        }
        cmd.extend(echo);
        cmd.extend(arch);
        self.net_host = backend::gateway_ip()?;
        crate::config::apply_stress_knobs(&cmd, &self.config.arch, &self.config.env)
    }
    fn diagnostic_process(&self, op: &str, payload: &Value) -> Result<Child> {
        let python = self
            .config
            .env
            .get("VIBEOS_TEST_PYTHON")
            .cloned()
            .unwrap_or_else(|| {
                let venv = self.repo.join(".venv/bin/python");
                if venv.is_file() {
                    venv.to_string_lossy().into_owned()
                } else {
                    "python3".into()
                }
            });
        let out = self.config.work_dir.join(format!("{op}.json"));
        let err = self.config.work_dir.join(format!("{op}.stderr.log"));
        let mut command = Command::new(python);
        command
            .args(["-m", "tests.qemu.native_worker", op])
            .current_dir(&self.repo)
            .envs(&self.config.env)
            .stdin(Stdio::piped())
            .stdout(File::create(out)?)
            .stderr(OpenOptions::new().create(true).append(true).open(err)?);
        let owner = std::process::id();
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::getppid() != owner as i32 {
                    libc::_exit(125);
                }
                Ok(())
            });
        }
        let mut child = command.spawn()?;
        child
            .stdin
            .take()
            .ok_or("diagnostic stdin unavailable")?
            .write_all(&serde_json::to_vec(payload)?)?;
        Ok(child)
    }
    fn prepare_diagnostics(&mut self, mut argv: Vec<String>) -> Result<Vec<String>> {
        let requested = [
            "VIBEOS_QEMU_RR",
            "VIBEOS_QEMU_RR_SAMPLE",
            "VIBEOS_QEMU_PC_SAMPLE",
        ]
        .iter()
        .any(|key| self.env(key).is_some_and(|v| !v.trim().is_empty()));
        if !requested {
            return Ok(argv);
        }
        if self
            .env("VIBEOS_QEMU_RR")
            .is_some_and(|v| !v.trim().is_empty())
            && !self.config.arch.starts_with("arm64")
        {
            argv = backend::rewrite_accel(&argv, "tcg", &self.config.arch, &self.config.env);
        }
        let payload = json!({"root":self.repo,"argv":argv,"arch":self.config.arch,"backend":self.config.backend,"env":self.config.env,"sock_dir":self.sock_dir.path(),"live_log":self.config.live_log,"work_dir":self.config.work_dir,"reserved_ports":self.launch_ports.ports()});
        let mut child = self.diagnostic_process("vm_diagnostic_prepare", &payload)?;
        let deadline = Instant::now() + Duration::from_secs(1800);
        let state = loop {
            if let Some(s) = child.try_wait()? {
                break s;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err("diagnostic preparation timed out".into());
            }
            thread::sleep(Duration::from_millis(50));
        };
        let response = read_json(&self.config.work_dir.join("vm_diagnostic_prepare.json"))?;
        if !state.success() || response.get("error").is_some() {
            return Err(Error::new(format!(
                "diagnostic preparation failed: {}; see vm_diagnostic_prepare.stderr.log",
                response
            )));
        }
        self.diagnostics = response
            .get("ok")
            .cloned()
            .ok_or("diagnostic worker missing ok payload")?;
        if let Some(sidecars) = self.diagnostics["sidecars"].as_array() {
            for sidecar in sidecars {
                if let Some(port) = sidecar["port"].as_u64() {
                    if port > 65535 || !self.launch_ports.reserve(port as u16) {
                        return Err(
                            "diagnostic sidecar selected a duplicate/invalid launch port".into(),
                        );
                    }
                }
            }
        }
        if let Some(min) = self.diagnostics["boot_timeout_min"].as_f64() {
            self.config.boot_timeout = self.config.boot_timeout.max(min);
        }
        if let Some(min) = self.diagnostics["hang_silence_min"].as_f64() {
            if self.config.hang_silence_seconds > 0.0 {
                self.config.hang_silence_seconds = self.config.hang_silence_seconds.max(min);
            }
        }
        Ok(serde_json::from_value(self.diagnostics["argv"].clone())?)
    }
    fn diagnostics_ready(&mut self) -> Result<()> {
        if let Some(icount) = self.diagnostics["rr_break_icount"].as_u64() {
            self.qmp.execute("replay-break", json!({"icount":icount}))?;
        }
        let payloads = self.diagnostics["sidecars"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for payload in payloads {
            self.sidecars
                .push(self.diagnostic_process("vm_diagnostic_sidecar", &payload)?);
        }
        Ok(())
    }
    pub fn report_replay_break(&mut self) -> Result<Value> {
        let Some(icount) = self.diagnostics["rr_break_icount"].as_u64() else {
            return Ok(Value::Null);
        };
        let event = self.qmp.saw_event("STOP");
        Ok(json!({"icount":icount,"fired":event.is_some(),"event":event}))
    }
    pub fn admit(&mut self) -> Result<()> {
        if self.admission.is_some() {
            return Ok(());
        }
        let accel = if !self.checkpoint.is_null() {
            "tcg".into()
        } else if self.config.backend == "windows-host" {
            backend::resolve_accel(
                &self.config.arch,
                self.config.windows["accel"].as_str().unwrap_or("tcg"),
                None,
            )
        } else if self.config.arch.starts_with("arm64") || !Path::new("/dev/kvm").exists() {
            "tcg".into()
        } else {
            "kvm".into()
        };
        self.admission = Some(Admission::acquire(
            &self.repo,
            &self.config.arch,
            &self.config.backend,
            &accel,
            &self.config.work_dir,
            &self.config.env,
        )?);
        Ok(())
    }
    pub fn override_arch_args(&mut self, args: Vec<String>) -> Result<()> {
        if self.admission.is_none() {
            return Err("custom media preparation requires native admission first".into());
        }
        if self.guardian.is_some() {
            return Err("cannot replace running VM launch plan".into());
        }
        let (disk_args, common) =
            crate::config::devices(&self.config.arch, &self.config.work_dir, &self.config.env)?;
        self.config.arch_args = args;
        self.config.disk_args = disk_args;
        self.config.common_args.extend(common);
        self.pending_media = false;
        Ok(())
    }
    pub fn start(&mut self) -> Result<Value> {
        let result = self.start_inner();
        if result.is_err() {
            let _ = self.stop();
        }
        result
    }
    fn start_inner(&mut self) -> Result<Value> {
        if self.guardian.is_some() {
            return Err("VM is already started".into());
        }
        let started = Instant::now();
        let accel = if !self.checkpoint.is_null() {
            "tcg".into()
        } else if self.config.backend == "windows-host" {
            let configured = self.config.windows["accel"].as_str().unwrap_or("tcg");
            backend::resolve_accel(
                &self.config.arch,
                configured,
                if configured == "whpx" && !self.config.arch.starts_with("arm64") {
                    backend::whpx_available()
                } else {
                    None
                },
            )
        } else {
            if self.config.arch.starts_with("arm64") || !Path::new("/dev/kvm").exists() {
                "tcg".into()
            } else {
                "kvm".into()
            }
        };
        self.launch_timings
            .insert("resolve_seconds".into(), started.elapsed().as_secs_f64());
        if accel == "whpx" {
            self.config.boot_timeout = self.config.boot_timeout.max(900.0);
            self.config.hang_silence_seconds = self.config.hang_silence_seconds.max(420.0);
        }
        let admitted = Instant::now();
        if self.admission.is_none() {
            self.admission = Some(Admission::acquire(
                &self.repo,
                &self.config.arch,
                &self.config.backend,
                &accel,
                &self.config.work_dir,
                &self.config.env,
            )?);
        }
        self.launch_timings
            .insert("admission_seconds".into(), admitted.elapsed().as_secs_f64());
        let preparation = Instant::now();
        if self.pending_media {
            let prepared = crate::config::prepare(&self.repo, &self.config)?;
            self.config.arch_args = prepared.arch_args;
            self.config.common_args = prepared.common_args;
            self.config.disk_args = prepared.disk_args;
            self.config.serial_args = prepared.serial_args;
            self.pending_media = false;
        }
        if self.config.backend == "windows-host" {
            let root = PathBuf::from(
                self.config.windows["staging_dir"]
                    .as_str()
                    .unwrap_or("/mnt/c/temp/vibeos-qemu"),
            );
            fs::create_dir_all(&root)?;
            let temp = tempfile::Builder::new()
                .prefix(&format!("{}-{}-", std::process::id(), self.config.arch))
                .tempdir_in(&root)?;
            self.staging = Some(temp.keep());
        }
        self.launch_timings.insert(
            "media_prepare_seconds".into(),
            preparation.elapsed().as_secs_f64(),
        );
        let waiting = Instant::now();
        let _handshake = port_handshake(&self.config.env, &self.repo, Duration::from_secs(120))?;
        self.launch_timings.insert(
            "launch_lock_seconds".into(),
            waiting.elapsed().as_secs_f64(),
        );
        if let Some(staging) = self.staging.as_ref() {
            let root = staging.parent().ok_or("staging lacks root")?;
            let controls = PathBuf::from(
                self.env("VIBEOS_HOSTLOCK_DIR")
                    .unwrap_or("/tmp/vibeos-qemu-locks/host"),
            );
            fs::create_dir_all(&controls)?;
            use sha2::{Digest, Sha256};
            let hash = format!(
                "{:x}",
                Sha256::digest(root.canonicalize()?.to_string_lossy().as_bytes())
            );
            let stamp = controls.join(format!("windows-cleanup-{}.stamp", &hash[..16]));
            let recent = fs::metadata(&stamp)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| SystemTime::now().duration_since(t).ok())
                .is_some_and(|age| age < Duration::from_secs(300));
            if !recent {
                backend::cleanup_staging(root, false, 3600.0)?;
                fs::write(stamp, b"")?;
            }
        }
        let attempts = crate::launch_policy::attempts(&self.config.backend, &self.config.env)?;
        let launch = Instant::now();
        for attempt in 0..attempts {
            let _ = fs::remove_file(self.sock_dir.path().join("qmp.sock"));
            let raw_command = self.make_command(&accel)?;
            if self.capture_restrict_network {
                let (template, media) = crate::checkpoint::validate_template(&raw_command)?;
                self.capture_plan = json!({"argv":template,"media":media});
            }
            let command = if self.checkpoint.is_null() {
                self.prepare_diagnostics(raw_command)?
            } else {
                raw_command
            };
            let owner = self.config.owner_pid;
            let server = std::process::id();
            let descriptor = json!({"argv":command,"cwd":self.staging.as_ref().map(|p|p.to_string_lossy().into_owned()).unwrap_or_default(),"env":self.config.env,"owner_pid":owner,"owner_tick":tick(owner).ok_or("VM client owner is not alive")?,"server_pid":server,"server_tick":tick(server).ok_or("VM API owner is not alive")?,"deadline":now()+self.config.max_lifetime,"status":self.status,"stderr":self.stderr,"uid":self.uid,"stop":false,"detached":false,"retain_staging_until_release":self.capture_restrict_network || !self.checkpoint.is_null(),"staging":self.staging,"lease_fds":self.admission.as_ref().map(Admission::inherited_fds).unwrap_or_default()});
            atomic_json(&self.descriptor, &descriptor)?;
            let mut guard = Command::new(std::env::current_exe()?);
            guard
                .arg("vm-guardian")
                .arg(&self.descriptor)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(self.config.work_dir.join("guardian.stderr.log"))?,
                );
            let lease_fds = self
                .admission
                .as_ref()
                .map(Admission::inherited_fds)
                .unwrap_or_default();
            unsafe {
                guard.pre_exec(move || {
                    for fd in &lease_fds {
                        let flags = libc::fcntl(*fd, libc::F_GETFD);
                        if flags < 0
                            || libc::fcntl(*fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                        {
                            return Err(std::io::Error::last_os_error());
                        }
                    }
                    if libc::setsid() < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            let _ = fs::remove_file(&self.status);
            self.guardian = Some(guard.spawn()?);
            let deadline = Instant::now() + Duration::from_secs(10);
            let receipt = loop {
                if let Ok(v) = read_json(&self.status) {
                    if v["state"] == "running" || v["state"] == "failed" || v["state"] == "stopped"
                    {
                        break v;
                    }
                }
                if Instant::now() >= deadline {
                    return Err("QEMU guardian did not publish launch receipt".into());
                }
                thread::sleep(Duration::from_millis(10));
            };
            if receipt["state"] != "running" {
                let error = receipt["error"]
                    .as_str()
                    .unwrap_or("QEMU exited during launch")
                    .to_owned();
                if self.retry_launch(attempt, attempts) {
                    self.stop_attempt()?;
                    continue;
                }
                return Err(Error::new(format!(
                    "{error}\nQEMU stderr:\n{}",
                    self.stderr_tail()
                )));
            }
            self.pid = receipt["pid"].as_u64().map(|x| x as u32);
            self.record_lock(&command)?;
            let early = Instant::now()
                + Duration::from_secs(if self.config.backend == "windows-host" {
                    2
                } else {
                    0
                });
            while Instant::now() < early {
                if self.poll_exit()?.is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            if self.poll_exit()?.is_some() {
                if self.retry_launch(attempt, attempts) {
                    self.stop_attempt()?;
                    continue;
                }
                return Err(Error::new(format!(
                    "QEMU exited during launch\n{}",
                    self.stderr_tail()
                )));
            }
            let serial_deadline = Instant::now() + Duration::from_secs(15);
            loop {
                match self.serial.accept(Duration::from_millis(100)) {
                    Ok(()) => break,
                    Err(e) => {
                        if let Some(code) = self.poll_exit()? {
                            return Err(Error::new(format!(
                                "QEMU exited with code {code} before serial connection\n{}",
                                self.stderr_tail()
                            )));
                        }
                        if !e.message.contains("timed out") || Instant::now() >= serial_deadline {
                            return Err(e);
                        }
                    }
                }
            }

            if self.config.backend == "windows-host" {
                self.qmp.accept(dur(self.config.qmp_timeout)?)?;
            } else {
                let qmp = self.sock_dir.path().join("qmp.sock");
                let end = Instant::now() + Duration::from_secs(10);
                while !qmp.exists() {
                    if self.poll_exit()?.is_some() {
                        return Err(Error::new(format!(
                            "QEMU exited before QMP connection\n{}",
                            self.stderr_tail()
                        )));
                    }
                    if Instant::now() >= end {
                        return Err("QMP socket did not appear".into());
                    }
                    thread::sleep(Duration::from_millis(20));
                }
                self.qmp.connect(&qmp, dur(self.config.qmp_timeout)?)?;
            }
            self.diagnostics_ready()?;
            self.launch_timings
                .insert("launch_seconds".into(), launch.elapsed().as_secs_f64());
            self.launch_timings.insert(
                "prepare_seconds".into(),
                preparation.elapsed().as_secs_f64(),
            );
            atomic_json(
                &self
                    .config
                    .live_log
                    .with_file_name(format!("{}_launch.json", self.config.arch)),
                &json!({"arch":self.config.arch,"backend":self.config.backend,"accel":accel,"argv":command,"timings":self.launch_timings}),
            )?;
            return self.handles();
        }
        Err("QEMU exhausted launch retries".into())
    }
    fn retry_launch(&self, attempt: usize, attempts: usize) -> bool {
        crate::launch_policy::should_retry(
            &self.config.backend,
            attempt,
            attempts,
            &self.stderr_tail(),
        )
    }
    fn record_lock(&mut self, command: &[String]) -> Result<()> {
        let owner = self.config.owner_pid;
        let pid = self.pid.ok_or("QEMU PID missing")?;
        let root = PathBuf::from(
            self.env("VIBEOS_QEMU_LOCK_DIR")
                .unwrap_or("/tmp/vibeos-qemu-locks"),
        );
        fs::create_dir_all(&root)?;
        let path = root.join(format!("{owner}-{pid}.json"));
        atomic_json(
            &path,
            &json!({"launcher_pid":owner,"launcher_start_tick":tick(owner),"qemu_pid":pid,"qemu_start_tick":tick(pid),"arch":self.config.arch,"work_dir":self.config.work_dir,"started":now(),"deadline":(now()+self.config.max_lifetime) as u64,"slot":self.admission.as_ref().map(|a|a.global_slot),"live_log":self.config.live_log,"cmd":command,"win_uid":self.uid,"guardian_pid":self.guardian.as_ref().map(Child::id)}),
        )?;
        let mut descriptor = read_json(&self.descriptor)?;
        descriptor["lock_path"] = json!(path);
        atomic_json(&self.descriptor, &descriptor)?;
        self.lock = Some(path);
        Ok(())
    }
    pub fn handles(&self) -> Result<Value> {
        Ok(
            json!({"argv":read_json(&self.descriptor).ok().and_then(|v|v.get("argv").cloned()).unwrap_or_else(||json!([])),"qemu_pid":self.pid,"gdb_host":self.gdb_host,"gdb_port":self.gdb_port,"net_host":self.net_host,"win_uid":self.uid,"staging":self.staging,"live_log":self.config.live_log,"launch_timings":self.launch_timings,"boot_output":self.boot_output,"config":self.config,"network":self.network,"capture_plan":self.capture_plan,"admission":self.admission.as_ref().map(Admission::receipt)}),
        )
    }
    pub fn poll_exit(&mut self) -> Result<Option<i32>> {
        for child in &mut self.sidecars {
            if let Some(state) = child.try_wait()? {
                if !state.success() {
                    return Err(Error::new(format!("requested VM diagnostic failed ({state}); see vm_diagnostic_sidecar.stderr.log")));
                }
            }
        }
        match read_json(&self.status) {
            Ok(v) if v["state"] == "stopped" => Ok(v["exit_code"].as_i64().map(|n| n as i32)),
            Ok(v) if v["state"] == "failed" => {
                Ok(Some(v["exit_code"].as_i64().unwrap_or(127) as i32))
            }
            Ok(_) => Ok(None),
            Err(e) if self.guardian.is_none() => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn poll_serial(&mut self, timeout: f64) -> Result<String> {
        let out = self.serial.read_available(dur(timeout)?)?;
        if out.is_empty() {
            if let Some(code) = self.poll_exit()? {
                return Err(Error::new(format!("QEMU exited with code {code}")));
            }
        }
        Ok(out)
    }
    pub fn wait_for_boot(&mut self) -> Result<String> {
        let result = if self.config.boot_marker.is_empty() {
            self.serial.read_until_idle(
                Duration::from_secs(5),
                dur(self.config.boot_timeout)?,
                self.config.min_boot_bytes,
            )
        } else {
            self.serial
                .read_until(&self.config.boot_marker, dur(self.config.boot_timeout)?)
        };
        match result {
            Ok(s) => self.boot_output = s,
            Err(e) => {
                self.boot_output = self.serial.partial_output.clone();
                let _ = self.save_hang_state("boot-wait timeout");
                return Err(e);
            }
        }
        if self.config.desktop_settle_seconds > 0.0 {
            match self.serial.read_until_idle(
                dur(self.config.desktop_settle_seconds)?,
                dur(self.config.desktop_settle_seconds + 10.0)?,
                0,
            ) {
                Ok(extra) => self.boot_output.push_str(&extra),
                Err(_) => self
                    .boot_output
                    .push_str(&self.serial.read_available(Duration::ZERO)?),
            }
        }
        Ok(self.boot_output.clone())
    }
    fn drain_paused(&mut self) -> Result<String> {
        let mut out = String::new();
        let end = Instant::now() + Duration::from_secs(2);
        while Instant::now() < end {
            let s = self.serial.read_available(Duration::from_millis(50))?;
            if s.is_empty() {
                break;
            }
            out.push_str(&s);
        }
        Ok(out)
    }
    pub fn save_snapshot(&mut self, name: &str) -> Result<()> {
        validate_snapshot(name)?;
        self.qmp.hmp("stop")?;
        let result = (|| {
            let extra = self.drain_paused()?;
            self.boot_output.push_str(&extra);
            self.qmp.hmp(&format!("savevm {name}"))
        })();
        let resumed = self.qmp.hmp("cont");
        let result = result?;
        resumed?;
        if result.contains("Error") {
            return Err(Error::new(format!("savevm failed: {result}")));
        }
        Ok(())
    }
    pub fn restore_snapshot(&mut self, name: &str) -> Result<()> {
        validate_snapshot(name)?;
        self.qmp.hmp("stop")?;
        let result = (|| {
            self.drain_paused()?;
            self.qmp.hmp(&format!("loadvm {name}"))
        })();
        let resumed = self.qmp.hmp("cont");
        let result = result?;
        resumed?;
        if result.contains("Error") {
            return Err(Error::new(format!("loadvm failed: {result}")));
        }
        Ok(())
    }
    pub fn screendump(&mut self, path: &Path) -> Result<PathBuf> {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
        let actual = if self.config.backend == "windows-host" {
            self.staging
                .as_ref()
                .ok_or("Windows staging missing")?
                .join(format!(
                    "screen-{}",
                    path.file_name()
                        .ok_or("screenshot has no basename")?
                        .to_string_lossy()
                ))
        } else {
            path.into()
        };
        let filename = if self.config.backend == "windows-host" {
            backend::win_path(&actual)?
        } else {
            actual.to_string_lossy().into_owned()
        };
        let result = self
            .qmp
            .execute("screendump", json!({"filename":filename}))?;
        thread::sleep(Duration::from_millis(500));
        if !actual.is_file() {
            return Err(Error::new(format!("screendump failed: {result}")));
        }
        if actual != path {
            fs::copy(&actual, path)?;
        }
        Ok(path.into())
    }
    fn diagnostic_hmp(&mut self, command: &str) -> String {
        self.qmp
            .hmp(command)
            .unwrap_or_else(|e| format!("<unavailable: {}>", e.message))
            .trim()
            .into()
    }
    fn guest_clock(&mut self) -> String {
        let base: u64 = if self.config.arch.starts_with("i386") {
            0xffdf0000
        } else {
            0xfffff78000000000
        };
        let body = self.diagnostic_hmp(&format!("x /3wx 0x{:x}", base + 8));
        let words = monitor_words(&body);
        if words.len() < 3 {
            return format!("=== guest clock ===\n<unreadable: {body}>");
        }
        if words[1] != words[2] {
            return format!("=== guest clock ===\n<torn read {body}; the clock is mid-update>");
        }
        let value = (words[1] << 32) | words[0];
        let ticks = self.diagnostic_hmp(&format!("x /1wx 0x{:x}", base + 0x320));
        format!(
            "=== guest clock ===\nInterruptTime=0x{value:x} ({:.3}s since boot)\nTickCount={}",
            value as f64 / 1e7,
            ticks
                .split_once(':')
                .map(|(_, s)| s)
                .unwrap_or(&ticks)
                .trim()
        )
    }
    fn walk_frames(&mut self, cpu: u64, hex: &str) -> String {
        let _ = self.qmp.hmp(&format!("cpu {cpu}"));
        let unit = if hex.len() > 8 { "g" } else { "w" };
        let mut fp = u64::from_str_radix(hex, 16).unwrap_or(0);
        let mut lines = Vec::new();
        for depth in 0..32 {
            let body = self.diagnostic_hmp(&format!("x /2{unit}x 0x{fp:x}"));
            let words = monitor_words(&body);
            if words.len() < 2 {
                lines.push(format!("  (end: unreadable at 0x{fp:x}: {body})"));
                break;
            }
            let next = words[0];
            let ret = words[1];
            lines.push(format!("  #{depth} fp={fp:x} ret={ret:x}"));
            if next == 0 {
                lines.push("  (end: null frame pointer)".into());
                break;
            }
            if next <= fp || next - fp > 0x100000 {
                lines.push(format!("  (end: next fp {next:x} does not ascend)"));
                break;
            }
            fp = next;
            if depth == 31 {
                lines.push("  (stopped at 32 frames)".into());
            }
        }
        let _ = self.qmp.hmp("cpu 0");
        lines.join("\n")
    }
    pub fn capture_cpu_state(&mut self) -> Result<String> {
        let mut chunks = Vec::new();
        for command in ["info status", "info cpus", "info registers -a"] {
            chunks.push(format!(
                "=== {command} ===\n{}",
                self.diagnostic_hmp(command)
            ));
        }
        chunks.push(self.guest_clock());
        let registers = chunks.join("\n");
        for (cpu, pc) in crate::verdict::cpu_pcs(&registers) {
            let command = format!("x /6i 0x{pc}");
            chunks.push(format!(
                "=== CPU#{cpu} {command} ===\n{}",
                self.diagnostic_hmp(&command)
            ));
        }
        for (cpu, sp) in crate::verdict::cpu_sps(&registers) {
            let command = format!("x /23gx 0x{sp}");
            chunks.push(format!(
                "=== CPU#{cpu} stack {command} ===\n{}",
                self.diagnostic_hmp(&command)
            ));
        }
        for (cpu, fp) in crate::verdict::cpu_fps(&registers) {
            chunks.push(format!(
                "=== CPU#{cpu} frames (frame-pointer chain) ===\n{}",
                self.walk_frames(cpu, &fp)
            ));
        }
        Ok(chunks.join("\n\n") + "\n")
    }
    pub fn save_hang_state(&mut self, phase: &str) -> Result<String> {
        let first = self
            .capture_cpu_state()
            .unwrap_or_else(|e| format!("<capture failed: {}>\n", e.message));
        self.hang_cpu_state = first.clone();
        thread::sleep(Duration::from_secs(5));
        let second = self
            .capture_cpu_state()
            .unwrap_or_else(|e| format!("<resample failed: {}>\n", e.message));
        let pcs1 = crate::verdict::cpu_pcs(&first);
        let pcs2 = crate::verdict::cpu_pcs(&second);
        let clock = |s: &str| -> Option<u64> {
            let start = s.find("InterruptTime=0x")? + 16;
            let token = s[start..].split_whitespace().next()?;
            u64::from_str_radix(token, 16).ok()
        };
        let verdict = if pcs1.is_empty() || pcs2.is_empty() {
            "UNKNOWN — no monitor PC samples to compare".into()
        } else if pcs1 != pcs2 {
            "EXECUTING — vCPU PCs moved between samples".into()
        } else {
            match (clock(&first), clock(&second)) {
                (Some(a), Some(b)) if b > a => format!(
                    "IDLE-BLOCKED — PCs identical, clock advanced {:.0}ms",
                    (b - a) as f64 / 1e4
                ),
                (Some(a), Some(b)) if a == b => {
                    "FROZEN — PCs identical and guest clock did not advance".into()
                }
                _ => "FROZEN? — PCs identical, guest clock unreadable".into(),
            }
        };
        let faults = crate::verdict::cpu_kernel_at_user_pc(&second);
        let verdict = if faults.is_empty() {
            verdict
        } else {
            format!("FAULT-LOOP — kernel execution at user PC {faults:?}; {verdict}")
        };
        let text=format!("phase: {phase}\narch: {}\nsmp: {}\nboot_timeout: {}\n{verdict}\n\n=== first sample ===\n{first}\n=== resample (5s later) ===\n{second}",self.config.arch,self.env("VIBEOS_QEMU_SMP").unwrap_or("1"),self.config.boot_timeout);
        let mut dirs = vec![self.config.work_dir.clone()];
        if let Some(dir) = self.env("VIBEOS_ARTIFACT_DIR") {
            dirs.push(PathBuf::from(dir));
        }
        for dir in dirs {
            if let Err(e) = fs::create_dir_all(&dir)
                .and_then(|_| fs::write(dir.join("hang-cpustate.txt"), &text))
            {
                eprintln!("cannot persist hang CPU state: {e}");
            }
        }
        Ok(first)
    }
    pub fn read_debugcon(&self) -> Result<Vec<u8>> {
        let path = if self.config.backend == "windows-host" {
            self.staging
                .as_ref()
                .unwrap_or(&self.config.work_dir)
                .join("debugcon.bin")
        } else {
            self.config.work_dir.join("debugcon.bin")
        };
        match fs::read(path) {
            Ok(v) => Ok(v),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn stderr_tail(&self) -> String {
        let Ok(mut f) = File::open(&self.stderr) else {
            return String::new();
        };
        use std::io::{Seek, SeekFrom};
        let size = f.metadata().map(|m| m.len()).unwrap_or(0);
        let _ = f.seek(SeekFrom::Start(size.saturating_sub(65536)));
        let mut b = Vec::new();
        let _ = f.read_to_end(&mut b);
        String::from_utf8_lossy(&b).into_owned()
    }
    fn stop_attempt(&mut self) -> Result<()> {
        for mut child in self.sidecars.drain(..) {
            if child.try_wait()?.is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
        self.serial.close();
        self.qmp.close();
        self.echo.clear();
        if self.guardian.is_some() {
            let mut d = read_json(&self.descriptor)?;
            d["stop"] = json!(true);
            atomic_json(&self.descriptor, &d)?;
            let end = Instant::now() + Duration::from_secs(20);
            while Instant::now() < end {
                if self.guardian.as_mut().unwrap().try_wait()?.is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            if self.guardian.as_mut().unwrap().try_wait()?.is_none() {
                return Err("QEMU guardian did not confirm stop".into());
            }
            let status = read_json(&self.status)?;
            if status["confirmed"] != true {
                return Err(Error::new(format!(
                    "QEMU Windows stop unconfirmed: {}",
                    status["windows_left"]
                )));
            }
            self.guardian = None;
            self.pid = None;
            if let Some(p) = self.lock.take() {
                fs::remove_file(p).or_else(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(e)
                    }
                })?;
            }
            let _ = fs::remove_file(&self.status);
        }
        Ok(())
    }
    pub fn stop(&mut self) -> Result<()> {
        if self.detached {
            return Ok(());
        }
        let result = self.stop_attempt();
        if result.is_ok() {
            if let Some(p) = self.staging.take() {
                for _ in 0..5 {
                    match fs::remove_dir_all(&p) {
                        Ok(()) => break,
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                        Err(_) => thread::sleep(Duration::from_millis(300)),
                    }
                }
            }
        }
        self.admission = None;
        result
    }
    pub fn detach_for_debug(&mut self, owner_pid: u32, run_id: &str) -> Result<Value> {
        if self.poll_exit()?.is_some() || self.pid.is_none() {
            return Err("cannot keep a VM whose QEMU has already exited".into());
        }
        let receipt = read_json(&self.status)?;
        let qemu_pid = self.pid.ok_or("QEMU has already exited")?;
        let qemu_tick = receipt["start_tick"]
            .as_u64()
            .ok_or("QEMU identity receipt missing")?;
        if !alive(qemu_pid, qemu_tick) {
            return Err("cannot keep a VM whose QEMU has already exited".into());
        }
        let start = tick(owner_pid).ok_or("debug owner is not alive")?;
        if !alive(owner_pid, start) {
            return Err("debug owner is not alive".into());
        }
        let mut d = read_json(&self.descriptor)?;
        d["owner_pid"] = json!(owner_pid);
        d["owner_tick"] = json!(start);
        d["detached"] = json!(true);
        d["deadline"] = json!(now() + self.config.max_lifetime);
        atomic_json(&self.descriptor, &d)?;
        if let Some(old) = self.lock.take() {
            let mut info = read_json(&old)?;
            info["launcher_pid"] = json!(owner_pid);
            info["launcher_start_tick"] = json!(start);
            info["deadline"] = json!((now() + self.config.max_lifetime) as u64);
            info["run_id"] = json!(run_id);
            info["source"] = json!("smoke --attach-on-fail");
            let new = old
                .parent()
                .unwrap()
                .join(format!("{owner_pid}-{}.json", self.pid.unwrap()));
            atomic_json(&new, &info)?;
            if old != new {
                fs::remove_file(old)?;
            }
            d["lock_path"] = json!(new);
            atomic_json(&self.descriptor, &d)?;
            self.lock = Some(new);
        }
        let handles = self.handles()?;
        self.serial.close();
        self.qmp.close();
        self.echo.clear();
        self.admission = None;
        self.detached = true;
        self.guardian = None;
        Ok(handles)
    }
}
impl Drop for Vm {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
fn validate_snapshot(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(
            "snapshot name must contain only ASCII letters, digits, '-', '_' or '.'".into(),
        );
    }
    Ok(())
}

#[derive(Default)]
pub struct VmManager {
    port_leases: crate::portlease::PortLeases,
    sessions: HashMap<u64, Vm>,
    next: u64,
    serials: HashMap<u64, SerialClient>,
    qmps: HashMap<u64, QmpClient>,
    media_locks: HashMap<u64, backend::MediaLock>,
}
impl VmManager {
    pub fn new() -> Self {
        Self::default()
    }
    fn protocol_handle(&mut self, op: &str, a: &Value) -> Result<Value> {
        if op == "serial_create" || op == "qmp_create" {
            self.next += 1;
            let id = self.next;
            if op == "serial_create" {
                self.serials.insert(id, SerialClient::new());
            } else {
                self.qmps.insert(id, QmpClient::new());
            }
            return Ok(json!({"id":id}));
        }
        let id = a["id"].as_u64().ok_or("missing protocol client id")?;
        let text = |name: &str| -> Result<&str> {
            a[name]
                .as_str()
                .ok_or_else(|| Error::new(format!("missing {name}")))
        };
        let timeout = dur(a["timeout"].as_f64().unwrap_or(10.0))?;
        if op.starts_with("serial_") {
            if op == "serial_close" {
                if let Some(mut s) = self.serials.remove(&id) {
                    s.close();
                }
                return Ok(Value::Null);
            }
            let s = self.serials.get_mut(&id).ok_or("unknown serial client")?;
            if let Some(alive) = a["alive"].as_bool() {
                s.set_external_alive(alive);
            }
            return Ok(match op {
                "serial_listen" => {
                    s.listen(Path::new(text("path")?))?;
                    Value::Null
                }
                "serial_listen_tcp" => json!(s.listen_tcp(
                    a["host"].as_str().unwrap_or("0.0.0.0"),
                    a["port"].as_u64().unwrap_or(0) as u16
                )?),
                "serial_accept" => {
                    s.accept(timeout)?;
                    Value::Null
                }
                "serial_tee" => {
                    s.set_tee(Path::new(text("path")?), a["header"].as_str().unwrap_or(""))?;
                    Value::Null
                }
                "serial_follow_file" => {
                    let pid = a["owner_pid"].as_u64().unwrap_or(std::process::id() as u64) as u32;
                    let start = tick(pid);
                    s.follow_file(
                        Path::new(text("path")?),
                        Arc::new(move || start.map(|t| alive(pid, t)).unwrap_or(false)),
                    );
                    Value::Null
                }
                "serial_read_until" => json!(s.read_until(text("marker")?, timeout)?),
                "serial_read_until_idle" => json!(s.read_until_idle(
                    dur(a["idle_seconds"].as_f64().unwrap_or(5.0))?,
                    timeout,
                    a["min_bytes"].as_u64().unwrap_or(1024) as usize
                )?),
                "serial_read_available" => json!(s.read_available(timeout)?),
                "serial_send" => {
                    let bytes: Vec<u8> = serde_json::from_value(a["data"].clone())?;
                    s.send(&bytes)?;
                    Value::Null
                }
                "serial_set_position" => {
                    s.set_consumed_position(
                        a["position"].as_u64().ok_or("missing position")? as usize
                    )?;
                    Value::Null
                }
                "serial_snapshot" => {
                    let (text, age, dead) = s.snapshot();
                    json!({"text":text,"last_data_age":age.as_secs_f64(),"dead":dead,"position":s.consumed_position(),"partial_output":s.partial_output})
                }
                _ => return Err(Error::new(format!("unknown serial operation {op}"))),
            });
        }
        if op == "qmp_close" {
            if let Some(mut q) = self.qmps.remove(&id) {
                q.close();
            }
            return Ok(Value::Null);
        }
        let q = self.qmps.get_mut(&id).ok_or("unknown QMP client")?;
        Ok(match op {
            "qmp_connect_tcp" => q.connect_tcp(
                a["host"].as_str().ok_or("missing host")?,
                a["port"].as_u64().ok_or("missing port")? as u16,
                timeout,
            )?,
            "qmp_connect" => q.connect(Path::new(text("path")?), timeout)?,
            "qmp_listen_tcp" => json!(q.listen_tcp(
                a["host"].as_str().unwrap_or("0.0.0.0"),
                a["port"].as_u64().unwrap_or(0) as u16
            )?),
            "qmp_accept" => q.accept(timeout)?,
            "qmp_execute" => q.execute(
                text("command")?,
                a.get("arguments").cloned().unwrap_or_else(|| json!({})),
            )?,
            "qmp_hmp" => json!(q.hmp(text("command")?)?),
            "qmp_events" => q.events_snapshot(),
            "qmp_saw_event" => json!(q.saw_event(text("name")?)),
            "qmp_wait_event" => json!(q.wait_event(text("name")?, timeout)?),
            _ => return Err(Error::new(format!("unknown QMP operation {op}"))),
        })
    }

    pub fn handle(&mut self, root: &Path, request: &Value) -> Result<Value> {
        if request["op"]
            .as_str()
            .is_some_and(|op| op.starts_with("portlease_"))
        {
            return self.port_leases.handle(root, request);
        }
        let a = request.get("kwargs").unwrap_or(request);
        let op = request["op"].as_str().unwrap_or("");
        if op.starts_with("serial_") || op.starts_with("qmp_") {
            return self.protocol_handle(op, a);
        }
        if matches!(
            op,
            "vm_config"
                | "vm_arch_config"
                | "vm_prepare_config"
                | "vm_env_options"
                | "vm_smp_count"
                | "vm_smp_boot_timeout"
                | "vm_unique_ports"
                | "vm_smp_selected_iso"
                | "vm_stortest_selected_iso"
                | "vm_testkernel_boot_img"
                | "vm_make_temp_disk"
                | "vm_sparse_image"
                | "vm_temp_disk_args"
                | "vm_make_usb_stick_image"
                | "vm_tripwire_args"
                | "vm_hostile_hazards"
                | "vm_xhci_interrupts"
                | "vm_nvme_bridge"
                | "vm_boot_media_device_args"
                | "vm_apply_stress_knobs"
                | "vm_console_uart_args"
                | "vm_acpitable_relative"
                | "vm_make_spcr"
                | "vm_machine_args"
        ) {
            return crate::config::dispatch(root, request);
        }
        if op == "vm_media_cache_lock" {
            self.next += 1;
            let id = self.next;
            let root = a["root"].as_str().map(Path::new);
            let guard =
                backend::MediaLock::acquire(a["name"].as_str().unwrap_or("cache.lock"), root)?;
            self.media_locks.insert(id, guard);
            return Ok(json!({"id":id}));
        }
        if op == "vm_media_cache_unlock" {
            let id = a["id"].as_u64().ok_or("missing media lock id")?;
            self.media_locks.remove(&id);
            return Ok(Value::Null);
        }
        if op == "vm_create" {
            let vm = Vm::from_request(root, a)?;
            self.next += 1;
            let id = self.next;
            let result = vm.handles()?;
            self.sessions.insert(id, vm);
            return Ok(json!({"id":id,"config":result["config"],"handles":result}));
        }
        let id = a["id"].as_u64().ok_or("missing VM session id")?;
        if op == "vm_close" {
            if let Some(mut vm) = self.sessions.remove(&id) {
                vm.stop()?;
            }
            return Ok(Value::Null);
        }
        let vm = self.sessions.get_mut(&id).ok_or("unknown VM session")?;
        let text = |name: &str| -> Result<&str> {
            a[name]
                .as_str()
                .ok_or_else(|| Error::new(format!("missing {name}")))
        };
        let timeout = || a["timeout"].as_f64().unwrap_or(0.1);
        Ok(match op {
            "vm_admit" => {
                vm.admit()?;
                Value::Null
            }
            "vm_override_arch_args" => {
                vm.override_arch_args(serde_json::from_value(a["args"].clone())?)?;
                Value::Null
            }
            "vm_start" => vm.start()?,
            "vm_handles" => vm.handles()?,
            "vm_wait_for_boot" => json!(vm.wait_for_boot()?),
            "vm_serial_output" | "vm_poll_serial" | "vm_serial_read_available" => {
                json!(vm.poll_serial(timeout())?)
            }
            "vm_serial_read_until" => json!(vm
                .serial
                .read_until(text("marker")?, dur(a["timeout"].as_f64().unwrap_or(60.0))?)?),
            "vm_serial_read_until_idle" => json!(vm.serial.read_until_idle(
                dur(a["idle_seconds"].as_f64().unwrap_or(5.0))?,
                dur(a["timeout"].as_f64().unwrap_or(120.0))?,
                a["min_bytes"].as_u64().unwrap_or(1024) as usize
            )?),
            "vm_serial_send" => {
                let bytes: Vec<u8> = serde_json::from_value(a["data"].clone())?;
                vm.serial.send(&bytes)?;
                Value::Null
            }
            "vm_serial_snapshot" => {
                let (s, age, dead) = vm.serial.snapshot();
                json!({"text":s,"last_data_age":age.as_secs_f64(),"dead":dead,"position":vm.serial.consumed_position(),"partial_output":vm.serial.partial_output})
            }
            "vm_serial_set_position" => {
                vm.serial.set_consumed_position(
                    a["position"].as_u64().ok_or("missing serial position")? as usize,
                )?;
                Value::Null
            }
            "vm_qmp_execute" => vm.qmp.execute(
                text("command")?,
                a.get("arguments").cloned().unwrap_or_else(|| json!({})),
            )?,
            "vm_qmp_hmp" => json!(vm.qmp.hmp(text("command")?)?),
            "vm_qmp_events" => vm.qmp.events_snapshot(),
            "vm_qmp_saw_event" => json!(vm.qmp.saw_event(text("name")?)),
            "vm_qmp_wait_event" => json!(vm.qmp.wait_event(text("name")?, dur(timeout())?)?),
            "vm_save_snapshot" => {
                vm.save_snapshot(a["name"].as_str().unwrap_or("ready"))?;
                Value::Null
            }
            "vm_restore_snapshot" => {
                vm.restore_snapshot(a["name"].as_str().unwrap_or("ready"))?;
                Value::Null
            }
            "vm_screendump" => json!(vm.screendump(Path::new(text("path")?))?),
            "vm_report_replay_break" => vm.report_replay_break()?,
            "vm_nvme_enabled" => json!(read_json(&vm.descriptor)?
                .get("argv")
                .and_then(Value::as_array)
                .is_some_and(|args| args
                    .iter()
                    .any(|a| a.as_str().is_some_and(|s| s.starts_with("nvme,"))))),
            "vm_get_hang_cpu_state" => json!(vm.hang_cpu_state),
            "vm_capture_cpu_state" => json!(vm.capture_cpu_state()?),
            "vm_save_hang_state" | "vm_hang_cpu_state" => {
                json!(vm.save_hang_state(a["phase"].as_str().unwrap_or("boot wait"))?)
            }
            "vm_read_debugcon" => json!(vm.read_debugcon()?),
            "vm_wait_exit" => json!(vm.wait_exit(a["timeout"].as_f64().unwrap_or(20.0))?),
            op if op.starts_with("vm_checkpoint_") => crate::checkpoint::dispatch_vm(vm, op, a)?,
            "vm_poll_exit" => json!(vm.poll_exit()?),
            "vm_stderr_tail" | "vm_stderr" => json!(vm.stderr_tail()),
            "vm_stop" => {
                vm.stop()?;
                Value::Null
            }
            "vm_detach" => vm.detach_for_debug(
                a["owner_pid"].as_u64().ok_or("missing debug owner")? as u32,
                text("run_id")?,
            )?,
            _ => return Err(Error::new(format!("unknown VM operation {op}"))),
        })
    }
    pub fn shutdown_all(&mut self) -> Result<()> {
        self.port_leases.close_all();
        let mut first = None;
        for (_, mut vm) in self.sessions.drain() {
            if let Err(e) = vm.stop() {
                if first.is_none() {
                    first = Some(e);
                }
            }
        }
        match first {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

fn monitor_words(body: &str) -> Vec<u64> {
    let body = body.split_once(':').map(|(_, s)| s).unwrap_or(body);
    regex::Regex::new(r"0x([0-9a-fA-F]+)")
        .unwrap()
        .captures_iter(body)
        .filter_map(|c| u64::from_str_radix(&c[1], 16).ok())
        .collect()
}
