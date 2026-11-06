//! Recover Cargo cell manifests from the configured Ninja graph.
use crate::{arch, local, read, Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
const CANON: &str = "/tmp/vibeos";
pub fn manifest_path(arch: &str, root: &Path) -> PathBuf {
    root.join(format!("sdk/lib/rust/cells/{arch}.jsonl"))
}
pub fn build_rel(path: &str, arch: &str) -> Result<String> {
    path.strip_prefix(&format!("{CANON}/build-{arch}/"))
        .map(str::to_owned)
        .ok_or_else(|| Error::new(format!("cell output outside build-{arch}: {path}")))
}
pub fn post(cmd: &[String], out: &str, _arch: &str) -> Result<String> {
    if cmd.last().map(String::as_str) != Some(out) {
        return Err(Error::new(format!(
            "post-link step not on its output: {}",
            cmd.join(" ")
        )));
    }
    if cmd.first().is_some_and(|x| x.ends_with("/pefixup")) {
        return Ok(format!(
            "pefixup {}",
            cmd.get(1)
                .ok_or_else(|| Error::new("pefixup missing option"))?
        ));
    }
    if cmd.len() >= 5
        && cmd[..4] == ["/usr/bin/cmake", "-E", "env", "python3"]
        && cmd[4].ends_with("sdk/tools/fix_lld_forwarding.py")
    {
        return Ok("fix_lld_forwarding".into());
    }
    if cmd.first().is_some_and(|x| x.ends_with("/llvm-strip")) {
        return Ok(format!(
            "strip {}",
            cmd.get(1..cmd.len().saturating_sub(1))
                .unwrap_or_default()
                .join(" ")
        ));
    }
    Err(Error::new(format!(
        "unmodelled post-link step: {}",
        cmd.join(" ")
    )))
}
pub fn parse_cell(outs: &str, command: &str, arch: &str) -> Result<Value> {
    let steps: Vec<Vec<String>> = command
        .split(" && ")
        .map(|s| {
            shlex::split(s).ok_or_else(|| Error::new("invalid shell quoting in cargo command"))
        })
        .collect::<Result<_>>()?;
    let short = command.chars().take(120).collect::<String>();
    if steps.first().and_then(|x| x.first()).map(String::as_str) != Some("cd") {
        return Err(Error::new(format!("cargo command without its cd: {short}")));
    }
    let run = steps
        .get(1)
        .ok_or_else(|| Error::new("cargo command missing env step"))?;
    if run.len() < 3 || run[..3] != ["/usr/bin/cmake", "-E", "env"] {
        return Err(Error::new(format!(
            "cargo command without its env: {short}"
        )));
    }
    let at = run
        .iter()
        .position(|x| x == "cargo")
        .ok_or_else(|| Error::new("cargo command missing cargo"))?;
    if at < 3 {
        return Err(Error::new("invalid cargo environment"));
    }
    let mut env = BTreeMap::new();
    for arg in &run[3..at] {
        let (k, v) = arg
            .split_once('=')
            .ok_or_else(|| Error::new(format!("invalid cargo env {arg}")))?;
        env.insert(k, v);
    }
    let args = &run[at..];
    let opt = |k: &str| -> Result<&str> {
        let i = args
            .iter()
            .position(|a| a == k)
            .ok_or_else(|| Error::new(format!("cargo command missing {k}")))?;
        args.get(i + 1)
            .map(String::as_str)
            .ok_or_else(|| Error::new(format!("cargo command missing value after {k}")))
    };
    let krate = opt("-p")?;
    let target = opt("--target")?.replace(&format!("{CANON}/"), "");
    let target_dir = Path::new(
        env.get("CARGO_TARGET_DIR")
            .ok_or_else(|| Error::new("cargo command missing CARGO_TARGET_DIR"))?,
    )
    .file_name()
    .ok_or_else(|| Error::new("invalid target directory"))?
    .to_string_lossy()
    .into_owned();
    let mut saved = serde_json::Map::new();
    for k in ["RUSTFLAGS", "RUSTUP_TOOLCHAIN", "VIBEOS_PIC_HAL"] {
        if let Some(v) = env.get(k) {
            saved.insert(k.into(), json!(v));
        }
    }
    let bins = if args.iter().any(|v| v == "--bins") {
        json!("all")
    } else {
        let mut bins = Vec::new();
        for (i, x) in args.iter().enumerate() {
            if x == "--bin" {
                bins.push(
                    args.get(i + 1)
                        .ok_or_else(|| Error::new("cargo --bin missing value"))?
                        .clone(),
                );
            }
        }
        json!(bins)
    };
    let mut outputs = Vec::new();
    let mut i = 2;
    while i < steps.len() {
        let s = &steps[i];
        if s.len() >= 5 && s[..3] == ["/usr/bin/cmake", "-E", "copy"] {
            let src = &s[3];
            let dst = &s[4];
            let mut posts = Vec::new();
            i += 1;
            while i < steps.len()
                && steps[i].last() == Some(dst)
                && !(steps[i].len() >= 3 && steps[i][..3] == ["/usr/bin/cmake", "-E", "copy"])
            {
                posts.push(post(&steps[i], dst, arch)?);
                i += 1;
            }
            outputs.push(json!({"bin":Path::new(src).file_stem().unwrap_or_default().to_string_lossy(),"out":build_rel(dst,arch)?,"post":posts}));
        } else {
            i += 1;
        }
    }
    let mut declared: Vec<_> = outs
        .split('|')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    declared.sort();
    let mut actual: Vec<_> = outputs
        .iter()
        .filter_map(|v| v["out"].as_str().map(str::to_owned))
        .collect();
    actual.sort();
    if declared != actual {
        return Err(Error::new(format!(
            "{krate}: copies do not match the declared outputs"
        )));
    }
    Ok(
        json!({"crate":krate,"target":target,"build_std":args.iter().any(|x|x=="build-std=core,alloc,compiler_builtins"),"env":saved,"target_dir":target_dir,"bins":bins,"outputs":outputs}),
    )
}
pub fn derive(arch_name: &str, root: &Path) -> Result<Vec<Value>> {
    arch(arch_name)?;
    let p = root.join(format!("build-{arch_name}/build.ninja"));
    if !p.exists() {
        return Err(Error::new(format!(
            "{} missing: configure with ./vibeos-cli build --arch {arch_name}",
            p.display()
        )));
    }
    crate::ninja::require_configured(&local(root, &root.join(format!("build-{arch_name}")))?, root)?;
    let text = read(&local(root, &p)?)?;
    let pattern = Regex::new(
        r"\nbuild ([^\n]*?): CUSTOM_COMMAND[^\n]*\n  COMMAND = ([^\n]*)\n  DESC = Cargo-linking ([^\n]*)",
    )?;
    let mut cells = Vec::new();
    let mut seen = BTreeSet::new();
    for c in pattern.captures_iter(&text) {
        let v = parse_cell(&c[1], &c[2], arch_name)?;
        let name = v["crate"].as_str().unwrap();
        if !seen.insert(name.to_owned()) {
            return Err(Error::new(format!(
                "build-{arch_name}: a crate links twice"
            )));
        }
        cells.push(v);
    }
    cells.sort_by(|a, b| a["crate"].as_str().cmp(&b["crate"].as_str()));
    Ok(cells)
}
pub fn render(cells: &[Value]) -> String {
    cells
        .iter()
        .map(|v| format!("{}\n", super::json::dumps(v, true)))
        .collect()
}
pub fn load(arch_name: &str, root: &Path) -> Result<Vec<Value>> {
    arch(arch_name)?;
    let p = manifest_path(arch_name, root);
    if !p.exists() {
        return Err(Error::new(format!(
            "{} missing: python3 tools/cell_manifest.py --write",
            p.display()
        )));
    }
    read(&local(root, &p)?)?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| serde_json::from_str(s).map_err(Into::into))
        .collect()
}
pub fn current(arch_name: &str, root: &Path) -> Result<Vec<Value>> {
    if root.join(format!("build-{arch_name}/build.ninja")).exists() {
        derive(arch_name, root)
    } else {
        load(arch_name, root)
    }
}
pub fn drift(committed: &[Value], derived: &[Value]) -> Result<Vec<String>> {
    let to_map = |v: &[Value]| -> Result<BTreeMap<String, Value>> {
        v.iter()
            .map(|x| {
                Ok((
                    x["crate"]
                        .as_str()
                        .ok_or_else(|| Error::new("manifest cell missing crate"))?
                        .into(),
                    x.clone(),
                ))
            })
            .collect()
    };
    let have = to_map(committed)?;
    let want = to_map(derived)?;
    let mut out = Vec::new();
    for k in want.keys().filter(|k| !have.contains_key(*k)) {
        out.push(format!("+ {k}"));
    }
    for k in have.keys().filter(|k| !want.contains_key(*k)) {
        out.push(format!("- {k}"));
    }
    for k in want
        .keys()
        .filter(|k| have.contains_key(*k) && want[*k] != have[*k])
    {
        out.push(format!("~ {k}"));
    }
    Ok(out)
}
pub fn write_plan(root: &Path, arches: &[String]) -> Result<BTreeMap<PathBuf, String>> {
    let mut out = BTreeMap::new();
    for a in arches {
        out.insert(manifest_path(a, root), render(&derive(a, root)?));
    }
    Ok(out)
}
