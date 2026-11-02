//! Cargo-cell orchestration planning. Execution is deliberately separate from planning.
use crate::{cell_manifest, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cell {
    pub target: String,
    pub crate_name: String,
    pub output: String,
    pub kind: String,
    pub deps: Vec<String>,
    pub arches: Vec<String>,
}

/// Resolve dependencies before their consumers and refuse unknown nodes and cycles.
pub fn topo(cells: &BTreeMap<String, Cell>, targets: &[String]) -> Result<Vec<Cell>> {
    fn visit(
        name: &str,
        cells: &BTreeMap<String, Cell>,
        seen: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
        out: &mut Vec<Cell>,
    ) -> Result<()> {
        if seen.contains(name) {
            return Ok(());
        }
        if stack.iter().any(|x| x == name) {
            let mut cycle = stack.clone();
            cycle.push(name.to_owned());
            return Err(Error::new(format!(
                "dependency cycle: {}",
                cycle.join(" -> ")
            )));
        }
        let c = cells.get(name).ok_or_else(|| {
            Error::new(format!(
                "unknown cell {name:?}; known: {}",
                cells.keys().cloned().collect::<Vec<_>>().join(", ")
            ))
        })?;
        stack.push(name.to_owned());
        for dep in &c.deps {
            visit(dep, cells, seen, stack, out)?;
        }
        stack.pop();
        seen.insert(name.to_owned());
        out.push(c.clone());
        Ok(())
    }
    let mut seen = BTreeSet::new();
    let mut stack = Vec::new();
    let mut out = Vec::new();
    for target in targets {
        visit(target, cells, &mut seen, &mut stack, &mut out)?;
    }
    Ok(out)
}
fn stem(arch: &str) -> Result<&'static str> {
    match arch {
        "i386" => Ok("i686-kernel"),
        "amd64" => Ok("x86_64-pc-windows-gnu"),
        "arm64" => Ok("aarch64-pc-windows-gnullvm"),
        _ => Err(Error::new(format!("unsupported architecture: {arch}"))),
    }
}
pub fn target_json(root: &Path, cell: &Cell, arch: &str) -> Result<PathBuf> {
    Ok(root
        .join("sdk/lib/rust")
        .join(&cell.crate_name)
        .join(format!("{}.json", stem(arch)?)))
}
pub fn build_std(root: &Path, cell: &Cell, arch: &str) -> Result<bool> {
    crate::arch(arch)?;
    Ok(arch == "i386"
        && !["win32dll", "win32exe"].contains(&cell.kind.as_str())
        && target_json(root, cell, arch)?.exists())
}
pub fn cell_stem(root: &Path, cell: &Cell, arch: &str) -> Result<&'static str> {
    if arch == "i386" && !build_std(root, cell, arch)? {
        Ok("i686-pc-windows-gnu")
    } else {
        stem(arch)
    }
}
pub fn cargo_target_dir(root: &Path, cell: &Cell, arch: &str) -> Result<PathBuf> {
    let dir = root.join("sdk/lib/rust/target");
    if !build_std(root, cell, arch)? {
        return Ok(dir);
    }
    let digest = format!(
        "{:x}",
        Sha256::digest(fs::read(target_json(root, cell, arch)?)?)
    );
    Ok(dir.join(format!("orchestrate-{arch}-{}", &digest[..12])))
}
pub fn built_exe(root: &Path, cell: &Cell, arch: &str) -> Result<PathBuf> {
    let bin = cell
        .crate_name
        .strip_suffix("-cargo")
        .unwrap_or(&cell.crate_name);
    Ok(cargo_target_dir(root, cell, arch)?
        .join(cell_stem(root, cell, arch)?)
        .join("release")
        .join(format!("{bin}.exe")))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputPlan {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub post: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestPlan {
    pub crate_name: String,
    pub argv: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub outputs: Vec<OutputPlan>,
    pub log: PathBuf,
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::new(format!("manifest: missing or invalid {key}")))
}
fn safe_rel(s: &str) -> Result<&Path> {
    let p = Path::new(s);
    if s.is_empty()
        || p.components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(Error::new(format!("unsafe manifest path: {s:?}")));
    }
    Ok(p)
}
/// Construct the exact per-cell Cargo command without starting a producer.
pub fn manifest_plan(
    root: &Path,
    arch: &str,
    out: &Path,
    gen: &Path,
    crates: Option<&[String]>,
    force_off: bool,
) -> Result<Vec<ManifestPlan>> {
    crate::arch(arch)?;
    let cells = cell_manifest::current(arch, root)?;
    if let Some(names) = crates {
        let known: BTreeSet<&str> = cells
            .iter()
            .map(|c| string(c, "crate"))
            .collect::<Result<_>>()?;
        let unknown: BTreeSet<_> = names
            .iter()
            .filter(|n| !known.contains(n.as_str()))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(Error::new(format!(
                "not in the {arch} cell manifest: {}",
                unknown.into_iter().collect::<Vec<_>>().join(", ")
            )));
        }
    }
    let ws = root.join("sdk/lib/rust");
    let mut result = Vec::new();
    for c in cells {
        let name = string(&c, "crate")?;
        if crates.is_some_and(|names| !names.iter().any(|n| n == name)) {
            continue;
        }
        safe_rel(name)?;
        let target = string(&c, "target")?;
        let target_path = if target.ends_with(".json") {
            root.join(safe_rel(target)?).to_string_lossy().into_owned()
        } else {
            target.to_owned()
        };
        let target_stem = if target.ends_with(".json") {
            Path::new(target)
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| Error::new("invalid target"))?
        } else {
            target
        };
        safe_rel(target_stem)?;
        let target_dir = ws.join("target").join(safe_rel(string(&c, "target_dir")?)?);
        let mut argv = vec![
            "cargo".into(),
            "build".into(),
            "--release".into(),
            "--target".into(),
            target_path,
        ];
        match c.get("build_std").and_then(Value::as_bool) {
            Some(true) => argv.extend(
                [
                    "-Z",
                    "build-std=core,alloc,compiler_builtins",
                    "-Z",
                    "build-std-features=compiler-builtins-mem",
                ]
                .map(String::from),
            ),
            Some(false) => (),
            None => return Err(Error::new("manifest: invalid build_std")),
        }
        argv.extend(["-p".into(), name.into()]);
        if c["bins"] == "all" {
            argv.push("--bins".into());
        } else {
            for bin in c["bins"]
                .as_array()
                .ok_or_else(|| Error::new("manifest: invalid bins"))?
            {
                argv.push("--bin".into());
                argv.push(
                    bin.as_str()
                        .ok_or_else(|| Error::new("manifest: invalid bin"))?
                        .into(),
                );
            }
        }
        argv.extend([
            "--manifest-path".into(),
            ws.join("Cargo.toml").to_string_lossy().into_owned(),
        ]);
        let mut env = BTreeMap::new();
        for (key, value) in c["env"]
            .as_object()
            .ok_or_else(|| Error::new("manifest: invalid env"))?
        {
            env.insert(
                key.clone(),
                value
                    .as_str()
                    .ok_or_else(|| Error::new("manifest: invalid env value"))?
                    .to_owned(),
            );
        }
        env.insert(
            "CARGO_TARGET_DIR".into(),
            target_dir.to_string_lossy().into_owned(),
        );
        env.insert("CARGO_BUILD_JOBS".into(), "1".into());
        env.insert("VIBEOS_GEN_DIR".into(), gen.to_string_lossy().into_owned());
        if force_off {
            env.insert("VIBEOS_FORCE_CMAKE_OFF".into(), "1".into());
        }
        let mut outputs = Vec::new();
        for o in c["outputs"]
            .as_array()
            .ok_or_else(|| Error::new("manifest: invalid outputs"))?
        {
            let bin = string(o, "bin")?;
            safe_rel(bin)?;
            let post = o["post"]
                .as_array()
                .ok_or_else(|| Error::new("manifest: invalid post"))?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| Error::new("manifest: invalid post step"))
                })
                .collect::<Result<Vec<_>>>()?;
            for step in &post {
                let args = shlex::split(step)
                    .ok_or_else(|| Error::new(format!("invalid post-link step {step:?}")))?;
                if !matches!(args.first().map(String::as_str), Some("pefixup") if args.len() == 2)
                    && !matches!(args.first().map(String::as_str), Some("fix_lld_forwarding") if args.len() == 1)
                    && !matches!(args.first().map(String::as_str), Some("strip") if args.len() >= 2)
                {
                    return Err(Error::new(format!(
                        "{name}: unknown post-link step {step:?}"
                    )));
                }
            }
            outputs.push(OutputPlan {
                source: target_dir
                    .join(target_stem)
                    .join("release")
                    .join(format!("{bin}.exe")),
                destination: out.join(safe_rel(string(o, "out")?)?),
                post,
            });
        }
        result.push(ManifestPlan {
            crate_name: name.into(),
            argv,
            env,
            outputs,
            log: out.join("logs").join(format!("{name}.log")),
        });
    }
    Ok(result)
}

