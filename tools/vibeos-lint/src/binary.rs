//! Post-link plugins share checked, content-addressed extraction, never verdicts.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

mod bindall;
mod cohab;
mod drift;
mod flex;
mod freeldr;
mod i386_abi;
mod imports;
mod ir;
mod sacred;
mod simple;
mod ud2;
mod unwind;
mod warnings;
mod wide_strings;

pub const RULES: &[&str] = &[
    "freeldr_size",
    "scan_ud2",
    "sacred_primitives",
    "rust_ub_deny",
    "scan_self_recursive",
    "scan_flex_array_panic",
    "scan_export_thunks",
    "scan_xchg_ebx_deref",
    "scan_unresolved_imports",
    "committed_drift",
    "verify_layout",
    "verify_cohab_coverage",
    "verify_arm64_unwind",
    "scan_i386_sse_abi",
    "lint_wide_strings",
    "scan_bind_all_matches",
    "scan_captures",
    "scan_ir_mismatch",
];
pub const STAGING: &[&str] = &[
    "boot/stripped",
    "boot/ramdisk",
    "boot/ramdisk-overlay",
    "livecd",
    "bootcd",
    "CMakeFiles",
    "host-tools",
];
pub const SUFFIXES: &[&str] = &[".sys", ".dll", ".exe", ".drv"];
pub const BASIS: &str = "build tree (the livecd DAG re-rolls every shipped binary each build)";
type Cached = Arc<Mutex<()>>;
#[derive(Default, Debug, Clone)]
pub struct CheckOutput {
    pub stdout: String,
    pub stderr: String,
    pub returncode: i32,
}
impl CheckOutput {
    pub fn new(stdout: String, returncode: i32) -> Self {
        Self {
            stdout,
            returncode,
            stderr: String::new(),
        }
    }
}
pub struct BinaryContext {
    pub root: PathBuf,
    pub build_dir: PathBuf,
    pub arch: String,
    pub inputs: Vec<PathBuf>,
    pub target: String,
    pub packages: Vec<String>,
    pub include_declarations: bool,
    pub crates: Vec<String>,
    commands: Mutex<HashMap<String, Cached>>,
    digests: Mutex<HashMap<PathBuf, (Stamp, String)>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Stamp {
    len: u64,
    modified: std::time::SystemTime,
    inode: u64,
    device: u64,
    ctime: (i64, i64),
}
fn stamp(p: &Path) -> Result<Stamp, String> {
    use std::os::unix::fs::MetadataExt;
    let m = fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(Stamp {
        len: m.len(),
        modified: m.modified().map_err(|e| e.to_string())?,
        inode: m.ino(),
        device: m.dev(),
        ctime: (m.ctime(), m.ctime_nsec()),
    })
}
fn hex(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}
pub fn executable(name: &str) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let paths = if name.contains('/') {
        vec![PathBuf::from(name)]
    } else {
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|d| d.join(name))
            .collect()
    };
    paths
        .into_iter()
        .find(|p| p.is_file() && fs::metadata(p).is_ok_and(|m| m.permissions().mode() & 0o111 != 0))
}
impl BinaryContext {
    pub fn new(root: PathBuf, build_dir: PathBuf, arch: String) -> Self {
        Self {
            root,
            build_dir,
            arch,
            inputs: Vec::new(),
            target: "x86_64-pc-windows-gnu".into(),
            packages: Vec::new(),
            include_declarations: false,
            crates: Vec::new(),
            commands: Mutex::new(HashMap::new()),
            digests: Mutex::new(HashMap::new()),
        }
    }
    pub fn objdump(&self) -> String {
        let p = self.root.join(".build-tools/llvm-mingw/bin/llvm-objdump");
        if p.is_file() {
            p.to_string_lossy().into()
        } else {
            "llvm-objdump".into()
        }
    }
    fn digest(&self, p: &Path) -> Result<(PathBuf, Stamp, String), String> {
        let resolved = fs::canonicalize(p).map_err(|e| format!("{}: {e}", p.display()))?;
        let initial = stamp(&resolved)?;
        if let Some((s, d)) = self.digests.lock().unwrap().get(&resolved) {
            if *s == initial {
                return Ok((resolved, initial, d.clone()));
            }
        }
        let mut hash = Sha256::new();
        let mut f = File::open(&resolved).map_err(|e| e.to_string())?;
        let mut buf = [0; 65536];
        loop {
            let n = f.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hash.update(&buf[..n]);
        }
        if stamp(&resolved)? != initial {
            return Err(format!("input changed while hashing: {}", p.display()));
        }
        let d = format!("{:x}", hash.finalize());
        self.digests
            .lock()
            .unwrap()
            .insert(resolved.clone(), (initial.clone(), d.clone()));
        Ok((resolved, initial, d))
    }
    pub fn command(&self, program: &str, args: &[String]) -> Result<String, String> {
        let tool =
            executable(program).ok_or_else(|| format!("cannot resolve executable: {program}"))?;
        let mut inputs = vec![
            tool.clone(),
            std::env::current_exe().map_err(|e| e.to_string())?,
        ];
        for arg in args {
            let p = PathBuf::from(arg);
            let p = if p.is_absolute() {
                p
            } else {
                self.root.join(p)
            };
            if p.is_file() {
                inputs.push(p)
            }
        }
        let mut identities = Vec::new();
        let mut snapshots = Vec::new();
        for p in &inputs {
            let (resolved, s, d) = self.digest(p)?;
            identities.push(json!([p, resolved, d]));
            snapshots.push((p, resolved, s));
        }
        let env: std::collections::BTreeMap<_, _> = std::env::vars()
            .filter(|(k, _)| {
                k.starts_with("LC_")
                    || k.starts_with("LD_")
                    || k.starts_with("DYLD_")
                    || k.starts_with("LLVM_")
                    || ["LANG", "LANGUAGE", "PATH", "SYSTEMROOT", "TZ"].contains(&k.as_str())
            })
            .collect();
        let key = hex(json!([
            "binary-extraction-v1",
            self.root,
            program,
            args,
            identities,
            env
        ])
        .to_string());
        let cell = self
            .commands
            .lock()
            .unwrap()
            .entry(key.clone())
            .or_default()
            .clone();
        let verify = || -> Result<(), String> {
            for (requested, resolved, s) in &snapshots {
                if fs::canonicalize(requested).map_err(|e| e.to_string())? != *resolved
                    || stamp(resolved)? != *s
                {
                    return Err(format!(
                        "extraction input changed: {}; retry scan",
                        requested.display()
                    ));
                }
            }
            Ok(())
        };
        let _command_lock = cell.lock().map_err(|_| "binary command lock poisoned")?;
        let value = (|| {
            let cache = self.root.join(".cache/binary-facts-rust");
            fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
            let enabled = std::env::var("VIBEOS_BINARY_CACHE").as_deref() != Ok("0");
            let lock =
                File::create(cache.join(format!("{key}.lock"))).map_err(|e| e.to_string())?;
            lock.lock().map_err(|e| e.to_string())?;
            verify()?;
            let record = cache.join(format!("{key}.json.zst"));
            if enabled && record.exists() {
                let v: Value = serde_json::from_slice(
                    &zstd::stream::decode_all(
                        fs::read(&record).map_err(|e| e.to_string())?.as_slice(),
                    )
                    .map_err(|e| {
                        format!(
                            "corrupt binary facts cache {}: {e}; remove record and retry",
                            record.display()
                        )
                    })?,
                )
                .map_err(|e| {
                    format!(
                        "corrupt binary facts cache {}: {e}; remove record and retry",
                        record.display()
                    )
                })?;
                let out = v["stdout"]
                    .as_str()
                    .ok_or("corrupt binary facts cache stdout")?;
                if v["key"] != key || v["sha256"] != hex(out) {
                    return Err(format!(
                        "corrupt binary facts cache {}; remove record and retry",
                        record.display()
                    ));
                }
                verify()?;
                return Ok(out.into());
            }
            let out = checked_command(&self.root, &tool, args)?;
            verify()?;
            if enabled {
                let tmp = cache.join(format!(".{key}.{}.tmp", std::process::id()));
                let payload = json!({"key":key,"stdout":out,"sha256":hex(&out)}).to_string();
                let result = (|| -> Result<(), String> {
                    let mut f = File::create(&tmp).map_err(|e| e.to_string())?;
                    f.write_all(
                        &zstd::stream::encode_all(payload.as_bytes(), 1)
                            .map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?;
                    f.sync_all().map_err(|e| e.to_string())?;
                    verify()?;
                    fs::rename(&tmp, &record).map_err(|e| e.to_string())
                })();
                let _ = fs::remove_file(tmp);
                result?;
            }
            Ok(out)
        })();
        verify()?;
        value
    }
    pub fn binaries(&self, suffixes: &[&str], staging: &[&str]) -> Result<Vec<PathBuf>, String> {
        let mut paths = Vec::new();
        if !self.build_dir.is_dir() {
            return Err(format!("no such build dir: {}", self.build_dir.display()));
        }
        fn walk(
            dir: &Path,
            root: &Path,
            suffixes: &[&str],
            staging: &[&str],
            out: &mut Vec<PathBuf>,
        ) -> Result<(), String> {
            for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
                let e = entry.map_err(|e| e.to_string())?;
                let p = e.path();
                let rel = p
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                let ty = e.file_type().map_err(|e| e.to_string())?;
                if ty.is_dir() {
                    if !staging.iter().any(|s| format!("/{rel}/").contains(s)) {
                        walk(&p, root, suffixes, staging, out)?
                    }
                } else if !p.is_dir()
                    && rel != "boot/freeldr/freeldr/freeldr.sys"
                    && suffixes.iter().any(|s| {
                        p.extension().is_some_and(|e| {
                            format!(".{}", e.to_string_lossy().to_lowercase()) == *s
                        })
                    })
                {
                    out.push(p)
                }
            }
            Ok(())
        }
        walk(
            &self.build_dir,
            &self.build_dir,
            suffixes,
            staging,
            &mut paths,
        )?;
        paths.sort();
        Ok(paths)
    }
}
fn checked_command(root: &Path, tool: &Path, args: &[String]) -> Result<String, String> {
    let mut child = Command::new(tool)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {e}", tool.display()))?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let errors = std::thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).map(|_| b)
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Ok(s),
            Ok(None) => {
                if start.elapsed() > Duration::from_secs(300) {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err("extraction timed out after 300 seconds".to_string());
                }
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(e.to_string());
            }
        }
    };
    let out = reader
        .join()
        .map_err(|_| "stdout reader panicked")?
        .map_err(|e| e.to_string())?;
    let err = errors
        .join()
        .map_err(|_| "stderr reader panicked")?
        .map_err(|e| e.to_string())?;
    let status = status?;
    if !status.success() {
        return Err(format!(
            "{} {:?} exited {status}: {}",
            tool.display(),
            args,
            String::from_utf8_lossy(&err)
        ));
    }
    Ok(String::from_utf8_lossy(&out).into())
}
pub fn run(ctx: &BinaryContext, names: &[String]) -> Result<Value, String> {
    for n in names {
        if !RULES.contains(&n.as_str()) {
            return Err(format!("unknown binary rule: {n}"));
        }
    }
    let mut result = serde_json::Map::new();
    for name in names {
        let started = Instant::now();
        let output = match name.as_str() {
            "freeldr_size" => freeldr::run(ctx),
            "scan_ud2" => ud2::run(ctx),
            "sacred_primitives" => sacred::run(ctx),
            "rust_ub_deny" => warnings::run(ctx),
            "scan_self_recursive" => simple::self_recursive(ctx),
            "scan_export_thunks" => simple::export_thunks(ctx),
            "scan_xchg_ebx_deref" => simple::xchg_ebx_deref(ctx),
            "scan_unresolved_imports" => imports::run(ctx),
            "scan_flex_array_panic" => flex::run(ctx),
            "committed_drift" => drift::run(ctx),
            "verify_layout" => drift::layout(ctx),
            "verify_cohab_coverage" => cohab::run(ctx),
            "verify_arm64_unwind" => unwind::run(ctx),
            "lint_wide_strings" => wide_strings::run(ctx),
            "scan_i386_sse_abi" => i386_abi::run(ctx),
            "scan_bind_all_matches" => bindall::run(ctx),
            "scan_captures" => ir::captures(ctx, &ctx.inputs),
            "scan_ir_mismatch" => ir::mismatch(ctx, &ctx.inputs, ctx.include_declarations),
            _ => unreachable!(),
        }
        .unwrap_or_else(|e| CheckOutput {
            stdout: String::new(),
            stderr: format!("FATAL: {name}: {e}\n"),
            returncode: 2,
        });
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!("TIMING\t{name}\t{:.3}", started.elapsed().as_secs_f64());
        }
        result.insert(
            name.clone(),
            json!({"stdout":output.stdout,"stderr":output.stderr,"returncode":output.returncode}),
        );
    }
    Ok(Value::Object(result))
}

