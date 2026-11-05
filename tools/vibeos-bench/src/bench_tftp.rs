use crate::{Error, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn parse_request(payload: &[u8]) -> (Option<String>, Option<String>, BTreeMap<String, String>) {
    let parts: Vec<_> = payload.split(|b| *b == 0).collect();
    if parts.len() < 2 {
        return (None, None, BTreeMap::new());
    }
    let latin = |s: &[u8]| s.iter().map(|b| char::from(*b)).collect::<String>();
    let mut options = BTreeMap::new();
    for pair in parts[2..].chunks_exact(2) {
        if !pair[0].is_empty() {
            options.insert(latin(pair[0]).to_lowercase(), latin(pair[1]));
        }
    }
    (
        Some(latin(parts[0])),
        Some(latin(parts[1]).to_lowercase()),
        options,
    )
}
pub fn resolve(root: &Path, filename: &str) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let name = filename.replace('\\', "/");
    let path = root
        .join(name.trim_start_matches('/'))
        .canonicalize()
        .ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}
pub fn dispatch(req: &Value) -> Result<Value> {
    match req["op"].as_str().unwrap_or("") {
        "parse" => {
            let bytes: Vec<u8> = req["payload"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_u64().and_then(|n| u8::try_from(n).ok()))
                        .collect()
                })
                .unwrap_or_default();
            let (f, m, o) = parse_request(&bytes);
            Ok(json!([f, m, o]))
        }
        "resolve" => Ok(resolve(
            Path::new(req["root"].as_str().unwrap_or(".")),
            req["filename"].as_str().unwrap_or(""),
        )
        .map(|p| json!(p.to_string_lossy()))
        .unwrap_or(Value::Null)),
        _ => Err(Error::from("unknown TFTP operation".to_string())),
    }
}
fn packet_error(sock: &UdpSocket, addr: SocketAddr, code: u16, msg: &str) -> std::io::Result<()> {
    let mut packet = vec![0, 5, (code >> 8) as u8, code as u8];
    packet.extend_from_slice(msg.as_bytes());
    packet.push(0);
    sock.send_to(&packet, addr)?;
    Ok(())
}
fn await_ack(
    sock: &UdpSocket,
    addr: SocketAddr,
    packets: &[Vec<u8>],
    acked: u64,
    top: u64,
) -> std::io::Result<Option<u64>> {
    for _ in 0..5 {
        for p in packets {
            sock.send_to(p, addr)?;
        }
        loop {
            let mut b = [0u8; 1024];
            match sock.recv_from(&mut b) {
                Ok((n, source)) if source == addr && n >= 4 => {
                    let op = u16::from_be_bytes([b[0], b[1]]);
                    let block = u16::from_be_bytes([b[2], b[3]]);
                    if op == 5 {
                        return Ok(None);
                    }
                    if op == 4 {
                        for i in (acked..=top).rev() {
                            if i as u16 == block {
                                if i > acked || top == acked {
                                    return Ok(Some(i));
                                }
                                break;
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok(None)
}
pub fn serve_file(
    path: &Path,
    addr: SocketAddr,
    options: &BTreeMap<String, String>,
) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let mut block = 512usize;
    let mut window = 1u64;
    let mut accepted = Vec::new();
    if let Some(v) = options.get("blksize") {
        block = v.parse::<i64>().unwrap_or(512).clamp(8, 65464) as usize;
        accepted.push(("blksize", block.to_string()));
    }
    if let Some(v) = options.get("windowsize") {
        window = v.parse::<i64>().unwrap_or(1).clamp(1, 64) as u64;
        accepted.push(("windowsize", window.to_string()));
    }
    if options.contains_key("tsize") {
        accepted.push(("tsize", size.to_string()));
    }
    let sock = UdpSocket::bind(if addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    })?;
    sock.set_read_timeout(Some(Duration::from_secs(2)))?;
    if !accepted.is_empty() {
        let mut oack = vec![0, 6];
        for (k, v) in accepted {
            oack.extend_from_slice(k.as_bytes());
            oack.push(0);
            oack.extend_from_slice(v.as_bytes());
            oack.push(0);
        }
        if await_ack(&sock, addr, &[oack], 0, 0)?.is_none() {
            return Ok(());
        }
    }
    let last = size / block as u64 + 1;
    let mut acked = 0u64;
    while acked < last {
        let top = (acked + window).min(last);
        let mut packets = Vec::new();
        for n in acked + 1..=top {
            let mut packet = vec![0, 3, (n >> 8) as u8, n as u8];
            file.seek(SeekFrom::Start((n - 1) * block as u64))?;
            let mut data = vec![0; block];
            let mut used = 0;
            while used < block {
                let got = file.read(&mut data[used..])?;
                if got == 0 {
                    break;
                }
                used += got;
            }
            packet.extend_from_slice(&data[..used]);
            packets.push(packet);
        }
        match await_ack(&sock, addr, &packets, acked, top)? {
            Some(n) => acked = n,
            None => return Ok(()),
        }
    }
    Ok(())
}
pub fn handle(
    sock: &UdpSocket,
    root: &Path,
    payload: &[u8],
    addr: SocketAddr,
) -> std::io::Result<()> {
    if payload.len() < 4 {
        return Ok(());
    }
    match u16::from_be_bytes([payload[0], payload[1]]) {
        2 => packet_error(sock, addr, 2, "server is read-only"),
        1 => {
            let (filename, _, options) = parse_request(&payload[2..]);
            if let Some(path) = filename.as_deref().and_then(|s| resolve(root, s)) {
                serve_file(&path, addr, &options)
            } else {
                packet_error(sock, addr, 1, "file not found")
            }
        }
        _ => Ok(()),
    }
}
pub fn run(root: &Path, bind: &str, port: u16) -> Result<()> {
    let root = fs::canonicalize(root)?;
    let sock = UdpSocket::bind((bind, port))?;
    println!("TFTP read-only on {bind}:{port} serving {}", root.display());
    let mut packet = [0u8; 65536];
    loop {
        let (n, addr) = sock.recv_from(&mut packet)?;
        handle(&sock, &root, &packet[..n], addr)?;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_options() {
        let (f, m, o) = parse_request(b"boot.img\0OCTET\0blksize\01408\0");
        assert_eq!(f.as_deref(), Some("boot.img"));
        assert_eq!(m.as_deref(), Some("octet"));
        assert_eq!(o.get("blksize").map(String::as_str), Some("1408"));
    }
    #[test]
    fn socket_oack_window_and_empty_tail() {
        let root = std::env::temp_dir().join(format!("bench-tftp-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("x"), b"12345678").unwrap();
        let path = root.join("x");
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let addr = client.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let mut options = BTreeMap::new();
            options.insert("blksize".into(), "8".into());
            options.insert("windowsize".into(), "2".into());
            serve_file(&path, addr, &options).unwrap();
        });
        let mut buf = [0u8; 1024];
        let (n, peer) = client.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..2], &[0, 6]);
        assert!(n > 2);
        client.send_to(&[0, 4, 0, 0], peer).unwrap();
        let (n, _) = client.recv_from(&mut buf).unwrap();
        assert_eq!(n, 12);
        assert_eq!(&buf[..4], &[0, 3, 0, 1]);
        let (n, _) = client.recv_from(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf[..4], &[0, 3, 0, 2]);
        client.send_to(&[0, 4, 0, 2], peer).unwrap();
        server.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn wraps_ack_and_ignores_other_peers() {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let server_addr = sock.local_addr().unwrap();
        let peer = UdpSocket::bind("127.0.0.1:0").unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let peer_addr = peer.local_addr().unwrap();
        let other = UdpSocket::bind("127.0.0.1:0").unwrap();
        let worker = std::thread::spawn(move || {
            await_ack(&sock, peer_addr, &[vec![0, 3, 0, 0]], 65535, 65536).unwrap()
        });
        let mut buf = [0u8; 16];
        peer.recv_from(&mut buf).unwrap();
        other.send_to(&[0, 4, 0, 0], server_addr).unwrap();
        peer.send_to(&[0, 4, 0, 0], server_addr).unwrap();
        assert_eq!(worker.join().unwrap(), Some(65536));
    }
    #[cfg(unix)]
    #[test]
    fn refuses_symlink_escape() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("bench-tftp-link-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        symlink("/etc/passwd", root.join("escape")).unwrap();
        assert!(resolve(&root, "escape").is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