/// Return the count of differing bytes and the last differing offset, including trailing bytes.
pub fn diff_offsets(a: &[u8], b: &[u8]) -> (usize, Option<usize>) {
    let mut count = 0;
    let mut last = None;
    for i in 0..a.len().max(b.len()) {
        if a.get(i) != b.get(i) {
            count += 1;
            last = Some(i);
        }
    }
    (count, last)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diff_includes_truncation() {
        assert_eq!(diff_offsets(b"abc", b"ax"), (2, Some(2)));
        assert_eq!(diff_offsets(b"a", b"a"), (0, None));
    }
    #[test]
    fn rejects_unsafe_paths() {
        for s in ["", "../x", "/tmp/x", "a/../b"] {
            assert!(safe_rel(s).is_err());
        }
        assert!(safe_rel("dir/file").is_ok());
    }
    #[test]
    fn detects_cycles_and_unknowns() {
        let cell = |target: &str, deps: &[&str]| Cell {
            target: target.into(),
            crate_name: target.into(),
            output: target.into(),
            kind: "native".into(),
            deps: deps.iter().map(|s| (*s).into()).collect(),
            arches: vec![],
        };
        let mut cells = BTreeMap::new();
        cells.insert("a".into(), cell("a", &["b"]));
        cells.insert("b".into(), cell("b", &["a"]));
        assert!(topo(&cells, &["a".into()]).is_err());
        assert!(topo(&cells, &["missing".into()]).is_err());
    }
}

/// Verify the producer and copied bytes on both sides of the copy before post-link.
fn copy_produced(source: &Path, dest: &Path) -> Result<()> {
    if dest.is_symlink() {
        return Err(Error::new(format!(
            "refusing symlink output {}",
            dest.display()
        )));
    }
    let before = Sha256::digest(crate::artifact_fs::verified_read(source, "Cargo output")?);
    fs::copy(source, dest)
        .map_err(|e| Error::new(format!("{} -> {}: {e}", source.display(), dest.display())))?;
    let after = Sha256::digest(crate::artifact_fs::verified_read(source, "Cargo output")?);
    let copied = Sha256::digest(crate::artifact_fs::verified_read(
        dest,
        "copied Cargo output",
    )?);
    if before != after || before != copied {
        return Err(Error::new(format!(
            "Cargo output changed during copy: {}",
            source.display()
        )));
    }
    Ok(())
}

/// Execute a manifest plan with Cargo output captured separately for each cell.
/// The caller owns supervisor lifetime, source stamp and SDK configuration.
pub fn execute_manifest(
    root: &Path,
    plan: &[ManifestPlan],
    pefixup: &Path,
    strip: &Path,
    store_env: &BTreeMap<String, String>,
    jobs: usize,
) -> Result<()> {
    use std::{
        process::Stdio,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Mutex,
        },
    };
    if jobs == 0 {
        return Err(Error::new("jobs must be positive"));
    }
    let next = AtomicUsize::new(0);
    let failures: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..jobs.min(plan.len()) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(cell) = plan.get(i) else { break };
                let execute = || -> Result<()> {
                    fs::create_dir_all(
                        cell.log
                            .parent()
                            .ok_or_else(|| Error::new("invalid log path"))?,
                    )?;
                    let log = fs::File::create(&cell.log)?;
                    let mut cmd = cargo_worker_command(root, &cell.argv)?;
                    cmd.current_dir(root.join("sdk/lib/rust"))
                        .env_remove("RUSTFLAGS")
                        .env_remove("RUSTUP_TOOLCHAIN")
                        .envs(&cell.env)
                        .envs(store_env)
                        .stdout(Stdio::from(log.try_clone()?))
                        .stderr(Stdio::from(log));
                    let status = cmd.status()?;
                    if !status.success() {
                        return Err(Error::new(format!(
                            "cargo failed for {}: {status} ({})",
                            cell.crate_name,
                            cell.log.display()
                        )));
                    }
                    for output in &cell.outputs {
                        fs::create_dir_all(
                            output
                                .destination
                                .parent()
                                .ok_or_else(|| Error::new("invalid destination"))?,
                        )?;
                        copy_produced(&output.source, &output.destination)?;
                        for step in &output.post {
                            let args = shlex::split(step).ok_or_else(|| {
                                Error::new(format!("invalid post-link step {step:?}"))
                            })?;
                            let executable = match args.first().map(String::as_str) {
                                Some("pefixup") if args.len() == 2 => {
                                    pefixup.to_string_lossy().into_owned()
                                }
                                Some("fix_lld_forwarding") if args.len() == 1 => "python3".into(),
                                Some("strip") if args.len() >= 2 => {
                                    strip.to_string_lossy().into_owned()
                                }
                                _ => {
                                    return Err(Error::new(format!(
                                        "unknown post-link step {step:?}"
                                    )))
                                }
                            };
                            let mut command = vec![executable];
                            if args[0] == "fix_lld_forwarding" {
                                command.push(
                                    root.join("sdk/tools/fix_lld_forwarding.py")
                                        .to_string_lossy()
                                        .into_owned(),
                                );
                            }
                            command.extend(args[1..].iter().cloned());
                            command.push(output.destination.to_string_lossy().into_owned());
                            checked(&mut cargo_worker_command(root, &command)?)?;
                        }
                        if !output.destination.is_file() {
                            return Err(Error::new(format!(
                                "post-link removed {}",
                                output.destination.display()
                            )));
                        }
                    }
                    Ok(())
                };
                match execute() {
                    Ok(()) => println!("[ws5] [{}/{}] ok   {}", i + 1, plan.len(), cell.crate_name),
                    Err(e) => {
                        eprintln!(
                            "[ws5] [{}/{}] FAIL {} ({})",
                            i + 1,
                            plan.len(),
                            cell.crate_name,
                            cell.log.display()
                        );
                        failures.lock().unwrap().push((i, e.to_string()));
                    }
                }
            });
        }
    });
    let mut failures = failures.into_inner().unwrap();
    failures.sort_by_key(|f| f.0);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(Error::new(
            failures
                .into_iter()
                .map(|(_, s)| s)
                .collect::<Vec<_>>()
                .join("\n"),
        ))
    }
}

/// The legacy explicit cell catalog, in its original order.
pub fn legacy_cells() -> Vec<Cell> {
    vec![
        Cell {
            target: "ntoskrnl".into(),
            crate_name: "ntoskrnl-cargo".into(),
            output: "ntoskrnl.exe".into(),
            kind: "kernelexe".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "ntkrnlmp".into(),
            crate_name: "ntkrnlmp-cargo".into(),
            output: "ntkrnlmp.exe".into(),
            kind: "kernelexe".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "ntdll".into(),
            crate_name: "ntdll-cargo".into(),
            output: "ntdll.dll".into(),
            kind: "win32dll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "ntdll_vista".into(),
            crate_name: "ntdll_vista-cargo".into(),
            output: "ntdll_vista.dll".into(),
            kind: "win32dll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "csrsrv".into(),
            crate_name: "csrsrv-cargo".into(),
            output: "csrsrv.dll".into(),
            kind: "win32dll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "basesrv".into(),
            crate_name: "basesrv-cargo".into(),
            output: "basesrv.dll".into(),
            kind: "win32dll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "winsrv".into(),
            crate_name: "winsrv-cargo".into(),
            output: "winsrv.dll".into(),
            kind: "win32dll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "csrss".into(),
            crate_name: "csrss-cargo".into(),
            output: "csrss.exe".into(),
            kind: "native".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "smss".into(),
            crate_name: "smss-cargo".into(),
            output: "smss.exe".into(),
            kind: "native".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "lsass".into(),
            crate_name: "lsass-cargo".into(),
            output: "lsass.exe".into(),
            kind: "win32exe".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "winlogon".into(),
            crate_name: "winlogon-cargo".into(),
            output: "winlogon.exe".into(),
            kind: "win32exe".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "services".into(),
            crate_name: "services-cargo".into(),
            output: "services.exe".into(),
            kind: "win32exe".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "userinit".into(),
            crate_name: "userinit-cargo".into(),
            output: "userinit.exe".into(),
            kind: "win32exe".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "hal".into(),
            crate_name: "hal-cargo".into(),
            output: "hal.dll".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "halmp".into(),
            crate_name: "halmp-cargo".into(),
            output: "halmp.dll".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "kdcom".into(),
            crate_name: "kdcom-cargo".into(),
            output: "kdcom.dll".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "bootvid".into(),
            crate_name: "bootvid-cargo".into(),
            output: "bootvid.dll".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "beep".into(),
            crate_name: "beep-cargo".into(),
            output: "beep.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "null".into(),
            crate_name: "null-cargo".into(),
            output: "null.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "nmidebug".into(),
            crate_name: "nmidebug-cargo".into(),
            output: "nmidebug.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "pci".into(),
            crate_name: "pci-cargo".into(),
            output: "pci.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "pciide".into(),
            crate_name: "pciide-cargo".into(),
            output: "pciide.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "classpnp".into(),
            crate_name: "classpnp-cargo".into(),
            output: "classpnp.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "condrv".into(),
            crate_name: "condrv-cargo".into(),
            output: "condrv.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "hidparse".into(),
            crate_name: "hidparse-cargo".into(),
            output: "hidparse.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "mouclass".into(),
            crate_name: "mouclass-cargo".into(),
            output: "mouclass.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "kbdclass".into(),
            crate_name: "kbdclass-cargo".into(),
            output: "kbdclass.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "i8042prt".into(),
            crate_name: "i8042prt-cargo".into(),
            output: "i8042prt.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "swenum".into(),
            crate_name: "swenum-cargo".into(),
            output: "swenum.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "usbd".into(),
            crate_name: "usbd-cargo".into(),
            output: "usbd.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "processr".into(),
            crate_name: "processr-cargo".into(),
            output: "processr.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "framebuf".into(),
            crate_name: "framebuf-cargo".into(),
            output: "framebuf.dll".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "netstack".into(),
            crate_name: "netstack-cargo".into(),
            output: "netstack.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "sysfb".into(),
            crate_name: "sysfb-cargo".into(),
            output: "sysfb.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["arm64".into()],
        },
        Cell {
            target: "arm64input".into(),
            crate_name: "arm64input-cargo".into(),
            output: "arm64input.sys".into(),
            kind: "kerneldll".into(),
            deps: vec![],
            arches: vec!["arm64".into()],
        },
        Cell {
            target: "win32k".into(),
            crate_name: "win32k-cargo".into(),
            output: "win32k.sys".into(),
            kind: "kernelmodedriver".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "uefildr".into(),
            crate_name: "uefildr-cargo".into(),
            output: "uefildr.efi".into(),
            kind: "efi".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into(), "arm64".into()],
        },
        Cell {
            target: "rosload".into(),
            crate_name: "rosload-cargo".into(),
            output: "rosload.exe".into(),
            kind: "native".into(),
            deps: vec![],
            arches: vec!["i386".into(), "amd64".into()],
        },
    ]
}

