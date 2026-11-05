//! Native Pico UART capture. The wire protocol is the SDK's raw TCP protocol.
use chrono::Local;
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn token() -> io::Result<Vec<u8>> {
    let path = env::var_os("PICO_DEVICE_TOKEN_FILE")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                .map(|p| PathBuf::from(p).join(".config/picousb/token"))
        })
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "cannot load device token file")
        })?;
    let mut data = Vec::new();
    File::open(path)
        .and_then(|f| f.take(131).read_to_end(&mut data))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "cannot load device token file"))?;
    if data.len() > 130 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot load device token file",
        ));
    }
    while matches!(data.last(), Some(b'\r' | b'\n')) {
        data.pop();
    }
    if !(16..=128).contains(&data.len())
        || data.iter().any(|b| matches!(b, b'\r' | b'\n' | 0))
        || std::str::from_utf8(&data).is_err()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot load device token file",
        ));
    }
    Ok(data)
}

#[cfg(target_os = "linux")]
fn process_start_ticks() -> Option<u64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    // The parenthesized command may contain spaces and closing parentheses.
    let rest = stat.rsplit_once(')')?.1;
    rest.split_whitespace().nth(19)?.parse().ok()
}
#[cfg(not(target_os = "linux"))]
fn process_start_ticks() -> Option<u64> {
    None
}

struct Capture {
    bench: PathBuf,
    host: String,
    port: u16,
    dir: PathBuf,
    log: File,
    times: File,
    offset: u64,
    started: String,
    connected: bool,
}
impl Capture {
    fn new(bench: &Path, host: &str, port: u16) -> io::Result<Self> {
        let dir = bench.join(format!("serial-{}", Local::now().format("%Y%m%d-%H%M%S")));
        fs::create_dir(&dir)?;
        let log = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("serial.bin"))?;
        let times = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("serial.times"))?;
        Ok(Self {
            bench: bench.into(),
            host: host.into(),
            port,
            dir,
            log,
            times,
            offset: 0,
            started: Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            connected: false,
        })
    }
    fn state(&self, status: &str, extra: Option<(&str, &str)>) -> io::Result<()> {
        let mut record = json!({"status": status, "port": format!("{}:{} (picousb)",self.host, self.port), "baud":115200, "format":"8N1", "pid":std::process::id(), "started":self.started, "directory":self.dir, "connected":self.connected, "engine":"vibeos-bench", "process_start_ticks":process_start_ticks()});
        if let Some((key, value)) = extra {
            record[key] = Value::String(value.into());
        }
        // A process-specific temporary file avoids collisions with other listeners.
        let tmp = self
            .bench
            .join(format!("serial-current.{}.tmp", std::process::id()));
        fs::write(&tmp, serde_json::to_vec_pretty(&record)?)?;
        replace(&tmp, &self.bench.join("serial-current.json"))
    }
    fn owns_state(&self) -> bool {
        fs::read(self.bench.join("serial-current.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|v| v.get("pid").and_then(Value::as_u64))
            .is_some_and(|pid| pid == u64::from(std::process::id()))
    }
    fn write_chunk(&mut self, data: &[u8]) -> io::Result<()> {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();
        writeln!(self.times, "{} {}", self.offset, ms)?;
        self.times.flush()?;
        self.log.write_all(data)?;
        self.log.flush()?;
        self.offset += data.len() as u64;
        Ok(())
    }
}
#[cfg(not(windows))]
fn replace(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}
#[cfg(windows)]
fn replace(from: &Path, to: &Path) -> io::Result<()> {
    // Windows rename does not replace an existing destination.
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};
    let a: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let b: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { MoveFileExW(a.as_ptr(), b.as_ptr(), MOVEFILE_REPLACE_EXISTING) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

enum ConnectError {
    Io(io::Error),
    Auth,
}
fn connect_port(host: &str, port: u16, token: &[u8]) -> Result<TcpStream, ConnectError> {
    let addrs = (host, port).to_socket_addrs().map_err(ConnectError::Io)?;
    let mut last = None;
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
            Ok(mut s) => {
                s.set_read_timeout(Some(Duration::from_secs(5)))
                    .map_err(ConnectError::Io)?;
                s.set_write_timeout(Some(Duration::from_secs(5)))
                    .map_err(ConnectError::Io)?;
                let mut hello = b"PICOSERIAL 1".to_vec();
                if !token.is_empty() {
                    hello.push(b' ');
                    hello.extend_from_slice(token);
                }
                hello.push(b'\n');
                s.write_all(&hello).map_err(ConnectError::Io)?;
                // Read bytewise to avoid consuming serial bytes delivered with the reply.
                let mut reply = Vec::new();
                let mut byte = [0];
                loop {
                    match s.read(&mut byte) {
                        Ok(0) => return Err(ConnectError::Auth),
                        Ok(_) => {
                            reply.push(byte[0]);
                            if byte[0] == b'\n' || reply.len() > 1024 {
                                break;
                            }
                        }
                        Err(e)
                            if matches!(
                                e.kind(),
                                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                            ) =>
                        {
                            return Err(ConnectError::Io(e))
                        }
                        Err(_) => return Err(ConnectError::Auth),
                    }
                }
                if reply != b"OK\n" {
                    return Err(ConnectError::Auth);
                }
                s.set_read_timeout(Some(Duration::from_millis(50)))
                    .map_err(ConnectError::Io)?;
                return Ok(s);
            }
            Err(e) => last = Some(e),
        }
    }
    Err(ConnectError::Io(last.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "host resolved to no addresses")
    })))
}
fn sleep_interruptible(stop: &AtomicBool, duration: Duration) {
    let end = Instant::now() + duration;
    while !stop.load(Ordering::SeqCst) && Instant::now() < end {
        thread::sleep(Duration::from_millis(50).min(end.saturating_duration_since(Instant::now())));
    }
}

