//! Resource recipes shared by native regeneration and the artifact drift checker.
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

pub const NAMES: &[&str] = &[
    "hal-res",
    "ntdll-res",
    "ntoskrnl-res",
    "win32k-res",
    "committed-res",
];
pub const ARCHES: &[&str] = &["i386", "amd64", "arm64"];
const CELLS: &[(&str, &str)] = &[
    ("basesrv", "subsystems/win/basesrv/basesrv.rc"),
    ("services", "base/system/services/services.rc"),
    ("userinit", "base/system/userinit/userinit.rc"),
    ("winlogon", "base/system/winlogon/winlogon.rc"),
];

fn specs() -> &'static Value {
    static SPECS: OnceLock<Value> = OnceLock::new();
    SPECS.get_or_init(|| {
        serde_json::from_str(include_str!("resource_specs.json")).expect("resource manifest")
    })
}
fn arr(value: &Value) -> &[Value] {
    value.as_array().expect("resource array")
}
fn text(value: &Value) -> &str {
    value.as_str().expect("resource string")
}
fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
fn define(args: &mut Vec<String>, pairs: &Value) {
    for pair in arr(pairs) {
        args.push(match pair[1].as_str() {
            Some(value) => format!("-D{}={value}", text(&pair[0])),
            None => format!("-D{}", text(&pair[0])),
        });
    }
}

// Python's shlex.split(command) disables comments; the Rust shell lexer enables
// them. Protect unquoted hashes so a legal path/argument cannot truncate a recipe.
fn split_command(command: &str) -> Result<Vec<String>, String> {
    let mut escaped = String::new();
    let mut quote = None;
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        if c == '\\' && quote != Some('\'') {
            escaped.push(c);
            if let Some(next) = chars.next() {
                escaped.push(next);
            }
            continue;
        }
        if c == '#' && quote.is_none() {
            escaped.push('\\');
        }
        if c == '\'' || c == '"' {
            if quote == Some(c) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(c);
            }
        }
        escaped.push(c);
    }
    shlex::split(&escaped).ok_or_else(|| "invalid shell quoting in resource compile command".into())
}

pub struct Job {
    pub stem: String,
    pub output: PathBuf,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    stage: Option<PathBuf>,
    checkout: PathBuf,
}

/// Original recipe shape retained for --jobs-json consumers. No command is executed.
pub type RawJob = (String, String, Vec<String>, String);
pub struct CommittedJobs {
    pub jobs: Option<Vec<RawJob>>,
    pub messages: String,
}
impl CommittedJobs {
    pub fn json(&self) -> Value {
        json!({"jobs":self.jobs,"messages":self.messages})
    }
}