/// Replay the SDK only when either recipe changes. A failed replay never updates the stamp.
pub fn configure_sdk(root: &Path, arch: &str, gen: &Path) -> Result<()> {
    let mut hash = Sha256::new();
    for host in [true, false] {
        hash.update(fs::read(crate::sdk_recipe::recipe_path(root, arch, host))?);
    }
    let digest = format!("{:x}", hash.finalize());
    let stamp = gen.join(".sdk-recipe");
    if fs::read_to_string(&stamp).is_ok_and(|v| v.trim() == digest) {
        return Ok(());
    }
    #[cfg(test)]
    {
        let plan = crate::sdk_recipe::replay_plan(root, arch, gen)?;
        crate::sdk_recipe::run(&plan)?;
    }
    #[cfg(not(test))]
    {
        let args = vec![
            std::env::current_exe()?.to_string_lossy().into_owned(),
            "--root".into(),
            root.to_string_lossy().into_owned(),
            "sdk-recipe".into(),
            "--run".into(),
            "--arch".into(),
            arch.into(),
            "--gen".into(),
            gen.to_string_lossy().into_owned(),
        ];
        checked(&mut cargo_worker_command(root, &args)?)?;
    }
    fs::write(stamp, format!("{digest}\n"))?;
    Ok(())
}

