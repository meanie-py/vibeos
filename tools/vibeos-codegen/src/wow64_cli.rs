use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::PathBuf,
};
use vibeos_codegen::wow64::{self, Syscall};
pub const NAMES: &[&str] = &["wow64-thunks"];

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}"))
}
fn decode<T: serde::de::DeserializeOwned>(v: &Value, key: &str) -> Result<T, String> {
    serde_json::from_value(
        v.get(key)
            .cloned()
            .ok_or_else(|| format!("missing field {key}"))?,
    )
    .map_err(|e| e.to_string())
}
fn api() -> Result<u8, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let output = match field(&v, "op")? {
        "arg_kinds" => json!(wow64::arg_kinds()),
        "parse_spec" => {
            let manual: HashSet<String> = decode(&v, "manual")?;
            json!(wow64::read_spec(
                &PathBuf::from(field(&v, "path")?),
                field(&v, "arch")?,
                &manual
            )?)
        }
        "parse_sysfuncs" => json!(wow64::read_sysfuncs(&PathBuf::from(field(&v, "path")?))?),
        "load_manual_list" => {
            let paths = field(&v, "paths")?;
            let paths = if paths.is_empty() {
                vec![]
            } else {
                paths.split(',').map(PathBuf::from).collect()
            };
            let mut names = wow64::load_manual_lists(&paths)?
                .into_iter()
                .collect::<Vec<_>>();
            names.sort();
            json!(names)
        }
        "order" => {
            let sys: Vec<(String, usize)> = decode(&v, "sysfuncs")?;
            let sc: Vec<Syscall> = decode(&v, "syscalls")?;
            json!(wow64::order_by_sysfuncs(&sys, &sc))
        }
        "classify" => {
            let mut sc: Syscall = decode(&v, "syscall")?;
            wow64::classify(&mut sc);
            json!(sc)
        }
        "emit" => {
            let sc: Vec<Syscall> = decode(&v, "syscalls")?;
            json!(wow64::emit_c(
                &sc,
                field(&v, "spec")?,
                field(&v, "arch")?,
                field(&v, "out")?,
                field(&v, "symbol_prefix")?
            )?)
        }
        _ => return Err("unknown wow64 API operation".into()),
    };
    println!("{output}");
    Ok(0)
}

pub fn run(raw: Vec<String>) -> Result<u8, String> {
    let expanded = raw
        .into_iter()
        .flat_map(|arg| {
            if arg.starts_with("--") && arg.contains('=') {
                let (k, v) = arg.split_once('=').unwrap();
                vec![k.to_owned(), v.to_owned()]
            } else {
                vec![arg]
            }
        })
        .collect::<Vec<_>>();
    let (mut spec, mut services, mut arch, mut out, mut manual, mut prefix) =
        (None, None, None, None, None, "Wow64Thunk".to_owned());
    let (mut check, mut stats, mut api_json) = (false, false, false);
    let mut args = expanded.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                args.next().ok_or("--root requires DIR")?;
            }
            "wow64-thunks" => {}
            "--spec" => spec = Some(args.next().ok_or("--spec requires FILE")?),
            "--sysfuncs" => services = Some(args.next().ok_or("--sysfuncs requires FILE")?),
            "--arch" => arch = Some(args.next().ok_or("--arch requires ARCH")?),
            "--out-c" | "--output" => out = Some(args.next().ok_or("--out-c requires FILE")?),
            "--manual-list" => manual = Some(args.next().ok_or("--manual-list requires FILES")?),
            "--symbol-prefix" => prefix = args.next().ok_or("--symbol-prefix requires NAME")?,
            "--check" => check = true,
            "--stats" => stats = true,
            "--api-json" => api_json = true,
            _ => return Err(format!("unexpected wow64 argument: {arg}")),
        }
    }
    if api_json {
        if spec.is_some()
            || services.is_some()
            || arch.is_some()
            || out.is_some()
            || manual.is_some()
            || check
            || stats
            || prefix != "Wow64Thunk"
        {
            return Err("--api-json cannot be combined with generation options".into());
        }
        return api();
    }
    let spec = spec.ok_or("wow64-thunks requires --spec")?;
    let services = services.ok_or("wow64-thunks requires --sysfuncs")?;
    let arch = arch.ok_or("wow64-thunks requires --arch")?;
    if !["i386", "amd64", "arm64"].contains(&arch.as_str()) {
        return Err(format!("unknown architecture: {arch}"));
    }
    if check && out.is_none() {
        return Err("--check requires --out-c".into());
    }
    let paths = manual
        .map(|s: String| {
            if s.is_empty() {
                vec![]
            } else {
                s.split(',').map(PathBuf::from).collect::<Vec<_>>()
            }
        })
        .unwrap_or_default();
    let manual = wow64::load_manual_lists(&paths)?;
    let specs = wow64::read_spec(&PathBuf::from(&spec), &arch, &manual)?;
    if specs.is_empty() {
        return Err(format!("wow64gen: no Nt* syscalls found in {spec}"));
    }
    let sys = wow64::read_sysfuncs(&PathBuf::from(&services))?;
    if sys.is_empty() {
        return Err(format!("wow64gen: no SVC_ entries found in {services}"));
    }
    let rows = wow64::order_by_sysfuncs(&sys, &specs);
    let text = wow64::emit_c(&rows, &spec, &arch, out.as_deref().unwrap_or("-"), &prefix)?;
    if let Some(out) = &out {
        let path = PathBuf::from(out);
        if check {
            if fs::read(&path).ok().as_deref() != Some(text.as_bytes()) {
                eprintln!("DRIFT: {}", path.display());
                return Ok(1);
            }
        } else {
            crate::write_atomic(&path, text.as_bytes(), 0)?;
        }
    } else {
        std::io::stdout()
            .write_all(text.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    if stats || out.is_some() {
        let manual = rows.iter().filter(|s| s.needs_manual()).count();
        eprintln!(
            "wow64gen: {} syscalls ({} auto, {} manual) for arch={arch}",
            rows.len(),
            rows.len() - manual,
            manual
        );
    }
    Ok(0)
}