pub fn committed_jobs(root: &Path, arch: &str) -> Result<CommittedJobs, String> {
    if !ARCHES.contains(&arch) {
        return Err(format!("unknown arch: {arch}"));
    }
    let db = root.join(format!("build-{arch}/compile_commands.json"));
    let source = match fs::read_to_string(&db) {
        Ok(source) => source,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CommittedJobs {
                jobs: None,
                messages: String::new(),
            })
        }
        Err(e) => return Err(format!("{}: {e}", db.display())),
    };
    let db: Value =
        serde_json::from_str(&source).map_err(|e| format!("compile_commands.json: {e}"))?;
    let db = db
        .as_array()
        .ok_or("compile_commands.json must be an array")?;
    let mut jobs = Vec::new();
    let mut messages = String::new();
    for (stem, rc) in CELLS {
        let basename = rc.rsplit('/').next().unwrap();
        let marker = format!("{stem}_res.dir");
        let hit = db.iter().find(|e| {
            e["file"].as_str().is_some_and(|f| f.ends_with(basename))
                && (e["command"].as_str().is_some_and(|c| c.contains(&marker))
                    || e["arguments"].as_array().is_some_and(|a| {
                        a.iter()
                            .any(|p| p.as_str().is_some_and(|s| s.contains(&marker)))
                    }))
        });
        let Some(hit) = hit else {
            messages += &format!("  [{arch}] {stem}: NO compile_commands entry\n");
            continue;
        };
        let argv: Vec<String> = if let Some(args) = hit.get("arguments") {
            args.as_array()
                .ok_or("compile command arguments must be an array")?
                .iter()
                .map(|s| {
                    s.as_str()
                        .map(str::to_owned)
                        .ok_or("compile command argument must be a string".to_string())
                })
                .collect::<Result<_, _>>()?
        } else {
            split_command(
                hit["command"]
                    .as_str()
                    .ok_or("compile command is missing command/arguments")?,
            )?
        };
        if argv.is_empty() {
            return Err(format!("empty resource command for {stem}"));
        }
        let cwd = hit["directory"]
            .as_str()
            .ok_or("compile command directory must be a string")?;
        jobs.push((
            (*stem).into(),
            format!("sdk/lib/rust/{stem}-cargo/data/{stem}-{arch}.rc.res"),
            argv,
            cwd.into(),
        ));
    }
    Ok(CommittedJobs {
        jobs: Some(jobs),
        messages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_words_keep_hashes_and_quoted_paths() {
        assert_eq!(
            split_command("tool #literal 'one two' \"a#b\" escaped\\#hash").unwrap(),
            ["tool", "#literal", "one two", "a#b", "escaped#hash"]
        );
        assert!(split_command("tool 'unterminated").is_err());
    }
    #[test]
    fn canonical_rebasing_does_not_match_another_checkout_prefix() {
        let root = Path::new("/our/checkout");
        assert_eq!(
            rebase(root, "--input=/tmp/vibeos/file.rc"),
            "--input=/our/checkout/file.rc"
        );
        assert_eq!(rebase(root, "/tmp/vibeos"), "/our/checkout");
        assert_eq!(
            rebase(root, "/tmp/vibeos-other/file.rc"),
            "/tmp/vibeos-other/file.rc"
        );
    }
}

/// Canonical build paths are aliases for THIS checkout, even if /tmp/vibeos exists
/// and currently belongs to another lane. Never use its existence as an ownership test.
fn rebase(root: &Path, token: &str) -> String {
    let result = token.replace("/tmp/vibeos/", &format!("{}/", root.display()));
    if let Some(prefix) = result.strip_suffix("/tmp/vibeos") {
        format!("{prefix}{}", root.display())
    } else {
        result
    }
}

pub fn localize_job(root: &Path, raw: &RawJob, work: &Path) -> Result<Job, String> {
    let cwd = PathBuf::from(rebase(root, &raw.3));
    if !cwd.starts_with(root) || !cwd.is_dir() {
        return Err(format!(
            "resource working directory is not in this checkout: {}",
            cwd.display()
        ));
    }
    // Canonicalize too: a symlink under root must not route mutable inputs to a sibling.
    let resolved = fs::canonicalize(&cwd).map_err(|e| e.to_string())?;
    let local_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    if !resolved.starts_with(&local_root) {
        return Err(format!(
            "resource cwd resolves outside checkout: {}",
            cwd.display()
        ));
    }
    let mut argv: Vec<_> = raw.2.iter().map(|s| rebase(root, s)).collect();
    if argv.len() < 3 {
        return Err("resource command has no source/output arguments".into());
    }
    let source = PathBuf::from(&argv[argv.len() - 2]);
    let source = if source.is_absolute() {
        source
    } else {
        cwd.join(source)
    };
    let source = fs::canonicalize(&source)
        .map_err(|e| format!("resource source {}: {e}", source.display()))?;
    if !source.starts_with(&local_root) {
        return Err(format!(
            "resource source resolves outside checkout: {}",
            source.display()
        ));
    }
    *argv.last_mut().ok_or("empty resource command")? =
        path(&work.join(format!("{}.rc.res", raw.0)));
    Ok(Job {
        stem: raw.0.clone(),
        output: root.join(&raw.1),
        argv,
        cwd,
        stage: None,
        checkout: root.into(),
    })
}

pub fn plan(
    root: &Path,
    family: &str,
    module: &str,
    arch: &str,
    work: &Path,
) -> Result<Job, String> {
    if !ARCHES.contains(&arch) {
        return Err(format!("unknown arch: {arch}"));
    }
    let all = specs();
    let cfg = &all[family];
    if cfg.is_null() {
        return Err(format!("unknown resource family: {family}"));
    }
    if family == "hal" && arch != "i386" {
        return Err("HAL resource recipe supports only i386".into());
    }
    let acfg = if family == "ntdll" {
        &cfg["ARCHES"][arch]
    } else {
        &all["KERNEL_ARCHES"][arch]
    };
    let triple = if family == "hal" {
        "i686-w64-mingw32"
    } else {
        text(&acfg["triple"])
    };
    let bindir = root.join(".build-tools/llvm-mingw/bin");
    let windres = bindir.join(format!("{triple}-windres"));
    let clang = bindir.join(format!("{triple}-clang"));
    let mut argv = vec![path(&windres), "-O".into(), "coff".into()];
    let mcfg = &cfg["MODULES"][module];
    let mut includes = Vec::new();
    let source;
    let destination;
    let mut stage = None;
    if family == "ntdll" {
        if mcfg.is_null() {
            return Err(format!("unknown ntdll module: {module}"));
        }
        source = root.join(text(&mcfg["rc"]));
        destination = root.join(text(&mcfg["out"]).replace("{arch}", arch));
        for inc in arr(&mcfg["includes"]) {
            let base = if text(&inc[0]) == "bld" {
                root.join(format!("build-{arch}"))
            } else {
                root.to_owned()
            };
            includes.push(base.join(text(&inc[1])));
        }
    } else {
        source = root.join(text(&cfg["RC_SOURCE"]));
        destination = if family == "hal" {
            root.join(text(&cfg["OUT_FILE"]))
        } else {
            root.join(format!(
                "sdk/lib/rust/{family}-cargo/data/{family}-{arch}.rc.res"
            ))
        };
        let key = if family == "win32k" {
            "WIN32K_INCLUDES"
        } else {
            "KERNEL_INCLUDES"
        };
        includes.extend(arr(&cfg[key]).iter().map(|s| root.join(text(s))));
        if family == "ntoskrnl" {
            includes.push(work.join("mc-stage"));
            stage = Some(root.join(format!("build-{arch}/sdk/include/reactos/mc")));
        } else if family == "win32k" {
            includes.push(root.join(format!("build-{arch}/sdk/include")));
            includes.push(root.join(format!("build-{arch}/sdk/include/reactos")));
        }
    }
    for inc in includes {
        argv.extend(["-I".into(), path(&inc)]);
    }
    argv.extend([
        format!("--preprocessor={}", clang.display()),
        format!("--preprocessor-arg=--target={triple}"),
        "--preprocessor-arg=-E".into(),
        "--preprocessor-arg=-nostdinc".into(),
        "--preprocessor-arg=-xc-header".into(),
    ]);
    if family == "ntdll" {
        define(&mut argv, &cfg["COMMON_DEFINES"]);
    } else {
        argv.extend([
            "-DRC_INVOKED".into(),
            "-D__WIN32__=1".into(),
            "-D__FLAT__=1".into(),
        ]);
    }
    if family != "hal" {
        argv.extend(
            arr(&all["LANGUAGES"])
                .iter()
                .map(|s| format!("-DLANGUAGE_{}", text(s))),
        );
    }
    match family {
        "hal" => define(&mut argv, &cfg["RC_DEFINES"]),
        "ntdll" => {
            define(&mut argv, &cfg["TAIL_DEFINES"]);
            argv.extend(
                arr(&acfg["defines"])
                    .iter()
                    .map(|s| format!("-D{}", text(s))),
            );
        }
        "ntoskrnl" => {
            define(&mut argv, &cfg["KERNEL_DEFINES_COMMON"]);
            define(&mut argv, &acfg["defines"]);
        }
        "win32k" => {
            define(&mut argv, &cfg["WIN32K_DEFINES_COMMON"]);
            define(&mut argv, &acfg["defines"]);
        }
        _ => unreachable!(),
    }
    argv.extend([path(&source), path(&work.join(format!("{module}.rc.res")))]);
    Ok(Job {
        stem: module.into(),
        output: destination,
        argv,
        cwd: root.into(),
        stage,
        checkout: root.into(),
    })
}

static TEMP_ID: AtomicU64 = AtomicU64::new(0);
pub struct TempDir(PathBuf);
impl TempDir {
    pub fn new() -> Result<Self, String> {
        for _ in 0..100 {
            let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let p = std::env::temp_dir()
                .join(format!("vibeos-codegen-res-{}-{id}", std::process::id()));
            match fs::create_dir(&p) {
                Ok(()) => return Ok(Self(p)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        Err("cannot allocate a resource work directory".into())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn execute(job: &Job, work: &Path) -> Result<Vec<u8>, String> {
    execute_with_argv(job, job.argv.clone(), work)
}

pub fn execute_with_argv(job: &Job, argv: Vec<String>, work: &Path) -> Result<Vec<u8>, String> {
    let root = fs::canonicalize(&job.checkout).map_err(|e| e.to_string())?;
    for pair in argv.windows(2) {
        if pair[0] != "-I" {
            continue;
        }
        let include = Path::new(&pair[1]);
        if include
            .strip_prefix(&job.checkout)
            .ok()
            .and_then(|r| r.components().next())
            .is_some_and(|c| c.as_os_str().to_string_lossy().starts_with("build-"))
        {
            match fs::canonicalize(include) {
                Ok(resolved) if !resolved.starts_with(&root) => {
                    return Err(format!(
                        "generated include resolves outside checkout: {}",
                        include.display()
                    ))
                }
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.to_string()),
                _ => {}
            }
        }
    }
    if let Some(source) = &job.stage {
        let rc = source.join("bugcodes.rc");
        if !rc.is_file() {
            return Err(format!("missing {}; build this arch first", rc.display()));
        }
        let dest = work.join("mc-stage");
        fs::create_dir(&dest).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_name().to_string_lossy().starts_with("bugcodes") && entry.path().is_file()
            {
                if !fs::canonicalize(entry.path())
                    .map_err(|e| e.to_string())?
                    .starts_with(&root)
                {
                    return Err("message sidecar resolves outside checkout".into());
                }
                fs::copy(entry.path(), dest.join(entry.file_name())).map_err(|e| e.to_string())?;
            }
        }
    }
    let (program, args) = argv.split_first().ok_or("empty resource command")?;
    let result = Command::new(program)
        .args(args)
        .current_dir(&job.cwd)
        .output()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "resource {} failed ({}): {}",
            job.stem,
            result.status,
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let output = args.last().ok_or("resource command has no output path")?;
    let bytes = fs::read(output)
        .map_err(|e| format!("resource {} produced no readable output: {e}", job.stem))?;
    if bytes.is_empty() {
        return Err(format!("resource {} produced an empty output", job.stem));
    }
    Ok(bytes)
}

pub fn selection(
    root: &Path,
    name: &str,
    arches: &[String],
    modules: &[String],
    work: &Path,
    execute_mode: bool,
) -> Result<(Vec<Job>, String), String> {
    if arches.is_empty() {
        return Err("resource generation requires --arch explicitly".into());
    }
    for arch in arches {
        if !ARCHES.contains(&arch.as_str()) {
            return Err(format!("unknown arch: {arch}"));
        }
    }
    let mut jobs = Vec::new();
    let mut messages = String::new();
    if name == "committed-res" {
        if !modules.is_empty() {
            return Err("--module is only valid for ntdll-res".into());
        }
        for arch in arches {
            let selected = committed_jobs(root, arch)?;
            messages += &selected.messages;
            let Some(raw) = selected.jobs else {
                return Err(format!("missing build-{arch}/compile_commands.json"));
            };
            if execute_mode && raw.len() != CELLS.len() {
                return Err(format!(
                    "incomplete resource recipes for {arch}:\n{messages}"
                ));
            }
            for job in raw {
                jobs.push(localize_job(root, &job, work)?);
            }
        }
    } else {
        let family = name
            .strip_suffix("-res")
            .ok_or("unknown resource generator")?;
        if family != "ntdll" && !modules.is_empty() {
            return Err("--module is only valid for ntdll-res".into());
        }
        let modules = if family == "ntdll" && modules.is_empty() {
            vec!["ntdll".into(), "ntdll_vista".into()]
        } else if modules.is_empty() {
            vec![family.to_owned()]
        } else {
            modules.to_vec()
        };
        for module in modules {
            for arch in arches {
                jobs.push(plan(root, family, &module, arch, work)?);
            }
        }
    }
    Ok((jobs, messages))
}