/// Configure the two generated version-header pairs, reusing an existing complete pair.
pub fn configure_gen(root: &Path, arch: &str, gen: &Path, seed: Option<&Path>) -> Result<()> {
    configure_gen_with(root, arch, gen, seed, |mut command, required| {
        let result = command.output()?;
        if !result.status.success() && required {
            return Err(Error::new(format!("{command:?}: {}", result.status)));
        }
        Ok(if result.status.success() {
            String::from_utf8_lossy(&result.stdout).trim().to_string()
        } else {
            String::new()
        })
    })
}
fn configure_gen_with<F>(
    root: &Path,
    arch: &str,
    gen: &Path,
    seed: Option<&Path>,
    mut output: F,
) -> Result<()>
where
    F: FnMut(std::process::Command, bool) -> Result<String>,
{
    use std::process::Command;
    crate::arch(arch)?;
    let dirs = ["sdk/include/reactos", "host-tools/bin/sdk/include/reactos"];
    if dirs.iter().all(|d| {
        ["version.h", "buildno.h"]
            .iter()
            .all(|h| gen.join(d).join(h).exists())
    }) {
        return Ok(());
    }
    if let Some(seed) = seed {
        for d in dirs {
            fs::create_dir_all(gen.join(d))?;
            for h in ["version.h", "buildno.h"] {
                fs::copy(seed.join(d).join(h), gen.join(d).join(h))?;
            }
        }
        return Ok(());
    }
    let contents = fs::read_to_string(root.join("sdk/include/reactos/version.cmake"))?;
    let re = regex::Regex::new(r#"(?m)^set\((\w+) \"([^\"]*)\"\)"#)?;
    let mut values: BTreeMap<String, String> = re
        .captures_iter(&contents)
        .map(|c| (c[1].into(), c[2].into()))
        .collect();
    let v = |key: &str| -> Result<String> {
        values
            .get(key)
            .cloned()
            .ok_or_else(|| Error::new(format!("missing version.cmake variable {key}")))
    };
    let major = v("KERNEL_VERSION_MAJOR")?;
    let minor = v("KERNEL_VERSION_MINOR")?;
    let patch = v("KERNEL_VERSION_PATCH_LEVEL")?;
    let tag = v("KERNEL_VERSION_BUILD_TYPE")?;
    let suffix = if tag.is_empty() {
        String::new()
    } else {
        format!("-{tag}")
    };
    let mut date = Command::new("date");
    date.args(["-u", "+%Y%m%d"]);
    let mut git_hash = Command::new("git");
    git_hash.arg("-C").arg(root).args(["rev-parse", "HEAD"]);
    let mut git_revision = Command::new("git");
    git_revision
        .arg("-C")
        .arg(root)
        .args(["describe", "--abbrev=7", "--long", "--always"]);
    let mut clang = Command::new(root.join(".build-tools/llvm-mingw/bin/clang"));
    clang.arg("--version");
    let clang_version = output(clang, true)?;
    let clang_re = regex::Regex::new(r"clang version (\S+)")?;
    let clang_version = clang_re
        .captures(&clang_version)
        .map(|c| c[1].to_owned())
        .unwrap_or_default();
    let mut cc = Command::new("cc");
    cc.arg("--version");
    let cc_version = output(cc, true)?;
    let host_id = if cc_version.contains("clang") {
        "Clang"
    } else {
        "GNU"
    };
    let mut cc = Command::new("cc");
    cc.arg(if host_id == "GNU" {
        "-dumpfullversion"
    } else {
        "-dumpversion"
    });
    let host_version = output(cc, true)?;
    let hash = output(git_hash, false)?;
    let revision = output(git_revision, false)?;
    values.insert("KERNEL_VERSION_BUILD".into(), output(date, true)?);
    values.insert(
        "KERNEL_VERSION".into(),
        format!(
            "{major}.{minor}.{patch}-{}{suffix}",
            if arch == "i386" { "x86" } else { arch }
        ),
    );
    let dll_major = major
        .parse::<u32>()
        .map_err(|e| Error::new(e.to_string()))?
        + 42;
    values.insert("REACTOS_DLL_VERSION_MAJOR".into(), dll_major.to_string());
    values.insert(
        "DLL_VERSION_STR".into(),
        format!("{dll_major}.{minor}.{patch}{suffix}"),
    );
    values.insert(
        "COMMIT_HASH".into(),
        if hash.is_empty() {
            "unknown-hash".into()
        } else {
            hash
        },
    );
    values.insert(
        "REVISION".into(),
        if revision.is_empty() {
            "unknown-revision".into()
        } else {
            revision
        },
    );
    let template = regex::Regex::new(r"@(\w+)@")?;
    for (index, dir) in dirs.iter().enumerate() {
        values.insert(
            "CMAKE_C_COMPILER_ID".into(),
            if index == 0 { "Clang" } else { host_id }.into(),
        );
        values.insert(
            "CMAKE_C_COMPILER_VERSION".into(),
            if index == 0 {
                clang_version.clone()
            } else {
                host_version.clone()
            },
        );
        fs::create_dir_all(gen.join(dir))?;
        for h in ["version.h", "buildno.h"] {
            let text =
                fs::read_to_string(root.join("sdk/include/reactos").join(format!("{h}.cmake")))?;
            for c in template.captures_iter(&text) {
                if !values.contains_key(&c[1]) {
                    return Err(Error::new(format!(
                        "unknown version template variable {}",
                        &c[1]
                    )));
                }
            }
            let text = template.replace_all(&text, |c: &regex::Captures<'_>| values[&c[1]].clone());
            fs::write(gen.join(dir).join(h), text.as_bytes())?;
        }
    }
    Ok(())
}

/// Rebuild pefixup whenever either input or its complete compiler command changes.
pub fn build_pefixup(root: &Path, arch: &str, out: &Path, dry: bool) -> Result<PathBuf> {
    build_pefixup_traced(root, arch, out, dry, &mut Vec::new())
}

fn build_pefixup_traced(
    root: &Path,
    arch: &str,
    out: &Path,
    dry: bool,
    traces: &mut Vec<String>,
) -> Result<PathBuf> {
    crate::arch(arch)?;
    let exe = out.join(format!("native-pefixup-{arch}"));
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let mut args = vec![
        "-O2".to_owned(),
        "-DTARGET_amd64".into(),
        "-I".into(),
        root.join("sdk/include/host").to_string_lossy().into_owned(),
    ];
    if arch != "i386" {
        args.push("-D_TARGET_PE64".into());
    }
    let src = root.join("sdk/tools/pefixup.c");
    args.extend([
        "-o".into(),
        exe.to_string_lossy().into_owned(),
        src.to_string_lossy().into_owned(),
    ]);
    let mut hash = Sha256::new();
    hash.update(fs::read(src)?);
    hash.update(fs::read(root.join("dll/win32/dbghelp/compat.h"))?);
    let mut command = vec![cc.clone()];
    command.extend(args.clone());
    hash.update(command.join("\0"));
    let digest = format!("{:x}", hash.finalize());
    let stamp = out.join(format!("native-pefixup-{arch}.stamp"));
    if !dry && exe.is_file() && fs::read_to_string(&stamp).is_ok_and(|s| s.trim() == digest) {
        return Ok(exe);
    }
    if dry {
        eprintln!("[ws5] DRY  {}", command.join(" "));
        return Ok(exe);
    }
    fs::create_dir_all(out)?;
    traces.push(shell_line(&command, None));
    checked(&mut cargo_worker_command(root, &command)?)?;
    if !exe.is_file() {
        return Err(Error::new(format!(
            "pefixup compiler did not produce {}",
            exe.display()
        )));
    }
    fs::write(stamp, format!("{digest}\n"))?;
    Ok(exe)
}

/// Execute and stamp a complete manifest only if its source remains unchanged.
/// `store_env` is owned by the caller's live artifact supervisor.
pub fn build_manifest(
    root: &Path,
    arch: &str,
    out: &Path,
    gen: &Path,
    pefixup: &Path,
    strip: &Path,
    crates: Option<&[String]>,
    force_off: bool,
    store_env: &BTreeMap<String, String>,
    jobs: usize,
) -> Result<()> {
    let source = execute_manifest_checked(
        root, arch, out, gen, pefixup, strip, crates, force_off, store_env, jobs,
    )?;
    stamp_manifest(root, arch, out, source.as_deref())
}

fn execute_manifest_checked(
    root: &Path,
    arch: &str,
    out: &Path,
    gen: &Path,
    pefixup: &Path,
    strip: &Path,
    crates: Option<&[String]>,
    force_off: bool,
    store_env: &BTreeMap<String, String>,
    jobs: usize,
) -> Result<Option<String>> {
    match fs::remove_file(out.join(".cells.json")) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    let source = if crates.is_none() {
        Some(crate::media_manifest::source_hash(root)?)
    } else {
        None
    };
    let plan = manifest_plan(root, arch, out, gen, crates, force_off)?;
    execute_manifest(root, &plan, pefixup, strip, store_env, jobs)?;
    Ok(source)
}

fn stamp_manifest(root: &Path, arch: &str, out: &Path, source: Option<&str>) -> Result<()> {
    if let Some(source) = source {
        if crate::media_manifest::source_hash(root)? != source {
            return Err(Error::new(
                "the source tree changed during the build: not stamping it",
            ));
        }
        fs::write(
            out.join(".cells.json"),
            format!("{}\n", serde_json::json!({"arch": arch, "source": source})),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    #[test]
    fn manifest_plan_preserves_commands_and_refuses_unknowns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cells = root.join("sdk/lib/rust/cells");
        fs::create_dir_all(&cells).unwrap();
        fs::write(cells.join("i386.jsonl"), r#"{"bins":["driver"],"build_std":true,"crate":"driver-cargo","env":{"RUSTUP_TOOLCHAIN":"nightly"},"outputs":[{"bin":"driver","out":"drivers/driver.sys","post":["pefixup --kerneldll","fix_lld_forwarding"]}],"target":"sdk/lib/rust/driver-cargo/i686-kernel.json","target_dir":"cell-link-v1-i386-1"}
"#).unwrap();
        let out = root.join("out");
        let gen = root.join("gen");
        let p = manifest_plan(root, "i386", &out, &gen, None, true).unwrap();
        assert_eq!(p.len(), 1);
        // Frozen from the Python executor observing this manifest fixture.
        assert_eq!(
            p[0].argv,
            vec![
                "cargo".to_owned(),
                "build".into(),
                "--release".into(),
                "--target".into(),
                root.join("sdk/lib/rust/driver-cargo/i686-kernel.json")
                    .to_string_lossy()
                    .into_owned(),
                "-Z".into(),
                "build-std=core,alloc,compiler_builtins".into(),
                "-Z".into(),
                "build-std-features=compiler-builtins-mem".into(),
                "-p".into(),
                "driver-cargo".into(),
                "--bin".into(),
                "driver".into(),
                "--manifest-path".into(),
                root.join("sdk/lib/rust/Cargo.toml")
                    .to_string_lossy()
                    .into_owned()
            ]
        );
        assert_eq!(
            p[0].outputs[0].source,
            root.join("sdk/lib/rust/target/cell-link-v1-i386-1/i686-kernel/release/driver.exe")
        );
        assert_eq!(
            p[0].outputs[0].post,
            vec!["pefixup --kerneldll", "fix_lld_forwarding"]
        );
        assert_eq!(p[0].env["RUSTUP_TOOLCHAIN"], "nightly");
        assert!(p[0]
            .argv
            .contains(&"build-std=core,alloc,compiler_builtins".to_owned()));
        assert_eq!(p[0].env["CARGO_BUILD_JOBS"], "1");
        assert_eq!(p[0].env["VIBEOS_FORCE_CMAKE_OFF"], "1");
        assert_eq!(p[0].outputs[0].destination, out.join("drivers/driver.sys"));
        assert!(manifest_plan(root, "i386", &out, &gen, Some(&["unknown".into()]), false).is_err());
        let path = cells.join("i386.jsonl");
        let bad = fs::read_to_string(&path)
            .unwrap()
            .replace("pefixup --kerneldll", "pefixup --kerneldll --extra");
        fs::write(path, bad).unwrap();
        assert!(manifest_plan(root, "i386", &out, &gen, None, false).is_err());
    }
}

/// Run a manifest under the native artifact supervisor, which owns the store lease.
/// The host-jobs executable must be provisioned by the existing broker bootstrap.
pub fn supervised_manifest(
    root: &Path,
    arch: &str,
    out: &Path,
    gen: &Path,
    pefixup: &Path,
    strip: &Path,
    crates: Option<&[String]>,
    force_off: bool,
    jobs: usize,
) -> Result<()> {
    let home = PathBuf::from(
        std::env::var_os("HOME")
            .ok_or_else(|| Error::new("HOME is required for the artifact store"))?,
    );
    let store = std::env::var_os("VIBEOS_ARTIFACT_STORE_ROOT")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("vibeos-artifact-store"));
    let real = std::env::var_os("VIBEOS_ORCHESTRATE_REPO")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| root.to_owned());
    let toolchain = home
        .join(".rustup/toolchains")
        .join(crate::artifact_executor::PIN);
    let options = crate::artifact_runtime::SupervisorOptions {
        arch: Some(arch.into()),
        ..Default::default()
    };
    let mut sup = crate::artifact_runtime::Supervisor::open(
        &real,
        &store,
        &toolchain,
        &out.join("artifact-supervisor"),
        &options,
    )?;
    let state: Value = serde_json::from_slice(&fs::read(sup.path())?)?;
    let mut env = BTreeMap::new();
    env.insert("VIBEOS_ARTIFACT_STORE_MODE".into(), "ON".into());
    env.insert(
        "VIBEOS_ARTIFACT_STORE_ROOT".into(),
        store.to_string_lossy().into_owned(),
    );
    env.insert("VIBEOS_ARTIFACT_VERIFY_NONCE".into(), "".into());
    env.insert(
        "VIBEOS_ARTIFACT_SUPERVISOR".into(),
        sup.path().to_string_lossy().into_owned(),
    );
    env.insert(
        "VIBEOS_ARTIFACT_TOOL".into(),
        real.join("tools/build-infra")
            .to_string_lossy()
            .into_owned(),
    );
    env.insert(
        "VIBEOS_ARTIFACT_TOOLCHAIN_ID".into(),
        string(&state, "toolchain_key")?.to_owned(),
    );
    let result = execute_manifest_checked(
        root, arch, out, gen, pefixup, strip, crates, force_off, &env, jobs,
    );
    let record = crate::artifact_runtime::record_outer_result(
        sup.path(),
        if result.is_ok() { 0 } else { 1 },
    );
    let failure = result
        .as_ref()
        .err()
        .or_else(|| record.as_ref().err())
        .map(|e| serde_json::json!({"type":"OuterBuildFailure", "message":e.to_string()}));
    let close = sup.close(failure);
    if result.is_err() || record.is_err() || close.is_err() {
        match fs::remove_file(out.join(".cells.json")) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    let source = result?;
    record?;
    close?;
    stamp_manifest(root, arch, out, source.as_deref())
}

/// Build the legacy per-cell command, keeping every package in its own invocation.
pub fn legacy_plan(
    root: &Path,
    cell: &Cell,
    arch: &str,
    out: &Path,
    trace: Option<&Path>,
    force_off: bool,
) -> Result<ManifestPlan> {
    let ws = root.join("sdk/lib/rust");
    let target_dir = cargo_target_dir(root, cell, arch)?;
    let std = build_std(root, cell, arch)?;
    let target = if std {
        target_json(root, cell, arch)?
            .to_string_lossy()
            .into_owned()
    } else {
        cell_stem(root, cell, arch)?.into()
    };
    let mut argv = vec![
        "cargo".into(),
        "build".into(),
        "--release".into(),
        "--target".into(),
        target,
    ];
    let mut env = BTreeMap::new();
    if std {
        argv.extend(
            [
                "-Z",
                "build-std=core,alloc,compiler_builtins",
                "-Z",
                "build-std-features=compiler-builtins-mem",
            ]
            .map(String::from),
        );
        env.insert(
            "RUSTUP_TOOLCHAIN".into(),
            fs::read_to_string(root.join(".rust-nightly"))?
                .trim()
                .into(),
        );
    } else {
        let mut flags = "-C force-frame-pointers=yes".to_owned();
        if ["amd64", "i386"].contains(&arch) {
            flags.push_str(" -C link-self-contained=no");
        }
        if arch == "i386" {
            flags.push_str(" -C target-feature=-sse,-sse2");
        }
        env.insert("RUSTFLAGS".into(), flags);
    }
    env.insert(
        "CARGO_TARGET_DIR".into(),
        target_dir.to_string_lossy().into_owned(),
    );
    if let Some(t) = trace {
        env.insert("VIBEOS_LINK_TRACE".into(), t.to_string_lossy().into_owned());
    }
    if force_off {
        env.insert("VIBEOS_FORCE_CMAKE_OFF".into(), "1".into());
    }
    argv.extend([
        "--manifest-path".into(),
        ws.join("Cargo.toml").to_string_lossy().into_owned(),
        "-p".into(),
        cell.crate_name.clone(),
    ]);
    let mut post = Vec::new();
    match cell.kind.as_str() {
        "kernelexe" => post.push("pefixup --kernel".into()),
        "kerneldll" => post.push("pefixup --kerneldll".into()),
        "kernelmodedriver" => post.push("pefixup --kernelmodedriver".into()),
        "win32dll" | "win32exe" | "native" => (),
        "efi" => {
            post.push("strip --remove-section=.rossym".into());
            post.push("strip --strip-all".into());
        }
        _ => {
            return Err(Error::new(format!(
                "unknown KIND {:?} for cell {}",
                cell.kind, cell.target
            )))
        }
    }
    if arch == "i386"
        && [
            "kernelexe",
            "kerneldll",
            "kernelmodedriver",
            "win32dll",
            "win32exe",
        ]
        .contains(&cell.kind.as_str())
    {
        post.push("fix_lld_forwarding".into());
    }
    Ok(ManifestPlan {
        crate_name: cell.crate_name.clone(),
        argv,
        env,
        outputs: vec![OutputPlan {
            source: built_exe(root, cell, arch)?,
            destination: out.join(&cell.output),
            post,
        }],
        log: out.join("logs").join(format!("{}.log", cell.crate_name)),
    })
}

#[cfg(test)]
mod isolation_tests {
    use super::*;
    #[test]
    fn custom_target_bytes_isolate_core_builds() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut a = legacy_cells()
            .into_iter()
            .find(|c| c.target == "ntoskrnl")
            .unwrap();
        let mut b = a.clone();
        b.crate_name = "other-cargo".into();
        for (c, bytes) in [(&a, b"soft".as_slice()), (&b, b"hard".as_slice())] {
            let path = target_json(root, c, "i386").unwrap();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        assert_ne!(
            cargo_target_dir(root, &a, "i386").unwrap(),
            cargo_target_dir(root, &b, "i386").unwrap()
        );
        fs::write(target_json(root, &b, "i386").unwrap(), b"soft").unwrap();
        assert_eq!(
            cargo_target_dir(root, &a, "i386").unwrap(),
            cargo_target_dir(root, &b, "i386").unwrap()
        );
        a.kind = "win32dll".into();
        assert!(!build_std(root, &a, "i386").unwrap());
        assert_eq!(cell_stem(root, &a, "i386").unwrap(), "i686-pc-windows-gnu");
    }
}

fn checked(cmd: &mut std::process::Command) -> Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(Error::new(format!("{cmd:?}: {status}")));
    }
    Ok(())
}
fn cross_strip(root: &Path) -> Result<PathBuf> {
    let tc = root.join(".build-tools/llvm-mingw/bin/llvm-strip");
    if tc.is_file() {
        return Ok(tc);
    }
    let mut candidates = Vec::new();
    if let Some(s) = std::env::var_os("STRIP").filter(|s| !s.is_empty()) {
        candidates.push(PathBuf::from(s));
    }
    candidates.extend(["llvm-strip", "strip"].map(PathBuf::from));
    for candidate in candidates {
        if candidate.components().count() > 1 && candidate.is_file() {
            return Ok(candidate);
        }
        if let Some(path) = std::env::var_os("PATH") {
            if std::env::split_paths(&path).any(|dir| dir.join(&candidate).is_file()) {
                return Ok(candidate);
            }
        }
    }
    Err(Error::new("cross strip tool not found"))
}
fn host_jobs_digest(root: &Path) -> Result<String> {
    fn walk(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<std::io::Result<_>>()?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let ty = e.file_type()?;
            let p = e.path();
            if ty.is_dir() && e.file_name() != "target" {
                walk(&p, paths)?;
            } else if !ty.is_dir()
                && !fs::metadata(&p)?.is_dir()
                && p.extension().is_some_and(|x| x == "rs")
            {
                paths.push(p);
            }
        }
        Ok(())
    }
    let source = root.join("tools/host-jobs/src");
    let mut files = Vec::new();
    walk(&source, &mut files)?;
    files.sort();
    files.push(root.join("rust-toolchain.toml"));
    let mut hash = Sha256::new();
    for p in files {
        hash.update(fs::read(p)?);
    }
    Ok(format!("{:x}", hash.finalize()))
}

