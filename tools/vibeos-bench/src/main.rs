use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use vibeos_bench::{Error, Result};
fn dispatch(_root: &Path, r: &Value) -> Result<Value> {
    let op = r["op"].as_str().ok_or("bench request needs op")?;
    let mut a = r.get("kwargs").cloned().unwrap_or(json!({}));
    if let Some(tail) = op.strip_prefix("bench_http_") {
        a["op"] = json!(tail);
        return vibeos_bench::bench_http::dispatch(&a);
    }
    if let Some(tail) = op.strip_prefix("bench_tftp_") {
        a["op"] = json!(tail);
        return vibeos_bench::bench_tftp::dispatch(&a);
    }
    if op.starts_with("bench_serial_times_") {
        a["op"] = json!(op);
        return vibeos_bench::serial_times::dispatch(&a);
    }
    #[cfg(unix)]
    if op.starts_with("bench_lease_") {
        return vibeos_bench::lease::dispatch(_root, r);
    }
    Err(Error::new(format!("unknown bench operation {op}")))
}
fn run() -> Result<u8> {
    let mut args = std::env::args().skip(1).peekable();
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut lines = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = PathBuf::from(args.next().ok_or("--root needs a directory")?),
            "--client-pid" => {
                let _ = args.next().ok_or("--client-pid needs PID")?;
            }
            "--api-jsonl" | "--resource-jsonl" => {
                lines = true;
                break;
            }
            "--api-json" => break,
            "http" | "tftp" => {
                let directory = PathBuf::from(args.next().ok_or("service needs root")?);
                let bind = args.next().unwrap_or("0.0.0.0".into());
                let port = args
                    .next()
                    .map(|s| {
                        s.parse::<u16>()
                            .map_err(|_| Error::new("invalid service port"))
                    })
                    .transpose()?
                    .unwrap_or(if arg == "http" { 8069 } else { 69 });
                if args.next().is_some() {
                    return Err("unexpected service argument".into());
                }
                if arg == "http" {
                    vibeos_bench::bench_http::run(&directory, &bind, port)?;
                } else {
                    vibeos_bench::bench_tftp::run(&directory, &bind, port)?;
                }
                return Ok(0);
            }
            "serial" => {
                let directory = PathBuf::from(args.next().ok_or("serial needs bench directory")?);
                let host = args.next().unwrap_or("picousb".into());
                if args.next().is_some() {
                    return Err("unexpected serial argument".into());
                }
                std::fs::create_dir_all(&directory)?;
                vibeos_bench::serial::run_serial(&directory.canonicalize()?, &host)?;
                return Ok(0);
            }
            "--help" | "-h" => {
                println!("Usage: bench-infra http ROOT [BIND [PORT]] | tftp ROOT [BIND [PORT]] | serial BENCH [HOST]\n       bench-infra [--root DIR] --api-json|--api-jsonl");
                return Ok(0);
            }
            _ => return Err(Error::new(format!("unknown bench option {arg}"))),
        }
    }
    let input = std::io::stdin();
    let output = std::io::stdout();
    let mut output = output.lock();
    for line in input.lock().lines() {
        let result = match serde_json::from_str::<Value>(&line?) {
            Ok(r) => dispatch(&root, &r),
            Err(e) => Err(e.into()),
        };
        let envelope = match result {
            Ok(v) => json!({"ok":v}),
            Err(e) => json!({"error":e}),
        };
        serde_json::to_writer(&mut output, &envelope)?;
        writeln!(output)?;
        output.flush()?;
        if !lines {
            break;
        }
    }
    Ok(0)
}
fn main() -> ExitCode {
    match run() {
        Ok(n) => ExitCode::from(n),
        Err(e) => {
            eprintln!("bench-infra: {e}");
            ExitCode::from(2)
        }
    }
}
