//! Attribute local daemon workers by the request environment, never by daemon
//! RSS or by timing overlap. Remote workers and the shared server are excluded.
use crate::memory;
use std::{
    collections::HashSet,
    fs,
    time::{Duration, Instant},
};

pub const MARKER: &str = "VIBEOS_HOST_MEMORY_REQUEST";

pub struct Workers {
    marker: Vec<u8>,
    servers: Vec<u32>,
    discovery: Instant,
    pub matched: HashSet<u32>,
    pub unreadable: usize,
    endpoint: Option<Vec<u8>>,
}

impl Workers {
    pub fn new(token: &str) -> Self {
        Self {
            marker: format!("{MARKER}={token}").into_bytes(),
            servers: vec![],
            discovery: Instant::now() - Duration::from_secs(1),
            matched: HashSet::new(),
            unreadable: 0,
            endpoint: ["SCCACHE_SERVER_UDS", "SCCACHE_SERVER_PORT"]
                .iter()
                .find_map(|key| {
                    std::env::var(key)
                        .ok()
                        .filter(|v| !v.is_empty())
                        .map(|value| format!("{key}={value}").into_bytes())
                }),
        }
    }

    pub fn extend(&mut self, pids: &mut HashSet<u32>, client: u32) {
        if self.discovery.elapsed() >= Duration::from_secs(1) {
            self.servers.clear();
            if let Ok(entries) = fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    let Some(pid) = entry.file_name().to_str().and_then(|s| s.parse().ok()) else {
                        continue;
                    };
                    if fs::read(entry.path().join("comm")).is_ok_and(|s| s == b"sccache\n") {
                        // Canonical builds select a worktree server endpoint.
                        // Avoid inspecting workers on every other lane's daemon.
                        if let Some(endpoint) = &self.endpoint {
                            match fs::read(entry.path().join("environ")) {
                                Ok(env) if matches_marker(&env, endpoint) => (),
                                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                                    self.unreadable += 1;
                                    continue;
                                }
                                _ => continue,
                            }
                        }
                        self.servers.push(pid);
                    }
                }
            } else {
                self.unreadable += 1;
            }
            self.discovery = Instant::now();
        }
        let mut candidates = HashSet::new();
        // A newly started server may briefly still be in the client's tree.
        // Its persistent allocation belongs to the host, never this request.
        pids.retain(|pid| *pid == client || !self.servers.contains(pid));
        for &server in &self.servers {
            // Recheck the name in case a previously discovered PID was reused.
            if fs::read(format!("/proc/{server}/comm")).is_ok_and(|s| s == b"sccache\n") {
                candidates.extend(memory::process_tree_pids(server));
            }
        }
        for pid in candidates {
            if pids.contains(&pid) || self.servers.contains(&pid) {
                continue;
            }
            match fs::read(format!("/proc/{pid}/environ")) {
                Ok(env) if matches_marker(&env, &self.marker) => {
                    pids.insert(pid);
                    self.matched.insert(pid);
                }
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => self.unreadable += 1,
                _ => (), // exited, or another request
            }
        }
    }
}

fn matches_marker(env: &[u8], marker: &[u8]) -> bool {
    env.split(|b| *b == 0).any(|field| field == marker)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };

    #[test]
    fn marker_is_an_exact_environment_field() {
        assert!(super::matches_marker(b"X=y\0TOKEN=123\0", b"TOKEN=123"));
        assert!(!super::matches_marker(b"TOKEN=1234\0", b"TOKEN=123"));
        assert!(!super::matches_marker(b"OTHER_TOKEN=123\0", b"TOKEN=123"));
    }

    #[test]
    fn daemon_requests_are_isolated_and_server_is_not_charged() {
        // The server is outside the measured client's tree, just like sccache.
        // Different request environments must not be conflated even when both
        // workers coexist under the same server.
        let token = format!("test-{}", std::process::id());
        let mut server = Command::new("python3")
            .args([
                "-c",
                r#"
import ctypes, os, subprocess, sys
ctypes.CDLL(None).prctl(15, b'sccache', 0, 0, 0)
body = '''import os,sys,ctypes
data=bytearray(64*1024*1024)
if os.environ['VIBEOS_HOST_MEMORY_REQUEST'] == 'another-request':
    ctypes.CDLL(None).prctl(4, 0, 0, 0, 0)
print(os.getpid(),flush=True)
sys.stdin.read()
'''
children=[]
for token in (sys.argv[1], 'another-request', 'readable-other-request'):
    env=dict(os.environ, VIBEOS_HOST_MEMORY_REQUEST=token)
    c=subprocess.Popen([sys.executable,'-c',body],env=env,stdout=subprocess.PIPE,text=True)
    print(c.stdout.readline().strip(),flush=True)
    children.append(c)
for c in children: c.wait()
"#,
                &token,
            ])
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(server.stdout.take().unwrap()).lines();
        let own: u32 = lines.next().unwrap().unwrap().parse().unwrap();
        let other: u32 = lines.next().unwrap().unwrap().parse().unwrap();
        let readable_other: u32 = lines.next().unwrap().unwrap().parse().unwrap();
        let mut workers = super::Workers::new(&token);
        let mut pids = HashSet::from([server.id()]);
        workers.extend(&mut pids, 0);
        let rss = crate::memory::process_rss_kib(&pids);
        drop(server.stdin.take());
        assert!(server.wait().unwrap().success());
        assert!(pids.contains(&own));
        assert!(!pids.contains(&other));
        assert!(!pids.contains(&readable_other));
        assert!(!pids.contains(&server.id()));
        assert!(rss >= 64 * 1024);
        assert!(workers.unreadable > 0);
    }
}