/// Provision the existing host-jobs broker under its shared flock and source stamp.
pub fn ensure_host_jobs(root: &Path) -> Result<PathBuf> {
    use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
    let source = root.join("tools/host-jobs/src");
    let target = root.join(".cache/host-jobs/host-jobs");
    fs::create_dir_all(target.parent().unwrap())?;
    let lock = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(target.with_extension("lock"))?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let digest = host_jobs_digest(root)?;
    let stamp = target.with_extension("sha256");
    if target.is_file() && fs::read_to_string(&stamp).is_ok_and(|s| s == digest) {
        return Ok(target);
    }
    let temp = target.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| -> Result<()> {
        let argv = vec![
            "rustc".into(),
            "--edition=2021".into(),
            "-O".into(),
            source.join("main.rs").to_string_lossy().into_owned(),
            "-o".into(),
            temp.to_string_lossy().into_owned(),
        ];
        let mut cmd = cargo_worker_command(root, &argv)?;
        cmd.current_dir(root).env_remove("RUSTUP_TOOLCHAIN");
        checked(&mut cmd)?;
        fs::rename(&temp, &target)?;
        fs::write(stamp, digest)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result?;
    Ok(target)
}

fn shell_line(argv: &[String], cwd: Option<&Path>) -> String {
    let cmd = argv
        .iter()
        .map(|s| {
            shlex::try_quote(s)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| s.clone())
        })
        .collect::<Vec<_>>()
        .join(" ");
    match cwd {
        Some(cwd) => format!("cd {} && {cmd}", cwd.display()),
        None => cmd,
    }
}
fn legacy_run(
    root: &Path,
    plans: &[ManifestPlan],
    pefixup: &Path,
    strip: &Path,
    dry: bool,
    force_off: bool,
    traces: &mut Vec<String>,
) -> Result<()> {
    let ws = root.join("sdk/lib/rust");
    for p in plans {
        if dry {
            println!(
                "[ws5] DRY  CARGO_TARGET_DIR={} {}  (cwd={})",
                p.env["CARGO_TARGET_DIR"],
                p.argv.join(" "),
                ws.display()
            );
            continue;
        }
        let mut cmd = cargo_worker_command(root, &p.argv)?;
        cmd.current_dir(&ws).envs(&p.env);
        if force_off {
            cmd.env("VIBEOS_FORCE_CMAKE_OFF", "1");
        }
        traces.push(shell_line(&p.argv, Some(&ws)));
        println!("[ws5] exec cargo build — {}", p.crate_name);
        checked(&mut cmd)?;
    }
    for p in plans {
        for o in &p.outputs {
            if dry {
                println!(
                    "[ws5] DRY  copy {} -> {}; post-link",
                    o.source.display(),
                    o.destination.display()
                );
            } else {
                fs::create_dir_all(o.destination.parent().ok_or("invalid destination")?)?;
                copy_produced(&o.source, &o.destination)?;
            }
            for step in &o.post {
                let args = shlex::split(step).ok_or("invalid post-link step")?;
                let executable = match args.first().map(String::as_str) {
                    Some("pefixup") => pefixup.to_path_buf(),
                    Some("strip") => strip.to_path_buf(),
                    Some("fix_lld_forwarding") => PathBuf::from("python3"),
                    _ => return Err(Error::new(format!("unknown post-link step {step:?}"))),
                };
                let mut command = vec![executable.to_string_lossy().into_owned()];
                if args[0] == "fix_lld_forwarding" {
                    command.push(
                        root.join("sdk/tools/fix_lld_forwarding.py")
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
                command.extend(args[1..].iter().cloned());
                command.push(o.destination.to_string_lossy().into_owned());
                if dry {
                    println!("[ws5] DRY  {}", command.join(" "));
                } else {
                    traces.push(shell_line(&command, None));
                    checked(&mut cargo_worker_command(root, &command)?)?;
                }
            }
            if !dry {
                println!(
                    "[ws5] OK   {}  ({} bytes)",
                    o.destination.display(),
                    fs::metadata(&o.destination)?.len()
                );
            }
        }
    }
    Ok(())
}

const SPINE: &[&str] = &[
    "ntdll",
    "ntdll_vista",
    "csrsrv",
    "basesrv",
    "winsrv",
    "csrss",
    "smss",
    "lsass",
    "winlogon",
    "services",
    "userinit",
];
const DRIVERS: &[&str] = &[
    "win32k",
    "hal",
    "kdcom",
    "bootvid",
    "beep",
    "null",
    "nmidebug",
    "pci",
    "pciide",
    "classpnp",
    "condrv",
    "hidparse",
    "mouclass",
    "kbdclass",
    "i8042prt",
    "swenum",
    "usbd",
    "processr",
    "framebuf",
    "netstack",
    "sysfb",
    "arm64input",
];
const LOADERS: &[&str] = &["uefildr", "rosload"];
#[derive(Default)]
struct CliOptions {
    arch: String,
    out: Option<PathBuf>,
    targets: Vec<String>,
    list: bool,
    spine: bool,
    drivers: bool,
    loaders: bool,
    all: bool,
    manifest: bool,
    cmake_off: bool,
    seed: bool,
    jobs: Option<usize>,
    no_store: bool,
    reconfigure: bool,
    dry: bool,
    verify: bool,
    trace: Option<Option<PathBuf>>,
}
fn options(raw: &[String]) -> Result<CliOptions> {
    let mut normalized = Vec::new();
    let mut positional = false;
    for arg in raw {
        if arg == "--" {
            positional = true;
        }
        if !positional {
            if let Some((key, value)) = arg.split_once('=') {
                if ["--arch", "--out-dir", "--jobs", "--trace"].contains(&key) {
                    normalized.push(key.to_owned());
                    normalized.push(value.to_owned());
                    continue;
                }
            }
        }
        normalized.push(arg.clone());
    }
    let args = &normalized;
    let mut o = CliOptions::default();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let has_next_value = args.get(i + 1).is_some_and(|s| !s.starts_with('-'));
        let mut value = || -> Result<String> {
            i += 1;
            args.get(i)
                .cloned()
                .ok_or_else(|| Error::new(format!("{arg} requires a value")))
        };
        match arg.as_str() {
            "--arch" => o.arch = value()?, "--out-dir" => o.out = Some(value()?.into()),
            "--jobs" => { let v = value()?; o.jobs = Some(v.parse().map_err(|_| Error::new(format!("invalid jobs: {v}")))?); },
            "--list" => o.list = true, "--spine" => o.spine = true, "--drivers" => o.drivers = true,
            "--loaders" => o.loaders = true, "--all" => o.all = true, "--manifest" => o.manifest = true,
            "--cmake-off" => o.cmake_off = true, "--seed-from-build" => o.seed = true,
            "--no-artifact-store" => o.no_store = true, "--reconfigure" => o.reconfigure = true,
            "--dry-run" => o.dry = true, "--verify" => o.verify = true,
            "--trace" => { if has_next_value { o.trace = Some(Some(value()?.into())); } else { o.trace = Some(None); } },
            "--help" | "-h" => return Err(Error::new("usage: cargo-orchestrate --arch i386|amd64|arm64 [--manifest] [--list] [--dry-run] [--verify] [--jobs N] [targets...]")),
            "--" => { o.targets.extend_from_slice(&args[i+1..]); break; },
            _ if arg.starts_with('-') => return Err(Error::new(format!("unknown option {arg}"))),
            _ => o.targets.push(arg.clone()),
        }
        i += 1;
    }
    crate::arch(&o.arch)?;
    if o.jobs == Some(0) {
        return Err(Error::new("jobs must be positive"));
    }
    Ok(o)
}
/// Native entry point; canonical mount re-entry retains the real checkout for supervision.
pub fn cli(root: &Path, args: &[String]) -> Result<u8> {
    if args
        .iter()
        .take_while(|s| s.as_str() != "--")
        .any(|s| s == "--help" || s == "-h")
    {
        println!("usage: cargo-orchestrate --arch i386|amd64|arm64 [--manifest] [--list] [--dry-run] [--verify] [--jobs N] [targets...]");
        return Ok(0);
    }
    let o = options(args)?;
    let cells = legacy_cells();
    if o.list {
        for c in &cells {
            println!(
                "{:<12} {:<18} {:<16} {}{}",
                c.target,
                c.crate_name,
                c.output,
                c.kind,
                if c.deps.is_empty() {
                    String::new()
                } else {
                    format!("  deps={}", c.deps.join(","))
                }
            );
        }
        return Ok(0);
    }
    let canon =
        PathBuf::from(std::env::var_os("VIBEOS_CANON").unwrap_or_else(|| "/tmp/vibeos".into()));
    if !o.dry && root != canon {
        use std::os::unix::process::CommandExt;
        let error = std::process::Command::new("bash")
            .arg(root.join("tools/canon-run.sh"))
            .arg(canon.join("tools/build-infra"))
            .arg("--root")
            .arg(&canon)
            .arg("cargo-orchestrate")
            .args(args)
            .current_dir(root)
            .env("VIBEOS_ORCHESTRATE_REPO", root)
            .exec();
        return Err(error.into());
    }
    let out = o
        .out
        .clone()
        .unwrap_or_else(|| root.join("cmake-off-out").join(&o.arch));
    let strip = || cross_strip(root);
    if o.manifest {
        let build = root.join(format!("build-{}", o.arch));
        let gen = if build.join("build.ninja").exists() {
            out.join("gen")
        } else {
            build.clone()
        };
        let selected = if o.targets.is_empty() {
            None
        } else {
            Some(o.targets.as_slice())
        };
        let preview = manifest_plan(root, &o.arch, &out, &gen, selected, o.cmake_off)?;
        if o.dry {
            for p in &preview {
                println!(
                    "[ws5] DRY  {}",
                    shell_line(&p.argv, Some(&root.join("sdk/lib/rust")))
                );
            }
            return Ok(0);
        }
        let pefixup = build_pefixup(root, &o.arch, &out, false)?;
        if o.reconfigure && gen.exists() {
            if gen != build {
                fs::remove_dir_all(&gen)?;
            } else {
                for path in [
                    "sdk/include/reactos/version.h",
                    "sdk/include/reactos/buildno.h",
                    ".sdk-recipe",
                ] {
                    match fs::remove_file(gen.join(path)) {
                        Ok(()) => (),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }
        configure_gen(
            root,
            &o.arch,
            &gen,
            if o.seed { Some(&build) } else { None },
        )?;
        if o.cmake_off {
            configure_sdk(root, &o.arch, &gen)?;
        }
        let strip = if preview
            .iter()
            .flat_map(|p| &p.outputs)
            .flat_map(|o| &o.post)
            .any(|s| s.starts_with("strip "))
        {
            strip()?
        } else {
            PathBuf::from("strip")
        };
        if o.no_store {
            build_manifest(
                root,
                &o.arch,
                &out,
                &gen,
                &pefixup,
                &strip,
                selected,
                o.cmake_off,
                &BTreeMap::new(),
                o.jobs.unwrap_or(4),
            )?;
        } else {
            ensure_host_jobs(root)?;
            supervised_manifest(
                root,
                &o.arch,
                &out,
                &gen,
                &pefixup,
                &strip,
                selected,
                o.cmake_off,
                o.jobs.unwrap_or(4),
            )?;
        }
        return Ok(0);
    }
    let mut targets = o.targets.clone();
    if o.spine || o.all {
        let mut first = vec!["ntoskrnl".into(), "ntkrnlmp".into()];
        first.extend(SPINE.iter().map(|s| (*s).into()));
        first.extend(targets);
        targets = first;
    }
    if o.drivers || o.all {
        targets.extend(DRIVERS.iter().map(|s| (*s).into()));
    }
    if o.loaders || o.all {
        targets.extend(LOADERS.iter().map(|s| (*s).into()));
    }
    if targets.is_empty() {
        targets.push("ntoskrnl".into());
    }
    let map = cells.into_iter().map(|c| (c.target.clone(), c)).collect();
    let ordered = topo(&map, &targets)?;
    let skipped = ordered
        .iter()
        .filter(|c| !c.arches.contains(&o.arch))
        .map(|c| c.target.clone())
        .collect::<Vec<_>>();
    if !skipped.is_empty() {
        println!("[ws5] skipping (not on {}): {}", o.arch, skipped.join(", "));
    }
    let cells = ordered
        .into_iter()
        .filter(|c| c.arches.contains(&o.arch))
        .collect::<Vec<_>>();
    println!(
        "[ws5] arch={} cells={}",
        o.arch,
        cells
            .iter()
            .map(|c| c.target.as_str())
            .collect::<Vec<_>>()
            .join(" -> ")
    );
    let trace_dir = if o.trace.is_some() && !o.dry {
        let p = out.join(".trace");
        if p.exists() {
            fs::remove_dir_all(&p)?;
        }
        fs::create_dir_all(&p)?;
        Some(p)
    } else {
        None
    };
    let mut traces = Vec::new();
    let pefixup = build_pefixup_traced(root, &o.arch, &out, o.dry, &mut traces)?;
    let strip = if cells.iter().any(|c| c.kind == "efi") {
        strip()?
    } else {
        PathBuf::from("strip")
    };
    if o.verify {
        if o.dry {
            return Err(Error::new("--verify cannot be combined with --dry-run"));
        }
        for (dir, force) in [(".verify-replay", false), (".verify-off", true)] {
            let plans = cells
                .iter()
                .map(|c| legacy_plan(root, c, &o.arch, &out.join(dir), None, force))
                .collect::<Result<Vec<_>>>()?;
            legacy_run(root, &plans, &pefixup, &strip, false, force, &mut traces)?;
        }
        let mut fails = Vec::new();
        for c in &cells {
            let a = fs::read(out.join(".verify-replay").join(&c.output))?;
            let b = fs::read(out.join(".verify-off").join(&c.output))?;
            let (n, last) = diff_offsets(&a, &b);
            let ok = last.is_none_or(|i| i < 0x400);
            println!(
                "[ws5]   {} {} ({} differing bytes, last {:?})",
                if ok { "OK" } else { "DRIFT" },
                c.output,
                n,
                last
            );
            if !ok {
                fails.push(c.target.clone());
            }
        }
        if !fails.is_empty() {
            eprintln!("[ws5] verify: DRIFT — {}", fails.join(", "));
            return Ok(1);
        }
        println!(
            "[ws5] verify: all {} cells match within PE header",
            cells.len()
        );
        return Ok(0);
    }
    let plans = cells
        .iter()
        .map(|c| legacy_plan(root, c, &o.arch, &out, trace_dir.as_deref(), false))
        .collect::<Result<Vec<_>>>()?;
    legacy_run(root, &plans, &pefixup, &strip, o.dry, false, &mut traces)?;
    if let Some(dir) = trace_dir {
        let trace_out = o
            .trace
            .flatten()
            .unwrap_or_else(|| out.join("build-trace.txt"));
        let mut lines = vec![
            format!(
                "# cmake-off build trace — arch={}, cells: {}",
                o.arch,
                cells
                    .iter()
                    .map(|c| c.target.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            "# (no cmake configure, no ninja; every command this build ran)".into(),
            "".into(),
            "# === orchestrator-spawned commands ===".into(),
        ];
        lines.extend(traces);
        lines.extend([
            "".into(),
            "# === in-cargo (reactos-link engine) commands, per cell ===".into(),
        ]);
        for c in &cells {
            let path = dir.join(format!("{}.trace", c.crate_name));
            let body = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(e) => return Err(e.into()),
            };
            let command_lines = body.lines().collect::<Vec<_>>();
            lines.push(format!(
                "## {} ({}) — {} commands",
                c.target,
                c.crate_name,
                command_lines.len()
            ));
            lines.extend(command_lines.iter().map(|s| (*s).to_owned()));
        }
        fs::write(&trace_out, format!("{}\n", lines.join("\n")))?;
        println!("[ws5] trace -> {}", trace_out.display());
    }
    println!("[ws5] done — cmake-off build complete (no cmake configure, no ninja)");
    Ok(0)
}

/// Compatibility transport for the Python API. Algorithms stay in this module.
pub fn dispatch(root: &Path, request: &Value) -> Result<Value> {
    let op = string(request, "op")?;
    let kw = &request["kwargs"];
    let cell = || -> Result<Cell> {
        Ok(serde_json::from_value(
            kw.get("cell").cloned().ok_or("missing cell")?,
        )?)
    };
    let arch = || string(kw, "arch");
    match op {
        "cargo_orchestrate_cells" => Ok(serde_json::to_value(legacy_cells())?),
        "cargo_orchestrate_ensure_host_jobs" => Ok(serde_json::to_value(ensure_host_jobs(root)?)?),
        "cargo_orchestrate_build_std" => Ok(serde_json::json!(build_std(root, &cell()?, arch()?)?)),
        "cargo_orchestrate_cell_stem" => Ok(serde_json::json!(cell_stem(root, &cell()?, arch()?)?)),
        "cargo_orchestrate_target_json" => {
            Ok(serde_json::json!(target_json(root, &cell()?, arch()?)?))
        }
        "cargo_orchestrate_target_dir" => Ok(serde_json::json!(cargo_target_dir(
            root,
            &cell()?,
            arch()?
        )?)),
        "cargo_orchestrate_built_exe" => Ok(serde_json::json!(built_exe(root, &cell()?, arch()?)?)),
        "cargo_orchestrate_legacy_plan" => {
            let out = PathBuf::from(string(kw, "out")?);
            let force = kw
                .get("force_off")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(serde_json::to_value(legacy_plan(
                root,
                &cell()?,
                arch()?,
                &out,
                None,
                force,
            )?)?)
        }
        "cargo_orchestrate_manifest_plan" => {
            let out = PathBuf::from(string(kw, "out")?);
            let gen = PathBuf::from(string(kw, "gen")?);
            let crates: Option<Vec<String>> =
                serde_json::from_value(kw.get("crates").cloned().unwrap_or(Value::Null))?;
            let force = kw
                .get("force_off")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(serde_json::to_value(manifest_plan(
                root,
                arch()?,
                &out,
                &gen,
                crates.as_deref(),
                force,
            )?)?)
        }
        "cargo_orchestrate_topo" => {
            let targets: Vec<String> =
                serde_json::from_value(kw.get("targets").cloned().ok_or("missing targets")?)?;
            let map = legacy_cells()
                .into_iter()
                .map(|c| (c.target.clone(), c))
                .collect();
            Ok(serde_json::to_value(topo(&map, &targets)?)?)
        }
        _ => Err(Error::new(format!(
            "unknown cargo orchestration operation {op}"
        ))),
    }
}

#[cfg(test)]
mod execution_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn requires_produced_outputs_and_checks_postlink() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ws = root.join("sdk/lib/rust");
        fs::create_dir_all(&ws).unwrap();
        let source = root.join("producer.exe");
        let dest = root.join("out/driver.sys");
        let tool = root.join("pefixup");
        fs::write(&tool, "#!/bin/sh\nprintf post >> \"$2\"\n").unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
        let mut p = ManifestPlan {
            crate_name: "driver-cargo".into(),
            argv: vec![
                "/bin/sh".into(),
                "-c".into(),
                "printf cargo > \"$1\"".into(),
                "_".into(),
                source.to_string_lossy().into_owned(),
            ],
            env: BTreeMap::new(),
            outputs: vec![OutputPlan {
                source: source.clone(),
                destination: dest.clone(),
                post: vec!["pefixup --kernel".into()],
            }],
            log: root.join("out/logs/driver.log"),
        };
        execute_manifest(root, &[p.clone()], &tool, &tool, &BTreeMap::new(), 2).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"cargopost");
        p.argv = vec!["/bin/true".into()];
        fs::remove_file(&source).unwrap();
        assert!(execute_manifest(root, &[p.clone()], &tool, &tool, &BTreeMap::new(), 1).is_err());
        fs::write(&source, b"x").unwrap();
        fs::write(&tool, "#!/bin/sh\nexit 4\n").unwrap();
        assert!(execute_manifest(root, &[p], &tool, &tool, &BTreeMap::new(), 1).is_err());
        assert!(execute_manifest(root, &[], &tool, &tool, &BTreeMap::new(), 0).is_err());
    }
}

/// One worker process per cell preserves parallel jobs while each owned group has
/// its own signal supervisor and watches the orchestrator's process incarnation.
fn cargo_worker_command(root: &Path, argv: &[String]) -> Result<std::process::Command> {
    if argv.is_empty() {
        return Err(Error::new("empty Cargo command"));
    }
    #[cfg(test)]
    {
        let _ = root;
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        Ok(cmd)
    }
    #[cfg(not(test))]
    {
        let mut cmd = std::process::Command::new(std::env::current_exe()?);
        cmd.arg("--root")
            .arg(root)
            .arg("cargo-orchestrate-worker")
            .args(argv);
        Ok(cmd)
    }
}
/// Internal process boundary; no shell is involved in executing the supplied argv.
pub fn worker_cli(argv: &[String]) -> Result<u8> {
    if argv.is_empty() {
        return Err(Error::new("cargo orchestrator worker requires a command"));
    }
    let parent = unsafe { libc::getppid() } as u32;
    if parent < 2 {
        return Err(Error::new("cargo orchestrator worker lost its parent"));
    }
    crate::artifact_lifetime::watch_owner(parent)?;
    crate::artifact_executor::supervised_status(
        std::process::Command::new(&argv[0]).args(&argv[1..]),
    )
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    #[test]
    fn changed_sources_do_not_get_a_stamp() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let status = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success());
        fs::write(root.join(".gitignore"), "out/\n").unwrap();
        fs::write(root.join("source.c"), "before").unwrap();
        let source = crate::media_manifest::source_hash(root).unwrap();
        fs::create_dir(root.join("out")).unwrap();
        fs::write(root.join("source.c"), "after").unwrap();
        assert!(stamp_manifest(root, "amd64", &root.join("out"), Some(&source)).is_err());
        assert!(!root.join("out/.cells.json").exists());
        let source = crate::media_manifest::source_hash(root).unwrap();
        stamp_manifest(root, "amd64", &root.join("out"), Some(&source)).unwrap();
        assert_eq!(
            crate::media_manifest::cells_source(&root.join("out"), "amd64").unwrap(),
            source
        );
    }
}

