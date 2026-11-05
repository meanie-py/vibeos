use crate::{Error, Result};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

pub fn resolve(root: &Path, name: &str) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let name = name.split('?').next()?.trim_start_matches('/');
    let path = root.join(name).canonicalize().ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}

pub fn span(size: u64, range: Option<&str>) -> Option<(u64, u64, bool)> {
    let Some(range) = range.filter(|s| !s.is_empty()) else {
        return Some((0, size.saturating_sub(1), false));
    };
    let (a, b) = range.trim().strip_prefix("bytes=")?.split_once('-')?;
    if a.is_empty() && b.is_empty() {
        return None;
    }
    if size == 0 {
        return None;
    }
    let (start, end) = if a.is_empty() {
        let count: u64 = b.parse().ok()?;
        (size.saturating_sub(count), size - 1)
    } else {
        let start: u64 = a.parse().ok()?;
        let end = if b.is_empty() {
            size - 1
        } else {
            b.parse::<u64>().ok()?.min(size - 1)
        };
        (start, end)
    };
    (start < size && start <= end).then_some((start, end, true))
}

pub fn dispatch(req: &Value) -> Result<Value> {
    match req["op"].as_str().unwrap_or("") {
        "span" => Ok(
            match span(req["size"].as_u64().unwrap_or(0), req["range"].as_str()) {
                Some((a, b, p)) => json!([a, b, p]),
                None => Value::Null,
            },
        ),
        "resolve" => Ok(resolve(
            Path::new(req["root"].as_str().unwrap_or(".")),
            req["path"].as_str().unwrap_or(""),
        )
        .map(|p| json!(p.to_string_lossy()))
        .unwrap_or(Value::Null)),
        _ => Err(Error::from("unknown HTTP operation".to_string())),
    }
}

fn response(stream: &mut TcpStream, status: &str, headers: &str) -> std::io::Result<()> {
    write!(stream, "HTTP/1.1 {status}\r\n{headers}\r\n")
}

pub fn handle(mut stream: TcpStream, root: &Path) -> std::io::Result<()> {
    stream.set_nodelay(true)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        if line.len() > 8192 {
            return Ok(());
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 3 {
            return Ok(());
        }
        let (method, target, version) = (fields[0], fields[1], fields[2]);
        let mut range = None;
        let mut close = version != "HTTP/1.1";
        let mut total = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header)? == 0 {
                return Ok(());
            }
            total += header.len();
            if total > 65536 {
                return Ok(());
            }
            if header == "\r\n" || header == "\n" {
                break;
            }
            if let Some((key, value)) = header.split_once(':') {
                if key.eq_ignore_ascii_case("range") {
                    range = Some(value.trim().to_owned());
                }
                if key.eq_ignore_ascii_case("connection") {
                    if value.trim().eq_ignore_ascii_case("close") {
                        close = true;
                    }
                    if value.trim().eq_ignore_ascii_case("keep-alive") {
                        close = false;
                    }
                }
                // Request bodies cannot be interpreted as another request.
                if key.eq_ignore_ascii_case("transfer-encoding")
                    || (key.eq_ignore_ascii_case("content-length") && value.trim() != "0")
                {
                    close = true;
                }
            }
        }
        let connection = if close { "Connection: close\r\n" } else { "" };
        if method != "GET" && method != "HEAD" {
            response(
                &mut stream,
                "501 Not Implemented",
                &format!("Content-Length: 0\r\n{connection}"),
            )?;
        } else if let Some(path) = resolve(root, target) {
            let mut file = File::open(path)?;
            let size = file.metadata()?.len();
            if let Some((start, end, partial)) = span(size, range.as_deref()) {
                let len = if size == 0 { 0 } else { end - start + 1 };
                let extra = if partial {
                    format!("Content-Range: bytes {start}-{end}/{size}\r\n")
                } else {
                    String::new()
                };
                response(
                    &mut stream,
                    if partial {
                        "206 Partial Content"
                    } else {
                        "200 OK"
                    },
                    &format!(
                        "Accept-Ranges: bytes\r\nContent-Length: {len}\r\n{extra}{connection}"
                    ),
                )?;
                if method == "GET" {
                    file.seek(SeekFrom::Start(start))?;
                    std::io::copy(&mut file.take(len), &mut stream)?;
                }
            } else {
                response(
                    &mut stream,
                    "416 Range Not Satisfiable",
                    &format!("Content-Range: bytes */{size}\r\nContent-Length: 0\r\n{connection}"),
                )?;
            }
        } else {
            response(
                &mut stream,
                "404 Not Found",
                &format!("Content-Length: 0\r\n{connection}"),
            )?;
        }
        if close {
            return Ok(());
        }
    }
}

pub fn run(root: &Path, bind: &str, port: u16) -> Result<()> {
    let root = fs::canonicalize(root)?;
    let listener = TcpListener::bind((bind, port))?;
    println!("serving {} on {bind}:{port}", root.display());
    for connection in listener.incoming() {
        let stream = connection?;
        let root = root.clone();
        thread::spawn(move || {
            let _ = handle(stream, &root);
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges() {
        assert_eq!(span(10, Some("bytes=-3")), Some((7, 9, true)));
        assert_eq!(span(10, Some("bytes=10-")), None);
        assert_eq!(span(0, None), Some((0, 0, false)));
    }
    #[test]
    fn socket_keepalive_head_range() {
        let root = std::env::temp_dir().join(format!("bench-http-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("x"), b"abcdef").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let r = root.clone();
        let server = thread::spawn(move || {
            let (s, _) = listener.accept().unwrap();
            handle(s, &r).unwrap();
        });
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"HEAD /x HTTP/1.1\r\n\r\nGET /x HTTP/1.1\r\nRange: bytes=2-4\r\nConnection: close\r\n\r\n").unwrap();
        let mut reply = String::new();
        client.read_to_string(&mut reply).unwrap();
        assert!(reply.contains("Content-Length: 6"));
        assert!(reply.contains("206 Partial Content"));
        assert!(reply.ends_with("cde"));
        server.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn refuses_symlink_escape() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("bench-http-link-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        symlink("/etc/passwd", root.join("escape")).unwrap();
        assert!(resolve(&root, "/escape").is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
