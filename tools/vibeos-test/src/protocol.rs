//! Native serial and QMP transports. Serial is drained for its entire lifetime.
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
}
impl Stream {
    pub fn clone_stream(&self) -> Result<Self> {
        Ok(match self {
            Self::Tcp(s) => Self::Tcp(s.try_clone()?),
            Self::Unix(s) => Self::Unix(s.try_clone()?),
        })
    }
    pub fn timeout(&self, v: Option<Duration>) -> Result<()> {
        match self {
            Self::Tcp(s) => {
                s.set_read_timeout(v)?;
                s.set_write_timeout(v)?
            }
            Self::Unix(s) => {
                s.set_read_timeout(v)?;
                s.set_write_timeout(v)?
            }
        }
        Ok(())
    }
    pub fn shutdown(&self) {
        let _ = match self {
            Self::Tcp(s) => s.shutdown(Shutdown::Both),
            Self::Unix(s) => s.shutdown(Shutdown::Both),
        };
    }
}
impl Read for Stream {
    fn read(&mut self, v: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(s) => s.read(v),
            Self::Unix(s) => s.read(v),
        }
    }
}
impl Write for Stream {
    fn write(&mut self, v: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(s) => s.write(v),
            Self::Unix(s) => s.write(v),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(s) => s.flush(),
            Self::Unix(s) => s.flush(),
        }
    }
}
pub enum Listener {
    Tcp(TcpListener),
    Unix(UnixListener),
}
impl Listener {
    pub fn tcp(host: &str, port: u16) -> Result<(Self, u16)> {
        let s = TcpListener::bind((host, port))?;
        s.set_nonblocking(true)?;
        let p = s.local_addr()?.port();
        Ok((Self::Tcp(s), p))
    }
    pub fn unix(path: &Path) -> Result<Self> {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let s = UnixListener::bind(path)?;
        s.set_nonblocking(true)?;
        Ok(Self::Unix(s))
    }
    pub fn accept(&self, timeout: Duration) -> Result<Stream> {
        let end = Instant::now() + timeout;
        loop {
            let r = match self {
                Self::Tcp(s) => s.accept().map(|(s, _)| Stream::Tcp(s)),
                Self::Unix(s) => s.accept().map(|(s, _)| Stream::Unix(s)),
            };
            match r {
                Ok(s) => return Ok(s),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e.into()),
            }
            if Instant::now() >= end {
                return Err(Error::new("socket accept timed out"));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|b| char::from(*b)).collect()
}
#[cfg(test)]
fn unicode_offset(text: &str, count: usize) -> usize {
    text.char_indices()
        .nth(count)
        .map(|x| x.0)
        .unwrap_or(text.len())
}

struct SerialBuffer {
    bytes: Vec<u8>,
    last: Instant,
    dead: bool,
    error: Option<String>,
}
pub struct SerialClient {
    listener: Option<Listener>,
    writer: Option<Stream>,
    shared: Arc<Mutex<SerialBuffer>>,
    closed: Arc<AtomicBool>,
    reader: Option<JoinHandle<()>>,
    pos: usize,
    pub partial_output: String,
    tee: Option<PathBuf>,
    stamp: bool,
    file: Option<PathBuf>,
    alive: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    external_alive: Arc<AtomicBool>,
}
impl Default for SerialClient {
    fn default() -> Self {
        Self::new()
    }
}
impl SerialClient {
    pub fn new() -> Self {
        Self {
            listener: None,
            writer: None,
            shared: Arc::new(Mutex::new(SerialBuffer {
                bytes: Vec::new(),
                last: Instant::now(),
                dead: false,
                error: None,
            })),
            closed: Arc::new(AtomicBool::new(false)),
            reader: None,
            pos: 0,
            partial_output: String::new(),
            tee: None,
            stamp: std::env::var("VIBEOS_SERIAL_TIMESTAMPS").as_deref() == Ok("1"),
            file: None,
            alive: None,
            external_alive: Arc::new(AtomicBool::new(true)),
        }
    }
    pub fn listen(&mut self, path: &Path) -> Result<()> {
        self.listener = Some(Listener::unix(path)?);
        Ok(())
    }
    pub fn listen_tcp(&mut self, host: &str, port: u16) -> Result<u16> {
        let (l, p) = Listener::tcp(host, port)?;
        self.listener = Some(l);
        Ok(p)
    }
    pub fn set_tee(&mut self, path: &Path, header: &str) -> Result<()> {
        let result = (|| -> std::io::Result<()> {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut f = File::create(path)?;
            f.write_all(header.as_bytes())?;
            f.flush()?;
            Ok(())
        })();
        if let Err(e) = result {
            let message = format!("serial tee {}: {e}", path.display());
            self.shared
                .lock()
                .unwrap()
                .error
                .get_or_insert(message.clone());
            return Err(Error::new(message).with_kind("OSError"));
        }
        self.tee = Some(path.into());
        Ok(())
    }
    pub fn set_external_alive(&self, value: bool) {
        self.external_alive.store(value, Ordering::Release);
    }
    pub fn follow_file(&mut self, path: &Path, alive: Arc<dyn Fn() -> bool + Send + Sync>) {
        self.file = Some(path.into());
        self.alive = Some(alive);
    }
    pub fn accept(&mut self, timeout: Duration) -> Result<()> {
        self.accept_with_probe(timeout, None)
    }
    fn refresh_liveness(&self, probe: &mut Option<&mut dyn FnMut() -> Result<bool>>) -> Result<()> {
        if let Some(callback) = probe.as_mut() {
            self.set_external_alive((**callback)()?);
        }
        Ok(())
    }
    pub fn accept_with_probe(
        &mut self,
        timeout: Duration,
        mut probe: Option<&mut dyn FnMut() -> Result<bool>>,
    ) -> Result<()> {
        if self.reader.is_some() {
            return Err("serial drainer already started".into());
        }
        let mut stream = None;
        let mut file = None;
        if let Some(path) = &self.file {
            let end = Instant::now() + timeout;
            while !path.exists() {
                self.refresh_liveness(&mut probe)?;
                if !self.external_alive.load(Ordering::Acquire) && !path.exists() {
                    return Err("Serial file producer exited before creating its output".into());
                }
                if Instant::now() >= end {
                    return Err(Error::new(format!(
                        "QEMU did not create {} within {}s",
                        path.display(),
                        timeout.as_secs_f64()
                    )));
                }
                thread::sleep(Duration::from_millis(50));
            }
            file = Some(File::open(path)?);
        } else {
            let s = self
                .listener
                .as_ref()
                .ok_or("serial listener not configured")?
                .accept(timeout)?;
            s.timeout(Some(Duration::from_millis(200)))?;
            self.writer = Some(s.clone_stream()?);
            stream = Some(s);
        }
        let shared = self.shared.clone();
        let closed = self.closed.clone();
        let tee = self.tee.clone();
        let stamp = self.stamp;
        let alive = self.alive.clone();
        let external_alive = self.external_alive.clone();
        self.reader = Some(thread::spawn(move || {
            let tee_path = tee;
            let mut tee = match tee_path
                .as_ref()
                .map(|p| OpenOptions::new().append(true).open(p))
            {
                Some(Ok(file)) => Some(file),
                Some(Err(e)) => {
                    shared.lock().unwrap().error.get_or_insert_with(|| {
                        format!("serial tee {}: {e}", tee_path.as_ref().unwrap().display())
                    });
                    None
                }
                None => None,
            };
            let mut first = None;
            let mut bol = true;
            let mut buf = [0u8; 65536];
            // A producer may append its final bytes between read(0) and the
            // liveness observation. Re-read once AFTER observing its death;
            // only an EOF beyond that fence proves every final byte drained.
            let mut producer_finished = false;
            while !closed.load(Ordering::Acquire) {
                let result = if let Some(s) = &mut stream {
                    s.read(&mut buf)
                } else {
                    file.as_mut().unwrap().read(&mut buf)
                };
                match result {
                    Ok(0) => {
                        if file.is_some() {
                            if external_alive.load(Ordering::Acquire)
                                && alive.as_ref().is_some_and(|a| a())
                            {
                                producer_finished = false;
                                thread::sleep(Duration::from_millis(50));
                                continue;
                            }
                            if !producer_finished {
                                producer_finished = true;
                                continue;
                            }
                        }
                        if let Ok(mut b) = shared.lock() {
                            b.dead = true;
                        }
                        break;
                    }
                    Ok(n) => {
                        let now = Instant::now();
                        let mut tee_error = None;
                        if let Some(f) = &mut tee {
                            let mut text = Vec::new();
                            if stamp {
                                let start = *first.get_or_insert(now);
                                let prefix =
                                    format!("[+{:9.3}] ", now.duration_since(start).as_secs_f64());
                                for &byte in &buf[..n] {
                                    if bol {
                                        text.extend_from_slice(prefix.as_bytes());
                                    }
                                    text.push(byte);
                                    bol = byte == b'\n' || byte == b'\r';
                                }
                            } else {
                                text.extend_from_slice(&buf[..n]);
                            }
                            if let Err(e) = f.write_all(&text).and_then(|_| f.flush()) {
                                tee_error = Some(format!(
                                    "serial tee {}: {e}",
                                    tee_path.as_ref().unwrap().display()
                                ));
                                tee = None;
                            }
                        }
                        // Publish each raw chunk and its log-I/O result under
                        // one mutex. No waiter can observe READY before learning
                        // that the requested durable write failed.
                        let tail = {
                            let mut b = shared.lock().unwrap();
                            b.bytes.extend_from_slice(&buf[..n]);
                            b.last = now;
                            if let Some(message) = tee_error {
                                b.error.get_or_insert(message);
                            }
                            b.bytes[b.bytes.len().saturating_sub(n + 80)..].to_vec()
                        };
                        if let Some(s) = &mut stream {
                            if tail.windows(5).any(|x| x == b"kdb:>") {
                                let _ = s.write_all(b"cont\n");
                            } else if tail
                                .windows(29)
                                .any(|x| x == b"any other key to continue ---")
                            {
                                let _ = s.write_all(b" ");
                            }
                        }
                    }
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock
                                | std::io::ErrorKind::TimedOut
                                | std::io::ErrorKind::Interrupted
                        ) => {}
                    Err(e) => {
                        if closed.load(Ordering::Acquire) {
                            break;
                        }
                        if let Ok(mut b) = shared.lock() {
                            b.error.get_or_insert_with(|| e.to_string());
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        }));
        Ok(())
    }
    pub fn snapshot(&self) -> (String, Duration, bool) {
        let b = self.shared.lock().unwrap();
        (latin1(&b.bytes), b.last.elapsed(), b.dead)
    }
    pub fn read_until(&mut self, marker: &str, timeout: Duration) -> Result<String> {
        self.read_until_with_probe(marker, timeout, None)
    }
    pub fn read_until_with_probe(
        &mut self,
        marker: &str,
        timeout: Duration,
        mut probe: Option<&mut dyn FnMut() -> Result<bool>>,
    ) -> Result<String> {
        let start = self.pos;
        let end = Instant::now() + timeout;
        loop {
            self.refresh_liveness(&mut probe)?;
            let b = self.shared.lock().unwrap();
            if let Some(message) = &b.error {
                self.partial_output = latin1(&b.bytes[start..]);
                return Err(Error::new(message.clone()).with_kind("OSError"));
            }
            let window = latin1(&b.bytes[start..]);
            if let Some(index) = window.find(marker) {
                let end_byte = index + marker.len();
                let chars = window[..end_byte].chars().count();
                self.pos = start + chars;
                return Ok(window[..end_byte].to_owned());
            }
            let dead = b.dead;
            drop(b);
            if dead {
                self.partial_output = window;
                return Err("Serial socket closed".into());
            }
            if Instant::now() >= end {
                self.partial_output = window.clone();
                return Err(Error::new(format!(
                    "Timed out waiting for marker {marker:?} after {}s ({} bytes captured)",
                    timeout.as_secs_f64(),
                    window.chars().count()
                )));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
    pub fn read_until_idle(
        &mut self,
        idle: Duration,
        timeout: Duration,
        min_bytes: usize,
    ) -> Result<String> {
        self.read_until_idle_with_probe(idle, timeout, min_bytes, None)
    }
    pub fn read_until_idle_with_probe(
        &mut self,
        idle: Duration,
        timeout: Duration,
        min_bytes: usize,
        mut probe: Option<&mut dyn FnMut() -> Result<bool>>,
    ) -> Result<String> {
        let start = self.pos;
        let end = Instant::now() + timeout;
        loop {
            self.refresh_liveness(&mut probe)?;
            let b = self.shared.lock().unwrap();
            if let Some(message) = &b.error {
                self.partial_output = latin1(&b.bytes[start..]);
                return Err(Error::new(message.clone()).with_kind("OSError"));
            }
            let window = latin1(&b.bytes[start..]);
            let done = b.bytes.len() - start >= min_bytes && b.last.elapsed() >= idle
                || b.dead && !window.is_empty();
            if done {
                self.pos = b.bytes.len();
                return Ok(window);
            }
            let dead = b.dead;
            drop(b);
            if probe.is_some() && dead {
                self.partial_output = window;
                return Err("Serial socket closed".into());
            }
            if Instant::now() >= end {
                self.partial_output = window;
                return Err("Timed out waiting for serial idle".into());
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
    pub fn read_available(&mut self, timeout: Duration) -> Result<String> {
        self.read_available_with_probe(timeout, None)
    }
    pub fn read_available_with_probe(
        &mut self,
        timeout: Duration,
        mut probe: Option<&mut dyn FnMut() -> Result<bool>>,
    ) -> Result<String> {
        let end = Instant::now() + timeout;
        loop {
            self.refresh_liveness(&mut probe)?;
            let b = self.shared.lock().unwrap();
            if let Some(message) = &b.error {
                self.partial_output = latin1(&b.bytes[self.pos..]);
                return Err(Error::new(message.clone()).with_kind("OSError"));
            }
            if b.bytes.len() > self.pos {
                let out = latin1(&b.bytes[self.pos..]);
                self.pos = b.bytes.len();
                return Ok(out);
            }
            let dead = b.dead;
            drop(b);
            if probe.is_some() && dead {
                return Ok(String::new());
            }
            if Instant::now() >= end {
                return Ok(String::new());
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        if self.file.is_some() {
            return Err("serial input is unavailable under record/replay (file transport)".into());
        }
        self.writer
            .as_mut()
            .ok_or("serial not connected")?
            .write_all(data)?;
        Ok(())
    }
    pub fn close(&mut self) {
        self.closed.store(true, Ordering::Release);
        if let Some(s) = self.writer.take() {
            s.shutdown();
        }
        if let Some(h) = self.reader.take() {
            let _ = h.join();
        }
        self.listener = None;
    }
    pub fn consumed_position(&self) -> usize {
        self.pos
    }
    pub fn set_consumed_position(&mut self, value: usize) -> Result<()> {
        if value > self.shared.lock().unwrap().bytes.len() {
            return Err("serial consume position past buffer".into());
        }
        self.pos = value;
        Ok(())
    }
}
impl Drop for SerialClient {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct QmpClient {
    stream: Option<Stream>,
    listener: Option<Listener>,
    buffer: Vec<u8>,
    pub events: VecDeque<Value>,
    timeout: Duration,
}
impl Default for QmpClient {
    fn default() -> Self {
        Self::new()
    }
}
fn qmp_io_error(error: std::io::Error) -> Error {
    use std::io::ErrorKind;
    let closed = matches!(
        error.kind(),
        ErrorKind::BrokenPipe
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::UnexpectedEof
    );
    Error::new(error.to_string()).with_kind(if closed { "QmpClosed" } else { "QmpTransport" })
}
impl QmpClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            listener: None,
            buffer: Vec::new(),
            events: VecDeque::new(),
            timeout: Duration::from_secs(10),
        }
    }
    pub fn connect(&mut self, path: &Path, timeout: Duration) -> Result<Value> {
        self.timeout = timeout;
        let s = Stream::Unix(UnixStream::connect(path)?);
        s.timeout(Some(timeout))?;
        self.stream = Some(s);
        self.handshake()
    }
    pub fn connect_tcp(&mut self, host: &str, port: u16, timeout: Duration) -> Result<Value> {
        self.timeout = timeout;
        let address = (host, port)
            .to_socket_addrs()?
            .next()
            .ok_or("QMP host resolved no address")?;
        let stream = Stream::Tcp(TcpStream::connect_timeout(&address, timeout)?);
        stream.timeout(Some(timeout))?;
        self.stream = Some(stream);
        self.handshake()
    }
    pub fn listen_tcp(&mut self, host: &str, port: u16) -> Result<u16> {
        let (l, p) = Listener::tcp(host, port)?;
        self.listener = Some(l);
        Ok(p)
    }
    pub fn accept(&mut self, timeout: Duration) -> Result<Value> {
        self.timeout = timeout;
        let s = self
            .listener
            .as_ref()
            .ok_or("QMP listener not configured")?
            .accept(timeout)?;
        s.timeout(Some(timeout))?;
        self.stream = Some(s);
        self.handshake()
    }
    fn remember(&mut self, v: Value) {
        if self.events.len() == 256 {
            self.events.pop_front();
        }
        self.events.push_back(v);
    }
    fn recv_one(&mut self) -> Result<Value> {
        loop {
            if let Some(n) = self.buffer.iter().position(|x| *x == b'\n') {
                let line = self.buffer.drain(..=n).collect::<Vec<_>>();
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                return Ok(serde_json::from_slice(&line)?);
            }
            let mut buf = [0u8; 4096];
            let n = self
                .stream
                .as_mut()
                .ok_or("QMP not connected")?
                .read(&mut buf)
                .map_err(qmp_io_error)?;
            if n == 0 {
                return Err(Error::new("QMP socket closed").with_kind("QmpClosed"));
            }
            self.buffer.extend_from_slice(&buf[..n]);
            if self.buffer.len() > 16 * 1024 * 1024 {
                return Err("QMP response exceeds 16 MiB".into());
            }
        }
    }
    fn recv_reply(&mut self) -> Result<Value> {
        loop {
            let v = self.recv_one()?;
            if v.get("event").is_some() {
                self.remember(v);
            } else {
                return Ok(v);
            }
        }
    }
    fn send(&mut self, v: &Value) -> Result<()> {
        let mut bytes = serde_json::to_vec(v)?;
        bytes.push(b'\n');
        self.stream
            .as_mut()
            .ok_or("QMP not connected")?
            .write_all(&bytes)
            .map_err(qmp_io_error)?;
        Ok(())
    }
    fn checked(command: &str, v: Value) -> Result<Value> {
        if let Some(e) = v.get("error") {
            return Err(Error {
                kind: Some("QmpError".into()),
                message: json!({"command":command,"error":e}).to_string(),
            });
        }
        Ok(v)
    }
    fn handshake(&mut self) -> Result<Value> {
        let greeting = self.recv_reply()?;
        if greeting.get("QMP").is_none() {
            return Err("QMP greeting lacks QMP field".into());
        }
        self.send(&json!({"execute":"qmp_capabilities"}))?;
        Self::checked("qmp_capabilities", self.recv_reply()?)?;
        Ok(greeting)
    }
    pub fn execute(&mut self, command: &str, args: Value) -> Result<Value> {
        let mut v = json!({"execute":command});
        if args.as_object().is_some_and(|x| !x.is_empty()) {
            v["arguments"] = args;
        }
        self.send(&v)?;
        Self::checked(command, self.recv_reply()?)
    }
    /// Bulk transfers may need longer than interactive monitor requests. Restore
    /// the connection's normal deadline on both success and failure.
    pub fn execute_with_timeout(
        &mut self,
        command: &str,
        args: Value,
        timeout: Duration,
    ) -> Result<Value> {
        self.stream
            .as_ref()
            .ok_or("QMP not connected")?
            .timeout(Some(timeout))?;
        let result = self.execute(command, args);
        let restored = self
            .stream
            .as_ref()
            .ok_or("QMP not connected")?
            .timeout(Some(self.timeout));
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                restored?;
                Ok(value)
            }
        }
    }
    pub fn hmp(&mut self, command: &str) -> Result<String> {
        let v = self.execute("human-monitor-command", json!({"command-line":command}))?;
        Ok(v["return"].as_str().unwrap_or("").to_owned())
    }
    pub fn events_snapshot(&self) -> Value {
        json!(self.events)
    }
    pub fn saw_event(&self, name: &str) -> Option<Value> {
        self.events
            .iter()
            .rev()
            .find(|v| v["event"].as_str() == Some(name))
            .cloned()
    }
    pub fn wait_event(&mut self, name: &str, timeout: Duration) -> Result<Option<Value>> {
        if let Some(v) = self.saw_event(name) {
            return Ok(Some(v));
        }
        let end = Instant::now() + timeout;
        while Instant::now() < end {
            let left = end
                .saturating_duration_since(Instant::now())
                .min(Duration::from_secs(5));
            self.stream
                .as_ref()
                .ok_or("QMP not connected")?
                .timeout(Some(left))?;
            match self.recv_one() {
                Ok(v) => {
                    if v.get("event").is_some() {
                        let found = v["event"].as_str() == Some(name);
                        self.remember(v.clone());
                        if found {
                            self.stream.as_ref().unwrap().timeout(Some(self.timeout))?;
                            return Ok(Some(v));
                        }
                    }
                }
                Err(e) => {
                    if !e.message.contains("timed out")
                        && !e.message.contains("temporarily unavailable")
                    {
                        self.stream.as_ref().unwrap().timeout(Some(self.timeout))?;
                        return Err(e);
                    }
                }
            }
        }
        if let Some(s) = &self.stream {
            s.timeout(Some(self.timeout))?;
        }
        Ok(None)
    }
    pub fn close(&mut self) {
        if let Some(s) = self.stream.take() {
            s.shutdown();
        }
        self.listener = None;
    }
}
impl Drop for QmpClient {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn latin1_is_byte_preserving() {
        assert_eq!(latin1(&[b'a', 255]), "aÿ");
        assert_eq!(unicode_offset("aÿ", 2), 3);
    }
    #[test]
    fn events_are_bounded() {
        let mut c = QmpClient::new();
        for n in 0..300 {
            c.remember(json!({"event":"STOP","n":n}));
        }
        assert_eq!(c.events.len(), 256);
        assert_eq!(c.saw_event("STOP").unwrap()["n"], 299);
    }
    #[test]
    fn lifetime_drainer_preserves_unread_bytes_and_marker_remainder() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("serial.sock");
        let mut client = SerialClient::new();
        client.listen(&path).unwrap();
        let writer = thread::spawn(move || {
            let mut stream = UnixStream::connect(path).unwrap();
            stream.write_all(b"READY\xffafter").unwrap();
            thread::sleep(Duration::from_millis(150));
        });
        client.accept(Duration::from_secs(1)).unwrap();
        thread::sleep(Duration::from_millis(50));
        assert_eq!(
            client.read_until("READY", Duration::from_secs(1)).unwrap(),
            "READY"
        );
        assert_eq!(
            client.read_available(Duration::from_secs(1)).unwrap(),
            "ÿafter"
        );
        writer.join().unwrap();
        client.close();
    }
    #[test]
    fn qmp_skips_and_retains_async_event_before_reply() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("qmp.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.write_all(b"{\"QMP\":{}}\n").unwrap();
            let mut input = std::io::BufReader::new(stream.try_clone().unwrap());
            use std::io::BufRead;
            for _ in 0..2 {
                let mut line = String::new();
                input.read_line(&mut line).unwrap();
                stream
                    .write_all(b"{\"event\":\"STOP\"}\n{\"return\":{}}\n")
                    .unwrap();
            }
        });
        let mut client = QmpClient::new();
        client.connect(&path, Duration::from_secs(1)).unwrap();
        client.execute("query-status", json!({})).unwrap();
        assert_eq!(client.saw_event("STOP").unwrap()["event"], "STOP");
        client.close();
        server.join().unwrap();
    }
}

// Append to protocol.rs. Uses the same native wait/parser and lifetime drainer.
#[cfg(test)]
mod serial_probe_controls {
    use super::*;
    #[test]
    fn file_death_fence_rereads_bytes_appended_after_the_first_eof() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("serial");
        fs::write(&path, b"before\n").unwrap();
        let final_path = path.clone();
        let appended = Arc::new(AtomicBool::new(false));
        let flag = appended.clone();
        let mut serial = SerialClient::new();
        serial.follow_file(
            &path,
            Arc::new(move || {
                // Runs precisely after read(0) but before the producer-dead branch.
                // Final bytes must be read again, never discarded as stale EOF.
                if !flag.swap(true, Ordering::AcqRel) {
                    let mut f = OpenOptions::new().append(true).open(&final_path).unwrap();
                    f.write_all(b"FINAL\n").unwrap();
                    f.flush().unwrap();
                }
                false
            }),
        );
        serial.accept(Duration::from_secs(1)).unwrap();
        assert_eq!(
            serial.read_until("FINAL", Duration::from_secs(1)).unwrap(),
            "before\nFINAL"
        );
        serial.close();
    }
    #[test]
    fn blocking_native_wait_refreshes_a_live_probe_and_preserves_partials() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("serial");
        fs::write(&path, b"partial progress").unwrap();
        let mut serial = SerialClient::new();
        serial.follow_file(&path, Arc::new(|| true));
        serial.accept(Duration::from_secs(1)).unwrap();
        let start = Instant::now();
        let mut calls = 0;
        let mut probe = || {
            calls += 1;
            Ok(start.elapsed() < Duration::from_millis(50))
        };
        let error = serial
            .read_until_with_probe("never", Duration::from_millis(800), Some(&mut probe))
            .unwrap_err();
        assert!(error.to_string().contains("closed"));
        assert!(calls > 1);
        assert_eq!(serial.partial_output, "partial progress");
        serial.close();
    }
}

// Append to protocol.rs. All I/O uses the actual native socket/file drainer.
#[cfg(test)]
mod serial_tee_controls {
    use super::*;
    #[test]
    fn failed_log_cannot_publish_a_successful_marker_and_does_not_stop_draining() {
        let d = tempfile::tempdir().unwrap();
        let socket = d.path().join("serial.sock");
        let mut serial = SerialClient::new();
        serial.set_tee(Path::new("/dev/full"), "").unwrap();
        serial.listen(&socket).unwrap();
        let mut writer = UnixStream::connect(&socket).unwrap();
        serial.accept(Duration::from_secs(1)).unwrap();
        writer.write_all(b"READY\n").unwrap();
        let error = serial
            .read_until("READY", Duration::from_secs(1))
            .unwrap_err();
        assert_eq!(error.kind.as_deref(), Some("OSError"));
        assert!(error.message.contains("serial tee /dev/full:"));
        assert!(serial.partial_output.contains("READY"));
        writer
            .write_all(b"raw evidence after failure\nEND")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !serial.snapshot().0.ends_with("END") && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        let (raw, _, dead) = serial.snapshot();
        assert!(raw.ends_with("END"));
        assert!(!dead);
        assert!(serial
            .read_available(Duration::ZERO)
            .unwrap_err()
            .message
            .contains("/dev/full"));
        assert!(serial
            .read_until_idle(Duration::ZERO, Duration::from_secs(1), 0)
            .unwrap_err()
            .message
            .contains("/dev/full"));
        serial.close();
    }
}

#[cfg(test)]
mod bulk_timeout_controls {
    use super::*;
    #[test]
    fn bulk_timeout_restores_interactive_socket_deadline_on_success_and_error() {
        for reply in [
            json!({"return":{}}),
            json!({"error":{"class":"GenericError","desc":"controlled rejection"}}),
        ] {
            let (client, mut server) = UnixStream::pair().unwrap();
            let mut qmp = QmpClient::new();
            qmp.timeout = Duration::from_millis(250);
            client.set_read_timeout(Some(qmp.timeout)).unwrap();
            client.set_write_timeout(Some(qmp.timeout)).unwrap();
            let baseline = (
                client.read_timeout().unwrap(),
                client.write_timeout().unwrap(),
            );
            qmp.stream = Some(Stream::Unix(client));
            writeln!(server, "{reply}").unwrap();
            let result = qmp.execute_with_timeout("pmemsave", json!({}), Duration::from_secs(120));
            assert_eq!(result.is_err(), reply.get("error").is_some());
            let Stream::Unix(socket) = qmp.stream.as_ref().unwrap() else {
                panic!("wrong socket");
            };
            assert_eq!(socket.read_timeout().unwrap(), baseline.0);
            assert_eq!(socket.write_timeout().unwrap(), baseline.1);
        }
    }
}