#[cfg(test)]
mod version_parity_tests {
    use super::*;
    #[test]
    fn frozen_python_version_headers_and_reuse() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src = root.join("sdk/include/reactos");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("version.cmake"), "set(KERNEL_VERSION_MAJOR \"1\")\nset(KERNEL_VERSION_MINOR \"2\")\nset(KERNEL_VERSION_PATCH_LEVEL \"3\")\nset(KERNEL_VERSION_BUILD_TYPE \"dev\")\n").unwrap();
        fs::write(
            src.join("version.h.cmake"),
            "@KERNEL_VERSION@|@DLL_VERSION_STR@|@CMAKE_C_COMPILER_ID@|@CMAKE_C_COMPILER_VERSION@\n",
        )
        .unwrap();
        fs::write(
            src.join("buildno.h.cmake"),
            "@KERNEL_VERSION_BUILD@|@COMMIT_HASH@|@REVISION@|@REACTOS_DLL_VERSION_MAJOR@\n",
        )
        .unwrap();
        let gen = root.join("gen");
        let run = |c: std::process::Command, _: bool| -> Result<String> {
            let program = c.get_program().to_string_lossy();
            let args: Vec<_> = c
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            Ok(if program.ends_with("clang") {
                "clang version 19.1.0"
            } else if program == "cc" && args[0] == "--version" {
                "gcc version"
            } else if program == "cc" {
                "13.3.0"
            } else if program == "date" {
                "20260102"
            } else if args.iter().any(|a| a == "rev-parse") {
                "abcdef"
            } else if args.iter().any(|a| a == "describe") {
                "r123"
            } else {
                return Err(Error::new("unexpected command"));
            }
            .into())
        };
        configure_gen_with(root, "i386", &gen, None, run).unwrap();
        // Captured from the original Python configure_gen with the same command replies.
        assert_eq!(
            fs::read_to_string(gen.join("sdk/include/reactos/version.h")).unwrap(),
            "1.2.3-x86-dev|43.2.3-dev|Clang|19.1.0\n"
        );
        assert_eq!(
            fs::read_to_string(gen.join("host-tools/bin/sdk/include/reactos/version.h")).unwrap(),
            "1.2.3-x86-dev|43.2.3-dev|GNU|13.3.0\n"
        );
        for dir in ["sdk/include/reactos", "host-tools/bin/sdk/include/reactos"] {
            assert_eq!(
                fs::read_to_string(gen.join(dir).join("buildno.h")).unwrap(),
                "20260102|abcdef|r123|43\n"
            );
        }
        configure_gen_with(root, "i386", &gen, None, |_, _| {
            Err(Error::new("must reuse"))
        })
        .unwrap();
    }
}

