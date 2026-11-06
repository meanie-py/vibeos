use std::{fs, path::PathBuf};
use vibeos_codegen::{
    payload::{apitest, winetest},
    probe,
};
pub const NAMES: &[&str] = &["apitest-suites", "winetest-suites"];

pub fn run(raw: Vec<String>) -> Result<u8, String> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut name, mut arch, mut build, mut out, mut quarantine) = (None, None, None, None, None);
    let (mut discover, mut ignore, mut check) = (false, false, false);
    let (mut only, mut skip, mut repeat, mut timeout, mut global) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut suites, mut register) = (None, None);
    let expanded = raw
        .into_iter()
        .flat_map(|arg| {
            if arg.starts_with("--") && arg.contains('=') {
                let (key, value) = arg.split_once('=').unwrap();
                vec![key.to_owned(), value.to_owned()]
            } else {
                vec![arg]
            }
        })
        .collect::<Vec<_>>();
    let mut args = expanded.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = args.next().ok_or("--root requires DIR")?.into(),
            "--arch" => arch = Some(args.next().ok_or("--arch requires ARCH")?),
            "--build-dir" => {
                build = Some(PathBuf::from(
                    args.next().ok_or("--build-dir requires DIR")?,
                ))
            }
            "--out-dir" | "--output-dir" => {
                out = Some(PathBuf::from(args.next().ok_or("--out-dir requires DIR")?))
            }
            "--quarantine-dir" => {
                quarantine = Some(PathBuf::from(
                    args.next().ok_or("--quarantine-dir requires DIR")?,
                ))
            }
            "--only" => only.push(args.next().ok_or("--only requires SUITE[:SUBTEST]")?),
            "--skip" => skip.push(args.next().ok_or("--skip requires SUITE:SUBTEST")?),
            "--repeat" => repeat.push(args.next().ok_or("--repeat requires SUITE:SUBTEST=N")?),
            "--timeout" => timeout.push(args.next().ok_or("--timeout requires SUITE=SECONDS")?),
            "--global-flag" => global.push(args.next().ok_or("--global-flag requires SUITE=HEX")?),
            "--suites" => suites = Some(args.next().ok_or("--suites requires LIST")?),
            "--register" => register = Some(args.next().ok_or("--register requires LIST")?),
            "--discover-json" => discover = true,
            "--ignore-quarantine" => ignore = true,
            "--check" => check = true,
            s if NAMES.contains(&s) && name.is_none() => name = Some(arg),
            _ => return Err(format!("unexpected suite-list argument: {arg}")),
        }
    }
    let name = name.ok_or("missing suite-list command")?;
    if let Some(a) = &arch {
        if !["i386", "amd64", "arm64"].contains(&a.as_str()) {
            return Err(format!("unknown architecture: {a}"));
        }
    }
    if name == "apitest-suites"
        && (suites.is_some()
            || register.is_some()
            || !skip.is_empty()
            || !repeat.is_empty()
            || !timeout.is_empty()
            || !global.is_empty())
    {
        return Err("Wine filtering/registration options require winetest-suites".into());
    }
    if name == "winetest-suites" && (build.is_some() || quarantine.is_some() || ignore || discover)
    {
        return Err("discovery/build/quarantine options require apitest-suites".into());
    }
    if discover && (ignore || check || !only.is_empty() || out.is_some() || quarantine.is_some()) {
        return Err("--discover-json cannot be combined with generation options".into());
    }
    let default_dir = root.join(if name == "apitest-suites" {
        "tools/payloads/apitest"
    } else {
        "tools/payloads/olewinetest"
    });
    let out = out.unwrap_or_else(|| default_dir.clone());
    let generated = if name == "apitest-suites" {
        let build = build
            .or_else(|| arch.as_ref().map(|a| root.join(format!("build-{a}"))))
            .ok_or("provide --arch or --build-dir for discovery")?;
        if build.exists() {
            probe::local(&root, &build)?;
        }
        if discover {
            println!(
                "{}",
                serde_json::to_string(&apitest::discover(&build)?).map_err(|e| e.to_string())?
            );
            return Ok(0);
        }
        let arch = arch.as_deref().ok_or("apitest-suites requires --arch")?;
        apitest::generate(&apitest::Options {
            arch,
            build_dir: &build,
            quarantine_dir: &quarantine.unwrap_or(default_dir),
            only: &only,
            ignore_quarantine: ignore,
        })
    } else {
        let arch = arch.as_deref().ok_or("winetest-suites requires --arch")?;
        let suites = suites
            .map(|s: String| {
                s.split(',')
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                winetest::DEFAULT_SUITES
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            });
        let register = register
            .map(|s: String| {
                s.split(',')
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                winetest::DEFAULT_REGISTER
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            });
        winetest::generate(&winetest::Options {
            root: &root,
            arch,
            out_dir: &out,
            suites: &suites,
            register: &register,
            only: &only,
            skip: &skip,
            global_flag: &global,
            repeat: &repeat,
            timeout: &timeout,
        })
    };
    let generated = match generated {
        Ok(g) => g,
        Err(error) => {
            eprintln!("{error}");
            return Ok(1);
        }
    };
    let arch = arch.as_deref().unwrap();
    for warning in &generated.warnings {
        eprintln!("{warning}");
    }
    let prefix = if name == "apitest-suites" {
        "apitest"
    } else {
        "winetest"
    };
    let outputs = [
        (
            out.join(format!("{prefix}-suites-{arch}.txt")),
            generated.suites,
        ),
        (
            out.join(format!("{prefix}-skip-{arch}.txt")),
            generated.skips,
        ),
    ];
    let mut stale = false;
    if !check {
        fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    }
    for (i, (path, text)) in outputs.iter().enumerate() {
        if check {
            if fs::read(path).ok().as_deref() != Some(text.as_bytes()) {
                stale = true;
                eprintln!("DRIFT: {}", path.display());
            }
        } else {
            crate::write_atomic(path, text.as_bytes(), i)?;
            // write_atomic leaves an identical file untouched, but payloads/compile judges these
            // lists stale when their generator script is NEWER (payload_build.rs stale_generator).
            // A merge that touched gen_suites.py then made a regenerated, byte-identical list stay
            // stale forever and PAYLOAD_BUILD_FAIL every build (2026-11-06). These outputs are
            // gitignored payload inputs, so stamping them now costs no rebuild elsewhere.
            fs::File::options()
                .write(true)
                .open(path)
                .and_then(|f| f.set_modified(std::time::SystemTime::now()))
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    if !check {
        println!("{}", generated.status);
    }
    Ok(u8::from(stale))
}