pub fn explain(ctx: &BinaryContext, substring: &str) -> Result<CheckOutput, String> {
    flex::explain(ctx, substring)
}

#[cfg(test)]
mod tests {
    use super::*;
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    fn fixture() -> (PathBuf, BinaryContext, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "vibeos-binary-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("build-amd64")).unwrap();
        let ctx = BinaryContext::new(root.clone(), root.join("build-amd64"), "amd64".into());
        let tool = root.join("objdump");
        use std::os::unix::fs::PermissionsExt;
        fs::write(&tool, "#!/bin/sh\nprintf x >> \"$1.calls\"\ncat \"$1\"\n").unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
        (root, ctx, tool)
    }
    #[test]
    fn content_cache_revalidates_bytes_and_tools() {
        let (root, ctx, tool) = fixture();
        let binary = root.join("fixture.exe");
        fs::write(&binary, "bad").unwrap();
        let args = vec![binary.to_string_lossy().into_owned()];
        assert_eq!(ctx.command(tool.to_str().unwrap(), &args).unwrap(), "bad");
        let fresh = BinaryContext::new(ctx.root.clone(), ctx.build_dir.clone(), ctx.arch.clone());
        assert_eq!(fresh.command(tool.to_str().unwrap(), &args).unwrap(), "bad");
        assert_eq!(
            fs::read_to_string(binary.with_extension("exe.calls")).unwrap(),
            "x"
        );
        let old = fs::metadata(&binary).unwrap().modified().unwrap();
        fs::write(&binary, "nop").unwrap();
        File::options()
            .write(true)
            .open(&binary)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(old))
            .unwrap();
        assert_eq!(fresh.command(tool.to_str().unwrap(), &args).unwrap(), "nop");
        assert_eq!(
            fs::read_to_string(binary.with_extension("exe.calls")).unwrap(),
            "xx"
        );
        fs::write(&tool, "#!/bin/sh\nprintf changed\n").unwrap();
        assert_eq!(
            fresh.command(tool.to_str().unwrap(), &args).unwrap(),
            "changed"
        );
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn corrupt_cache_and_failed_extraction_fail_loud() {
        let (root, ctx, tool) = fixture();
        let binary = root.join("fixture.exe");
        fs::write(&binary, "fixture").unwrap();
        let args = vec![binary.to_string_lossy().into_owned()];
        ctx.command(tool.to_str().unwrap(), &args).unwrap();
        let record = fs::read_dir(root.join(".cache/binary-facts-rust"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.to_string_lossy().ends_with(".json.zst"))
            .unwrap();
        fs::write(record, "{}").unwrap();
        let fresh = BinaryContext::new(ctx.root.clone(), ctx.build_dir.clone(), ctx.arch.clone());
        assert!(fresh
            .command(tool.to_str().unwrap(), &args)
            .unwrap_err()
            .contains("corrupt"));
        fs::write(&tool, "#!/bin/sh\necho invalid PE >&2\nexit 7\n").unwrap();
        assert!(fresh
            .command(tool.to_str().unwrap(), &args)
            .unwrap_err()
            .contains("invalid PE"));
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn corpus_prunes_before_descent_and_keeps_each_scanners_policy() {
        let (root, ctx, _) = fixture();
        for rel in [
            "real.exe",
            "boot/stripped/copy.exe",
            "boot/staging/copy.exe",
            "livecd/copy.exe",
            "xCMakeFiles/real.exe",
            "boot/freeldr/freeldr/freeldr.sys",
            "boot/freeldr/freeldr/freeldr_pe.exe",
        ] {
            let p = ctx.build_dir.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, []).unwrap();
        }
        let default = ctx.binaries(SUFFIXES, STAGING).unwrap();
        assert_eq!(default.len(), 3);
        assert!(default.iter().any(|p| p.ends_with("boot/staging/copy.exe")));
        let _ = fs::remove_dir_all(root);
    }
}