#[cfg(test)]
mod broker_parity_tests {
    use super::*;
    #[test]
    fn frozen_python_source_order_and_exclusion() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src = root.join("tools/host-jobs/src");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::create_dir_all(src.join("target")).unwrap();
        fs::write(src.join("a.rs"), "first\n").unwrap();
        fs::write(src.join("nested/b.rs"), "second\n").unwrap();
        fs::write(src.join("target/ignored.rs"), "ignored").unwrap();
        fs::write(root.join("rust-toolchain.toml"), "toolchain\n").unwrap();
        // Frozen from hashlib.sha256(b''.join(sorted(source_files(...)) + [toolchain])).
        assert_eq!(
            host_jobs_digest(root).unwrap(),
            "6db7bf6967bc43fa8a45b7017e23fcfc2a6bfc484f02a16e422d4cb771e090bc"
        );
    }
}

#[cfg(test)]
mod legacy_parity_tests {
    use super::*;
    #[test]
    fn frozen_legacy_commands_for_all_architectures() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join(".rust-nightly"), "nightly-pinned\n").unwrap();
        let cell = legacy_cells()
            .into_iter()
            .find(|c| c.target == "ntoskrnl")
            .unwrap();
        let target = target_json(root, &cell, "i386").unwrap();
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"soft").unwrap();
        let ws = root.join("sdk/lib/rust");
        let out = root.join("out");
        for (arch, stem, flags) in [
            (
                "amd64",
                "x86_64-pc-windows-gnu",
                "-C force-frame-pointers=yes -C link-self-contained=no",
            ),
            (
                "arm64",
                "aarch64-pc-windows-gnullvm",
                "-C force-frame-pointers=yes",
            ),
        ] {
            let p = legacy_plan(root, &cell, arch, &out, None, false).unwrap();
            assert_eq!(
                p.argv,
                vec![
                    "cargo".to_owned(),
                    "build".into(),
                    "--release".into(),
                    "--target".into(),
                    stem.into(),
                    "--manifest-path".into(),
                    ws.join("Cargo.toml").to_string_lossy().into_owned(),
                    "-p".into(),
                    "ntoskrnl-cargo".into()
                ]
            );
            assert_eq!(p.env["RUSTFLAGS"], flags);
            assert_eq!(p.outputs[0].post, vec!["pefixup --kernel"]);
        }
        let p = legacy_plan(root, &cell, "i386", &out, None, false).unwrap();
        assert_eq!(
            p.argv,
            vec![
                "cargo".to_owned(),
                "build".into(),
                "--release".into(),
                "--target".into(),
                target.to_string_lossy().into_owned(),
                "-Z".into(),
                "build-std=core,alloc,compiler_builtins".into(),
                "-Z".into(),
                "build-std-features=compiler-builtins-mem".into(),
                "--manifest-path".into(),
                ws.join("Cargo.toml").to_string_lossy().into_owned(),
                "-p".into(),
                "ntoskrnl-cargo".into()
            ]
        );
        assert_eq!(p.env["RUSTUP_TOOLCHAIN"], "nightly-pinned");
        assert_eq!(
            p.outputs[0].post,
            vec!["pefixup --kernel", "fix_lld_forwarding"]
        );
        let user = legacy_cells()
            .into_iter()
            .find(|c| c.target == "ntdll")
            .unwrap();
        let p = legacy_plan(root, &user, "i386", &out, None, false).unwrap();
        assert_eq!(p.argv[4], "i686-pc-windows-gnu");
        assert_eq!(
            p.env["RUSTFLAGS"],
            "-C force-frame-pointers=yes -C link-self-contained=no -C target-feature=-sse,-sse2"
        );
        assert_eq!(p.outputs[0].post, vec!["fix_lld_forwarding"]);
    }
}
