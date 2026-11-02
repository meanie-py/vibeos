//! ABI/module generators retain explicit selection and legacy stdout defaults.
use std::{fs, io::Write, path::PathBuf};
use vibeos_codegen::{cnt, cohab_registry, global_barrier};
pub const NAMES: &[&str] = &["cnt", "cohab-registry", "global-barrier"];

pub fn run(args: Vec<String>) -> Result<u8, String> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut name, mut output, mut output_dir, mut source_dir) = (None, None, None, None);
    let (mut check, mut stdout, mut report) = (false, false, false);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = args.next().ok_or("--root requires a directory")?.into(),
            "--output" | "-o" => {
                output = Some(PathBuf::from(
                    args.next().ok_or("--output requires a file")?,
                ))
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(
                    args.next().ok_or("--output-dir requires a directory")?,
                ))
            }
            "--source-dir" => {
                source_dir = Some(PathBuf::from(
                    args.next().ok_or("--source-dir requires a directory")?,
                ))
            }
            "--check" => check = true,
            "--stdout" => stdout = true,
            "--report" => report = true,
            s if NAMES.contains(&s) && name.is_none() => name = Some(arg),
            _ => return Err(format!("unexpected module generator argument: {arg}")),
        }
    }
    let name = name.ok_or("missing module generator")?;
    if check && stdout {
        return Err("--check and --stdout are mutually exclusive".into());
    }
    if source_dir.is_some() && name != "global-barrier" {
        return Err("--source-dir requires global-barrier".into());
    }
    if output_dir.is_some() && name != "cnt" {
        return Err("--output-dir requires cnt".into());
    }
    if report && (name != "cohab-registry" || check || stdout || output.is_some()) {
        return Err("--report requires cohab-registry and no output/check mode".into());
    }
    let outputs: Vec<(PathBuf, String)> = match name.as_str() {
        "cnt" => {
            if stdout || output.is_some() {
                return Err("cnt has three outputs; use --output-dir or --check".into());
            }
            cnt::generate(&root)?
                .into_iter()
                .map(|(p, t)| {
                    let p = output_dir
                        .as_ref()
                        .map(|d| d.join(p.file_name().unwrap()))
                        .unwrap_or(p);
                    (p, t)
                })
                .collect()
        }
        "global-barrier" => {
            let dir = source_dir.unwrap_or_else(|| root.join("sdk/lib/rust/ntoskrnl-rust/src"));
            let globals = global_barrier::find_globals(&dir)?;
            eprintln!("Found {} globals", globals.len());
            vec![(
                output
                    .clone()
                    .unwrap_or_else(|| dir.join("global_barrier.rs")),
                global_barrier::render(&globals),
            )]
        }
        "cohab-registry" => {
            let scan = cohab_registry::scan(&root)?;
            eprintln!("{}", cohab_registry::report(&scan));
            if report {
                return Ok(0);
            }
            vec![(
                output
                    .clone()
                    .unwrap_or_else(|| root.join("tools/cohab-registry.json")),
                cohab_registry::render(&scan)?,
            )]
        }
        _ => unreachable!(),
    };
    if name != "cnt" && (stdout || (!check && output.is_none())) {
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
                fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?
            }
            crate::write_atomic(path, text.as_bytes(), i)?
        }
    }
    if !stale {
        eprintln!(
            "OK: {} generated outputs {}",
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
