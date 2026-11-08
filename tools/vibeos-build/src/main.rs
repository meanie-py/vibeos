use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
use vibeos_build::{self as b, Error, Result};
const HELP:&str="Usage: tools/build-infra [--root DIR] COMMAND [OPTIONS]\n\nCommands:\n  build-bench --arches ARCH... [--clean] [--repeat N] [--output FILE]\n  cargo-orchestrate --arch ARCH [--manifest] [--list] [--dry-run] [TARGET...]\n  ramdisk-tree LIST... --out DIR [--canon FROM TO]\n  strip-cd-media --lst FILE --out FILE --cache DIR --objcopy TOOL\n  artifact-executor [--repo DIR --store DIR --toolchain DIR --body NAME --output DIR]\n  artifact-runtime request|repair --supervisor FILE [OPTIONS]\n  artifact-economics INVENTORY REPO OUTPUT [--eligibility-top N --scratch DIR] [--cargo PATH]\n  cargo-depgraph --manifest FILE --packages NAME...\n  retarget-depfile --source FILE --cargo-output FILE --target FILE --output FILE\n    [--dependency-source FILE --dependency-snapshot FILE]\n  ninja-prune BUILD TARGET... [--canon-root DIR] [--delete]\n  link-guard LLD_ARGS...\n  cell-manifest --write|--check [--arch ARCH]\n  media-manifest --write|--check|--assemble [--arch ARCH] [--cd livecd] [--out DIR] [--cells DIR] [--allow-drop]\n  sdk-recipe --write|--check|--run [--arch ARCH] [--gen DIR]\n\n--api-json reads one request; --api-jsonl serves newline-delimited requests until EOF. Inspect/check modes never write.\n";
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
        s if s.starts_with("payload_firmware_") => b::payload_firmware::dispatch(root, r),
        s if s.starts_with("payload_build_") => b::payload_build::dispatch(root, r),
        s if s.starts_with("payload_pe_") => b::payload_pe::dispatch(root, r),
        s if s.starts_with("payload_providers_") => b::payload_providers::dispatch(root, r),
        s if s.starts_with("payload_closure_") => {
            let mut request = kw.clone();
            request["op"] = json!(s.trim_start_matches("payload_closure_"));
            b::payload_closure::dispatch(root, &request)
        }
        s if s.starts_with("tooling_receipts_") => b::tooling_receipts::dispatch(root, r),
        s if s.starts_with("cli_canon_") => b::cli_canon::dispatch(root, r),
        s if s.starts_with("cli_build_check_") => b::cli_build_checks::dispatch(root, r),
        s if s.starts_with("cli_arguments_") => b::cli_arguments::dispatch(r),
        s if s.starts_with("cli_artifacts_") => b::cli_artifacts::dispatch(root, r),
        s if s.starts_with("cargo_orchestrate_") => b::cargo_orchestrate::dispatch(root, r),
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
        "depfile_check_compile_snapshot" => {
            b::depfile::check_compile_snapshot(&serde_json::from_value::<PathBuf>(required(
                "build",
            )?)?)?;
            Ok(Value::Null)
        }
        s if s.starts_with("cell_") || s.starts_with("media_") => {
            b::manifest_api::dispatch(root, r)
        }
        s if s.starts_with("artifact_") => b::artifact_api::dispatch(root, r),
        s if s.starts_with("build_bench_") => b::build_bench::dispatch(root, r),
        s if s.starts_with("build_policy_") => b::build_policy::dispatch(r),
        s if s.starts_with("trim_") => b::trim::dispatch(r),
        s if s.starts_with("prune_") => b::prune::dispatch(r),
        s if s.starts_with("disk_") => b::diskspace::dispatch(r),
        s if s.starts_with("warmth_") => b::warmth::dispatch(r),
        s if s.starts_with("repro_") => b::repro::dispatch(r),
        s if s.starts_with("provenance_") => b::provenance::dispatch(root, r),
        s if s.starts_with("stage_") => b::media_stage::api(r),
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
    allow_drop: bool,
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
            "--allow-drop" => o.allow_drop = true,
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
            "--manifest"
            | "--source"
            | "--cargo-output"
            | "--target"
            | "--output"
            | "--dependency-source"
            | "--dependency-snapshot"
            | "--canon-root"
            | "--cd"
            | "--out"
            | "--cells"
            | "--gen" => {
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
    let mut api_lines = false;
    let mut client_pid: Option<u32> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--client-pid" => {
                client_pid = Some(
                    args.next()
                        .ok_or("--client-pid requires PID")?
                        .parse()
                        .map_err(|_| Error::new("invalid native client PID"))?,
                )
            }
            "--root" => root = args.next().ok_or("--root requires DIR")?.into(),
            "--api-json" => api_json = true,
            "--api-jsonl" => {
                api_json = true;
                api_lines = true;
            }
            "--help" | "-h" if command.is_none() => {
                print!("{HELP}");
                return Ok(0);
            }
            s if command.is_none() && !s.starts_with('-') => {
                command = Some(a.clone());
                if matches!(
                    a.as_str(),
                    "link-guard"
                        | "owned-command-fd"
                        | "cargo-orchestrate-worker"
                        | "build-bench-worker"
                ) {
                    rest.extend(args);
                    break;
                }
            }
            _ => rest.push(a),
        }
    }
    if let Some(pid) = client_pid {
        if unsafe { libc::getppid() } != pid as libc::pid_t {
            return Err("native helper client is no longer its parent".into());
        }
        b::artifact_lifetime::watch_owner(pid)?;
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
    b::artifact_executor::set_implementation_repo(&root);
    if api_json {
        if command.is_some() || !rest.is_empty() {
            return Err("--api-json requires no command/options".into());
        }
        if api_lines {
            let stdin = std::io::stdin();
            let mut stdout = std::io::stdout().lock();
            for line in stdin.lock().lines() {
                let result = serde_json::from_str(&line?)
                    .map_err(Error::from)
                    .and_then(|r| api(&root, &r));
                let response = match result {
                    Ok(v) => json!({"ok":v}),
                    Err(e) => json!({"error":e}),
                };
                writeln!(stdout, "{response}")?;
                stdout.flush()?;
            }
            return Ok(0);
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
    if matches!(
        command.as_str(),
        "ramdisk-tree" | "strip-cd-media" | "cargo-orchestrate" | "build-bench"
    ) {
        let parent = unsafe { libc::getppid() } as u32;
        if parent < 2 {
            return Err("native media owner exited before startup".into());
        }
        b::artifact_lifetime::watch_owner(parent)?;
    }
    if command == "cli-command-worker" {
        return b::cli_artifacts::worker(&rest);
    }
    if command == "owned-command-fd" {
        return b::process_guard::runner(&rest);
    }
    if command == "build-bench" {
        return b::build_bench::cli(&root, &rest);
    }
    if command == "build-bench-worker" {
        return b::build_bench::worker_cli(&rest);
    }
    if command == "cargo-orchestrate" {
        return b::cargo_orchestrate::cli(&root, &rest);
    }
    if command == "cargo-orchestrate-worker" {
        return b::cargo_orchestrate::worker_cli(&rest);
    }
    if command == "ramdisk-tree" {
        return b::media_stage::ramdisk_cli(&rest);
    }
    if command == "strip-cd-media" {
        return b::media_stage::strip_cli(&root, &rest);
    }
    if command == "artifact-executor" {
        return b::artifact_executor::cli(&rest);
    }
    if command == "artifact-runtime" {
        let parent = unsafe { libc::getppid() } as u32;
        if parent < 2 {
            return Err("artifact runtime client exited before startup".into());
        }
        b::artifact_lifetime::watch_owner(parent)?;
        return b::artifact_runtime::cli(&rest);
    }
    if command == "artifact-supervisor-service" {
        return b::artifact_execution_api::serve();
    }
    if command == "artifact-store-service" {
        return b::artifact_store_api::serve();
    }
    if command == "artifact-store-copy" {
        if rest.len() != 2 {
            return Err("artifact-store-copy requires source and destination fd".into());
        }
        return b::artifact_store_api::copy_fds(
            rest[0]
                .parse()
                .map_err(|_| Error::new("invalid source fd"))?,
            rest[1]
                .parse()
                .map_err(|_| Error::new("invalid destination fd"))?,
        );
    }
    if command == "artifact-economics" {
        return b::artifact_economics::cli(&rest);
    }
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
        "retarget-depfile" => &[
            "--source",
            "--cargo-output",
            "--target",
            "--output",
            "--dependency-source",
            "--dependency-snapshot",
        ],
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
    if o.allow_drop && (command != "media-manifest" || o.mode != "--write") {
        return Err("--allow-drop requires media-manifest --write".into());
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
            let mut new =
                b::depfile::retarget(&text, req(&o, "--cargo-output")?, req(&o, "--target")?)?;
            match (
                o.values.get("--dependency-source"),
                o.values.get("--dependency-snapshot"),
            ) {
                (None, None) => {}
                (Some(source), Some(snapshot)) => {
                    new = b::depfile::snapshot_dependency(
                        &new,
                        Path::new(source),
                        Path::new(snapshot),
                    )?;
                }
                _ => {
                    return Err(
                        "--dependency-source and --dependency-snapshot must be passed together"
                            .into(),
                    )
                }
            }
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
                "media-manifest" if o.mode == "--write" => {
                    b::media_manifest::write_plan_with_options(&root, &arches, o.allow_drop)?
                }
                "media-manifest" => b::media_manifest::check_plan(&root, &arches)?,
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
