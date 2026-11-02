use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
use vibeos_build::{self as b, Error, Result};
const HELP:&str="Usage: tools/build-infra [--root DIR] COMMAND [OPTIONS]\n\nCommands:\n  cargo-depgraph --manifest FILE --packages NAME...\n  retarget-depfile --source FILE --cargo-output FILE --target FILE --output FILE\n  ninja-prune BUILD TARGET... [--canon-root DIR] [--delete]\n  link-guard LLD_ARGS...\n  cell-manifest --write|--check [--arch ARCH]\n  media-manifest --write|--check|--assemble [--arch ARCH] [--cd livecd] [--out DIR] [--cells DIR]\n  sdk-recipe --write|--check|--run [--arch ARCH] [--gen DIR]\n\n--api-json reads a single compatibility request from stdin. Inspect/check modes never write.\n";
fn atomic(path: &Path, bytes: &[u8], serial: usize) -> Result<()> {
    if path.is_symlink() {
        return Err(Error::new(format!(
            "output is a symlink: {}",
            path.display()
        )));
    }
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let parent = path.parent().ok_or("output has no parent")?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(".vibeos-build-{}-{serial}.tmp", std::process::id()));
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| -> std::io::Result<()> {
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(Error::from)
}
fn output(root: &Path, p: &Path) -> Result<()> {
    if !p.starts_with(root)
        || p.components()
            .any(|x| matches!(x, std::path::Component::ParentDir))
    {
        return Err(Error::new(format!(
            "output escapes selected checkout: {}",
            p.display()
        )));
    }
    let mut a = p;
    loop {
        if a.is_symlink() {
            return Err(Error::new(format!(
                "output traverses symlink: {}",
                a.display()
            )));
        }
        if a.exists() {
            b::local(root, a)?;
        }
        if a == root {
            break;
        }
        a = a.parent().ok_or("output has no root ancestor")?;
    }
    Ok(())
}
fn edits(root: &Path, plan: &BTreeMap<PathBuf, String>) -> Result<()> {
    for p in plan.keys() {
        output(root, p)?;
    }
    for (i, (p, s)) in plan.iter().enumerate() {
        atomic(p, s.as_bytes(), i)?;
    }
    Ok(())
}
fn api(root: &Path, r: &Value) -> Result<Value> {
    let op = r["op"].as_str().ok_or("API request requires op")?;
    let kw = &r["kwargs"];
    let required = |key: &str| {
        kw.get(key)
            .cloned()
            .ok_or_else(|| Error::new(format!("missing {key}")))
    };
    match op {
        "cargo_local_closure" => Ok(serde_json::to_value(b::cargo::local_closure(
            &required("metadata")?,
            &serde_json::from_value::<Vec<String>>(required("roots")?)?,
        )?)?),
        "cargo_inspect" => b::cargo::inspect(
            &serde_json::from_value::<PathBuf>(required("manifest")?)?,
            &serde_json::from_value::<Vec<String>>(required("roots")?)?,
        ),
        "depfile_escaped_output" => Ok(json!(b::depfile::escaped_output(
            &serde_json::from_value::<String>(required("path")?)?
        )?)),
        "depfile_retarget" => Ok(json!(b::depfile::retarget(
            &serde_json::from_value::<String>(required("text")?)?,
            &serde_json::from_value::<String>(required("cargo_output")?)?,
            &serde_json::from_value::<String>(required("copied_output")?)?
        )?)),
        s if s.starts_with("cell_") || s.starts_with("media_") => {
            b::manifest_api::dispatch(root, r)
        }
        s if s.starts_with("sdk_") => b::sdk_api::dispatch(root, r),
        s if s.starts_with("ninja_") || s.starts_with("link_") => b::helper_api::dispatch(root, r),
        _ => Err(Error::new(format!("unknown build operation {op}"))),
    }
}
#[derive(Default)]
struct Options {
    mode: String,
    arches: Vec<String>,
    positionals: Vec<String>,
    values: BTreeMap<String, String>,
    delete: bool,
}
fn parse(raw: Vec<String>) -> Result<Options> {
    let mut o = Options::default();
    let mut args = raw
        .into_iter()
        .flat_map(|a| {
            if a.starts_with("--") && a.contains('=') {
                let (k, v) = a.split_once('=').unwrap();
                vec![k.to_owned(), v.to_owned()]
            } else {
                vec![a]
            }
        })
        .peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--write" | "--check" | "--assemble" | "--run" => {
                if !o.mode.is_empty() {
                    return Err("choose one operation".into());
                }
                o.mode = a;
            }
            "--delete" => o.delete = true,
            "--arch" => o.arches.extend(
                args.next()
                    .ok_or("--arch requires ARCH")?
                    .split(',')
                    .map(str::to_owned),
            ),
            "--packages" => {
                let mut n = 0;
                while args.peek().is_some_and(|a| !a.starts_with('-')) {
                    o.positionals.push(args.next().unwrap());
                    n += 1;
                }
                if n == 0 {
                    return Err("--packages requires names".into());
                }
            }
            "--manifest" | "--source" | "--cargo-output" | "--target" | "--output"
            | "--canon-root" | "--cd" | "--out" | "--cells" | "--gen" => {
                let v = args
                    .next()
                    .ok_or_else(|| Error::new(format!("{a} requires a value")))?;
                if o.values.insert(a.clone(), v).is_some() {
                    return Err(Error::new(format!("duplicate option {a}")));
                }
            }
            s if s.starts_with('-') => return Err(Error::new(format!("unknown option {a}"))),
            _ => o.positionals.push(a),
        }
    }
    Ok(o)
}
fn req<'a>(o: &'a Options, k: &str) -> Result<&'a str> {
    o.values
        .get(k)
        .map(String::as_str)
        .ok_or_else(|| Error::new(format!("{k} is required")))
}
fn run() -> Result<u8> {
    let raw: Vec<_> = std::env::args().skip(1).collect();
    let mut args = raw.into_iter();
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut command = None;
    let mut rest = Vec::new();
    let mut api_json = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => root = args.next().ok_or("--root requires DIR")?.into(),
            "--api-json" => api_json = true,
            "--help" | "-h" if command.is_none() => {
                print!("{HELP}");
                return Ok(0);
            }
            s if command.is_none() && !s.starts_with('-') => {
                command = Some(a.clone());
                if a == "link-guard" {
                    rest.extend(args);
                    break;
                }
            }
            _ => rest.push(a),
        }
    }
    let root = if api_json {
        if root.is_absolute() {
            root
        } else {
            std::env::current_dir()?.join(root)
        }
    } else {
        root.canonicalize()?
    };
    if api_json {
        if command.is_some() || !rest.is_empty() {
            return Err("--api-json requires no command/options".into());
        }
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        let v = serde_json::from_str(&text)
            .map_err(Error::from)
            .and_then(|r| api(&root, &r));
        println!(
            "{}",
            match v {
                Ok(v) => json!({"ok":v}),
                Err(e) => json!({"error":e}),
            }
        );
        return Ok(0);
    }
    let command = command.ok_or("missing build command (use --help)")?;
    if command == "link-guard" {
        return Ok(b::link_guard::execute(&root, &rest)?.clamp(0, 255) as u8);
    }
    if rest.iter().any(|s| s == "--help" || s == "-h") {
        print!("{HELP}");
        return Ok(0);
    }
    let o = parse(rest.clone())?;
    let allowed: &[&str] = match command.as_str() {
        "cargo-depgraph" => &["--manifest"],
        "retarget-depfile" => &["--source", "--cargo-output", "--target", "--output"],
        "ninja-prune" => &["--canon-root"],
        "cell-manifest" => &[],
        "media-manifest" => &["--cd", "--out", "--cells"],
        "sdk-recipe" => &["--gen"],
        _ => return Err(Error::new(format!("unknown build command {command}"))),
    };
    for key in o.values.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(Error::new(format!("{key} does not apply to {command}")));
        }
    }
    if !["cell-manifest", "media-manifest", "sdk-recipe"].contains(&command.as_str())
        && (!o.mode.is_empty() || !o.arches.is_empty())
    {
        return Err("manifest modes and architectures require a manifest/recipe command".into());
    }
    if o.delete && command != "ninja-prune" {
        return Err("--delete requires ninja-prune".into());
    }
    if command == "retarget-depfile" && !o.positionals.is_empty() {
        return Err("retarget-depfile accepts only named options".into());
    }
    if command == "media-manifest" && o.mode != "--assemble" && !o.values.is_empty() {
        return Err("--cd/--out/--cells require --assemble".into());
    }
    if command == "sdk-recipe" && o.mode != "--run" && o.values.contains_key("--gen") {
        return Err("--gen requires --run".into());
    }
    match command.as_str() {
        "cargo-depgraph" => {
            if o.positionals.is_empty() {
                return Err("--packages requires at least one package".into());
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&b::cargo::inspect(
                    Path::new(req(&o, "--manifest")?),
                    &o.positionals
                )?)?
            );
        }
        "retarget-depfile" => {
            let text = b::read(Path::new(req(&o, "--source")?))?;
            let new =
                b::depfile::retarget(&text, req(&o, "--cargo-output")?, req(&o, "--target")?)?;
            atomic(Path::new(req(&o, "--output")?), new.as_bytes(), 0)?;
        }
        "ninja-prune" => {
            if o.positionals.len() < 2 {
                return Err("ninja-prune requires BUILD and TARGET...".into());
            }
            let build = Path::new(&o.positionals[0]);
            let p = b::ninja::plan(
                build,
                &o.positionals[1..],
                o.values.get("--canon-root").map(String::as_str),
            )?;
            if !p.unknown.is_empty() {
                return Err(Error::new(format!(
                    "ninja-prune: unknown target(s): {}",
                    p.unknown.join(" ")
                )));
            }
            if o.delete {
                b::ninja::delete(build, &p)?;
            } else {
                for s in &p.victims {
                    println!("{s}")
                }
            }
            println!(
                "{} file(s) outside the closure{}.",
                p.victims.len(),
                if o.delete { " removed" } else { "" }
            );
        }
        "cell-manifest" | "media-manifest" | "sdk-recipe" => {
            if !o.positionals.is_empty() || o.delete {
                return Err(
                    "manifest/recipe command does not accept positional arguments or --delete"
                        .into(),
                );
            }
            let arches = b::sdk_recipe::selected_arches(&o.arches)?;
            if o.mode == "--assemble" {
                if command != "media-manifest" || o.arches.len() != 1 {
                    return Err("--assemble requires media-manifest and one --arch".into());
                }
                let out = PathBuf::from(req(&o, "--out")?);
                let cells = o.values.get("--cells").map(PathBuf::from);
                let cd = o.values.get("--cd").map(String::as_str).unwrap_or("livecd");
                if root != Path::new("/tmp/vibeos") {
                    let canonical = root.join("tools/canon-run.sh");
                    b::local(&root, &canonical)?;
                    let mut cmd = Command::new("bash");
                    cmd.arg(canonical)
                        .arg("/tmp/vibeos/tools/build-infra")
                        .args(["--root", "/tmp/vibeos", "media-manifest"])
                        .args(&rest)
                        .current_dir(&root);
                    let status = cmd.status()?;
                    return Ok(status.code().unwrap_or(1).clamp(0, 255) as u8);
                }
                println!(
                    "{}",
                    b::media_manifest::assemble(&arches[0], cd, &out, cells.as_deref(), &root)?
                        .display()
                );
                return Ok(0);
            }
            if o.mode == "--run" {
                if command != "sdk-recipe" || o.arches.len() != 1 {
                    return Err("--run requires sdk-recipe and one --arch".into());
                }
                let p =
                    b::sdk_recipe::replay_plan(&root, &arches[0], Path::new(req(&o, "--gen")?))?;
                b::sdk_recipe::run(&p)?;
                return Ok(0);
            }
            if o.mode != "--write" && o.mode != "--check" {
                return Err("choose --write or --check".into());
            }
            let plan = match command.as_str() {
                "cell-manifest" => b::cell_manifest::write_plan(&root, &arches)?,
                "media-manifest" => b::media_manifest::write_plan(&root, &arches)?,
                _ => b::sdk_recipe::write_plan(&root, &arches)?
                    .into_iter()
                    .map(|r| (r.path, r.text))
                    .collect(),
            };
            if o.mode == "--write" {
                edits(&root, &plan)?;
                for p in plan.keys() {
                    println!("wrote {}", p.strip_prefix(&root).unwrap_or(p).display())
                }
            } else {
                let mut stale = false;
                for (p, text) in plan {
                    if fs::read(&p).ok().as_deref() != Some(text.as_bytes()) {
                        eprintln!(
                            "{}: stale (tools/build-infra {command} --write)",
                            p.strip_prefix(&root).unwrap_or(&p).display()
                        );
                        stale = true
                    }
                }
                return Ok(u8::from(stale));
            }
        }
        _ => return Err(Error::new(format!("unknown build command {command}"))),
    }
    Ok(0)
}
fn main() -> ExitCode {
    match run() {
        Ok(n) => n.into(),
        Err(e) => {
            eprintln!("build-infra: {e}");
            2.into()
        }
    }
}