/// Capture until SIGINT/SIGTERM (or the platform's console termination signal).
pub fn run_serial(bench: &Path, host: &str) -> crate::Result<()> {
    let token = token()?;
    let port = match env::var("PICO_SERIAL_PORT") {
        Ok(value) => value
            .parse::<u16>()
            .ok()
            .filter(|p| *p != 0)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid PICO_SERIAL_PORT")
            })?,
        Err(env::VarError::NotPresent) => 8082,
        Err(_) => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "invalid PICO_SERIAL_PORT").into(),
            )
        }
    };
    let mut capture = Capture::new(bench, host, port)?;
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    ctrlc::set_handler(move || flag.store(true, Ordering::SeqCst))?;
    capture.state("LISTENING", None)?;
    println!("LISTENING {host}:{port} -> {}", capture.dir.display());
    let result = (|| -> crate::Result<()> {
        let mut delay = 1;
        while !stop.load(Ordering::SeqCst) {
            match connect_port(host, port, &token) {
                Ok(mut stream) => {
                    capture.connected = true;
                    delay = 1;
                    capture.state("LISTENING", None)?;
                    let mut last_data = Instant::now();
                    let mut data = [0; 4096];
                    let mut idle = false;
                    while !stop.load(Ordering::SeqCst) {
                        match stream.read(&mut data) {
                            Ok(0) => break,
                            Ok(n) => {
                                capture.write_chunk(&data[..n])?;
                                last_data = Instant::now();
                            }
                            Err(e)
                                if matches!(
                                    e.kind(),
                                    io::ErrorKind::TimedOut
                                        | io::ErrorKind::WouldBlock
                                        | io::ErrorKind::Interrupted
                                ) => {}
                            Err(_) => break,
                        }
                        if last_data.elapsed() >= Duration::from_secs(120) {
                            idle = true;
                            break;
                        }
                        let path = bench.join("serial-send.bin");
                        match fs::read(&path) {
                            Ok(bytes) => {
                                if stream.write_all(&bytes).is_err() {
                                    break;
                                }
                                match fs::remove_file(&path) {
                                    Ok(()) => {}
                                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                                    Err(e) => return Err(e.into()),
                                }
                            }
                            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                            Err(e) => return Err(e.into()),
                        }
                    }
                    capture.connected = false;
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    if idle {
                        delay = 1;
                        continue;
                    }
                    eprintln!("pico-serial: console dropped; reconnecting");
                    capture.state("LISTENING", Some(("last_error", "ConnectionError")))?;
                }
                Err(ConnectError::Auth) => {
                    capture.connected = false;
                    capture.state("FAILED", Some(("error", "console authentication failed")))?;
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "console authentication failed",
                    )
                    .into());
                }
                Err(ConnectError::Io(e)) => {
                    capture.connected = false;
                    capture.state(
                        "LISTENING",
                        Some(("last_error", &format!("{:?}", e.kind()))),
                    )?;
                }
            }
            sleep_interruptible(&stop, Duration::from_secs(delay));
            delay = (delay * 2).min(8);
        }
        Ok(())
    })();
    capture.connected = false;
    if capture.owns_state() {
        if result.is_ok() {
            capture.state("STOPPED", None)?;
        } else if !matches!(result.as_ref().err().map(|e| e.to_string()), Some(s) if s == "console authentication failed")
        {
            capture.state("FAILED", Some(("error", "capture failed")))?;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    #[test]
    fn handshake_preserves_immediate_console_bytes() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut hello = Vec::new();
            let mut b = [0];
            loop {
                stream.read_exact(&mut b).unwrap();
                hello.push(b[0]);
                if b[0] == b'\n' {
                    break;
                }
            }
            assert_eq!(hello, b"PICOSERIAL 1 abcdefghijklmnop\n");
            stream.write_all(b"OK\n\0hello\xff").unwrap();
        });
        let mut stream = connect_port("127.0.0.1", port, b"abcdefghijklmnop")
            .ok()
            .unwrap();
        let mut data = [0; 7];
        stream.read_exact(&mut data).unwrap();
        assert_eq!(&data, b"\0hello\xff");
        server.join().unwrap();
    }
    #[test]
    fn rejected_auth_is_terminal() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            s.write_all(b"ERR\n").unwrap();
        });
        assert!(matches!(
            connect_port("127.0.0.1", port, b"abcdefghijklmnop"),
            Err(ConnectError::Auth)
        ));
        server.join().unwrap();
    }
}
