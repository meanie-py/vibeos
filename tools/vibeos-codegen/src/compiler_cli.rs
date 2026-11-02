use std::{fs, io::Write, path::PathBuf};
use vibeos_codegen::{crt_probes, ctype, probe, ucrt_cases};
pub const NAMES: &[&str] = &["ctype-tables", "msvcrt-consts", "seh-layout", "ucrt-cases"];

pub fn run(raw: Vec<String>) -> Result<u8, String> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut name, mut output, mut dir, mut arch, mut cache) = (None, None, None, None, None);
    let (mut check, mut stdout, mut stats, mut reverse, mut ast_json, mut recipe_json) =
        (false, false, false, false, false, false);
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
    let mut args = expanded.into_iter().peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = args.next().ok_or("--root requires DIR")?.into(),
            "--output" | "--out" | "-o" => {
                output = Some(PathBuf::from(args.next().ok_or("--output requires FILE")?))
            }
            "--output-dir" => {
                dir = Some(PathBuf::from(
                    args.next().ok_or("--output-dir requires DIR")?,
                ))
            }
            "--arch" => arch = Some(args.next().ok_or("--arch requires ARCH")?),
            "--cache" => cache = Some(PathBuf::from(args.next().ok_or("--cache requires FILE")?)),
            "--check" => {
                check = true;
                if args
                    .peek()
                    .is_some_and(|s| !s.starts_with('-') && !NAMES.contains(&s.as_str()))
                {
                    output = Some(PathBuf::from(args.next().unwrap()))
                }
            }
            "--emit" => {
                if args.peek().is_some_and(|s| s == "-" || !s.starts_with('-')) {
                    let p = args.next().unwrap();
                    if p != "-" {
                        output = Some(p.into())
                    }
                } else {
                    stdout = true
                }
            }
            "--stdout" => stdout = true,
            "--stats" => stats = true,
            "--reverse" => reverse = true,
            "--ast-json" => ast_json = true,
            "--recipe-json" => recipe_json = true,
            s if NAMES.contains(&s) && name.is_none() => name = Some(arg),
            _ => return Err(format!("unexpected compiler generator argument: {arg}")),
        }
    }
    let name = name.ok_or("missing compiler generator")?;
    if check && stdout {
        return Err("--check and --stdout are mutually exclusive".into());
    }
    if name != "ucrt-cases" && (arch.is_some() || cache.is_some() || stats || reverse || ast_json) {
        return Err("architecture/cache/stats/reverse/AST options require ucrt-cases".into());
    }
    if name == "ucrt-cases" && dir.is_some() {
        return Err("ucrt-cases uses --out FILE, not --output-dir".into());
    }
    if (recipe_json || ast_json)
        && (check || stdout || output.is_some() || dir.is_some() || stats || reverse)
    {
        return Err(
            "compiler metadata modes cannot be combined with check/write/stats/reverse modes"
                .into(),
        );
    }
    if recipe_json {
        if !matches!(name.as_str(), "msvcrt-consts" | "seh-layout") {
            return Err("--recipe-json requires a CRT compiler probe".into());
        }
        let r = crt_probes::recipe(&name)?;
        println!(
            "{}",
            serde_json::json!({"arch":r.arch,"includes":r.includes,"include_dirs":r.include_dirs,"defines":r.defines,"expressions":r.expressions})
        );
        return Ok(0);
    }
    let outputs: Vec<(PathBuf, String)> = match name.as_str() {
        "ctype-tables" => {
            if dir.is_some() {
                return Err("ctype-tables uses --output FILE".into());
            }
            let generated = ctype::generate(&root)?;
            if generated.proof {
                eprintln!("ctype tables: 514/514 values proven against this checkout's object")
            }
            let path = output
                .clone()
                .unwrap_or_else(|| root.join("sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs"));
            if stdout || (!check && output.is_none()) {
                print!("{}\n{}{}\n", ctype::BEGIN, generated.block, ctype::END);
                return Ok(0);
            }
            let old = vibeos_codegen::read(&path)?;
            let marked = format!("{}\n{}{}", ctype::BEGIN, generated.block, ctype::END);
            let updated = vibeos_codegen::replace_block(&old, ctype::BEGIN, ctype::END, &marked)?;
            vec![(path, updated)]
        }
        "msvcrt-consts" | "seh-layout" => {
            let values = probe::harvest(&root, &crt_probes::recipe(&name)?)?;
            let rendered = crt_probes::render(&name, &values)?;
            if output.is_some() && rendered.len() != 1 {
                return Err("--output requires a single generated header; use --output-dir".into());
            }
            rendered
                .into_iter()
                .map(|(p, t)| {
                    let p = PathBuf::from(p);
                    let dest = output.clone().unwrap_or_else(|| {
                        dir.as_ref()
                            .map(|d| d.join(p.file_name().unwrap()))
                            .unwrap_or_else(|| root.join(p))
                    });
                    (dest, t)
                })
                .collect()
        }
        "ucrt-cases" => {
            let arch = arch.ok_or("ucrt-cases requires --arch")?;
            if ast_json {
                let functions: std::collections::BTreeMap<_, _> =
                    ucrt_cases::ast_functions(&root, &arch, cache.as_deref())?
                        .into_iter()
                        .collect();
                println!(
                    "{}",
                    serde_json::to_value(functions).map_err(|e| e.to_string())?
                );
                return Ok(0);
            }
            let mut selection = ucrt_cases::select(&root, &arch, cache.as_deref())?;
            if reverse {
                selection.rows.reverse()
            }
            if let Some(report) = ucrt_cases::quarantine_report(&selection) {
                eprintln!("{report}")
            }
            if selection.rows.is_empty() {
                eprintln!("gen-ucrt-tier2-cases ({arch}): no generator-drivable exports left")
            }
            if stats {
                print!("{}", ucrt_cases::stats(&arch, &selection.rows));
                return Ok(0);
            }
            let cases = ucrt_cases::render_rows(&selection.rows);
            if stdout {
                print!("{cases}");
                return Ok(0);
            }
            let out = output.unwrap_or_else(|| {
                root.join(format!("tools/payloads/ucrtdiff/generated-{arch}.inc"))
            });
            let refs = ucrt_cases::render_refs(&root, &arch)?;
            vec![
                (out.clone(), cases),
                (
                    out.parent()
                        .ok_or("output has no parent")?
                        .join("generated-refs.h"),
                    refs,
                ),
            ]
        }
        _ => unreachable!(),
    };
    if stdout {
        if outputs.len() != 1 {
            return Err("--stdout requires one output".into());
        }
        std::io::stdout()
            .write_all(outputs[0].1.as_bytes())
            .map_err(|e| e.to_string())?;
        return Ok(0);
    }
    let mut stale = false;
    for (i, (path, text)) in outputs.iter().enumerate() {
        if check {
            if fs::read(path).ok().as_deref() != Some(text.as_bytes()) {
                eprintln!("DRIFT: {}", path.display());
                stale = true
            }
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?
            }
            crate::write_atomic(path, text.as_bytes(), i)?
        }
    }
    if !stale {
        eprintln!(
            "OK: {} compiler-derived outputs {}",
            outputs.len(),
            if check {
                "match"
            } else {
                "written or unchanged"
            }
        )
    }
    Ok(u8::from(stale))
}
